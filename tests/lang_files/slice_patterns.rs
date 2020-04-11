#![allow(unused)]

fn main() {
    let x = [1, 2, 3, 4, 5];
    match x {
        [1, 2, ..] => {},
        [1, .., 5] => {},
        [.., 4, 5] => {},
        _ => {},
    }

    let x = [1, 2, 3, 4, 5];
    match x {
        [xs @ .., 4, 5] => {},
        [1, xs @ .., 5] => {},
        [1, 2, xs @ ..] => {},
        _ => {},
    }
}