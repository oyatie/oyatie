//! # ksync_mpmc_queue — a sound lock-free MPMC ring buffer (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This completes the
//! queue family: `spsc_ring` (single producer / single consumer) → `mpsc_ring`
//! (many producers, one consumer) → **`mpmc_queue` (many producers AND many
//! consumers)**. It is the full **bounded MPMC sequence-number queue** (Dmitry
//! Vyukov): `mpsc_ring` already used this algorithm but kept the consumer single
//! (`pop(&mut self)`, no dequeue contention); here both ends contend, so the
//! **dequeue side also CAS-reserves** its position. The new property over `mpsc_ring`
//! is *consumer-side* correctness: two consumers must never dequeue the same slot, no
//! element is lost or delivered twice, across all interleavings — verified with loom
//! + Miri + invariant tests.
//!
//! ## Kernel provenance
//! The fully-concurrent counterpart of `lib/kfifo.c` — the MPMC generalization of the
//! SPSC/MPSC fifo ports. The kernel serialises multi-end fifo access with locks; this
//! ports the lock-free per-cell-sequence discipline. Not a verbatim Linux file.
//!
//! ## Protocol (and why it stays sound under contention)
//! Each cell has a `seq`. A producer at logical position `p` writes cell `p % N` only
//! when `seq == p` (CAS-reserve `enqueue_pos`, write, publish `seq = p + 1`). A
//! consumer at position `c` reads that cell only when `seq == c + 1` (CAS-reserve
//! `dequeue_pos`, read, then publish `seq = c + N` to free it for the producer `N`
//! laps later). So each cell is written by exactly one producer and read by exactly
//! one consumer per lap, the reader strictly after the writer's `Release` of `seq`
//! (paired with the reader's `Acquire`) — never concurrently. The `enqueue_pos` /
//! `dequeue_pos` CASes are `Relaxed` (they only reserve a position; all cross-thread
//! *data* visibility rides the per-cell `seq` Acquire/Release). No `SeqCst` is needed
//! (no StoreLoad/Dekker hazard), which keeps the loom model tractable. This per-cell
//! single-writer/single-reader-published-before-read invariant is what loom and Miri
//! check.
//!
//! ## Scope (documented, not faked)
//! Fixed power-of-two `N >= 2` capacity; `T: Copy` (no Drop of in-flight elements);
//! returns elements by value. A verified bounded-MPMC core, not the full kfifo API.
//! Not Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(loom)]
use loom::cell::UnsafeCell;

#[cfg(not(loom))]
struct UnsafeCell<T>(core::cell::UnsafeCell<T>);
#[cfg(not(loom))]
impl<T> UnsafeCell<T> {
    #[inline]
    fn new(v: T) -> Self {
        Self(core::cell::UnsafeCell::new(v))
    }
    #[inline]
    fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

#[cfg(loom)]
use loom::sync::Arc;
#[cfg(not(loom))]
use alloc::sync::Arc;

/// One ring cell: a sequence number gating access plus the stored element.
struct Slot<T: Copy> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
}

/// Shared lock-free MPMC ring of `N` slots (`N` a power of two, `>= 2`). Holds at most
/// `N` in-flight `T: Copy` elements. Split into cloneable [`Producer`] and [`Consumer`]
/// endpoints; any number of threads may enqueue and any number may dequeue.
pub struct MpmcQueue<T: Copy, const N: usize> {
    enqueue_pos: AtomicUsize,
    dequeue_pos: AtomicUsize,
    buffer: [Slot<T>; N],
}

// SAFETY: the sequence-number protocol gives each cell exactly one producer-writer and
// one consumer-reader per lap, the reader ordered strictly after the writer's Release
// publication, so the interior `UnsafeCell`s are never accessed concurrently; the
// queue may be shared across all producer and consumer threads.
unsafe impl<T: Copy + Send, const N: usize> Sync for MpmcQueue<T, N> {}
unsafe impl<T: Copy + Send, const N: usize> Send for MpmcQueue<T, N> {}

impl<T: Copy + Default, const N: usize> MpmcQueue<T, N> {
    /// Build a queue and split it into cloneable producer and consumer endpoints.
    /// `N` must be a power of two and `>= 2`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> (Producer<T, N>, Consumer<T, N>) {
        assert!(
            N.is_power_of_two() && N >= 2,
            "ring size N must be a power of two and >= 2"
        );
        let buffer = core::array::from_fn(|i| Slot {
            seq: AtomicUsize::new(i),
            data: UnsafeCell::new(T::default()),
        });
        let q = Arc::new(MpmcQueue {
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
            buffer,
        });
        (Producer(Arc::clone(&q)), Consumer(q))
    }
}

/// Producer endpoint (`Clone` + `Send` + `Sync`): any number of threads may enqueue.
pub struct Producer<T: Copy, const N: usize>(Arc<MpmcQueue<T, N>>);
/// Consumer endpoint (`Clone` + `Send` + `Sync`): any number of threads may dequeue.
pub struct Consumer<T: Copy, const N: usize>(Arc<MpmcQueue<T, N>>);

impl<T: Copy, const N: usize> Clone for Producer<T, N> {
    fn clone(&self) -> Self {
        Producer(Arc::clone(&self.0))
    }
}
impl<T: Copy, const N: usize> Clone for Consumer<T, N> {
    fn clone(&self) -> Self {
        Consumer(Arc::clone(&self.0))
    }
}

impl<T: Copy, const N: usize> Producer<T, N> {
    /// Enqueue one element (lock-free, concurrent). Returns `Err(value)` if full.
    pub fn push(&self, value: T) -> Result<(), T> {
        let r = &*self.0;
        let mut pos = r.enqueue_pos.load(Ordering::Relaxed);
        loop {
            let cell = &r.buffer[pos & (N - 1)];
            let seq = cell.seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(pos) as isize;
            if diff == 0 {
                match r.enqueue_pos.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY: seq == pos means we exclusively reserved this cell for
                        // this lap (no producer holds it, the consumer already freed it);
                        // we are the sole writer until the Release store below.
                        cell.data.with_mut(|p| unsafe { *p = value });
                        cell.seq.store(pos.wrapping_add(1), Ordering::Release);
                        return Ok(());
                    }
                    Err(actual) => pos = actual,
                }
            } else if diff < 0 {
                return Err(value); // full
            } else {
                pos = r.enqueue_pos.load(Ordering::Relaxed);
            }
        }
    }
}

impl<T: Copy, const N: usize> Consumer<T, N> {
    /// Dequeue one element in FIFO order (lock-free, concurrent). Returns `None` if
    /// empty. Multiple consumers contend: the `dequeue_pos` CAS gives each a distinct
    /// position, so no two consumers ever read the same cell.
    pub fn pop(&self) -> Option<T> {
        let r = &*self.0;
        let mut pos = r.dequeue_pos.load(Ordering::Relaxed);
        loop {
            let cell = &r.buffer[pos & (N - 1)];
            let seq = cell.seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(pos.wrapping_add(1)) as isize;
            if diff == 0 {
                match r.dequeue_pos.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY: seq == pos + 1 means the producer for this position
                        // finished its Release store of seq (our Acquire load synced with
                        // it), so the slot is fully written; we exclusively reserved this
                        // position via the CAS, so no other consumer reads it.
                        let value = cell.data.with_mut(|p| unsafe { *p });
                        // Release: free the cell for the producer N positions later.
                        cell.seq.store(pos.wrapping_add(N), Ordering::Release);
                        return Some(value);
                    }
                    Err(actual) => pos = actual,
                }
            } else if diff < 0 {
                return None; // empty
            } else {
                pos = r.dequeue_pos.load(Ordering::Relaxed);
            }
        }
    }
}

// ===========================================================================
// Behavioural / invariant tests (std build; small variants under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc as StdArc, Mutex};
    use std::thread;

    #[test]
    fn single_thread_fifo_and_full_empty() {
        let (p, c) = MpmcQueue::<u32, 4>::new();
        assert_eq!(c.pop(), None);
        for v in [10, 20, 30, 40] {
            assert!(p.push(v).is_ok());
        }
        assert_eq!(p.push(50), Err(50)); // full
        assert_eq!(c.pop(), Some(10));
        assert!(p.push(50).is_ok());
        for want in [20, 30, 40, 50] {
            assert_eq!(c.pop(), Some(want));
        }
        assert_eq!(c.pop(), None);
    }

    // Many producers + many consumers: every value (disjoint per-producer ranges) is
    // dequeued exactly once across all consumers — none lost, none delivered twice.
    #[test]
    fn concurrent_mpmc_conservation() {
        let producers = 3usize;
        let consumers = 3usize;
        let per = if cfg!(miri) { 20u32 } else { 20_000 };
        let total = producers as u32 * per;
        let (p, c) = MpmcQueue::<u32, 16>::new();

        let mut handles = Vec::new();
        for t in 0..producers {
            let p = p.clone();
            handles.push(thread::spawn(move || {
                for i in 0..per {
                    let v = t as u32 * per + i;
                    while p.push(v).is_err() {
                        std::hint::spin_loop();
                    }
                }
            }));
        }

        let seen = StdArc::new(Mutex::new(BTreeSet::new()));
        let remaining = StdArc::new(AtomicUsize::new(total as usize));
        let mut chandles = Vec::new();
        for _ in 0..consumers {
            let c = c.clone();
            let seen = StdArc::clone(&seen);
            let remaining = StdArc::clone(&remaining);
            chandles.push(thread::spawn(move || {
                loop {
                    if remaining.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    if let Some(v) = c.pop() {
                        assert!(seen.lock().unwrap().insert(v), "duplicate dequeue of {v}");
                        remaining.fetch_sub(1, Ordering::AcqRel);
                    } else {
                        std::hint::spin_loop();
                    }
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        for h in chandles {
            h.join().unwrap();
        }
        assert_eq!(c.pop(), None, "queue drained");
        assert_eq!(seen.lock().unwrap().len(), total as usize, "lost/duplicated");
    }
}

// ===========================================================================
// loom model — the new property vs mpsc_ring: CONSUMER-side contention.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;

    #[test]
    fn loom_two_consumers_no_double_dequeue() {
        loom::model(|| {
            // Pre-fill two distinct values, then two consumers race to dequeue. Each
            // value is taken by exactly one consumer — never the same value twice, and
            // none lost. This is the property mpsc_ring (single consumer) could not
            // exercise: the dequeue_pos CAS must serialise the two consumers.
            let (p, c) = MpmcQueue::<u8, 2>::new();
            p.push(1).unwrap();
            p.push(2).unwrap();
            let c1 = c.clone();
            let c2 = c.clone();
            let t1 = loom::thread::spawn(move || c1.pop());
            let t2 = loom::thread::spawn(move || c2.pop());
            let a = t1.join().unwrap();
            let b = t2.join().unwrap();
            let mut got: Vec<u8> = Vec::new();
            got.extend(a);
            got.extend(b);
            while let Some(v) = c.pop() {
                got.push(v);
            }
            got.sort_unstable();
            assert_eq!(got, vec![1, 2], "lost/duplicated under consumer contention: {got:?}");
        });
    }

    #[test]
    fn loom_producer_consumer_pair() {
        loom::model(|| {
            // One producer + one consumer racing on a 2-slot queue: the consumer sees
            // the exact value (never torn), exactly once.
            let (p, c) = MpmcQueue::<u8, 2>::new();
            let prod = loom::thread::spawn(move || {
                while p.push(0xCD).is_err() {
                    loom::thread::yield_now();
                }
            });
            let mut got = None;
            for _ in 0..3 {
                if let Some(v) = c.pop() {
                    got = Some(v);
                    break;
                }
                loom::thread::yield_now();
            }
            prod.join().unwrap();
            while got.is_none() {
                got = c.pop();
            }
            assert_eq!(got, Some(0xCD), "torn or lost value");
            assert_eq!(c.pop(), None, "duplicated value");
        });
    }
}
