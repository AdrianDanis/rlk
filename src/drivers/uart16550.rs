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
    // Define additional invalid bit patterns so that all 3 bit patterns are represented
    // This makes the transmute to convert from an integer to the enum less unsafe
    Invalid0 = 2,
    Invalid1 = 4,
    Invalid2 = 6,
}

#[repr(u8)]
enum WordLength {
    Bits5 = 0,
    Bits6 = 1,
    Bits7 = 2,
    Bits8 = 3,
}

impl From<u8> for WordLength {
    fn from(value: u8) -> WordLength {
        unsafe{mem::transmute(value)}
    }
}

impl Into<u8> for WordLength {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Parity {
    fn from(value: u8) -> Parity {
        unsafe{mem::transmute(value)}
    }
}

impl Into<u8> for Parity {
    fn into(self) -> u8 {
        self as u8
    }
}

bitfield!{
    pub struct LCR(u8);
    no default BitRange;
    dlab, set_dlab: 7,7;
    sbe, set_sbe: 6,6;
    Parity, into Parity, parity, set_parity: 5, 3;
    stops, set_stops: 2, 2;
    WordLength, into WordLength, word_len, set_word_len: 0, 2;
}

impl<T> BitRange<T> for LCR where T: Into<u8>, T: From<u8> {
    fn bit_range(&self, msb: usize, lsb: usize) -> T {
        T::from((self as &BitRange<u8>).bit_range(msb, lsb))
    }
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: T) {
        (self as &mut BitRange<u8>).set_bit_range(msb, lsb, value.into())
    }
}

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

bitflags! {
    /// Modem Control Register
    struct MCR: u8 {
        const RESERVED7 = 0b10000000;
        const RESERVED6 = 0b01000000;
        /// 16750 bit Autoflow Control Enabled
        const ACE = 0b100000;
        /// Loopback Mode
        const LM = 0b10000;
        /// Auxiliary Output 2
        const AO2 = 0b1000;
        /// Auxiliary Output 1
        const AO1 = 0b100;
        /// Request To Send
        const RTS = 0b10;
        /// Data Terminal Ready
        const DTS = 0b1;
    }
}

bitflags! {
    /// Line Status Register
    struct LSR: u8 {
        /// Error in Received FIFO
        const ERFIFO = 0b10000000;
        /// Empty Data Holding Registers
        const EDHR = 0b1000000;
        /// Empty Transmitter Holding Registers
        const ETHR = 0b100000;
        /// Break INterrupt
        const BI = 0b10000;
        /// Framing Error
        const FE = 0b1000;
        /// Parity Error
        const PE = 0b100;
        /// Overrun Error
        const OE = 0b10;
        /// Data Read
        const DR = 0b1;
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
    unsafe fn read_mcr(&mut self) -> MCR {
        MCR::from_bits_truncate(self.io.read(R::from(4)))
    }
    unsafe fn write_mcr(&mut self, mcr: MCR) {
        self.io.write(R::from(4), mcr.bits())
    }
    unsafe fn read_lsr(&mut self) -> LSR {
        LSR::from_bits_truncate(self.io.read(R::from(5)))
    }
    // We do not model the FIFO Control Register at all as we do not support
    // doing anything with it
    unsafe fn write_fifo_0(&mut self) {
        self.io.write(R::from(2), 0)
    }
    unsafe fn set_dlab(&mut self, state: bool) {
        let mut lcr = self.read_lcr();
        lcr.set_dlab(state as u8);
        self.write_lcr(lcr);
    }
    unsafe fn set_break(&mut self, state: bool) {
        let mut lcr = self.read_lcr();
        lcr.set_sbe(state as u8);
        self.write_lcr(lcr);
    }
    unsafe fn disable_interrupts(&mut self) {
        let ier = self.read_ier();
        self.write_ier(ier - IER::ALL_INT)
    }
    unsafe fn disable_fifo(&mut self) {
        self.write_fifo_0()
    }
    unsafe fn configure_line(&mut self, bits: WordLength, two_stops: bool, parity: Parity) {
        let mut lcr = self.read_lcr();
        lcr.set_stops(two_stops as u8);
        lcr.set_word_len(bits);
        lcr.set_parity(parity);
    }
    unsafe fn write_latch(&mut self, latch: u16) {
        self.set_dlab(true);
        let high = (latch >> 8) as u8;
        let low = (latch & 0xff) as u8;
        self.io.write(R::from(1), high);
        self.io.write(R::from(0), low);
        self.set_dlab(false);
    }
    unsafe fn set_baud_rate(&mut self, baud: u32) {
        let latch = (115200 / baud) as u16;
        self.write_latch(latch);
    }
    unsafe fn init(&mut self) {
        // Minimal attempt to create sane state by disabling interrupts, the fifo and
        // ensuring the dlab is in its default (off) position
        self.set_dlab(false);
        self.disable_interrupts();
        self.disable_fifo();
        // TODO: Just go ahead and initialize it with some assumptions. Init should
        // really be done through the generic serial interface
        self.configure_line(WordLength::Bits8, false, Parity::None);
        self.set_break(false);
        self.set_baud_rate(115200);
        let mcr = self.read_mcr();
        self.write_mcr((mcr - MCR::ACE - MCR::LM) | MCR::AO2 | MCR::AO1 | MCR::RTS | MCR::DTS);
    }
    unsafe fn new(io: T) -> Uart<T> {
        let mut sp = Uart { io: io };
        sp.init();
        sp
    }
}

impl<T, R> Serial for Uart<T> where T: Io<Item = u8, Range=R>, R: From<u8> {
    unsafe fn write_byte(&mut self, byte: u8) {
        while !self.read_lsr().contains(LSR::ETHR) {}
        self.write_data(byte);
    }
}
