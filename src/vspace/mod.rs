//! General vspace definitions

mod window;
mod paging;

pub use self::paging::make_kernel_address_space;
pub use self::window::{Window, declare_obj, declare_slice};

use util::units::GB;
use core::ops::Range;

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

/// Physical address of KERNEL_BASE and KERNEL_IMAGE_BASE
pub const KERNEL_PHYS_BASE: usize = 0x0;
