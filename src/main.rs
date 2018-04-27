#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(used)]
#![feature(linkage)]
#![no_std]
#![no_main]

extern crate rlibc;
//extern crate compiler_builtins;

#[repr(C,packed)]
struct MultibootHeader {
    pub magic: u32,
    pub flags: u32,
    pub checksum: u32,
    pub header_addr: u32,
    pub load_addr: u32,
    pub load_end_addr: u32,
    pub bss_end_addr: u32,
    pub entry_addr: u32,
    pub mode_type: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C,packed)]
struct Multiboot2Header {
    pub magic: u32,
    pub arch: u32,
    pub header_length: u32,
    pub checksum: u32,
    pub end_tag_type: u16,
    pub end_tag_flags: u16,
    pub end_tag_size: u32,
}

#[repr(C,align(8))]
struct MultibootAlign {
    mb1: MultibootHeader,
    mb2: Multiboot2Header,
}

#[link_section=".rodata"]
#[used]
#[linkage="external"]
#[export_name="MBHEADER"]
static MBHEADER: MultibootAlign = MultibootAlign{
    mb1: MultibootHeader {
        magic: 0x1BADB002u32,
        flags: 0,
        checksum: 0xFFFFFFFFu32 - 0x1BADB002u32 + 1,
        header_addr: 0,
        load_addr: 0,
        load_end_addr: 0,
        bss_end_addr: 0,
        entry_addr: 0,
        mode_type: 0,
        width: 0,
        height: 0,
        depth: 0,
    },
    mb2: Multiboot2Header {
        magic: 0xE85250D6u32,
        arch: 0,
        header_length: 24u32,
        checksum: 0xFFFFFFFFu32 - 0xe85250d6u32 + 1u32 - 24u32,
        end_tag_type: 0,
        end_tag_flags: 0,
        end_tag_size: 8,
    },
};

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

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32,
                               _column: u32) -> ! {
    loop {}
}
