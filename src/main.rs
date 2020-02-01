#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_feature;
extern crate rustc_interface;
extern crate rustc_span;
extern crate syntax;

mod driver;
mod feature;
mod ipc;

use std::env;
use std::path::Path;
use std::process::{self, Command};
use std::str;

use anyhow::{bail, Result};

use crate::ipc::Server;

// TODO: move stuff to lib.rs, leave the minimum here
// TODO: structopt for parsing arguments
// TODO: pin to specific nightly

// TODO: configurable
const SERVER_ADDRESS: &str = "127.0.0.1:64221";
const WRAPPER_ENV: &str = "RUSTC_WRAPPER";

// TODO: print error chain
fn main() {
    process::exit(cargo_minver().map_or(101, |_| 0));
}

// TODO: add automatic +nightly...
fn cargo_minver() -> Result<()> {
    let current_exe = env::current_exe()?;

    if is_compiler_wrapper(&current_exe) {
        run_as_compiler_wrapper()
    } else {
        run_as_cargo_subcommand(&current_exe)
    }
}

fn run_as_compiler_wrapper() -> Result<()> {
    let mut args = env::args().collect::<Vec<_>>();
    // Remove "rustc" from the argument list
    args.remove(1);

    if args.iter().any(|arg| arg == "--print=cfg") {
        // Cargo is collecting information about the crate: passthrough to the actual compiler.
        Command::new("rustc").args(&args[1..]).status()?;
        Ok(())
    } else {
        // Cargo is building a crate: run the compiler using our driver.
        args.extend(vec!["--sysroot".to_string(), fetch_sysroot()?]);

        let analysis = driver::run_compiler(&args)?;
        ipc::send_message(&SERVER_ADDRESS, &ipc::Message::Analysis(analysis))?;
        Ok(())
    }
}

fn run_as_cargo_subcommand<P: AsRef<Path>>(current_exe: P) -> Result<()> {
    // We need to run `cargo clean` to make sure we see all the code.
    let exit_status = Command::new("cargo") //
        .arg("clean")
        .spawn()?
        .wait()?;
    if !exit_status.success() {
        bail!("error running cargo clean")
    }

    // Start a server that will receive the results of the analysis of each crate.
    let server = Server::new(SERVER_ADDRESS)?;

    // Run `cargo check` to build all the crates.
    let exit_status = Command::new("cargo")
        .env(WRAPPER_ENV, current_exe.as_ref())
        .args(vec!["check", "--tests", "--examples", "--benches"])
        .spawn()?
        .wait()?;
    if !exit_status.success() {
        bail!("error running cargo check")
    }

    // Process the results of the analysis.
    let analysis = server.collect()?;
    let mut features = analysis
        .iter()
        .map(|a| &a.features)
        .flatten()
        .collect::<Vec<_>>();
    features.sort_by(|a, b| a.since.partial_cmp(&b.since).unwrap());

    dbg!(&features);
    Ok(())
}

fn is_compiler_wrapper<P: AsRef<Path>>(current_exe: P) -> bool {
    current_exe.as_ref().to_str().map_or(false, |p| {
        env::var(WRAPPER_ENV).map_or(false, |v| v.contains(p))
    })
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
