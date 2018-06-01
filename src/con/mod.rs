// The kernel console is an output only device that may or may not be handed over to the
// user after startup.
// Process for manipulating the console is roughly
// 1. Check cmdline
// 2. Initialize any early consoles
// 3. Perform early system init
// 4. Shutdown early console and initialize actual consoles
// 5. Rest of system init
// 6. Shutdown consoles that are to be handed to the user (typically this will be all of them)
// After this console output can be controlled by the user to control verbosity
//
// Should a panic occur and there are no consoles for any reason then the following steps occur
// occur at least one console was found
// 1. Reinitialize any consoles from the cmdline
// 2. Reinitialize any early consoles from the cmdline
// 3. Attempt default init of early consoles

use core::result::Result;
use core::{fmt, ptr, intrinsics};
use util;
use x86::io;
use drivers;
use drivers::Serial;
use drivers::uart16550::Uart;

mod vga;
mod serial;

use self::vga::init_vga_80_25;
use self::serial::ConSerial;

// Verbosity level
#[derive(Debug, Copy, Clone)]
pub enum V {
    Panic,
    Error,
    Info,
    Debug,
    Trace,
}

// TODO: define buffer for con


pub trait Con {
    fn print(&mut self, s: &str) -> fmt::Result;
    fn prepare(&mut self, v: V) -> fmt::Result;
    fn end(&mut self) -> fmt::Result;
}

pub trait EarlyCon: Con {
    // TODO: should this be a general `Con` trait?
    fn shutdown(&mut self);
    // EarlyCon's start in a kernel without any device mappings, this tells the con that virtual
    // memory services are available and it should switch to them in preparation of early physical
    // boot window going away
    // This will only be called once
    // TODO: what kind of error handling?
    fn become_virtual(&mut self) -> Result<(),()>;
}

struct EarlyConEntry {
    name: &'static str,
    init: fn(args: &str) -> Result<&'static mut EarlyCon,()>,
}

static EARLY_CONS: [EarlyConEntry; 2] = [
    EarlyConEntry {name: "vga_80_25", init: init_vga_80_25},
    EarlyConEntry {name: "serial", init: ConSerial::early_init},
];

pub struct State {
    // Only support one early con at a time
    early: Option<&'static mut EarlyCon>,
    verbosity: V,
}

static mut CON_STATE: State = State {early: None, verbosity: V::Debug};

impl fmt::Write for EarlyCon {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s)
    }
}

impl State {
    // Early console initialize always succeeds as there will be no way to inform the user if it
    // went wrong so we might as well just keep going and hope we can get a real console eventually
    // and let them know
    pub fn early_init(&mut self, early: &str) {
        let (name, rest) = util::split_first_str(early, ",");
        for con in EARLY_CONS.iter() {
            if con.name == name {
                match (con.init)(rest) {
                    Ok(con) =>
                        self.early = Some(con),
                    Err(()) =>
                        return,
                };
            }
        }
    }

    pub fn set_verbosity(&mut self, verbosity: V) {
         self.verbosity = verbosity
    }

    fn log_allowed(&self, _v: V) -> bool {
        true
    }

    fn print_line(&mut self, verbosity: V, args: fmt::Arguments) -> fmt::Result {
        // Currently only assume the early con
        unsafe {
            match self.early {
                Some(ref mut con) => {
                    con.prepare(verbosity)?;
                    if let err@Err(_) = fmt::Write::write_fmt(con, args) {
                        // still run `end`, but return the error from write_fmt
                        let _ = con.end();
                        err
                    } else {
                        con.end()
                    }
                },
                None => Ok(()),
            }
        }
    }

    pub fn print(&mut self, verbosity: V, args: fmt::Arguments) -> fmt::Result {
        // TODO utf8 handling
        if self.log_allowed(verbosity) {
            // Generate actual message and print it
            let seconds = 0 as u64;
            let micros = 0 as u32;
            self.print_line(verbosity, format_args!("[{:0>5}.{:0>5}] {}", seconds, micros, args))
        } else {
            Ok(())
        }
    }
}

unsafe fn get() -> &'static mut State {
    &mut CON_STATE
}

pub fn early_init(early: &str) {
    unsafe{get().early_init(early);}
}

pub fn print(verbosity: V, message: &str) {
    print_fmt(verbosity, format_args!("{}", message))
}

pub fn print_fmt(verbosity: V, args: fmt::Arguments) {
    if let Err(err) = unsafe{get()}.print(verbosity, args) {
        panic!("Print failed with {}", err);
    }
}

#[macro_export]
macro_rules! print {
    ($v:ident, $($arg:tt)*) => ($crate::con::print_fmt($crate::con::V::$v, format_args!($($arg)*)));
}

/// Format of the --earlycon= parameter is: CON_NAME,ARG1=foo,ARG2=bar
/// For example --earlycon=serial,port=3f8
make_cmdline_decl!("earlycon", early_init, EARLYCON);

// TODO: add this as a test once we have a self test system
fn self_test() -> bool {
    print!(Debug, "unicode: 🍳  ");
    print!(Debug, "newline \n are escaped");
    print!(Debug, "Can put \" quotes \" in \'");
    true
}
