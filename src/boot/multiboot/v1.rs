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

pub fn init(mb: usize) {
    
}
