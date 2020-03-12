use anyhow::Result;
use structopt::StructOpt;

use cargo_minver::{Driver, Options};

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Cargo {
    Minver(Options),
}

fn main() -> Result<()> {
    let Cargo::Minver(options) = Cargo::from_args();

    let analysis = Driver::from(options).execute()?;
    let features = analysis.all_features();
    dbg!(&features);

    Ok(())
}
