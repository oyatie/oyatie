//! Reschedule (wake-an-idle-CPU) IPI for the aarch64 Frame (P4·SMP·S4b). Frame
//! code (TCB).
//!
//! The aarch64 mirror of `crate::reschedule` on x86: when work lands on an idle
//! CPU's run queue — a `fork` placed a child there, a `wait4` woke a parent
//! assigned there, or a stealer wants a parked victim to re-evaluate — the placing
//! CPU sends a **reschedule SGI** so the target leaves `wfe` and re-runs its
//! idle→schedule loop *now* instead of waiting up to one periodic tick. Pure
//! liveness/latency: a missed reschedule SGI only means the target wakes on its
//! next timer tick (slower, never incorrect), so unlike the S4c shootdown there
//! is NO rendezvous and NO ack — taking the SGI is the wake.
//!
//! ## Delivery + INTID choice
//! A GIC **Software Generated Interrupt** delivered via `ICC_SGI1R_EL1` (GICv3) or
//! `GICD_SGIR` (GICv2) — the same `crate::gicv3::send_sgi` primitive the S4c
//! shootdown uses. The **shootdown already occupies SGI INTID 0** (landed in S4c,
//! `crate::shootdown::SHOOTDOWN_SGI`), so the reschedule SGI takes **INTID 1**.
//! (The S4 plan §4.3 assigned the two the other way around — reschedule 0,
//! shootdown 1 — but S4c landed the shootdown on 0; using 1 here keeps the landed,
//! golden-verified shootdown wiring untouched, which is the minimal-diff choice.
//! SGIs 0..15 are always-enabled Group1 on the QEMU `virt` board, so INTID 1
//! needs no extra `ISENABLER0` programming beyond the per-CPU bring-up.)
//!
//! ## The IPI-vs-idle lost-wakeup race (§3.4)
//! `wfe` wakes on any pending SGI/IRQ even if it was delivered just before the
//! `wfe`, and the WFE **event register** makes a `wfe` after a set-event fall
//! through — so the SGI sent in the window between "AP checked its empty queue"
//! and "AP executed wfe" is not lost. A spurious wakeup is harmless (re-check
//! finds nothing, idles again). The work (`runq_push`) is published under the
//! `PROCS` lock before the SGI is sent, so a woken AP observes it.
//!
//! ## 1-vCPU
//! One online CPU ⇒ no *other* CPU to target: [`notify`] returns immediately, no
//! SGI is ever sent, the module is inert — byte-identical to pre-S4b.

/// The SGI INTID used for the reschedule IPI. INTID **1** (INTID 0 is the landed
/// S4c shootdown SGI). A Software Generated Interrupt, always-enabled Group1 on
/// the QEMU `virt` board.
pub const RESCHED_SGI: u32 = 1;

/// SENDER: wake CPU `target_cpu` (if online and not this CPU) so it re-enters its
/// idle→schedule loop and picks up the work just placed on its run queue. A no-op
/// when `target_cpu` is this CPU or offline, or on 1-vCPU. Safe to call from the
/// safe kernel via the HAL `IrqChip::send_ipi` seam; the `unsafe` `ICC_SGI1R_EL1`
/// / `GICD_SGIR` write stays in [`crate::gicv3::send_sgi`].
///
/// The caller MUST have already published the work (`runq_push` under the `PROCS`
/// lock) before sending the SGI, so the woken CPU observes it.
pub fn notify(target_cpu: usize) {
    let mask = crate::smp::online_mask();
    if mask.count_ones() <= 1 {
        return; // 1-vCPU: no other CPU to wake.
    }
    let self_cpu = this_cpu();
    if target_cpu == self_cpu {
        return; // we are already in the scheduler.
    }
    if mask & (1u64 << target_cpu) == 0 {
        return; // target offline.
    }
    send_sgi_to(target_cpu, RESCHED_SGI);
}

/// This CPU's logical index, from the per-CPU `TPIDR_EL1` anchor.
#[inline]
fn this_cpu() -> usize {
    // SAFETY: called from EL1 contexts where the per-CPU anchor is installed (the
    // same invariant `this_cpu_token`/`crate::shootdown::this_cpu` require).
    let token = unsafe { crate::percpu::this_cpu_token() };
    token.cpu_index()
}

/// GICv2 distributor base (same as `crate::gic` / `crate::shootdown`), for the
/// `GICD_SGIR` write on the v2 leg.
const GICD_BASE: usize = 0x0800_0000;
/// `GICD_SGIR` offset in the GICv2 distributor (Software Generated Interrupt
/// register). Writing it generates an SGI to a target CPU list.
const GICD_SGIR_OFFSET: usize = 0xF00;

/// Send SGI `intid` to the single CPU with logical index `cpu`. Mirrors
/// `crate::shootdown::send_sgi_to` (flat QEMU `virt` topology ⇒ Aff0 == cpu, so
/// the GICv3 target-list bit and the GICv2 CPU-target byte are both `1 << cpu`).
fn send_sgi_to(cpu: usize, intid: u32) {
    match crate::gicv3::active_gic() {
        crate::gicv3::GicVersion::V3 => {
            // ICC_SGI1R_EL1: INTID[27:24] | TargetList[15:0]; flat topology ⇒
            // Aff1..3 = 0, Aff0 == cpu, so the target-list bit is `1 << cpu`.
            let value = ((intid as u64 & 0xF) << 24) | (1u64 << (cpu & 0xF));
            // SAFETY: a single architected ICC_SGI1R_EL1 write generating the SGI;
            // `crate::gicv3::send_sgi` is the audited primitive.
            unsafe { crate::gicv3::send_sgi(value) };
        }
        crate::gicv3::GicVersion::V2 => {
            // GICD_SGIR: TargetListFilter[25:24]=00 (use list), CPUTargetList
            // [23:16] = 1<<cpu, SGIINTID[3:0] = intid.
            let value: u32 = ((1u32 << (cpu & 0x7)) << 16) | (intid & 0xF);
            // SAFETY: a single MMIO write to the GICv2 distributor's SGI-generate
            // register, inside the identity-mapped device block.
            unsafe {
                core::ptr::write_volatile((GICD_BASE + GICD_SGIR_OFFSET) as *mut u32, value);
            }
        }
    }
}
