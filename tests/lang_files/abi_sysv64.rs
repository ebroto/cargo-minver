#![allow(unused)]

extern "sysv64" fn fun() {}

extern "sysv64" {
    fn more_fun();
}

fn main() {}
