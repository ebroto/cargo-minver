use std::ops::AddAssign;

struct I(i32);

impl AddAssign<i32> for I {
    fn add_assign(&mut self, _: i32) {}
}

fn main() {
    let mut i = I(42);
    i += 21;

    // should be ignored
    let mut _x = 42;
    _x += 21;
}
