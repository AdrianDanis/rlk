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

trait Con {
    fn print(&mut self, character: u8);
}

trait EarlyCon: Con  {
    // TODO: should this be a general `Con` trait?
    fn shutdown(&mut self);
}

struct EarlyConEntry {
    name: &'static str,
    init: fn(args: &str) -> &'static EarlyCon,
}

struct TextFB {
    base: usize,
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
    }
}

impl EarlyCon for TextFB {
    fn shutdown(&mut self) -> () {
    }
}

static EARLY_VGA_80_25: TextFB = TextFB {
    base: 0,
    width: 0,
    height: 0,
    line_stride: 0,
    char_stride: 0,
    cursor_x: 0,
    cursor_y: 0,
};

fn init_vga_80_25(_args: &str) -> &'static EarlyCon {
    &EARLY_VGA_80_25
}

static EARLY_CONS: [EarlyConEntry; 1] = [
    EarlyConEntry {name: "vga_80_25", init: init_vga_80_25},
];
