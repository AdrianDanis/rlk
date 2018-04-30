#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![no_std]
#![no_main]

extern crate rlibc;
//extern crate compiler_builtins;

mod boot;
mod panic;
#[macro_use]
mod con;
mod test;

use test::FooType;

pub use panic::*;

extern {
    static decls_section_begin: FooType;
    static decls_section_end: FooType;
}

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    con::early_init("vga_80_25");
    print!(Panic, "Panic");
    print!(Error, "Error");
    print!(Info, "Info");
    print!(Debug, "Debug");
    print!(Trace, "Trace");
    unsafe {
        let begin = &decls_section_begin as *const FooType;
        let end = &decls_section_end as *const FooType;
        let slice = core::slice::from_raw_parts(begin, end.offset_from(begin) as usize);
        for foo in slice.iter() {
            (foo.fn_ref)();
        }
    }
    loop {}
}
