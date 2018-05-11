//! Heap allocation for the kernel

mod buddy;

use core::alloc::{Layout, Opaque};
use alloc::alloc::GlobalAlloc;
use core::ops::Range;

pub struct AllocProxy {
    alloc_fn: unsafe fn(Layout) -> *mut Opaque,
    dealloc_fn: unsafe fn(*mut Opaque, Layout),
}

unsafe fn alloc_error(layout: Layout) -> *mut Opaque {
    panic!("Allocation before allocator is set")
}

unsafe fn dealloc_error(ptr: *mut Opaque, layout: Layout) {
    panic!("Deallocation before allocator is set")
}

#[global_allocator]
static mut ALLOCATOR: AllocProxy = AllocProxy {alloc_fn: alloc_error, dealloc_fn: dealloc_error};

unsafe impl GlobalAlloc for AllocProxy {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        (self.alloc_fn)(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        (self.dealloc_fn)(ptr, layout)
    }
}

const MAX_USED_MEM: usize = 8;
const MAX_EXTRA_MEM: usize = 8;
/// Regions of memory that we need to exclude from memory we are given
static mut USED_MEM: [Option<Range<usize>>; MAX_USED_MEM] = [None, None, None, None, None, None, None, None];
/// Regions of memory that could not be added early due to not being in the initial kernel window
static mut EXTRA_MEM: [Option<Range<usize>>; MAX_EXTRA_MEM] = [None, None, None, None, None, None, None, None];

pub fn add_used_mem(range: [Range<usize>; 1]) {
    unsafe {
        for iter in USED_MEM.iter_mut() {
            if iter.is_none() {
                (*iter) = Some(range[0].clone());
                return;
            }
        }
    }
    panic!("Failed to record used memory. Increase MAX_USED_MEM")
}
