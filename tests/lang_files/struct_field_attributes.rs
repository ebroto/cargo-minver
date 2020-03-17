#![allow(unused, deprecated)]

struct A {
    #[cfg(any())]
    value: u32,
}

struct B {
    #[deprecated]
    other: i32,
}

fn main() {
    let a = A {
        #[cfg(any())]
        value: 42,
    };
    let b = B {
        #[deprecated]
        other: 21,
    };

    let A {
        #[cfg(any())]
        value,
    } = a;
    let B {
        #[deprecated]
        other,
    } = b;
}
