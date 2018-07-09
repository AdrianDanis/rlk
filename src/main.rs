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
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![feature(range_contains)]
#![feature(align_offset)]
#![feature(box_syntax)]
#![feature(panic_implementation)]
#![feature(panic_info_message)]
#![feature(iterator_step_by)]
#![feature(never_type)]
#![feature(untagged_unions)]
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
extern crate static_assertions;

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

fn boot_continued(_no_arg: ()) -> ! {
    // TODO: switch to non early cons
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    panic!("End of boot");
}

#[no_mangle]
pub extern "C" fn boot_system(arg1: usize, arg2: usize) -> ! {
    if arg1 as u32 == multiboot::SIGNATURE_EAX {
        boot::multiboot::v1::init(unsafe{&boot::state::STATE}, arg2);
    } else {
        panic!("Unknown boot style");
    }
    if !cpu::init() {
        panic!("Failed to init cpu");
    }
    print!(Info, "Switching to full kernel address space");
    unsafe {vspace::make_kernel_address_space(&mut boot::state::STATE)};
    unsafe {
        print!(Info, "Switching to proper kernel stack");
        let mut stack = vspace::Stack::new_kernel(&mut state::STATE.kernel_as).unwrap();
        stack.run_on_stack((), boot_continued)
    }
}

#[lang = "oom"]
#[no_mangle]
pub extern fn rust_oom() -> ! {
    panic!("oom");
}

