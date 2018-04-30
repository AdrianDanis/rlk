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

mod boot;
mod panic;
    #[macro_use]
mod con;
#[macro_use]
mod decls;

pub use panic::*;

fn hello_world(_s: &str) {
    print!(Info, "hello world");
}

make_cmdline_decl!("foo", hello_world, test);

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    con::early_init("vga_80_25");
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    decls_iter!(CMDLine)
        .for_each(|x| (x.f)("hello"));
    loop {}
}
