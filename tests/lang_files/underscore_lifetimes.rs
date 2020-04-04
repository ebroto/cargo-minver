#![allow(unused)]

struct S<'a> {
    value: &'a i32,
}

trait T {
    fn foo(&self) {}
}

impl S<'_> {
    fn frob() {}
}

impl T for S<'_> {
    fn foo(&self) {}
}

fn mk_s<'a>(value: &'a i32) -> S<'_> {
    S { value }
}

fn main() {}
