#![feature(rustc_private)]
#![feature(or_patterns)]

extern crate rustc;
extern crate rustc_ast;
extern crate rustc_attr;
extern crate rustc_driver;
extern crate rustc_feature;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_parse;
extern crate rustc_resolve;
extern crate rustc_session;
extern crate rustc_span;

mod context;
mod post_analysis;
mod post_expansion;
mod pre_expansion;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::Compiler;
use rustc_interface::Queries;

use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::{env, str};

use anyhow::{format_err, Context, Result};

use cargo_minver::ipc::{self, Message};
use cargo_minver::{CrateAnalysis, Feature, Span, SERVER_PORT_ENV};

fn main() -> Result<()> {
    let mut args = env::args().collect::<Vec<_>>();
    args.remove(1); // Remove "rustc" from the argument list

    if args.iter().any(|arg| arg == "--print=cfg") {
        // Cargo is collecting information about the crate: passthrough to the actual compiler.
        Command::new("rustc").args(&args[1..]).status().context("failed to execute rustc")?;
        Ok(())
    } else {
        // Cargo is building a crate: run the compiler using our wrapper.
        args.extend(vec!["--sysroot".to_string(), fetch_sysroot().context("could not fetch sysroot")?]);
        let mut wrapper = Wrapper::default();
        rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(&args, &mut wrapper, None, None).ok())
            .map_err(|_| format_err!("compiler returned error exit status"))?;

        // Send the results to the server.
        let port = server_port_from_env().context("invalid server port in environment")?;
        let message = Message::AnalysisResult(CrateAnalysis::from(wrapper));
        ipc::send_message(port, &message).context("failed to send analysis result to server")?;
        Ok(())
    }
}

// TODO: full-fledged sysroot detection (see e.g. clippy)
fn fetch_sysroot() -> Result<String> {
    let output = Command::new("rustc").args(vec!["--print", "sysroot"]).output()?;
    let sysroot = str::from_utf8(&output.stdout)?;
    Ok(sysroot.trim_end().to_string())
}

fn server_port_from_env() -> Result<u16> {
    let port_var = env::var(SERVER_PORT_ENV)?;
    let port = port_var.parse()?;
    Ok(port)
}

#[derive(Debug, Default)]
pub struct Wrapper {
    crate_name: String,
    features: HashSet<Feature>,
    uses: HashMap<String, HashSet<Span>>,
}

impl Callbacks for Wrapper {
    fn after_parsing<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        let session = compiler.session();
        session.abort_if_errors();

        let krate = &*queries.parse().unwrap().peek();
        pre_expansion::process_crate(self, session, krate);

        Compilation::Continue
    }

    fn after_expansion<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        let session = compiler.session();
        session.abort_if_errors();

        let (krate, boxed_resolver, ..) = &*queries.expansion().unwrap().peek();
        boxed_resolver.borrow().borrow_mut().access(|resolver| {
            post_expansion::process_crate(self, session, krate, resolver);
        });

        Compilation::Continue
    }

    fn after_analysis<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        compiler.session().abort_if_errors();

        self.crate_name = queries.crate_name().unwrap().peek().clone();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            post_analysis::process_crate(self, tcx);
        });

        Compilation::Continue
    }
}

impl From<Wrapper> for CrateAnalysis {
    fn from(wrapper: Wrapper) -> Self {
        CrateAnalysis {
            name: wrapper.crate_name,
            features: wrapper.features.into_iter().collect(),
            uses: wrapper.uses.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect(),
        }
    }
}
