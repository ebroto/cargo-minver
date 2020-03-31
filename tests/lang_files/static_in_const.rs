#![allow(unused)]

static FOO: &str = "foo";
static BAR: &'static str = "bar"; // should be ignored

const BAZ: &str = "baz";
const QUX: &'static str = "qux"; // should be ignored

fn main() {}
