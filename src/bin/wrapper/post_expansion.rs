use rustc_ast::ast::{self, Pat, RangeEnd, RangeSyntax};
use rustc_ast::ptr::P;
use rustc_ast::visit::{self, FnKind};
use rustc_attr::{self as attr, Stability, StabilityLevel};
use rustc_resolve::{ParentScope, Resolver};
use rustc_session::lint::Level;
use rustc_session::Session;
use rustc_span::hygiene::{ExpnData, ExpnKind};
use rustc_span::source_map::{SourceMap, Spanned};
use rustc_span::symbol::{kw, sym, Symbol};
use rustc_span::Span;

use std::collections::HashMap;

use super::{context::StabCtxt, Wrapper};

struct Visitor<'a, 'scx, 'res> {
    stab_ctx: &'a mut StabCtxt<'scx>,
    resolver: &'a mut Resolver<'res>,
    source_map: &'a SourceMap,
    imported_macros: HashMap<Symbol, Option<Stability>>,
    // NOTE: `advanced_slice_patterns` was renamed to `slice_patterns`, so we need a new symbol to track the former feature.
    min_slice_patterns: Symbol,
}

impl<'a, 'scx, 'res> Visitor<'a, 'scx, 'res> {
    fn new(stab_ctx: &'a mut StabCtxt<'scx>, resolver: &'a mut Resolver<'res>, source_map: &'a SourceMap) -> Self {
        Self {
            stab_ctx,
            resolver,
            source_map,
            imported_macros: Default::default(),
            min_slice_patterns: Symbol::intern("min_slice_patterns"),
        }
    }

    fn check_non_exhaustive(&mut self, item: &ast::Item) {
        if item.attrs.iter().any(|a| a.has_name(sym::non_exhaustive)) {
            self.stab_ctx.record_lang_feature(sym::non_exhaustive, item.span);
        }
    }

    fn check_repr(&mut self, item: &ast::Item) {
        let metas = item.attrs.iter().filter(|a| a.has_name(sym::repr)).filter_map(|a| a.meta_item_list()).flatten();

        for meta in metas {
            match meta.name_or_empty() {
                sym::transparent => match &item.kind {
                    // NOTE: repr transparent for unions is still unstable
                    ast::ItemKind::Struct(..) => {
                        self.stab_ctx.record_lang_feature(sym::repr_transparent, item.span);
                    },
                    ast::ItemKind::Enum(..) => {
                        self.stab_ctx.record_lang_feature(sym::transparent_enums, item.span);
                    },
                    _ => {},
                },
                sym::align => match &item.kind {
                    ast::ItemKind::Struct(..) | ast::ItemKind::Union(..) => {
                        self.stab_ctx.record_lang_feature(sym::repr_align, item.span);
                    },
                    ast::ItemKind::Enum(..) => {
                        self.stab_ctx.record_lang_feature(sym::repr_align_enum, item.span);
                    },
                    _ => {},
                },
                _ => {},
            }

            if let Some((sym::packed, _)) = meta.name_value_literal() {
                if let ast::ItemKind::Struct(..) = item.kind {
                    self.stab_ctx.record_lang_feature(sym::repr_packed, item.span);
                }
            }
        }
    }

    fn check_tool_lint(&mut self, attr: &ast::Attribute) {
        if Level::from_symbol(attr.name_or_empty()).is_none() {
            return;
        }

        let meta = match attr.meta() {
            Some(m) => m,
            _ => return,
        };

        if meta
            .meta_item_list()
            .unwrap_or_default()
            .iter()
            .filter_map(|mi| mi.meta_item())
            .filter(|mi| mi.is_word() && mi.path.segments.len() > 1)
            .any(|mi| attr::is_known_lint_tool(mi.path.segments[0].ident))
        {
            self.stab_ctx.record_lang_feature(sym::tool_lints, attr.span);
        }
    }

    fn check_variant_data(&mut self, variant_data: &ast::VariantData, span: Span) {
        match variant_data {
            ast::VariantData::Struct(fields, _) if fields.is_empty() => {
                self.stab_ctx.record_lang_feature(sym::braced_empty_structs, span);
            },
            ast::VariantData::Tuple(fields, _) if fields.is_empty() => {
                self.stab_ctx.record_lang_feature(sym::relaxed_adts, span);
            },
            _ => {},
        }
    }

    fn check_static_in_const(&mut self, ty: &ast::Ty) {
        if let ast::TyKind::Rptr(None, ..) = ty.kind {
            self.stab_ctx.record_lang_feature(sym::static_in_const, ty.span);
        }
    }

    fn check_abi_sysv64(&mut self, abi: &ast::StrLit) {
        if abi.symbol_unescaped.as_str() == "sysv64" {
            self.stab_ctx.record_lang_feature(sym::abi_sysv64, abi.span);
        }
    }

    fn check_const_indexing(&mut self, size_expr: &ast::Expr) {
        if let ast::ExprKind::Index(..) = size_expr.kind {
            self.stab_ctx.record_lang_feature(sym::const_indexing, size_expr.span);
        }
    }

    fn check_macro_use(&mut self, span: Span) {
        if !span.from_expansion() {
            return;
        }

        if let Some(ExpnData { kind: ExpnKind::Macro(_, name), def_site, .. }) = span.source_callee() {
            if !self.source_map.is_imported(def_site) {
                return;
            }

            let resolver = &mut self.resolver;
            let maybe_stab = self.imported_macros.entry(name).or_insert_with(|| {
                let path = ast::Path::from_ident(ast::Ident::new(name, def_site));
                let scope = ParentScope::module(resolver.graph_root());
                match resolver.resolve_macro_path(&path, None, &scope, false, false) {
                    Ok((Some(ext), ..)) => match ext.stability {
                        stab @ Some(Stability { level: StabilityLevel::Stable { .. }, .. }) => stab,
                        _ => None,
                    },
                    _ => None,
                }
            });

            if let Some(stab) = maybe_stab {
                self.stab_ctx.record_lib_feature(*stab, span.source_callsite());
            }
        }
    }
}

fn starts_with_digit(s: &str) -> bool {
    s.as_bytes().first().cloned().map_or(false, |b| b >= b'0' && b <= b'9')
}

impl<'ast> visit::Visitor<'ast> for Visitor<'_, '_, '_> {
    fn visit_use_tree(&mut self, use_tree: &ast::UseTree, node_id: ast::NodeId, nested: bool) {
        if nested {
            let record = match use_tree.kind {
                ast::UseTreeKind::Simple(..) if use_tree.prefix.segments.len() != 1 => true,
                ast::UseTreeKind::Nested(..) | ast::UseTreeKind::Glob => true,
                _ => false,
            };
            if record {
                self.stab_ctx.record_lang_feature(sym::use_nested_groups, use_tree.span);
            }
        }

        if let ast::UseTreeKind::Simple(Some(ast::Ident { name: kw::Underscore, span }), ..) = use_tree.kind {
            self.stab_ctx.record_lang_feature(sym::underscore_imports, span);
        }

        visit::walk_use_tree(self, use_tree, node_id);
    }

    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        self.check_tool_lint(attr);

        if let ast::AttrKind::Normal(attr_item) = &attr.kind {
            let path_segments = &attr_item.path.segments;
            if !path_segments.is_empty() {
                match path_segments[0].ident.name {
                    feature
                    @
                    (sym::no_std
                    | sym::target_feature
                    | sym::deprecated
                    | sym::panic_handler
                    | sym::windows_subsystem) => {
                        self.stab_ctx.record_lang_feature(feature, attr.span);
                    },

                    // NOTE: These two seem to be hardcoded until register_tool is stabilized
                    sym::rustfmt | sym::clippy => {
                        self.stab_ctx.record_lang_feature(sym::tool_attributes, attr.span);
                    },

                    _ => {},
                }
            }
        }

        visit::walk_attribute(self, attr);
    }

    fn visit_mac(&mut self, _mac: &ast::MacCall) {
        // Do nothing.
    }

    fn visit_item(&mut self, item: &ast::Item) {
        self.check_macro_use(item.span);

        match &item.kind {
            ast::ItemKind::ExternCrate(original) => {
                if item.ident.name == kw::Underscore {
                    self.stab_ctx.record_lang_feature(sym::underscore_imports, item.ident.span);
                }
                if let Some(kw::SelfLower) = original {
                    self.stab_ctx.record_lang_feature(sym::extern_crate_self, item.span);
                }
            },
            ast::ItemKind::ForeignMod(foreign_mod) => {
                if let Some(abi) = foreign_mod.abi {
                    self.check_abi_sysv64(&abi);
                }
            },
            ast::ItemKind::Struct(variant_data, _) => {
                self.check_non_exhaustive(item);
                self.check_repr(item);
                self.check_variant_data(variant_data, item.span);
            },
            ast::ItemKind::Enum(..) => {
                self.check_non_exhaustive(item);
                self.check_repr(item);
            },
            ast::ItemKind::Union(..) => {
                self.check_repr(item);
            },
            ast::ItemKind::Static(ty, ..) => {
                if item.attrs.iter().any(|a| a.has_name(sym::used)) {
                    self.stab_ctx.record_lang_feature(sym::used, item.span);
                }
                self.check_static_in_const(ty);
                // NOTE: Athough declared as a lang feature, the global_allocator attribute
                // macro is defined in libcore and will be detected as a library feature.
            },
            ast::ItemKind::Const(_, ty, ..) => {
                if item.ident.name == kw::Underscore {
                    self.stab_ctx.record_lang_feature(sym::underscore_const_names, item.ident.span);
                }
                self.check_static_in_const(ty);
            },
            ast::ItemKind::Fn(..) => {
                if item.attrs.iter().any(|a| a.has_name(sym::must_use)) {
                    self.stab_ctx.record_lang_feature(sym::fn_must_use, item.span);
                }
            },
            ast::ItemKind::Impl { items, .. } => {
                for impl_item in items {
                    if let ast::AssocItemKind::Fn(_, fn_sig, ..) = &impl_item.kind {
                        if impl_item.attrs.iter().any(|a| a.has_name(sym::must_use)) {
                            self.stab_ctx.record_lang_feature(sym::fn_must_use, impl_item.span);
                        }
                        if let ast::Const::Yes(span) = fn_sig.header.constness {
                            self.stab_ctx.record_lang_feature(sym::min_const_fn, span);
                        }
                    }
                }
            },
            _ => {},
        }

        visit::walk_item(self, item);
    }

    fn visit_assoc_item(&mut self, item: &ast::AssocItem, ctxt: visit::AssocCtxt) {
        if let ast::AssocItemKind::Const(..) = item.kind {
            self.stab_ctx.record_lang_feature(sym::associated_consts, item.span);
        }

        visit::walk_assoc_item(self, item, ctxt);
    }

    fn visit_variant(&mut self, variant: &ast::Variant) {
        self.check_variant_data(&variant.data, variant.span);
        visit::walk_variant(self, variant);
    }

    fn visit_fn(&mut self, fn_kind: FnKind, span: Span, _node_id: ast::NodeId) {
        if let Some(header) = fn_kind.header() {
            if header.asyncness.is_async() {
                self.stab_ctx.record_lang_feature(sym::async_await, span);
            }
            if let ast::Const::Yes(span) = header.constness {
                self.stab_ctx.record_lang_feature(sym::min_const_fn, span);
            }
            if let ast::Extern::Explicit(abi) = header.ext {
                self.check_abi_sysv64(&abi);
            }
        }

        visit::walk_fn(self, fn_kind, span);
    }

    fn visit_param(&mut self, param: &ast::Param) {
        if !param.attrs.is_empty() {
            self.stab_ctx.record_lang_feature(sym::param_attrs, param.span);
        }
        if let ast::TyKind::ImplTrait(..) = param.ty.kind {
            self.stab_ctx.record_lang_feature(sym::universal_impl_trait, param.span);
        }

        visit::walk_param(self, param);
    }

    fn visit_generic_param(&mut self, param: &ast::GenericParam) {
        if !param.attrs.is_empty() {
            self.stab_ctx.record_lang_feature(sym::generic_param_attrs, param.attrs[0].span);
        }

        visit::walk_generic_param(self, param);
    }

    fn visit_lifetime(&mut self, lifetime: &ast::Lifetime) {
        if lifetime.ident.name == kw::UnderscoreLifetime {
            self.stab_ctx.record_lang_feature(sym::underscore_lifetimes, lifetime.ident.span);
        }

        visit::walk_lifetime(self, lifetime);
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &ast::FnRetTy) {
        if let ast::FnRetTy::Ty(ty) = ret_ty {
            if let ast::TyKind::ImplTrait(..) = ty.kind {
                self.stab_ctx.record_lang_feature(sym::conservative_impl_trait, ty.span);
            }
        }

        visit::walk_fn_ret_ty(self, ret_ty);
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        self.check_macro_use(ty.span);

        match &ty.kind {
            ast::TyKind::TraitObject(_, ast::TraitObjectSyntax::Dyn) => {
                self.stab_ctx.record_lang_feature(sym::dyn_trait, ty.span);
            },
            ast::TyKind::Array(_, size) => {
                self.check_const_indexing(&size.value);
            },
            _ => {},
        }

        visit::walk_ty(self, ty);
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        self.check_macro_use(stmt.span);
        visit::walk_stmt(self, stmt);
    }

    fn visit_local(&mut self, local: &ast::Local) {
        self.check_macro_use(local.span);
        visit::walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        self.check_macro_use(expr.span);

        match &expr.kind {
            ast::ExprKind::Range(_, _, ast::RangeLimits::Closed) => {
                self.stab_ctx.record_lang_feature(sym::inclusive_range_syntax, expr.span);
            },
            ast::ExprKind::Repeat(_, size) => {
                self.check_const_indexing(&size.value);
            },
            ast::ExprKind::Break(_, Some(_)) => {
                self.stab_ctx.record_lang_feature(sym::loop_break_value, expr.span);
            },
            ast::ExprKind::Async(..) | ast::ExprKind::Await(_) => {
                self.stab_ctx.record_lang_feature(sym::async_await, expr.span);
            },
            ast::ExprKind::Lit(lit) => {
                use ast::LitIntType::{Signed, Unsigned};
                if let ast::LitKind::Int(_, Signed(ast::IntTy::I128) | Unsigned(ast::UintTy::U128)) = lit.kind {
                    self.stab_ctx.record_lang_feature(sym::i128_type, expr.span);
                }
            },
            ast::ExprKind::Let(pat, _) => {
                if let ast::PatKind::Or(_) = pat.kind {
                    self.stab_ctx.record_lang_feature(sym::if_while_or_patterns, pat.span);
                }
            },
            ast::ExprKind::Struct(_, fields, _) => {
                if fields.iter().any(|f| starts_with_digit(&f.ident.name.as_str())) {
                    self.stab_ctx.record_lang_feature(sym::relaxed_adts, expr.span);
                }
                for field in fields {
                    if !field.attrs.is_empty() {
                        self.stab_ctx.record_lang_feature(sym::struct_field_attributes, field.span)
                    }
                    if field.is_shorthand {
                        self.stab_ctx.record_lang_feature(sym::field_init_shorthand, field.span);
                    }
                }
            },
            ast::ExprKind::Try(..) => {
                self.stab_ctx.record_lang_feature(sym::question_mark, expr.span);
            },
            _ => {},
        }

        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &ast::Pat) {
        fn has_rest(ps: &[P<Pat>]) -> bool {
            ps.iter().any(|p| p.is_rest())
        }

        self.check_macro_use(pat.span);

        match &pat.kind {
            ast::PatKind::Range(.., Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), .. }) => {
                self.stab_ctx.record_lang_feature(sym::dotdoteq_in_patterns, pat.span);
            },
            ast::PatKind::Struct(_, fields, _) => {
                if fields.iter().any(|f| starts_with_digit(&f.ident.name.as_str())) {
                    self.stab_ctx.record_lang_feature(sym::relaxed_adts, pat.span);
                }
                for field in fields {
                    if !field.attrs.is_empty() {
                        self.stab_ctx.record_lang_feature(sym::struct_field_attributes, field.span)
                    }
                }
            },
            ast::PatKind::Tuple(ps) if has_rest(ps) => {
                self.stab_ctx.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
            },
            ast::PatKind::TupleStruct(_, ps) => {
                if ps.is_empty() {
                    self.stab_ctx.record_lang_feature(sym::relaxed_adts, pat.span);
                } else if ps.len() > 1 && has_rest(ps) {
                    self.stab_ctx.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
                }
            },
            ast::PatKind::Paren(..) => {
                self.stab_ctx.record_lang_feature(sym::pattern_parentheses, pat.span);
            },
            ast::PatKind::Slice(pats) => {
                self.stab_ctx.record_lang_feature(self.min_slice_patterns, pat.span);

                for pat in &*pats {
                    let span = pat.span;
                    let inner_pat = match &pat.kind {
                        ast::PatKind::Ident(.., Some(pat)) => pat,
                        _ => pat,
                    };
                    if inner_pat.is_rest() {
                        self.stab_ctx.record_lang_feature(sym::slice_patterns, span);
                    }
                }
            },
            _ => {},
        }

        visit::walk_pat(self, pat);
    }

    fn visit_path_segment(&mut self, span: Span, segment: &ast::PathSegment) {
        if segment.ident.name == kw::Crate {
            self.stab_ctx.record_lang_feature(sym::crate_in_paths, segment.ident.span);
        }

        visit::walk_path_segment(self, span, segment);
    }
}

pub fn process_crate(wrapper: &mut Wrapper, session: &Session, krate: &ast::Crate, resolver: &mut Resolver) {
    let mut stab_ctx = StabCtxt::new(session);
    let mut visitor = Visitor::new(&mut stab_ctx, resolver, session.source_map());
    visit::walk_crate(&mut visitor, &krate);

    let raw_ident_spans = session.parse_sess.raw_identifier_spans.borrow();
    for span in raw_ident_spans.iter() {
        stab_ctx.record_lang_feature(sym::raw_identifiers, *span);
    }

    stab_ctx.dump(wrapper);
}
