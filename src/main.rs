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
#[macro_use]
extern crate alloc;
//extern crate compiler_builtins;

#[macro_use]
mod decls;
#[macro_use]
mod con;

mod boot;
mod panic;
mod util;
mod drivers;
mod vspace;
mod heap;
mod state;

pub use panic::*;

use drivers::Serial;
use vspace::{Window, declare_obj};

struct Foo {
    c: u8,
    col: u8,
}

/// Allocator has to be defined in the root of the crate so we extern it here and actually declare in heap
#[global_allocator]
static ALLOCATOR: heap::AllocProxy = heap::AllocProxy::new();

#[no_mangle]
pub extern "C" fn boot_system(arg1: usize, arg2: usize) -> ! {
    if arg1 as u32 == multiboot::SIGNATURE_EAX {
        boot::multiboot::v1::init(arg2);
    } else {
        panic!("Unknown boot style");
    }
    {
        let f: &'static mut Foo;
        f = unsafe{declare_obj(state::KERNEL_WINDOW, 0xb8000usize).unwrap()};
        print!(Info, "using ptr");
        unsafe{f.c = 0};
    }
    print!(Info, "arg1 is {:x}", arg1);
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    loop {}
}

#[lang = "oom"]
#[no_mangle]
pub extern fn rust_oom() -> ! {
    panic!("oom");
}

