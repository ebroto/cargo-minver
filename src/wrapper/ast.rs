use rustc_attr::{Stability, StabilityLevel};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_resolve::{ParentScope, Resolver};
use rustc_session::Session;
use rustc_span::source_map::Spanned;
use rustc_span::symbol::{sym, Symbol};
use syntax::ast::{self, Pat, PatKind, RangeEnd, RangeSyntax};
use syntax::ptr::P;
use syntax::visit;

use std::collections::HashSet;

use crate::feature::Feature;

#[derive(Debug, Default)]
struct Visitor {
    features: HashSet<Symbol>,
}

impl Visitor {
    fn into_features(self) -> Vec<Feature> {
        self.features
            .into_iter()
            .flat_map(|name| ACCEPTED_FEATURES.iter().find(|f| f.name == name))
            .map(Into::into)
            .collect()
    }
}

impl<'a> visit::Visitor<'a> for Visitor {
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
            #[rustfmt::skip]
            PatKind::Range(.., Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), .. }) => {
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

pub fn walk_crate(krate: &ast::Crate) -> Vec<Feature> {
    let mut visitor = Visitor::default();
    visit::walk_crate(&mut visitor, &krate);
    visitor.into_features()
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
            let path = ast::Path::from_ident(ast::Ident::from_str(name));
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
