#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![no_std]
#![no_main]

extern crate rlibc;
//extern crate compiler_builtins;

mod boot;
mod panic;
#[macro_use]
mod con;

pub use panic::*;

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    con::early_init("vga_80_25");
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    loop {}
}
