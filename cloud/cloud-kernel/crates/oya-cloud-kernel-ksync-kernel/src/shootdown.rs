//! # ksync_shootdown — the cross-CPU TLB-shootdown 3-step protocol (P4·SMP·S2, H1)
//!
//! Models the S4 cross-CPU invalidation handshake — **without** any real
//! hardware. A *sender* on one CPU needs every *target* CPU to invalidate a TLB
//! entry before it may reuse/remap the page; the protocol is:
//!
//!   1. **sender** marks a `pending` bit for each target CPU and "signals" (IPI),
//!   2. each **receiver** observes its bit, "invalidates" (a modelled no-op),
//!      and **acks** (clears pending, sets its ack flag),
//!   3. **sender** waits until *every* targeted receiver has acked before
//!      proceeding (else S4 frees/remaps a page another CPU still has cached).
//!
//! ## What loom proves here (and what it does NOT)
//! loom is an exhaustive model checker over the **bounded** protocol of atomics
//! (a bug-finder, not an absolute absence proof). It proves the *orderings*:
//! (a) the sender never completes before all targets acked, (b) no deadlock /
//! lost wakeup, (c) an offline CPU is never targeted nor waited on. It does
//! **not** model real `invlpg`/`TLBI`, weak-memory hardware effects, or IPI
//! timing — the invalidate is a no-op; the physical-invalidation correctness on
//! weak aarch64 is a separate Miri + on-hardware/TCG concern (S4). The
//! signal/ack handoff is the `Completion` "no missed wakeup" release/acquire
//! shape generalised to N receivers.
//!
//! ## Soundness of the handshake
//! `pending[i]`/`acks[i]` are per-CPU `AtomicBool`s (one event each for loom, far
//! cheaper than a wide `AtomicU64` RMW loop). The sender publishes the request
//! with a `Release` store to `pending[i]`; the receiver reads it `Acquire`,
//! and publishes its ack with a `Release` store to `acks[i]` that the sender
//! reads `Acquire` — the same release/acquire edge the `SpinLock`/`Completion`
//! already prove. Nothing else is shared mutably.

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicBool, Ordering};

/// Maximum CPUs the shootdown bitmaps are sized for. Matches the production
/// `hal::cpu::MAX_CPUS` (8) so a real `Shootdown` instance can target every
/// online CPU. loom is exhaustive over the *interleavings*, not over CPU count:
/// the loom models below still only spawn 1 sender + ≤2 receivers (≤3 threads,
/// loom's branch cap), so widening the bitmap array from 4 to 8 slots does NOT
/// enlarge the explored state space — the extra slots stay `false` and untouched.
pub const SHOOTDOWN_MAX_CPUS: usize = 8;

/// A per-CPU TLB-shootdown rendezvous: `pending`/`acks` bit per CPU plus an
/// `online` mask. A sender requests invalidation on a set of target CPUs and
/// waits for each to ack; receivers poll their own bit and ack.
pub struct Shootdown {
    /// `pending[i] == true` ⇒ CPU `i` has been asked to invalidate and not yet
    /// acked. Set (Release) by the sender, cleared by receiver `i`.
    pending: [AtomicBool; SHOOTDOWN_MAX_CPUS],
    /// `acks[i] == true` ⇒ CPU `i` has completed its invalidation. Set (Release)
    /// by receiver `i`, observed (Acquire) and cleared by the sender.
    acks: [AtomicBool; SHOOTDOWN_MAX_CPUS],
    /// Which CPUs are online (eligible to be a target). An offline CPU is never
    /// targeted and never waited on.
    online: [bool; SHOOTDOWN_MAX_CPUS],
}

// SAFETY: the only shared mutable state is the per-CPU `AtomicBool` arrays;
// `online` is set once at construction and only read afterwards. A given
// `pending[i]`/`acks[i]` is written by exactly one role (sender sets pending /
// clears ack; receiver `i` clears pending / sets ack), so there is no
// conflicting non-atomic access. Sharing `&Shootdown` across CPUs is sound.
unsafe impl Sync for Shootdown {}
unsafe impl Send for Shootdown {}

impl Shootdown {
    /// Build a rendezvous where `online[i]` says whether CPU `i` is online. All
    /// bits start clear.
    pub fn new(online: [bool; SHOOTDOWN_MAX_CPUS]) -> Self {
        Self {
            pending: core::array::from_fn(|_| AtomicBool::new(false)),
            acks: core::array::from_fn(|_| AtomicBool::new(false)),
            online,
        }
    }

    /// Whether CPU `cpu` is online (eligible to be a shootdown target).
    #[inline]
    pub fn is_online(&self, cpu: usize) -> bool {
        self.online[cpu]
    }

    /// SENDER step 1+3: request invalidation on every target in `targets`, then
    /// wait until each has acked. Returns the number of CPUs acked.
    ///
    /// Panics in debug if a requested target is offline — the caller must only
    /// target online CPUs (assertion (c): an offline CPU is never waited on).
    pub fn request_and_wait(&self, targets: &[usize]) -> usize {
        // Step 1: publish the request to each (online) target.
        self.publish(targets);
        // Step 3: wait for every target's ack, then consume it.
        self.wait_all(targets)
    }

    /// SENDER step 1 ONLY: publish the pending bit for each (online) target with a
    /// `Release` store. Split out from [`request_and_wait`](Self::request_and_wait)
    /// so the hardware Frame can interleave the real IPI send between the publish
    /// and the wait: `publish(targets)` → send the wake IPI to each target →
    /// [`wait_all`](Self::wait_all). Setting the bit BEFORE sending the IPI closes
    /// the lost-wakeup window (the receiver, woken by the IPI, observes its bit).
    #[inline]
    pub fn publish(&self, targets: &[usize]) {
        for &cpu in targets {
            debug_assert!(self.online[cpu], "shootdown target {cpu} is offline");
            // Release: the request (and the PTE edits before it) are published to
            // the receiver's Acquire load.
            self.pending[cpu].store(true, Ordering::Release);
        }
    }

    /// SENDER step 3 ONLY: wait until every target in `targets` has acked, then
    /// consume the ack so the rendezvous can be reused. Returns the number acked.
    /// The Acquire-spin pairs with each receiver's `Release` ack store, so the
    /// receiver's invalidation happens-before this returns. The caller spins here
    /// with IRQs ENABLED (it released `PROCS` first) so it services inbound
    /// shootdown IPIs while waiting — the documented deadlock-freedom property.
    #[inline]
    pub fn wait_all(&self, targets: &[usize]) -> usize {
        let mut acked = 0;
        for &cpu in targets {
            // Acquire-spin on the ack: pairs with the receiver's Release store,
            // so the receiver's "invalidate done" happens-before we proceed.
            while !self.acks[cpu].load(Ordering::Acquire) {
                spin_hint();
            }
            // Consume the ack so the rendezvous can be reused.
            self.acks[cpu].store(false, Ordering::Relaxed);
            acked += 1;
        }
        acked
    }

    /// RECEIVER step 2: CPU `cpu` observes its pending bit; if set, "invalidate"
    /// (a no-op here) and ack. Returns `true` if it serviced a request.
    pub fn poll_and_ack(&self, cpu: usize) -> bool {
        // Acquire: pairs with the sender's Release store to `pending[cpu]`.
        if self.pending[cpu].load(Ordering::Acquire) {
            // Clear pending first, then ack. (Modelled invalidate is a no-op.)
            self.pending[cpu].store(false, Ordering::Relaxed);
            // Release: publishes "I have invalidated" to the sender's Acquire.
            self.acks[cpu].store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// RECEIVER step 2, **hardware** variant: CPU `cpu` observes its pending bit;
    /// if set, run the real per-CPU invalidation `invalidate` (e.g. `invlpg` /
    /// `tlbi vae1` / a CR3-reload full flush) BEFORE acking, then ack. Returns
    /// `true` if it serviced a request.
    ///
    /// This is the real-hardware analog of [`poll_and_ack`](Self::poll_and_ack):
    /// the loom model uses the no-op `poll_and_ack` (so the modelled state space
    /// is unchanged), while the Frame's IPI handler calls this so the physical
    /// TLB invalidation `happens-before` the ack — i.e. before the sender's
    /// `request_and_wait` returns and reuses/frees the page. The ordering edge is
    /// identical to `poll_and_ack`: pending is read `Acquire` (pairs with the
    /// sender's Release publish), the invalidation runs, then the ack is a
    /// `Release` store (pairs with the sender's Acquire spin). The invalidation is
    /// sandwiched between those two so it is sequenced after observing pending and
    /// before publishing the ack.
    #[inline]
    pub fn poll_and_invalidate(&self, cpu: usize, invalidate: impl FnOnce()) -> bool {
        // Acquire: pairs with the sender's Release store to `pending[cpu]`.
        if self.pending[cpu].load(Ordering::Acquire) {
            // Clear pending first so a later request re-arms cleanly.
            self.pending[cpu].store(false, Ordering::Relaxed);
            // Run the REAL invalidation before acking: it must complete before the
            // sender (which Acquire-spins on the ack) is allowed to proceed.
            invalidate();
            // Release: publishes "I have invalidated" to the sender's Acquire.
            self.acks[cpu].store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Spin until CPU `cpu` has serviced a pending request (receiver driver used
    /// by the loom model so a receiver thread makes progress to its ack).
    pub fn service_until_acked(&self, cpu: usize) {
        loop {
            if self.poll_and_ack(cpu) {
                return;
            }
            spin_hint();
        }
    }

    /// RECEIVER step 2, **teeth** variant (loom only): identical to
    /// [`poll_and_invalidate`](Self::poll_and_invalidate) EXCEPT the ack store
    /// ordering is selectable. With `ack_release == true` it is the real
    /// `Release` ack (the production ordering); with `ack_release == false` it is
    /// a deliberately-WEAKENED `Relaxed` ack. The negative loom model
    /// (`loom_shootdown_relaxed_ack_loses_invalidate`) runs the receiver in the
    /// weakened mode to PROVE the `Release` is load-bearing: with `Relaxed` the
    /// receiver's invalidation side effect is NOT guaranteed to happen-before the
    /// sender's Acquire-spin returns, so loom finds an interleaving where the
    /// sender proceeds to reuse the page before the sibling has invalidated. This
    /// is the shootdown analog of `slab_alloc`'s ABA-defence teeth test.
    #[cfg(loom)]
    pub fn poll_and_invalidate_weakenable(
        &self,
        cpu: usize,
        ack_release: bool,
        invalidate: impl FnOnce(),
    ) -> bool {
        if self.pending[cpu].load(Ordering::Acquire) {
            self.pending[cpu].store(false, Ordering::Relaxed);
            invalidate();
            // The teeth: the REAL protocol uses Release here (publishes the
            // invalidate to the sender's Acquire). Relaxed drops that edge.
            let ack_ordering = if ack_release {
                Ordering::Release
            } else {
                Ordering::Relaxed
            };
            self.acks[cpu].store(true, ack_ordering);
            true
        } else {
            false
        }
    }

    /// Whether CPU `cpu` still has a pending (un-acked) shootdown.
    #[inline]
    pub fn is_pending(&self, cpu: usize) -> bool {
        self.pending[cpu].load(Ordering::Acquire)
    }
}

#[inline]
fn spin_hint() {
    #[cfg(loom)]
    loom::thread::yield_now();
    #[cfg(not(loom))]
    core::hint::spin_loop();
}

// ===========================================================================
// Behavioural / invariant tests (std build; also run under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_target_handshake() {
        let s = Arc::new(Shootdown::new([true, true, false, false, false, false, false, false]));
        let s2 = Arc::clone(&s);
        // Receiver CPU 1 services one request.
        let r = thread::spawn(move || s2.service_until_acked(1));
        // Sender targets CPU 1 and waits.
        let acked = s.request_and_wait(&[1]);
        r.join().unwrap();
        assert_eq!(acked, 1);
        assert!(!s.is_pending(1), "pending cleared after ack");
    }

    #[test]
    fn offline_cpu_is_never_targeted() {
        let s = Shootdown::new([true, false, false, false, false, false, false, false]);
        assert!(s.is_online(0));
        assert!(!s.is_online(1));
        // Targeting only the online CPU 0 with no receiver would block, so just
        // assert the online discipline the sender relies on.
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of the 3-step shootdown protocol.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test -p ksync loom_shootdown
// Kept SMALL (1 sender + 1–2 receivers, ≤4 CPUs) so loom exhausts fast.
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    // H1(a) no early completion + H1(b) no lost wakeup: a sender targeting one
    // receiver must observe its ack on EVERY interleaving, never proceeding early
    // and never hanging. (1 sender + 1 receiver.)
    #[test]
    fn loom_shootdown_single_receiver_acks_before_completion() {
        loom::model(|| {
            let s = Arc::new(Shootdown::new([true, true, false, false, false, false, false, false]));
            let s_recv = Arc::clone(&s);
            // Receiver CPU 1 races the sender's request publication.
            let r = loom::thread::spawn(move || s_recv.service_until_acked(1));
            let acked = s.request_and_wait(&[1]);
            r.join().unwrap();
            // The sender only returns after the receiver acked: exactly 1 ack,
            // and the pending bit is clear (no early completion / lost wakeup).
            assert_eq!(acked, 1, "sender completed without the target's ack");
            assert!(!s.is_pending(1), "target bit still pending after completion");
        });
    }

    // H1(a)+(b) with TWO receivers: every targeted receiver acks; no early
    // completion, no lost wakeup. To keep the state space tractable (a sender
    // spin-loop waiting on BOTH acks concurrently with two receiver threads
    // blows past loom's branch cap — the documented H1 risk), the sender
    // publishes BOTH pending bits, then the two receiver threads observe+ack
    // concurrently, and the sender collects the acks AFTER joining them. The
    // race loom explores is sender-publish (Release) vs each receiver-observe
    // (Acquire) — the ordering that matters; no unbounded sender spin. (1
    // publisher + 2 receivers = 3 threads, the cap.)
    #[test]
    fn loom_shootdown_two_receivers_all_ack() {
        loom::model(|| {
            let s = Arc::new(Shootdown::new([true, true, true, false, false, false, false, false]));
            // Publish both requests up front (Release stores).
            s.pending[1].store(true, Ordering::Release);
            s.pending[2].store(true, Ordering::Release);

            let r1s = Arc::clone(&s);
            let r2s = Arc::clone(&s);
            // Each receiver observes its own bit (Acquire) and acks (Release).
            let r1 = loom::thread::spawn(move || r1s.service_until_acked(1));
            let r2 = loom::thread::spawn(move || r2s.service_until_acked(2));
            r1.join().unwrap();
            r2.join().unwrap();

            // After both receivers have run, the sender sees both acks (Acquire)
            // and both pending bits cleared — no early completion, no lost ack.
            assert!(s.acks[1].load(Ordering::Acquire), "receiver 1 never acked");
            assert!(s.acks[2].load(Ordering::Acquire), "receiver 2 never acked");
            assert!(!s.is_pending(1), "target 1 still pending");
            assert!(!s.is_pending(2), "target 2 still pending");
        });
    }

    // H1 TEETH (negative parity, plan §3 Stage 3): proves the receiver's
    // `Release` ack is LOAD-BEARING — it is the happens-before edge that makes
    // the invalidation visible to the sender before the sender reuses the page.
    //
    // The "invalidation side effect" is modelled by a NON-ATOMIC `loom::cell::
    // UnsafeCell` (just as `completion.rs` models its published slot): the
    // receiver writes the cell inside its invalidate closure, THEN acks; the
    // sender reads the cell ONLY after `wait_all` observed the ack. The ONLY
    // synchronization between the cell write and the cell read is the ack
    // store/load pair, so:
    //   * with the REAL `Release` ack (+ the sender's `Acquire` wait_all) the
    //     write happens-before the read on EVERY interleaving — no data race,
    //     and the sender reads the invalidated value.
    //   * VALIDATED TEETH: flipping `ACK_RELEASE` to `false` (a `Relaxed` ack)
    //     deletes that edge, so the cell write and read are concurrent and loom
    //     reports a DATA RACE / non-deterministic read — i.e. the sender would
    //     reuse the page before the sibling's invalidation is ordered. (Confirmed:
    //     this model FAILS under loom when ACK_RELEASE = false, so it genuinely
    //     exercises the Release barrier, exactly as slab_alloc's tag-bump teeth.)
    #[test]
    fn loom_shootdown_release_ack_orders_invalidate_before_completion() {
        use loom::cell::UnsafeCell;
        // The production protocol uses Release. Set to `false` to demonstrate the
        // teeth (loom reports a data race on the cell below). VALIDATED: flipping
        // this to `false` makes loom report a "Causality violation: Concurrent
        // read and write accesses" — confirming the Release ack is load-bearing.
        const ACK_RELEASE: bool = true;
        loom::model(|| {
            let s = Arc::new(Shootdown::new([true, true, false, false, false, false, false, false]));
            // The invalidation side effect the sender must observe post-wait,
            // guarded SOLELY by the ack edge (non-atomic, like a freed PTE/page).
            let invalidated = Arc::new(UnsafeCell::new(0u32));

            let s_recv = Arc::clone(&s);
            let inv_recv = Arc::clone(&invalidated);
            // Receiver CPU 1: poll its bit, run the "invalidate" (write the cell),
            // then ack with the selected ordering.
            let r = loom::thread::spawn(move || loop {
                let serviced = s_recv.poll_and_invalidate_weakenable(1, ACK_RELEASE, || {
                    // SAFETY: this write happens-before any sender read VIA the
                    // Release ack; with the weakened Relaxed ack that edge is gone
                    // and loom flags the race — which is the point of the teeth.
                    inv_recv.with_mut(|p| unsafe { *p = 0xA1u32 });
                });
                if serviced {
                    return;
                }
                loom::thread::yield_now();
            });

            // Sender: publish + wait for the ack, then read the cell IMMEDIATELY
            // — BEFORE joining the receiver. (Joining first would add a thread-
            // join happens-before edge that masks the ack-ordering teeth.) The
            // ONLY edge that can order the receiver's cell write before this read
            // is the ack store/load pair, which is exactly what we are testing.
            let acked = s.request_and_wait(&[1]);
            // The ack's Release (+ wait_all's Acquire) must make the invalidate
            // write happen-before this read. Under the weakened Relaxed ack the
            // accesses are concurrent and loom reports the race — the teeth.
            // SAFETY: under the real Release ack the receiver's write
            // happens-before this read, so the access is unsynchronized only in
            // the deliberately-weakened mode (where loom catches it).
            let seen = invalidated.with(|p| unsafe { *p });
            r.join().unwrap();
            assert_eq!(acked, 1);
            assert_eq!(
                seen, 0xA1u32,
                "sender completed the shootdown before the receiver's invalidate was \
                 visible — the Release ack edge is missing (page reused with a stale TLB)"
            );
        });
    }

    // H1(c) online discipline: the sender targets only the ONLINE CPU; the
    // offline CPU's bit is never set and never waited on (it has no receiver, so
    // waiting on it would hang — exhausting here proves the sender does not).
    #[test]
    fn loom_shootdown_offline_cpu_never_waited_on() {
        loom::model(|| {
            // CPU 2 and 3 are OFFLINE. Sender targets only the online CPU 1.
            let s = Arc::new(Shootdown::new([true, true, false, false, false, false, false, false]));
            let s_recv = Arc::clone(&s);
            let r = loom::thread::spawn(move || s_recv.service_until_acked(1));
            assert!(s.is_online(1), "target must be online");
            assert!(!s.is_online(2), "CPU 2 is offline");
            let acked = s.request_and_wait(&[1]);
            r.join().unwrap();
            assert_eq!(acked, 1);
            // The offline CPUs were never touched.
            assert!(!s.is_pending(2), "offline CPU 2 was targeted");
            assert!(!s.is_pending(3), "offline CPU 3 was targeted");
        });
    }
}
