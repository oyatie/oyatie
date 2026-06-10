//! # ksync_ebr_stack — a lock-free Treiber stack with EPOCH-BASED reclamation (Phase 2, hard tier)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This is the
//! companion to `crates_phase2/hazard_stack`: both close the safe-memory-reclamation
//! boundary that `treiber_stack` deferred (it leaked), but via the **other** major
//! lock-free reclamation scheme — **epoch-based reclamation (EBR)** (Fraser; the
//! design behind `crossbeam-epoch`) instead of hazard pointers. `pop` actually
//! **frees** nodes.
//!
//! ## How EBR works (and why it is sound)
//! A single **global epoch** counter advances 0,1,2,…. A thread that is about to
//! touch shared nodes **pins**: it announces the current global epoch into its own
//! per-thread slot (and marks itself active). Retired nodes are stamped with the
//! epoch they were unlinked in and deferred, never freed immediately. The epoch is
//! advanced (by `pop`, best-effort) only when **every pinned thread has announced
//! the current epoch** — so to advance from `e` to `e+1` no thread may still be
//! pinned below `e`. Consequently, once the global epoch reaches `e+2` **no thread
//! can still be pinned at `e`**, so any node retired at epoch `e` is then safe to
//! free. That is the whole invariant: **free a node retired at epoch `e_r` only once
//! `global ≥ e_r + 2`.**
//!
//! Deref safety follows: while pinned at `e`, this thread blocks the global epoch
//! from reaching `e+2` (the advance scan would see it lagging and abort), so nothing
//! retired at `≤ e-1` — the only epochs collectible while we hold the pin — can be a
//! node we just loaded from `head` (a node we can reach from `head` after pinning was
//! not yet unlinked, so its retire epoch is `≥ e`). No use-after-free.
//!
//! ## The ordering crux (StoreLoad, same shape as hazard_stack)
//! The pinner does `store(local = e); <deref / scan>` and a collector does
//! `advance global; scan locals`. That is the **Dekker / StoreLoad** pattern: with
//! only Release/Acquire both sides may reorder, so a collector could advance past a
//! pin it failed to observe and free a referenced node. The fix is a **`SeqCst`
//! fence on each side**: one after a pin's announce-store (before it derefs), and one
//! the advance/scan is ordered behind. The two fences are ordered in the single total
//! order of `SeqCst` fences, so at least one side sees the other. (Minimal fencing —
//! not all-`SeqCst` — keeps the loom model tractable.) loom checks the protocol;
//! Miri checks the real frees for use-after-free (and the leak checker stays clean).
//!
//! ## What is scoped out (documented, not faked)
//! Fixed `SLOTS` concurrent-participant bound (registration recycles slots; panics if
//! exceeded), a global epoch that never wraps in any realistic run (plain `usize`
//! compares), per-participant deferral lists with a lock-free **orphan stack** drained
//! at teardown so nothing leaks, and no support for a participant dying mid-pin. A
//! *verified EBR core*, not a production epoch GC (`crossbeam-epoch` scale). Not Linux
//! parity.

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

/// Maximum number of concurrently-registered participants. Slots are recycled on
/// drop; registration panics if more than `SLOTS` are live at once. Kept small to
/// bound the loom model.
const SLOTS: usize = 2;

/// Sentinel stored in a participant's local-epoch slot when it is NOT pinned.
const UNPINNED: usize = usize::MAX;

struct Node<T> {
    next: *mut Node<T>,
    value: ManuallyDrop<T>,
}

/// A lock-free LIFO stack reclaimed by epoch-based reclamation. Concurrent access
/// goes through per-thread [`Participant`] handles from [`EbrStack::register`].
pub struct EbrStack<T> {
    head: AtomicPtr<Node<T>>,
    /// Global epoch (free-running; advanced best-effort by `pop`).
    global_epoch: AtomicUsize,
    /// Per-participant announced epoch (`UNPINNED` when not in a critical section).
    locals: [AtomicUsize; SLOTS],
    /// Bitset of in-use participant slots (claimed on register, freed on drop).
    slot_mask: AtomicUsize,
    /// Lock-free stack of nodes deferred past their participant's lifetime; drained
    /// and freed in `Drop` (when no participant is left and all are safe).
    orphans: AtomicPtr<Node<T>>,
}

// SAFETY: all shared mutable state is atomic; nodes are published via Release CAS,
// dereferenced only while the accessing thread is epoch-pinned (which blocks the
// global epoch from advancing far enough to free them), and freed only when the
// epoch guarantees no pinned thread can reference them. Values of type T are only
// moved across threads (pushed by one, popped by another), never shared by &T, so
// `T: Send` (not `Sync`) is the correct bound — the channel/Mutex precedent.
unsafe impl<T: Send> Send for EbrStack<T> {}
unsafe impl<T: Send> Sync for EbrStack<T> {}

impl<T> EbrStack<T> {
    /// Create an empty stack. (Not `const`: the local-epoch array of atomics is built
    /// with `array::from_fn`, and loom's atomics are not const-constructible.)
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            global_epoch: AtomicUsize::new(0),
            locals: core::array::from_fn(|_| AtomicUsize::new(UNPINNED)),
            slot_mask: AtomicUsize::new(0),
            orphans: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Claim a participant slot for the calling thread. Panics only if `SLOTS`
    /// participants are *concurrently* registered (slots are freed on drop and
    /// reused). Each participant must be used by exactly one thread (`!Sync`).
    pub fn register(&self) -> Participant<'_, T> {
        loop {
            let mask = self.slot_mask.load(Ordering::Acquire);
            let idx = (0..SLOTS)
                .find(|&i| mask & (1 << i) == 0)
                .expect("ksync_ebr_stack: more than SLOTS concurrently-registered participants");
            if self
                .slot_mask
                .compare_exchange(mask, mask | (1 << idx), Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Participant {
                    stack: self,
                    idx,
                    garbage: RefCell::new(Vec::new()),
                };
            }
        }
    }

    /// Whether the stack is currently empty (a momentary observation).
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

impl<T> Default for EbrStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for EbrStack<T> {
    fn drop(&mut self) {
        // Exclusive (&mut self): no participant is left (they borrow &self) and none
        // is pinned, so every deferred and live node is safe to free now.
        // 1) Orphaned nodes (retired; value already moved out → free memory only).
        let mut cur = self.orphans.load(Ordering::Relaxed);
        while !cur.is_null() {
            // SAFETY: orphan-stack nodes are popped nodes we uniquely own; their value
            // was moved out in `pop`, so reclaiming frees memory only (no double-drop).
            let boxed = unsafe { Box::from_raw(cur) };
            cur = boxed.next;
        }
        // 2) Nodes still on the list (never popped → live value, drop it explicitly).
        let mut cur = self.head.load(Ordering::Relaxed);
        while !cur.is_null() {
            // SAFETY: list nodes are owned, never freed; we have exclusive access.
            let mut boxed = unsafe { Box::from_raw(cur) };
            cur = boxed.next;
            unsafe { ManuallyDrop::drop(&mut boxed.value) };
        }
    }
}

/// A per-thread handle into an [`EbrStack`]. Owns one epoch slot and a private list
/// of deferred (retired) nodes. `!Sync` (a `RefCell` deferral list) — one per thread.
pub struct Participant<'a, T> {
    stack: &'a EbrStack<T>,
    idx: usize,
    /// `(retire_epoch, node)` deferrals awaiting `global ≥ retire_epoch + 2`.
    garbage: RefCell<Vec<(usize, *mut Node<T>)>>,
}

// SAFETY: a Participant may be moved to another thread, so it is Send when T is. It
// is intentionally NOT Sync: the RefCell deferral list and the single epoch slot are
// single-writer, so one thread must own each Participant.
unsafe impl<T: Send> Send for Participant<'_, T> {}

impl<T> Participant<'_, T> {
    /// Push a value (lock-free). Push never dereferences an existing node — it only
    /// CAS-links a fresh node — so it needs no epoch pin.
    pub fn push(&self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            next: ptr::null_mut(),
            value: ManuallyDrop::new(value),
        }));
        loop {
            let head = self.stack.head.load(Ordering::Relaxed);
            // SAFETY: `node` is our freshly-allocated, not-yet-published node.
            unsafe { (*node).next = head };
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

    /// Pop the most-recently-pushed value (lock-free), reclaiming nodes safely by
    /// epoch. Returns `None` if empty.
    pub fn pop(&self) -> Option<T> {
        let local = &self.stack.locals[self.idx];
        // --- PIN: announce the current global epoch, then a SeqCst fence so our pin
        // is ordered before we dereference shared nodes and before any collector's
        // scan can decide we are not pinned (the StoreLoad barrier). ---
        let epoch = self.stack.global_epoch.load(Ordering::Acquire);
        local.store(epoch, Ordering::Release);
        fence(Ordering::SeqCst);

        // Best-effort: try to advance the global epoch (frees future garbage sooner).
        self.try_advance(epoch);

        // --- Critical section: safe to dereference nodes reachable from `head`. ---
        let result = loop {
            let head = self.stack.head.load(Ordering::Acquire);
            if head.is_null() {
                break None;
            }
            // SAFETY: we are pinned at `epoch`, which blocks the global epoch from
            // reaching `epoch + 2`; nothing reachable from `head` can be freed under
            // us, so reading `next` is valid and race-free.
            let next = unsafe { (*head).next };
            if self
                .stack
                .head
                .compare_exchange(head, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // SAFETY: exactly one popper wins this CAS, so we uniquely own the
                // node; move its value out once (ManuallyDrop ⇒ freeing the node later
                // will not double-drop it).
                let value = unsafe { ManuallyDrop::into_inner(ptr::read(&(*head).value)) };
                // Defer for reclamation, stamped with the global epoch AT RETIREMENT
                // (not the pin epoch). This is required for soundness: any thread that
                // still references this node pinned at some epoch `f ≤ retire_epoch`
                // (it loaded the node before we unlinked it), and a thread pinned at
                // `f` blocks the global epoch from passing `f + 1 < retire_epoch + 2`,
                // so the node cannot be freed (collected at `global ≥ retire_epoch + 2`)
                // while still referenced. Stamping the (smaller) pin epoch could free
                // it an epoch too early → use-after-free.
                // This load observes `global >= f` for every thread `f` that could
                // still reference this node, so `retire_epoch >= f` (the property the
                // free-at-`global >= retire_epoch + 2` rule needs). Why it can't read a
                // stale value below `f`: this thread always ran `try_advance` just
                // above (after its own pin fence), whose CAS — success-write OR
                // failure-load — or whose Acquire scan of a peer's `local == f` (a
                // Release store made after that peer read `global == f`) imports the
                // `global = f` write into this thread's happens-before history before
                // this Acquire load. So the stamp is never an epoch too small.
                let retire_epoch = self.stack.global_epoch.load(Ordering::Acquire);
                self.garbage.borrow_mut().push((retire_epoch, head));
                break Some(value);
            }
        };

        // --- UNPIN, then collect what is now safe. ---
        local.store(UNPINNED, Ordering::Release);
        self.collect();
        result
    }

    /// Whether the stack is currently empty (a momentary observation).
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Try to advance the global epoch from `epoch` to `epoch + 1`. Succeeds only if
    /// every *pinned* participant has announced `epoch` (none lags behind), which is
    /// what makes `epoch - 1` and earlier garbage safe once we reach `epoch + 1`.
    ///
    /// SOUNDNESS INVARIANT (the StoreLoad/Dekker barrier depends on it): the caller
    /// MUST have just pinned — i.e. executed a `Release` announce-store followed by a
    /// `fence(SeqCst)` — before calling this. That pin fence is what places the
    /// scanning thread in the single total order of SeqCst fences opposite every
    /// pinner's fence, so this scan cannot miss a concurrent pin and over-advance the
    /// epoch past a live reference. `pop` is the only caller and satisfies this
    /// (lines just above). A future caller that scans/advances WITHOUT a preceding
    /// pin fence would reopen the StoreLoad hazard (→ use-after-free) and must add its
    /// own `fence(SeqCst)` here first.
    fn try_advance(&self, epoch: usize) {
        for slot in &self.stack.locals {
            let v = slot.load(Ordering::Acquire);
            if v != UNPINNED && v != epoch {
                return; // a pinned participant lags; cannot advance yet
            }
        }
        // All pinned participants are at `epoch`; bump the global epoch (best effort).
        let _ = self.stack.global_epoch.compare_exchange(
            epoch,
            epoch.wrapping_add(1),
            Ordering::AcqRel,
            Ordering::Relaxed,
        );
    }

    /// Free every deferred node retired at an epoch `e_r` with `global ≥ e_r + 2`.
    fn collect(&self) {
        let global = self.stack.global_epoch.load(Ordering::Acquire);
        let mut garbage = self.garbage.borrow_mut();
        garbage.retain(|&(retire_epoch, node)| {
            if global.wrapping_sub(retire_epoch) >= 2 {
                // SAFETY: global ≥ retire_epoch + 2 ⇒ no participant can still be
                // pinned at retire_epoch, so no thread references this node; it was
                // unlinked by us (unique CAS winner) and its value already moved out,
                // so we free memory only.
                unsafe { drop(Box::from_raw(node)) };
                false
            } else {
                true // not yet safe; keep deferring
            }
        });
    }

    /// Push a still-unsafe node onto the stack's lock-free orphan list (used at drop).
    fn orphan(&self, node: *mut Node<T>) {
        loop {
            let head = self.stack.orphans.load(Ordering::Relaxed);
            // SAFETY: `node` is ours; reusing its `next` to link the orphan stack is
            // fine — it is no longer in the main list.
            unsafe { (*node).next = head };
            if self
                .stack
                .orphans
                .compare_exchange_weak(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }
    }
}

impl<T> Drop for Participant<'_, T> {
    fn drop(&mut self) {
        // Ensure we are unpinned, collect everything now safe, and hand any remaining
        // (not-yet-safe) deferrals to the stack's orphan list so they are freed at
        // stack teardown rather than leaked — sound because the stack outlives us and
        // is dropped only once no participant remains pinned.
        self.stack.locals[self.idx].store(UNPINNED, Ordering::Release);
        self.collect();
        let leftovers: Vec<*mut Node<T>> =
            self.garbage.borrow_mut().drain(..).map(|(_, n)| n).collect();
        for node in leftovers {
            self.orphan(node);
        }
        self.stack
            .slot_mask
            .fetch_and(!(1 << self.idx), Ordering::Release);
    }
}

// ===========================================================================
// Behavioural / invariant tests (std build; small variants under Miri).
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
        let s = EbrStack::new();
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

    // Drop-counting value type: proves every pushed value is dropped EXACTLY once —
    // popped values by the caller, never-popped values by EbrStack::drop — with no
    // double-free, and that epoch reclamation frees deferred nodes (no leak).
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
            let s = EbrStack::new();
            let p = s.register();
            for _ in 0..8 {
                p.push(DropCounter(Arc::clone(&drops)));
            }
            for _ in 0..5 {
                drop(p.pop().expect("value"));
            }
            assert_eq!(drops.load(O::Relaxed), 5, "popped values dropped once each");
            // p drop (collect + orphan remaining), then s drop (free orphans + the 3
            // never-popped nodes, dropping their live values).
        }
        assert_eq!(
            drops.load(O::Relaxed),
            8,
            "every value dropped exactly once (no leak, no double-free)"
        );
    }

    // Concurrent push/pop across SLOTS threads with REAL epoch reclamation in flight:
    // every pushed value is popped exactly once (no lost/duplicated element) and
    // (Miri verifies) no use-after-free. Hang-proof: a consumer stops only after all
    // producers finish AND the stack is empty.
    #[test]
    fn concurrent_conservation_with_reclamation() {
        let threads = SLOTS;
        let per = if cfg!(miri) { 12u32 } else { 3_000 };
        let s = Arc::new(EbrStack::new());
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
                for i in 0..per {
                    p.push(t as u32 * per + i);
                }
                started.fetch_add(1, O::Release);
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
// loom model — interleaving check of the epoch protocol, bounded preemption.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=2 cargo test loom_
// The concurrent region is kept to the pops (the novel pin/advance/reclaim race);
// elements are pre-pushed sequentially (concurrent push is proven in treiber_stack).
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_concurrent_pop_conservation_with_reclaim() {
        loom::model(|| {
            let s = Arc::new(EbrStack::new());
            {
                let p = s.register();
                p.push(1u32);
                p.push(2u32);
            }
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
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
            let s = Arc::new(EbrStack::new());
            {
                let p = s.register();
                p.push(42u32);
            }
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
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
