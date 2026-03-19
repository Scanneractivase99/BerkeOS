/*
 * BerkeOS — Linker Script
 */

ENTRY(start)

SECTIONS
{
    . = 1M;

    .multiboot_header ALIGN(8) :
    {
        KEEP(*(.multiboot_header))
    }

    .text.boot ALIGN(4) :
    {
        KEEP(*(.text.boot))
    }

    .data ALIGN(4) :
    {
        *(.data .data.*)
    }

    .rodata ALIGN(4) :
    {
        *(.rodata .rodata.*)
    }

    .text ALIGN(4K) :
    {
        *(.text .text.*)
    }

    .bss ALIGN(4K) :
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
        *(.comment)
        *(.debug*)
        *(.gcc_except_table)
    }
}
