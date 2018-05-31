//! Definitions for boot time vspaces

use vspace::Window;
use util::units::GB;
use util::range_contains;
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

/// Physical address of KERNEL_BASE and KERNEL_IMAGE_BASE
const KERNEL_PHYS_BASE: usize = 0x0;

/// Initial kernel window, which is only 4gb
const INIT: Range<usize> = KERNEL_BASE..KERNEL_BASE + 4*GB;
/// Kernel image window, which is the first 1gb from the image base
const INIT_IMAGE: Range<usize> = KERNEL_IMAGE_BASE..KERNEL_IMAGE_BASE + GB;

unsafe impl Window for Init {
    fn range_valid(&self, range: Range<usize>) -> bool {
        range_contains(&INIT, &range) || range_contains(&INIT_IMAGE, &range)
    }
    fn vaddr_to_paddr_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        if range_contains(&INIT, &range) {
            Some(range.start - INIT.start..range.end - INIT.start)
        } else if range_contains(&INIT_IMAGE, &range) {
            Some(range.start - INIT_IMAGE.start..range.end - INIT_IMAGE.start)
        } else {
            None
        }
    }
    fn paddr_to_vaddr_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        self.vaddr_to_paddr_range(INIT_IMAGE)
            .and_then(|x| if range_contains(&x, &range) { Some(range.start + INIT_IMAGE.start..range.end + INIT_IMAGE.start) } else { None})
    }
}

impl Init {
    /// Construct the Init vspace window
    pub const fn make() -> Self {
        Init
    }
}

pub static INIT_WINDOW: Init = Init::make();
