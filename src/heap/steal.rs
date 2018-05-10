//! Initial steal memory allocator for boot strapping

use core::ops::Range;
use alloc::linked_list::LinkedList;

// All memory is in terms of virtual addresses
static mut FREE_MEM: Option<LinkedList<Range<usize>>> = None;
static mut USED_MEM: Option<LinkedList<Range<usize>>> = None;
static mut BOOTSTRAP: [Range<usize>; 1] = [0..0];

