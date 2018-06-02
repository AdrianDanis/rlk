//! Define generic IO traits and implementations

use core::marker::PhantomData;
use x86::shared::io;

pub trait Io {
    type Item: Copy;
    type Range = usize;

    /// Read from an IO offset
    ///
    /// Requires mutable as reading from I/O can have side effects
    unsafe fn read(&mut self, offset: Self::Range) -> Self::Item;
    unsafe fn write(&mut self, offset: Self::Range, value: Self::Item);
}

pub struct PortIO<T> {
    base: u16,
    data: PhantomData<T>,
}

impl<T> PortIO<T> {
    pub fn new(base: u16) -> PortIO<T> {
        PortIO {base: base, data: PhantomData}
    }
}

impl Io for PortIO<u8> {
    type Item = u8;
    type Range = u16;
    unsafe fn read(&mut self, offset: u16) -> u8 {
        io::inb(self.base + offset)
    }
    unsafe fn write(&mut self, offset: u16, value: u8) {
        io::outb(self.base + offset, value)
    }
}
