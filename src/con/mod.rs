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
use core::{fmt,ptr};

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


trait Con {
    fn print(&mut self, s: &str);
    fn prepare(&mut self, v: V);
    fn end(&mut self);
}

trait EarlyCon: Con {
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

struct VGAText {
    base: *mut u8,
    width: u16,
    height: u16,
    line_stride: u32,
    cursor_x: u16,
    cursor_y: u16,
}

impl VGAText {
    fn put_at_cursor(&mut self, c: u8, color: u8) {
        let off = self.cursor_y as isize * self.line_stride as isize + self.cursor_x as isize * 2 as isize;
        unsafe {
            ptr::write_volatile(self.base.offset(off), c);
            ptr::write_volatile(self.base.offset(off + 1), color);
        }
        self.cursor_x = self.cursor_x + 1;
    }
    fn increment_cursor(&mut self)  {
    }
    fn next_line(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + 1 == self.height {
            self.scroll();
        } else {
            self.cursor_y = self.cursor_y + 1;
        }
    }
    fn scroll(&mut self) {
    }
}

impl Con for VGAText {
    fn print(&mut self, s: &str) -> () {
        for c in s.chars() {
            for e in c.escape_default() {
                self.put_at_cursor(e as u8, 0xb);
                self.increment_cursor();
            }
        }
    }
    fn prepare(&mut self, v: V) {
    }
    fn end(&mut self) {
        self.next_line();
    }
}

impl EarlyCon for VGAText {
    fn shutdown(&mut self) -> () {
    }
    fn become_virtual(&mut self) -> Result<(), ()> {
        unimplemented!("not implemented")
    }
}

static mut EARLY_VGA_80_25: VGAText = VGAText {
    base: 0xb8000 as *mut u8,
    width: 80,
    height: 25,
    line_stride: 80 * 2,
    cursor_x: 0,
    cursor_y: 0,
};

fn init_vga_80_25(_args: &str) -> Result<&'static mut EarlyCon, ()> {
    // TODO: validate that the base is within the memory limit
    Ok(unsafe{&mut EARLY_VGA_80_25})
}

static EARLY_CONS: [EarlyConEntry; 1] = [
    EarlyConEntry {name: "vga_80_25", init: init_vga_80_25},
];

pub struct State {
    // Only support one early con at a time
    early: Option<&'static mut EarlyCon>,
    verbosity: V,
}

static mut CON_STATE: State = State {early: None, verbosity: V::Debug};

impl fmt::Write for EarlyCon {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

impl State {
    // Early console initialize always succeeds as there will be no way to inform the user if it
    // went wrong so we might as well just keep going and hope we can get a real console eventually
    // and let them know
    // Format of the --earlycon= parameter is: CON_NAME,ARG1=foo,ARG2=bar
    // For example --earlycon=serial,port=3f8
    pub fn early_init(&mut self, early: &str) {
        if unsafe{self.early.is_some()} {
            //TODO: print an error for later
        }
        let mut iter = early.splitn(2, ",");
        // expect at least one element
        let (name, rest) = match iter.next() {
            Some(n) =>
                match iter.next() {
                    Some(r) => (n, r),
                    None => (n, ""),
                },
            None => {
                // TODO: print error
                return;
            },
        };
        for con in EARLY_CONS.iter() {
            if con.name == name {
                match (con.init)(rest) {
                    Ok(con) =>
                        self.early = Some(con),
                    Err(()) =>
                        //TODO: print out an error
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

    fn print_line(&mut self, verbosity: V, args: fmt::Arguments) {
        // Currently only assume the early con
        unsafe {
            match self.early {
                Some(ref mut con) => {
                    con.prepare(verbosity);
                    fmt::Write::write_fmt(con, args);
                    con.end();
                    },
                None => (),
            }
        }
    }

    pub fn print(&mut self, verbosity: V, args: fmt::Arguments) {
        // TODO utf8 handling
        if self.log_allowed(verbosity) {
            // Generate actual message and print it
            let seconds = 0 as u64;
            let micros = 0 as u32;
            self.print_line(verbosity, format_args!("[{:0>5}.{:0>5}] {}", seconds, micros, args));
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
    print_fmt(verbosity, format_args!("{}", message));
}

pub fn print_fmt(verbosity: V, args: fmt::Arguments) -> fmt::Result {
    unsafe{get().print(verbosity, args);};
    Ok(())
}

#[macro_export]
macro_rules! print {
    ($v:ident, $($arg:tt)*) => ($crate::con::print_fmt($crate::con::V::$v, format_args!($($arg)*)).unwrap());
}
