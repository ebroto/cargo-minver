#![allow(unused)]

#[repr(packed(1))]
struct A {
    value: i32,
}

// should be ignored
#[repr(packed)]
struct B {
    value: i32,
}

fn main() {}
