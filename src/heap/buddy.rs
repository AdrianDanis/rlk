//! Buddy memory allocator

use core::ops::Range;
use core::cmp::min;
use alloc::linked_list::LinkedList;
use util::log2_usize;

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
    unsafe fn free(&mut self, base: usize, len: usize) {
        unimplemented!()
    }
    /// Add new memory to the allocator
    ///
    /// Memory has no requirements on size or alignment and will be split into multiple pieces as required
    ///
    /// # Safety
    ///
    /// Provided virtual address range must not be used by any existing object or already provided to the allocator
    pub unsafe fn add(&mut self, mut base: usize, mut len: usize) {
        // track how much memor we waste due to alignment
        let mut wasted: usize = 0;
        // convert base into a correctly aligned pointer of our MIN_ORDER
        let offset = min((base as *mut Node).align_offset(1 << MIN_ORDER), len);
        base+=offset;
        len-=offset;
        wasted+=offset;
        while len > 0 {
            // determine next power of 2 size
            let mut node_size = 1<< min(log2_usize(len), log2_usize(base));
            if node_size > len {
                node_size = len;
                wasted+=node_size;
            } else {
                self.free(base, node_size);
            }
            base += node_size;
            len -= node_size;
        }
        if wasted != 0 {
            print!(Debug, "Threw away {} bytes of memory due to bad alignments", wasted);
        }
    }
}
