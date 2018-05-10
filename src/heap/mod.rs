//! Heap allocation for the kernel

use core::alloc::{Layout, Opaque};
use alloc::alloc::GlobalAlloc;

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
