#![allow(unused)]

#[repr(transparent)]
enum A {
    Variant(u32),
}

#[repr(transparent)]
enum B {
    Variant { value: u32, nothing: () },
}

#[repr(transparent)]
enum C<T> {
    Variant(T, ()),
}

// should be ignored
enum X {
    Variant(u32),
}

fn main() {}
