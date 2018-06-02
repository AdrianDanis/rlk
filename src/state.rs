//! Global kernel state

use boot;
use vspace::Window;
use cpu::Features;

/// Kernel virtual address window definition
///
/// Starts as the low window on boot. This is mostly needed by early boot code and allocator
/// setup.
pub static mut KERNEL_WINDOW: &'static Window = &boot::vspace::INIT_WINDOW;

/// Available CPU features
///
/// This is not an `Option` type as we want the features to always be queriable, but this means we must
/// construct in a state that claims features exist before we have tested them. We must ensure that
/// we perform a feature check to ensure that these default reuired features are available as soon
/// as possible in startup.
pub static mut CPU_FEATURES: Features = unsafe{Features::empty()};
