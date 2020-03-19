#![allow(unused)]

struct TupleStruct(i32);

impl TupleStruct {
    fn new() -> Self {
        Self(42)
    }

    fn pattern(&self) {
        match self {
            Self(42) => {},
            _ => {},
        }
    }

    fn map(&self, val: Option<i32>) -> Option<Self> {
        val.map(Self)
    }
}

struct UnitStruct;

impl UnitStruct {
    fn new() -> Self {
        Self
    }

    fn pattern(&self) {
        match self {
            Self => {},
            _ => {},
        }
    }
}

fn main() {}
