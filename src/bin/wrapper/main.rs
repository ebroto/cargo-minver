use std::env;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    cargo_minver::run_as_compiler_wrapper(env::args()).context("could not run as compiler wrapper")
}
