#![allow(unused)]

use std::sync::Condvar; // not nested, should be ignored
use std::sync::{self, *}; // nested glob
use std::sync::{atomic::Ordering, Arc}; // nested multi-segment path
use std::sync::{atomic::{AtomicBool, AtomicI32}, Barrier}; // two levels of nesting
use std::sync::{Mutex, Once}; // nested single-segment path, should be ignored

fn main() {}
