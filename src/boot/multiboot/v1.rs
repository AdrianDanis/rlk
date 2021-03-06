use multiboot::*;
use boot;
use heap;
use vspace;
use boot::state::BootState;

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

fn paddr_to_slice<'a, 'b, B: BootState>(state: &'b B, p: PAddr, sz: usize) -> Option<&'a [u8]> {
    unsafe {
        state.get_kernel_as().paddr_to_vaddr_range(p as usize..p as usize + sz)
            .map(|x| slice::from_raw_parts(mem::transmute(x.start), sz))
    }
}

pub fn init<'a, B: BootState>(state: &'a B, mb: usize) {
    // Process cmdline as we want to get this done as soon as possible for earlycon
    let mb = unsafe{Multiboot::new(mb as PAddr, |p, sz| paddr_to_slice(state, p, sz))}.unwrap();
    let cmdline = mb.command_line();
    if let Some(x) = cmdline {
        boot::cmdline::process(unsafe{mem::transmute(x)});
    }
    // Process memory map and initialize allocators
    // First mark as reserved any common data
    boot::mark_image_mem(state);
    // Now mark anything additional from multiboot specifically that we want to still have
    // *after* we have enabled the heap later on
    if let Some(x) = cmdline {
        // Can unwrap as we already know the commandline is valid as we originally retrieved it from
        // paddr_to_slice, which checked the KERNEL_WINDOW. We are recreating it, despite this being
        // undefined behaviour, as we need to give a mutable slice to add_mem_owned.
        unsafe{heap::add_used_mem(vspace::declare_slice(state.get_kernel_as(), x.as_ptr() as usize, x.len()).unwrap())}
    }

    // Add free memory
    if let Some(memiter) = mb.memory_regions() {
        print!(Info, "Parsing regions");
        unsafe {
            memiter.filter(|x| x.memory_type() == MemoryType::Available)
                .for_each(|x| heap::add_mem_physical(state.get_kernel_as().as_translation_ref(), x.base_address() as usize..x.base_address() as usize+x.length() as usize));
        }
    } else {
        print!(Error, "Found no memory regions");
    }

    // Enable the heap
    heap::enable_heap();

    // Now that we have an allocator set the cmdline to preserve it
    if let Some(x) = cmdline {
        boot::cmdline::set(x);
    }
}
