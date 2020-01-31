extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_span;
extern crate syntax;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::{interface::Compiler, Queries};
use rustc_span::source_map::Spanned;
use syntax::ast::{self, PatKind, RangeEnd, RangeSyntax};
use syntax::attr;
use syntax::visit::{self, Visitor};

use std::collections::HashSet;

use anyhow::{format_err, Result};

use crate::feature::CrateAnalysis;

#[derive(Debug, Default)]
struct PostExpansionVisitor {
    features: HashSet<&'static str>,
}

impl<'a> Visitor<'a> for PostExpansionVisitor {
    // TODO: use features sym?
    // TODO: add missing lang features
    fn visit_expr(&mut self, e: &'a ast::Expr) {
        match e.kind {
            ast::ExprKind::Range(_, _, ast::RangeLimits::Closed) => {
                self.features.insert("inclusive range syntax");
            }
            _ => {}
        }
        visit::walk_expr(self, e);
    }

    fn visit_pat(&mut self, pattern: &'a ast::Pat) {
        match &pattern.kind {
            #[rustfmt::skip]
            PatKind::Range(_, _, Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), ..}) => {
                self.features.insert("..= syntax in patterns");
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
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        compiler.session().abort_if_errors();

        let krate = queries.parse().unwrap().take();
        // TODO: see what's up with all the unknown crates, are all of them build scripts?
        //       also, link build_scripts with crates
        self.analysis.name = match attr::find_crate_name(&krate.attrs) {
            Some(name) => name.to_string(),
            None => String::from("unknown_crate"),
        };

        let mut visitor: PostExpansionVisitor = Default::default();
        visit::walk_crate(&mut visitor, &krate);
        // TODO: fetch features with given name from ACCEPTED and translate to this crate feature repr
        self.analysis.features = vec![];

        Compilation::Continue
    }
}

pub fn run_compiler(args: &[String]) -> Result<CrateAnalysis> {
    let mut callbacks: MinverCallbacks = Default::default();

    // NOTE: The error returned from the driver was already displayed.
    rustc_driver::catch_fatal_errors(|| {
        rustc_driver::run_compiler(args, &mut callbacks, None, None)
    })
    .map(|_| callbacks.analysis)
    .map_err(|_| format_err!("error running the compiler"))
}
