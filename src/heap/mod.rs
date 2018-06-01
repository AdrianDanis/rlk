//! Heap allocation for the kernel

mod buddy;

use core::alloc::{Layout, Opaque};
use alloc::alloc::GlobalAlloc;
use core::ops::Range;
use state::KERNEL_WINDOW;
use ::ALLOCATOR;
use util::{log2_usize, PrintRange};
use core::cmp::max;
use vspace::declare_slice;

pub struct AllocProxy {
    alloc_fn: unsafe fn(Layout) -> *mut Opaque,
    dealloc_fn: unsafe fn(*mut Opaque, Layout),
}

unsafe fn alloc_error(_layout: Layout) -> *mut Opaque {
    panic!("Allocation before allocator is set")
}

unsafe fn dealloc_error(_ptr: *mut Opaque, _layout: Layout) {
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

#[derive(Debug)]
enum StoredMemRegion {
    /// Memory is used and we have been given ownership of it
    USED(&'static mut [u8]),
    /// Memory does not fit in the initial kernel window and should be added later, stored by physical address
    HIGH(Range<usize>),
    /// Memory is used during boot but can be used after that
    #[allow(dead_code)]
    BOOT(&'static mut [u8]),
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
pub fn add_used_mem(mem: &'static mut [u8]) {
    let display = PrintRange::<usize>::from(mem as &[u8]);
    if !add_mem_region(StoredMemRegion::USED(mem)) {
        panic!("Failed to record used memory {:x}. Increase MAX_USED_MEM", display);
    }
    print!(Info, "Marked region {:x} as initially allocated", display);
}

/// Add a region of memory to the heap
///
/// This works by passing ownership of a slice of memory to the allocator. As a result this
/// function is not `unsafe` as, assuming type safety hasn't already been broken, the provided
/// memory is not used for anything else.
///
/// Generally you probably do not want to use this as it bypasses any splitting around used mem
/// since you *shouldn't* have the slice if the memory region is used, as a slice would have
/// already been created to mark it used. Use `add_mem` if you just have a virtual address range
pub fn add_mem_owned(mem: &'static mut [u8]) {
    // TODO: assert that we haven't passed a range that is initially allocated
    // Provide to the buddy allocator
    let display = PrintRange::<usize>::from(mem as &[u8]);
    print!(Info, "Adding usable memory region {:x}", display);
    unsafe {BUDDY.add(mem)}
}

/// Add a virtual address range of memory to the heap
///
/// # Safety
///
/// In passing the range your are claiming that the range of memory is either not owned,
/// and is safe to start using, or is owned and has already been passed to the heap as
/// a used memory region.
///
/// # Panics
///
/// Will panic if the memory provided is not deemed valid according to the current `KERNEL_WINDOW`
pub unsafe fn add_mem(range: Range<usize>) {
    for region in MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x| match x { StoredMemRegion::USED(range) => Some(range), _ => None })) {
        let start = region.as_ptr() as usize;
        let end = start + region.len();
        if range.end <= start || range.start >= end {
            // range is completely outside, nothing to be done
        } else {
            // see if we need to add an initial region
            if range.start < start {
                add_mem(range.start..start);
            }
            // see if we need to add a final region
            if range.end > end {
                add_mem(end..range.end);
            }
            return;
        }
    }
    // Range not already used, grab it from the KERNEL_WINDOW just to be sure
    if let Some(mem) = declare_slice(KERNEL_WINDOW, range.start, range.end - range.start) {
        add_mem_owned(mem);
    } else {
        panic!("Invalid memory range {:?} according to current KERNEL_WINDOW", range);
    }
}

unsafe fn heap_alloc(layout: Layout) -> *mut Opaque {
    let size = max(layout.align(), layout.size());
    match size.checked_next_power_of_two() {
        None => panic!("No power of two size for allocation"),
        Some(neat_size) => {
            let value = BUDDY.alloc(log2_usize(neat_size));
            if value.is_null() {
                panic!("Failed to allocate {} bytes for allocation with layout {:?}", neat_size, layout);
            }
            value
        }
    }
}

unsafe fn heap_dealloc(_ptr: *mut Opaque, _layout: Layout) {
    unimplemented!()
}

pub fn enable_heap() {
    print!(Debug, "Enabling kernel heap");
    unsafe {
        ALLOCATOR.alloc_fn = heap_alloc;
        ALLOCATOR.dealloc_fn = heap_dealloc;
    }
}

/// Adds memory by physical address
///
/// This is a more general version of `add_mem` and allows for adding memory that is not yet
/// available to describe virtually
pub unsafe fn add_mem_physical(range: Range<usize>) {
    if let Some(vaddr) = KERNEL_WINDOW.paddr_to_vaddr_range(range.clone()) {
        add_mem(vaddr)
    } else {
        if !add_mem_region(StoredMemRegion::HIGH(range.clone())) {
            print!(Info, "Had to throw away memory region {:?} as it is not in kernel window and ran out of EXTRA_MEM slots. Consider increasing MAX_EXTRA_MEM", range);
            return;
        }
    }
}
