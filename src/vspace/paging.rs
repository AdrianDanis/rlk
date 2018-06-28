use x86::bits64::paging::*;
use x86::shared::paging::VAddr;
use cpu::features::{Page1GB, NXE, PGE};
use state::{CPU_FEATURES, KERNEL_WINDOW};
use util::units::GB;
use vspace::*;
use cpu::MemoryType;
use alloc::boxed::Box;
use core::mem;

enum Rights {
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
enum PageSize {
    // 4KiB
    Small,
    // 2MiB
    Large,
    // 1GiB (if supported)
    Huge(Page1GB),
}

#[derive(Debug, Clone, Copy)]
struct Access {
    write: bool,
    user: bool,
    nxe: Option<NXE>,
}

impl From<Access> for PML4Entry {
    fn from(access: Access) -> PML4Entry {
        let mut entry = PML4Entry::empty();
        if access.write {
            entry.insert(PML4_RW);
        }
        if access.user {
            entry.insert(PML4_US);
        }
        if access.nxe.is_some() {
            entry.insert(PML4_XD);
        }
        entry
    }
}

impl From<Access> for PDPTEntry {
    fn from(access: Access) -> PDPTEntry {
        let mut entry = PDPTEntry::empty();
        if access.write {
            entry.insert(PDPT_RW);
        }
        if access.user {
            entry.insert(PDPT_US);
        }
        if access.nxe.is_some() {
            entry.insert(PDPT_XD);
        }
        entry
    }
}

impl Access {
    /// Default access permissions for a kernel paging structure
    ///
    /// Since a high level paging structure might want to contain a variety of permissions we default
    /// to writable and executable and leave further refinement to sub pages
    fn default_kernel_paging() -> Self {
        Self { write: true, user: false, nxe: None }
    }
}

#[derive(Debug, Clone, Copy)]
struct PageMapping {
    vaddr: usize,
    paddr: usize,
    pge: Option<PGE>,
    mt: MemoryType,
    size: PageSize,
    access: Access,
}

impl From<PageMapping> for PDPTEntry {
    fn from(mapping: PageMapping) -> PDPTEntry {
        if let PageSize::Huge(_) = mapping.size {
            let mut entry = PDPTEntry::new(PAddr::from_u64(mapping.paddr as u64), PDPTEntry::from(mapping.access) | PDPTEntry::from(mapping.mt) | PDPT_PS);
            if mapping.pge.is_some() {
                entry.insert(PDPT_G);
            }
            entry
        } else {
            panic!("Cannot create PDPT entry from non 1gb page mapping");
        }
    }
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
                access: Access{
                    write: false,
                    user: false,
                    nxe: None,
                },
                pge: None,
                mt: MemoryType::WB,
                size: size,
            }
        }
    }
    pub fn user(mut self) -> Self {
        self.internal.access.user = true;
        self.internal.pge = None;
        self
    }
    pub fn kernel(mut self) -> Self {
        self.internal.access.user = false;
        self.internal.pge = unsafe{CPU_FEATURES}.get_pge();
        self
    }
    pub fn write(mut self) -> Self {
        self.internal.access.write = true;
        self
    }
    pub fn read_only(mut self) -> Self {
        self.internal.access.write = false;
        self
    }
    pub fn no_execute(mut self) -> Self {
        self.internal.access.nxe = unsafe{CPU_FEATURES}.get_nxe();
        self
    }
    pub fn executable(mut self) -> Self {
        self.internal.access.nxe = None;
        self
    }
    pub fn finish(&self) -> PageMapping {
        self.internal
    }
}

#[repr(C, align(4096))]
struct PDPTWrap(PDPT);
assert_eq_size!(pdpt_page_size; PDPTWrap, [u8; 4096]);

impl Default for PDPTWrap {
    fn default() -> PDPTWrap {
        PDPTWrap{0: [PDPTEntry::empty(); 512]}
    }
}

impl PDPTWrap {
    fn make_entry(&self, access: Access) -> PML4Entry {
        PML4Entry::new(PAddr::from_u64(unsafe{KERNEL_WINDOW}.vaddr_to_paddr(self as *const PDPTWrap as usize).unwrap() as u64), PML4Entry::from(access) | PML4_P)
    }
    unsafe fn from_entry(entry: PML4Entry) -> &'static mut PDPTWrap {
        if !entry.is_present() {
            panic!("No PDPT entry in PML4");
        }
        let vaddr = KERNEL_WINDOW.paddr_to_vaddr(entry.get_address().as_u64() as usize).unwrap();
        mem::transmute(vaddr as *mut PDPTWrap)
    }
}

#[repr(C, align(4096))]
pub struct AS(PML4);
assert_eq_size!(as_page_size; AS, [u8; 4096]);

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

        let pml4ent = &self.0[pml4_index(VAddr::from_usize(mapping.vaddr))];
        let pdpt = PDPTWrap::from_entry(*pml4ent);
        let pdptent = &mut pdpt.0[pdpt_index(VAddr::from_usize(mapping.vaddr))];
        if pdptent.is_present() {
            panic!("Mapping already present in PDPT");
        }
        if let PageSize::Huge(page1gb) = mapping.size {
            *pdptent = PDPTEntry::from(mapping) | PDPT_P;
            return;
        }
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
        let pml4ent = &mut self.0[pml4_index(VAddr::from_usize(mapping.vaddr))];
        if !pml4ent.is_present() {
            let pdpt = box PDPTWrap::default();
            *pml4ent = pdpt.make_entry(Access::default_kernel_paging());
            Box::into_raw(pdpt);
        }
        if let PageSize::Huge(page1gb) = mapping.size {
            // all done
            return;
        }
        unimplemented!()
    }
    pub unsafe fn map_kernel_window(&mut self) {
        // currently assume 1gb pages
        let page1gb: Page1GB = unsafe{CPU_FEATURES}.get_page1gb().expect("Require 1GB page support");
        // create the guaranteed kernel mappings
        for gb in KERNEL_BASE_DEFAULT_RANGE.step_by(GB) {
            // as this is not the kernel image, no need for executable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_BASE - KERNEL_PHYS_BASE), PageSize::Huge(page1gb)).kernel().no_execute().write().finish();
            unsafe {
                self.ensure_mapping_entry(mapping);
                self.raw_map_page(mapping);
            }
        }
        // map in the kernel image
        for gb in KERNEL_IMAGE_RANGE.step_by(GB) {
            // unfortunately the data and bss is also here so we need this both executable and writable
            let mapping = PageMappingBuilder::new(gb, gb - (KERNEL_IMAGE_BASE - KERNEL_PHYS_BASE), PageSize::Huge(page1gb)).kernel().executable().write().finish();
            unsafe {
                self.ensure_mapping_entry(mapping);
                self.raw_map_page(mapping);
            }
        }
    }
}

impl Default for AS {
    fn default() -> AS {
        AS{0: [PML4Entry::empty(); 512]}
    }
}
