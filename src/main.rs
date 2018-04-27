#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![no_std]
#![no_main]

extern crate rlibc;
//extern crate compiler_builtins;

mod boot;
mod panic;

pub use panic::*;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn boot_system() -> ! {
    let vga_buffer = 0xb8000 as *const u8 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
