//! Driver for a 16550 UART

use super::io::Io;

const THR_OFF: u16 = 0;
const RBR_OFF: u16 = 0;

/// Description of the registers of the serial port
pub struct SerialPort<T: Io<Item = u8>> {
    /// Store the underlying IO accessor
    io: T
}

impl<T, R> SerialPort<T> where T: Io<Item = u8, Range=R>, R: From<u8> {
    unsafe fn read_data(&mut self) -> u8 {
        self.io.read(R::from(0))
    }
    unsafe fn write_data(&mut self, data: u8) {
        self.io.write(R::from(0), data)
    }
    unsafe fn init(&mut self) {
        // Minimal attempt to create sane state by disabling interrupts, the fifo and
        // ensuring the dlab is in its default (off) position
//        self.set_dlab(false);
//        self.disable_interrupts();
//        self.disable_fifo();
    }
    unsafe fn new(io: T) -> SerialPort<T> {
        let mut sp = SerialPort { io: io };
        sp.init();
        sp
    }
}
