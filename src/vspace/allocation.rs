use util::units::{KB, MB, GB};
use core::mem;

pub const PAGE_SIZE_4K: usize = 4 * KB;
pub const PAGE_SIZE_2M: usize = 2 * MB;
pub const PAGE_SIZE_1G: usize = GB;

/// Trait that defines allocation of a virtual address space
pub unsafe trait Allocation {
    /// Allocate a portion of the virtual address space
    ///
    /// If this returns a Some(x) then x is the base of a region of `size` and aligned to `align`
    ///
    /// This is allowed to fail if size and align are not suitable multiples
    fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8>;
    /// Reserve a portion of the virtual address space
    ///
    /// This returns a usize and not a *mut u8 (like `alloc`) as this only reserves the range
    /// and there *may* not be actual memory in the range, although there could be memory or
    /// any number of intermediate paging structures there
    ///
    /// Like `alloc1 this is allowed to fail if sign and align or not suitable
    fn reserve(&mut self, size: usize, align: usize) -> Option<usize>;
    /// Fill in memory (as it was from alloc)
    ///
    /// Like alloc, this is allowed to fail if align and base are not suitable multiples
    ///
    /// The returned pointer (if None was not returned) is guaranteed to be just `base` casted
    ///
    /// # Safety
    ///
    /// The function is unsafe as it is assumed that you received the range from reserve
    ///
    /// # Panics
    ///
    /// May panic if given a region that is not from reserve
    fn fill(&mut self, base: usize, size: usize) -> Option<*mut u8>;
}

/// Base trait for mappable pages
///
/// This trait is unsafe as implementing it declares that this can be treated as page to
/// mapping functions which could violate safety if incorrectly implemented
unsafe trait Page {
    fn base(&self) -> usize;
    fn size(&self) -> PageSize;
}

#[repr(C, align(4096))]
struct Page4K {
    inner: [u8; PAGE_SIZE_4K],
}

impl Page for Page4K {
    fn base(&self) -> usize {
        se

impl Default for Page4K {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}

#[repr(C, align(2097152))]
struct Page2M {
    inner: [u8; PAGE_SIZE_2M],
}

impl Default for Page2M {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}
