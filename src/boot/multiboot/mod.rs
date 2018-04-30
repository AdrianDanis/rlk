pub mod v1;

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
    mb1: v1::Header,
    mb2: Multiboot2Header,
}

#[link_section=".multiboot"]
#[used]
#[linkage="external"]
static MBHEADER: MultibootAlign = MultibootAlign{
    mb1: v1::Header::new(),
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

