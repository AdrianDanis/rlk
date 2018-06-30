//! Routines for manipulating stacks

use vspace::*;

pub struct Stack {
    base: usize,
    guard_size: usize,
    top_offset: usize,
}

impl Stack {
    /// Execute the given function on this stack
    ///
    /// # Safety
    ///
    /// Caller is responsible for making sure that the stack being executed exists in the
    /// currently activated address space and that nothing else is using this stack. We
    /// cannot consume the stack as the stack has meaning beyond running on it.
    pub unsafe fn run_on_stack<A, F: FnOnce(A) -> !>(&mut self) -> ! {
        unimplemented!()
    }
    /// Creates new stack with default options for kernel
    ///
    /// # Safety
    ///
    /// Stacks are not free'd when they are dropped. For this reason the function is unsafe
    /// as it is the callers responsibility to ensure that memory is cleaned up.
    pub unsafe fn new_kernel(vspace: &mut VSpace) -> Stack {
        unimplemented!()
    }
}
