#![allow(unused)]

#[cfg_attr(all(), warn(nonstandard_style), allow(unused_attributes))]
fn fun() {}

#[rustfmt::skip]
#[cfg_attr(all(), )]
fn more_fun() {}

#[cfg_attr(all(), cfg_attr(all(), warn(nonstandard_style), allow(unused_attributes)))]
fn much_more_fun() {}

// should be ignored
#[cfg_attr(all(), warn(nonstandard_style))]
fn not_fun() {}

fn main() {}
