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
    pub unsafe fn run_on_stack<A, F: FnOnce(A) -> !>(&mut self, arg: A, f: F) -> ! {
        unimplemented!()
    }
    /// Creates new stack with default options for kernel
    ///
    /// # Safety
    ///
    /// Stacks are not free'd when they are dropped. For this reason the function is unsafe
    /// as it is the callers responsibility to ensure that memory is cleaned up.
    pub unsafe fn new_kernel<V: VSpace>(vspace: &mut V) -> Option<Stack> {
        vspace.reserve(2 * PAGE_SIZE_2M, PAGE_SIZE_2M)
            .and_then(|base| vspace.fill(base + PAGE_SIZE_2M, PAGE_SIZE_2M))
            .map(|base| Stack {base: base as usize, guard_size: PAGE_SIZE_2M, top_offset: 2 * PAGE_SIZE_2M})
    }
}
