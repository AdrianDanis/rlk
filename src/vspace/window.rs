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

/// Window allocated box
///
/// Has similar semantics to a regular Box except that it will 'free' by simply dropping
/// the value and forgetting about it as windows are not allocators and do not track
/// what objects exist
pub struct WBox<T: ?Sized>(Unique<T>);

/// Defines a virtual address range bound to a particular window
pub struct VRange<'a, T: Window<'a> + 'a> {
    /// Virtual address range
    range: Range<usize>,
    /// Phantom data that 'binds' this vrange to a specific Window in the type system
    marker: PhantomData<&'a T>,
}

impl<'a, T: Window<'a> + 'a> VRange<'a, T> {
    fn new(range: Range<usize>) -> VRange<'a, T> {
        VRange{range: range, marker: PhantomData}
    }
}

pub struct VObj<'a, W:Window<'a> + 'a, T> {
    /// Underlying range
    ///
    /// This must always be constructed with a base,len and alignment that is valid for type T
    range: VRange<'a, W>,
    /// Marker to consume T
    marker: PhantomData<T>,
}

impl<'a, W:Window<'a> + 'a, T> VObj<'a, W, T> {
    fn from_range(range: VRange<'a, W>) -> Option<VObj<'a, W, T>> {
        if (range.range.start % align_of::<T>()) == 0 && range.range.start + size_of::<T>() <= range.range.end {
            Some(VObj{range: range, marker: PhantomData})
        } else {
            None
        }
    }
}

pub unsafe trait Window<'a> where Self: Sized {
    /// Declares that an object exists at this virtual address
    ///
    /// A reference to the object is potentially produced that has a lifetime for
    /// as long as this window.
    ///
    /// # Safety
    ///
    /// This is unsafe as even if the VRange is valid it still requires that a correctly
    /// construct T lives inside that virtual address range and that you have not already
    /// constructed an object in that range.
    unsafe fn declare<T>(vobj: VObj<'a, Self, T>) -> WBox<T> {
        WBox(Unique::new_unchecked(vobj.range.range.start as *mut T))
    }
    /// Check if a range is valid
    fn range_valid(range: [Range<usize>; 1]) -> bool;
    /// Construct a virtual address range
    fn make_range(range: [Range<usize>; 1]) -> Option<VRange<'a, Self>> {
        if Self::range_valid(range.clone()) {
            Some(VRange::new(range[0].clone()))
        } else {
            None
        }
    }
    /// Construct a virtual object
    fn make_obj<T: Sized>(base: usize) -> Option<VObj<'a, Self, T>> {
        Self::make_range([base..base + size_of::<T>()])
            .and_then(|x| VObj::from_range(x))
    }
    fn wrap<T: Sized>(&self, base: usize) -> Option<VObj<'a, Self, T>> {
        Self::make_obj(base)
    }
}
