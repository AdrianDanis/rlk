//! Implementation of final kernel address space

use vspace::*;
use vspace::paging::*;
use cpu::features::Page1GB;
use state::CPU_FEATURES;
use alloc::boxed::Box;
use cpu;
use heap;
use con;
use core::ptr::Unique;
use boot::state::BootState;
use state::STATE;
use util::units::MB;

pub struct KernelVSpace {
    root: Unique<AS>,
    dynamic_free: Range<usize>,
}

impl Default for KernelVSpace {
    fn default() -> Self {
        KernelVSpace{root: unsafe{Unique::new_unchecked(Box::into_raw(box AS::default()))}, dynamic_free: KERNEL_DYNAMIC_RANGE}
    }
}

unsafe impl Allocation for KernelVSpace {
    fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        unimplemented!()
    }
    fn reserve(&mut self, size: usize, align: usize) -> Option<usize> {
        // Currently no reserve restrictions as we restrict in fill
        if let Some(base) = self.dynamic_free.start.checked_add((self.dynamic_free.start as *mut u8).align_offset(align)) {
            if let Some(top) = base.checked_add(size) {
                if top < self.dynamic_free.end {
                    self.dynamic_free.start = top;
                    return Some(base);
                }
            }
        }
        None
    }
    fn fill(&mut self, base: usize, size: usize) -> Option<*mut u8> {
        // for now we will only support filling 2Mb pages so we don't have to check structures
        if base % PAGE_SIZE_2M != 0 || size % PAGE_SIZE_2M != 0 {
            return None;
        }
        for mb in (base..base+size).step_by(PAGE_SIZE_2M) {
            // allocate a page and turn it into a raw
            
//            let mapping = PageMappingBuilder(mb, 
        }
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
    unsafe fn map_kernel_window<'a, T: Translation + ?Sized>(&mut self, translation: &'a T) {
        // currently assume 1gb pages
        let page1gb: Page1GB = unsafe{CPU_FEATURES}.get_page1gb().expect("Require 1GB page support");
        // create the guaranteed kernel mappings
        for gb in KERNEL_BASE_DEFAULT_RANGE.step_by(GB) {
            let page = Page::new_unchecked(gb, page1gb);
            // as this is not the kernel image, no need for executable
            let mapping = PageMappingBuilder::new_page(page, translation).unwrap().kernel().no_execute().write().finish();
            unsafe {
                self.root.as_mut().ensure_mapping_entry(translation, mapping.clone());
                self.root.as_mut().raw_map_page(translation, mapping);
            }
        }
        // map in the kernel image
        for gb in KERNEL_IMAGE_RANGE.step_by(GB) {
            // unfortunately the data and bss is also here so we need this both executable and writable
            let page = Page::new_unchecked(gb, page1gb);
            let mapping = PageMappingBuilder::new_page(page, translation).unwrap().kernel().executable().write().finish();
            unsafe {
                self.root.as_mut().ensure_mapping_entry(translation, mapping.clone());
                self.root.as_mut().raw_map_page(translation, mapping);
            }
        }
    }
}

pub unsafe fn make_kernel_address_space<'a, B: BootState>(state: &'a B) {
    // create kernel address space
    let mut kernel_as = KernelVSpace::default();
    kernel_as.map_kernel_window(state.get_kernel_as().as_translation_ref());
    con::disable_physical_con();
    // enable address space
    let kernel_as_paddr = state.get_kernel_as().vaddr_to_paddr(kernel_as.root.as_ptr() as usize).unwrap();
    // Load CR3, this will invalidate all our translation information so there is nothing else
    // we need to do
    cpu::load_cr3(kernel_as_paddr, KERNEL_PCID, false);
    STATE.kernel_as = kernel_as;
    // tell the heap that we can use all the memory now?
    heap::enable_high_mem(STATE.kernel_as.as_translation_ref());
}

