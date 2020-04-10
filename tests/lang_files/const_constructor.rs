#![allow(unused)]

struct S(i32);

enum E {
    V(i32),
}

const _: S = S(42);
const _: E = E::V(42);

const fn fun() {
    let _ = S(42);
    let _ = E::V(42);
}

fn main() {}
