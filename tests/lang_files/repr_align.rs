#![allow(unused)]

#[repr(align(16))]
struct A {
    value: i32,
}

#[repr(align(16))]
union B {
    first: i32,
    second: u32,
}

fn main() {}
