fn main() {
    let val = &Some(42);
    match val {
        Some(_answer) => {},
        _ => {},
    }

    // should be ignored
    match val {
        &Some(_answer) => {},
        _ => {},
    }

    // should be ignored
    match *val {
        Some(_answer) => {},
        _ => {},
    }
}
