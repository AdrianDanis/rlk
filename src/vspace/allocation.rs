/// Trait that defines allocation of a virtual address space
pub unsafe trait Allocation {
    /// Allocate a portion of the virtual address space
    ///
    /// If this returns a Some(x) then x is the base of a region of `size` and aligned to `align`
    fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8>;
}
