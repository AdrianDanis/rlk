//! Interface for paddr<->vaddr translation

use core::ops::Range;

// TODO: paddr and vaddr types?
/// Generic trait for defining paddr<->vaddr translation
///
/// A translation understands what virtual address ranges are valid, and how to convert
/// between these virtual and their corresponding physical addresses. As a translation might
/// have the same physical memory viewable from different virtual addresses converting
/// between vaddr->paddr->vaddr is not guaranteed to be the identity function, however
/// conversations are expected to be stable such that vaddr->paddr->vaddr->paddr->vaddr
/// will keep producing the same paddr and vaddr (as long as paddr->vaddr is defined to
/// ever produce anything)
pub unsafe trait Translation {
    /// Check if a virtuall address range is valid
    ///
    /// It is a requirement that iff it has a vaddr->paddr translation
    fn range_valid(&self, range: Range<usize>) -> bool;
    /// Convert a virtual address to a physical address
    ///
    /// This must provide a translation for any range for which `range_valid` returns true
    /// and this should translate any valid vaddr.
    fn vaddr_to_paddr(&self, vaddr: usize) -> Option<usize> {
        self.vaddr_to_paddr_range(vaddr..vaddr+1).map(|x| x.start)
    }
    /// Convert a physical address to a virtual address
    ///
    /// In comparison to virtual addresses this does not have to return something even if
    /// there exists a vaddr->paddr translation for it. Specifically paddr_to_vaddr(vaddr_to_addr(x))
    /// can be None
    fn paddr_to_vaddr(&self, paddr: usize) -> Option<usize> {
        self.paddr_to_vaddr_range(paddr..paddr+1).map(|x| x.start)
    }
    /// Convert a virtual address range to a physical address range
    ///
    /// Compared to `vaddr_to_paddr` this ensures that the underlying physical address
    /// range is contiguous
    fn vaddr_to_paddr_range(&self, range: Range<usize>) -> Option<Range<usize>>;
    /// Convert a physical address range to a virtual address range
    ///
    /// Compared to `paddr_to_vaddr` this ensures that the final virtual address
    /// range is contiguous
    fn paddr_to_vaddr_range(&self, range:Range<usize>) -> Option<Range<usize>>;
}
