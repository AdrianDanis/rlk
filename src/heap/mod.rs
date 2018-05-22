//! Heap allocation for the kernel

mod buddy;

use core::alloc::{Layout, Opaque};
use alloc::alloc::GlobalAlloc;
use core::ops::Range;
use state::KERNEL_WINDOW;

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

impl AllocProxy {
    pub const fn new() -> AllocProxy {
        AllocProxy {alloc_fn: alloc_error, dealloc_fn: dealloc_error }
    }
}

unsafe impl GlobalAlloc for AllocProxy {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        (self.alloc_fn)(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        (self.dealloc_fn)(ptr, layout)
    }
}

#[derive(Debug, Clone, Copy)]
enum RegionTag {
    /// Memory is used and should never be allocated
    USED,
    /// Memory does not fit in the initial kernel window and should be added later
    HIGH,
    /// Memory is used during boot but can be used after that
    BOOT,
}

const MAX_REGIONS: usize = 8;
const MAX_EXTRA_MEM: usize = 8;
// TODO: build some kind of statically allocated array type out of this, but array types are
// currently bloody annoying to try and generalize and needs const generics (see issue #44580)
static mut MEM_REGIONS: [Option<(Range<usize>, RegionTag)>; MAX_REGIONS] = [None, None, None, None, None, None, None, None];

fn add_mem_region(region: (Range<usize>, RegionTag)) -> bool {
    unsafe {
        for iter in MEM_REGIONS.iter_mut() {
            if iter.is_none() {
                (*iter) = Some(region);
                return true;
            }
        }
    }
    false
}

pub fn add_used_mem(range: [Range<usize>; 1]) {
    if !add_mem_region((range[0].clone(), RegionTag::USED)) {
        panic!("Failed to record used memory {:?}. Increase MAX_USED_MEM", range[0]);
    }
}

pub fn add_mem(range: [Range<usize>; 1]) {
    // Check if it is valid in the window
    unsafe {
        if (!KERNEL_WINDOW.range_valid(range.clone())) {
            if !add_mem_region((range[0].clone(), RegionTag::HIGH)) {
                print!(Info, "Had to throw away memory region {:?} as it is not in kernel window and ran out of EXTRA_MEM slots. Consider increasing MAX_EXTRA_MEM", range[0]);
                return;
            }
        }
    }
    // Provide to the buddy allocator
    unimplemented!()
}
