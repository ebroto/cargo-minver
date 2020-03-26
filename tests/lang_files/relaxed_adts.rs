#![allow(unused)]

struct TS(i32);
struct EmptyTS();

enum E {
    TV(i32),
    EmptyTV(),
}

struct S {
    val: i32,
}

enum F {
    SV { val: i32 },
}

fn main() {
    // Tuple struct

    let ts = TS(42);
    let ts = TS { ..ts };
    let ts = TS { 0: 42 };
    match ts {
        TS { 0: 42 } => {},
        TS { .. } => {},
    }

    let ets = EmptyTS {};
    match ets {
        EmptyTS {} => {},
    }

    // Tuple enum variant

    // NOTE: FRU is not allowed on struct-like enum variants
    let tv = E::TV { 0: 42 };
    let tv = E::EmptyTV {};
    match tv {
        E::TV { 0: 42 } => {},
        E::TV { .. } => {},
        E::EmptyTV {} => {},
    }

    // Ignored struct and struct-like enum variant

    let s = S { val: 42 };
    let s = S { ..s };
    match s {
        S { .. } => {},
    }

    let sv = F::SV { val: 42 };
    match sv {
        F::SV { .. } => {},
    }
}
