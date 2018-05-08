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

/// Empty trait with no methods
///
/// Useful if you want to guarantee that an object has some trait that can be converted into
/// a trait object. Only useful for a marker lifetime as the trait object itself will have
/// no functionality
pub trait Empty {}

pub mod units {
    pub const KB: usize = 1024;
    pub const MB: usize = KB * 1024;
    pub const GB: usize = MB * 1024;
}
