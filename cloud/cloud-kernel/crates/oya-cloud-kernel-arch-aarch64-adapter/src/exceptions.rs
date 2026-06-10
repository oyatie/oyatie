//! Exception vectors (`VBAR_EL1`) for the aarch64 Frame.
//!
//! AArch64 defines a 2 KiB-aligned vector table of 16 entries (128 bytes each),
//! grouped by exception kind (Synchronous / IRQ / FIQ / SError) and by the
//! source state (current EL with SP0, current EL with SPx, lower EL aarch64,
//! lower EL aarch32). We run entirely at EL1h, so the "current EL with SPx"
//! group is the one that fires.
//!
//! Each vector saves the integer registers, calls a Rust handler with a pointer
//! to that saved frame, then restores and `eret`s. Synchronous/SError/FIQ
//! handlers panic with decoded fault info (so faults are debuggable); the IRQ
//! handler dispatches to the timer/GIC code.

use core::arch::global_asm;

use aarch64_cpu::registers::{ESR_EL1, FAR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

/// The integer register frame pushed by the vector entry stubs.
#[repr(C)]
pub struct TrapFrame {
    /// x0..x30 (x30 = LR).
    pub regs: [u64; 31],
    /// Stack pointer / padding slot kept for 16-byte alignment.
    pub sp: u64,
    /// Exception Link Register (return address).
    pub elr: u64,
    /// Saved Program Status Register.
    pub spsr: u64,
}

global_asm!(
    r#"
    // Each hardware vector slot is exactly 128 bytes, so it must stay small.
    // A slot only stashes x0/x1, loads the vector `kind`, and branches to the
    // shared trampoline `__trap_common`, which does the full save/dispatch/
    // restore. This keeps every slot well under 0x80 bytes.
    .macro VECTOR kind
        sub     sp,  sp,  #272        // reserve the trap frame
        stp     x0,  x1,  [sp, #16 * 0]
        mov     x1,  #\kind
        b       __trap_common
    .endm

    .section .text.vectors, "ax"
    .align 11                 // 2 KiB alignment required by VBAR_EL1.
    .global __exception_vectors
__exception_vectors:
    // ---- Current EL with SP0 (unused; we run at SPx). ----
    .align 7
    VECTOR 0
    .align 7
    VECTOR 1
    .align 7
    VECTOR 2
    .align 7
    VECTOR 3
    // ---- Current EL with SPx (this is what fires at EL1h). ----
    .align 7
    VECTOR 4                  // Synchronous
    .align 7
    VECTOR 5                  // IRQ
    .align 7
    VECTOR 6                  // FIQ
    .align 7
    VECTOR 7                  // SError
    // ---- Lower EL, aarch64. ----
    .align 7
    VECTOR 8
    .align 7
    VECTOR 9
    .align 7
    VECTOR 10
    .align 7
    VECTOR 11
    // ---- Lower EL, aarch32. ----
    .align 7
    VECTOR 12
    .align 7
    VECTOR 13
    .align 7
    VECTOR 14
    .align 7
    VECTOR 15

    // Shared trap trampoline. On entry: sp points at a 272-byte frame whose
    // first 16 bytes already hold x0/x1; x1 holds the vector `kind`.
    .section .text, "ax"
    .align 4
__trap_common:
        // Save the remaining GPRs (x2..x30) and a padding slot.
        stp     x2,  x3,  [sp, #16 * 1]
        stp     x4,  x5,  [sp, #16 * 2]
        stp     x6,  x7,  [sp, #16 * 3]
        stp     x8,  x9,  [sp, #16 * 4]
        stp     x10, x11, [sp, #16 * 5]
        stp     x12, x13, [sp, #16 * 6]
        stp     x14, x15, [sp, #16 * 7]
        stp     x16, x17, [sp, #16 * 8]
        stp     x18, x19, [sp, #16 * 9]
        stp     x20, x21, [sp, #16 * 10]
        stp     x22, x23, [sp, #16 * 11]
        stp     x24, x25, [sp, #16 * 12]
        stp     x26, x27, [sp, #16 * 13]
        stp     x28, x29, [sp, #16 * 14]
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        stp     x30, xzr, [sp, #16 * 15]
        stp     x2,  x3,  [sp, #16 * 16]

        mov     x0, sp               // &TrapFrame
        // x1 already holds `kind`.
        bl      {rust_trap}

        // Restore ELR/SPSR then the GPRs and return.
        ldp     x2,  x3,  [sp, #16 * 16]
        msr     elr_el1,  x2
        msr     spsr_el1, x3
        ldp     x2,  x3,  [sp, #16 * 1]
        ldp     x4,  x5,  [sp, #16 * 2]
        ldp     x6,  x7,  [sp, #16 * 3]
        ldp     x8,  x9,  [sp, #16 * 4]
        ldp     x10, x11, [sp, #16 * 5]
        ldp     x12, x13, [sp, #16 * 6]
        ldp     x14, x15, [sp, #16 * 7]
        ldp     x16, x17, [sp, #16 * 8]
        ldp     x18, x19, [sp, #16 * 9]
        ldp     x20, x21, [sp, #16 * 10]
        ldp     x22, x23, [sp, #16 * 11]
        ldp     x24, x25, [sp, #16 * 12]
        ldp     x26, x27, [sp, #16 * 13]
        ldp     x28, x29, [sp, #16 * 14]
        ldr     x30,      [sp, #16 * 15]
        ldp     x0,  x1,  [sp, #16 * 0]
        add     sp,  sp,  #272
        eret
"#,
    rust_trap = sym rust_trap,
);

extern "C" {
    static __exception_vectors: u8;
}

/// Install the exception vector base into `VBAR_EL1`.
///
/// # Safety
/// Must be called once on the boot core before enabling interrupts. The vector
/// table is statically linked and `.text.vectors` is 2 KiB aligned.
pub unsafe fn init() {
    let base = core::ptr::addr_of!(__exception_vectors) as u64;
    VBAR_EL1.set(base);
}

/// Vector kinds, matching the immediate passed by the asm stubs.
const KIND_CUR_SPX_SYNC: u64 = 4;
const KIND_CUR_SPX_IRQ: u64 = 5;
/// Lower-EL (EL0) aarch64 synchronous exception (where SVCs from user land).
const KIND_LOWER_AARCH64_SYNC: u64 = 8;
/// Lower-EL (EL0) aarch64 IRQ.
const KIND_LOWER_AARCH64_IRQ: u64 = 9;

/// Exception Class for an `SVC` executed in AArch64 state (ESR_EL1.EC).
const EC_SVC_AARCH64: u64 = 0x15;
/// Exception Class for a Data Abort taken from a lower EL (EL0 -> EL1).
const EC_DATA_ABORT_LOWER: u64 = 0x24;
/// Exception Class for an Instruction Abort taken from a lower EL (EL0 -> EL1).
/// A stale-TLB artifact under SMP (a sibling CPU kept an old executable mapping
/// for a page another CPU re-mapped PXN/UXN during a COW copy) lands here with
/// `DFSC = permission fault L3` and `FAR == ELR` (the faulting fetch address).
const EC_INSN_ABORT_LOWER: u64 = 0x20;
/// ESR_EL1.ISS bit 6 (WnR): the aborting access was a **write**.
const ISS_WNR_WRITE: u64 = 1 << 6;
/// ESR_EL1.ISS[5:0] (DFSC) value for a **permission fault, level 3** — what a
/// write to a write-protected (read-only) leaf page produces. A copy-on-write
/// page is exactly such a write-protected leaf.
const DFSC_PERM_FAULT_L3: u64 = 0b00_1111;

/// Rust trap entry. `frame` points at the saved [`TrapFrame`]; `kind` is the
/// vector index (0..15).
#[no_mangle]
extern "C" fn rust_trap(frame: *mut TrapFrame, kind: u64) {
    match kind {
        KIND_CUR_SPX_IRQ => {
            // IRQ while the kernel itself was running at EL1 (e.g. during the
            // boot timer demo). Just service it; nothing to preempt. Route the
            // ack/EOI to the live GIC: the GICv3 leg uses the ICC sysreg CPU
            // interface, the GICv2 leg the GICC MMIO — the single version-gated
            // branch (mirrors `apic::ack_timer` swapping only the EOI source).
            let _ = match crate::gicv3::active_gic() {
                crate::gicv3::GicVersion::V3 => crate::gicv3::handle_irq(),
                crate::gicv3::GicVersion::V2 => crate::gic::handle_irq(),
            };
        }
        KIND_LOWER_AARCH64_IRQ => {
            // IRQ while a user process was running at EL0. Service it; ONLY a
            // timer tick drives a preemption (round-robin time slicing). A
            // P4·SMP·S4c shootdown SGI returns `false` — it must NOT reschedule:
            // it only flushed this CPU's stale TLB + acked, and the interrupted
            // EL0 context resumes unchanged (rescheduling around a shootdown
            // would cross two CPUs' saved contexts → register corruption).
            let was_tick = match crate::gicv3::active_gic() {
                crate::gicv3::GicVersion::V3 => crate::gicv3::handle_irq(),
                crate::gicv3::GicVersion::V2 => crate::gic::handle_irq(),
            };
            if was_tick {
                // SAFETY: the vector stub just pushed a valid frame at `frame`;
                // the preemption hook saves the current EL0 context and may switch.
                unsafe {
                    crate::user::on_timer_preempt(frame);
                }
            }
        }
        KIND_LOWER_AARCH64_SYNC => {
            // A synchronous exception from EL0. If it is an SVC, dispatch the
            // Linux-ABI syscall layer; otherwise it is a genuine user fault.
            let esr = ESR_EL1.get();
            let ec = (esr >> 26) & 0x3f;
            if ec == EC_SVC_AARCH64 {
                // SAFETY: the vector stub just pushed a valid frame at `frame`;
                // the syscall layer validates any user pointers it touches.
                unsafe {
                    crate::user::handle_svc(frame);
                }
                return;
            }
            // A write data abort from EL0 may be a copy-on-write fault: a write to
            // a page write-protected + COW-tagged by `fork`. If so, copy the page
            // and resume; otherwise it is a genuine fault and stays fatal.
            let iss = esr & 0x01ff_ffff;
            if ec == EC_DATA_ABORT_LOWER
                && (iss & ISS_WNR_WRITE) != 0
                && (iss & 0b11_1111) == DFSC_PERM_FAULT_L3
            {
                let far = FAR_EL1.get() as usize;
                // SAFETY: the current process's space is the live TTBR0 of the
                // faulting EL0 thread; `try_cow_fault` copies the page + flushes
                // THIS CPU's TLB (under the PROCS lock). Resolving the fault
                // returns to re-execute the store.
                let resolved =
                    crate::process::with_sched(|s| unsafe { s.try_cow_fault(far) });
                if resolved {
                    // NOTE (P4·SMP·S4c): `cow_fault` re-maps ONLY the FAULTING
                    // process's OWN PTE (a private copy) — no SIBLING address space
                    // is touched, so no other CPU can hold a stale entry for this
                    // process's now-private page that it isn't itself about to
                    // re-walk on the retried store. The local `tlbi vae1` in
                    // `cow_fault` therefore suffices; no cross-CPU shootdown is
                    // needed here. (The cross-CPU shootdown is required at the
                    // sites that mutate an AS ANOTHER CPU runs: `cow_clone` write-
                    // protecting the parent in place, and exec/exit freeing an AS —
                    // wired in `user.rs`.)
                    return;
                }
            }
            // A lower-EL INSTRUCTION abort (EC=0x20) with a permission fault may be
            // a STALE-TLB artifact under SMP: a sibling CPU re-mapped this page
            // PXN/UXN (a COW copy) while this CPU kept the old executable mapping
            // cached, and a late/missed shootdown left it stale. Defense-in-depth
            // (the proactive shootdown above is the primary fix): if the PTE for
            // FAR is actually valid+executable now, this is a stale fetch — do a
            // local `tlbi vae1` for that VA and retry; only if the PTE genuinely
            // forbids EL0 execution is it a real fault that we let fall through.
            if ec == EC_INSN_ABORT_LOWER && (iss & 0b11_1111) == DFSC_PERM_FAULT_L3 {
                let far = FAR_EL1.get() as usize;
                // SAFETY: the current process's space is the live TTBR0; this only
                // INSPECTS the PTE and, on a stale hit, invalidates one VA.
                let retried = crate::process::with_sched(|s| unsafe {
                    s.try_retry_stale_insn_fetch(far)
                });
                if retried {
                    return;
                }
            }
            // SAFETY: the vector stub just pushed a valid frame at `frame`.
            let elr = unsafe { (*frame).elr };
            let far = FAR_EL1.get();
            panic!(
                "EL0 synchronous fault: ESR_EL1={:#018x} (EC={:#04x}) FAR_EL1={:#018x} ELR_EL1={:#018x}",
                esr, ec, far, elr
            );
        }
        KIND_CUR_SPX_SYNC => {
            // SAFETY: the vector stub just pushed a valid frame at `frame`.
            let elr = unsafe { (*frame).elr };
            let esr = ESR_EL1.get();
            let far = FAR_EL1.get();
            let ec = (esr >> 26) & 0x3f;
            panic!(
                "synchronous exception: ESR_EL1={:#018x} (EC={:#04x}) FAR_EL1={:#018x} ELR_EL1={:#018x}",
                esr, ec, far, elr
            );
        }
        other => {
            let esr = ESR_EL1.get();
            let far = FAR_EL1.get();
            panic!(
                "unexpected exception kind {} ESR_EL1={:#018x} FAR_EL1={:#018x}",
                other, esr, far
            );
        }
    }
}
