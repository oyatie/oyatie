/* Linker script for the freestanding ring-3 PID1 init program user-init-x86_64.
 *
 * Static, non-PIE ELF64. Links at the user base 0x40_0000 (matching
 * user_layout::USER_BASE), so the kernel's process-model ELF loader maps the
 * PT_LOAD segments at their recorded vaddrs verbatim and enters _start with rsp
 * pointing at a SysV initial stack. Code (R+X), read-only data (R), and
 * writable data/bss (R+W) are placed in separate, page-aligned PT_LOAD segments
 * so the kernel can enforce W^X per segment.
 *
 * This program is the init contract: it installs SIGCHLD + SIGTERM handlers
 * (rt_sigaction + SA_RESTORER), clone()s a worker that execve()s the real svc,
 * loops on a short clock_nanosleep watching a .bss flag the SIGCHLD handler
 * sets, then wait4(WNOHANG)-reaps the worker and exits — all via raw syscalls.
 */

ENTRY(_start)

/* User base. Matches user_layout::USER_BASE. */
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
