//! Implementation of final kernel address space

use vspace::*;
use vspace::paging::*;
use cpu::features::Page1GB;
use state::{CPU_FEATURES, KERNEL_WINDOW};
use alloc::boxed::Box;
use cpu;
use heap;
use con;

struct KernelVSpace(AS);

impl Default for KernelVSpace {
    fn default() -> Self {
        KernelVSpace{0: AS::default()}
    }
}

unsafe impl Allocation for KernelVSpace {
    fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        unimplemented!()
    }
    fn reserve(&mut self, size: usize, align: usize) -> Option<usize> {
        unimplemented!()
    }
    fn fill(&mut self, base: usize, size: usize) -> Option<*mut u8> {
        unimplemented!()
    }
}

unsafe impl Translation for KernelVSpace {
    fn range_valid(&self, range: Range<usize>) -> bool {
        unimplemented!()
    }
    fn vaddr_to_paddr_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        unimplemented!()
    }
    fn paddr_to_vaddr_range(&self, range:Range<usize>) -> Option<Range<usize>> {
        unimplemented!()
    }
}

unsafe impl VSpace for KernelVSpace {}

impl KernelVSpace {
    unsafe fn map_kernel_window(&mut self) {
        // currently assume 1gb pages
        let page1gb: Page1GB = unsafe{CPU_FEATURES}.get_page1gb().expect("Require 1GB page support");
        // create the guaranteed kernel mappings
        for gb in KERNEL_BASE_DEFAULT_RANGE.step_by(GB) {
            // as this is not the kernel image, no need for executable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_BASE - KERNEL_PHYS_BASE), PageSize::Huge(page1gb)).kernel().no_execute().write().finish();
            unsafe {
                self.0.ensure_mapping_entry(mapping);
                self.0.raw_map_page(mapping);
            }
        }
        // map in the kernel image
        for gb in KERNEL_IMAGE_RANGE.step_by(GB) {
            // unfortunately the data and bss is also here so we need this both executable and writable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_IMAGE_BASE - KERNEL_PHYS_BASE), PageSize::Huge(page1gb)).kernel().executable().write().finish();
            unsafe {
                self.0.ensure_mapping_entry(mapping);
                self.0.raw_map_page(mapping);
            }
        }
    }
}

pub unsafe fn make_kernel_address_space() {
    // create kernel address space
    let kernel_as = Box::leak(box KernelVSpace::default());
    kernel_as.map_kernel_window();
    con::disable_physical_con();
    // enable address space
    let kernel_as_paddr = KERNEL_WINDOW.vaddr_to_paddr(&kernel_as.0 as *const AS as usize).unwrap();
    // Load CR3, this will invalidate all our translation information so there is nothing else
    // we need to do
    cpu::load_cr3(kernel_as_paddr, KERNEL_PCID, false);
    // update the KERNEL_WINDOW
    KERNEL_WINDOW = kernel_as;
    // tell the heap that we can use all the memory now?
    heap::enable_high_mem();
}

