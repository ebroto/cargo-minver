#![allow(unused)]

// From the tracking issue ( https://github.com/rust-lang/rust/issues/55607#issue-376680713 ):
//
// "Exhaustive list of features supported in const fn with #![feature(min_const_unsafe_fn)]:
//     ... internal compiler detail omitted ...
//     1. Calling const unsafe fn functions inside const fn functions inside an unsafe { ... } block.
//     2. Calling const unsafe fn functions inside const unsafe fn functions."

const unsafe fn foo() -> i32 {
    42
}

const fn bar() -> i32 {
    unsafe { foo() } // 1
}

const fn baz() -> i32 {
    const X: i32 = unsafe { foo() }; // 1 (check that it works when assigning to an intermediate variable)
    X
}

const unsafe fn quux() -> i32 {
    foo() // 2
}

struct S {}
impl S {
    const fn foo(&self) -> i32 {
        unsafe { foo() } // 1
    }

    const unsafe fn bar(&self) -> i32 {
        foo() // 2
    }

    const fn baz() -> i32 {
        unsafe { foo() } // 1
    }

    const unsafe fn quux() -> i32 {
        foo() // 2
    }
}

fn main() {}
