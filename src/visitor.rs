use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_attr::{self as attr, Stability};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_hir::intravisit::{self, NestedVisitorMap};
use rustc_span::source_map::Spanned;
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;
use syntax::ast::{self, Pat, PatKind, RangeEnd, RangeSyntax};
use syntax::ptr::P;
use syntax::visit;

use std::collections::HashSet;

use crate::feature::{Feature, FeatureKind};

#[derive(Debug, Default)]
pub struct PostExpansionVisitor {
    features: HashSet<Symbol>,
}

impl PostExpansionVisitor {
    pub fn collect_features(self) -> Vec<Feature> {
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
            }
            _ => {}
        }

        visit::walk_expr(self, e);
    }

    fn visit_pat(&mut self, pattern: &'a ast::Pat) {
        fn has_rest(ps: &[P<Pat>]) -> bool {
            ps.iter().any(|p| p.is_rest())
        }

        match &pattern.kind {
            #[rustfmt::skip]
            PatKind::Range(.., Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), ..}) => {
                self.features.insert(sym::dotdoteq_in_patterns);
            }
            PatKind::Tuple(ps) if has_rest(ps) => {
                self.features.insert(sym::dotdot_in_tuple_patterns);
            }
            PatKind::TupleStruct(_, ps) if ps.len() > 1 && has_rest(ps) => {
                self.features.insert(sym::dotdot_in_tuple_patterns);
            }
            _ => {}
        }

        visit::walk_pat(self, pattern);
    }

    fn visit_mac(&mut self, _mac: &ast::Mac) {
        // Do nothing. The default implementation will panic to avoid misuse.
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct LibFeature {
    name: Symbol,
    since: Option<Symbol>,
}

impl From<LibFeature> for Feature {
    fn from(feature: LibFeature) -> Self {
        Feature {
            name: feature.name.to_string(),
            kind: FeatureKind::Lib,
            since: feature.since.map(|s| s.as_str().parse().unwrap()),
        }
    }
}

pub struct StabilityCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    features: HashSet<LibFeature>,
}

impl<'tcx> StabilityCollector<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        StabilityCollector {
            tcx,
            features: HashSet::new(),
        }
    }

    pub fn collect_features(self) -> Vec<Feature> {
        self.features.into_iter().map(Into::into).collect()
    }

    fn process_stability(&mut self, def_id: DefId, span: Span) {
        if def_id.is_local() {
            return;
        }

        let stability = self.tcx.lookup_stability(def_id);
        match stability {
            Some(&Stability {
                level: attr::Unstable { .. },
                feature,
                ..
            }) => {
                // ignore internal features
                if !span.allows_unstable(feature) {
                    self.features.insert(LibFeature {
                        name: feature,
                        since: None,
                    });
                }
            }
            Some(&Stability {
                level: attr::Stable { since },
                feature,
                ..
            }) => {
                self.features.insert(LibFeature {
                    name: feature,
                    since: Some(since),
                });
            }
            _ => {}
        }
    }
}

// TODO: check extern crate and trait impls.
// TODO: see if the rest of stability checks can be done here.
impl<'tcx> intravisit::Visitor<'tcx> for StabilityCollector<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<'_, Self::Map> {
        // TODO: check if OnlyBodies is enough
        NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_path(&mut self, path: &'tcx hir::Path<'tcx>, _id: hir::HirId) {
        if let Some(def_id) = path.res.opt_def_id() {
            self.process_stability(def_id, path.span);
        }
        intravisit::walk_path(self, path);
    }
}
