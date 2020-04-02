#![allow(unused)]

const fn fun() {}

struct S {}
impl S {
    const fn meth() {}
}

// should be ignored
fn not_fun() {}
impl S {
    fn not_meth() {}
}

fn main() {}
