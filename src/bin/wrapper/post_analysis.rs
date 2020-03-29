use rustc::hir::map::Map;
use rustc::ty::{self, TyCtxt, TypeckTables};
use rustc_ast::ast;
use rustc_attr::{Stability, Stable};
use rustc_hir as hir;
use rustc_hir::def::{CtorKind, CtorOf, DefKind, Res};
use rustc_hir::def_id::{DefId, CRATE_DEF_INDEX, LOCAL_CRATE};
use rustc_hir::intravisit::{self, NestedVisitorMap};
use rustc_hir::pat_util::EnumerateAndAdjustIterator;
use rustc_session::config::EntryFnType;
use rustc_span::symbol::{sym, Ident};
use rustc_span::Span;

use std::mem;

use super::{context::StabilityContext, Wrapper};

struct Visitor<'a, 'scx, 'tcx> {
    stab_ctx: &'a mut StabilityContext<'scx>,
    tcx: TyCtxt<'tcx>,
    tables: &'a TypeckTables<'tcx>,
    empty_tables: &'a TypeckTables<'tcx>,
    visiting_adt_def: bool,
}

impl<'a, 'scx, 'tcx> Visitor<'a, 'scx, 'tcx> {
    pub fn new(
        stab_ctx: &'a mut StabilityContext<'scx>,
        tcx: TyCtxt<'tcx>,
        empty_tables: &'a TypeckTables<'tcx>,
    ) -> Self {
        Visitor { stab_ctx, tcx, tables: empty_tables, empty_tables, visiting_adt_def: false }
    }

    fn process_stability(&mut self, def_id: DefId, span: Span) {
        if def_id.is_local() {
            return;
        }

        if let Some(stab @ Stability { level: Stable { .. }, .. }) = self.tcx.lookup_stability(def_id) {
            self.stab_ctx.record_lib_feature(*stab, span.source_callsite());
        }
    }

    fn process_struct(&mut self, ty_kind: &ty::TyKind, res: Res, span: Span) {
        if let ty::Adt(def, _) = ty_kind {
            let variant = def.variant_of_res(res);
            if variant.fields.is_empty() {
                self.stab_ctx.record_lang_feature(sym::braced_empty_structs, span);
            }
            if let CtorKind::Fn = variant.ctor_kind {
                self.stab_ctx.record_lang_feature(sym::relaxed_adts, span);
            }
        }
    }

    fn process_fields(&mut self, ty_kind: &ty::TyKind, fields: &[(Ident, Span)]) {
        match ty_kind {
            // NOTE: Stability attributes in struct enum variants are not checked by rustc.
            // See FnCtx::check_expr_struct_fields in librustc_typeck.
            ty::Adt(def, _) if !def.is_enum() => {
                let variant = def.non_enum_variant();
                for (ident, span) in fields {
                    if let Some(ty_field) =
                        self.tcx.find_field_index(*ident, variant).map(|index| &variant.fields[index])
                    {
                        self.process_stability(ty_field.did, *span);
                    }
                }
            },
            _ => {},
        }
    }

    fn process_res(&mut self, res: Res, span: Span) {
        match res {
            Res::PrimTy(hir::PrimTy::Int(ast::IntTy::I128) | hir::PrimTy::Uint(ast::UintTy::U128)) => {
                self.stab_ctx.record_lang_feature(sym::i128_type, span);
            },
            Res::SelfTy(..) if self.visiting_adt_def => {
                self.stab_ctx.record_lang_feature(sym::self_in_typedefs, span);
            },
            _ => {},
        }

        if let Some(def_id) = res.opt_def_id() {
            self.process_stability(def_id, span);
        }
    }

    fn check_alias_enum_variants(&mut self, qpath: &hir::QPath, hir_id: hir::HirId, span: Span) {
        if let Res::Def(DefKind::Variant | DefKind::Ctor(CtorOf::Variant, _), _) = self.tables.qpath_res(qpath, hir_id)
        {
            if let hir::QPath::TypeRelative(ty, _) = qpath {
                if let hir::TyKind::Path(hir::QPath::Resolved(None, ref path)) = ty.kind {
                    if let Res::Def(DefKind::TyAlias, _) | Res::SelfTy(..) = path.res {
                        self.stab_ctx.record_lang_feature(sym::type_alias_enum_variants, span);
                    }
                }
            }
        }
    }

    fn with_item_tables<F>(&mut self, hir_id: hir::HirId, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let def_id = self.tcx.hir().local_def_id(hir_id);
        let tables =
            if self.tcx.has_typeck_tables(def_id) { self.tcx.typeck_tables_of(def_id) } else { self.empty_tables };

        let old_tables = mem::replace(&mut self.tables, tables);
        f(self);
        self.tables = old_tables;
    }
}

impl<'tcx> intravisit::Visitor<'tcx> for Visitor<'_, '_, 'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::OnlyBodies(self.tcx.hir())
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        match item.kind {
            hir::ItemKind::ExternCrate(_) => {
                if item.span.is_dummy() {
                    return;
                }

                let def_id = self.tcx.hir().local_def_id(item.hir_id);
                let cnum = match self.tcx.extern_mod_stmt_cnum(def_id) {
                    Some(cnum) => cnum,
                    None => return,
                };
                let def_id = DefId { krate: cnum, index: CRATE_DEF_INDEX };
                self.process_stability(def_id, item.span);
            },
            hir::ItemKind::Impl { of_trait: Some(ref t), items, .. } => {
                if let Res::Def(DefKind::Trait, trait_did) = t.path.res {
                    for impl_item_ref in items {
                        let impl_item = self.tcx.hir().impl_item(impl_item_ref.id);
                        let trait_item_def_id = self
                            .tcx
                            .associated_items(trait_did)
                            .filter_by_name_unhygienic(impl_item.ident.name)
                            .next()
                            .map(|item| item.def_id);
                        if let Some(def_id) = trait_item_def_id {
                            self.process_stability(def_id, impl_item.span);
                        }
                    }
                }
            },
            hir::ItemKind::Enum(..) | hir::ItemKind::Struct(..) | hir::ItemKind::Union(..) => {
                self.visiting_adt_def = true;
            },
            _ => {},
        }

        self.with_item_tables(item.hir_id, |v| {
            intravisit::walk_item(v, item);
        });
        self.visiting_adt_def = false;
    }

    fn visit_impl_item(&mut self, impl_item: &'tcx hir::ImplItem<'tcx>) {
        self.with_item_tables(impl_item.hir_id, |v| {
            intravisit::walk_impl_item(v, impl_item);
        })
    }

    fn visit_trait_item(&mut self, trait_item: &'tcx hir::TraitItem<'tcx>) {
        self.with_item_tables(trait_item.hir_id, |v| {
            intravisit::walk_trait_item(v, trait_item);
        })
    }

    fn visit_pat(&mut self, pat: &'tcx hir::Pat<'tcx>) {
        match &pat.kind {
            hir::PatKind::Struct(qpath, fields, _) => {
                self.check_alias_enum_variants(qpath, pat.hir_id, pat.span);

                let res = self.tables.qpath_res(qpath, pat.hir_id);
                if let Res::Def(DefKind::AssocTy, _) | Res::SelfTy(..) = res {
                    self.stab_ctx.record_lang_feature(sym::more_struct_aliases, pat.span);
                }

                if let Some(pat_ty) = self.tables.pat_ty_opt(pat) {
                    self.process_struct(&pat_ty.kind, res, pat.span);

                    let fields = fields.iter().map(|f| (f.ident, f.span)).collect::<Vec<_>>();
                    self.process_fields(&pat_ty.kind, &fields);
                }
            },
            hir::PatKind::TupleStruct(qpath, subpats, ddpos) => {
                self.check_alias_enum_variants(qpath, pat.hir_id, pat.span);

                if let Some(pat_ty) = self.tables.pat_ty_opt(pat) {
                    match pat_ty.kind {
                        ty::Adt(def, _) if !def.is_enum() => {
                            let variant = def.non_enum_variant();
                            for (i, subpat) in subpats.iter().enumerate_and_adjust(variant.fields.len(), *ddpos) {
                                self.process_stability(variant.fields[i].did, subpat.span);
                            }
                        },
                        _ => {},
                    }
                }
            },
            hir::PatKind::Path(qpath) => {
                self.check_alias_enum_variants(qpath, pat.hir_id, pat.span);
            },
            _ => {},
        }

        intravisit::walk_pat(self, pat);
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        match expr.kind {
            hir::ExprKind::MethodCall(..) => {
                if let Some(def_id) = self.tables.type_dependent_def_id(expr.hir_id) {
                    self.process_stability(def_id, expr.span);
                }
            },
            hir::ExprKind::Field(subexpr, ident) => {
                if let Some(expr_ty) = self.tables.expr_ty_adjusted_opt(subexpr) {
                    self.process_fields(&expr_ty.kind, &[(ident, subexpr.span)]);
                }
            },
            hir::ExprKind::Struct(qpath, fields, _) => {
                self.check_alias_enum_variants(qpath, expr.hir_id, expr.span);

                let res = self.tables.qpath_res(qpath, expr.hir_id);
                if let Res::Def(DefKind::AssocTy, _) | Res::SelfTy(..) = res {
                    self.stab_ctx.record_lang_feature(sym::more_struct_aliases, expr.span);
                }

                if let Some(expr_ty) = self.tables.expr_ty_adjusted_opt(expr) {
                    self.process_struct(&expr_ty.kind, res, expr.span);

                    let idents = fields.iter().map(|f| (f.ident, f.span)).collect::<Vec<_>>();
                    self.process_fields(&expr_ty.kind, &idents);
                }
            },
            hir::ExprKind::Path(ref qpath) => {
                self.check_alias_enum_variants(qpath, expr.hir_id, expr.span);
            },
            _ => {},
        }

        intravisit::walk_expr(self, expr);
    }

    fn visit_path(&mut self, path: &'tcx hir::Path<'tcx>, _id: hir::HirId) {
        self.process_res(path.res, path.span);
        intravisit::walk_path(self, path);
    }

    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: hir::HirId, span: Span) {
        let res = self.tables.qpath_res(qpath, id);

        match qpath {
            hir::QPath::Resolved(..) => {
                if let hir::def::Res::SelfCtor(_) = res {
                    self.stab_ctx.record_lang_feature(sym::self_struct_ctor, span);
                }
                // NOTE: Lib stability will be checked when visiting its inner path
            },
            hir::QPath::TypeRelative(..) => {
                self.process_res(res, span);
            },
        }

        intravisit::walk_qpath(self, qpath, id, span);
    }
}

fn check_termination_trait(stab_ctx: &mut StabilityContext, tcx: TyCtxt) {
    if let Some((main_did, EntryFnType::Main)) = tcx.entry_fn(LOCAL_CRATE) {
        let hir_id = tcx.hir().as_local_hir_id(main_did).unwrap();
        if let Some(fn_sig) = tcx.hir().fn_sig_by_hir_id(hir_id) {
            let output = &fn_sig.decl.output;
            match output {
                hir::FnRetTy::DefaultReturn(_) => {},
                hir::FnRetTy::Return(hir::Ty { kind: hir::TyKind::Tup(elems), .. }) if elems.is_empty() => {},
                _ => {
                    stab_ctx.record_lang_feature(sym::termination_trait, output.span());
                },
            }
        }
    }
}

pub fn process_crate(wrapper: &mut Wrapper, tcx: TyCtxt) {
    use intravisit::Visitor as _;

    let mut stab_ctx = StabilityContext::new(tcx.sess);
    check_termination_trait(&mut stab_ctx, tcx);

    let empty_tables = TypeckTables::empty(None);
    let mut visitor = Visitor::new(&mut stab_ctx, tcx, &empty_tables);
    tcx.hir().krate().visit_all_item_likes(&mut visitor.as_deep_visitor());

    stab_ctx.dump(wrapper);
}
