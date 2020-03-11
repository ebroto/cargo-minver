#![allow(unused)]

use std::marker::PhantomData;

#[repr(transparent)]
struct A(u32);

#[repr(transparent)]
struct B {
    value: u32,
}

#[repr(transparent)]
struct C<T> {
    value: u32,
    marker: PhantomData<T>,
}

// should be ignored
struct X {
    value: u32,
}

fn main() {}
