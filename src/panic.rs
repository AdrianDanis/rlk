use core;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments,
                               file: &'static str,
                               line: u32,
                               column: u32) -> ! {
    // TODO: As we try to print! in panic (and print! itself may panic) we should
    // attempt to detect a re-entrant panic and skip to something else
    print!(Panic, "Panic at {} {}:{}: {}", file, line, column, msg);
    loop {}
}
