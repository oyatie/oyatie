//! # ksync_treiber_stack — a lock-free Treiber stack (Phase 2, hard tier)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This is the
//! genuinely-concurrent version of the kernel's lock-less list (`llist`), which
//! Phase 1 modelled single-threaded against a C2Rust oracle. Here the property
//! that matters — that concurrent `push`/`pop` linearise with no lost or
//! duplicated element — exists only across thread interleavings, so it is verified
//! with **loom** (all interleavings) + **Miri** (UB) + invariant tests.
//!
//! ## Kernel provenance
//! `llist_add` / `llist_del_first` (`include/linux/llist.h`, `lib/llist.c`): a
//! singly-linked LIFO list mutated by a `cmpxchg` loop on the head — the classic
//! Treiber stack. Here `head: AtomicPtr<Node<T>>`, push CAS-prepends, pop CAS-pops.
//!
//! ## THE HARD PART — memory reclamation (named honestly, not faked)
//! A reclaiming Treiber stack has a **use-after-free**: between a popper loading
//! `head` and dereferencing it, another thread can pop and free that same node.
//! Making `pop` sound therefore requires *safe memory reclamation* — **hazard
//! pointers** or **epoch-based reclamation** (e.g. crossbeam-epoch) — which is the
//! real, hard, open part of lock-free programming and a substantial slice of its
//! own.
//!
//! This slice deliberately does NOT pretend to solve that. Instead `pop` **leaks**
//! the popped node (never frees it). Leaking is *genuinely sound* — a never-freed
//! node stays valid forever, so there is no use-after-free and no double-free
//! (Miri confirms). What loom verifies here is the **lock-free algorithm itself**
//! (the push/pop `cmpxchg` interleavings: LIFO order, no lost/duplicated element),
//! independent of reclamation. The leak is the explicit, documented boundary: a
//! production version must add hazard-pointer/epoch reclamation to free nodes.
//!
//! ## Why leaking also removes data races (structural ABA-elimination)
//! Leaking does more than kill use-after-free — it kills the **ABA problem** too,
//! which is what makes every node-field access race-free. Because a popped node is
//! never freed *and* `push` always allocates a **fresh** node (it never re-inserts
//! an existing one), a node address that was `head` once can never become `head`
//! again. So for any node `X`, the state `head == X` occurs **at most once** in the
//! whole program lifetime ⇒ exactly one `pop` ever CAS-wins on `X` and reads its
//! value exactly once; `X.next` is written once (pre-publish) and read-only after.
//! Every cross-thread field access is separated by a Release-CAS / Acquire-load
//! happens-before edge, so no read races a write under the Rust memory model — the
//! property a *reclaiming* stack would need hazard pointers to recover. (Confirmed
//! by an independent two-lens soundness audit, 2026-06-04: verdict sound.)

#[cfg(loom)]
use loom::sync::atomic::{AtomicPtr, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(not(loom))]
use alloc::boxed::Box;

use core::ptr;

struct Node<T> {
    next: *mut Node<T>,
    value: T,
}

/// A lock-free LIFO stack. `push`/`pop` are safe and may be called concurrently
/// from any number of threads. NOTE: `pop` leaks the node it removes (see the
/// crate-level "memory reclamation" section); use only where that is acceptable
/// or as the verified algorithm core under a future reclamation scheme.
pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

// SAFETY: `head` is atomic and the only shared mutable state; nodes are published
// via Release CAS and read via Acquire load, so the linked structure is race-free.
// Nodes are never freed (leaked), so no use-after-free can arise from sharing.
unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Push a value onto the stack (lock-free).
    pub fn push(&self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            next: ptr::null_mut(),
            value,
        }));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            // SAFETY: `node` is our freshly-allocated, not-yet-published node, so
            // we are its only accessor; writing its `next` field is exclusive.
            unsafe { (*node).next = head };
            // Release: publishes the node (and its `next` write) to a popper's
            // Acquire load of `head`. On failure, retry with the new head.
            match self.head.compare_exchange_weak(
                head,
                node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(_) => continue,
            }
        }
    }

    /// Pop the most-recently-pushed value (lock-free). Returns `None` if empty.
    ///
    /// The removed node is leaked (not freed) — see the crate-level reclamation
    /// note. This keeps the dereference below sound without hazard pointers.
    pub fn pop(&self) -> Option<T> {
        loop {
            // Acquire: pairs with a pusher's Release CAS so the node and its
            // `next` field are visible.
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }
            // SAFETY: nodes are never freed (leaked), so `head` points to valid
            // memory even if another thread is concurrently popping it; reading
            // `next` cannot be a use-after-free. (This is exactly the property a
            // reclaiming stack would need hazard pointers to guarantee.)
            let next = unsafe { (*head).next };
            // Try to swing head to next. The Acquire half synchronizes with a
            // concurrent push/pop Release store; that is the only ordering this CAS
            // strictly needs (on success it stores `next`, an already-published
            // pointer, so it publishes no new data; on failure it re-loads `head`
            // with Acquire next iteration). AcqRel/Acquire is therefore deliberately
            // conservative — never wrong, just stronger than the minimal Acquire/
            // Relaxed; kept for clarity and margin in a kernel primitive.
            if self
                .head
                .compare_exchange_weak(head, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // We won this node exclusively (CAS is atomic, exactly one popper
                // succeeds for a given `head`). SAFETY: read the value out once;
                // the node is leaked, so no Drop/free races our read.
                let value = unsafe { ptr::read(&(*head).value) };
                return Some(value);
            }
            spin_hint();
        }
    }

    /// Whether the stack is currently empty (a momentary observation).
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

impl<T> Default for TreiberStack<T> {
    fn default() -> Self {
        Self::new()
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
// Behavioural / invariant tests (std; small variant under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_lifo() {
        let s = TreiberStack::new();
        assert!(s.is_empty());
        assert_eq!(s.pop(), None);
        s.push(1);
        s.push(2);
        s.push(3);
        assert_eq!(s.pop(), Some(3));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    // Concurrent push/pop: every pushed value is popped exactly once across all
    // threads — no lost or duplicated element. (Miri runs a small version.)
    #[test]
    fn concurrent_push_pop_conservation() {
        use std::sync::atomic::{AtomicBool, Ordering as O};

        let threads = if cfg!(miri) { 3 } else { 8 };
        let per = if cfg!(miri) { 20u32 } else { 5_000 };
        let s = Arc::new(TreiberStack::new());
        let total = threads as u32 * per;

        // Producers push disjoint id ranges so every value is globally unique.
        let mut producers = Vec::new();
        for t in 0..threads {
            let s = Arc::clone(&s);
            producers.push(thread::spawn(move || {
                for i in 0..per {
                    s.push(t as u32 * per + i);
                }
            }));
        }

        // Consumers drain into per-thread sets. Termination is HANG-PROOF and does
        // not assume conservation: a consumer stops once (a) all producers have
        // joined — so every value is already on the stack or already popped — and
        // (b) the stack is observed empty. In this stack `head == null` iff zero
        // nodes remain (an in-flight pop keeps its node linked until its CAS), and
        // after `producers_done` no node is ever added, so an empty observation is
        // stable. A buggy algorithm that lost an element therefore terminates and
        // FAILS the final count assert, rather than spinning forever.
        let producers_done = Arc::new(AtomicBool::new(false));
        let popped = Arc::new(std::sync::Mutex::new(BTreeSet::new()));
        let mut consumers = Vec::new();
        for _ in 0..threads {
            let s = Arc::clone(&s);
            let popped = Arc::clone(&popped);
            let producers_done = Arc::clone(&producers_done);
            consumers.push(thread::spawn(move || {
                let mut local = Vec::new();
                loop {
                    if let Some(v) = s.pop() {
                        local.push(v);
                    } else if producers_done.load(O::Acquire) && s.is_empty() {
                        break;
                    }
                }
                let mut g = popped.lock().unwrap();
                for v in local {
                    assert!(g.insert(v), "duplicate pop of {v}");
                }
            }));
        }

        for h in producers {
            h.join().unwrap();
        }
        producers_done.store(true, O::Release);
        for h in consumers {
            h.join().unwrap();
        }

        // Stack fully drained and every pushed element popped exactly once.
        assert!(s.is_empty(), "stack not drained");
        let g = popped.lock().unwrap();
        assert_eq!(g.len(), total as usize, "lost or duplicated elements");
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of push/pop linearizability.
//   RUSTFLAGS="--cfg loom" cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_two_pushers_one_popper_conservation() {
        loom::model(|| {
            let s = Arc::new(TreiberStack::new());
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            // Two threads push 1 and 2 concurrently; the main thread pops up to two
            // items. Across EVERY interleaving the popped set is a subset of {1,2}
            // with no duplicates, and a final drain recovers the rest — i.e. each
            // pushed element is popped exactly once (no loss, no duplication, no
            // torn pointer).
            let t1 = loom::thread::spawn(move || s1.push(1u32));
            let t2 = loom::thread::spawn(move || s2.push(2u32));
            let mut got = Vec::new();
            for _ in 0..2 {
                if let Some(v) = s.pop() {
                    got.push(v);
                }
            }
            t1.join().unwrap();
            t2.join().unwrap();
            while let Some(v) = s.pop() {
                got.push(v);
            }
            got.sort_unstable();
            assert_eq!(got, vec![1, 2], "lost/duplicated element: {got:?}");
        });
    }

    #[test]
    fn loom_concurrent_pop_no_double_take() {
        loom::model(|| {
            let s = Arc::new(TreiberStack::new());
            s.push(42u32);
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            // Two poppers race for one element: exactly one gets Some(42), the
            // other gets None — never both Some (which would be a double-take of a
            // single node).
            let t1 = loom::thread::spawn(move || s1.pop());
            let t2 = loom::thread::spawn(move || s2.pop());
            let a = t1.join().unwrap();
            let b = t2.join().unwrap();
            let takes = a.iter().count() + b.iter().count();
            assert_eq!(takes, 1, "element taken {takes} times (want exactly 1)");
            assert!(a == Some(42) || b == Some(42));
        });
    }
}
