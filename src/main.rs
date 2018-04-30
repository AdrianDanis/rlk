#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![no_std]
#![no_main]
#![feature(plugin)]
#![plugin(interpolate_idents)]

extern crate rlibc;
//extern crate compiler_builtins;

#[macro_use]
mod decls;
#[macro_use]
mod con;

mod boot;
mod panic;

pub use panic::*;

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    decls_iter!(CMDLine)
        .filter(|x| x.option == "earlycon")
        .for_each(|x| (x.f)("vga_80_25"));
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    loop {}
}
