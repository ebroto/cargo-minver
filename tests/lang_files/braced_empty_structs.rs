#![allow(unused)]

struct A {}

struct X; // should be ignored

// should be ignored
struct S {
    pub value: u32,
}

enum B {
    C {},
    Y,                // should be ignored
    Z { value: u32 }, // should be ignored
}

fn main() {
    // struct, expression position
    let a = A {};
    // struct, pattern position
    match a {
        A {} => {},
    }

    // struct, expression position using functional record update syntax
    let a = A { ..a };
    // struct, pattern position using rest
    match a {
        A { .. } => {},
    }

    // enum, expression position
    let c = B::C {};
    // enum, pattern position
    match c {
        B::C {} => {},
        _ => {},
    }
    // enum, pattern position with rest
    match c {
        B::C { .. } => {},
        B::Z { .. } => {}, // should be ignored
        _ => {},
    }

    // should be ignored
    let s = S { value: 42 };
    match s {
        S { .. } => {},
    }
}
