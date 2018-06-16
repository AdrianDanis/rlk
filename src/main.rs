#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![feature(pattern)]
#![feature(associated_type_defaults)]
#![feature(alloc)]
#![feature(global_allocator)]
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![feature(range_contains)]
#![feature(align_offset)]
#![feature(box_syntax)]
#![feature(panic_implementation)]
#![feature(panic_info_message)]
#![feature(iterator_step_by)]
#![no_std]
#![no_main]
#![feature(plugin)]
#![plugin(interpolate_idents)]

extern crate rlibc;
extern crate multiboot;
extern crate x86;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bitfield;
extern crate alloc;
extern crate raw_cpuid;

#[macro_use]
pub mod decls;
#[macro_use]
pub mod con;

pub mod boot;
pub mod panic;
pub mod util;
pub mod drivers;
pub mod vspace;
pub mod heap;
pub mod state;
pub mod ip_collections;
pub mod cpu;

/// Allocator has to be defined in the root of the crate so we extern it here and actually declare in heap
#[global_allocator]
pub static mut ALLOCATOR: heap::AllocProxy = heap::AllocProxy::new();

#[no_mangle]
pub extern "C" fn boot_system(arg1: usize, arg2: usize) -> ! {
    if arg1 as u32 == multiboot::SIGNATURE_EAX {
        boot::multiboot::v1::init(arg2);
    } else {
        panic!("Unknown boot style");
    }
    if !cpu::init() {
        panic!("Failed to init cpu");
    }
    print!(Info, "Switching to full kernel address space");
    unsafe {vspace::make_kernel_address_space()};
    // TODO: switch to a new stack that is guarded and not in our boot memory
    print!(Info, "arg1 is {:x}", arg1);
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    panic!("End of boot");
}

#[lang = "oom"]
#[no_mangle]
pub extern fn rust_oom() -> ! {
    panic!("oom");
}

