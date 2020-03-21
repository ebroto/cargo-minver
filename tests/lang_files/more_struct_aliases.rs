#![allow(unused)]

struct A {}

impl A {
    fn fun(&self) {
        let _ = Self {};

        match self {
            Self {} => {},
        }
    }
}

trait B {
    type C;
}

fn fun<T: B<C = A>>(c: T::C) {
    let _ = T::C {};

    match c {
        T::C {} => {},
    }
}

fn main() {}
