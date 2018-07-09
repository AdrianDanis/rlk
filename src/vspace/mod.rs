//! General vspace definitions

mod paging;
mod translation;
mod allocation;
mod kernel;
mod stack;

pub use self::paging::AS;
pub use self::translation::{AsTranslation, Translation};
pub use self::allocation::*;
pub use self::kernel::make_kernel_address_space;
pub use self::stack::Stack;
pub use self::kernel::KernelVSpace;

use util::units::{MB, GB};
use core::ops::Range;
use core::slice;
use core::mem::{align_of, size_of, transmute};

pub unsafe trait VSpace: Translation + Allocation {}

/// Start of kernel window
///
/// The first part of the kernel window is the 'low' portion, which eventually includes all but
/// the final 2gb of virtual address space. Initially it only contains 4gb of mappings
pub const KERNEL_BASE: usize = 0xffffff8000000000;

/// Default (and initial) range of the kernel window
///
/// The final kernel window will only create mappings where memory actually exists, except for this range
/// that is guaranteed to always be mapped. We guarantee it is always mapped as it is also the range
/// that is mapped by the boot code prior to setting up the actual kernel window, and we make assumptions
/// on nothing getting unmapped from here.
pub const KERNEL_BASE_DEFAULT_RANGE: Range<usize> = KERNEL_BASE..KERNEL_BASE + 4*GB;

/// Start of the kernel image window
///
/// This represents the 'high' portion of the kernel window and is used to provide a place to
/// virtually link and execute the kernel from. The window itself is in two parts, the first 1gb
/// is the kernel itself and is initially mapped and the second gb is for any device mappings
pub const KERNEL_IMAGE_BASE: usize = 0xffffffff80000000;

/// Kernel image virtual address range
///
/// The kernel image is given a 1GB slot to live in. This is purely to hold the kernel binary
pub const KERNEL_IMAGE_RANGE: Range<usize> = KERNEL_IMAGE_BASE .. KERNEL_IMAGE_BASE + GB;

/// Kernel dynamic virtual address range
///
/// This address range contains dynamically allocated non contiguous vaddr->paddr ranges.
/// Used for kernel devices, stack, etc
///
/// Note that we cannot actually define the full range that we want due to overflow, so we arbitrarily
/// make this range 1 byte smaller, effectively forbidding the last page from being used
pub const KERNEL_DYNAMIC_RANGE: Range<usize> = KERNEL_IMAGE_BASE + GB..KERNEL_IMAGE_BASE + (2 * GB - 1);

/// Physical address of KERNEL_BASE and KERNEL_IMAGE_BASE
pub const KERNEL_PHYS_BASE: usize = 0x0;

/// Physical address the kernel is initially loaded to
pub const KERNEL_PADDR_LOAD: usize = MB;

/// Kernel PCID
pub const KERNEL_PCID: u16 = 1;

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
pub unsafe fn declare_obj<'a, T, W: AsTranslation + ?Sized>(window: &'a W, base_vaddr: usize) -> Option<&'static mut T> {
    if (base_vaddr % align_of::<T>()) == 0 && window.as_translation_ref().range_valid(base_vaddr..base_vaddr + size_of::<T>()) {
        Some(transmute(base_vaddr as *mut T))
    } else {
        None
    }
}

pub unsafe fn declare_slice<'a, T, W: AsTranslation + ?Sized>(window: &'a W, base_vaddr: usize, items: usize) -> Option<&'static mut [T]> {
    if (base_vaddr % align_of::<T>()) == 0 && window.as_translation_ref().range_valid(base_vaddr..base_vaddr + size_of::<T>() * items) {
        Some(slice::from_raw_parts_mut(base_vaddr as *mut T, items))
    } else {
        None
    }
}

