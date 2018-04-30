ENTRY(_start)

OUTPUT_FORMAT(elf64-x86-64)

KERNEL_OFFSET = 0xffffffff80000000;

SECTIONS {
    /* We load physically to 1M */
    . = 1M;

    /* Place phys code/data etc here */
    .phys . : AT(ADDR(.phys)) {
        /* Place the header first */
        KEEP(*(.multiboot))
        *(.phys.text)
        *(.phys.data)
    }

    . = . + KERNEL_OFFSET;

    kernel_image_start = .;

    .text . : AT(ADDR(.text) - KERNEL_OFFSET) {
        *(.text*)
    }

    .rodata . : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        *(.rodata*)
    }

    .decls . : AT(ADDR(.decls) - KERNEL_OFFSET) {
        . = ALIGN(64);
        decls_section_begin = .;
        KEEP(*(.test*))
        decls_section_end = .;
    }

    .data . : AT(ADDR(.data) - KERNEL_OFFSET) {
        *(.data*)
    }

    .bss . : AT(ADDR(.bss) - KERNEL_OFFSET) {
        *(.bss*)
        *(COMMON)
    }

    kernel_image_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note.gnu.build-id)
        *(.comment)
    }
}
