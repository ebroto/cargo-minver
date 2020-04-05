#![allow(unused)]

use std::fmt::Debug;

#[derive(Debug)]
struct S<'a> {
    value: &'a i32,
}

fn foo(s: S<'_>) {}

struct A {
    value: i32,
}
impl A {
    fn foo(&self) -> S<'_> {
        S { value: &self.value }
    }

    fn bar(&self) -> Box<dyn Debug + '_> {
        Box::new(S { value: &self.value })
    }

    fn baz(&self) -> Box<impl Debug + '_> {
        Box::new(S { value: &self.value })
    }
}

fn main() {}
