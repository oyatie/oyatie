/* Linker script for the aarch64 Frame on the QEMU `virt` machine.
 *
 * QEMU `-M virt` loads a `-kernel` ELF and begins executing at its entry
 * point. RAM on `virt` starts at 0x4000_0000; QEMU's loader places a flat
 * `-kernel` image at 0x4008_0000 (the conventional Linux/arm64 text offset of
 * 0x80000 above RAM base), so we link there and make `_start` the entry. */

ENTRY(_start)

/* Base load/link address: RAM_BASE (0x4000_0000) + TEXT_OFFSET (0x8_0000). */
__load_addr = 0x40080000;

SECTIONS
{
    . = __load_addr;

    /* Keep the entry stub first so the very first instruction QEMU runs is
     * _start. KEEP() prevents --gc-sections from discarding it. */
    .text : ALIGN(8)
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    /* Exception vector table. VBAR_EL1 requires 2 KiB alignment; the table is
     * itself .align 11 internally, but place it in its own section and KEEP it
     * so --gc-sections never drops the (only-referenced-from-asm) vectors. */
    .text.vectors : ALIGN(2048)
    {
        KEEP(*(.text.vectors))
    }

    .rodata : ALIGN(8)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(8)
    {
        *(.data .data.*)
    }

    /* BSS is zero-initialized by the boot stub. Symbols are 8-byte aligned so
     * the clear loop can store 64 bits at a time. */
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }

    /* Boot stack: 64 KiB, grows downward from __stack_top. */
    . = ALIGN(16);
    __stack_bottom = .;
    . += 0x10000;
    __stack_top = .;

    /* P4·SMP·S3 per-AP stacks. Placed in their OWN NOLOAD section AFTER the boot
     * stack (NOT in `.bss`) so adding the 8×64 KiB AP-stack array does not grow
     * `.bss` and shift `__stack_top` / the `.bss`-resident heap — keeping the
     * default 1-vCPU layout (and the byte-identical golden) untouched. Still
     * inside the identity-mapped RAM block. NOLOAD; the AP entry does not rely on
     * pre-zeroed stack bytes (and `_start`'s bss clear does not cover it). */
    . = ALIGN(4096);
    .ap_stacks (NOLOAD) : ALIGN(4096)
    {
        __ap_stacks_start = .;
        KEEP(*(.ap_stacks))
        __ap_stacks_end = .;
    }

    /* Drop ELF metadata we do not need in a freestanding image. */
    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame_hdr)
        *(.ARM.exidx .ARM.exidx.*)
        *(.ARM.extab .ARM.extab.*)
    }
}
