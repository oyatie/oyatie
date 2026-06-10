/* Linker script for the operating-system EL0 user program `user-spawn`.
 *
 * Unlike `user-hello` (which links at 0x1000_0000), this program links at the
 * kernel loader's *user window* base 0x0040_0000 (USER_BASE), because the
 * process-model loader maps PT_LOAD segments inside the 2 MiB window
 * 0x0040_0000..0x0060_0000 and requires ET_EXEC with vaddrs in that range
 * (matching where static musl binaries link). The kernel builds the SysV
 * initial stack itself and enters at `_start` with SP_EL0 already set, so this
 * program does NOT need to set up its own stack — but it still establishes a
 * known-good SP from the linker for robustness and zeroes .bss.
 *
 * The program demonstrates the process model: it `clone()`s (fork ABI), the
 * child exits with a status, and the parent `wait4()`s to reap it, all via raw
 * Linux/aarch64 syscalls (svc #0). */

ENTRY(_start)

/* Must match the kernel loader's USER_BASE. */
__user_base = 0x00400000;

SECTIONS
{
    . = __user_base;

    .text : ALIGN(16)
    {
        KEEP(*(.text.start))
        *(.text .text.*)
    }

    .rodata : ALIGN(16)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(16)
    {
        *(.data .data.*)
    }

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }

    . = ALIGN(16);
    __stack_bottom = .;
    . += 0x4000;
    __stack_top = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame_hdr)
        *(.ARM.exidx .ARM.exidx.*)
        *(.ARM.extab .ARM.extab.*)
        *(.dynamic)
        *(.dynsym .dynstr .hash .gnu.hash)
        *(.got .got.plt)
        *(.plt .plt.*)
    }
}
