//! VGA console

use core::{ptr, intrinsics};
use x86::io;

use super::{Con, EarlyCon, V};

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
        // Make the cursor go away
        unsafe {
            io::outb(0x3d4, 0x0a);
            io::outb(0x3d5, 0x20);
        }
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
            // TODO: this is duplicated
            // We want default escaping *except* for quotes as they are regular printable ascii characters
            if c == '"' || c == '\'' {
                self.put_at_cursor(c as u8, color);
                self.increment_cursor();
            } else {
                for e in c.escape_default() {
                    self.put_at_cursor(e as u8, color);
                    self.increment_cursor();
                }
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

pub fn init_vga_80_25(_args: &str) -> Result<&'static mut EarlyCon, ()> {
    // TODO: validate that the base is within the memory limit
    unsafe {
        EARLY_VGA_80_25.reset();
    }
    Ok(unsafe{&mut EARLY_VGA_80_25})
}
