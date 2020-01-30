#![feature(rustc_private)]
#![feature(process_exitcode_placeholder)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_span;
extern crate syntax;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::{interface::Compiler, Queries};
use rustc_span::source_map::Spanned;
use syntax::ast::{self, PatKind, RangeEnd, RangeSyntax};
use syntax::visit::{self, Visitor};

use std::env;
use std::process::{Command, ExitCode};
use std::str;

use anyhow::{bail, Error};

// TODO: modularize and move the rest to lib.rs
// TODO: structopt for parsing arguments
// TODO: pin to specific nightly

type Result<T> = std::result::Result<T, Error>;

struct MinverVisitor {}

impl<'a> Visitor<'a> for MinverVisitor {
    // TODO: This is just a POC. Instead of printing to stderr,
    //       collect information about the lang/lib features used.
    fn visit_expr(&mut self, e: &'a ast::Expr) {
        match e.kind {
            ast::ExprKind::Range(_, _, ast::RangeLimits::Closed) => {
                eprintln!("inclusive range syntax found");
            }
            _ => {}
        }
        visit::walk_expr(self, e);
    }

    fn visit_pat(&mut self, pattern: &'a ast::Pat) {
        match &pattern.kind {
            #[rustfmt::skip]
            PatKind::Range(_, _, Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), ..}) => {
                eprintln!("..= syntax in patterns found");
            }
            _ => {}
        }
        visit::walk_pat(self, pattern);
    }

    fn visit_mac(&mut self, _mac: &ast::Mac) {
        // Do nothing. The default implementation will panic to avoid misuse.
    }
}

struct MinverCallbacks {}

impl Callbacks for MinverCallbacks {
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        compiler.session().abort_if_errors();

        queries.global_ctxt().unwrap().peek_mut().enter(|_tcx| {
            let krate = queries.parse().unwrap().take();
            visit::walk_crate(&mut MinverVisitor {}, &krate);
        });

        Compilation::Continue
    }
}

// TODO: print error chain
fn main() -> ExitCode {
    match cargo_minver() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

// TODO: make this more readable
// TODO: add automatic +nightly...
fn cargo_minver() -> Result<()> {
    if env::var("CARGO_MINVER_INTERCEPT").is_ok() {
        let mut args = env::args().collect::<Vec<_>>();
        // Remove "rustc" from the argument list
        args.remove(1);

        if args.iter().any(|arg| arg == "--print=cfg") {
            Command::new("rustc").args(&args[1..]).status()?;
            Ok(())
        } else {
            args.extend(vec!["--sysroot".to_string(), fetch_sysroot()?]);
            match rustc_driver::catch_fatal_errors(|| {
                rustc_driver::run_compiler(&args, &mut MinverCallbacks {}, None, None)
            }) {
                Ok(_) => Ok(()),
                Err(_) => bail!("error running the compiler"),
            }
        }
    } else {
        run_cargo()
    }
}

fn run_cargo() -> Result<()> {
    let current_exe = env::current_exe()?;
    let exit_status = Command::new("cargo")
        .env("CARGO_MINVER_INTERCEPT", "1")
        .env("RUSTC_WRAPPER", current_exe)
        .args(vec!["check", "--tests", "--examples", "--benches"])
        .spawn()?
        .wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        bail!("error running cargo check")
    }
}

// TODO: check if we really need the more complex approaches
fn fetch_sysroot() -> Result<String> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;

    let sysroot = str::from_utf8(&output.stdout)?;
    Ok(sysroot.trim_end().to_string())
}
