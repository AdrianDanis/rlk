//! Driver for a 16550 UART

use super::io::Io;
use super::Serial;
use bitfield::BitRange;
use core::mem;

#[repr(u8)]
enum Parity {
    None = 0,
    Odd = 1,
    Even = 3,
    Mark = 5,
    Space = 7,
}

#[repr(u8)]
enum WordLength {
    Bits5 = 0,
    Bits6 = 1,
    Bits7 = 2,
    Bits8 = 3,
}

impl BitRange<Parity> for u8 {
    fn bit_range(&self, msb: usize, lsb: usize) -> Parity {
        // This is a bit unsafe but we shall assume the hardware is
        // correct here and that we never have invalid bit patterns
        unsafe{mem::transmute((self as &BitRange<u8>).bit_range(msb, lsb))}
    }
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: Parity) {
        (self as &mut BitRange<u8>).set_bit_range(msb, lsb, value as u8)
    }
}

impl BitRange<WordLength> for u8 {
    fn bit_range(&self, msb: usize, lsb: usize) -> WordLength {
        unsafe{mem::transmute((self as &BitRange<u8>).bit_range(msb, lsb))}
    }
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: WordLength) {
        (self as &mut BitRange<u8>).set_bit_range(msb, lsb, value as u8)
    }
}

impl From<Parity> for u8 {
    fn from(p: Parity) -> u8 {
        p as u8
    }
}

impl From<WordLength> for u8 {
    fn from(p: WordLength) -> u8 {
        p as u8
    }
}

bitfield!{
    pub struct LCR(u8);
    dlab, set_dlab: 7,7;
    sbe, set_sbe: 6,6;
    Parity, into Parity, parity, set_parity: 5, 3;
    stops, set_stops: 2, 2;
    WordLength, into WordLength, word_len, set_word_len: 0, 2;
}

//bitflags! {
    /// Line Control Register
//    struct LCR: u8 {
        /// Divisor Latch Access Bit
//        const DLAB = 0b10000000;
        /// Set Break Enable
//        const SBE = 0b1000000;
        /// Defines the set of bits that make up parity. You should always 
        /// Space parity
//        const P_SPACE = 0b111000;
        /// Mark parity
//        const P_MARK = 0b101000;
        /// Even parity
//        const P_EVEN = 0b011000;
        /// Odd parity
//        const P_ODD = 0b001000;
        /// No parity
//        const P_NONE = 0b000000;
//    }
//}
bitflags! {
    /// Interrupt Enable Register
    struct IER: u8 {
        /// Bit 7 is reserved, need a flag to preserve it
        const RESERVED7 = 0b10000000;
        /// Bit 6 is reserved, need a flag to preserve it
        const RESERVED6 = 0b01000000;
        /// 16750 bit Enables Low Power Mode
        const ELPM = 0b100000;
        /// 16750 bit Enables Sleep Mode
        const ESM = 0b10000;
        /// Enable Modem Status Interrupt
        const EMSI = 0b1000;
        /// Enable Receiver Line Status Interrupt
        const ERLSI = 0b100;
        /// Enable Transmitter Holding Register Empty Interrupt
        const ETHREI = 0b10;
        /// Enable Received Data Available Interrupt
        const ERDAI = 0b1;
        /// Meta flag for all of the actual interrupts
        const ALL_INT = Self::EMSI.bits | Self::ERLSI.bits | Self::ETHREI.bits | Self::ERDAI.bits;
        /// Meta flag for the 16750 features
        const ONLY_16750 = Self::ELPM.bits | Self::ESM.bits;
    }
}

/// Description of the registers of the serial port
pub struct Uart<T: Io<Item = u8>> {
    /// Store the underlying IO accessor
    io: T
}

impl<T, R> Uart<T> where T: Io<Item = u8, Range=R>, R: From<u8> {
    // Due to register overlapping we don't define a register map and just have direct logical
    // accessor functions
    unsafe fn read_data(&mut self) -> u8 {
        self.io.read(R::from(0))
    }
    unsafe fn write_data(&mut self, data: u8) {
        self.io.write(R::from(0), data)
    }
    unsafe fn read_lcr(&mut self) -> LCR {
//        LCR::from_bits_truncate(self.io.read(R::from(3)))
        LCR(self.io.read(R::from(3)))
    }
    unsafe fn write_lcr(&mut self, lcr: LCR) {
        self.io.write(R::from(3), lcr.0)
    }
    unsafe fn read_ier(&mut self) -> IER {
        IER::from_bits_truncate(self.io.read(R::from(1)))
    }
    unsafe fn write_ier(&mut self, ier: IER) {
        self.io.write(R::from(1), ier.bits())
    }
    unsafe fn set_dlab(&mut self, state: bool) {
        let mut lcr = self.read_lcr();
        lcr.set_dlab(state as u8);
        self.write_lcr(lcr);
    }
    unsafe fn disable_interrupts(&mut self) {
        let ier = self.read_ier();
        self.write_ier(ier - IER::ALL_INT)
    }
    unsafe fn init(&mut self) {
        // Minimal attempt to create sane state by disabling interrupts, the fifo and
        // ensuring the dlab is in its default (off) position
        self.set_dlab(false);
        self.disable_interrupts();
//        self.disable_fifo();
    }
    unsafe fn new(io: T) -> Uart<T> {
        let mut sp = Uart { io: io };
        sp.init();
        sp
    }
}

impl<T, R> Serial for Uart<T> where T: Io<Item = u8, Range=R>, R: From<u8> {
    unsafe fn write_byte(&mut self, byte: u8) {
    }
}
