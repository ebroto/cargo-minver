#[must_use]
fn fun() -> i32 {
    42
}

fn not_fun() -> i32 {
    21
}

struct S {}
impl S {
    #[must_use]
    fn meth(&self) -> i32 {
        42
    }

    fn ignore_me(&self) -> i32 {
        21
    }
}

fn main() {
    let _a = fun();
    let _a = not_fun();

    let s = S {};
    let _b = s.meth();
    let _b = s.ignore_me();
}
