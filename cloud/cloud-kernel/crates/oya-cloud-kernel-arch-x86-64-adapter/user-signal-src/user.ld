/* Linker script for the freestanding ring-3 user program user-signal-x86_64.
 *
 * Static, non-PIE ELF64. Links at the user base 0x40_0000 (matching
 * user_layout::USER_BASE). Code (R+X), read-only data (R), and writable data/bss
 * (R+W) live in separate, page-aligned PT_LOAD segments so the kernel enforces
 * W^X per segment. Unlike user-exec, this program needs a writable `.bss` flag
 * the SIGUSR1 handler sets, plus a `.text` `__restore` trampoline the handler
 * returns through (SA_RESTORER) — so it keeps the writable `data` segment.
 */

ENTRY(_start)

__user_base = 0x400000;

PHDRS
{
    text   PT_LOAD FLAGS(5);  /* R + X */
    rodata PT_LOAD FLAGS(4);  /* R     */
    data   PT_LOAD FLAGS(6);  /* R + W */
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

    . = ALIGN(0x1000);
    .data : ALIGN(16)
    {
        *(.data .data.*)
    } :data
    .bss (NOLOAD) : ALIGN(16)
    {
        *(.bss .bss.* COMMON)
    } :data

    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame_hdr)
    }
}
