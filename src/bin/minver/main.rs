use anyhow::Result;
use structopt::StructOpt;

use cargo_minver::Driver;

// TODO: pin to specific nightly
// TODO: add automatic +nightly...

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Cargo {
    Minver(Options),
}

#[derive(Debug, StructOpt)]
struct Options {
    /// The port used by the local server.
    #[structopt(short, long, name = "PORT", default_value = "64221")]
    server_port: u16,
}

fn main() -> Result<()> {
    let Cargo::Minver(options) = Cargo::from_args();
    let analysis = Driver::new(options.server_port).execute()?;
    let features = analysis.all_features();
    dbg!(&features);
    Ok(())
}
