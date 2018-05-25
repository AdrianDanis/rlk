//! Handles kernel cmdline processing

use util;
use alloc::String;
use alloc::string::ToString;

static mut CMDLINE: Option<String> = None;

/// Sets a record of the cmdline. This *should* be the same cmdline as was provided
/// to `process` initially, but there is no way to enforce that
///
/// TODO: This is currently assuming the commandline is a 'static str which is not true
/// and violating all sorts of safety. Fix this once we have some early kind of allocation
pub fn set(cmdline: &str) {
    unsafe {
        CMDLINE = Some(cmdline.to_string());
    }
}

/// Test a &str for some notion of being 'true'
///
/// Utility for testing for cmdline options that have been set to a true value
/// matches 1, true, on and enabled as being 'true'
pub fn option_is_true(value: &str) -> bool {
    match (value) {
        "1" | "ON" | "on" | "TRUE" | "true" | "ENABLED" | "enabled" => true,
        _ => false
    }
}

/// Process the passed cmdline calling any registered handlers. We process the one that is
/// passed as we want to process the cmdline before we have initialized any memory
/// allocators to setup any earlycons
///
/// Handlers are registered through the `decls` interface using the `make_cmdline_decl!` macro
pub fn process(cmdline: &str) {
    cmdline.split_whitespace()
        .map(|x| util::split_first_str(x,"="))
        .filter_map(|(option, value)| if option.starts_with("--") { Some((&option[2..], value))} else { None })
        .for_each(|(option, value)|
            decls_iter!(CMDLine)
                .filter(|x| x.option == option)
                .for_each(|x| (x.f)(value))
        );
    // There was no point printing out the cmdline before this, now that we've processed it, print it out
    // so that any earlycon's that might have just been initialized will display it
    print!(Info, "Successfully processed initial kernel cmdline: {}", cmdline);
}
