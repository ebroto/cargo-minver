#![allow(unused)]

trait T {
    const A: i32;
}

struct S {}
impl S {
    const B: i32 = 42;
}

impl T for S {
    const A: i32 = 42;
}

fn main() {}
