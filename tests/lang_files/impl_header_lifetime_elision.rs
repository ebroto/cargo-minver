#![allow(unused)]

struct S<'a> {
    value: &'a i32,
}
impl S<'_> {} // struct impl, lifetime elided

trait T {}
impl T for S<'_> {} // trait impl, struct lifetime elided
impl T for (&str, &str) {} // trait impl, ref lifetimes elided

fn main() {}
