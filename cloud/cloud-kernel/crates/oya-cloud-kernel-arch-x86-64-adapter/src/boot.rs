//! x86_64 PVH boot trampoline: 32-bit protected mode -> long mode.
//!
//! Boot protocol: **PVH** (the Xen `hvm_start_info` ABI). We emit a `PT_NOTE`
//! with `XEN_ELFNOTE_PHYS32_ENTRY` (type 18) pointing at [`_start`]. When
//! launched as `qemu-system-x86_64 -kernel <elf>`, QEMU's PVH loader reads that
//! note and jumps to `_start` in **32-bit protected mode** with paging off, a
//! flat GDT, interrupts disabled, and `%ebx` holding the physical address of an
//! `hvm_start_info` block. (We do not need the start_info for this MVP.)
//!
//! From there this assembly trampoline:
//!   1. loads our own GDT (with a 64-bit code segment),
//!   2. builds 4-level identity page tables for the low 1 GiB using 2 MiB
//!      huge pages (PML4 -> one PDPT -> one PD of 512 entries),
//!   3. enables PAE (CR4.PAE), sets EFER.LME, enables paging (CR0.PG),
//!   4. far-jumps into 64-bit code (`long_mode_entry`), which sets the data
//!      segments, installs the stack, zeroes `.bss`, and calls `kmain`.
//!
//! This is the canonical "unsafe Frame" code: hand-written boot assembly the
//! framekernel isolates so everything above can be safe.

use core::arch::global_asm;

// ---------------------------------------------------------------------------
// PVH ELF note: XEN_ELFNOTE_PHYS32_ENTRY (type 18). Name "Xen", desc = 32-bit
// physical entry point. QEMU enters protected mode at this address.
// ---------------------------------------------------------------------------
global_asm!(
    r#"
    .section .note.kuberos, "a", @note
    .align 4
    .long 4                 // namesz = len("Xen\0")
    .long 4                 // descsz = 4 bytes (a 32-bit address)
    .long 18                // type  = XEN_ELFNOTE_PHYS32_ENTRY
    .asciz "Xen"            // name
    .align 4
    .long _start            // desc: 32-bit physical entry point
    .align 4
"#
);

// ---------------------------------------------------------------------------
// 32-bit entry trampoline -> long mode. Written as raw asm because it runs
// before any Rust ABI / stack guarantees and switches CPU mode mid-function.
// ---------------------------------------------------------------------------
global_asm!(
    r#"
    .code32
    .section .text.boot, "ax"
    .global _start
    .type _start, @function
_start:
    // We arrive in 32-bit protected mode, paging off, IF clear. %ebx -> start_info.
    cli

    // Use our own boot stack early (top of the 64-bit stack region works in
    // 32-bit mode too since it is < 4 GiB).
    lea     esp, [__stack_top]

    // Load a GDT that has 32-bit code (0x08), 64-bit code (0x18), data (0x10).
    lgdt    [gdt32_ptr]

    // ---- Build identity page tables for the low 1 GiB (2 MiB pages). ----
    // PML4[0] -> PDPT ; PDPT[0] -> PD ; PD[i] = i*2MiB | PRESENT|RW|PS.
    // Zero PML4 and PDPT first (PD is fully written below).
    mov     edi, offset boot_pml4
    xor     eax, eax
    mov     ecx, 1024            // 4096 bytes / 4 = 1024 dwords (PML4 + PDPT)
    rep     stosd

    // PML4[0] = PDPT | PRESENT | RW
    mov     eax, offset boot_pdpt
    or      eax, 0x3
    mov     [boot_pml4], eax

    // PDPT[0] = PD | PRESENT | RW
    mov     eax, offset boot_pd
    or      eax, 0x3
    mov     [boot_pdpt], eax

    // PD[i] = (i << 21) | PRESENT | RW | PS(huge)  for i in 0..512
    mov     edi, offset boot_pd
    mov     ecx, 512
    xor     eax, eax             // running physical address (low 32 bits)
1:
    mov     ebx, eax
    or      ebx, 0x83            // PRESENT | RW | PS
    mov     [edi], ebx           // low dword
    mov     dword ptr [edi+4], 0 // high dword (phys < 4 GiB)
    add     eax, 0x200000        // += 2 MiB
    add     edi, 8
    loop    1b

    // ---- Enable PAE (CR4.PAE = bit 5). ----
    mov     eax, cr4
    or      eax, 1 << 5
    mov     cr4, eax

    // ---- Point CR3 at PML4. ----
    mov     eax, offset boot_pml4
    mov     cr3, eax

    // ---- Set EFER.LME (long mode enable, bit 8) via MSR 0xC0000080. ----
    mov     ecx, 0xC0000080
    rdmsr
    or      eax, 1 << 8
    wrmsr

    // ---- Enable paging (CR0.PG = bit 31). Also ensure PE (bit 0) set. ----
    mov     eax, cr0
    or      eax, (1 << 31) | (1 << 0)
    mov     cr0, eax

    // Now in compatibility mode; far-jump to reload CS with the 64-bit segment.
    ljmp    0x18, offset long_mode_entry

    .size _start, . - _start

    // ---- 64-bit code ----
    .code64
long_mode_entry:
    // Reload data segment registers with the 64-bit data selector (0x10).
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    // Re-establish a 64-bit stack pointer.
    lea     rsp, [rip + __stack_top]

    // Zero .bss (Rust statics assume zeroed). rax=0, [rdi..rcx) stored.
    lea     rdi, [rip + __bss_start]
    lea     rcx, [rip + __bss_end]
    sub     rcx, rdi
    xor     rax, rax
    rep     stosb

    // Hand off to Rust. Never returns.
    call    rust_start

    // Should never be reached.
2:  hlt
    jmp     2b

    // ---- GDT (defined in this section so labels resolve from asm). ----
    .align 16
gdt32:
    .quad   0x0000000000000000   // 0x00 null
    // 0x08 32-bit code: base 0, limit 0xFFFFF, gran 4K, 32-bit, present, exec/read
    .quad   0x00CF9A000000FFFF
    // 0x10 data: present, writable (used in both 32 and 64-bit)
    .quad   0x00CF92000000FFFF
    // 0x18 64-bit code: present, exec, L=1 (long mode)
    .quad   0x00AF9A000000FFFF
gdt32_end:
    .align 4
gdt32_ptr:
    .word   gdt32_end - gdt32 - 1
    .long   gdt32
"#
);

// ---------------------------------------------------------------------------
// Page-table storage. 4 KiB-aligned, in .bss (zeroed by the trampoline's bss
// clear is too late — they are filled *before* the bss clear runs in 32-bit
// code, but the bss clear in 64-bit mode runs *after* we are already paging
// off these tables, so re-zeroing them would be harmless only because we no
// longer read PML4/PDPT/PD after CR3 load... actually CR3 still points here.
// To be safe we place them in their own NOLOAD region via #[no_mangle] statics
// in dedicated sections, and the 64-bit bss clear DOES touch them — but it
// only writes the same zero pattern except the entries we set. That would
// corrupt paging!  => We therefore put the page tables in .data-like aligned
// storage OUTSIDE .bss so the bss clear never touches them.
// ---------------------------------------------------------------------------

/// 4 KiB-aligned page-table page.
#[repr(C, align(4096))]
struct PageTable([u64; 512]);

// These live in `.pagetables` (a loaded, non-.bss section) so the 64-bit
// `.bss` clear in the trampoline cannot zero the entries the 32-bit code wrote
// (which are still live via CR3). They are referenced by name from the asm
// above (`boot_pml4`, `boot_pdpt`, `boot_pd`).
#[no_mangle]
#[link_section = ".pagetables"]
static mut boot_pml4: PageTable = PageTable([0; 512]);
#[no_mangle]
#[link_section = ".pagetables"]
static mut boot_pdpt: PageTable = PageTable([0; 512]);
#[no_mangle]
#[link_section = ".pagetables"]
static mut boot_pd: PageTable = PageTable([0; 512]);

/// First Rust code after the long-mode jump. Stack + `.bss` are ready.
///
/// Hands control to the safe kernel entry (`kmain`), which performs the
/// safe-wrapped bring-up and never returns.
#[no_mangle]
extern "C" fn rust_start() -> ! {
    extern "C" {
        fn kmain() -> !;
    }
    // SAFETY: `kmain` is the kernel-provided `#[no_mangle]` entry symbol taking
    // no args and returning `!`. This is the intended Frame -> safe-kernel
    // handoff once the stack and BSS are valid in long mode.
    unsafe { kmain() }
}

/// Panic handler for the x86_64 Frame. A freestanding binary needs exactly one.
/// By the time most panics fire the console is up, so print, then halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::console::_print(format_args!("\n*** KERNEL PANIC ***\n{}\n", info));
    loop {
        // SAFETY: `hlt` is a side-effect-free idle that parks the core until an
        // interrupt; with IRQs masked in panic context this is a pure idle.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
