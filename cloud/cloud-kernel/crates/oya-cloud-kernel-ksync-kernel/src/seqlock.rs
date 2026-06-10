//! # ksync_seqlock — a *sound* seqlock, and the documented UB boundary (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This slice exists
//! as much to **document a soundness boundary** as to ship code — exactly the
//! Phase-2 honesty principle ("flag the primitives whose kernel design is UB-in-Rust;
//! don't fake them").
//!
//! ## Kernel provenance — and why the literal port is UB in Rust
//! A kernel seqlock (`include/linux/seqlock.h`) lets readers run **without blocking
//! writers**: the writer bumps a sequence counter to odd before writing and to even
//! after; a reader snapshots `seq`, reads the protected data, re-reads `seq`, and
//! **retries** if it changed or was odd. Crucially, the reader reads the data
//! *concurrently with the writer's write* and simply **discards** any torn value it
//! may have seen.
//!
//! That "benign data race" is **Undefined Behaviour in the C11/C++11/Rust memory
//! model**: a non-atomic read racing a non-atomic write to the same location is a
//! data race, which is UB *regardless of whether the result is later discarded*. The
//! model has no notion of "read garbage but it's OK because I'll throw it away" — the
//! race itself is the UB. (This is the same edge Mara Bos's `SeqLock` sits on: it
//! reads the payload through `UnsafeCell`+`ptr::read` while a writer may be writing,
//! and notes this is technically a race the abstract machine forbids.) So a *verbatim*
//! Rust port of the kernel seqlock — payload in a plain `UnsafeCell<T>`, read with
//! `ptr::read` during a possible write — is **unsound**; Miri's data-race detector
//! flags it. **This crate does NOT ship that.**
//!
//! ## The sound version (what this crate ships)
//! Make every payload access **atomic**. Atomic accesses *cannot* form a data race,
//! so reading the payload concurrently with a write is well-defined (the reader may
//! observe an old or new value per field, which the seq-counter retry then rejects if
//! inconsistent). The protected data here is a pair of [`AtomicU64`] words; `read`
//! returns a **consistent snapshot** — both words from the *same* write — by the
//! classic seqcount protocol (snapshot `seq`, read both words, `Acquire` fence,
//! re-read `seq`, retry on mismatch/odd). loom exhaustively explores this small model
//! and finds no torn snapshot on any interleaving; Miri finds no UB (no data race —
//! all payload accesses are atomic, the very property the literal port lacks). (Both
//! are *bug-finders*, not proofs of absence: loom is exhaustive only over its bounded
//! model, and Miri checks only the executions/seeds it runs — but here every fence is
//! load-bearing, confirmed by weakening each one and watching loom report a tear.)
//!
//! ## Scope (documented, not faked)
//! Single writer (the `seqcount_t` contract — multi-writer needs an external writer
//! lock; out of scope, would compose with `ksync_spinlock`). A fixed two-word `u64`
//! payload (the canonical seqlock use: read a multi-word value — e.g. a `{sec, nsec}`
//! timestamp — consistently without blocking the writer). The general lesson: a
//! *sound* seqlock requires atomic payload access; the kernel's non-atomic design is
//! the UB boundary. Not Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{fence, AtomicU64, AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{fence, AtomicU64, AtomicUsize, Ordering};

/// A seqlock guarding a two-word `u64` payload, read without blocking the (single)
/// writer. Reads return a consistent snapshot (both words from the same write).
pub struct SeqLock {
    /// Sequence counter: even = stable, odd = a write is in progress.
    seq: AtomicUsize,
    /// Two payload words, accessed **atomically** (this is what makes it sound — the
    /// kernel's non-atomic payload would be a data race / UB; see the module docs).
    a: AtomicU64,
    b: AtomicU64,
}

// NOTE: no `unsafe impl Send/Sync` is needed — every field is atomic, so `SeqLock`
// auto-derives `Send + Sync`. That is the headline of the *sound* design: it contains
// **zero `unsafe`**, precisely because the payload is accessed atomically. The literal
// kernel/`UnsafeCell`+`ptr::read` port would require `unsafe` AND be a data race (UB);
// using atomics removes both the `unsafe` and the UB (see the module docs).

impl SeqLock {
    #[cfg(not(loom))]
    pub const fn new(a: u64, b: u64) -> Self {
        Self {
            seq: AtomicUsize::new(0),
            a: AtomicU64::new(a),
            b: AtomicU64::new(b),
        }
    }

    #[cfg(loom)]
    pub fn new(a: u64, b: u64) -> Self {
        Self {
            seq: AtomicUsize::new(0),
            a: AtomicU64::new(a),
            b: AtomicU64::new(b),
        }
    }

    /// Read a consistent snapshot `(a, b)` — both words from the same write. Lock-free
    /// for readers (never blocks the writer); retries while a write is in progress or
    /// a write lands mid-read.
    pub fn read(&self) -> (u64, u64) {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            if s1 & 1 != 0 {
                // A write is in progress; the snapshot would be inconsistent. Retry.
                spin_hint();
                continue;
            }
            // Atomic payload loads — NOT a data race even if a writer stores
            // concurrently (the whole point vs the unsound non-atomic kernel design).
            let a = self.a.load(Ordering::Relaxed);
            let b = self.b.load(Ordering::Relaxed);
            // Acquire fence: order the payload loads before the validation re-read, so
            // a write that lands after our loads is detected by `s1 != s2`.
            fence(Ordering::Acquire);
            let s2 = self.seq.load(Ordering::Relaxed);
            if s1 == s2 {
                // No write occurred between the two seq reads ⇒ `a` and `b` are from
                // the same stable version: a consistent snapshot.
                return (a, b);
            }
            // A write landed mid-read; the snapshot may be torn. Retry.
            spin_hint();
        }
    }

    /// Update the payload (single-writer contract). Bumps `seq` to odd, writes both
    /// words, bumps `seq` to even; concurrent readers retry across the odd window and
    /// observe either the full old or full new snapshot — never a mix.
    ///
    /// # Single-writer contract
    /// At most one thread may call `write` at a time (like the kernel `seqcount_t`).
    /// Concurrent writers would corrupt the parity sequence; serialize them externally
    /// (e.g. with a spinlock) if multiple writers are needed.
    pub fn write(&self, a: u64, b: u64) {
        let s = self.seq.load(Ordering::Relaxed);
        debug_assert!(s & 1 == 0, "seqlock: concurrent writers (parity already odd)");
        // Enter the write: seq -> odd. Readers that observe odd will retry.
        self.seq.store(s.wrapping_add(1), Ordering::Relaxed);
        // Release fence: order the odd-seq publication before the payload stores, so a
        // reader cannot see new payload while still observing the old (even) seq.
        fence(Ordering::Release);
        self.a.store(a, Ordering::Relaxed);
        self.b.store(b, Ordering::Relaxed);
        // Exit the write: seq -> even (Release publishes the payload to a reader's
        // Acquire load of seq).
        self.seq.store(s.wrapping_add(2), Ordering::Release);
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
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_snapshot() {
        let s = SeqLock::new(1, 1);
        assert_eq!(s.read(), (1, 1));
        s.write(7, 7);
        assert_eq!(s.read(), (7, 7));
    }

    // One writer keeps the invariant a == b (writes the same counter to both words);
    // concurrent readers must NEVER observe a torn snapshot (a from one write, b from
    // another). Miri additionally proves there is no data race (payload is atomic).
    #[test]
    fn concurrent_reader_never_sees_torn_snapshot() {
        let iters = if cfg!(miri) { 50u64 } else { 200_000 };
        let readers = if cfg!(miri) { 2 } else { 4 };
        let s = Arc::new(SeqLock::new(0, 0));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let mut hs = Vec::new();
        for _ in 0..readers {
            let s = Arc::clone(&s);
            let stop = Arc::clone(&stop);
            hs.push(thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    let (a, b) = s.read();
                    assert_eq!(a, b, "TORN SNAPSHOT: a={a} b={b} from different writes");
                }
            }));
        }
        // Single writer.
        let sw = Arc::clone(&s);
        let writer = thread::spawn(move || {
            for i in 1..=iters {
                sw.write(i, i); // a == b == i for every consistent snapshot
            }
        });
        writer.join().unwrap();
        stop.store(true, Ordering::Release);
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(s.read(), (iters, iters));
    }
}

// ===========================================================================
// loom model — the reader never returns a TORN snapshot across all interleavings.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_reader_snapshot_is_consistent() {
        loom::model(|| {
            // Start at (0,0). A single writer publishes (5,5). A concurrent reader must
            // observe EITHER (0,0) or (5,5) — never (0,5) or (5,0) — across every
            // interleaving, because the seq-counter retry rejects a snapshot torn by a
            // concurrent write. (Payload is atomic, so the read itself is never UB.)
            let s = Arc::new(SeqLock::new(0, 0));
            let sw = Arc::clone(&s);
            let writer = loom::thread::spawn(move || sw.write(5, 5));
            let (a, b) = s.read();
            assert_eq!(a, b, "torn snapshot: ({a}, {b})");
            assert!(a == 0 || a == 5, "unexpected value {a}");
            writer.join().unwrap();
            assert_eq!(s.read(), (5, 5));
        });
    }

    #[test]
    fn loom_two_writes_reader_consistent() {
        loom::model(|| {
            // The writer does two updates (1,1) then (2,2); a concurrent reader's
            // snapshot is always a == b (one of (0,0)/(1,1)/(2,2)) — never torn.
            let s = Arc::new(SeqLock::new(0, 0));
            let sw = Arc::clone(&s);
            let writer = loom::thread::spawn(move || {
                sw.write(1, 1);
                sw.write(2, 2);
            });
            let (a, b) = s.read();
            assert_eq!(a, b, "torn snapshot: ({a}, {b})");
            writer.join().unwrap();
            assert_eq!(s.read(), (2, 2));
        });
    }
}
