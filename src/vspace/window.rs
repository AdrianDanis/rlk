// Thoughts on vspace windows

// Use windows with lifetimes for booting. Window into the physical address range has a
// lifetime of the window. Allocations from that window will therefore be dropped when
// the window ends. Allocations from the final kernel window have static. Allocations
// from the early window need some way to transfer to the final window, but this means
// they need an address change. is there a way to do that? 'create' a new object
// that happens to be the same as the original at a new address? Probably not, maybe
// a Copy needs to be done, but how to do that in place? I think Clone is sufficient
// as it implies the object can be 'memcpy'd to duplicate, so should also be safe
// to reinterpret at a new virtual address.

use core::ops::Range;
use core::mem::{align_of, size_of, transmute};
use core::slice;

// TODO: paddr and vaddr types?
/// Generic trait for a view (aka Window) that is a virtual address spaces
///
/// The window understands what virtual address ranges are valid, and how to convert
/// between these virtual and their corresponding physical addresses. As a window might
/// have the same physical memory viewable from different virtual addresses converting
/// between vaddr->paddr->vaddr is not guaranteed to be the identity function, however
/// conversations are expected to be stable such that vaddr->paddr->vaddr->paddr->vaddr
/// will keep producing the same paddr and vaddr
pub unsafe trait Window {
    /// Check if a range is valid
    fn range_valid(&self, range: Range<usize>) -> bool;
    /// Convert a virtual address to a physical address
    fn vaddr_to_paddr(&self, vaddr: usize) -> Option<usize> {
        self.vaddr_to_paddr_range(vaddr..vaddr+1).map(|x| x.start)
    }
    /// Convert a physical address tyo a virtual address
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
    /// Request an unused portion of the Window
    fn alloc_unused(&mut self, _size: usize, _align: usize) -> Option<usize> {
        None
    }
}

/// Declares that an object exists at this virtual address
///
/// Virtual addresses (for the kernel) are never allowed to go away and so the produced
/// reference has a static lifetime.
///
/// This is a module level function so that the Window trait is able to be turned into a
/// trait object.
///
/// # Safety
///
/// This is unsafe as even if the range is valid it still requires that a correctly
/// construct T lives inside that virtual address range and that you have not already
/// constructed an object in that range.
pub unsafe fn declare_obj<'a, T>(window: &'a Window, base_vaddr: usize) -> Option<&'static mut T> {
    if (base_vaddr % align_of::<T>()) == 0 && window.range_valid(base_vaddr..base_vaddr + size_of::<T>()) {
        Some(transmute(base_vaddr as *mut T))
    } else {
        None
    }
}

pub unsafe fn declare_slice<'a, T>(window: &'a Window, base_vaddr: usize, items: usize) -> Option<&'static mut [T]> {
    if (base_vaddr % align_of::<T>()) == 0 && window.range_valid(base_vaddr..base_vaddr + size_of::<T>() * items) {
        Some(slice::from_raw_parts_mut(base_vaddr as *mut T, items))
    } else {
        None
    }
}
