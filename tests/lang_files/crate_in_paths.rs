pub struct A {
    val: u32,
}

pub trait B {
    type X;
}

struct C(u32);

fn main() {
    crate::module::fun();

    let c = crate::C(21);
    match c {
        crate::C(42) => {},
        _ => {},
    }

    let a = crate::A { val: 42 };
    if let crate::A { val: 21 } = a {}
}

mod module {
    use crate::A;

    impl crate::B for A {
        type X = u32;
    }

    pub fn fun() {
        let _a: A;
    }

    pub fn _fun2(_b: impl crate::B) -> <A as crate::B>::X {
        42
    }
}
