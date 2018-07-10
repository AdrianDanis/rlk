use x86::bits64::paging::*;
use x86::shared::paging::VAddr;
use cpu::features::{Page1GB, NXE, PGE};
use state::CPU_FEATURES;
use util::units::GB;
use vspace::*;
use cpu::MemoryType;
use alloc::boxed::Box;
use core::mem;
use core::marker::PhantomData;

pub enum Rights {
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
pub enum PageSize {
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

#[derive(Debug, Copy)]
pub struct PageMapping<S: PageLevel> {
    vaddr: usize,
    paddr: usize,
    pge: Option<PGE>,
    mt: MemoryType,
    access: Access,
    marker: PhantomData<S>,
}

// Have to manually implement clone for some reason.... compiler bug?
impl<S: PageLevel> Clone for PageMapping<S> {
    fn clone(&self) -> Self {
        PageMapping {
            vaddr: self.vaddr,
            paddr: self.paddr,
            pge: self.pge,
            mt: self.mt,
            access: self.access,
            marker: self.marker
        }
    }
}

impl From<PageMapping<Page1G>> for PDPTEntry {
    fn from(mapping: PageMapping<Page1G>) -> PDPTEntry {
        let mut entry = PDPTEntry::new(PAddr::from_u64(mapping.paddr as u64), PDPTEntry::from(mapping.access) | PDPTEntry::from(mapping.mt) | PDPT_PS);
        if mapping.pge.is_some() {
            entry.insert(PDPT_G);
        }
        entry
    }
}

pub struct PageMappingBuilder<S: PageLevel> {
    internal: PageMapping<S>,
}

impl<S: PageLevel> PageMappingBuilder<S> {
    pub fn new_page<'a, T: Translation + ?Sized>(page: Page<S>, translation: &'a T) -> Option<Self> {
        translation.vaddr_to_paddr_range(page.range())
            .map(|paddr_range|
                PageMappingBuilder {
                    internal: PageMapping {
                        vaddr: page.range().start,
                        paddr: paddr_range.start,
                        access: Access {write: false, user: false, nxe: None},
                        pge: None,
                        mt: MemoryType::WB,
                        marker: PhantomData,
                    },
                }
            )
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
    pub fn finish(&self) -> PageMapping<S> {
        self.internal.clone()
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
    fn make_entry<'a, T: Translation + ?Sized>(&self, translation: &'a T, access: Access) -> PML4Entry {
        PML4Entry::new(PAddr::from_u64(translation.vaddr_to_paddr(self as *const PDPTWrap as usize).unwrap() as u64), PML4Entry::from(access) | PML4_P)
    }
    unsafe fn from_entry<'a, T: Translation + ?Sized>(entry: PML4Entry, translation: &'a T) -> &'static mut PDPTWrap {
        if !entry.is_present() {
            panic!("No PDPT entry in PML4");
        }
        let vaddr = translation.paddr_to_vaddr(entry.get_address().as_u64() as usize).unwrap();
        mem::transmute(vaddr as *mut PDPTWrap)
    }
}

#[repr(C, align(4096))]
pub struct AS(PML4);
assert_eq_size!(as_page_size; AS, [u8; 4096]);

// TODO: generalize mapping ops over all the levels
pub unsafe trait ASMappingOps<S: PageLevel> {
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
    unsafe fn raw_map_page<'a, T: Translation + ?Sized>(&mut self, translation: &'a T, mapping: PageMapping<S>);
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
    unsafe fn ensure_mapping_entry<'a, T: Translation + ?Sized>(&mut self, translation: &'a T, mapping: PageMapping<S>);
}

unsafe impl ASMappingOps<Page1G> for AS {
    unsafe fn raw_map_page<'a, T: Translation + ?Sized>(&mut self, translation: &'a T, mapping: PageMapping<Page1G>) {

        let pml4ent = &self.0[pml4_index(VAddr::from_usize(mapping.vaddr))];
        let pdpt = PDPTWrap::from_entry(*pml4ent, translation);
        let pdptent = &mut pdpt.0[pdpt_index(VAddr::from_usize(mapping.vaddr))];
        if pdptent.is_present() {
            panic!("Mapping already present in PDPT");
        }
        *pdptent = PDPTEntry::from(mapping) | PDPT_P;
    }
    unsafe fn ensure_mapping_entry<'a, T: Translation + ?Sized>(&mut self, translation: &'a T, mapping: PageMapping<Page1G>) {
        let pml4ent = &mut self.0[pml4_index(VAddr::from_usize(mapping.vaddr))];
        if !pml4ent.is_present() {
            let pdpt = box PDPTWrap::default();
            *pml4ent = pdpt.make_entry(translation, Access::default_kernel_paging());
            Box::into_raw(pdpt);
        }
    }
}

impl Default for AS {
    fn default() -> AS {
        AS{0: [PML4Entry::empty(); 512]}
    }
}
