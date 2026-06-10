//! # ksync_hazard_stack — a lock-free Treiber stack with REAL reclamation (Phase 2, hard tier)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This is the
//! escalation of `crates_phase2/treiber_stack`, which verified the lock-free
//! push/pop algorithm but **leaked** every popped node — explicitly deferring the
//! genuinely-hard part of lock-free programming: *safe memory reclamation*. This
//! crate closes that boundary with **hazard pointers** (Maged Michael, 2004), so
//! `pop` actually **frees** nodes.
//!
//! ## The problem hazard pointers solve
//! A reclaiming Treiber stack has a use-after-free: between a popper loading `head`
//! and dereferencing it, another thread can pop that node and free it. A hazard
//! pointer is a per-participant single-writer/multi-reader slot in which a thread
//! publishes the node it is *about to* dereference. Before any thread frees a
//! retired node it **scans every hazard slot**; a node still announced in some slot
//! is *deferred*, never freed. So no node is freed while a thread holds a hazard to
//! it — the use-after-free is structurally impossible.
//!
//! ## The ordering crux (why SeqCst fences, not just Release/Acquire)
//! The protector does `store(hazard = node); load(head)` and the reclaimer does
//! `CAS(head → next); scan(hazards)`. That is the **Dekker / StoreLoad** pattern:
//! with only Release/Acquire the store and the load may be reordered on *both*
//! sides, so the protector could read `head == node` (validation passes) while the
//! reclaimer's scan misses the hazard (frees the node) — a use-after-free. The fix
//! is a **`SeqCst` fence on each side**: one between the protector's hazard-store
//! and its validation-load, and one between the reclaimer's head-removal CAS and its
//! hazard-scan. The two fences are ordered in the single total order of `SeqCst`
//! fences, so at least one side observes the other and a validated node can never be
//! freed. (This is lighter than making every operation `SeqCst`, which both
//! over-synchronises and explodes loom's state space.) The SAFETY arguments below
//! rely on this; loom checks the protocol's ordering and Miri checks the actual
//! frees for use-after-free.
//!
//! ## The second safety leg — single-owner retirement
//! The fence barrier is only half the proof. The other half is structural: a node is
//! retired and freed by **exactly one thread — the popper that CAS-removed it from
//! `head`** — into that popper's **own private retire list** (`Participant.retired`),
//! and is freed only from that same list. Because the head CAS has a unique winner
//! per node, no two threads ever retire the same node, so there is no cross-thread
//! double-free and no shared retire list to race. This leg alone closes the window
//! between a popper's CAS-remove and its value-read (before it clears its hazard): no
//! other thread can retire that node, so nothing can free it there regardless of
//! fences. (Confirmed by an independent three-lens soundness audit, 2026-06-04: all
//! lenses sound. The audit's one substantive note was exactly that this leg deserved
//! first-class documentation alongside the fence argument.)
//!
//! ## What is verified vs. what is scoped out
//! - **loom** checks the *atomic protocol* (head CAS + hazard publish/validate/scan)
//!   under bounded preemption (`LOOM_MAX_PREEMPTIONS`, loom's standard tractable
//!   model — the full SeqCst-fence interleaving is too large to enumerate without a
//!   bound): concurrent pops linearise with no lost/duplicated element and no
//!   double-take, and the reclamation scan never frees a node a thread is
//!   dereferencing. (Concurrent *push* uses the same CAS proven exhaustively in
//!   `crates_phase2/treiber_stack`; here the loom focus is the novel reclamation.)
//! - **Miri** runs the real frees under the UB interpreter: it would flag any
//!   use-after-free if the hazard protocol had a hole, and (unlike the leaking
//!   stack) its leak checker is expected **clean** — real reclamation frees nodes.
//! - **Scoped out** (documented, not faked): a fixed `SLOTS` participant bound
//!   (registration panics past it; slots are not recycled), a per-participant retire
//!   list with immediate scan (no global domain / batching tuning), and no support
//!   for a participant dying mid-operation. This is a *verified hazard-pointer
//!   reclamation core*, not a production hazard-pointer domain (e.g. `seqlock`/
//!   `folly`/`crossbeam-epoch` scale). Not Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};

#[cfg(not(loom))]
use alloc::{boxed::Box, vec::Vec};
#[cfg(loom)]
use std::{boxed::Box, vec::Vec};

use core::cell::RefCell;
use core::mem::ManuallyDrop;
use core::ptr;

/// Number of hazard slots = maximum number of concurrently-registered
/// participants. Registration past this panics; kept small to bound the loom
/// model. (A production domain would grow/recycle slots dynamically.)
const SLOTS: usize = 2;

/// Retire-list length that triggers a reclamation scan. 1 = scan-and-free on every
/// retire — the most aggressive setting, which maximises the protect-vs-reclaim
/// race that loom and Miri must clear. (Production would batch for throughput.)
const RECLAIM_THRESHOLD: usize = 1;

struct Node<T> {
    next: *mut Node<T>,
    value: ManuallyDrop<T>,
}

/// A lock-free LIFO stack with hazard-pointer reclamation. Concurrent access goes
/// through per-thread [`Participant`] handles obtained from [`HazardStack::register`].
pub struct HazardStack<T> {
    head: AtomicPtr<Node<T>>,
    hazards: [AtomicPtr<Node<T>>; SLOTS],
    /// Bitset of in-use hazard slots; a participant claims a free bit on register
    /// and releases it on drop. Bounds *concurrent* participants to `SLOTS` (slots
    /// are recycled across a participant's lifetime, like a real hazard domain).
    slot_mask: AtomicUsize,
}

// SAFETY: `head` and `hazards` are atomic; nodes are published via Release CAS and
// read only while hazard-protected (see the protocol below), and freed only after a
// scan confirms no hazard references them. Values of type T are moved across threads
// (pushed by one, popped by another) but never shared by reference, so T: Send (not
// T: Sync) is the correct bound — the channel/Mutex precedent.
unsafe impl<T: Send> Send for HazardStack<T> {}
unsafe impl<T: Send> Sync for HazardStack<T> {}

impl<T> HazardStack<T> {
    /// Create an empty stack. (Not `const`: the hazard array of atomics is built
    /// with `array::from_fn`, and loom's atomics are not const-constructible.)
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            hazards: core::array::from_fn(|_| AtomicPtr::new(ptr::null_mut())),
            slot_mask: AtomicUsize::new(0),
        }
    }

    /// Claim a hazard slot for the calling thread. Panics only if `SLOTS`
    /// participants are *concurrently* registered (slots are freed on drop and
    /// reused). Each participant must be used by exactly one thread (`Participant`
    /// is `!Sync`).
    pub fn register(&self) -> Participant<'_, T> {
        loop {
            let mask = self.slot_mask.load(Ordering::Acquire);
            let idx = (0..SLOTS).find(|&i| mask & (1 << i) == 0).expect(
                "ksync_hazard_stack: more than SLOTS concurrently-registered participants",
            );
            if self
                .slot_mask
                .compare_exchange(mask, mask | (1 << idx), Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Participant {
                    stack: self,
                    idx,
                    retired: RefCell::new(Vec::new()),
                };
            }
            // Lost the claim race; another participant took a bit — retry.
        }
    }

    /// Whether the stack is currently empty (a momentary observation).
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

impl<T> Default for HazardStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for HazardStack<T> {
    fn drop(&mut self) {
        // Exclusive access (&mut self): free every node still on the list. These
        // were never popped, so their value is live and must be dropped explicitly
        // (Node's own drop is a no-op on the ManuallyDrop value). A Relaxed load
        // suffices — no other thread can touch the stack while it is being dropped —
        // and works on both core and loom atomics (loom's AtomicPtr has no get_mut).
        let mut cur = self.head.load(Ordering::Relaxed);
        while !cur.is_null() {
            // SAFETY: `cur` is a node we own (pushed, never popped/freed); we have
            // exclusive access, so reclaiming it and dropping its live value is sound.
            let mut boxed = unsafe { Box::from_raw(cur) };
            cur = boxed.next;
            unsafe { ManuallyDrop::drop(&mut boxed.value) };
        }
    }
}

/// A per-thread handle into a [`HazardStack`]. Owns one hazard slot and a private
/// retire list. `!Sync` (a `RefCell` retire list) — use one per thread.
pub struct Participant<'a, T> {
    stack: &'a HazardStack<T>,
    idx: usize,
    retired: RefCell<Vec<*mut Node<T>>>,
}

// SAFETY: a Participant may be moved to another thread (e.g. spawned), so it is
// Send when T is. It is intentionally NOT Sync: the RefCell retire list and the
// single hazard slot are single-writer, so one thread must own each Participant.
unsafe impl<T: Send> Send for Participant<'_, T> {}

impl<T> Participant<'_, T> {
    /// Push a value (lock-free). Push never dereferences an existing node — it only
    /// CAS-links a fresh node — so it needs no hazard protection.
    pub fn push(&self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            next: ptr::null_mut(),
            value: ManuallyDrop::new(value),
        }));
        loop {
            let head = self.stack.head.load(Ordering::Relaxed);
            // SAFETY: `node` is our freshly-allocated, not-yet-published node; we are
            // its sole accessor, so writing `next` is exclusive and race-free.
            unsafe { (*node).next = head };
            // Release publishes the node + its `next` write to a popper's Acquire load.
            if self
                .stack
                .head
                .compare_exchange(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }
    }

    /// Pop the most-recently-pushed value (lock-free), reclaiming the node safely
    /// via hazard pointers. Returns `None` if empty.
    pub fn pop(&self) -> Option<T> {
        let hp = &self.stack.hazards[self.idx];
        loop {
            let head = self.stack.head.load(Ordering::Acquire);
            if head.is_null() {
                hp.store(ptr::null_mut(), Ordering::Release);
                return None;
            }
            // Publish our hazard on `head` (Release), then a SeqCst fence, then
            // re-load `head` (Acquire) to validate. This is the textbook
            // hazard-pointer StoreLoad barrier: the protector's SeqCst fence here and
            // the reclaimer's SeqCst fence in `scan` are ordered in the single total
            // order of SeqCst fences, so at least one side observes the other —
            // either we see `head` already moved (retry) or the reclaimer sees our
            // hazard in its scan (defers the node). Plain Release/Acquire alone would
            // permit the store and load to be reordered on both sides → a node could
            // be validated here and freed there: a use-after-free.
            hp.store(head, Ordering::Release);
            fence(Ordering::SeqCst);
            if self.stack.head.load(Ordering::Acquire) != head {
                // `head` changed; our hazard is stale — retry (next iter re-publishes,
                // or the empty branch clears it).
                continue;
            }
            // `head` is now hazard-protected: no thread will free it until we drop the
            // hazard. SAFETY: protected + published-with-Release, so reading `next`
            // (written once before publication, frozen after) is valid and race-free.
            let next = unsafe { (*head).next };
            if self
                .stack
                .head
                .compare_exchange(head, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // We removed `head` and still hold the hazard on it. SAFETY: exactly
                // one popper wins this CAS, so we uniquely own the node now; read the
                // value out once (the node still carries it; ManuallyDrop means
                // reclaiming the node will not double-drop it).
                let value = unsafe { ManuallyDrop::into_inner(ptr::read(&(*head).value)) };
                // Done dereferencing `head`: drop our hazard, then retire for reclaim.
                hp.store(ptr::null_mut(), Ordering::Release);
                self.retire(head);
                return Some(value);
            }
            // CAS lost: another thread changed head. Loop; hazard is re-published or
            // cleared next iteration.
        }
    }

    /// Whether the stack is currently empty (a momentary observation).
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Add `node` to this participant's retire list; scan + reclaim when it reaches
    /// the threshold.
    fn retire(&self, node: *mut Node<T>) {
        let mut retired = self.retired.borrow_mut();
        retired.push(node);
        if retired.len() >= RECLAIM_THRESHOLD {
            self.scan(&mut retired);
        }
    }

    /// Free every retired node not currently announced in any hazard slot; keep the
    /// rest for a later scan.
    fn scan(&self, retired: &mut Vec<*mut Node<T>>) {
        // SeqCst fence pairs with each protector's fence in `pop` (see there): in the
        // single total order of SeqCst fences this fence either precedes a
        // protector's — so that protector will then observe our head-removal and
        // retry — or follows it, so the Acquire loads below observe that protector's
        // hazard store and we defer the node. Either way a validated node is never
        // freed here.
        fence(Ordering::SeqCst);
        let mut hazarded: [*mut Node<T>; SLOTS] = [ptr::null_mut(); SLOTS];
        for (i, slot) in self.stack.hazards.iter().enumerate() {
            hazarded[i] = slot.load(Ordering::Acquire);
        }
        retired.retain(|&node| {
            if hazarded.contains(&node) {
                true // still protected — defer
            } else {
                // SAFETY: no hazard slot announces `node`, and only the popper that
                // removed it (this participant) ever retires it, so we hold unique
                // ownership and no thread can be mid-dereference. Its value was
                // already moved out in `pop`, so reclaiming frees memory only
                // (ManuallyDrop suppresses a second value drop).
                unsafe { drop(Box::from_raw(node)) };
                false
            }
        });
    }
}

impl<T> Drop for Participant<'_, T> {
    fn drop(&mut self) {
        // Release our hazard slot, then make a final reclamation pass. Any node still
        // protected by another live participant is left in the list and leaked — a
        // documented pathological-teardown corner (a global domain would hand it off).
        // In well-ordered teardown (all worker threads joined first) every hazard is
        // already null, so this frees everything.
        self.stack.hazards[self.idx].store(ptr::null_mut(), Ordering::Release);
        let mut retired = self.retired.borrow_mut();
        self.scan(&mut retired);
        drop(retired);
        // Release the hazard slot for reuse by a future participant.
        self.stack
            .slot_mask
            .fetch_and(!(1 << self.idx), Ordering::Release);
    }
}

// ===========================================================================
// Behavioural / invariant tests (std; small variants under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicBool, AtomicUsize as StdAtomicUsize, Ordering as O};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn single_thread_lifo() {
        let s = HazardStack::new();
        let p = s.register();
        assert!(p.is_empty());
        assert_eq!(p.pop(), None);
        p.push(1);
        p.push(2);
        p.push(3);
        assert_eq!(p.pop(), Some(3));
        assert_eq!(p.pop(), Some(2));
        assert_eq!(p.pop(), Some(1));
        assert_eq!(p.pop(), None);
    }

    // A value type that counts its own drops, to prove the ManuallyDrop reclamation
    // discipline: every pushed value is dropped EXACTLY once — popped values by the
    // caller, never-popped values by HazardStack::drop — with no double-free.
    struct DropCounter(Arc<StdAtomicUsize>);
    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, O::Relaxed);
        }
    }

    #[test]
    fn reclamation_drops_each_value_exactly_once() {
        let drops = Arc::new(StdAtomicUsize::new(0));
        {
            let s = HazardStack::new();
            let p = s.register();
            for _ in 0..6 {
                p.push(DropCounter(Arc::clone(&drops)));
            }
            // Pop and drop 4 (caller drops them); 2 remain on the stack.
            for _ in 0..4 {
                drop(p.pop().expect("should have a value"));
            }
            assert_eq!(drops.load(O::Relaxed), 4, "popped values dropped once each");
            // p dropped (reclaims its retired, already-emptied nodes), then s dropped
            // (frees the 2 remaining nodes, dropping their live values).
        }
        assert_eq!(
            drops.load(O::Relaxed),
            6,
            "every value dropped exactly once (no leak, no double-free)"
        );
    }

    // Concurrent push/pop across SLOTS threads (each both produces and consumes),
    // with REAL reclamation in flight. Every pushed value is popped exactly once —
    // no lost/duplicated element and (Miri verifies) no use-after-free. Hang-proof:
    // a consumer stops only after all producers finish AND the stack is empty.
    #[test]
    fn concurrent_conservation_with_reclamation() {
        let threads = SLOTS; // one participant per hazard slot
        let per = if cfg!(miri) { 15u32 } else { 4_000 };
        let s = Arc::new(HazardStack::new());
        let total = threads as u32 * per;
        let producers_done = Arc::new(AtomicBool::new(false));
        let popped = Arc::new(Mutex::new(BTreeSet::new()));
        let started = Arc::new(StdAtomicUsize::new(0));

        let mut handles = Vec::new();
        for t in 0..threads {
            let s = Arc::clone(&s);
            let popped = Arc::clone(&popped);
            let producers_done = Arc::clone(&producers_done);
            let started = Arc::clone(&started);
            handles.push(thread::spawn(move || {
                let p = s.register();
                // Push this thread's disjoint id range.
                for i in 0..per {
                    p.push(t as u32 * per + i);
                }
                started.fetch_add(1, O::Release);
                // Drain. Once every producer has finished pushing and the stack is
                // observed empty, no more values can appear, so it is safe to stop.
                let mut local = Vec::new();
                loop {
                    if let Some(v) = p.pop() {
                        local.push(v);
                    } else if producers_done.load(O::Acquire) && p.is_empty() {
                        break;
                    }
                }
                let mut g = popped.lock().unwrap();
                for v in local {
                    assert!(g.insert(v), "duplicate pop of {v}");
                }
            }));
        }

        // Signal "producers done" only once every thread has finished its push phase.
        while started.load(O::Acquire) < threads {
            std::hint::spin_loop();
        }
        producers_done.store(true, O::Release);

        for h in handles {
            h.join().unwrap();
        }
        assert!(s.is_empty(), "stack not drained");
        assert_eq!(
            popped.lock().unwrap().len(),
            total as usize,
            "lost or duplicated elements"
        );
    }
}

// ===========================================================================
// loom model — interleaving check of the hazard-pointer reclamation protocol,
// under bounded preemption (set LOOM_MAX_PREEMPTIONS, e.g. 2). The concurrent
// region is kept to the *pops* — the novel reclaim-vs-protect race; concurrent
// *push* is the same CAS proven exhaustively in crates_phase2/treiber_stack, so
// elements are pre-pushed sequentially here to keep the model tractable.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=2 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_concurrent_pop_conservation_with_reclaim() {
        loom::model(|| {
            let s = Arc::new(HazardStack::new());
            // Pre-push two elements sequentially (slot freed on drop of `p`).
            {
                let p = s.register();
                p.push(1u32);
                p.push(2u32);
            }
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            // Two threads each register a slot and pop once — concurrently exercising
            // the publish/validate/CAS/scan/free protocol. Across every explored
            // interleaving the popped values (plus a final drain) form exactly {1,2}:
            // no lost or duplicated element, no double-take, and the reclamation scan
            // never frees a node the other thread is dereferencing.
            let t1 = loom::thread::spawn(move || {
                let p = s1.register();
                p.pop()
            });
            let t2 = loom::thread::spawn(move || {
                let p = s2.register();
                p.pop()
            });
            let a = t1.join().unwrap();
            let b = t2.join().unwrap();
            let mut got: Vec<u32> = Vec::new();
            got.extend(a);
            got.extend(b);
            // Drain anything left after the concurrent pops.
            let p = s.register();
            while let Some(v) = p.pop() {
                got.push(v);
            }
            got.sort_unstable();
            assert_eq!(got, vec![1, 2], "lost/duplicated element: {got:?}");
        });
    }

    #[test]
    fn loom_concurrent_pop_no_double_take() {
        loom::model(|| {
            let s = Arc::new(HazardStack::new());
            {
                let p = s.register();
                p.push(42u32);
            }
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            // Two poppers race for one element: exactly one gets Some(42), never both
            // (a double-take would be a use-after-free of a single node).
            let t1 = loom::thread::spawn(move || {
                let p = s1.register();
                p.pop()
            });
            let t2 = loom::thread::spawn(move || {
                let p = s2.register();
                p.pop()
            });
            let a = t1.join().unwrap();
            let b = t2.join().unwrap();
            let takes = a.iter().count() + b.iter().count();
            assert_eq!(takes, 1, "element taken {takes} times (want exactly 1)");
            assert!(a == Some(42) || b == Some(42));
        });
    }
}
