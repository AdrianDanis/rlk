//! Definitions for boot time vspaces

use vspace::Window;
use util::units::GB;
use util::Empty;
use core::ops::Range;
use core::marker::PhantomData;
use core::borrow::Borrow;

pub struct Init;

/// Start of kernel window
///
/// The first part of the kernel window is the 'low' portion, which eventually includes all but
/// the final 2gb of virtual address space. Initially it only contains 4gb of mappings
const KERNEL_BASE: usize = 0xffffff8000000000;
/// Start of the kernel image window
///
/// This represents the 'high' portion of the kernel window and is used to provide a place to
/// virtually link and execute the kernel from. The window itself is in two parts, the first 1gb
/// is the kernel itself and is initially mapped and the second gb is for any device mappings
const KERNEL_IMAGE_BASE: usize = 0xffffffff80000000;


/// Initial kernel window, which is only 4gb
const INIT: [Range<usize>; 1] = [KERNEL_BASE..KERNEL_BASE + 4*GB];
/// Kernel image window, which is the first 1gb from the image base
const INIT_IMAGE: [Range<usize>; 1] = [KERNEL_IMAGE_BASE..KERNEL_IMAGE_BASE + GB];

unsafe impl Window for Init {
    fn range_valid(&self, range: [Range<usize>; 1]) -> bool {
        INIT[0].contains(&range[0].start) && INIT[0].contains(&(range[0].end - 1)) ||
        INIT_IMAGE[0].contains(&range[0].start) && INIT_IMAGE[0].contains(&(range[0].end - 1))
    }
    fn vaddr_to_paddr(&self, vaddr: usize) -> Option<usize> {
        if INIT[0].contains(&vaddr) {
            Some(vaddr - INIT[0].start)
        } else if INIT_IMAGE[0].contains(&vaddr) {
            Some(vaddr - INIT_IMAGE[0].start)
        } else {
            None
        }
    }
}

impl Init {
    /// Construct the Init vspace window
    pub const fn make() -> Self {
        Init
    }
}

pub static INIT_WINDOW: Init = Init::make();
