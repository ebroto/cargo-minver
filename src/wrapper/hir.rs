use rustc::hir::map::Map;
use rustc::ty::{self, TyCtxt};
use rustc_attr as attr;
use rustc_attr::Stability;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, CRATE_DEF_INDEX};
use rustc_hir::intravisit::{self, NestedVisitorMap};
use rustc_hir::pat_util::EnumerateAndAdjustIterator;
use rustc_span::symbol::Ident;
use rustc_span::Span;

use std::collections::HashSet;
use std::mem;

use crate::feature::Feature;

pub struct Visitor<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    features: HashSet<Stability>,
    tables: &'a ty::TypeckTables<'tcx>,
    empty_tables: &'a ty::TypeckTables<'tcx>,
}

impl<'a, 'tcx> Visitor<'a, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, empty_tables: &'a ty::TypeckTables<'tcx>) -> Self {
        Visitor {
            tcx,
            features: HashSet::new(),
            tables: empty_tables,
            empty_tables,
        }
    }

    pub fn into_features(self) -> Vec<Feature> {
        self.features.into_iter().map(Into::into).collect()
    }

    fn process_stability(&mut self, def_id: DefId, span: Span) {
        if def_id.is_local() {
            return;
        }

        if let Some(stab) = self.tcx.lookup_stability(def_id) {
            dbg!(&def_id);
            match stab.level {
                attr::Unstable { .. } if span.allows_unstable(stab.feature) => {},
                _ => {
                    self.features.insert(*stab);
                },
            }
        }
    }

    fn process_fields(&mut self, ty_kind: &ty::TyKind, fields: &[(Ident, Span)]) {
        match ty_kind {
            ty::Adt(def, _) if !def.is_enum() => {
                let variant = def.non_enum_variant();
                for (ident, span) in fields {
                    if let Some(ty_field) = self
                        .tcx
                        .find_field_index(*ident, variant)
                        .map(|index| &variant.fields[index])
                    {
                        self.process_stability(ty_field.did, *span);
                    }
                }
            },
            _ => {},
        }
    }

    fn with_item_tables<F>(&mut self, hir_id: hir::HirId, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let def_id = self.tcx.hir().local_def_id(hir_id);
        let tables = if self.tcx.has_typeck_tables(def_id) {
            self.tcx.typeck_tables_of(def_id)
        } else {
            self.empty_tables
        };

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
                let def_id = DefId {
                    krate: cnum,
                    index: CRATE_DEF_INDEX,
                };
                self.process_stability(def_id, item.span);
            },

            hir::ItemKind::Impl {
                of_trait: Some(ref t),
                items,
                ..
            } => {
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

pub fn walk_crate<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<Feature> {
    use intravisit::Visitor as _;

    let empty_tables = ty::TypeckTables::empty(None);
    let mut visitor = Visitor::new(tcx, &empty_tables);
    tcx.hir().krate().visit_all_item_likes(&mut visitor.as_deep_visitor());

    visitor.into_features()
}
