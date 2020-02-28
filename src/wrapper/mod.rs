use rustc_attr::Stability;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::Compiler;
use rustc_interface::Queries;
use rustc_span::source_map::SourceMap;

use std::collections::HashMap;
use std::process::Command;
use std::{env, str};

use anyhow::{format_err, Context, Result};

use crate::feature::{CrateAnalysis, Span};
use crate::ipc::{self, Message};
use crate::SERVER_PORT_ENV;

mod ast;
mod hir;

#[derive(Debug, Default)]
struct MinverCallbacks {
    analysis: CrateAnalysis,
    // Maps an imported macro name to its stability attributes. This is used when inspecting the HIR
    // to relate the feature to a set of spans. See process_imported_macros for more details.
    imported_macros: HashMap<String, Stability>,
}

impl Callbacks for MinverCallbacks {
    fn after_expansion<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        let session = compiler.session();
        session.abort_if_errors();

        let (krate, boxed_resolver, ..) = &*queries.expansion().unwrap().peek();
        boxed_resolver.borrow().borrow_mut().access(|resolver| {
            self.imported_macros = ast::process_imported_macros(session, resolver);
        });

        ast::walk_crate(&krate, session.source_map(), &mut self.analysis);

        Compilation::Continue
    }

    fn after_analysis<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
        let session = compiler.session();
        session.abort_if_errors();

        self.analysis.name = queries.crate_name().unwrap().peek().clone();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            hir::walk_crate(tcx, session.source_map(), &mut self.analysis, &mut self.imported_macros);
        });

        Compilation::Continue
    }
}

pub fn run<I: IntoIterator<Item = String>>(args: I) -> Result<()> {
    let mut args = args.into_iter().collect::<Vec<_>>();
    args.remove(1); // Remove "rustc" from the argument list

    if args.iter().any(|arg| arg == "--print=cfg") {
        // Cargo is collecting information about the crate: passthrough to the actual compiler.
        Command::new("rustc")
            .args(&args[1..])
            .status()
            .context("failed to execute rustc")?;
        Ok(())
    } else {
        // Cargo is building a crate: run the compiler using our wrapper.
        args.extend(vec![
            "--sysroot".to_string(),
            fetch_sysroot().context("could not fetch sysroot")?,
        ]);
        let mut callbacks = MinverCallbacks::default();
        rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(&args, &mut callbacks, None, None).ok())
            .map_err(|_| format_err!("compiler returned error exit status"))?;

        // Send the results to the server.
        let port = server_port_from_env().context("invalid server port in environment")?;
        ipc::send_message(port, &Message::AnalysisResult(callbacks.analysis))
            .context("failed to send analysis result to server")?;
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

fn convert_span(source_map: &SourceMap, span: rustc_span::Span) -> Span {
    let start = source_map.lookup_char_pos(span.lo());
    let end = source_map.lookup_char_pos(span.hi());

    Span {
        file_name: start.file.name.to_string(),
        start_line: start.line,
        start_col: start.col.0,
        end_line: end.line,
        end_col: end.col.0,
    }
}
