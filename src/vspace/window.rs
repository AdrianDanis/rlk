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

use core::ptr::Unique;
use core::marker::PhantomData;
use core::ops::Range;
use core::mem::{align_of, size_of};
use core::borrow::Borrow;
use util::Empty;

/// Window allocated box
///
/// Has similar semantics to a regular Box except that it will 'free' by simply dropping
/// the value and forgetting about it as windows are not allocators and do not track
/// what objects exist
pub struct WBox<'a, T: ?Sized> {
    pub ptr: Unique<T>,
    borrow: &'a Empty,
}

pub unsafe trait Window where Self: Empty + Sized {
    /// Declares that an object exists at this virtual address
    ///
    /// A reference to the object is potentially produced that has a lifetime for
    /// as long as this window.
    ///
    /// # Safety
    ///
    /// This is unsafe as even if the range is valid it still requires that a correctly
    /// construct T lives inside that virtual address range and that you have not already
    /// constructed an object in that range.
    unsafe fn declare_obj<'a, T>(&self, base_vaddr: usize) -> Option<WBox<T>> {
        if (base_vaddr % align_of::<T>()) == 0 && self.range_valid([base_vaddr..base_vaddr + size_of::<T>()]) {
            Some(WBox{ptr: Unique::new_unchecked(base_vaddr as *mut T), borrow: self as &Empty})
        } else {
            None
        }
    }
    /// Check if a range is valid
    fn range_valid(&self, range: [Range<usize>; 1]) -> bool;
}
