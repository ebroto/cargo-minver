use rustc_driver::{Callbacks, Compilation};
use rustc_hir::intravisit;
use rustc_interface::interface::Compiler;
use rustc_interface::Queries;
use syntax::visit;

use anyhow::{format_err, Result};

use crate::feature::CrateAnalysis;
use crate::visitor::{self, PostExpansionVisitor, StabilityCollector};

#[derive(Debug, Default)]
struct MinverCallbacks {
    analysis: CrateAnalysis,
}

impl Callbacks for MinverCallbacks {
    fn after_expansion<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        let session = compiler.session();
        session.abort_if_errors();

        let (krate, boxed_resolver, ..) = &*queries.expansion().unwrap().peek();
        boxed_resolver.borrow().borrow_mut().access(|resolver| {
            let features = visitor::process_imported_macros(session, resolver);
            self.analysis.features.extend(features);
        });

        let mut visitor = PostExpansionVisitor::default();
        visit::walk_crate(&mut visitor, &krate);
        self.analysis.features.extend(visitor.into_features());

        Compilation::Continue
    }

    fn after_analysis<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        compiler.session().abort_if_errors();

        self.analysis.name = queries.crate_name().unwrap().peek().clone();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let krate = tcx.hir().krate();
            let mut visitor = StabilityCollector::new(tcx);
            intravisit::walk_crate(&mut visitor, krate);

            self.analysis.features.extend(visitor.into_features());
        });

        Compilation::Continue
    }
}

pub fn run_compiler(args: &[String]) -> Result<CrateAnalysis> {
    let mut callbacks = MinverCallbacks::default();

    // NOTE: The error returned from the driver was already displayed.
    rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(args, &mut callbacks, None, None))
        .map(|_| callbacks.analysis)
        .map_err(|_| format_err!("compiler errored out"))
}
