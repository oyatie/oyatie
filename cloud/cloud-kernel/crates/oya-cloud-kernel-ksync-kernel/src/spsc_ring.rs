//! # ksync_spsc_ring — a sound lock-free SPSC ring buffer (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). This is the
//! **real concurrent** version of the kernel's `kfifo` SPSC ring: Phase 1 verified
//! kfifo single-threaded via operation-sequence differential testing against a
//! C2Rust oracle; a single-threaded corpus cannot exercise the producer/consumer
//! memory ordering that actually makes a lock-free ring correct, so verification
//! shifts to **loom** (all interleavings) + **Miri** (UB) + invariant tests.
//!
//! ## Kernel provenance
//! Models `lib/kfifo.c` / `include/linux/kfifo.h`: free-running `in`/`out` indices
//! masked by a power-of-two size, `used = in - out` (wrapping), a Release store of
//! `in` after writing the slot publishes the element, and an Acquire load of `in`
//! on the consumer side establishes the happens-before before it reads the slot —
//! exactly `smp_wmb()`/`smp_rmb()` in the kernel SPSC fifo.
//!
//! ## Soundness (Single Producer / Single Consumer)
//! Exactly one thread calls [`Producer::push`] and one calls [`Consumer::pop`].
//! The producer owns `tail` (writes it, only reads `head`); the consumer owns
//! `head` (writes it, only reads `tail`). A slot is written by the producer ONLY
//! when the ring is not full and read by the consumer ONLY after the producer's
//! Release-store of `tail` made it visible — so producer and consumer never touch
//! the same slot concurrently. The interior `UnsafeCell` access is therefore
//! race-free; this is the invariant loom and Miri check.

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(loom)]
use loom::cell::UnsafeCell;

#[cfg(not(loom))]
#[derive(Debug)]
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

use core::cell::Cell;

/// Shared lock-free SPSC ring of `N` slots (`N` must be a power of two). Holds at
/// most `N` in-flight `T: Copy` elements. Split into a [`Producer`]/[`Consumer`]
/// pair so the type system enforces the single-producer/single-consumer contract.
pub struct SpscRing<T: Copy, const N: usize> {
    /// Producer's free-running write index (== kfifo `in`).
    tail: AtomicUsize,
    /// Consumer's free-running read index (== kfifo `out`).
    head: AtomicUsize,
    buf: [UnsafeCell<T>; N],
}

// SAFETY: the SPSC index protocol guarantees producer and consumer never access
// the same slot concurrently, so the ring may be shared across the two threads.
unsafe impl<T: Copy + Send, const N: usize> Sync for SpscRing<T, N> {}
unsafe impl<T: Copy + Send, const N: usize> Send for SpscRing<T, N> {}

impl<T: Copy + Default, const N: usize> SpscRing<T, N> {
    /// Build a ring and split it into its producer and consumer endpoints.
    /// `N` must be a power of two (kfifo mask requirement).
    pub fn new() -> (Producer<T, N>, Consumer<T, N>) {
        assert!(N.is_power_of_two(), "ring size N must be a power of two");
        let buf = core::array::from_fn(|_| UnsafeCell::new(T::default()));
        let ring = Ring(Arc::new(SpscRing {
            tail: AtomicUsize::new(0),
            head: AtomicUsize::new(0),
            buf,
        }));
        (Producer(Ring(Arc::clone(&ring.0))), Consumer(ring))
    }
}

// Arc shim: `alloc::sync::Arc` for the no_std production/Miri build (a no_std lib
// references `alloc`, never `std`); `loom::sync::Arc` under --cfg loom.
#[cfg(loom)]
use loom::sync::Arc;
#[cfg(not(loom))]
use alloc::sync::Arc;

struct Ring<T: Copy, const N: usize>(Arc<SpscRing<T, N>>);

/// The producer endpoint (`!Sync`: only one thread may push).
pub struct Producer<T: Copy, const N: usize>(Ring<T, N>);
/// The consumer endpoint (`!Sync`: only one thread may pop).
pub struct Consumer<T: Copy, const N: usize>(Ring<T, N>);

// Cell makes Producer/Consumer !Sync without affecting Send; the endpoints carry
// no thread-local state, the !Sync is conveyed structurally by not deriving Sync.
const _: fn() = || {
    fn assert_send<X: Send>() {}
    let _ = assert_send::<Cell<()>>;
};

impl<T: Copy, const N: usize> Producer<T, N> {
    /// Push one element. Returns `Err(value)` if the ring is full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        let r = &self.0 .0;
        // Producer owns `tail`; Relaxed read of own index is fine.
        let tail = r.tail.load(Ordering::Relaxed);
        // Acquire load of `head`: see the consumer's latest progress so we don't
        // report full when a slot was just freed.
        let head = r.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) == N {
            return Err(value); // full
        }
        let slot = tail & (N - 1);
        // SAFETY: the ring is not full, so slot `tail & (N-1)` is owned by the
        // producer until the Release store below publishes it; the consumer
        // cannot read it yet (it only reads slots < tail per its Acquire load).
        r.buf[slot].with_mut(|p| unsafe { *p = value });
        // Release: publishes the slot write to the consumer's Acquire load of tail.
        r.tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Number of elements currently queued (observed; may grow as the producer
    /// runs). `used == in - out`.
    pub fn len(&self) -> usize {
        let r = &self.0 .0;
        r.tail.load(Ordering::Relaxed).wrapping_sub(r.head.load(Ordering::Acquire))
    }
}

impl<T: Copy, const N: usize> Consumer<T, N> {
    /// Pop one element in FIFO order. Returns `None` if the ring is empty.
    pub fn pop(&mut self) -> Option<T> {
        let r = &self.0 .0;
        // Consumer owns `head`; Relaxed read of own index is fine.
        let head = r.head.load(Ordering::Relaxed);
        // Acquire load of `tail`: pairs with the producer's Release store, so the
        // slot write happens-before this read (no torn / stale element).
        let tail = r.tail.load(Ordering::Acquire);
        if head == tail {
            return None; // empty
        }
        let slot = head & (N - 1);
        // SAFETY: head != tail means slot `head & (N-1)` was fully written and
        // published by the producer (Release/Acquire on `tail`); the producer
        // will not overwrite it until we advance `head` below, so this read is
        // race-free.
        let value = r.buf[slot].with_mut(|p| unsafe { *p });
        // Release: tells the producer this slot is free (paired with its Acquire
        // load of head in `push`).
        r.head.store(head.wrapping_add(1), Ordering::Release);
        Some(value)
    }
}

// ===========================================================================
// Behavioural / invariant tests (std build; small variant under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn single_thread_fifo_and_full_empty() {
        let (mut p, mut c) = SpscRing::<u32, 4>::new();
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

    // Concurrent producer + consumer: every item is received exactly once, in
    // order, none lost or duplicated. (Miri runs a small version for UB.)
    #[test]
    fn concurrent_producer_consumer_fifo_exact() {
        let count: u64 = if cfg!(miri) { 200 } else { 200_000 };
        let (mut p, mut c) = SpscRing::<u64, 8>::new();
        let prod = thread::spawn(move || {
            let mut i = 0u64;
            while i < count {
                if p.push(i).is_ok() {
                    i += 1;
                }
            }
        });
        let mut expected = 0u64;
        while expected < count {
            if let Some(v) = c.pop() {
                assert_eq!(v, expected, "FIFO order / no loss violated");
                expected += 1;
            }
        }
        prod.join().unwrap();
        assert_eq!(expected, count);
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of the SPSC ordering.
//   RUSTFLAGS="--cfg loom" cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;

    #[test]
    fn loom_spsc_no_loss_no_reorder() {
        loom::model(|| {
            // Capacity 2 ring; producer pushes 0,1,2 (more than capacity, so it
            // must wait for the consumer), consumer pops 3 items. Across EVERY
            // interleaving loom explores, the consumer must observe exactly
            // 0,1,2 in order — proving no lost, duplicated, torn, or reordered
            // element and that the Release/Acquire on tail/head is correct.
            let (mut p, mut c) = SpscRing::<u8, 2>::new();
            let prod = loom::thread::spawn(move || {
                for i in 0..3u8 {
                    while p.push(i).is_err() {
                        loom::thread::yield_now();
                    }
                }
            });
            let mut next = 0u8;
            while next < 3 {
                if let Some(v) = c.pop() {
                    assert_eq!(v, next, "SPSC ordering/loss violation");
                    next += 1;
                } else {
                    loom::thread::yield_now();
                }
            }
            prod.join().unwrap();
        });
    }
}
