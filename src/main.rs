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

pub use panic::*;

use drivers::Serial;
use vspace::{Window};

struct NullAlloc;

#[global_allocator]
static ALLOCATOR: NullAlloc = NullAlloc;

unsafe impl alloc::alloc::GlobalAlloc for NullAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut core::alloc::Opaque {
        unimplemented!()
    }
    unsafe fn dealloc(&self, ptr: *mut core::alloc::Opaque, layout: core::alloc::Layout) {
        unimplemented!()
    }
}

struct Foo {
    c: u8,
    col: u8,
}

#[no_mangle]
pub extern "C" fn boot_system(arg1: usize, arg2: usize) -> ! {
    if arg1 as u32 == multiboot::SIGNATURE_EAX {
        boot::multiboot::v1::init(arg2);
    } else {
        panic!("Unknown boot style");
    }
    boot::cmdline::process();
    {
        let boot_window = unsafe{boot::vspace::Init::make()};
        let f: &'static mut Foo;
        f = unsafe{boot_window.declare_obj(0xb8000usize).unwrap()};
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

