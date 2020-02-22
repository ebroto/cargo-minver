#![feature(rustc_private)]

// TODO: documentation

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
pub mod ipc;
mod visitor;
pub mod wrapper;

pub const SERVER_PORT_ENV: &str = "MINVER_SERVER_PORT";

pub use driver::Driver;
