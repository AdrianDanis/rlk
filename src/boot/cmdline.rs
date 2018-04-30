//! Handles kernel cmdline processing

use util;

static mut CMDLINE: &str = "";

/// Provides the cmdline for later parsing
///
/// TODO: This is currently assuming the commandline is a 'static str which is not true
/// and violating all sorts of safety. Fix this once we have some early kind of allocation
pub fn set(cmdline: &'static str) {
    unsafe {CMDLINE = cmdline}
}

/// Process the cmdline calling any registered handlers
///
/// Handlers are registered through the `decls` interface using the `make_cmdline_decl!` macro
pub fn process() {
    let cmd = unsafe{CMDLINE};
    cmd.split_whitespace()
        .map(|x| util::split_first_str(x,"="))
        .filter_map(|(option, value)| if option.starts_with("--") { Some((&option[2..], value))} else { None })
        .for_each(|(option, value)|
            decls_iter!(CMDLine)
                .filter(|x| x.option == option)
                .for_each(|x| (x.f)(value))
        );
    // There was no point printing out the cmdline before this, now that we've processed it, print it out
    // so that any earlycon's that might have just been initialized will display it
    print!(Info, "Successfully processed initial kernel cmdline: {}", cmd);
}
