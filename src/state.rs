//! Global kernel state

use boot;
use vspace::Window;

/// Kernel virtual address window definition
///
/// Starts as the low window on boot. This is mostly needed by early boot code and allocator
/// setup.
pub static mut KERNEL_WINDOW: &'static Window = &boot::vspace::INIT_WINDOW;

