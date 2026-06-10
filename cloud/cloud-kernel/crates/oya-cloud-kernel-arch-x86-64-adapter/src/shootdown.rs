//! Cross-CPU TLB shootdown for the x86_64 Frame (P4·SMP·S4c). Frame code (TCB).
//!
//! The x86 analog of `crate::arch_aarch64::shootdown`: wires the loom-verified
//! [`ksync::shootdown::Shootdown`] H1 3-step protocol to real hardware. x86 has
//! NO architected cross-CPU TLB-invalidation instruction (`invlpg` is local
//! only), so the cross-CPU shootdown is MANDATORY — a sender mutating a PTE of an
//! address space another CPU runs MUST IPI that CPU to invalidate, or it reads
//! through a stale translation (silent corruption on the unmap/free path).
//!
//! The IPI is a fixed-delivery x2APIC IPI (IDT vector [`SHOOTDOWN_VECTOR`]) sent
//! via the ICR MSR (`crate::apic::send_fixed_ipi`); the receiver ISR
//! ([`service_on_ipi`]) reloads CR3 (a full local TLB flush) and acks.
//!
//! ## Deadlock freedom
//! Identical to the aarch64 module: the sender mutates the PTE UNDER the `PROCS`
//! lock (`lock_irqsave` masks IF while held), RELEASES the lock (restoring IF),
//! then [`request_and_wait_others`] spins for acks WITH IF ENABLED, so it keeps
//! servicing inbound shootdown IPIs while waiting. A global sender lock
//! serializes shootdowns so the single-sender H1 rendezvous is never driven by
//! two senders at once (two senders sharing a target's single `pending`/`acks`
//! bit would lose an ack). A CPU spinning to become a sender does so with IF
//! enabled, so it still acks the current sender's IPI → no cycle.
//!
//! ## 1-vCPU
//! With one online CPU the target set ("all online except self") is empty, so
//! [`request_and_wait_others`] is a no-op and the existing local `invlpg` at each
//! site is the whole flush — byte-identical to pre-S4c. The default x86 golden/
//! talos runs use `-cpu qemu64` (NO x2apic) and never start an AP, so this code
//! never sends an IPI there.

use core::sync::atomic::{AtomicBool, Ordering};

use ksync::shootdown::Shootdown;

use hal::cpu::MAX_CPUS;

/// IDT vector for the cross-CPU TLB-shootdown IPI. `0xF1`, in the
/// user/software-vector range above the timer (`0x20`) and below the spurious
/// vector (`0xFF`); registered in `crate::interrupts`'s IDT.
pub const SHOOTDOWN_VECTOR: u8 = 0xF1;

/// The one global shootdown rendezvous, seeded with the online mask the first
/// time a shootdown is requested. See the aarch64 mirror for the publication
/// discipline.
static mut SHOOTDOWN: Option<Shootdown> = None;
/// Construction claim flag (the CPU that flips it false->true builds SHOOTDOWN).
static SHOOTDOWN_CLAIMED: AtomicBool = AtomicBool::new(false);
/// Publication flag (set Release after SHOOTDOWN is written; read Acquire).
static SHOOTDOWN_READY: AtomicBool = AtomicBool::new(false);
/// Global sender lock (serializes shootdowns so H1 stays single-sender).
static SENDER_LOCK: AtomicBool = AtomicBool::new(false);

/// Whether the proactive cross-CPU IPI shootdown actively SENDS (vs being a
/// no-op that relies on the per-site local `invlpg` + the per-switch CR3 flush).
///
/// **Baseline `true` (send ENABLED) — Posture A, symmetric with the aarch64
/// mirror.** The S4c shootdown machinery — the loom-verified H1 rendezvous, the
/// fixed-vector ICR IPI, the CR3-reload receiver, the deadlock-free sender lock —
/// is fully implemented and wired, and is now ON.
///
/// ## Why enabled (mandatory on x86 — and the prior `false` blocker is resolved)
/// x86 has NO architected cross-CPU TLB-invalidation instruction (`invlpg` is
/// local-only), so the IPI shootdown is REQUIRED FOR CORRECTNESS, not perf: the
/// moment an AP runs a forked/COW'd address space whose PTEs another CPU mutates
/// (write-protect on `clone`, unmap-free on `exec`/`exit`/reap), leaving the
/// send `false` is a latent silent-corruption hole — a CPU reads through a stale
/// translation to a freed/remapped frame. Enabling it CLOSES that hole.
///
/// This was previously gated OFF, blamed on an intermittent `-smp 4` talos fault
/// (the old "regresses ~5/6→0/8" note). That fault was root-caused and FIXED by
/// F-0020 (commit `43b64eaa`): an interrupt gate clears `IF`/`TF` but NOT the
/// direction flag `DF`, and musl's `memmove` leaves `DF=1`, so the trampoline's
/// `rep movs` context copy ran BACKWARD and smeared the kernel stack. The fix is
/// a `cld` at the top of the interrupt-entry trampolines. That fault reproduced
/// with `SHOOTDOWN_SEND_ENABLED` both `true` and `false`, so it was NEVER the TLB
/// shootdown — the old "non-TLB race" / "regresses ~5/6→0/8" justification is
/// stale (it measured the DF bug, pre-fix) and is deleted here. Re-verified by
/// the full gate suite (loom H1 + teeth, check-tcb, diff-oracle, assert-smp-boot
/// with the positive `shootdown: cpu N invalidated` marker, 40-run stress).
const SHOOTDOWN_SEND_ENABLED: bool = true;

fn shootdown() -> &'static Shootdown {
    if SHOOTDOWN_READY.load(Ordering::Acquire) {
        // SAFETY: published (Release) + immutable thereafter; shared read is sound.
        return unsafe { (*core::ptr::addr_of!(SHOOTDOWN)).as_ref().unwrap_unchecked() };
    }
    init_shootdown()
}

#[cold]
fn init_shootdown() -> &'static Shootdown {
    let mask = crate::smp::online_mask();
    let mut online = [false; ksync::shootdown::SHOOTDOWN_MAX_CPUS];
    for (cpu, slot) in online.iter_mut().enumerate() {
        *slot = mask & (1u64 << cpu) != 0;
    }
    if SHOOTDOWN_CLAIMED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        // SAFETY: sole writer (won the one-shot claim); published below.
        unsafe {
            *core::ptr::addr_of_mut!(SHOOTDOWN) = Some(Shootdown::new(online));
        }
        SHOOTDOWN_READY.store(true, Ordering::Release);
    } else {
        while !SHOOTDOWN_READY.load(Ordering::Acquire) {
            core::hint::spin_loop();
        }
    }
    // SAFETY: published + immutable.
    unsafe { (*core::ptr::addr_of!(SHOOTDOWN)).as_ref().unwrap_unchecked() }
}

/// This CPU's logical index (read from `gs:16` via the per-CPU anchor). Callers
/// must have the KERNEL GS base active (the `with_sched`/ISR-with-kernel-gs
/// invariant).
#[inline]
fn this_cpu() -> usize {
    // SAFETY: kernel GS active (guaranteed by every caller: the sender runs in a
    // `with_sched`-shaped context, the receiver brackets in `with_kernel_gs`);
    // a single `gs:[16]` read.
    let token = unsafe { crate::user::this_cpu_token() };
    token.cpu_index()
}

/// Online CPUs OTHER than `self_cpu`, into `buf`; returns the count.
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

/// SENDER: request a TLB shootdown on every online CPU OTHER than this one and
/// wait (IF ENABLED) for each ack. The caller MUST have mutated the PTE(s) and
/// RELEASED the `PROCS` lock first (see the module note). No-op on 1-vCPU.
pub fn request_and_wait_others() {
    if !SHOOTDOWN_SEND_ENABLED {
        return;
    }
    let self_cpu = this_cpu();
    let mut buf = [0usize; MAX_CPUS];
    let n = other_online(self_cpu, &mut buf);
    if n == 0 {
        return; // 1-vCPU / alone online: local `invlpg` already sufficed.
    }

    // Save IF, then ENABLE interrupts for the whole shootdown (lock-spin + ack-
    // spin) so this CPU keeps servicing inbound shootdown IPIs while waiting.
    let if_was_enabled = x86_64::instructions::interrupts::are_enabled();
    x86_64::instructions::interrupts::enable();

    // Serialize senders so the single-sender H1 rendezvous is never shared.
    acquire_sender_lock();

    // Order the PTE edits (made under the released PROCS lock) before the IPIs.
    core::sync::atomic::fence(Ordering::SeqCst);

    let sd = shootdown();
    let targets = &buf[..n];
    sd.publish(targets); // H1 step 1: pending bits Release.
    for &cpu in targets {
        let apic_id = crate::smp::apic_id_of(cpu);
        // SAFETY: x2APIC enabled (an AP is online ⇒ x2APIC was enabled on the BSP
        // + each AP); `SHOOTDOWN_VECTOR` is a registered IDT gate that EOIs.
        unsafe { crate::apic::send_fixed_ipi(apic_id, SHOOTDOWN_VECTOR) };
    }
    sd.wait_all(targets); // H1 step 3: wait acks (IF enabled).

    release_sender_lock();

    // Restore the prior IF (re-disable iff it was disabled on entry).
    if !if_was_enabled {
        x86_64::instructions::interrupts::disable();
    }
}

fn acquire_sender_lock() {
    while SENDER_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

fn release_sender_lock() {
    SENDER_LOCK.store(false, Ordering::Release);
}

/// RECEIVER: the shootdown-IPI ISR. If this CPU has a pending shootdown, flush
/// its TLB by reloading CR3 (the architected full local flush) and ack. A CR3
/// reload is the safe simple floor: the broadcast carries no VA range and
/// `cow_clone` shoots many VAs at once, so a full flush is correct + cheap.
pub fn service_on_ipi() {
    let cpu = this_cpu();
    let sd = shootdown();
    let invalidated = sd.poll_and_invalidate(cpu, || {
        // SAFETY: reload CR3 with its current value — a full local TLB flush for
        // non-global pages. Reads + writes the live CR3; affects only the TLB.
        unsafe {
            let (frame, flags) = x86_64::registers::control::Cr3::read();
            x86_64::registers::control::Cr3::write(frame, flags);
        }
    });
    // P4·SMP·S4c POSITIVE GATE MARKER: emit a one-line receiver-side proof that
    // the cross-CPU shootdown actually FIRED (a remote sender's IPI landed here
    // and this CPU flushed its TLB before acking), turning the gate from a
    // clean-reap PROXY into a positive "the IPI path executed" assertion
    // (`scripts/assert-smp-boot.sh`). Gated behind the `smp-sched-demo` carrier
    // so it never perturbs the byte-identical golden / talos boot path.
    #[cfg(feature = "smp-sched-demo")]
    if invalidated {
        crate::kprintln!("shootdown: cpu {} invalidated", cpu);
    }
    #[cfg(not(feature = "smp-sched-demo"))]
    let _ = invalidated;
}
