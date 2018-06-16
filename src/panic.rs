use core::panic::PanicInfo;

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

#[panic_implementation]
#[no_mangle]
pub extern fn panic(info: &PanicInfo) -> ! {
    // TODO: As we try to print! in panic (and print! itself may panic) we should
    // attempt to detect a re-entrant panic and skip to something else
    if let (Some(location), Some(message)) = (info.location(), info.message()) {
        print!(Panic, "Panic at {} {}:{} {}", location.file(), location.line(), location.column(), message);
    } else {
        print!(Panic, "Panic at {:?} with {:?}", info.location(), info.message());
    }
    // No power management yet for power off, so try and trigger a reset instead
    unsafe {reboot()}
}
