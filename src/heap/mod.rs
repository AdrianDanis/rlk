//! Heap allocation for the kernel

mod buddy;

use core::alloc::Layout;
use alloc::alloc::GlobalAlloc;
use core::ops::Range;
use ::ALLOCATOR;
use util::{log2_usize, PrintRange};
use core::cmp::max;
use vspace::declare_slice;
use boot::state::BootState;
use vspace::Translation;

pub struct AllocProxy {
    alloc_fn: unsafe fn(Layout) -> *mut u8,
    dealloc_fn: unsafe fn(*mut u8, Layout),
}

unsafe fn alloc_error(_layout: Layout) -> *mut u8 {
    panic!("Allocation before allocator is set")
}

unsafe fn dealloc_error(_ptr: *mut u8, _layout: Layout) {
    panic!("Deallocation before allocator is set")
}

impl AllocProxy {
    pub const fn new() -> AllocProxy {
        AllocProxy {alloc_fn: alloc_error, dealloc_fn: dealloc_error }
    }
}

unsafe impl GlobalAlloc for AllocProxy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (self.alloc_fn)(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
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

/// Mark a region of virtual memory as used during boot
pub fn add_boot_mem(mem: &'static mut [u8]) {
    let display = PrintRange::<usize>::from(mem as &[u8]);
    if !add_mem_region(StoredMemRegion::BOOT(mem)) {
        panic!("Failed to record boot memory {:x}. Increase MAX_USED_MEM", display);
    }
    print!(Info, "Marked region {:x} as boot memory", display);
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
/// Will panic if the memory provided is not deemed valid according to the provided translation
pub unsafe fn add_mem<'a, T: Translation + ?Sized>(translation: &'a T, range: Range<usize>) {
    for region in MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x| match x { StoredMemRegion::USED(range) | StoredMemRegion::BOOT(range) => Some(range), _ => None })) {
        let start = region.as_ptr() as usize;
        let end = start + region.len();
        if range.end <= start || range.start >= end {
            // range is completely outside, nothing to be done
        } else {
            // see if we need to add an initial region
            if range.start < start {
                add_mem(translation, range.start..start);
            }
            // see if we need to add a final region
            if range.end > end {
                add_mem(translation, end..range.end);
            }
            return;
        }
    }
    // Range not already used, grab it from the KERNEL_WINDOW just to be sure
    if let Some(mem) = declare_slice(translation, range.start, range.end - range.start) {
        add_mem_owned(mem);
    } else {
        panic!("Invalid memory range {:?} according to provided translation", range);
    }
}

unsafe fn heap_alloc(layout: Layout) -> *mut u8 {
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

unsafe fn heap_dealloc(_ptr: *mut u8, _layout: Layout) {
    unimplemented!()
}

pub fn enable_heap() {
    unsafe {
        // Count memory still in boot or high mem
        let boot_mem = MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x|
                if let StoredMemRegion::BOOT(slice) = x { Some(slice.len()) } else { None }
            )).fold(0, |acc, x| acc + x);
        let high_mem = MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x|
                if let StoredMemRegion::HIGH(range) = x { Some(range.end - range.start) } else { None }
            )).fold(0, |acc, x| acc + x);
        print!(Debug, "Enabling kernel heap: Still have {} bytes in boot mem and {} bytes in high mem", boot_mem ,high_mem);
        ALLOCATOR.alloc_fn = heap_alloc;
        ALLOCATOR.dealloc_fn = heap_dealloc;
    }
}

unsafe fn try_add_mem_physical<'a, T: Translation + ?Sized>(translation: &'a T, range: Range<usize>) -> bool {
    if let Some(vaddr) = translation.paddr_to_vaddr_range(range.clone()) {
        add_mem(translation, vaddr);
        true
    } else {
        false
    }
}

pub unsafe fn enable_high_mem<'a, T: Translation + ?Sized>(translation: &'a T) {
    let mut high = 0;
    for range in MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x|
            if let StoredMemRegion::HIGH(range) = x { Some(range) } else {None})) {
        if !try_add_mem_physical(translation, range.clone()) {
            panic!("High memory region {:?} still not in kernel window", range);
        }
        high += range.end - range.start;
    }
    let boot_mem = MEM_REGIONS.iter().filter_map(|x| x.as_ref().and_then(|x|
            if let StoredMemRegion::BOOT(slice) = x { Some(slice.len()) } else { None }
        )).fold(0, |acc, x| acc + x);
    print!(Debug, "Added {} bytes of high memory to kernel heap. Have {} bytes still in boot mem", high, boot_mem);
}

/// Adds memory by physical address
///
/// This is a more general version of `add_mem` and allows for adding memory that is not yet
/// available to describe virtually
pub unsafe fn add_mem_physical<'a, T: Translation + ?Sized>(translation: &'a T, range: Range<usize>) {
    if let Some(vaddr) = translation.paddr_to_vaddr_range(range.clone()) {
        add_mem(translation, vaddr)
    } else {
        if !add_mem_region(StoredMemRegion::HIGH(range.clone())) {
            print!(Info, "Had to throw away memory region {:?} as it is not in kernel window and ran out of EXTRA_MEM slots. Consider increasing MAX_EXTRA_MEM", range);
            return;
        }
    }
}
