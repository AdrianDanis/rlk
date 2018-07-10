use util::units::{KB, MB, GB};
use core::mem;

pub const PAGE_SIZE_4K: usize = 4 * KB;
pub const PAGE_SIZE_2M: usize = 2 * MB;
pub const PAGE_SIZE_1G: usize = GB;

#[repr(C, align(4096))]
struct Page4K {
    inner: [u8; PAGE_SIZE_4K],
}

impl Default for Page4K {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}

#[repr(C, align(2097152))]
struct Page2M {
    inner: [u8; PAGE_SIZE_2M],
}

impl Default for Page2M {
    fn default() -> Self {
        Self { inner: unsafe{mem::uninitialized()} }
    }
}
