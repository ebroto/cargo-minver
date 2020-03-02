// TODO: documentation

mod driver;
mod feature;
pub mod ipc;

pub const SERVER_PORT_ENV: &str = "MINVER_SERVER_PORT";

pub use driver::{Driver, Options};
pub use feature::*;
