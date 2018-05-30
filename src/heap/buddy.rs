//! Buddy memory allocator

use core::ops::Range;
use core::cmp::{min, max, Ordering};
use core::ptr::NonNull;
use core::alloc::Opaque;
use core::slice;
use util::log2_usize;
use boot::cmdline::option_is_true;
use ip_collections::LinkedList;

//TODO: stop using base+len everywhere and start using slices of [u8]

#[derive(Debug, Clone, PartialEq)]
struct Node {
    /// Address
    ///
    /// Record the address of the node in the node so that we can perform our desired ordering
    addr: usize,
    /// Size of the node
    ///
    /// We knew the order of the node when we found it in a list, but we're storing
    /// in free memory so this doesn't hurt
    order: u32,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        // Only performing ordering by the address, we do not care about the size
        self.addr.partial_cmp(&rhs.addr)
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
    pools: [LinkedList<Node>; NUM_ORDERS],
}

impl Buddy {
    pub const fn new() -> Buddy {
        Buddy { pools: [LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(),
                        LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new(),
                        LinkedList::new(), LinkedList::new(), LinkedList::new(), LinkedList::new()]
        }
    }
    fn fill_level(&mut self, bits: u32) {
        assert!(bits >= MIN_ORDER);
        let index = (bits - MIN_ORDER) as usize;
        assert!(self.pools[index].is_empty());
        if bits == MAX_ORDER {
            // no way to get more of these unless some get freed
            return;
        }
        // alloc from a higher level
        // TODO: have a wrapping alloc that returns a (slice,node) so we do less nonsense in here
        let node = self.alloc(bits + 1);
        // see if we got something
        if !node.is_null() {
            // insert it in two pieces. don't use free to make sure we don't immediately coalesce it back up
            let addr_a = node as usize;
            let addr_b = addr_a + (1 << bits);
            let node_a = Node {addr: addr_a, order: bits};
            let node_b = Node {addr: addr_b, order: bits};
            unsafe {
                let slice_a = slice::from_raw_parts_mut(addr_a as *mut u8, 1 << bits);
                let slice_b = slice::from_raw_parts_mut(addr_b as *mut u8, 1 << bits);
                self.pools[index].insert((slice_b, node_b));
                self.pools[index].insert((slice_a, node_a));
            }
        }
    }
    pub fn alloc(&mut self, mut bits: u32) -> *mut Opaque {
        if bits < MIN_ORDER {
            bits = MIN_ORDER;
        }
        if bits > MAX_ORDER {
            panic!("Requested allocation for {} bits, larger than maximum {} bits");
        }
        let index = (bits - MIN_ORDER) as usize;
        // see if we need to fill the layer
        if self.pools[index].is_empty() {
            print!(Trace, "Refilling level {} before allocation", bits);
            self.fill_level(bits);
        }
        match self.pools[index].pop_front() {
            None => 0 as *mut Opaque,
            Some((slice, node)) => {
                assert!(node.addr == slice.as_ptr() as usize);
                assert!(node.order == bits);
                slice.as_mut_ptr() as *mut Opaque
            },
        }
    }
    fn free(&mut self, mem: &'static mut[u8]) {
        let base = mem.as_ptr() as usize;
        let len = mem.len();
        // Should always be size aligned
        assert!((len % len) == 0);
        assert!(base != 0);
        if (heap_debug_free_enabled()) {
            // walk all the nodes, check for any overlaps etc
            // TODO
        }
        let size = log2_usize(mem.len());
        if (size < MIN_ORDER || size > MAX_ORDER) {
            panic!("Free of object with invalid size");
        }
        let index = (size - MIN_ORDER) as usize;
        if size != MAX_ORDER {
            // see if we would be node a or node b from a split node by checking our alignment
            let other_base =
                if (base as *const u8).align_offset(len * 2) == 0 {
                    // As we are aligned to the next higher size, we are node a and want b
                    base + len
                } else {
                    // We are not aligned, so we are b
                    base - len
                };
            if let Some((slice, node)) = self.pools[index].remove(Node {addr: other_base, order: size}) {
                // Insert the larger node instead
                return self.free(unsafe{slice::from_raw_parts_mut(min(slice.as_ptr() as usize, base) as *mut u8, len * 2)});
            }
        }
        let node = Node {addr: base, order: size};
        let slice = unsafe{slice::from_raw_parts_mut(base as *mut u8, len)};
        self.pools[index].insert((slice, node));
    }
    /// Add new memory to the allocator
    ///
    /// Memory has no requirements on size or alignment and will be split into multiple pieces as required
    ///
    pub fn add(&mut self, mut mem: &'static mut [u8]) {
        let mut base = mem.as_ptr() as usize;
        let mut len = mem.len();
        // track how much memor we waste due to alignment
        let mut wasted: usize = 0;
        // pointer::align_offset behaves in completely insane ways and fails to determine of offset to align a pointer that is
        // completely alignable. so we calculate an offset ourself
        let offset = min(if (base % (1 << MIN_ORDER)) == 0 { 0 } else { (1 << MIN_ORDER) - (base % (1 << MIN_ORDER)) }, len);
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
                self.free(unsafe{slice::from_raw_parts_mut(base as *mut u8, node_size)});
            }
            base += node_size;
            len -= node_size;
        }
        if wasted != 0 {
            print!(Debug, "Threw away {} bytes of memory due to bad alignments", wasted);
        }
    }
}
