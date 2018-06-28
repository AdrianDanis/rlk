//! Implementation of final kernel address space

use vspace::*;
use vspace::paging::*;
use cpu::features::Page1GB;
use state::{CPU_FEATURES, KERNEL_WINDOW};
use alloc::boxed::Box;
use cpu;

struct KernelVSpace(AS);

impl Default for KernelVSpace {
    fn default() -> Self {
        KernelVSpace{0: AS::default()}
    }
}

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
    let kernel_as = Box::into_raw(box KernelVSpace::default());
    (*kernel_as).map_kernel_window();
    // TODO: inform any early cons that we are switching
    // enable address space
    // TODO: pull out 'AS' portion of kernel_as
    let kernel_as_paddr = KERNEL_WINDOW.vaddr_to_paddr(kernel_as as usize).unwrap();
    // Load CR3, this will invalidate all our translation information so there is nothing else
    // we need to do
    cpu::load_cr3(kernel_as_paddr, KERNEL_PCID, false);
    // update the KERNEL_WINDOW
    // tell the heap that we can use all the memory now?
    unimplemented!()
}

