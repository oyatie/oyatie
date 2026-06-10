/* Linker script for the x86_64 Frame booted via the PVH (xen ELFNOTE
 * PHYS32_ENTRY) protocol under `qemu-system-x86_64 -kernel <elf>`.
 *
 * QEMU's PVH loader reads the PHYS32_ENTRY ELF note and jumps to that physical
 * address in 32-bit protected mode (paging off), with %ebx pointing at a
 * `hvm_start_info`. We link the image at 2 MiB so the low 1 MiB (BIOS/legacy)
 * stays clear and a single 2 MiB identity page covers our entry trampoline.
 *
 * The entry symbol `_start` is our 32-bit trampoline in `boot.rs`.
 */

ENTRY(_start)

/* Physical load/link base. 2 MiB keeps us above legacy low memory and aligns
 * to a 2 MiB huge-page boundary so identity paging is trivial. */
__load_addr = 0x200000;

SECTIONS
{
    . = __load_addr;

    /* Entry trampoline first so it sits at a known, low physical address and
     * the PHYS32_ENTRY note can point at `_start`. KEEP guards against GC. */
    .text.boot : ALIGN(16)
    {
        KEEP(*(.text.boot))
    }

    /* The PVH ELF note. KEEP so --gc-sections never drops it; it must be in a
     * PT_NOTE program header for QEMU to find PHYS32_ENTRY. */
    .note.operating-system : ALIGN(4)
    {
        KEEP(*(.note.operating-system))
    }

    .text : ALIGN(16)
    {
        *(.text .text.*)
    }

    /* P4·SMP·S3 AP trampoline. KEEP so --gc-sections never drops the
     * .code16/.code32/.code64 blob (only referenced from the SMP module by its
     * `ap_trampoline_start`/`_end` symbols + the asm). It is assembled here but
     * COPIED to AP_TRAMPOLINE_PA (0x8000) at bring-up; the 64-bit `_ap_long_mode`
     * tail runs at its linked high address (reached via absolute far-jump).
     *
     * Placed AFTER the main .text (not before) so adding it does not shift the
     * addresses of the existing kernel code — keeping the byte-identical 1-vCPU
     * boot (and the layout-sensitive talos-init leg) unperturbed. */
    .text.ap_trampoline : ALIGN(16)
    {
        KEEP(*(.text.ap_trampoline))
    }

    .rodata : ALIGN(16)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(16)
    {
        *(.data .data.*)
    }

    /* Boot page tables live in a loaded (non-.bss) section so the long-mode
     * `.bss` clear in the trampoline cannot zero the live PML4/PDPT/PD entries
     * (CR3 still points here after the switch). 4 KiB aligned per the asm. */
    .pagetables : ALIGN(4096)
    {
        KEEP(*(.pagetables))
    }

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

    /* Kernel heap, anchored ABOVE the ring-3 user window [0x40_0000, 0xC0_0000).
     *
     * x86_64 has a SINGLE per-CR3 address space, so once a process page table is
     * live every kernel virtual address inside [USER_BASE, USER_TOP) is shadowed
     * by that process's user mapping. The heap must therefore live OUTSIDE the
     * user window, or any kernel data structure the allocator places there (e.g.
     * the scheduler's process table) becomes unreadable under a process CR3 and
     * reads back the user image instead. (aarch64 is immune: its heap links at
     * ~1 GiB, well above the window.) We pin the heap base at USER_TOP = 0xC0_0000
     * so the whole 16 MiB heap sits above the window yet stays inside the boot
     * identity map's low 1 GiB. NOLOAD: zero-init is provided by the allocator's
     * own free-list node + each frame/page-table being zeroed on allocation, so
     * the boot trampoline's [__bss_start,__bss_end) clear need not cover it. */
    . = 0xC00000;
    .heap (NOLOAD) : ALIGN(4096)
    {
        __heap_start = .;
        KEEP(*(.heap))
        __heap_end = .;
    }

    /* P4·SMP·S3 per-AP stacks. Placed in their OWN NOLOAD section AFTER the heap
     * (not in `.bss`) so adding the 8×64 KiB AP-stack array does NOT grow `.bss`
     * and therefore does NOT shift `__stack_top` / the boot stack / the page
     * tables — keeping the default (1-vCPU) image's boot-stack + frame layout
     * byte-identical to the pre-S3 image. Still inside the boot identity map's
     * low 1 GiB. NOLOAD + the AP entry never relies on pre-zeroed stack bytes. */
    .ap_stacks (NOLOAD) : ALIGN(4096)
    {
        __ap_stacks_start = .;
        KEEP(*(.ap_stacks))
        __ap_stacks_end = .;
    }

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame .eh_frame_hdr)
    }
}
