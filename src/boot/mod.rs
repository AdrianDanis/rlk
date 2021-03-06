pub mod multiboot;
pub mod cmdline;
pub mod vspace;
pub mod state;

use heap;
use vspace::*;
use boot::state::BootState;

extern {
    static kernel_image_start: usize;
    static kernel_image_end: usize;
}

/// Marks memory to the allocator that is used by the kernel image
///
/// As this is boot inspecific this just adds 'static' memory from the kernel image
fn mark_image_mem<'a, B: BootState>(state: &'a B) {
    unsafe {
        let begin_vaddr = &kernel_image_start as *const usize as usize;
        let end_vaddr = &kernel_image_end as *const usize as usize;
        // Convert to physical addresses. We unwrap as it is a fundamental assumption that
        // the kernel is mapped in and has valid virtual addresses
        let image_paddr = declare_slice(state.get_kernel_as(), begin_vaddr, end_vaddr - begin_vaddr).unwrap();
        heap::add_used_mem(image_paddr);
        // Include the region used by the phys boot code
        // This can be reclaimed later
        let phys_vaddr = state.get_kernel_as().paddr_to_vaddr(KERNEL_PADDR_LOAD).unwrap();
        let phys_boot_paddr = declare_slice(state.get_kernel_as(), phys_vaddr, begin_vaddr - phys_vaddr).unwrap();
        heap::add_boot_mem(phys_boot_paddr);
    }
}
