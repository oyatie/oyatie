//! Cross-CPU TLB shootdown for the aarch64 Frame (P4·SMP·S4c). Frame code (TCB).
//!
//! Wires the loom-verified [`ksync::shootdown::Shootdown`] H1 3-step protocol
//! (sender marks+signals / receivers observe+invalidate+ack / sender waits all
//! acks) to real hardware: the IPI is a **GIC Software Generated Interrupt**
//! (SGI INTID 0) delivered via `ICC_SGI1R_EL1` (GICv3) or `GICD_SGIR` (GICv2),
//! and each receiver performs a real local TLB invalidation (`tlbi vmalle1`)
//! before acking.
//!
//! ## Why this is needed
//! `cow_fault` / `cow_clone` mutate (and `exit`/`exec` free) PTEs of an address
//! space another CPU may be *actively running*. The local `tlbi vae1` those
//! sites already issue only flushes THIS CPU; a stale entry on ANOTHER CPU then
//! produces a spurious permission fault (the S4a blocker: an EL0 instruction
//! abort, EC=0x20 DFSC=0x0f, after a COW copy re-mapped a page PXN/UXN on one
//! CPU while a sibling kept the old executable mapping cached) — or, worse, on
//! the unmap/free path, silent use of a freed/remapped frame. The shootdown
//! makes every other online CPU drop its stale translations before the sender
//! reuses the page.
//!
//! ## Deadlock freedom (the load-bearing argument)
//! The sender mutates the PTE **under** the `PROCS` lock (`lock_irqsave` masks
//! DAIF.I while held), then RELEASES the lock (restoring DAIF.I), and only THEN
//! calls [`request_and_wait_others`], which spins for acks **with IRQs enabled**.
//! So while a sender waits, it still takes and services other CPUs' shootdown
//! SGIs (and its own periodic tick). A CPU holding `PROCS` finishes its short,
//! non-blocking critical section, releases, unmasks DAIF.I, and only then can
//! take the pending SGI and ack. No CPU ever holds `PROCS` across the wait, so
//! there is no "sender waits for a receiver that is itself a blocked sender"
//! cycle.
//!
//! ## 1-vCPU
//! With one online CPU the target set ("all online CPUs except self") is empty,
//! so [`request_and_wait_others`] is a no-op and the existing local `tlbi vae1`
//! at each site is the entire flush — byte-identical to pre-S4c.

use core::sync::atomic::{AtomicBool, Ordering};

use ksync::shootdown::Shootdown;

use hal::cpu::MAX_CPUS;

/// The SGI INTID used as the TLB-shootdown IPI. INTID 0 is a Software Generated
/// Interrupt (SGI) on both GICv2 and GICv3; it is enabled by default on the
/// redistributor/distributor SGI window (SGIs 0..15 are always-enabled Group1
/// on the QEMU `virt` board) so no extra `ISENABLER0` programming is required
/// beyond what the per-CPU bring-up already does for the timer PPI.
pub const SHOOTDOWN_SGI: u32 = 0;

/// Whether the proactive cross-CPU SGI shootdown actively SENDS (vs being a
/// no-op that relies on the per-site local `tlbi vae1`).
///
/// **Baseline `true` (send ENABLED) — Posture A, symmetric with the x86 mirror.**
/// The S4c shootdown machinery — the loom-verified H1 rendezvous, the GIC SGI
/// send (v3 `ICC_SGI1R_EL1` / v2 `GICD_SGIR`), the `tlbi vmalle1` receiver, the
/// deadlock-free sender lock — is fully implemented and wired, and is now ON.
///
/// ## Why enabled (the prior `false` blocker is resolved)
/// This was previously gated OFF, blamed on an intermittent `-smp 4` talos fault.
/// That fault was root-caused and FIXED by F-0020 (commit `43b64eaa`): an x86
/// interrupt gate clears `IF`/`TF` but NOT the direction flag `DF`, and musl's
/// `memmove` leaves `DF=1`, so the trampoline's `rep movs` context copy ran
/// BACKWARD and smeared the kernel stack. The fix is a `cld` at the top of the
/// interrupt-entry trampolines. That fault reproduced with `SHOOTDOWN_SEND_ENABLED`
/// both `true` and `false`, so it was NEVER the TLB shootdown — the old
/// "non-TLB SMP scheduler/state race" / "`-smp 4` regresses" justification is
/// stale and is deleted here. With F-0020 landed there is no measured reason to
/// gate the send off, and on aarch64 a stale sibling TLB entry after a COW
/// write-protect / unmap-free is a real correctness hazard the local `tlbi vae1`
/// alone cannot close, so the cross-CPU shootdown must run. Re-verified by the
/// full gate suite (loom H1 + teeth, check-tcb, diff-oracle ×2, assert-smp-boot
/// ×2 +gicv3 with the positive `shootdown: cpu N invalidated` marker, stress).
const SHOOTDOWN_SEND_ENABLED: bool = true;

/// The one global shootdown rendezvous, seeded with the live online mask the
/// first time a shootdown is requested (the mask is set once at S3 boot and only
/// read thereafter — `crate::smp::online_mask`). Stored in an `Option` behind a
/// one-shot init flag so construction (which reads the online mask) happens after
/// the APs have published their online bits.
static mut SHOOTDOWN: Option<Shootdown> = None;
/// Construction claim: the CPU that flips this `false`->`true` (AcqRel) owns
/// building [`SHOOTDOWN`]. Other CPUs spin on [`SHOOTDOWN_READY`] instead.
static SHOOTDOWN_CLAIMED: AtomicBool = AtomicBool::new(false);
/// Publication flag: set `true` (Release) by the constructor AFTER `SHOOTDOWN` is
/// fully written. A reader that observes this `true` (Acquire) is guaranteed the
/// instance is published — this is the happens-before edge that makes the
/// `static mut` read sound.
static SHOOTDOWN_READY: AtomicBool = AtomicBool::new(false);

/// Lazily build the global [`Shootdown`] from the current online mask, returning
/// a shared reference. Safe to call from any CPU: the first caller wins the
/// `SHOOTDOWN_READY` CAS and constructs it; later callers observe the published
/// instance. Construction only ever reads immutable online bits.
fn shootdown() -> &'static Shootdown {
    // Fast path: already published.
    if SHOOTDOWN_READY.load(Ordering::Acquire) {
        // SAFETY: once `SHOOTDOWN_READY` is observed `true` (Acquire), the
        // `SHOOTDOWN` Option was fully written (Release in the init below) and is
        // never mutated again — only `&` shared reads remain, which is sound.
        return unsafe { (*core::ptr::addr_of!(SHOOTDOWN)).as_ref().unwrap_unchecked() };
    }
    init_shootdown()
}

/// Slow-path one-shot constructor. Builds the online-mask array and publishes the
/// instance, winning a CAS so exactly one CPU constructs it.
#[cold]
fn init_shootdown() -> &'static Shootdown {
    let mask = crate::smp::online_mask();
    let mut online = [false; ksync::shootdown::SHOOTDOWN_MAX_CPUS];
    for (cpu, slot) in online.iter_mut().enumerate() {
        *slot = mask & (1u64 << cpu) != 0;
    }
    // Race to claim construction: the winner builds `SHOOTDOWN`, THEN publishes
    // (Release on `SHOOTDOWN_READY`); losers spin on `SHOOTDOWN_READY`.
    if SHOOTDOWN_CLAIMED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        // SAFETY: we won the one-shot claim, so we are the sole writer of
        // `SHOOTDOWN`. No other CPU reads it until it observes `SHOOTDOWN_READY`,
        // which we publish (Release) only AFTER the write below completes.
        unsafe {
            *core::ptr::addr_of_mut!(SHOOTDOWN) = Some(Shootdown::new(online));
        }
        SHOOTDOWN_READY.store(true, Ordering::Release);
    } else {
        // Lost the race: wait until the winner has published the instance.
        while !SHOOTDOWN_READY.load(Ordering::Acquire) {
            core::hint::spin_loop();
        }
    }
    // SAFETY: `SHOOTDOWN_READY` is now observed `true` (Acquire), which the
    // constructor stored AFTER fully writing `SHOOTDOWN`; the instance is
    // published and immutable thereafter, so a shared read is sound.
    unsafe { (*core::ptr::addr_of!(SHOOTDOWN)).as_ref().unwrap_unchecked() }
}

/// This CPU's logical index, from the per-CPU `TPIDR_EL1` anchor.
#[inline]
fn this_cpu() -> usize {
    // SAFETY: called from EL1 contexts where the per-CPU anchor is installed (the
    // same invariant `this_cpu_token` requires); a side-effect-free sysreg read.
    let token = unsafe { crate::percpu::this_cpu_token() };
    token.cpu_index()
}

/// Collect the online CPUs OTHER than `self_cpu` into `buf`, returning the count.
/// Over-approximate by design (all-online): shooting a CPU that does not actually
/// cache the AS just wastes one SGI; missing one would be corruption.
fn other_online(self_cpu: usize, buf: &mut [usize; MAX_CPUS]) -> usize {
    let mask = crate::smp::online_mask();
    let mut n = 0;
    for cpu in 0..MAX_CPUS {
        if cpu != self_cpu && (mask & (1u64 << cpu)) != 0 {
            buf[n] = cpu;
            n += 1;
        }
    }
    n
}

/// SENDER: request a TLB shootdown on every online CPU OTHER than this one, then
/// wait (IRQs ENABLED) for each to ack. The caller MUST have already mutated the
/// PTE(s) and RELEASED the `PROCS` lock (so DAIF.I is restored) before calling
/// this — see the module deadlock-freedom note. The local CPU's own invalidation
/// is done directly at the call site (the existing `tlbi vae1`), so this only
/// targets the *other* CPUs.
///
/// No-op on 1-vCPU (no other online CPU) — preserving the byte-identical golden.
pub fn request_and_wait_others() {
    if !SHOOTDOWN_SEND_ENABLED {
        return;
    }
    let self_cpu = this_cpu();
    let mut buf = [0usize; MAX_CPUS];
    let n = other_online(self_cpu, &mut buf);
    if n == 0 {
        return; // 1-vCPU (or alone online): the local invalidate already sufficed.
    }

    // Save + UNMASK DAIF.I for the whole shootdown: the sender lock-spin AND the
    // ack-spin must run with IRQs enabled so this CPU keeps SERVICING inbound
    // shootdown SGIs (a target may itself be a blocked sender waiting on US) and
    // never deadlocks. This is called from BOTH the EL0 syscall path (IRQs
    // maskable) AND the EL0 sync-abort COW path (DAIF.I masked on entry), so we
    // explicitly unmask here and restore the prior state at the end.
    let daif: u64;
    // SAFETY: side-effect-free DAIF read (bit 7 = I mask).
    unsafe {
        core::arch::asm!("mrs {x}, daif", x = out(reg) daif, options(nostack, nomem));
    }
    // SAFETY: unmask IRQs (DAIFClr.I).
    unsafe {
        core::arch::asm!("msr daifclr, #2", options(nostack, nomem));
    }

    // Serialize senders: the loom-verified H1 `Shootdown` is a SINGLE-SENDER
    // protocol (one sender per rendezvous). Two CPUs concurrently shooting the
    // SAME target collide on that target's single `pending`/`acks` bit (one
    // sender consumes the ack the other is still waiting for → lost ack). A
    // global sender lock makes every shootdown single-sender (loom-valid). We
    // spin for it WITH IRQs ENABLED, so a CPU waiting to become a sender still
    // services the current sender's SGI + acks → no deadlock.
    acquire_sender_lock();

    // Ensure our PTE edits (made under the now-released PROCS lock) are globally
    // visible before the receivers re-walk the page tables.
    // SAFETY: a barrier with no other effect.
    unsafe {
        core::arch::asm!("dsb ish", options(nostack, nomem, preserves_flags));
    }

    let sd = shootdown();
    let targets = &buf[..n];
    // H1 step 1: publish the pending bits FIRST (Release) so a target woken by
    // the SGI is guaranteed to observe its bit (no lost wakeup).
    sd.publish(targets);
    // Send the wake SGI to each target now that its bit is set.
    for &cpu in targets {
        send_sgi_to(cpu, SHOOTDOWN_SGI);
    }
    // H1 step 3: wait for every target's ack (IRQs enabled, see above).
    sd.wait_all(targets);

    release_sender_lock();

    // Restore the prior DAIF.I (re-mask iff it was masked on entry).
    if (daif & (1 << 7)) != 0 {
        // SAFETY: re-mask IRQs to the prior state.
        unsafe {
            core::arch::asm!("msr daifset, #2", options(nostack, nomem));
        }
    }
}

/// Global shootdown-sender lock (a plain test-and-set spinlock). Held only across
/// the H1 3-step so the single-sender `Shootdown` is never driven by two senders
/// at once. Acquired/released with IRQs ENABLED at the caller (so a CPU spinning
/// for it still services the current sender's shootdown SGI).
static SENDER_LOCK: AtomicBool = AtomicBool::new(false);

/// Acquire [`SENDER_LOCK`] (spin, IRQs must be enabled by the caller). Acquire
/// ordering: pairs with the [`release_sender_lock`] Release so the previous
/// sender's H1 handshake (incl. its ack consumption that resets the rendezvous)
/// happens-before this sender publishes.
fn acquire_sender_lock() {
    while SENDER_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

/// Release [`SENDER_LOCK`] (Release: publishes this sender's completed handshake
/// — all acks consumed, bits reset — to the next acquirer).
fn release_sender_lock() {
    SENDER_LOCK.store(false, Ordering::Release);
}

/// RECEIVER: called from the SGI IRQ handler. If this CPU has a pending
/// shootdown, perform a full local TLB invalidation (`tlbi vmalle1`) and ack.
/// A full flush (rather than a per-VA `tlbi vae1`) is the safe simple floor: the
/// broadcast does not carry a VA range, and `cow_clone` shoots down many VAs at
/// once, so invalidating the whole EL1&0 TLB for this CPU is correct and cheap
/// under TCG.
pub fn service_on_sgi() {
    let cpu = this_cpu();
    let sd = shootdown();
    let invalidated = sd.poll_and_invalidate(cpu, || {
        // SAFETY: architectural TLB maintenance — invalidate ALL stage-1 EL1&0
        // entries for this CPU. Sound regardless of which AS is current; it only
        // drops cached translations, forcing fresh page-table walks.
        unsafe {
            core::arch::asm!(
                "dsb ish",
                "tlbi vmalle1",
                "dsb ish",
                "isb",
                options(nostack, nomem, preserves_flags),
            );
        }
    });
    // P4·SMP·S4c POSITIVE GATE MARKER: emit a one-line receiver-side proof that
    // the cross-CPU shootdown actually FIRED (a remote sender's SGI landed here
    // and this CPU invalidated its TLB before acking), turning the gate from a
    // clean-reap PROXY into a positive "the IPI/SGI path executed" assertion
    // (`scripts/assert-smp-boot.sh`). Gated behind the `smp-sched-demo` carrier
    // so it never perturbs the byte-identical golden / talos boot path.
    #[cfg(feature = "smp-sched-demo")]
    if invalidated {
        crate::kprintln!("shootdown: cpu {} invalidated", cpu);
    }
    #[cfg(not(feature = "smp-sched-demo"))]
    let _ = invalidated;
}

// ---------------------------------------------------------------------------
// GIC SGI delivery (the IPI primitive). v3 via ICC_SGI1R_EL1, v2 via GICD_SGIR.
// ---------------------------------------------------------------------------

/// GICv2 distributor base (same as `crate::gic`), for the `GICD_SGIR` write.
const GICD_BASE: usize = 0x0800_0000;
/// `GICD_SGIR` offset in the GICv2 distributor (Software Generated Interrupt
/// register). Writing it generates an SGI to a target CPU list.
const GICD_SGIR_OFFSET: usize = 0xF00;

/// Send SGI `intid` to the single CPU with logical index `cpu`.
///
/// On QEMU `virt` the logical CPU index equals Aff0 (flat topology, confirmed in
/// `smp.rs`'s enumeration), so the GICv3 `ICC_SGI1R_EL1` target-list bit and the
/// GICv2 `GICD_SGIR` CPU-target byte are both `1 << cpu`.
fn send_sgi_to(cpu: usize, intid: u32) {
    match crate::gicv3::active_gic() {
        crate::gicv3::GicVersion::V3 => {
            // ICC_SGI1R_EL1 payload: Aff3[55:48] | Aff2[39:32] | Aff1[23:16] |
            // INTID[27:24] | TargetList[15:0]. Flat QEMU topology ⇒ Aff1..3 = 0,
            // Aff0 == cpu, so the 16-bit target list bit is `1 << cpu`.
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
                core::ptr::write_volatile(
                    (GICD_BASE + GICD_SGIR_OFFSET) as *mut u32,
                    value,
                );
            }
        }
    }
}
