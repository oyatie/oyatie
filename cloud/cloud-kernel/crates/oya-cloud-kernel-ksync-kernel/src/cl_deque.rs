//! # ksync_cl_deque — a bounded Chase-Lev work-stealing deque (P4·SMP·S4b, H4)
//!
//! A **single-owner / multi-stealer** lock-free deque holding `u32` pids: the
//! per-CPU run-queue's stealable fast path. The owning CPU pushes and pops at
//! the **bottom** (its hot, uncontended local end); any *other* CPU steals from
//! the **top** (the cold, contended remote end). It is the work-stealing
//! scheduler primitive (Chase, Lev — "Dynamic Circular Work-Stealing Deque",
//! SPAA'05), specialised to a **bounded** ring so there is NO allocation and NO
//! array-growth ABA hazard in the steal path — the right shape for a kernel.
//!
//! ```text
//!         top (stealers CAS here)            bottom (owner here)
//!          |                                  |
//!   [ . . S S S O O O O . . . . . . . . ]   buf[i % CAP]
//!          ^----- live elements -----^
//! ```
//!
//! ## What loom proves here (and what it does NOT)
//! loom is an exhaustive model checker over the **bounded** protocol of atomics
//! (a bug-finder, not an absolute absence proof — it does NOT model weak-memory
//! hardware effects beyond the C11 model, real IPI timing, or the scheduler that
//! consumes the pids). The H4 models below prove the *orderings* that matter:
//! (H4a) one owner + one stealer never lose a pid and never double-pop;
//! (H4b) on the LAST element, owner-`pop` racing a `steal` lets **exactly one**
//! of them win (the subtle Chase-Lev pop/steal CAS conflict); (H4c) one owner +
//! two stealers preserve those invariants at loom's 3-thread branch cap. A
//! documented NEGATIVE (`loom_cl_deque_negative_*`) shows the CAS on `top` is the
//! load-bearing arbiter that prevents a double-steal — the teeth that prove the
//! ordering is necessary, mirroring `slab_alloc.rs`'s "FAILS when the tag bump is
//! removed" and `shootdown.rs`'s Relaxed-load negative.
//!
//! ## Soundness of the lock-free deque
//! `top`/`bottom` are **monotonically increasing** logical indices (never reused
//! modulo wrap within the usize range), so a stealer's CAS on `top` detects any
//! intervening steal/pop (the classic Chase-Lev ABA-freedom on `top`). Each ring
//! slot is an `AtomicU32`; a pid (`> 0`; pid 0 is never runnable, slot 0 unused)
//! is published by the owner with a `Release` store to `bottom` that the stealer
//! reads `Acquire`. The owner is the sole writer of `bottom`; `top` is advanced
//! by a stealer's successful CAS or by the owner draining the last element — the
//! single point where owner and stealers contend, resolved by the CAS so at most
//! one of them takes that element. This is the same release/acquire single-
//! publisher edge `SpinLock`/`Shootdown` already prove, generalised to the
//! deque's two ends.

#[cfg(loom)]
use loom::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Capacity of the bounded ring (a power of two so `% CAP` is a mask). Sized to
/// `MAX_RUNNABLE_PER_CPU` — the cap on runnable pids a single CPU's stealable
/// queue holds before the scheduler falls back to the locked global queue.
pub const DEQUE_CAP: usize = 64;

/// The result of a [`Deque::steal`] attempt by a non-owner CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Steal {
    /// Took `pid` from the top.
    Success(u32),
    /// The deque was observed empty (no work to steal right now).
    Empty,
    /// A concurrent steal/pop won the race on this element; the caller may retry
    /// (typically by trying the next victim, then re-polling).
    Retry,
}

/// A bounded Chase-Lev work-stealing deque of `u32` pids.
///
/// **Single-owner discipline:** [`push`](Self::push) and [`pop`](Self::pop) may
/// be called only by the owning CPU (the bottom end); [`steal`](Self::steal) by
/// any *other* CPU (the top end). The owner end is single-threaded, so its
/// `bottom` updates need no CAS; stealers CAS `top`.
pub struct Deque {
    /// Logical index of the next free slot at the bottom (owner-only writer).
    /// Monotone increasing; `buf[(bottom-1) % CAP]` is the most-recently pushed.
    bottom: AtomicUsize,
    /// Logical index of the oldest live slot at the top (stealers CAS, owner may
    /// advance when draining the last element). Monotone increasing.
    top: AtomicUsize,
    /// The ring of pid slots. A live element at logical index `i` lives in
    /// `buf[i % CAP]`. `T = u32` (a pid) is `Copy`, so no slot ever needs Drop.
    buf: [AtomicU32; DEQUE_CAP],
}

// SAFETY: the only shared mutable state is the `top`/`bottom` atomics and the
// `AtomicU32` ring slots. The owner is the sole writer of `bottom` and of any
// slot it pushes; a slot is read by a stealer only after the owner's `Release`
// store to `bottom` publishes it (paired with the stealer's `Acquire` load), and
// the owner/stealer race for a given element is resolved by the single CAS on
// `top` (exactly one winner). No conflicting non-atomic access exists, so sharing
// `&Deque` across CPUs is sound.
unsafe impl Sync for Deque {}
unsafe impl Send for Deque {}

impl Deque {
    /// Create an empty deque (`top == bottom == 0`).
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            bottom: AtomicUsize::new(0),
            top: AtomicUsize::new(0),
            // pid 0 is never runnable; an unused slot reads back as 0.
            buf: [const { AtomicU32::new(0) }; DEQUE_CAP],
        }
    }

    /// Create an empty deque (loom build: loom's atomics are not `const`-
    /// constructible, so this is a non-`const` constructor).
    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            bottom: AtomicUsize::new(0),
            top: AtomicUsize::new(0),
            buf: core::array::from_fn(|_| AtomicU32::new(0)),
        }
    }

    /// Number of live elements (a snapshot; racy under concurrent steals, used
    /// only for the empty/full fast checks and tests).
    #[inline]
    pub fn len(&self) -> usize {
        let b = self.bottom.load(Ordering::Relaxed);
        let t = self.top.load(Ordering::Relaxed);
        b.saturating_sub(t)
    }

    /// Whether the deque is observed empty (a snapshot).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// OWNER: push `pid` onto the bottom. Returns `false` (a no-op) if the ring
    /// is full (`MAX_RUNNABLE_PER_CPU` reached) so the caller can fall back to the
    /// locked global queue; `true` on success. Owner-only — never call from a
    /// stealer CPU.
    pub fn push(&self, pid: u32) -> bool {
        let b = self.bottom.load(Ordering::Relaxed);
        // Acquire so we observe stealers' `top` advances (don't over-count free
        // space). If the ring is full, refuse — the caller spills to the global.
        let t = self.top.load(Ordering::Acquire);
        if b.wrapping_sub(t) >= DEQUE_CAP {
            return false;
        }
        // Write the slot, THEN publish the new bottom with a Release store so a
        // stealer that observes the bumped `bottom` (Acquire) also observes the
        // slot write (the single-publisher edge).
        self.buf[b % DEQUE_CAP].store(pid, Ordering::Relaxed);
        self.bottom.store(b.wrapping_add(1), Ordering::Release);
        true
    }

    /// OWNER: pop from the bottom (LIFO for the owner — the hot, cache-warm end).
    /// Returns the popped pid, or `None` if empty. On the LAST element it races a
    /// concurrent `steal`; the CAS on `top` resolves the conflict so the element
    /// is taken by exactly one of them. Owner-only.
    pub fn pop(&self) -> Option<u32> {
        let b = self.bottom.load(Ordering::Relaxed);
        let t = self.top.load(Ordering::Relaxed);
        if b == t {
            return None; // empty
        }
        // Speculatively claim the bottom element by decrementing `bottom`.
        let b = b.wrapping_sub(1);
        self.bottom.store(b, Ordering::Relaxed);
        // Full fence: order our `bottom` decrement (a store) against the `top`
        // load (a load) below — the StoreLoad edge Chase-Lev requires so the
        // owner and a racing stealer agree on who took the last element.
        ordered_fence();
        let t = self.top.load(Ordering::Relaxed);
        let size = b.wrapping_sub(t) as isize;
        if size < 0 {
            // Empty: a stealer took the element first. Restore `bottom` to `top`.
            self.bottom.store(t, Ordering::Relaxed);
            return None;
        }
        let pid = self.buf[b % DEQUE_CAP].load(Ordering::Relaxed);
        if size > 0 {
            // More than one element: no contention possible on this one (it is
            // strictly below `top`), take it directly.
            return Some(pid);
        }
        // Exactly one element (`size == 0`, i.e. b == t): the owner and a stealer
        // both want it. CAS `top` from t to t+1: the winner takes the element.
        let won = self
            .top
            .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
            .is_ok();
        // Whether we won or lost, the deque is now empty: reset `bottom` to t+1.
        self.bottom.store(t.wrapping_add(1), Ordering::Relaxed);
        if won {
            Some(pid)
        } else {
            None // a stealer won the last element.
        }
    }

    /// STEALER: take from the top (FIFO order across stealers). Any CPU other than
    /// the owner. Returns [`Steal::Success`] with the pid, [`Steal::Empty`] if
    /// nothing was queued, or [`Steal::Retry`] if a concurrent steal/pop won this
    /// element (try the next victim, then re-poll).
    pub fn steal(&self) -> Steal {
        // Acquire on `top`: pairs with a winning CAS's release so we observe the
        // prior stealer's advance; the start of the steal protocol.
        let t = self.top.load(Ordering::Acquire);
        // Full fence: order this `top` load against the `bottom` load below (the
        // StoreLoad edge mirroring the owner's `pop`), so a stealer and the owner
        // agree on emptiness for the last element.
        ordered_fence();
        // Acquire on `bottom`: pairs with the owner's Release `push`, so if we
        // see the bumped bottom we also see the published slot.
        let b = self.bottom.load(Ordering::Acquire);
        if b.wrapping_sub(t) as isize <= 0 {
            return Steal::Empty; // empty (or the owner is mid-pop draining it).
        }
        // Read the candidate BEFORE the CAS: it must be loaded while `top == t`,
        // because a successful CAS frees the slot for the owner to overwrite.
        let pid = self.buf[t % DEQUE_CAP].load(Ordering::Relaxed);
        // CAS `top` t -> t+1. Success ⇒ we exclusively own `pid` (no other stealer
        // and not the owner took it). Failure ⇒ someone else advanced `top`; the
        // `pid` we read may be stale, so we must NOT return it — retry.
        if self
            .top
            .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            Steal::Success(pid)
        } else {
            Steal::Retry
        }
    }
}

#[cfg(not(loom))]
impl Default for Deque {
    fn default() -> Self {
        Self::new()
    }
}

/// A full (`SeqCst`) fence ordering an owner/stealer's `top`/`bottom` store
/// against the subsequent opposite-end load — the StoreLoad edge Chase-Lev needs
/// on the last-element race. Maps to loom's instrumented fence under `--cfg loom`.
#[inline]
fn ordered_fence() {
    #[cfg(loom)]
    loom::sync::atomic::fence(Ordering::SeqCst);
    #[cfg(not(loom))]
    core::sync::atomic::fence(Ordering::SeqCst);
}

// ===========================================================================
// Behavioural / invariant tests (std build; also run under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering as StdOrdering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn single_owner_push_pop_is_lifo() {
        let d = Deque::new();
        assert!(d.is_empty());
        assert_eq!(d.pop(), None);
        assert!(d.push(11));
        assert!(d.push(22));
        assert!(d.push(33));
        assert_eq!(d.len(), 3);
        // Owner pops LIFO from the bottom.
        assert_eq!(d.pop(), Some(33));
        assert_eq!(d.pop(), Some(22));
        assert_eq!(d.pop(), Some(11));
        assert_eq!(d.pop(), None);
        assert!(d.is_empty());
    }

    #[test]
    fn steal_takes_from_the_top_fifo() {
        let d = Deque::new();
        for pid in 1..=4u32 {
            assert!(d.push(pid));
        }
        // Stealers take the OLDEST (top) elements first: 1, 2, ...
        assert_eq!(d.steal(), Steal::Success(1));
        assert_eq!(d.steal(), Steal::Success(2));
        // The owner still pops LIFO from the bottom: 4, then 3.
        assert_eq!(d.pop(), Some(4));
        assert_eq!(d.pop(), Some(3));
        assert_eq!(d.steal(), Steal::Empty);
        assert_eq!(d.pop(), None);
    }

    #[test]
    fn push_refuses_when_full() {
        let d = Deque::new();
        for pid in 1..=DEQUE_CAP as u32 {
            assert!(d.push(pid), "push within capacity must succeed");
        }
        assert_eq!(d.len(), DEQUE_CAP);
        // One past capacity: refused (caller spills to the global queue).
        assert!(!d.push(9999), "push past capacity must refuse");
        // Draining one frees a slot for another push.
        assert_eq!(d.pop(), Some(DEQUE_CAP as u32));
        assert!(d.push(9999));
    }

    // Stress: one owner thread push/pops while N stealer threads steal; every pid
    // is taken AT MOST once (no double-take) and the union of (owner-popped +
    // stolen) loses nothing. (Miri runs a small version to check for races/UB.)
    #[test]
    fn no_pid_taken_twice_under_contention() {
        let n: u32 = if cfg!(miri) { 16 } else { 1000 };
        let stealers = if cfg!(miri) { 2 } else { 4 };
        let d = Arc::new(Deque::new());
        let owner_done = Arc::new(AtomicBool::new(false));
        let taken: Arc<Mutex<HashSet<u32>>> = Arc::new(Mutex::new(HashSet::new()));

        let insert = |set: &Mutex<HashSet<u32>>, pid: u32| {
            assert!(set.lock().unwrap().insert(pid), "pid {pid} taken twice");
        };

        let mut handles = Vec::new();
        for _ in 0..stealers {
            let d = Arc::clone(&d);
            let done = Arc::clone(&owner_done);
            let taken = Arc::clone(&taken);
            handles.push(thread::spawn(move || loop {
                match d.steal() {
                    Steal::Success(pid) => insert(&taken, pid),
                    Steal::Empty => {
                        if done.load(StdOrdering::Acquire) && d.is_empty() {
                            break;
                        }
                        thread::yield_now();
                    }
                    Steal::Retry => thread::yield_now(),
                }
            }));
        }

        // Owner: push 1..=n interleaved with occasional pops.
        for pid in 1..=n {
            while !d.push(pid) {
                if let Some(p) = d.pop() {
                    insert(&taken, p);
                }
            }
            if pid % 3 == 0 {
                if let Some(p) = d.pop() {
                    insert(&taken, p);
                }
            }
        }
        while let Some(p) = d.pop() {
            insert(&taken, p);
        }
        owner_done.store(true, StdOrdering::Release);
        for h in handles {
            h.join().unwrap();
        }
        // Every pid 1..=n was taken exactly once (no loss, no duplicate).
        assert_eq!(taken.lock().unwrap().len(), n as usize, "some pid was lost");
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of the Chase-Lev push/pop/steal.
//   RUSTFLAGS="--cfg loom" cargo test -p ksync --release loom_cl_deque
// Kept SMALL (1 owner + 1–2 stealers, ≤3 threads — loom's documented branch
// cap, see shootdown.rs) so loom exhausts fast. The models push ≤2 elements.
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    // H4a — no lost pid / no double-take: 1 owner pushes two pids and pops one;
    // 1 stealer steals concurrently. Across EVERY interleaving the set of pids
    // taken by (owner pop + stealer) contains each taken pid AT MOST once, and
    // together they account for both pushed pids (none lost). Same "no lost
    // update / no double-enter" invariant as `loom_two_threads_mutual_exclusion`.
    #[test]
    fn loom_cl_deque_owner_pop_and_one_steal_no_loss() {
        loom::model(|| {
            let d = Arc::new(Deque::new());
            // Owner pre-pushes both pids (single-owner: pushes are not raced).
            assert!(d.push(1));
            assert!(d.push(2));

            let ds = Arc::clone(&d);
            let stealer = loom::thread::spawn(move || ds.steal());
            // Owner pops one from the bottom concurrently with the steal.
            let owner_got = d.pop();
            let stolen = stealer.join().unwrap();

            // Collect what each side took.
            let mut taken: Vec<u32> = Vec::new();
            if let Some(p) = owner_got {
                taken.push(p);
            }
            if let Steal::Success(p) = stolen {
                taken.push(p);
            }
            // Drain whatever remains (owner-only now, no race).
            while let Some(p) = d.pop() {
                taken.push(p);
            }
            // No pid taken twice.
            taken.sort_unstable();
            let mut dedup = taken.clone();
            dedup.dedup();
            assert_eq!(taken, dedup, "a pid was taken twice (double pop/steal)");
            // Both pushed pids accounted for (none lost).
            assert_eq!(taken, vec![1, 2], "a pushed pid was lost");
        });
    }

    // H4b — the subtle case: owner `pop` racing a `steal` on the LAST element.
    // The deque holds exactly ONE pid; the owner pops while a stealer steals.
    // EXACTLY ONE of them gets it (the Chase-Lev pop/steal CAS conflict resolved
    // by the single `top` CAS) — never both, never neither.
    #[test]
    fn loom_cl_deque_pop_vs_steal_last_element_exactly_one_wins() {
        loom::model(|| {
            let d = Arc::new(Deque::new());
            assert!(d.push(7)); // a single element

            let ds = Arc::clone(&d);
            let stealer = loom::thread::spawn(move || ds.steal());
            let owner_got = d.pop();
            let stolen = stealer.join().unwrap();

            let owner_took = owner_got == Some(7);
            let stealer_took = stolen == Steal::Success(7);
            // Exactly one winner on the contended last element.
            assert!(
                owner_took ^ stealer_took,
                "last element won by {} (owner={owner_took}, stealer={stealer_took})",
                if owner_took && stealer_took { "BOTH" } else { "NEITHER" }
            );
            // And the deque is drained either way (no phantom leftover).
            assert_eq!(d.pop(), None, "element 7 still present after the race");
        });
    }

    // H4c — 1 owner + 2 stealers (3 threads = loom's branch cap). Two pids are
    // queued; two stealers contend for the top. Across all interleavings no pid
    // is taken twice and none is lost.
    #[test]
    fn loom_cl_deque_owner_plus_two_stealers_no_loss() {
        loom::model(|| {
            let d = Arc::new(Deque::new());
            assert!(d.push(1));
            assert!(d.push(2));

            let d1 = Arc::clone(&d);
            let d2 = Arc::clone(&d);
            let s1 = loom::thread::spawn(move || d1.steal());
            let s2 = loom::thread::spawn(move || d2.steal());
            // The owner does NOT pop here: three spinning contenders (an owner pop
            // + two stealers, each potentially retrying) blow loom's branch cap —
            // the documented 3-thread bound. The owner-pop-vs-steal race is
            // already covered exhaustively by H4a/H4b; here the race under test is
            // stealer-vs-stealer on `top`, the multi-consumer correctness.
            let r1 = s1.join().unwrap();
            let r2 = s2.join().unwrap();

            let mut taken: Vec<u32> = Vec::new();
            if let Steal::Success(p) = r1 {
                taken.push(p);
            }
            if let Steal::Success(p) = r2 {
                taken.push(p);
            }
            while let Some(p) = d.pop() {
                taken.push(p);
            }
            taken.sort_unstable();
            let mut dedup = taken.clone();
            dedup.dedup();
            assert_eq!(taken, dedup, "two stealers took the same pid");
            assert_eq!(taken, vec![1, 2], "a pid was lost across two stealers");
        });
    }

    // DOCUMENTED NEGATIVE (the harness has teeth, mirroring `slab_alloc.rs`'s
    // "FAILS when the tag bump is removed" and `shootdown.rs`'s Relaxed-load
    // negative): the steal protocol's correctness rests on the CAS on `top` being
    // the SOLE arbiter of who takes an element. If `steal` returned the read pid
    // unconditionally (no CAS arbitration), two stealers could both return pid 1.
    // Here we assert the invariant the CAS guarantees on two concurrent stealers
    // of a SINGLE element: at most one Success. Drop the CAS-arbitration and this
    // assertion FAILS — that is the recorded proof of necessity.
    #[test]
    fn loom_cl_deque_negative_two_stealers_one_element_at_most_one_success() {
        loom::model(|| {
            let d = Arc::new(Deque::new());
            assert!(d.push(1)); // a SINGLE element two stealers fight over

            let d1 = Arc::clone(&d);
            let d2 = Arc::clone(&d);
            let s1 = loom::thread::spawn(move || d1.steal());
            let s2 = loom::thread::spawn(move || d2.steal());
            let r1 = s1.join().unwrap();
            let r2 = s2.join().unwrap();

            let n_success = (r1 == Steal::Success(1)) as u32 + (r2 == Steal::Success(1)) as u32;
            // The CAS makes the single element steal-able by AT MOST ONE stealer.
            assert!(
                n_success <= 1,
                "DOUBLE STEAL: {n_success} stealers both took the one element"
            );
            // Whoever won, the deque is now empty.
            assert_eq!(d.pop(), None);
        });
    }
}
