//! Collection of drivers used by the kernel

pub mod uart16550;
pub mod io;

pub trait Serial {
    // TODO: should have errors or timeouts?
    unsafe fn write_byte(&mut self, byte: u8);
}
