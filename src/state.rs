//! Global kernel state

use boot;
use vspace::VSpace;
use cpu::Features;
use vspace;
use util;

/// Holder of mutable kernel state
pub struct State {
    pub kernel_as: vspace::KernelVSpace,
}

pub static mut STATE: State = unsafe {
    State {
        kernel_as: util::uninitialized(),
    }
};

/// Available CPU features
///
/// This is not an `Option` type as we want the features to always be queriable, but this means we must
/// construct in a state that claims features exist before we have tested them. We must ensure that
/// we perform a feature check to ensure that these default reuired features are available as soon
/// as possible in startup.
pub static mut CPU_FEATURES: Features = unsafe{Features::empty()};
