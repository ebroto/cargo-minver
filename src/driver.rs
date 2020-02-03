use rustc_driver::{Callbacks, Compilation};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_interface::{interface::Compiler, Queries};
use rustc_span::source_map::Spanned;
use rustc_span::symbol::{sym, Symbol};
use syntax::ast::{self, Pat, PatKind, RangeEnd, RangeSyntax};
use syntax::attr;
use syntax::ptr::P;
use syntax::visit::{self, Visitor};

use std::collections::HashSet;

use anyhow::{format_err, Result};

use crate::feature::{CrateAnalysis, Feature};

#[derive(Debug, Default)]
struct PostExpansionVisitor {
    features: HashSet<Symbol>,
}

impl<'a> Visitor<'a> for PostExpansionVisitor {
    // TODO: add missing lang features
    fn visit_expr(&mut self, e: &'a ast::Expr) {
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
            PatKind::Range(_, _, Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), ..}) => {
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

#[derive(Debug, Default)]
struct MinverCallbacks {
    analysis: CrateAnalysis,
}

impl Callbacks for MinverCallbacks {
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        let krate = queries.parse().unwrap().take();
        // TODO: see what's up with all the unknown crates, are all of them build scripts?
        //       also, link build_scripts with crates
        self.analysis.name = match attr::find_crate_name(&krate.attrs) {
            Some(name) => name.to_string(),
            None => String::from("unknown_crate"),
        };

        let mut visitor = PostExpansionVisitor::default();
        visit::walk_crate(&mut visitor, &krate);

        use std::convert::TryInto;
        let features = visitor
            .features
            .iter()
            .flat_map(|name| ACCEPTED_FEATURES.iter().find(|f| &f.name == name))
            .flat_map(|feature| feature.try_into().ok())
            .collect::<Vec<Feature>>();

        self.analysis.features.extend(features);
        Compilation::Continue
    }
}

pub fn run_compiler(args: &[String]) -> Result<CrateAnalysis> {
    let mut callbacks = MinverCallbacks::default();

    // NOTE: The error returned from the driver was already displayed.
    rustc_driver::catch_fatal_errors(|| {
        rustc_driver::run_compiler(args, &mut callbacks, None, None)
    })
    .map(|_| callbacks.analysis)
    .map_err(|_| format_err!("compiler errored out"))
}
