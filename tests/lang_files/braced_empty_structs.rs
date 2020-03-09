struct A {}

enum B {
    C {},
}

fn main() {
    let a = A {};
    match a {
        A {} => {},
    }

    let b = B::C {};
    match b {
        B::C {} => {},
    }
}
