use core::fmt;
use drivers;
use drivers::Serial;

use super::{Con, EarlyCon, V};

pub struct ConSerial {
    uart: Option<drivers::uart16550::Uart<drivers::io::PortIO<u8>>>,
}

static mut EARLY_SERIAL: ConSerial = ConSerial { uart: None };

impl Con for ConSerial {
    fn print(&mut self, s: &str) -> fmt::Result {
        unsafe {
            match self.uart {
                Some(ref mut uart) =>
                    for c in s.chars() {
                        // TODO: for now we assume a terminal that understands unicode. this is helpful
                        // as it means our colour control codes also get passed through unescaped
                        uart.write_byte(c as u8);
                    },
                None => (),
            }
        }
        Ok(())
    }
    fn prepare(&mut self, v: V) -> fmt::Result {
        //unimplemented!()
        // TODO: option for disabling ansi terminal assumption
        let mut wbf = |col| fmt::Write::write_fmt(self as &mut EarlyCon, format_args!("\x1B[1;{}m", col));
        match v {
            V::Panic => wbf(31),
            V::Error => wbf(33),
            V::Info => wbf(37),
            V::Debug => wbf(32),
            V::Trace => wbf(34),
        }
    }
    fn end(&mut self) -> fmt::Result {
        //unimplemented!()
        unsafe {
            match self.uart {
                Some(ref mut uart) => {
                        uart.write_byte(b'\r');
                        uart.write_byte(b'\n');
                    },
                None => (),
            }
        };
        Ok(())
    }
}

impl EarlyCon for ConSerial {
    fn shutdown(&mut self) -> () {
        unimplemented!()
    }
    fn become_virtual(&mut self) -> Result<(), ()> {
        unimplemented!()
    }
}

impl ConSerial {
    pub fn early_init(_args: &str) ->Result<&'static mut EarlyCon, ()> {
        unsafe {
            EARLY_SERIAL.uart = Some(drivers::uart16550::Uart::new(drivers::io::PortIO::new(0x3f8)));
        }
        Ok(unsafe{&mut EARLY_SERIAL})
    }
}
