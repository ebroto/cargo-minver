#![allow(unused)]

#[non_exhaustive]
struct S {}

#[non_exhaustive]
enum E {}

// should be ignored
struct X {}
enum Y {}

fn main() {}
