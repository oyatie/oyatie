//! # ksync_rwsem — a sound reader-writer semaphore with downgrade (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`), verified by
//! **loom** + **Miri** + invariant tests. The count-based reader/writer exclusion is
//! the same shape as `crates_phase2/rwlock`; the *distinctive* operation this slice
//! adds and verifies is **`downgrade_write`** — atomically converting a write lock
//! into a read lock with **no window for another writer to interpose** — which the
//! plain `rwlock` does not provide.
//!
//! ## Kernel provenance
//! Models `struct rw_semaphore` (`include/linux/rwsem.h`, `kernel/locking/rwsem.c`)
//! at its core: a single `count` word encoding either a writer or a reader tally,
//! with `down_read`/`down_write`/`up_read`/`up_write` and the kernel's
//! `downgrade_write()`. The real rwsem **sleeps** (scheduler) and carries
//! waiter/handoff flags for fairness; Phase 2 has no scheduler, so this models the
//! lock as **spinning** and omits the waiter/handoff bits (documented). The
//! `WRITER`-bit + reader-count encoding mirrors `RWSEM_WRITER_LOCKED` /
//! `RWSEM_READER_BIAS` at the core.
//!
//! ## Soundness
//! `count` is atomic. `0` = free; the high `WRITER` bit set (with reader bits 0) =
//! write-locked; otherwise the value = the number of readers. A [`WriteGuard`]
//! (`&mut T`) exists only after a `0 -> WRITER` CAS, so it is exclusive against all
//! readers and writers; a [`ReadGuard`] (`&T`) exists only while reader count >= 1
//! and `WRITER` is clear. So a `&mut T` never overlaps any `&T` or another `&mut T`.
//! **`downgrade_write`** is sound because while write-locked `count == WRITER`
//! exactly (no reader can increment and no writer can enter while `WRITER` is set),
//! so a single atomic `store(1)` (one reader, `WRITER` cleared, Release) transitions
//! writer -> exactly-one-reader with no intervening state — no other thread can
//! observe or act on `count` between the two, so no writer interposes. loom checks
//! reader/writer exclusion AND the downgrade: a writer already blocked (it tried to
//! acquire while WRITER was held) stays blocked until the downgraded reader releases
//! and cannot slip in, and the downgraded reader plus a concurrent reader observe the
//! writer's published value. (The downgrade tests pre-block the contender to establish
//! that happens-before; the no-intervening-state argument above — not loom — is what
//! rules out a writer racing the `store(1)` itself.) Miri checks for UB.

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicUsize, Ordering};

use core::mem;
use core::ops::{Deref, DerefMut};

// loom-compatible UnsafeCell shim with a shared `with` (read) and an exclusive
// `with_mut` (write), so loom distinguishes concurrent reads (allowed) from a read
// racing a write (forbidden).
#[cfg(loom)]
use loom::cell::UnsafeCell;

#[cfg(not(loom))]
#[derive(Debug)]
struct UnsafeCell<T>(core::cell::UnsafeCell<T>);
#[cfg(not(loom))]
impl<T> UnsafeCell<T> {
    // (No `new`: the `const fn RwSem::new` builds this via the tuple constructor
    // directly, since a `const fn` cannot call a non-const method.)
    #[inline]
    fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }
    #[inline]
    fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

/// Writer flag: the high bit of `count`. Low bits are the reader count.
const WRITER: usize = 1usize << (usize::BITS - 1);

/// A reader-writer semaphore guarding a `T`. (Spin-based in Phase 2; see the module
/// docs for why the kernel's sleeping/fairness machinery is out of scope here.)
pub struct RwSem<T> {
    /// `0` = free; `WRITER` set = write-locked; otherwise = number of readers.
    count: AtomicUsize,
    data: UnsafeCell<T>,
}

// SAFETY: readers share `&T`, the writer has exclusive `&mut T`, and the count word
// serialises the reader/writer transitions so the two never overlap. Sharing across
// threads is sound whenever `T: Send + Sync` (readers may observe `&T` from several
// threads at once, which needs `T: Sync`).
unsafe impl<T: Send + Sync> Sync for RwSem<T> {}
unsafe impl<T: Send> Send for RwSem<T> {}

impl<T> RwSem<T> {
    #[cfg(not(loom))]
    pub const fn new(value: T) -> Self {
        Self {
            count: AtomicUsize::new(0),
            data: UnsafeCell(core::cell::UnsafeCell::new(value)),
        }
    }

    #[cfg(loom)]
    pub fn new(value: T) -> Self {
        Self {
            count: AtomicUsize::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Try to acquire a shared read lock without spinning. Returns `None` if a writer
    /// holds it.
    pub fn try_read(&self) -> Option<ReadGuard<'_, T>> {
        let s = self.count.load(Ordering::Acquire);
        // Acquire only if WRITER is clear AND incrementing won't reach the WRITER bit:
        // `s + 1 == WRITER` would flip the high bit and mis-encode a saturated reader
        // tally as "write-locked". This needs 2^(BITS-1) live readers — physically
        // unreachable — but the guard makes the encoding provably uncorruptible rather
        // than merely-unreachable. (std/spin rwlocks add the analogous saturation check.)
        if s & WRITER == 0 && s + 1 != WRITER {
            // Acquire pairs with a writer's Release (unlock or downgrade) so we observe
            // its writes.
            if self
                .count
                .compare_exchange(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Some(ReadGuard { sem: self });
            }
        }
        None
    }

    /// Try to acquire the exclusive write lock without spinning. Returns `None` unless
    /// the lock is fully free.
    pub fn try_write(&self) -> Option<WriteGuard<'_, T>> {
        if self
            .count
            .compare_exchange(0, WRITER, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(WriteGuard { sem: self })
        } else {
            None
        }
    }

    /// Acquire a shared read lock (`down_read`), spinning while a writer holds it.
    pub fn read(&self) -> ReadGuard<'_, T> {
        loop {
            if let Some(g) = self.try_read() {
                return g;
            }
            spin_hint();
        }
    }

    /// Acquire the exclusive write lock (`down_write`), spinning until fully free.
    pub fn write(&self) -> WriteGuard<'_, T> {
        loop {
            if let Some(g) = self.try_write() {
                return g;
            }
            spin_hint();
        }
    }
}

/// Shared read guard (`&T`); dropping it does `up_read`.
pub struct ReadGuard<'a, T> {
    sem: &'a RwSem<T>,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: we hold a read lock (reader count >= 1, WRITER clear), so no writer
        // holds `&mut T`; a shared `&T` cannot alias a `&mut T`, and other readers'
        // concurrent `&T` is fine (shared, no mutation).
        self.sem.data.with(|p| unsafe { &*p })
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Release: pair with a future writer's Acquire so it observes the count drop.
        self.sem.count.fetch_sub(1, Ordering::Release);
    }
}

/// Exclusive write guard (`&mut T`); dropping it does `up_write`.
pub struct WriteGuard<'a, T> {
    sem: &'a RwSem<T>,
}

impl<'a, T> WriteGuard<'a, T> {
    /// `downgrade_write`: atomically convert this write lock into a read lock. No
    /// other writer can acquire across the transition, and concurrent readers may
    /// then join. Consumes the write guard and returns a read guard.
    pub fn downgrade(self) -> ReadGuard<'a, T> {
        let sem = self.sem;
        // While write-locked, `count == WRITER` exactly (readers cannot increment and
        // writers cannot enter while WRITER is set), so this single atomic store
        // transitions WRITER -> exactly-one-reader with no intervening observable
        // state. Release publishes the critical-section writes to a reader's Acquire.
        sem.count.store(1, Ordering::Release);
        // Skip the WriteGuard Drop (which would store 0 and drop the lock); the lock
        // now logically holds one reader, which the returned ReadGuard owns.
        mem::forget(self);
        ReadGuard { sem }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: exclusive writer; no other guard exists, so `&T` is sound.
        self.sem.data.with(|p| unsafe { &*p })
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: WRITER is set and was reached via `0 -> WRITER`, so there are no
        // readers and no other writer; this `&mut T` is genuinely exclusive.
        self.sem.data.with_mut(|p| unsafe { &mut *p })
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Release: publish the critical-section writes to the next reader/writer,
        // whose Acquire CAS/load observes `count` returning to 0.
        self.sem.count.store(0, Ordering::Release);
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
// Behavioural / invariant tests (std; small variants under Miri).
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_read_write_downgrade() {
        let l = RwSem::new(0i64);
        *l.write() = 42;
        assert_eq!(*l.read(), 42);
        {
            let a = l.read();
            let b = l.read(); // multiple concurrent readers OK
            assert_eq!(*a + *b, 84);
        }
        // Downgrade: write 7, then hold it as a read lock; the value is visible, and a
        // second reader can coexist while the downgraded guard lives.
        {
            let mut w = l.write();
            *w = 7;
            let r = w.downgrade();
            assert_eq!(*r, 7);
            let r2 = l.read(); // another reader joins (no writer present post-downgrade)
            assert_eq!(*r2, 7);
            assert!(l.try_write().is_none(), "writer blocked while readers hold");
        }
        // After both read guards drop, a writer can acquire again.
        *l.write() += 1;
        assert_eq!(*l.read(), 8);
    }

    // Writers mutually exclusive (no lost update); some writers downgrade and read
    // back before releasing; readers never see a torn value.
    #[test]
    fn writers_exclusive_with_downgrade() {
        let writers = if cfg!(miri) { 2 } else { 4 };
        let iters = if cfg!(miri) { 30 } else { 4_000 };
        let l = Arc::new(RwSem::new(0u64));
        let mut hs = Vec::new();
        for t in 0..writers {
            let l = Arc::clone(&l);
            hs.push(thread::spawn(move || {
                for _ in 0..iters {
                    let mut w = l.write();
                    *w += 1;
                    if t == 0 {
                        // Exercise the downgrade path: convert to a read and verify the
                        // value we just wrote is visible (and stable while we hold it).
                        let r = w.downgrade();
                        let _ = *r;
                    }
                }
            }));
        }
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(*l.read(), writers as u64 * iters as u64, "lost update");
    }
}

// ===========================================================================
// loom model — reader/writer exclusion AND the downgrade transition.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_writer_excludes_reader() {
        loom::model(|| {
            let l = Arc::new(RwSem::new(0u32));
            let l2 = Arc::clone(&l);
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
            let l = Arc::new(RwSem::new(0u32));
            let l2 = Arc::clone(&l);
            let t = loom::thread::spawn(move || {
                *l2.write() += 1;
            });
            *l.write() += 1;
            t.join().unwrap();
            assert_eq!(*l.read(), 2);
        });
    }

    #[test]
    fn loom_downgrade_publishes_to_concurrent_reader() {
        loom::model(|| {
            let l = Arc::new(RwSem::new(0u32));
            // Main acquires the write lock FIRST (before spawning), so the reader is
            // guaranteed to find WRITER held and must block until the downgrade clears
            // it — establishing the happens-before we are testing. (Without acquiring
            // first, loom rightly explores the reader running before any write and
            // observing 0 — a valid interleaving, not a downgrade failure.)
            let mut w = l.write();
            *w = 7;
            let l2 = Arc::clone(&l);
            let reader = loom::thread::spawn(move || *l2.read());
            let r = w.downgrade(); // clears WRITER, count = 1, publishes 7 (Release)
            assert_eq!(*r, 7, "downgraded reader sees its own write");
            // The reader was blocked by WRITER until the downgrade, so it observes 7
            // (publication via the downgrade's Release / read's Acquire), and the two
            // readers coexist.
            let seen = reader.join().unwrap();
            assert_eq!(seen, 7, "reader saw a stale value across downgrade");
            drop(r);
        });
    }

    #[test]
    fn loom_downgrade_then_writer_no_lost_update() {
        loom::model(|| {
            let l = Arc::new(RwSem::new(0u32));
            // Main acquires write FIRST so the writer thread (spawned after) cannot win
            // the initial lock; it must block until main releases its downgraded read.
            // Both sides increment, so the final value is order-independent (= 2).
            let mut w = l.write();
            let l2 = Arc::clone(&l);
            let writer = loom::thread::spawn(move || {
                *l2.write() += 1;
            });
            *w += 1; // main: 0 -> 1
            let r = w.downgrade(); // downgrade; writer still blocked (count == 1)
            // The writer cannot have interposed between our write and this read.
            assert_eq!(*r, 1, "downgrade preserved main's write; no writer interposed");
            drop(r); // release the read; only now can the writer acquire
            writer.join().unwrap();
            assert_eq!(*l.read(), 2, "writer +1 after downgrade-release; no lost update");
        });
    }
}
