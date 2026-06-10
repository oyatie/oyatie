/* Linker script for the freestanding ring-3 user program user-exec-x86_64.
 *
 * Static, non-PIE ELF64. Links at the user base 0x40_0000 (matching
 * user_layout::USER_BASE). Code (R+X) and read-only data (R) live in separate,
 * page-aligned PT_LOAD segments so the kernel enforces W^X per segment. The
 * program only prints its pid/ppid and `exit(7)`s, so it needs no writable data.
 */

ENTRY(_start)

__user_base = 0x400000;

PHDRS
{
    text   PT_LOAD FLAGS(5);  /* R + X */
    rodata PT_LOAD FLAGS(4);  /* R     */
}

SECTIONS
{
    . = __user_base;

    .text : ALIGN(16)
    {
        *(.text._start)
        *(.text .text.*)
    } :text

    . = ALIGN(0x1000);
    .rodata : ALIGN(16)
    {
        *(.rodata .rodata.*)
    } :rodata

    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame_hdr)
        *(.data .data.* .bss .bss.* COMMON)
    }
}
