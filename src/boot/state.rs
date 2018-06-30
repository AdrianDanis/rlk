//! Definitions for state that is valid for boot

use vspace::VSpace;
use boot;

pub trait BootState {
    fn get_kernel_as(&self) -> &VSpace;
    fn get_kernel_as_mut(&mut self) -> &mut VSpace;
}

pub struct State {
    boot_as: boot::vspace::Init,
}

impl BootState for State {
    fn get_kernel_as(&self) -> &VSpace {
        &self.boot_as
    }
    fn get_kernel_as_mut(&mut self) -> &mut VSpace {
        &mut self.boot_as
    }
}

pub static mut STATE: State = State {
    boot_as: boot::vspace::Init::make()
};
