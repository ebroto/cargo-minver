#![allow(unused)]

pub(crate) mod foo {}

pub(self) mod bar {}

struct S {
    pub(self) x: i32,
}
impl S {
    pub(self) fn fun() {}
}
extern "C" {
    pub(self) fn fun();
}

fn main() {}
