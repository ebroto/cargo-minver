use rustc_ast::ast::{self, Pat, RangeEnd, RangeSyntax};
use rustc_ast::ptr::P;
use rustc_ast::visit::{self, FnKind};
use rustc_attr::{Stability, StabilityLevel};
use rustc_resolve::{ParentScope, Resolver};
use rustc_session::Session;
use rustc_span::hygiene::{ExpnData, ExpnKind};
use rustc_span::source_map::{SourceMap, Spanned};
use rustc_span::symbol::{kw, sym, Symbol};
use rustc_span::Span;

use std::collections::HashMap;

use super::{context::Context, Wrapper};

struct Visitor<'a, 'res> {
    ctx: &'a mut Context,
    resolver: &'a mut Resolver<'res>,
    source_map: &'a SourceMap,
    imported_macros: HashMap<Symbol, Option<Stability>>,
}

impl<'a, 'res> Visitor<'a, 'res> {
    fn new(ctx: &'a mut Context, resolver: &'a mut Resolver<'res>, source_map: &'a SourceMap) -> Self {
        Self { ctx, resolver, source_map, imported_macros: Default::default() }
    }

    fn check_non_exhaustive(&mut self, item: &ast::Item) {
        if item.attrs.iter().any(|a| a.has_name(sym::non_exhaustive)) {
            self.ctx.record_lang_feature(sym::non_exhaustive, item.span);
        }
    }

    fn check_repr(&mut self, item: &ast::Item) {
        let metas = item.attrs.iter().filter(|a| a.has_name(sym::repr)).filter_map(|a| a.meta_item_list()).flatten();

        for meta in metas {
            match meta.name_or_empty() {
                sym::transparent => match &item.kind {
                    // NOTE: repr transparent for unions is still unstable
                    ast::ItemKind::Struct(..) => {
                        self.ctx.record_lang_feature(sym::repr_transparent, item.span);
                    },
                    ast::ItemKind::Enum(..) => {
                        self.ctx.record_lang_feature(sym::transparent_enums, item.span);
                    },
                    _ => {},
                },
                sym::align => match &item.kind {
                    ast::ItemKind::Struct(..) | ast::ItemKind::Union(..) => {
                        self.ctx.record_lang_feature(sym::repr_align, item.span);
                    },
                    ast::ItemKind::Enum(..) => {
                        self.ctx.record_lang_feature(sym::repr_align_enum, item.span);
                    },
                    _ => {},
                },
                _ => {},
            }

            if let Some((sym::packed, _)) = meta.name_value_literal() {
                if let ast::ItemKind::Struct(..) = item.kind {
                    self.ctx.record_lang_feature(sym::repr_packed, item.span);
                }
            }
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
                self.ctx.record_lib_feature(*stab, span.source_callsite());
            }
        }
    }
}

impl<'ast> visit::Visitor<'ast> for Visitor<'_, '_> {
    fn visit_use_tree(&mut self, use_tree: &ast::UseTree, node_id: ast::NodeId, _nested: bool) {
        if let ast::UseTreeKind::Simple(Some(ident), ..) = use_tree.kind {
            if ident.name == kw::Underscore {
                self.ctx.record_lang_feature(sym::underscore_imports, ident.span);
            }
        }

        visit::walk_use_tree(self, use_tree, node_id);
    }

    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        if let feature @ (sym::target_feature | sym::deprecated | sym::panic_handler) = attr.name_or_empty() {
            self.ctx.record_lang_feature(feature, attr.span);
        }

        visit::walk_attribute(self, attr);
    }

    fn visit_mac(&mut self, _mac: &ast::MacCall) {
        // Do nothing.
    }

    fn visit_item(&mut self, item: &ast::Item) {
        self.check_macro_use(item.span);

        match &item.kind {
            ast::ItemKind::ExternCrate(_) => {
                if item.ident.name == kw::Underscore {
                    self.ctx.record_lang_feature(sym::underscore_imports, item.ident.span);
                }
            },
            ast::ItemKind::Struct(variant_data, _) => {
                if let ast::VariantData::Struct(fields, _) = variant_data {
                    if fields.is_empty() {
                        self.ctx.record_lang_feature(sym::braced_empty_structs, item.span);
                    }
                }
                self.check_non_exhaustive(item);
                self.check_repr(item);
            },
            ast::ItemKind::Enum(..) => {
                self.check_non_exhaustive(item);
                self.check_repr(item);
            },
            ast::ItemKind::Union(..) => {
                self.check_repr(item);
            },
            ast::ItemKind::Static(..) => {
                if item.attrs.iter().any(|a| a.has_name(sym::used)) {
                    self.ctx.record_lang_feature(sym::used, item.span);
                }
                // NOTE: Athough declared as a lang feature, the global_allocator attribute
                // macro is defined in libcore and will be detected as a library feature.
            },
            _ => {},
        }

        visit::walk_item(self, item);
    }

    fn visit_variant(&mut self, variant: &ast::Variant) {
        if let ast::VariantData::Struct(fields, _) = &variant.data {
            if fields.is_empty() {
                self.ctx.record_lang_feature(sym::braced_empty_structs, variant.span);
            }
        }

        visit::walk_variant(self, variant);
    }

    fn visit_fn(&mut self, fn_kind: FnKind, span: Span, _node_id: ast::NodeId) {
        if let Some(header) = fn_kind.header() {
            if header.asyncness.is_async() {
                self.ctx.record_lang_feature(sym::async_await, span);
            }
        }

        visit::walk_fn(self, fn_kind, span);
    }

    fn visit_param(&mut self, param: &ast::Param) {
        if !param.attrs.is_empty() {
            self.ctx.record_lang_feature(sym::param_attrs, param.span);
        }

        visit::walk_param(self, param);
    }

    fn visit_generic_param(&mut self, param: &ast::GenericParam) {
        if !param.attrs.is_empty() {
            self.ctx.record_lang_feature(sym::generic_param_attrs, param.attrs[0].span);
        }

        visit::walk_generic_param(self, param);
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        self.check_macro_use(ty.span);
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
                self.ctx.record_lang_feature(sym::inclusive_range_syntax, expr.span);
            },
            ast::ExprKind::Break(_, Some(_)) => {
                self.ctx.record_lang_feature(sym::loop_break_value, expr.span);
            },
            ast::ExprKind::Async(..) | ast::ExprKind::Await(_) => {
                self.ctx.record_lang_feature(sym::async_await, expr.span);
            },
            ast::ExprKind::Lit(lit) => {
                use ast::LitIntType::{Signed, Unsigned};
                if let ast::LitKind::Int(_, Signed(ast::IntTy::I128) | Unsigned(ast::UintTy::U128)) = lit.kind {
                    self.ctx.record_lang_feature(sym::i128_type, expr.span);
                }
            },
            ast::ExprKind::Let(pat, _) => {
                if let ast::PatKind::Or(_) = pat.kind {
                    self.ctx.record_lang_feature(sym::if_while_or_patterns, pat.span);
                }
            },
            ast::ExprKind::Struct(_, fields, _) => {
                if fields.iter().any(|f| !f.attrs.is_empty()) {
                    self.ctx.record_lang_feature(sym::struct_field_attributes, expr.span)
                }
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
                self.ctx.record_lang_feature(sym::dotdoteq_in_patterns, pat.span);
            },
            ast::PatKind::Struct(_, fields, _) => {
                if fields.iter().any(|f| !f.attrs.is_empty()) {
                    self.ctx.record_lang_feature(sym::struct_field_attributes, pat.span)
                }
            },
            ast::PatKind::Tuple(ps) if has_rest(ps) => {
                self.ctx.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
            },
            ast::PatKind::TupleStruct(_, ps) if ps.len() > 1 && has_rest(ps) => {
                self.ctx.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
            },
            ast::PatKind::Paren(..) => {
                self.ctx.record_lang_feature(sym::pattern_parentheses, pat.span);
            },
            _ => {},
        }

        visit::walk_pat(self, pat);
    }

    fn visit_path_segment(&mut self, span: Span, segment: &ast::PathSegment) {
        if segment.ident.name == kw::Crate {
            self.ctx.record_lang_feature(sym::crate_in_paths, segment.ident.span);
        }

        visit::walk_path_segment(self, span, segment);
    }
}

pub fn process_crate(wrapper: &mut Wrapper, session: &Session, krate: &ast::Crate, resolver: &mut Resolver) {
    let mut ctx = Context::default();
    let mut visitor = Visitor::new(&mut ctx, resolver, session.source_map());
    visit::walk_crate(&mut visitor, &krate);

    let raw_ident_spans = session.parse_sess.raw_identifier_spans.borrow();
    for span in raw_ident_spans.iter() {
        ctx.record_lang_feature(sym::raw_identifiers, *span);
    }

    ctx.dump(wrapper, session);
}
