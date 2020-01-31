#![feature(rustc_private)]

mod driver;
mod feature;
mod ipc;

use std::env;
use std::process::{self, Command};
use std::str;

use anyhow::{bail, Error};

use crate::ipc::Server;

// TODO: move stuff to lib.rs, leave the minimum here
// TODO: structopt for parsing arguments
// TODO: pin to specific nightly

type Result<T> = std::result::Result<T, Error>;

// TODO: configurable
const SERVER_ADDRESS: &str = "127.0.0.1:64221";

// TODO: print error chain
fn main() {
    process::exit(cargo_minver().map_or(101, |_| 0));
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

            let analysis = driver::run_compiler(&args)?;
            ipc::send_message(&SERVER_ADDRESS, &ipc::Message::Analysis(analysis))?;
            Ok(())
        }
    } else {
        run_cargo_check()
    }
}

// TODO: force "cargo clean"
fn run_cargo_check() -> Result<()> {
    let server = Server::new(SERVER_ADDRESS)?;

    let current_exe = env::current_exe()?;
    let exit_status = Command::new("cargo")
        .env("CARGO_MINVER_INTERCEPT", "1")
        .env("RUSTC_WRAPPER", current_exe)
        .args(vec!["check", "--tests", "--examples", "--benches"])
        .spawn()?
        .wait()?;

    if !exit_status.success() {
        bail!("error running cargo check")
    }

    let analysis = server.collect()?;
    dbg!(analysis);
    Ok(())
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
