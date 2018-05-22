pub mod multiboot;
pub mod cmdline;
pub mod vspace;
pub mod state;

use state::KERNEL_WINDOW;
use heap;

extern {
    static kernel_image_start: usize;
    static kernel_image_end: usize;
}

/// Marks memory to the allocator that is used by the kernel image
///
/// As this is boot inspecific this just adds 'static' memory from the kernel image
fn mark_image_mem() {
    unsafe {
        let begin_vaddr = &kernel_image_start as *const usize as usize;
        let end_vaddr = &kernel_image_end as *const usize as usize;
        // Convert to physical addresses. We unwrap as it is a fundamental assumption that
        // the kernel is mapped in and has valid virtual addresses
        let begin_paddr = KERNEL_WINDOW.vaddr_to_paddr(begin_vaddr).unwrap();
        let end_paddr = KERNEL_WINDOW.vaddr_to_paddr(end_vaddr).unwrap();
        heap::add_used_mem([begin_paddr..end_paddr]);
    }
}
