/* Linker script for the freestanding ring-3 user program user-hello-x86_64.
 *
 * Static, non-PIE ELF64. Links at the user base 0x40_0000 (matching
 * user_layout::USER_BASE), so the kernel's ELF loader maps the PT_LOAD segments
 * at their recorded vaddrs verbatim and `iretq`s to _start. Code (R+X) and the
 * read-only message (R) are placed in separate, page-aligned PT_LOAD segments
 * so the kernel can enforce W^X per segment.
 */

ENTRY(_start)

/* User base. Matches user_layout::USER_BASE so the loader window math lines up
 * and the program sits well above the x86_64 kernel image (linked at 2 MiB). */
__user_base = 0x400000;

PHDRS
{
    text PT_LOAD FLAGS(5);   /* R + X */
    rodata PT_LOAD FLAGS(4); /* R     */
}

SECTIONS
{
    . = __user_base;

    .text : ALIGN(16)
    {
        *(.text._start)
        *(.text .text.*)
    } :text

    /* Page-align rodata onto its own PT_LOAD so it never shares a page with the
     * R+X text segment (keeps the loader's per-segment W^X strict). */
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
