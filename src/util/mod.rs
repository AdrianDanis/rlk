//! Place for util routines that haven't found a home yet

use core;
use core::ops::Range;
use core::mem::size_of;
use core::fmt;

/// A range type that has nice printability
///
/// This provides no useful methods for manipulation and is intended only for using as
/// a wrapper for formatting other ranges
pub struct PrintRange<T> {
    start: T,
    end: T,
}

impl<'a, F, T: From<usize>> From<&'a[F]> for PrintRange<T> {
    fn from(slice: &'a [F]) -> PrintRange<T> {
        PrintRange {
            start: T::from(slice.as_ptr() as usize),
            end: T::from( (slice.as_ptr() as usize) + size_of::<F>() * slice.len()),
        }
    }
}

impl<'a, F, T: From<usize>> From<&'a mut[F]> for PrintRange<T> {
    fn from(slice: &'a mut [F]) -> PrintRange<T> {
        PrintRange {
            start: T::from(slice.as_ptr() as usize),
            end: T::from( (slice.as_ptr() as usize) + size_of::<F>() * slice.len()),
        }
    }
}

impl<T: fmt::Display> fmt::Display for PrintRange<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "[{}..{}]", self.start, self.end)
    }
}

impl<T: fmt::LowerHex> fmt::LowerHex for PrintRange<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "[{:x}..{:x}]", self.start, self.end)
    }
}

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

/// Calculates the log base 2 of a number
///
/// # Safety
///
/// Passed value must not be zero
pub fn log2_usize(v: usize) -> u32 {
    assert!(v != 0);
    // find first set 1 and return this
    0usize.count_zeros() - v.leading_zeros() - 1
}

// TODO: make this generic instead of just being for usize
pub fn range_contains(parent: &[Range<usize>; 1], child: &[Range<usize>; 1]) -> bool {
    parent[0].contains(&child[0].start) && parent[0].contains(&(child[0].end - 1))
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
