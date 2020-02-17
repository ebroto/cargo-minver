use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_attr::{self as attr, Stability, StabilityLevel};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, CRATE_DEF_INDEX};
use rustc_hir::intravisit::{self, NestedVisitorMap};
use rustc_resolve::{ParentScope, Resolver};
use rustc_session::Session;
use rustc_span::source_map::Spanned;
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;
use syntax::ast::{self, Ident, Pat, PatKind, RangeEnd, RangeSyntax};
use syntax::ptr::P;
use syntax::visit;

use std::collections::HashSet;

use crate::feature::Feature;

// TODO: stability module, separate visitors in different file(s)?

#[derive(Debug, Default)]
pub struct PostExpansionVisitor {
    features: HashSet<Symbol>,
}

impl PostExpansionVisitor {
    pub fn into_features(self) -> Vec<Feature> {
        self.features
            .into_iter()
            .flat_map(|name| ACCEPTED_FEATURES.iter().find(|f| f.name == name))
            .map(Into::into)
            .collect()
    }
}

impl<'a> visit::Visitor<'a> for PostExpansionVisitor {
    // TODO: add missing lang features
    fn visit_expr(&mut self, e: &'a ast::Expr) {
        #[allow(clippy::single_match)]
        match e.kind {
            ast::ExprKind::Range(_, _, ast::RangeLimits::Closed) => {
                self.features.insert(sym::inclusive_range_syntax);
            },
            _ => {},
        }

        visit::walk_expr(self, e);
    }

    fn visit_pat(&mut self, pattern: &'a ast::Pat) {
        fn has_rest(ps: &[P<Pat>]) -> bool {
            ps.iter().any(|p| p.is_rest())
        }

        match &pattern.kind {
            PatKind::Range(
                ..,
                Spanned {
                    node: RangeEnd::Included(RangeSyntax::DotDotEq),
                    ..
                },
            ) => {
                self.features.insert(sym::dotdoteq_in_patterns);
            },
            PatKind::Tuple(ps) if has_rest(ps) => {
                self.features.insert(sym::dotdot_in_tuple_patterns);
            },
            PatKind::TupleStruct(_, ps) if ps.len() > 1 && has_rest(ps) => {
                self.features.insert(sym::dotdot_in_tuple_patterns);
            },
            _ => {},
        }

        visit::walk_pat(self, pattern);
    }

    fn visit_mac(&mut self, _mac: &ast::Mac) {
        // Do nothing.
    }
}

pub struct StabilityCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    features: HashSet<Stability>,
}

impl<'tcx> StabilityCollector<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        StabilityCollector {
            tcx,
            features: HashSet::new(),
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
            if let attr::Unstable { .. } = stab.level {
                if span.allows_unstable(stab.feature) {
                    return;
                }
            }

            self.features.insert(*stab);
        }
    }
}

// TODO: see if the rest of stability checks can be done here.
impl<'tcx> intravisit::Visitor<'tcx> for StabilityCollector<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<'_, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
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
                            .find(|item| item.ident.name == impl_item.ident.name)
                            .map(|item| item.def_id);
                        if let Some(def_id) = trait_item_def_id {
                            self.process_stability(def_id, impl_item.span);
                        }
                    }
                }
            },

            _ => {},
        }

        intravisit::walk_item(self, item);
    }

    fn visit_path(&mut self, path: &'tcx hir::Path<'tcx>, _id: hir::HirId) {
        if let Some(def_id) = path.res.opt_def_id() {
            self.process_stability(def_id, path.span);
        }

        intravisit::walk_path(self, path);
    }
}

// TODO: improve this. We need to check stability attributes for unexpanded macros; after parsing
// we have the invocation but not the definition and the resolver is not available, and after
// expansion we have the resolver but not the invocation. Luckily, the session stores info about
// imported macros (only used by the RLS), but we lose the context of which invocations triggered
// which expansions.
// In this function we are able to fetch the stability attributes from those macros, but without
// context we can't correctly evaluate the #[allow_internal_unstable] attribute in case where one
// macro uses another macro with that attribute. As a workaround, we check all unstable features
// against all the allowed unstable attributes, which is OKish for our use case, but not correct.
// There's probable a better solution. This is not over!!
pub fn process_imported_macros(session: &Session, resolver: &mut Resolver) -> Vec<Feature> {
    // Resolve again all of the imported macros. This gives us access to the stability attributes.
    let resolved = session
        .imported_macro_spans
        .borrow()
        .iter()
        .filter_map(|(_, (name, _))| {
            let path = ast::Path::from_ident(Ident::from_str(name));
            let result =
                resolver.resolve_macro_path(&path, None, &ParentScope::module(resolver.graph_root()), false, false);
            match result {
                Ok((ext, ..)) => ext,
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    // Collect all of the allowed unstable features. See the comment above.
    let allowed_unstable = resolved
        .iter()
        .flat_map(|ext| ext.allow_internal_unstable.as_ref())
        .map(|list| list.iter().cloned())
        .flatten()
        .collect::<Vec<_>>();

    // Transform and dedup features that are stable or unstable and not allowed.
    let features = resolved
        .into_iter()
        .flat_map(|ext| ext.stability)
        .filter(|stab| match stab {
            Stability {
                level: StabilityLevel::Stable { .. },
                ..
            } => true,
            Stability { feature, .. } => !allowed_unstable.iter().any(|allowed| feature == allowed),
        })
        .collect::<HashSet<_>>();

    // Map into our feature representation.
    features.into_iter().map(Into::into).collect::<Vec<_>>()
}
