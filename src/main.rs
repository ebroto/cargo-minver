#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_attr;
extern crate rustc_driver;
extern crate rustc_feature;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_resolve;
extern crate rustc_session;
extern crate rustc_span;
extern crate syntax;

mod driver;
mod feature;
mod ipc;
mod visitor;

use std::env;
use std::path::Path;
use std::process::Command;
use std::str;

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use crate::ipc::Server;

// TODO: move stuff to lib.rs, leave the minimum here
// TODO: pin to specific nightly

const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const SERVER_PORT_ENV: &str = "MINVER_SERVER_PORT";

// TODO: add automatic +nightly...
fn main() -> Result<()> {
    let current_exe = env::current_exe()?;

    if is_compiler_wrapper(&current_exe) {
        run_as_compiler_wrapper()
    } else {
        run_as_cargo_subcommand(&current_exe)
    }
}

fn is_compiler_wrapper<P: AsRef<Path>>(current_exe: P) -> bool {
    current_exe.as_ref().to_str().map_or(false, |p| {
        env::var(WRAPPER_ENV).map_or(false, |v| v.contains(p))
    })
}

fn run_as_compiler_wrapper() -> Result<()> {
    let mut args = env::args().collect::<Vec<_>>();
    // Remove "rustc" from the argument list
    args.remove(1);

    if args.iter().any(|arg| arg == "--print=cfg") {
        // Cargo is collecting information about the crate: passthrough to the actual compiler.
        Command::new("rustc")
            .args(&args[1..])
            .status()
            .context("failed to execute rustc")?;
        Ok(())
    } else {
        // Cargo is building a crate: run the compiler using our driver.
        args.extend(vec![
            "--sysroot".to_string(),
            fetch_sysroot().context("could not fetch sysroot")?,
        ]);
        let analysis = driver::run_compiler(&args)?;

        let port = server_port_from_env().context("invalid server port in environment")?;
        let address = ipc::server_address(port);
        let message = ipc::Message::Analysis(analysis);
        ipc::send_message(address, &message).context("failed to send analysis result to server")?;
        Ok(())
    }
}

// TODO: check if we really need the more complex approaches
fn fetch_sysroot() -> Result<String> {
    let output = Command::new("rustc")
        .args(vec!["--print", "sysroot"])
        .output()?;

    let sysroot = str::from_utf8(&output.stdout)?;
    Ok(sysroot.trim_end().to_string())
}

fn server_port_from_env() -> Result<u16> {
    let port_var = env::var(SERVER_PORT_ENV)?;
    let port = port_var.parse()?;
    Ok(port)
}

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Cargo {
    Minver(Options),
}

#[derive(Debug, StructOpt)]
struct Options {
    /// The port used by the local server.
    #[structopt(short, long, default_value = "64221")]
    server_port: u16,
}

fn run_as_cargo_subcommand<P: AsRef<Path>>(current_exe: P) -> Result<()> {
    // We need to run `cargo clean` to make sure we see all the code.
    cargo_clean().context("failed to execute cargo clean")?;

    // Start a server that will receive the results of the analysis of each crate.
    let Cargo::Minver(options) = Cargo::from_args();
    let address = ipc::server_address(options.server_port);
    let server = Server::new(address).context("could not start server")?;

    // Run `cargo check` to build all the crates.
    cargo_check(current_exe.as_ref(), options.server_port)
        .context("failed to execute cargo check")?;

    // Process the results of the analysis.
    let analysis = server
        .collect()
        .context("failed to retrieve analysis result")?;

    let mut features = analysis
        .iter()
        .map(|a| &a.features)
        .flatten()
        .collect::<Vec<_>>();

    features.sort_unstable_by(|a, b| {
        if a.since != b.since {
            b.since.cmp(&a.since)
        } else {
            a.name.cmp(&b.name)
        }
    });
    features.dedup();
    dbg!(&features);

    Ok(())
}

fn cargo_clean() -> Result<()> {
    let exit_status = Command::new("cargo") //
        .arg("clean")
        .spawn()?
        .wait()?;
    if !exit_status.success() {
        bail!("process returned error exit status")
    }
    Ok(())
}

fn cargo_check(current_exe: &Path, server_port: u16) -> Result<()> {
    let exit_status = Command::new("cargo")
        .env(WRAPPER_ENV, current_exe)
        .env(SERVER_PORT_ENV, server_port.to_string())
        .args(vec!["check", "--tests", "--examples", "--benches"])
        .spawn()?
        .wait()?;
    if !exit_status.success() {
        bail!("process returned error exit status")
    }
    Ok(())
}
