use multiboot;

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
    pub const fn new() -> Self {
        Self {
            magic: MAGIC,
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
        }
    }
}

pub fn init(mb: usize) {
    
}
