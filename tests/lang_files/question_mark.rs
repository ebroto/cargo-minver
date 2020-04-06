type Result<T> = std::result::Result<T, &'static dyn std::error::Error>;

fn foo() -> Result<i32> {
    Ok(42)
}

fn bar() -> Result<i32> {
    let val = foo()?;
    Ok(val)
}

fn main() {
    let _ = bar();
}
