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
