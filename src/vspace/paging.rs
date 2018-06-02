use x86::bits64::paging::*;
use cpu::features::{Page1GB, NXE, Global};
use state::CPU_FEATURES;
use util::units::GB;
use vspace::*;

enum Rights {
    Read,
    ReadWrite,
}

enum PageSize {
    // 4KiB
    Small,
    // 2MiB
    Large,
    // 1GiB (if supported)
    Huge(Page1GB),
}

struct PageMapping {
    vaddr: usize,
    paddr: usize,
    write: bool,
    user: bool,
    nxe: Option<NXE>,
    global: Option<Global>,
    mt: MemoryType,
    size: PageSize;
}

struct PageMappingBuilder {
    internal: PageMapping,
}

impl PageMappingBuilder {
    pub fn new(vaddr: usize, paddr: usize, size: PageSize) -> Self {
        PageMappingBuilder {
            internal: PageMapping {
                vaddr: vaddr,
                paddr: paddr,
                write: false,
                user: false,
                nxe: None,
                global: None,
                mt: MemoryType::WB,
                size: size,
            }
        }
    }
    pub fn user(mut self) -> Self {
        self.internal.user = true;
        self.internal.global = None;
        self
    }
    pub fn kernel(mut self) -> Self {
        self.internal.user = false;
        self.internal.global = unsafe{CPU_FEATURES}.global();
        self
    }
    pub fn write(mut self) -> Self {
        self.internal.write = true;
        self
    }
    pub fn read_only(mut self) -> Self {
        self.internal.write = false;
        self
    }
    pub fn no_execute(mut self) -> Self {
        self.internal.nxe = unsafe{CPU_FEATURES.get_nxe()}};
        self
    }
    pub fn executable(mut self) -> Self {
        self.internal.nxe = None;
        self
    }
    pub fn finish(&self) -> PageMapping {
        self.internal
    }
}

#[repr(C, packed)]
struct AS(PML4);

impl AS {
    /// Maps a page without performing consistency updates
    ///
    /// This function assumes you will perform any TLB + structures cache updates on
    /// this and any other core as required. Also assumes that the target page is mapped
    ///
    /// No memory allocations are performed and all paging structures must already exist
    ///
    /// # Panics
    ///
    /// If the desired virtual address is already marked as present then 
    unsafe fn raw_map_page(&mut self, mapping: PageMapping) {
        unimplemented!()
    }
    /// Ensure paging structures exist to support mapping
    ///
    /// This allocates structures from the heap and so is only intended to be used for
    /// creating the kernel window during bootup.
    ///
    /// # Panics
    ///
    /// Will panic if the entry cannot be created due to a frame existing at a higher level.
    /// i.e. if trying to ensure an entry for a 4K frame but there is already a 2M frame
    /// covering the region, preventing the necessary page table from being created.
    unsafe fn ensure_mapping_entry(&mut self, mapping: PageMapping) {
        unimplemented!()
    }
    fn map_kernel_window(&mut self) {
        // currently assume 1gb pages
        let _gb: Page1GB = unsafe{CPU_FEATURES}.page1gb().expect("Require 1GB page support");
        // create the guaranteed kernel mappings
        for gb in KERNEL_BASE_DEFAULT_RANGE.step_by(GB) {
            // as this is not the kernel image, no need for executable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_BASE - KERNEL_PHYS_BASE), PageSize::Huge(Page1GB)).kernel().no_execute().write().finish();
            self.ensure_mapping_entry(mapping);
            self.raw_map_page(mapping);
        }
        // map in the kernel image
        for gb in KERNEL_IMAGE_RANGE.step_by(GB) {
            // unfortunately the data and bss is also here so we need this both executable and writable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_IMAGE_BASE - KERNEL_PHYS_BASE), PageSize::Huge(Page1GB)).kernel().executable().write().finish();
            self.ensure_mapping_entry(mapping);
            self.raw_map_page(mapping);
        }
    }
}

impl Default for AS {
    fn default() -> AS {
        AS{0: [PML4Entry::empty(); 512]}
    }
}

pub unsafe fn make_kernel_address_space() {
    // create kernel address space
    let mut kernel_as = box AS::default();
    kernel_as.map_kernel_window();
    // inform any early cons that we are switching
    // enable address space
    // tell the heap that we can use all the memory now?
}
