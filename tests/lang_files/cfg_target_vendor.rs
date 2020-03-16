#![allow(unused)]

#[cfg(target_vendor = "unknown")]
fn fun() {}

#[cfg(not(target_vendor = "unknown"))]
fn not_fun() {}

fn main() {
    if cfg!(target_vendor = "unknown") {}
    if cfg!(not(target_vendor = "unknown")) {}
}
