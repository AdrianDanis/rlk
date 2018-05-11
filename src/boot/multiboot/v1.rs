use multiboot::*;
use boot;

use core::{mem, slice};

const MAGIC: u32 = 0x1BADB002;

#[repr(C,packed)]
pub struct Header {
    magic: u32,
    flags: u32,
    checksum: u32,
    header_addr: u32,
    load_addr: u32,
    load_end_addr: u32,
    bss_end_addr: u32,
    entry_addr: u32,
    mode_type: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl Header {
    // Slightly awkward checksum generation to work around current limitations of rust `const fn`
    const fn with_checksum(candidate: Self) -> Self {
        Self {
            magic: candidate.magic,
            flags: candidate.flags,
            checksum: u32::max_value()
                - candidate.magic
                - candidate.flags
                - candidate.header_addr
                - candidate.load_addr
                - candidate.load_end_addr
                - candidate.bss_end_addr
                - candidate.entry_addr
                - candidate.mode_type
                - candidate.width
                - candidate.height
                - candidate.depth
                + 1,
            header_addr: candidate.header_addr,
            load_addr: candidate.load_addr,
            load_end_addr: candidate.load_end_addr,
            bss_end_addr: candidate.bss_end_addr,
            entry_addr: candidate.entry_addr,
            mode_type: candidate.mode_type,
            width: candidate.width,
            height: candidate.height,
            depth: candidate.depth,
        }
    }
    pub const fn new() -> Self {
        Self::with_checksum( Self {
            magic: MAGIC,
            flags: 0,
            checksum: 0,
            header_addr: 0,
            load_addr: 0,
            load_end_addr: 0,
            bss_end_addr: 0,
            entry_addr: 0,
            mode_type: 0,
            width: 0,
            height: 0,
            depth: 0,
        })
    }
}

// TODO: make this common and check that the region is valid for whatever the current kernel window is
fn paddr_to_slice<'a>(p: PAddr, sz: usize) -> Option<&'a [u8]> {
    unsafe {
        let ptr = mem::transmute(p);
        Some(slice::from_raw_parts(ptr, sz))
    }
}

pub fn init(mb: usize) {
    // Process cmdline as we want to get this done as soon as possible for earlycon
    let mb = unsafe{Multiboot::new(mb as PAddr, paddr_to_slice)}.unwrap();
    if let Some(x) = mb.command_line() {
        boot::cmdline::process(unsafe{mem::transmute(x)});
    }
    // Process memory map and initialize allocators

    // Now that we have an allocator set the cmdline to preserve it
    if let Some(x) = mb.command_line() {
        boot::cmdline::set(unsafe{mem::transmute(x)});
    }
}
