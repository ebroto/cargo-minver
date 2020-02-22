use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, str};

use anyhow::{bail, Context, Result};

use crate::feature::Analysis;
use crate::ipc::Server;
use crate::SERVER_PORT_ENV;

const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const WRAPPER_NAME: &str = "minver-wrapper";

#[derive(Debug)]
pub struct Driver {
    server_port: u16,
    wrapper_path: Option<PathBuf>,
}

impl Driver {
    pub fn new(server_port: u16) -> Self {
        Self {
            server_port,
            wrapper_path: None,
        }
    }

    pub fn wrapper_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.wrapper_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn execute(&mut self) -> Result<Analysis> {
        // We need to run `cargo clean` to make sure we see all the code.
        // TODO: improve this.
        cargo_clean().context("failed to execute cargo clean")?;

        // Start a server that will receive the results of the analysis of each crate.
        let server = Server::new(self.server_port).context("could not start server")?;

        // Run `cargo check` to build all the crates.
        let wrapper_path = self.path_to_wrapper().context("could not find compiler wrapper")?;
        cargo_check(wrapper_path.as_ref(), self.server_port).context("failed to execute cargo check")?;

        // Process the results of the analysis.
        let analysis = server.into_analysis().context("failed to retrieve analysis result")?;
        Ok(analysis)
    }

    fn path_to_wrapper(&self) -> Result<PathBuf> {
        let path = match self.wrapper_path.clone() {
            Some(path) => path,
            None => {
                let mut path = env::current_exe()?;
                path.pop();
                path.push(WRAPPER_NAME);
                path
            },
        };
        if !path.is_file() {
            bail!("{} does not exist or is not a file", path.display());
        }
        Ok(path)
    }
}

fn cargo_clean() -> Result<()> {
    let exit_status = Command::new("cargo").arg("clean").spawn()?.wait()?;
    if !exit_status.success() {
        bail!("process returned error exit status")
    }
    Ok(())
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
