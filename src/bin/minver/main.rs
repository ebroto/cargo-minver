use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, str};

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use cargo_minver::ipc::{self, Server};
use cargo_minver::SERVER_PORT_ENV;

// TODO: move stuff to lib.rs, leave the minimum here
// TODO: pin to specific nightly

const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const WRAPPER_NAME: &str = "minver-wrapper";

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

// TODO: add automatic +nightly...
fn main() -> Result<()> {
    // We need to run `cargo clean` to make sure we see all the code.
    // TODO: improve this.
    cargo_clean().context("failed to execute cargo clean")?;

    // Start a server that will receive the results of the analysis of each crate.
    let Cargo::Minver(options) = Cargo::from_args();
    let address = ipc::server_address(options.server_port);
    let server = Server::new(address).context("could not start server")?;

    // Run `cargo check` to build all the crates.
    let wrapper_path = path_to_wrapper().context("could not find compiler wrapper")?;
    cargo_check(wrapper_path.as_ref(), options.server_port).context("failed to execute cargo check")?;

    // Process the results of the analysis.
    let analysis = server.collect().context("failed to retrieve analysis result")?;
    let mut features = analysis.iter().map(|a| &a.features).flatten().collect::<Vec<_>>();
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
    let exit_status = Command::new("cargo").arg("clean").spawn()?.wait()?;
    if !exit_status.success() {
        bail!("process returned error exit status")
    }
    Ok(())
}

fn path_to_wrapper() -> Result<PathBuf> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push(WRAPPER_NAME);
    if !path.is_file() {
        bail!("{} does not exist or is not a file", path.display());
    }
    Ok(path)
}

fn cargo_check(wrapper_path: &Path, server_port: u16) -> Result<()> {
    let exit_status = Command::new("cargo")
        .env(WRAPPER_ENV, wrapper_path)
        .env(SERVER_PORT_ENV, server_port.to_string())
        .args(vec!["check", "--tests", "--examples", "--benches"])
        .spawn()?
        .wait()?;
    if !exit_status.success() {
        bail!("process returned error exit status")
    }
    Ok(())
}
