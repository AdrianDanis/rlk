use core;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments,
                               file: &'static str,
                               line: u32,
                               column: u32) -> ! {
    print!(Panic, "Panic at {} {}:{}: {}", file, line, column, msg);
    loop {}
}
