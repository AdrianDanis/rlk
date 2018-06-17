//! Implementation of final kernel address space

use vspace::{AS, KERNEL_PCID};
use state::KERNEL_WINDOW;
use alloc::boxed::Box;
use cpu;

pub unsafe fn make_kernel_address_space() {
    // create kernel address space
    let kernel_as = Box::into_raw(box AS::default());
    (*kernel_as).map_kernel_window();
    // inform any early cons that we are switching
    // enable address space
    let kernel_as_paddr = KERNEL_WINDOW.vaddr_to_paddr(kernel_as as usize).unwrap();
    // Load CR3, this will invalidate all our translation information so there is nothing else
    // we need to do
    cpu::load_cr3(kernel_as_paddr, KERNEL_PCID, false);
    // update the KERNEL_WINDOW
    // tell the heap that we can use all the memory now?
    unimplemented!()
}

