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
#[macro_use]
mod link_decls;

pub use panic::*;

fn hello_world(_s: &str) {
    print!(Info, "hello world");
}

static REAL_THING: link_decls::Type =
    link_decls::Type::CMDLine(link_decls::CMDLine{option:"foo",f: hello_world});

#[link_section=".decls"]
#[used]
#[linkage="external"]
static THING: link_decls::RawDecl = link_decls::RawDecl {
    nonce: 42,
    decl: &REAL_THING,
};

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
