#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![feature(pattern)]
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
mod util;

pub use panic::*;

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    let cmdline = "--earlycon=vga_80_25";
    cmdline.split_whitespace()
        .map(|x| util::split_first_str(x,"="))
        .filter_map(|(option, value)| if option.starts_with("--") { Some((&option[2..], value))} else { None })
        .for_each(|(option, value)|
            decls_iter!(CMDLine)
                .filter(|x| x.option == option)
                .for_each(|x| (x.f)(value))
        );
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    loop {}
}
