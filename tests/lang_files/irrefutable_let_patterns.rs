#![allow(irrefutable_let_patterns)]

fn main() {
    if let _ = 42 {}
    while let _ = 42 {
        break;
    }

    // should be ignored
    let a = 42;
    if let 42 = a {}
    while let 21 = a {}
}
