//! Reschedule (wake-an-idle-CPU) IPI for the x86_64 Frame (P4·SMP·S4b). Frame
//! code (TCB).
//!
//! When work lands on an idle CPU's run queue — a `fork` placed a child there, a
//! `wait4` woke a parent assigned there, or a stealer wants a parked victim to
//! re-evaluate — the placing CPU sends this CPU a **reschedule IPI** so it leaves
//! `hlt` and re-runs its idle→schedule loop *now* instead of waiting up to one
//! periodic tick. It is a pure liveness/latency primitive: a missed reschedule
//! IPI only means the target wakes on its next timer tick instead (slower, never
//! incorrect), so unlike the S4c shootdown there is NO rendezvous and NO ack —
//! the mere act of taking the IRQ is the wake.
//!
//! ## Delivery
//! A Fixed-delivery x2APIC IPI (IDT vector [`RESCHED_VECTOR`] = `0xF0`) sent via
//! the ICR MSR `0x830` — the same path `apic::send_init_sipi` uses for SIPI and
//! `shootdown` uses for the shootdown IPI, but Fixed delivery (`apic::send_fixed_ipi`).
//! The receiver ISR ([`crate::interrupts`]) does nothing but EOI the APIC: a
//! halted AP `sti; hlt` is awoken by the delivered interrupt and falls back into
//! [`crate::user::ap_run_scheduler`], where it re-pops/steals.
//!
//! ## The IPI-vs-idle lost-wakeup race (§3.4)
//! The AP idles with the standard `sti; hlt` (atomic w.r.t. an IRQ delivered just
//! before the `hlt`, because `sti` only enables IRQs after the *next* instruction
//! boundary), and re-checks its queue with IRQs disabled before idling. So an IPI
//! sent in the window between "AP checked its empty queue" and "AP executed hlt"
//! is taken immediately by the `sti; hlt`, not lost. A spurious wakeup (the work
//! was already taken by a tick) is harmless: the re-check finds nothing and idles
//! again. Sender side: the work (the `runq_push`) is published under the `PROCS`
//! lock BEFORE the IPI is sent, so a woken AP that re-takes the lock observes it.
//!
//! ## 1-vCPU
//! With one online CPU there is no *other* CPU to target: [`notify`] returns
//! immediately (the only set bit of `online_mask()` is self), no IPI is ever
//! sent, and this module is inert — byte-identical to pre-S4b. The default x86
//! golden/talos runs use `-cpu qemu64` (no x2apic) and never start an AP, so this
//! code never sends an IPI there.

/// IDT vector for the reschedule IPI. `0xF0`, in the user/software-vector range
/// above the timer (`0x20`) and below the shootdown vector (`0xF1`) and the
/// spurious vector (`0xFF`); registered in `crate::interrupts`'s IDT.
pub const RESCHED_VECTOR: u8 = 0xF0;

/// SENDER: wake CPU `target_cpu` (if it is online and not this CPU) so it
/// re-enters its idle→schedule loop and picks up the work just placed on its run
/// queue. A no-op when `target_cpu` is this CPU (it is already scheduling) or
/// offline. Safe to call from the safe kernel via the HAL `IrqChip::send_ipi`
/// seam; the `unsafe` ICR MSR write stays in [`crate::apic::send_fixed_ipi`].
///
/// The caller MUST have already published the work (`runq_push` under the `PROCS`
/// lock) before sending the IPI, so the woken CPU observes it.
pub fn notify(target_cpu: usize) {
    let mask = crate::smp::online_mask();
    // Only one CPU online ⇒ no other CPU to wake (1-vCPU short-circuit).
    if mask.count_ones() <= 1 {
        return;
    }
    let self_cpu = this_cpu();
    if target_cpu == self_cpu {
        return; // we are already in the scheduler; no IPI to ourselves.
    }
    if mask & (1u64 << target_cpu) == 0 {
        return; // target offline — nothing to wake.
    }
    let apic_id = crate::smp::apic_id_of(target_cpu);
    // SAFETY: x2APIC is enabled (an AP is online ⇒ x2APIC was enabled on the BSP
    // + each AP); `RESCHED_VECTOR` is a registered IDT gate that EOIs the APIC.
    unsafe { crate::apic::send_fixed_ipi(apic_id, RESCHED_VECTOR) };
}

/// This CPU's logical index (read from `gs:16` via the per-CPU anchor). Callers
/// run with the kernel GS base active (the `with_sched`/ISR-with-kernel-gs
/// invariant), exactly as `crate::shootdown::this_cpu` requires.
#[inline]
fn this_cpu() -> usize {
    // SAFETY: kernel GS active (the sender runs in a `with_sched`-shaped context);
    // a single `gs:[16]` read.
    let token = unsafe { crate::user::this_cpu_token() };
    token.cpu_index()
}
