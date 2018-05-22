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

#[derive(Debug, Clone)]
enum StoredMemRegion {
    /// Memory is used and should never be allocated, stored as virtual address
    USED(Range<usize>),
    /// Memory does not fit in the initial kernel window and should be added later, stored by physical address
    HIGH(Range<usize>),
    /// Memory is used during boot but can be used after that, stored by virtual address
    BOOT(Range<usize>),
}

const MAX_REGIONS: usize = 8;
// TODO: build some kind of statically allocated array type out of this, but array types are
// currently bloody annoying to try and generalize and needs const generics (see issue #44580)
static mut MEM_REGIONS: [Option<StoredMemRegion>; MAX_REGIONS] = [None, None, None, None, None, None, None, None];

/// Global buddy allocator
static mut BUDDY: buddy::Buddy = buddy::Buddy::new();

fn add_mem_region(region: StoredMemRegion) -> bool {
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

/// Mark a region of virtual memory as already used
pub fn add_used_mem(range: [Range<usize>; 1]) {
    if !add_mem_region(StoredMemRegion::USED(range[0].clone())) {
        panic!("Failed to record used memory {:?}. Increase MAX_USED_MEM", range[0]);
    }
    print!(Info, "Marked region [{:x}..{:x}] as initially allocated", range[0].start, range[0].end);
}

/// Add memory by virtual address
pub unsafe fn add_mem(range: [Range<usize>; 1]) {
    assert!(unsafe{KERNEL_WINDOW.range_valid(range.clone())});
    // Provide to the buddy allocator
    print!(Info, "Adding usable memory region [{:x}..{:x}]", range[0].start, range[0].end);
    BUDDY.add(range[0].start, range[0].end - range[0].start);
}

/// Adds memory by physical address
///
/// This is a more general version of `add_mem` and allows for adding memory that is not yet
/// available to describe virtually
pub unsafe fn add_mem_physical(range: [Range<usize>; 1]) {
    unsafe {
        if let Some(vaddr) = KERNEL_WINDOW.paddr_to_vaddr_range(range.clone()) {
            add_mem(vaddr)
        } else {
            if !add_mem_region(StoredMemRegion::HIGH(range[0].clone())) {
                print!(Info, "Had to throw away memory region {:?} as it is not in kernel window and ran out of EXTRA_MEM slots. Consider increasing MAX_EXTRA_MEM", range[0]);
                return;
            }
        }
    }
}
