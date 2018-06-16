//! Definitions for boot time vspaces

use vspace::*;
use util::range_contains;
use core::ops::Range;

pub struct Init;

unsafe impl Translation for Init {
    fn range_valid(&self, range: Range<usize>) -> bool {
        range_contains(&KERNEL_BASE_DEFAULT_RANGE, &range) || range_contains(&KERNEL_IMAGE_RANGE, &range)
    }
    fn vaddr_to_paddr_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        if range_contains(&KERNEL_BASE_DEFAULT_RANGE, &range) {
            Some(range.start - (KERNEL_BASE_DEFAULT_RANGE.start - KERNEL_PHYS_BASE)..range.end - (KERNEL_BASE_DEFAULT_RANGE.start - KERNEL_PHYS_BASE))
        } else if range_contains(&KERNEL_IMAGE_RANGE, &range) {
            Some(range.start - (KERNEL_IMAGE_RANGE.start - KERNEL_PHYS_BASE)..range.end - (KERNEL_IMAGE_RANGE.start - KERNEL_PHYS_BASE))
        } else {
            None
        }
    }
    fn paddr_to_vaddr_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        self.vaddr_to_paddr_range(KERNEL_IMAGE_RANGE)
            .and_then(|x| if range_contains(&x, &range) { Some(range.start - KERNEL_PHYS_BASE + KERNEL_IMAGE_RANGE.start..range.end - KERNEL_PHYS_BASE + KERNEL_IMAGE_RANGE.start) } else { None})
    }
}

impl Init {
    /// Construct the Init vspace window
    pub const fn make() -> Self {
        Init
    }
}

pub static INIT_WINDOW: Init = Init::make();
