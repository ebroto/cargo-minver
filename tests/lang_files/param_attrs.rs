#![feature(c_variadic)] // Added for completeness

fn _fun(#[allow(unused_variables)] a: u32, _b: u32) {}

struct S {}
impl S {
    fn _inherent_method(#[cfg(any())] &self, _a: u32) {}
}

trait T {
    fn _trait_method(&self, #[allow(unused_variables)] a: u32, _b: u32);
}
impl T for S {
    fn _trait_method(&self, #[allow(unused_variables)] a: u32, _b: u32) {}
}

#[rustfmt::skip]
extern "C" { fn _var_fun(_a: u32, #[allow(unused_variables)] ...); }

macro_rules! in_macro {
    () => {
        fn _more_fun(#[allow(unused_variables)] a: u32, _b: u32) {}
    };
}

in_macro!();

fn main() {
    let f1 = |#[allow(unused_variables)] x, _y| {};
    let f2 = |#[allow(unused_variables)] x: u32, _y| {};

    let a = 42;
    let b = 21;
    f1(a, b);
    f2(a, b);
}
