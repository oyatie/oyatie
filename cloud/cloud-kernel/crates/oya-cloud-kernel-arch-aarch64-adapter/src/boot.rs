//! aarch64 boot entry stub.
//!
//! QEMU `-M virt` enters our ELF at `_start` (see `linker.ld`) on a single
//! core, in EL1, with the MMU off and interrupts masked. Our job in this MVP:
//!
//!   1. Park secondary cores (only CPU 0 proceeds).
//!   2. Install the boot stack pointer.
//!   3. Zero the `.bss` section (Rust assumes statics start zeroed).
//!   4. Jump to Rust and spin in a low-power loop.
//!
//! This is freestanding assembly + one Rust function. It is the canonical
//! "unsafe Frame" code the framekernel design isolates so the rest of the tree
//! can be safe.

use core::arch::global_asm;

// The boot stub lives in its own section (`.text.boot`) which the linker
// script KEEPs first, guaranteeing `_start` is the image entry point.
//
// Registers/symbols used:
//   mpidr_el1  - core id; we keep only Aff0 == 0.
//   __stack_top, __bss_start, __bss_end - provided by linker.ld.
global_asm!(
    r#"
    .section .text.boot, "ax"
    .global _start
    .type _start, %function
_start:
    // P4·SMP·S3: PRESERVE the DTB base QEMU passes in x0 BEFORE we clobber x0.
    // QEMU `-M virt` enters `_start` with x0 = physical address of the flattened
    // device tree. The existing bring-up immediately reuses x0 (mpidr read, then
    // the bss loop), destroying it — so today the DTB is never read. We stash it
    // in a callee-saved reg (x21) the instant we enter, then hand it to
    // `rust_start` so the `/cpus` enumeration (cpu_count + MPIDRs) can parse it.
    mov     x21, x0

    // Park all cores except CPU 0 (Aff0 == 0).
    mrs     x0, mpidr_el1
    and     x0, x0, #0xff
    cbnz    x0, .Lpark

    // Set up the stack pointer for CPU 0.
    ldr     x0, =__stack_top
    mov     sp, x0

    // Zero .bss: x0 = start, x1 = end, store 16 bytes per iteration.
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end
.Lbss_loop:
    cmp     x0, x1
    b.hs    .Lbss_done
    stp     xzr, xzr, [x0], #16
    b       .Lbss_loop
.Lbss_done:

    // Enter Rust with the preserved DTB base in x0 (the C ABI first arg).
    // Should never return.
    mov     x0, x21
    bl      {rust_start}

    // Fall through to the park loop if it ever does.
.Lpark:
    wfe
    b       .Lpark
    .size _start, . - _start
"#,
    rust_start = sym rust_start,
);

extern "C" {
    /// The safe kernel entry point. Defined `#[no_mangle]` in the `kernel`
    /// crate; the Frame calls into it once the stack and BSS are ready. It must
    /// not return.
    fn kmain() -> !;
}

/// First Rust code to run after the assembly boot stub.
///
/// The stack is set and `.bss` is zeroed; `dtb` is the device-tree base QEMU
/// passed in x0 (preserved through the bss clobber by `_start`, P4·SMP·S3). We
/// record it for the `/cpus` enumeration the SMP path needs, then hand control
/// to the safe kernel entry, which performs the (safe-wrapped) bring-up and
/// never returns.
#[no_mangle]
extern "C" fn rust_start(dtb: *const u8) -> ! {
    // SAFETY: storing the preserved DTB pointer for later read-only FDT parsing
    // (`crate::smp`); a plain pointer store with no dereference here.
    crate::smp::store_dtb(dtb);
    // SAFETY: `kmain` is the kernel-provided entry symbol; it takes no
    // arguments and is `-> !`. Calling it here is the intended handoff from the
    // Frame's boot stub to the safe kernel.
    unsafe {
        kmain();
    }
}

// ---------------------------------------------------------------------------
// P4·SMP·S3 — secondary-CPU (AP) entry.
// ---------------------------------------------------------------------------
//
// PSCI `CPU_ON` (issued by `crate::smp`) hands an AP control here with the MMU
// off, caches off, IRQs masked, and x0 = the `context_id` we passed = this AP's
// logical cpu_index. PSCI guarantees that delivery, so the AP reads its index
// straight from x0. Steps (mirroring the BSP bring-up but per-CPU): save the
// index, install this AP's stack from the static `AP_STACKS` array, then drop
// into Rust (`ap_rust_entry`) which enables translation against the SHARED L1
// table, sets the per-CPU anchor, wakes its own GICR + ICC, publishes its
// online bit, and `wfe`-idles. The AP NEVER returns.
global_asm!(
    r#"
    .section .text.boot, "ax"
    .global _ap_start
    .type _ap_start, %function
_ap_start:
    // x0 = context_id = our logical cpu_index (PSCI-delivered). Preserve it
    // across the stack-pointer setup in a callee-saved reg.
    mov     x19, x0

    // Install this AP's stack: sp = &AP_STACKS[idx] + AP_STACK_SIZE (top; the
    // stack grows down). `AP_STACK_SIZE` is provided as an asm const below.
    ldr     x1, =__ap_stacks
    mov     x2, {stack_size}
    mul     x3, x19, x2          // x3 = idx * AP_STACK_SIZE
    add     x1, x1, x3           // x1 = &AP_STACKS[idx]
    add     x1, x1, x2           // x1 = top of this AP's stack
    mov     sp, x1

    // Enter Rust with the cpu_index as the first C-ABI arg. Diverges.
    mov     x0, x19
    bl      {ap_rust_entry}

.Lap_park:
    wfe
    b       .Lap_park
    .size _ap_start, . - _ap_start
"#,
    ap_rust_entry = sym crate::smp::ap_rust_entry,
    stack_size = const crate::smp::AP_STACK_SIZE,
);

/// Panic handler for the aarch64 Frame. A freestanding binary must define
/// exactly one. By the time most panics fire the console is up, so print the
/// panic info, then halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Mask IRQs so the panic message is not interleaved with the timer ISR.
    crate::console::_print(format_args!("\n*** KERNEL PANIC ***\n{}\n", info));
    loop {
        // SAFETY: `wfe` is a side-effect-free idle hint; safe in any context.
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
        }
    }
}
