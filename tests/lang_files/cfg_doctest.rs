#![allow(unused)]

#[cfg(doctest)]
fn fun() {}

#[cfg(not(doctest))]
fn not_fun() {}

fn main() {
    if cfg!(doctest) {}
    if cfg!(all(doctest, test)) {}
}
