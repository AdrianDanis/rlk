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

pub unsafe trait Window {
    /// Declares that an object exists at this virtual address
    ///
    /// Virtual addresses (for the kernel) are never allowed to go away and so the produced
    /// reference has a static lifetime.
    ///
    /// # Safety
    ///
    /// This is unsafe as even if the range is valid it still requires that a correctly
    /// construct T lives inside that virtual address range and that you have not already
    /// constructed an object in that range.
    unsafe fn declare_obj<'a, T>(&self, base_vaddr: usize) -> Option<&'static mut T> {
        if (base_vaddr % align_of::<T>()) == 0 && self.range_valid([base_vaddr..base_vaddr + size_of::<T>()]) {
            Some(transmute(base_vaddr as *mut T))
        } else {
            None
        }
    }
    /// Check if a range is valid
    fn range_valid(&self, range: [Range<usize>; 1]) -> bool;
}
