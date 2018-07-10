use util::units::{KB, MB, GB};
use core::mem;
use core::marker::PhantomData;
use core::ptr::Unique;
use cpu::features::Page1GB;
use core::ops::Range;

pub const PAGE_SIZE_4K: usize = 4 * KB;
pub const PAGE_SIZE_2M: usize = 2 * MB;
pub const PAGE_SIZE_1G: usize = GB;

#[repr(C, align(4096))]
struct RawPage4K {
    inner: [u8; PAGE_SIZE_4K],
}

impl Default for RawPage4K {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}

#[repr(C, align(2097152))]
struct RawPage2M {
    inner: [u8; PAGE_SIZE_2M],
}

impl Default for RawPage2M {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}

pub trait PageLevel {
    fn bytes() -> usize;
}

pub struct Page4K;
pub struct Page2M;
pub struct Page1G;

impl PageLevel for Page4K {
    fn bytes() -> usize {
        PAGE_SIZE_4K
    }
}

impl PageLevel for Page2M {
    fn bytes() -> usize {
        PAGE_SIZE_2M
    }
}

impl PageLevel for Page1G {
    fn bytes() -> usize {
        PAGE_SIZE_1G
    }
}

/// Page is a frame that has a virtual address
pub struct Page<S: PageLevel> {
    inner: *mut u8,
    phantom: PhantomData<S>,
}

impl<S: PageLevel> Page<S> {
    pub fn range(&self) -> Range<usize> {
        let base = self.inner as usize;
        return base..base + S::bytes();
    }
}

impl Page<Page1G> {
    pub unsafe fn new_unchecked(vaddr: usize, _marker: Page1GB) -> Page<Page1G> {
        Page { inner: vaddr as *mut u8, phantom: PhantomData}
    }
}

/// Raw unit of memory referenced by physical address
struct Frame<S: PageLevel> {
    paddr: usize,
    phantom: PhantomData<S>,
}

impl Frame<Page1G> {
    /// Declares that a 1GB frame exists at the provided virtual address
    ///
    /// Declaring 1GB pages we restrict by the Page1GB feature so that we can never even begin
    /// to talk about 1GB pages without it
    unsafe fn new_unchecked(paddr: usize, _marker: Page1GB) -> Frame<Page1G> {
        Frame { paddr: paddr, phantom: PhantomData}
    }
}
