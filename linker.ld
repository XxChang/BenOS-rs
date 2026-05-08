ENTRY(_start)

KERNEL_BASE = 0x80000;

SECTIONS
{
    . = KERNEL_BASE;

    __kernel_start = .;

    .text : ALIGN(4K)
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss (NOLOAD) : ALIGN(4K)
    {
        bss_begin = .;
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        bss_end = .;
        __bss_end = .;
    }

    .stack (NOLOAD) : ALIGN(4K)
    {
        _stack_bottom = .;
        . += 0x10000;
        _stack_top = .;
    }

    . = ALIGN(4K);
    __kernel_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.note .note.*)
    }
}
