//! Initial steal memory allocator for boot strapping

use core::ops::Range;
use alloc::linked_list::LinkedList;

/// Structure for creating linked list of ranges in free memory (similar to a classic frame table)
struct LinkedFreemem {
    /// Virtual address range in the kernel window for this memory
    ///
    /// This instance of `LinkedFreemem` sits at the start of this range
    range: [Range<usize>; 1],
    next: Option<&'static mut LinkedFreemem>,
    prev: Option<&'static mut LinkedFreemem>,
}

const MAX_USED_MEM: usize = 8;

struct Heap {
    /// Freemem list
    free_mem: Option<&'static mut LinkedFreemem>,
    /// Regions of memory that may already be in use and must never be added to the freemem list
    used_mem: [Option<[Range<usize>;1]>; MAX_USED_MEM],
}

static mut HEAP: Heap = Heap { free_mem: None, used_mem: [None, None, None, None, None, None, None, None]};

impl LinkedFreemem {
}

/// Initialize the steal memory allocator and enable it
///
/// Requires an initial block of free memory to get things going
unsafe fn init() {
}
