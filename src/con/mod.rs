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
    scroll_next: bool,
    active_color: u8,
    panic_color: u8,
    error_color: u8,
    info_color: u8,
    debug_color: u8,
    trace_color: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum BackgroundColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum ForegroundColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

const fn make_color(fore: ForegroundColor, back: BackgroundColor) -> u8 {
    ((back as u8) << 4) | (fore as u8)
}

// TODO: define trait for text screens that defines common logic that can be shared to
// framebuffer implementations that mimic text modes
impl VGAText {
    fn put_at(&mut self, x: u16, y: u16, c: u8, color: u8) {
        let off = y as isize * self.line_stride as isize + x as isize * 2 as isize;
        unsafe {
            ptr::write_volatile(self.base.offset(off), c);
            ptr::write_volatile(self.base.offset(off + 1), color);
        }
    }
    fn put_at_cursor(&mut self, c: u8, color: u8) {
        let x = self.cursor_x;
        let y = self.height - 1;
        self.put_at(x, y, c, color);
    }
    fn increment_cursor(&mut self)  {
        self.cursor_x = self.cursor_x + 1;
        if self.cursor_x == self.width {
            self.next_line();
        }
    }
    fn reset(&mut self) {
        for i in 0..self.height {
            self.blank_line(i);
        }
        // TODO: use I/O ports to disable the cursor
        //outb 0x3d4 0x0a
        //outb 0x3d5 0x20
    }
    fn next_line(&mut self) {
        self.cursor_x = 0;
        self.scroll();
    }
    fn copy_line(&mut self, dest: u16, src: u16) {
        unsafe{intrinsics::volatile_copy_nonoverlapping_memory(
            self.base.offset(dest as isize * self.line_stride as isize),
            self.base.offset(src as isize * self.line_stride as isize),
            self.line_stride as usize
        );}
    }
    fn blank_line(&mut self, line: u16) {
        for i in 0..self.width {
            self.put_at(i, line, ' ' as u8, 0xb);
        }
    }
    fn scroll(&mut self) {
        let h = self.height;
        for i in 0..h - 1 {
            self.copy_line(i, i + 1);
        }
        self.blank_line(h - 1);
    }
}

impl Con for VGAText {
    fn print(&mut self, s: &str) -> () {
        if self.scroll_next {
            self.next_line();
            self.scroll_next = false;
        }
        let color = self.active_color;
        for c in s.chars() {
            for e in c.escape_default() {
                self.put_at_cursor(e as u8, color);
                self.increment_cursor();
            }
        }
    }
    fn prepare(&mut self, v: V) {
        self.active_color = match v {
            V::Panic => self.panic_color,
            V::Error => self.error_color,
            V::Info => self.info_color,
            V::Debug => self.debug_color,
            V::Trace => self.trace_color,
        }
    }
    fn end(&mut self) {
        self.scroll_next = true;
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
    scroll_next: false,
    active_color: 0,
    // TODO: Should probably make default colors for this?
    panic_color: make_color(ForegroundColor::LightRed, BackgroundColor::Black),
    error_color: make_color(ForegroundColor::Yellow, BackgroundColor::Black),
    info_color: make_color(ForegroundColor::White, BackgroundColor::Black),
    debug_color: make_color(ForegroundColor::Green, BackgroundColor::Black),
    trace_color: make_color(ForegroundColor::LightBlue, BackgroundColor::Black),
};

fn init_vga_80_25(_args: &str) -> Result<&'static mut EarlyCon, ()> {
    // TODO: validate that the base is within the memory limit
    unsafe {
        EARLY_VGA_80_25.reset();
    }
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

/// Format of the --earlycon= parameter is: CON_NAME,ARG1=foo,ARG2=bar
/// For example --earlycon=serial,port=3f8
make_cmdline_decl!("earlycon", early_init, EARLYCON);
