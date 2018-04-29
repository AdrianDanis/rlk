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

// Verbosity level
pub enum V {
    Panic,
    Error,
    Info,
    Debug,
    Trace,
}

// TODO: define buffer for con


trait Con {
    fn print(&mut self, character: u8);
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

struct TextFB {
    base: *mut u8,
    width: u16,
    height: u16,
    line_stride: u32,
    char_stride: u32,
    cursor_x: u16,
    cursor_y: u16,
    // TODO: define color modes
}

impl Con for TextFB {
    fn print(&mut self, character: u8) -> () {
        let off = self.cursor_y as isize * self.line_stride as isize + self.cursor_x as isize * self.char_stride as isize;
        unsafe {
            *self.base.offset(off) = character;
            *self.base.offset(off + 1) = 0xb;
        }
        self.cursor_x = self.cursor_x + 1;
    }
}

impl EarlyCon for TextFB {
    fn shutdown(&mut self) -> () {
    }
    fn become_virtual(&mut self) -> Result<(), ()> {
        unimplemented!("not implemented")
    }
}

static mut EARLY_VGA_80_25: TextFB = TextFB {
    base: 0xb8000 as *mut u8,
    width: 80,
    height: 25,
    line_stride: 80 * 2,
    char_stride: 2,
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

// Only support one early con at a time
static mut EARLY_CON: Option<&'static mut EarlyCon> = None;

// Early console initialize always succeeds as there will be no way to inform the user if it
// went wrong so we might as well just keep going and hope we can get a real console eventually
// and let them know
// Format of the --earlycon= parameter is: CON_NAME,ARG1=foo,ARG2=bar
// For example --earlycon=serial,port=3f8
pub fn early_init(early: &str) {
    if unsafe{EARLY_CON.is_some()} {
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
                    unsafe{EARLY_CON = Some(con)},
                Err(()) =>
                    //TODO: print out an error
                    return,
            };
        }
    }
}

pub fn print(verbosity: V, message: &str) {
    // TODO utf8 handling
    unsafe {
        match EARLY_CON {
            Some(ref mut con) =>
                for c in message.chars() {
                    con.print(c as u8);
                },
            None => (),
        }
    }
}
