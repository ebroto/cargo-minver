use rustc::hir::map::Map;
use rustc::ty::{self, TyCtxt};
use rustc_attr as attr;
use rustc_attr::Stability;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, CRATE_DEF_INDEX};
use rustc_hir::intravisit::{self, NestedVisitorMap};
use rustc_hir::pat_util::EnumerateAndAdjustIterator;
use rustc_span::hygiene::{ExpnData, ExpnKind};
use rustc_span::source_map::SourceMap;
use rustc_span::symbol::Ident;
use rustc_span::Span;

use std::collections::{HashMap, HashSet};
use std::mem;

use super::{convert_span, convert_stability, Wrapper};

struct Visitor<'a, 'tcx> {
    lib_features: HashMap<Stability, HashSet<Span>>,
    tcx: TyCtxt<'tcx>,
    tables: &'a ty::TypeckTables<'tcx>,
    empty_tables: &'a ty::TypeckTables<'tcx>,
    imported_macros: &'a HashMap<String, Stability>,
}

impl<'a, 'tcx> Visitor<'a, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        empty_tables: &'a ty::TypeckTables<'tcx>,
        imported_macros: &'a HashMap<String, Stability>,
    ) -> Self {
        Visitor { lib_features: HashMap::new(), tcx, tables: empty_tables, empty_tables, imported_macros }
    }

    fn process_stability(&mut self, def_id: DefId, span: Span) {
        if def_id.is_local() {
            return;
        }

        if let Some(stab) = self.tcx.lookup_stability(def_id) {
            if let attr::Stable { .. } = stab.level {
                self.lib_features.entry(*stab).or_default().insert(span.source_callsite());
            }
        }
    }

    fn process_fields(&mut self, ty_kind: &ty::TyKind, fields: &[(Ident, Span)]) {
        match ty_kind {
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

    fn process_macros(&mut self, span: Span) {
        if !span.from_expansion() {
            return;
        }

        if let Some(ExpnData { kind: ExpnKind::Macro(_, name), .. }) = span.source_callee() {
            if let Some(stab) = self.imported_macros.get(&name.to_string()) {
                self.lib_features.entry(*stab).or_default().insert(span.source_callsite());
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

// TODO: do the rest of lib stability checks here.
impl<'a, 'tcx> intravisit::Visitor<'tcx> for Visitor<'a, 'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> NestedVisitorMap<'_, Self::Map> {
        NestedVisitorMap::OnlyBodies(&self.tcx.hir())
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        self.process_macros(item.span);

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

            _ => {},
        }

        self.with_item_tables(item.hir_id, |v| {
            intravisit::walk_item(v, item);
        });
    }

    fn visit_impl_item(&mut self, impl_item: &'tcx hir::ImplItem<'tcx>) {
        self.process_macros(impl_item.span);

        self.with_item_tables(impl_item.hir_id, |v| {
            intravisit::walk_impl_item(v, impl_item);
        })
    }

    fn visit_trait_item(&mut self, trait_item: &'tcx hir::TraitItem<'tcx>) {
        self.process_macros(trait_item.span);

        self.with_item_tables(trait_item.hir_id, |v| {
            intravisit::walk_trait_item(v, trait_item);
        })
    }

    fn visit_pat(&mut self, pat: &'tcx hir::Pat<'tcx>) {
        self.process_macros(pat.span);

        match pat.kind {
            hir::PatKind::Struct(_, fields, _) => {
                if let Some(pat_ty) = self.tables.pat_ty_opt(pat) {
                    let fields = fields.iter().map(|f| (f.ident, f.span)).collect::<Vec<_>>();
                    self.process_fields(&pat_ty.kind, &fields);
                }
            },
            hir::PatKind::TupleStruct(_, subpats, ddpos) => {
                if let Some(pat_ty) = self.tables.pat_ty_opt(pat) {
                    match pat_ty.kind {
                        ty::Adt(def, _) if !def.is_enum() => {
                            let variant = def.non_enum_variant();
                            for (i, subpat) in subpats.iter().enumerate_and_adjust(variant.fields.len(), ddpos) {
                                self.process_stability(variant.fields[i].did, subpat.span);
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        intravisit::walk_pat(self, pat);
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        self.process_macros(expr.span);

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
            hir::ExprKind::Struct(_, fields, _) => {
                if let Some(expr_ty) = self.tables.expr_ty_adjusted_opt(expr) {
                    let idents = fields.iter().map(|f| (f.ident, f.span)).collect::<Vec<_>>();
                    self.process_fields(&expr_ty.kind, &idents);
                }
            },
            _ => {},
        }

        intravisit::walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'tcx hir::Stmt<'tcx>) {
        self.process_macros(stmt.span);
        intravisit::walk_stmt(self, stmt)
    }

    fn visit_local(&mut self, local: &'tcx hir::Local<'tcx>) {
        self.process_macros(local.span);
        intravisit::walk_local(self, local);
    }

    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx>) {
        self.process_macros(t.span);
        intravisit::walk_ty(self, t);
    }

    fn visit_path(&mut self, path: &'tcx hir::Path<'tcx>, _id: hir::HirId) {
        if let Some(def_id) = path.res.opt_def_id() {
            self.process_stability(def_id, path.span);
        }

        intravisit::walk_path(self, path);
    }

    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: hir::HirId, span: Span) {
        // NOTE: QPath::Resolved will be checked when visiting its inner path
        if let hir::QPath::TypeRelative(..) = qpath {
            if let Some(def_id) = self.tables.qpath_res(qpath, id).opt_def_id() {
                self.process_stability(def_id, span);
            }
        }

        intravisit::walk_qpath(self, qpath, id, span);
    }
}

pub fn walk_crate<'tcx>(wrapper: &mut Wrapper, tcx: TyCtxt<'tcx>, source_map: &SourceMap) {
    use intravisit::Visitor as _;

    let empty_tables = ty::TypeckTables::empty(None);
    let mut visitor = Visitor::new(tcx, &empty_tables, &wrapper.imported_macros);
    tcx.hir().krate().visit_all_item_likes(&mut visitor.as_deep_visitor());

    for (stab, spans) in visitor.lib_features {
        wrapper.features.insert(convert_stability(stab));
        wrapper
            .uses
            .entry(stab.feature.to_string())
            .or_default()
            .extend(spans.into_iter().map(|s| convert_span(source_map, s)));
    }
}
