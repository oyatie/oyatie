//! # ksync_mpsc_ring — a sound lock-free MPSC ring buffer (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This is the
//! **multi-producer** generalization of the verified `crates_phase2/spsc_ring`
//! (the concurrent `kfifo`): many producer threads enqueue concurrently while a
//! single consumer dequeues. The single-producer ring synchronises with two indices
//! and a Release/Acquire on each; with *contending* producers that is no longer
//! enough (two producers could claim the same slot), so this uses the classic
//! **bounded MPMC sequence-number queue** (Dmitry Vyukov) run in MPSC mode. The
//! property that matters — every element enqueued by some producer is dequeued
//! exactly once, in per-slot publication order, with no lost/duplicated/torn element
//! — exists only across producer interleavings, so it is verified with **loom**
//! (interleavings) + **Miri** (UB) + invariant tests.
//!
//! ## Kernel provenance
//! The concurrent multi-producer counterpart of `lib/kfifo.c` / `kfifo.h`. The kernel
//! serialises multiple fifo producers with a spinlock (`__kfifo`/`ptr_ring`'s
//! producer lock); this slice instead ports the *lock-free* multi-producer
//! discipline (per-cell sequence numbers gating publication) that underlies
//! lock-free kernel-style per-CPU / event rings. It is the MPSC sibling of the
//! SPSC kfifo port, NOT a verbatim single Linux source file.
//!
//! ## How the sequence-number protocol stays sound
//! Each cell carries a `seq`. A producer at logical position `pos` may write cell
//! `pos % N` only when `seq == pos` (it then CAS-reserves `enqueue_pos` and, after
//! writing, publishes `seq = pos + 1`). The consumer at `pos` may read that cell
//! only when `seq == pos + 1`, and after reading sets `seq = pos + N` (freeing it
//! for the producer `N` positions later). So a given cell is written by exactly one
//! producer and read by the consumer strictly after that producer's `Release` store
//! of `seq` — never concurrently. The `enqueue_pos` CAS is `Relaxed` (it only
//! reserves an index); all cross-thread *data* visibility rides the per-cell
//! `seq` Acquire/Release. (No `SeqCst` is needed — there is no StoreLoad/Dekker
//! hazard here — which also keeps the loom model tractable.) This per-cell
//! single-writer/published-before-read invariant is what loom and Miri check.

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

// Arc shim: `alloc::sync::Arc` for the no_std production/Miri build (a no_std lib
// references `alloc`, never `std`); `loom::sync::Arc` under --cfg loom.
#[cfg(loom)]
use loom::sync::Arc;
#[cfg(not(loom))]
use alloc::sync::Arc;

/// One ring cell: a sequence number gating access plus the stored element.
struct Slot<T: Copy> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
}

/// Shared lock-free MPSC ring of `N` slots (`N` must be a power of two and `>= 2`).
/// Holds at most `N` in-flight `T: Copy` elements. Split into a cloneable
/// [`Producer`] (many threads may enqueue) and a single [`Consumer`] (`pop` takes
/// `&mut self`, so the type system enforces one consumer).
pub struct MpscRing<T: Copy, const N: usize> {
    /// Next logical position a producer will try to claim (free-running).
    enqueue_pos: AtomicUsize,
    /// Next logical position the consumer will read (free-running, single writer).
    dequeue_pos: AtomicUsize,
    buffer: [Slot<T>; N],
}

// SAFETY: the sequence-number protocol guarantees each cell is written by exactly
// one producer and read by the consumer only strictly after that producer's Release
// publication, so the interior `UnsafeCell`s are never accessed concurrently; the
// ring may therefore be shared across all producer threads and the consumer thread.
unsafe impl<T: Copy + Send, const N: usize> Sync for MpscRing<T, N> {}
unsafe impl<T: Copy + Send, const N: usize> Send for MpscRing<T, N> {}

impl<T: Copy + Default, const N: usize> MpscRing<T, N> {
    /// Build a ring and split it into a (cloneable) producer and a single consumer.
    /// `N` must be a power of two.
    // Returns an endpoint pair rather than `Self` — the channel-style split
    // constructor (cf. `mpsc::channel`); the ring itself is never exposed unsplit.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> (Producer<T, N>, Consumer<T, N>) {
        // N must be a power of two AND at least 2. N=1 is UNSOUND: with a single cell
        // the producer's publish sequence `pos + 1` equals the consumer's free
        // sequence `pos + N`, so "full (holds last lap's element)" and "free for this
        // lap" become indistinguishable — a second push would overwrite an unconsumed
        // element and race the consumer's read of the same cell (a data race). The
        // bare `is_power_of_two()` check is not enough because 1 is a power of two.
        assert!(
            N.is_power_of_two() && N >= 2,
            "ring size N must be a power of two and >= 2 (N=1 cannot distinguish full from empty)"
        );
        // Cell i starts "free for position i": seq == i.
        let buffer = core::array::from_fn(|i| Slot {
            seq: AtomicUsize::new(i),
            data: UnsafeCell::new(T::default()),
        });
        let ring = Arc::new(MpscRing {
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
            buffer,
        });
        (Producer(Arc::clone(&ring)), Consumer(ring))
    }
}

/// Producer endpoint. `Clone` + `Send` + `Sync`: any number of threads may enqueue
/// concurrently (each via its own clone or a shared `&Producer`).
pub struct Producer<T: Copy, const N: usize>(Arc<MpscRing<T, N>>);

impl<T: Copy, const N: usize> Clone for Producer<T, N> {
    fn clone(&self) -> Self {
        Producer(Arc::clone(&self.0))
    }
}

impl<T: Copy, const N: usize> Producer<T, N> {
    /// Enqueue one element (lock-free, callable concurrently). Returns `Err(value)`
    /// if the ring is full.
    pub fn push(&self, value: T) -> Result<(), T> {
        let r = &*self.0;
        let mut pos = r.enqueue_pos.load(Ordering::Relaxed);
        loop {
            let cell = &r.buffer[pos & (N - 1)];
            let seq = cell.seq.load(Ordering::Acquire);
            // Signed gap between the cell's sequence and our target position.
            let diff = seq.wrapping_sub(pos) as isize;
            if diff == 0 {
                // Cell is free for `pos`. Try to claim the position.
                match r.enqueue_pos.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // We exclusively own this cell until we publish `seq`.
                        // SAFETY: `seq == pos` means no producer holds this cell and
                        // the consumer has already freed it (it sets seq = pos for
                        // this lap); we are the sole writer until the Release store.
                        cell.data.with_mut(|p| unsafe { *p = value });
                        // Release: publishes the data write; the consumer's Acquire
                        // load of `seq` synchronises-with this, so it sees the value.
                        cell.seq.store(pos.wrapping_add(1), Ordering::Release);
                        return Ok(());
                    }
                    Err(actual) => pos = actual, // lost the race; retry at new pos
                }
            } else if diff < 0 {
                // seq < pos: the cell still holds an element not yet consumed for an
                // earlier lap — the ring is full.
                return Err(value);
            } else {
                // seq > pos: another producer already advanced; reload and retry.
                pos = r.enqueue_pos.load(Ordering::Relaxed);
            }
        }
    }
}

/// Consumer endpoint (single consumer: `pop` takes `&mut self`).
pub struct Consumer<T: Copy, const N: usize>(Arc<MpscRing<T, N>>);

impl<T: Copy, const N: usize> Consumer<T, N> {
    /// Dequeue one element in FIFO order. Returns `None` if the ring is empty.
    /// `&mut self` enforces a single consumer, so `dequeue_pos` has no contention.
    pub fn pop(&mut self) -> Option<T> {
        let r = &*self.0;
        // Single consumer owns `dequeue_pos`; a Relaxed read of our own index is fine.
        let pos = r.dequeue_pos.load(Ordering::Relaxed);
        let cell = &r.buffer[pos & (N - 1)];
        let seq = cell.seq.load(Ordering::Acquire);
        // The cell is readable for `pos` once a producer published `seq = pos + 1`.
        let diff = seq.wrapping_sub(pos.wrapping_add(1)) as isize;
        if diff == 0 {
            // SAFETY: `seq == pos + 1` means the producer for this position finished
            // its Release store of `seq` (which our Acquire load synchronised with),
            // so the slot is fully written and no producer will touch it until we
            // free it below; we are the only consumer, so this read is race-free.
            let value = cell.data.with_mut(|p| unsafe { *p });
            // Release: free this cell for the producer `N` positions later (it waits
            // for seq == pos + N), publishing that the data slot may be reused.
            cell.seq.store(pos.wrapping_add(N), Ordering::Release);
            // Advance our own position (single consumer, so no CAS needed).
            r.dequeue_pos.store(pos.wrapping_add(1), Ordering::Relaxed);
            Some(value)
        } else {
            // diff < 0: empty (producer hasn't published this position yet). diff > 0
            // cannot occur for a single consumer reading positions in order.
            None
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
        let (p, mut c) = MpscRing::<u32, 4>::new();
        assert_eq!(c.pop(), None);
        assert!(p.push(10).is_ok());
        assert!(p.push(20).is_ok());
        assert!(p.push(30).is_ok());
        assert!(p.push(40).is_ok());
        assert_eq!(p.push(50), Err(50)); // full (N=4)
        assert_eq!(c.pop(), Some(10));
        assert!(p.push(50).is_ok());
        assert_eq!(c.pop(), Some(20));
        assert_eq!(c.pop(), Some(30));
        assert_eq!(c.pop(), Some(40));
        assert_eq!(c.pop(), Some(50));
        assert_eq!(c.pop(), None);
    }

    // Multiple producers + one consumer: every value (disjoint per-producer ranges)
    // is received exactly once — no lost or duplicated element. Hang-proof: the
    // consumer stops once it has received `total` items; producers are finite.
    #[test]
    fn concurrent_multi_producer_conservation() {
        let producers = 4usize;
        let per = if cfg!(miri) { 25u32 } else { 50_000 };
        let total = producers as u32 * per;
        let (p, mut c) = MpscRing::<u32, 16>::new();

        let mut handles = Vec::new();
        for t in 0..producers {
            let p = p.clone();
            handles.push(thread::spawn(move || {
                for i in 0..per {
                    let v = t as u32 * per + i;
                    // Spin until the bounded ring accepts it.
                    while p.push(v).is_err() {
                        std::hint::spin_loop();
                    }
                }
            }));
        }

        let seen = StdArc::new(Mutex::new(BTreeSet::new()));
        let mut received = 0u32;
        while received < total {
            if let Some(v) = c.pop() {
                assert!(seen.lock().unwrap().insert(v), "duplicate element {v}");
                received += 1;
            } else {
                std::hint::spin_loop();
            }
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(c.pop(), None, "ring should be drained");
        assert_eq!(seen.lock().unwrap().len(), total as usize, "lost/duplicated");
    }

    // Regression guard for the audit-found N=1 soundness hole: a capacity-1 ring
    // cannot distinguish full from empty in the sequence protocol, so the
    // constructor must reject it (1 passes is_power_of_two() but fails the N>=2 gate).
    #[test]
    #[should_panic(expected = "must be a power of two and >= 2")]
    fn rejects_capacity_one() {
        let _ = MpscRing::<u32, 1>::new();
    }
}

// ===========================================================================
// loom model — interleaving check of the multi-producer sequence protocol.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;

    #[test]
    fn loom_two_producers_one_consumer_conservation() {
        loom::model(|| {
            // Capacity-2 ring; two producers concurrently enqueue one distinct value
            // each, the (main) consumer drains both. Across every explored
            // interleaving the two values arrive exactly once with none lost,
            // duplicated, or torn — proving the contending-producer claim protocol
            // (Relaxed pos-CAS + per-cell Release/Acquire seq) is correct.
            let (p, mut c) = MpscRing::<u8, 2>::new();
            let p1 = p.clone();
            let p2 = p.clone();
            // yield_now() on the not-ready branch tells loom the thread is spinning so
            // it deschedules and lets the thread that makes progress run; without it
            // loom keeps the spinner running and exceeds its per-execution branch
            // budget ("Model exceeded maximum number of branches").
            let t1 = loom::thread::spawn(move || {
                while p1.push(1u8).is_err() {
                    loom::thread::yield_now();
                }
            });
            let t2 = loom::thread::spawn(move || {
                while p2.push(2u8).is_err() {
                    loom::thread::yield_now();
                }
            });

            let mut got: Vec<u8> = Vec::new();
            while got.len() < 2 {
                if let Some(v) = c.pop() {
                    got.push(v);
                } else {
                    loom::thread::yield_now();
                }
            }
            t1.join().unwrap();
            t2.join().unwrap();
            got.sort_unstable();
            assert_eq!(got, vec![1, 2], "lost/duplicated element: {got:?}");
        });
    }

    #[test]
    fn loom_producer_consumer_no_torn_value() {
        loom::model(|| {
            // One producer enqueues a single value while the consumer races to read;
            // the consumer must observe either nothing yet or the exact value 0xAB
            // (never a torn/partial slot), and exactly once.
            let (p, mut c) = MpscRing::<u8, 2>::new();
            let prod = loom::thread::spawn(move || {
                while p.push(0xABu8).is_err() {
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
            // Drain in case the consumer observed empty before the publish.
            while got.is_none() {
                got = c.pop();
            }
            assert_eq!(got, Some(0xAB), "torn or lost value");
            assert_eq!(c.pop(), None, "duplicated value");
        });
    }
}
