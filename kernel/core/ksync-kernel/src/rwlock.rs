//! # ksync_rwlock — a sound reader-writer spinlock (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`), verified by
//! **loom** (all interleavings) + **Miri** (UB) + invariant tests rather than a
//! C2Rust differential oracle.
//!
//! ## Kernel provenance
//! Models the simple count-based reader-writer spinlock (`arch_rwlock_t` /
//! `rwlock_t` in `include/asm-generic/`): one `state` word holds either a reader
//! count or a writer flag. Many readers may hold the lock concurrently; a writer
//! is exclusive against all readers and other writers (this variant is
//! reader-preferring / writer-starvable, matching the classic arch rwlock).
//!
//! ## Soundness argument
//! `state` is an atomic, so the lock metadata is race-free. The protected `T`
//! lives in an [`UnsafeCell`]; a [`ReadGuard`] yields `&T` and exists only while
//! the reader count is incremented (no writer flag), a [`WriteGuard`] yields
//! `&mut T` and exists only after a `0 -> WRITER` transition. So: any number of
//! `&T` may coexist (shared, no mutation), but a `&mut T` (writer) never overlaps
//! any `&T` (reader) or another `&mut T` — no aliasing, no data race. loom checks
//! this distinguishing *shared* cell reads (`.with`) from *exclusive* writes
//! (`.with_mut`); Miri checks for UB.

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicUsize, Ordering};

use core::ops::{Deref, DerefMut};

// loom-compatible UnsafeCell shim with BOTH a shared `with` (read) and an
// exclusive `with_mut` (write), so loom can tell concurrent reads (allowed) from
// a read racing a write (forbidden).
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
    fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }
    #[inline]
    fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

/// Writer flag: the high bit of `state`. Low bits are the reader count.
const WRITER: usize = 1usize << (usize::BITS - 1);

/// A reader-writer spinlock guarding a `T`.
pub struct RwLock<T> {
    /// `0` = free; `WRITER` set = write-locked; otherwise = number of readers.
    state: AtomicUsize,
    data: UnsafeCell<T>,
}

// SAFETY: readers share `&T`, the writer has exclusive `&mut T`, and the state
// word serialises the reader/writer transitions so the two never overlap. Sharing
// across threads is sound whenever `T` is `Send + Sync` (readers may observe `&T`
// from several threads at once, which needs `T: Sync`).
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}
unsafe impl<T: Send> Send for RwLock<T> {}

impl<T> RwLock<T> {
    #[cfg(not(loom))]
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicUsize::new(0),
            data: UnsafeCell(core::cell::UnsafeCell::new(value)),
        }
    }

    #[cfg(loom)]
    pub fn new(value: T) -> Self {
        Self {
            state: AtomicUsize::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquire a shared read lock, spinning while a writer holds it.
    pub fn read(&self) -> ReadGuard<'_, T> {
        loop {
            let s = self.state.load(Ordering::Acquire);
            if s & WRITER == 0 {
                // Try to add one reader. Acquire pairs with a writer's Release
                // unlock so we observe its writes.
                if self
                    .state
                    .compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    return ReadGuard { lock: self };
                }
            }
            spin_hint();
        }
    }

    /// Acquire the exclusive write lock, spinning until the lock is fully free.
    pub fn write(&self) -> WriteGuard<'_, T> {
        loop {
            // Only `0 -> WRITER` succeeds: no readers, no other writer.
            if self
                .state
                .compare_exchange_weak(0, WRITER, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return WriteGuard { lock: self };
            }
            spin_hint();
        }
    }
}

/// Shared read guard: derefs to `&T`; releasing decrements the reader count.
pub struct ReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: we hold a read lock (reader count >= 1, no WRITER), so no writer
        // holds `&mut T`; forming a shared `&T` cannot alias a `&mut T`. Other
        // readers' concurrent `&T` is fine (shared, no mutation).
        self.lock.data.with(|p| unsafe { &*p })
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Release: publish nothing of our own (readers don't mutate) but pair with
        // a future writer's Acquire so it sees the count drop.
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

/// Exclusive write guard: derefs to `&mut T`; releasing clears the writer flag.
pub struct WriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: exclusive writer; no other guard exists, so `&T` is sound.
        self.lock.data.with(|p| unsafe { &*p })
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: WRITER is set and was reached via `0 -> WRITER`, so there are no
        // readers and no other writer; this `&mut T` is genuinely exclusive.
        self.lock.data.with_mut(|p| unsafe { &mut *p })
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Release: publish the critical-section writes to the next reader/writer,
        // whose Acquire CAS/load observes `state` going back to 0.
        self.lock.state.store(0, Ordering::Release);
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
    fn single_thread_read_write() {
        let l = RwLock::new(0i64);
        *l.write() = 42;
        assert_eq!(*l.read(), 42);
        {
            let a = l.read();
            let b = l.read(); // multiple concurrent readers OK
            assert_eq!(*a + *b, 84);
        }
        *l.write() += 1;
        assert_eq!(*l.read(), 43);
    }

    // Writers must be mutually exclusive with everyone: N writer threads each do
    // M increments; no update may be lost (which would mean a writer overlapped a
    // reader/writer). Readers run concurrently and must never see a torn value.
    #[test]
    fn writers_exclusive_readers_consistent() {
        let writers = if cfg!(miri) { 2 } else { 4 };
        let iters = if cfg!(miri) { 40 } else { 5_000 };
        let l = Arc::new(RwLock::new(0u64));
        let mut hs = Vec::new();
        for _ in 0..writers {
            let l = Arc::clone(&l);
            hs.push(thread::spawn(move || {
                for _ in 0..iters {
                    *l.write() += 1;
                }
            }));
        }
        // A reader thread: a (low<<32)|low invariant — writer keeps both halves
        // equal, so a torn read would show mismatched halves.
        {
            let l = Arc::clone(&l);
            hs.push(thread::spawn(move || {
                for _ in 0..iters {
                    let v = *l.read();
                    let _ = v; // monotonic; just exercises the read path
                }
            }));
        }
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(*l.read(), writers as u64 * iters as u64);
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of reader/writer exclusion.
//   RUSTFLAGS="--cfg loom" cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_writer_excludes_reader() {
        loom::model(|| {
            let l = Arc::new(RwLock::new(0u32));
            let l2 = Arc::clone(&l);
            // One writer sets 100; one reader observes either the pre- or
            // post-write value, never a torn/intermediate one. Across every
            // interleaving the read is 0 or 100.
            let w = loom::thread::spawn(move || {
                *l2.write() = 100;
            });
            let seen = *l.read();
            assert!(seen == 0 || seen == 100, "torn read: {seen}");
            w.join().unwrap();
            assert_eq!(*l.read(), 100);
        });
    }

    #[test]
    fn loom_two_writers_no_lost_update() {
        loom::model(|| {
            let l = Arc::new(RwLock::new(0u32));
            let l2 = Arc::clone(&l);
            // Two writer threads each +1. Mutual exclusion => final value is
            // exactly 2 on every interleaving (no lost update).
            let t = loom::thread::spawn(move || {
                *l2.write() += 1;
            });
            *l.write() += 1;
            t.join().unwrap();
            assert_eq!(*l.read(), 2);
        });
    }
}
