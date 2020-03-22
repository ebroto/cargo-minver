#![allow(unused)]

enum A {
    Struct { val: i32 },
    Tuple(i32),
    Unit,
}

impl A {
    fn fun() {
        let _ = Self::Struct { val: 42 };
        let _ = Self::Tuple(42);
        let s = Self::Unit;

        match s {
            Self::Struct { .. } => {},
            Self::Tuple(..) => {},
            Self::Unit => {},
        }
    }
}

type B = A;

fn main() {
    let _ = B::Struct { val: 42 };
    let _ = B::Tuple(42);
    let b = B::Unit;

    match b {
        B::Struct { .. } => {},
        B::Tuple(..) => {},
        B::Unit => {},
    }

    // Should be ignored

    let _ = A::Struct { val: 42 };
    let _ = A::Tuple(42);
    let a = A::Unit;

    match a {
        A::Struct { .. } => {},
        A::Tuple(..) => {},
        A::Unit => {},
    }
}
