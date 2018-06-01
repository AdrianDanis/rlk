use core;

use drivers::io::{Io, PortIO};

pub unsafe fn reboot() -> ! {
    let mut kb = PortIO::new(0x64);
    // drain keyboard buffer
    while (kb.read(0) & 0x2) != 0 {}
    // toggle reset pin
    kb.write(0, 0xFE);
    print!(Error, "Reboot by 8042 seems to have failed");
    loop {}
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments,
                               file: &'static str,
                               line: u32,
                               column: u32) -> ! {
    // TODO: As we try to print! in panic (and print! itself may panic) we should
    // attempt to detect a re-entrant panic and skip to something else
    print!(Panic, "Panic at {} {}:{}: {}", file, line, column, msg);
    // No power management yet for power off, so try and trigger a reset instead
    unsafe {reboot()}
}
