//! Place for util routines that haven't found a home yet

use core;

pub fn split_first_str<'a, P: core::str::pattern::Pattern<'a>> (slice: &'a str, predicate: P) -> (&'a str, &'a str) {
    let mut iter = slice.splitn(2, predicate);
    match iter.next() {
        Some(n) =>
            match iter.next() {
                Some(r) => (n, r),
                None => (n, ""),
            },
        None => ("", ""),
    }
}
