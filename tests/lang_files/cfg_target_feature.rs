#![allow(unused)]

#[cfg(target_feature = "x")]
fn fun() {}

#[cfg(not(target_feature = "x"))]
fn not_fun() {}

fn main() {
    if cfg!(target_feature = "x") {}
    if cfg!(not(target_feature = "x")) {}
}
