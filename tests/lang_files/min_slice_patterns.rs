fn main() {
    let x = [1, 2, 3, 4, 5];

    match x {
        [1, 2, _, _, _] => {},
        [_, _, 3, 4, 5] => {},
        _ => {},
    }
}
