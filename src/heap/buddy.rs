//! Buddy memory allocator

use core::ops::Range;
use alloc::linked_list::LinkedList;

struct Node {
    /// Size of the node
    ///
    /// We knew the order of the node when we found it in a list, but we're storing
    /// in free memory so this doesn't hurt
    order: u32,
    /// Next node in the series
    next: Option<&'static mut Node>,
    /// Previous node in the series
    prev: Option<&'static mut Node>,
}

/// Smallest allocation is 128 bytes
const MIN_ORDER: u32 = 7;
/// Largest allocation is 1GiB
const MAX_ORDER: u32 = 30;

const NUM_ORDERS: usize = MAX_ORDER as usize - MIN_ORDER as usize + 1;

pub struct Buddy {
    heads: [Option<Node>; NUM_ORDERS],
}

impl Buddy {
    pub const fn new() -> Buddy {
        Buddy { heads: [None, None, None, None, None, None, None, None, None, None,
                        None, None, None, None, None, None, None, None, None, None,
                        None, None, None, None]
        }
    }
}
