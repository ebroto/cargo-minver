use std::path::PathBuf;
use std::process::Command;
use std::{env, str};

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use crate::feature::Analysis;
use crate::ipc::Server;
use crate::SERVER_PORT_ENV;

const WRAPPER_ENV: &str = "RUSTC_WRAPPER";
const WRAPPER_NAME: &str = "minver-wrapper";

#[derive(Debug, Default, StructOpt)]
pub struct Options {
    /// The port used by the local server.
    #[structopt(short = "p", long, default_value = "64221")]
    server_port: u16,
    /// Path to the compiler wrapper.
    #[structopt(long)]
    wrapper_path: Option<PathBuf>,
    /// Path to Cargo.toml.
    #[structopt(long)]
    manifest_path: Option<PathBuf>,
    /// Don't print progress output.
    #[structopt(short = "q", long)]
    quiet: bool,
    /// Space-separated list of cargo features to activate
    #[structopt(long)]
    features: Option<String>,
    /// Activate all available cargo features
    #[structopt(long)]
    all_features: bool,
    /// Do not activate the `default` cargo feature
    #[structopt(long)]
    no_default_features: bool,
}

#[derive(Debug)]
pub struct Driver {
    opts: Options,
}

impl From<Options> for Driver {
    fn from(opts: Options) -> Self {
        Self { opts }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self { opts: Options { server_port: 64221, ..Default::default() } }
    }
}

impl Driver {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn server_port(&mut self, port: u16) -> &mut Self {
        self.opts.server_port = port;
        self
    }

    pub fn wrapper_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.opts.wrapper_path = Some(path.into());
        self
    }

    pub fn manifest_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.opts.manifest_path = Some(path.into());
        self
    }

    pub fn quiet(&mut self, value: bool) -> &mut Self {
        self.opts.quiet = value;
        self
    }

    pub fn features(&mut self, features: &str) -> &mut Self {
        self.opts.features = Some(features.into());
        self
    }

    pub fn all_features(&mut self, value: bool) -> &mut Self {
        self.opts.all_features = value;
        self
    }

    pub fn no_default_features(&mut self, value: bool) -> &mut Self {
        self.opts.no_default_features = value;
        self
    }

    pub fn execute(&mut self) -> Result<Analysis> {
        // Start a server that will receive the results of the analysis of each crate.
        let server = Server::new(self.opts.server_port).context("could not start server")?;

        // Build the crate and its dependencies. Run cargo clean before to make sure we see all the code.
        // TODO: Store stability information to avoid unnecessary rebuilds.
        self.cargo_clean().context("failed to execute cargo clean")?;
        self.cargo_check().context("failed to execute cargo check")?;

        // Process the results of the analysis.
        let analysis = server.into_analysis().context("failed to retrieve analysis result")?;
        Ok(analysis)
    }

    fn cargo_clean(&self) -> Result<()> {
        let mut command = Command::new("cargo");
        let mut builder = command.arg("clean");

        if let Some(path) = &self.opts.manifest_path {
            builder = builder.arg("--manifest-path").arg(path);
        }
        if self.opts.quiet {
            builder = builder.arg("--quiet");
        }

        let exit_status = builder.spawn()?.wait()?;
        if !exit_status.success() {
            bail!("process returned error exit status")
        }
        Ok(())
    }

    fn cargo_check(&self) -> Result<()> {
        fn path_to_wrapper(wrapper_path: Option<PathBuf>) -> Result<PathBuf> {
            let path = match wrapper_path {
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

        let toolchain = env!("MINVER_TOOLCHAIN");
        let wrapper_path = path_to_wrapper(self.opts.wrapper_path.clone()) //
            .context("could not find compiler wrapper")?;

        let mut command = Command::new("cargo");
        let mut builder = command
            .env(WRAPPER_ENV, wrapper_path)
            .env(SERVER_PORT_ENV, self.opts.server_port.to_string())
            .args(vec![toolchain, "check", "--tests", "--examples", "--benches"]);

        if let Some(path) = &self.opts.manifest_path {
            builder = builder.arg("--manifest-path").arg(path);
        }
        if self.opts.quiet {
            builder = builder.arg("--quiet");
        }
        if let Some(features) = self.opts.features.as_ref() {
            builder = builder.arg("--features").arg(features);
        }
        if self.opts.all_features {
            builder = builder.arg("--all-features");
        }
        if self.opts.no_default_features {
            builder = builder.arg("--no-default-features");
        }

        let exit_status = builder.spawn()?.wait()?;
        if !exit_status.success() {
            bail!("process returned error exit status")
        }
        Ok(())
    }
}
