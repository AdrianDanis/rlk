//! Buddy memory allocator

use core::ops::Range;
use core::cmp::{min, max};
use core::ptr::NonNull;
use core::alloc::Opaque;
use alloc::linked_list::LinkedList;
use util::log2_usize;
use boot::cmdline::option_is_true;

struct Node {
    /// Size of the node
    ///
    /// We knew the order of the node when we found it in a list, but we're storing
    /// in free memory so this doesn't hurt
    order: u32,
    /// Next node in the series
    next: Option<NonNull<Node>>,
    /// Previous node in the series
    prev: Option<NonNull<Node>>,
}

impl Node {
    fn addr(&self) -> usize {
        self as *const Node as usize
    }
}

/// Smallest allocation is 128 bytes
const MIN_ORDER: u32 = 7;
/// Largest allocation is 1GiB
const MAX_ORDER: u32 = 30;

const NUM_ORDERS: usize = MAX_ORDER as usize - MIN_ORDER as usize + 1;

/// Debug flag for doing expensive assertion checking of frees
///
/// This can be enabled with --heap_debug_free=on cmdline option
static mut HEAP_DEBUG_FREE: bool = false;

fn heap_debug_free(debug_free: &str) {
    if (option_is_true(debug_free)) {
        unsafe {
            HEAP_DEBUG_FREE = true;
        }
    }
}

fn heap_debug_free_enabled() -> bool {
    unsafe {HEAP_DEBUG_FREE}
}

make_cmdline_decl!("heap_debug_free", heap_debug_free, HEAP_DEBUG_FREE);

pub struct Buddy {
    heads: [Option<NonNull<Node>>; NUM_ORDERS],
}

impl Buddy {
    pub const fn new() -> Buddy {
        Buddy { heads: [None, None, None, None, None, None, None, None, None, None,
                        None, None, None, None, None, None, None, None, None, None,
                        None, None, None, None]
        }
    }
    fn insert_sorted(mut node: NonNull<Node>, head: &mut Option<NonNull<Node>>) {
        unsafe {
            match *head {
                None => *head = Some(node),
                Some(head_node) => {
                    let mut current = head_node;
                    // see if we should insert before the head
                    if node.as_ref().addr() < current.as_ref().addr() {
                        node.as_mut().next = *head;
                        match head {
                            None => (),
                            Some(mut x) => x.as_mut().prev = Some(node),
                        };
                        *head = Some(node);
                    } else {
                        // loop to find the node we should insert *after*
                        // at this point we know we have a higher address than current so
                        // we want to move next if we would still be less than current->next
                        loop {
                            // See if we should go to the next
                            let next = match current.as_mut().next {
                                None => None,
                                Some(x) => if node.as_ref().addr() < x.as_ref().addr() { None } else { Some(x) },
                            };
                            // Update current or leave the loop
                            match next {
                                None => break,
                                Some(x) => current = x,
                            }
                        }
                        // insert after current
                        node.as_mut().prev = Some(current);
                        node.as_mut().next = current.as_ref().next;
                        current.as_mut().next = Some(node);
                        match node.as_mut().next {
                            None => (),
                            Some(mut x) => x.as_mut().prev = Some(current),
                        }
                    }
                }
            }
        }
    }
    fn fill_level(&mut self, bits: u32) {
        assert!(self.heads[bits as usize].is_none());
        if bits == MAX_ORDER {
            // no way to get more of these unless some get freed
            return;
        }
        // alloc from a higher level
        let node = self.alloc(bits + 1);
        // see if we got something
        if !node.is_null() {
            // insert it in two pieces using insert_sorted instead of free to make sure we don't immediately
            // coalesce it back up
            let nodea = node as *mut Node;
            let nodeb = (node as usize + 1 << bits) as *mut Node;
            unsafe {
                *nodea = Node {order: bits, next: None, prev: None};
                *nodeb = Node {order: bits, next: None, prev: None};
            }
            Buddy::insert_sorted(NonNull::new(nodeb).unwrap(), &mut self.heads[bits as usize]);
            Buddy::insert_sorted(NonNull::new(nodea).unwrap(), &mut self.heads[bits as usize]);
        }
    }
    pub fn alloc(&mut self, mut bits: u32) -> *mut Opaque {
        if bits < MIN_ORDER {
            bits = MIN_ORDER;
        }
        if bits > MAX_ORDER {
            panic!("Requested allocation for {} bits, larger than maximum {} bits");
        }
        // see if we need to fill the layer
        if self.heads[bits as usize].is_none() {
            print!(Trace, "Refilling level {} before allocation", bits);
            self.fill_level(bits);
        }
        match self.heads[bits as usize] {
            None => 0 as *mut Opaque,
            Some(node) => {
                unsafe {
                    self.heads[bits as usize] = node.as_ref().next;
                    node.as_ref().addr() as *mut Opaque
                }
            },
        }
    }
    unsafe fn free(&mut self, base: usize, len: usize) {
        // Should always be size aligned
        assert!((base % len) == 0);
        assert!(base != 0);
        if (heap_debug_free_enabled()) {
            // walk all the nodes, check for any overlaps etc
            // TODO
        }
        let size = log2_usize(len);
        if (size < MIN_ORDER || size > MAX_ORDER) {
            panic!("Free of object with invalid size");
        }
        let index = size - MIN_ORDER;
        let node = base as *mut Node;
        *node = Node {order: size, next: None, prev: None};

        Buddy::insert_sorted(NonNull::new(node).unwrap(), &mut self.heads[index as usize]);
        // see if node can be combined
        // TODO
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
            let mut node_size = 1<< min(min(log2_usize(len), base.trailing_zeros()), MAX_ORDER);
            if node_size > len || node_size <  1 << MIN_ORDER {
                print!(Trace, "Throwing {} bytes at {:x}", node_size, base);
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
