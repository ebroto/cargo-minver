#![allow(unused)]

trait T {}
impl T for usize {}

fn fun() -> impl T {
    42
}

fn main() {}
