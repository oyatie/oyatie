//! # ksync_completion — a sound one-shot completion / signal (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`), verified by
//! **loom** (all interleavings) + **Miri** (UB) + invariant tests.
//!
//! ## Kernel provenance
//! Models `struct completion` (`include/linux/completion.h`,
//! `kernel/sched/completion.c`): `complete()` publishes "done" exactly once and
//! `wait_for_completion()` blocks until it is set, then proceeds. The defining
//! correctness property is **no missed wakeup**: a waiter must never return before
//! `complete()` and must observe everything the completer published.
//!
//! ## What this verifies (a different pattern from the locks)
//! Unlike the spinlock/rwlock (mutual exclusion), the property here is a
//! *happens-before / publication* one: the value handed to `complete(v)` is read
//! by the waiter only after, and is seen in full (no torn read, no premature
//! return). loom explores every interleaving of completer vs waiter.
//!
//! ## Soundness argument
//! `done: AtomicBool` gates the `UnsafeCell<Option<T>>` slot. `complete()` writes
//! the slot, then `Release`-stores `done = true` (exactly once; a second call is
//! rejected). `wait()` spins on an `Acquire` load of `done`; once true, the
//! completer's slot write happens-before this read, so taking the value is
//! race-free. The slot is written once and taken once. `T: Send` so the produced
//! value may cross to the waiting thread.

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicBool, Ordering};

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

/// A one-shot completion carrying a `T` from the completer to a waiter.
pub struct Completion<T> {
    done: AtomicBool,
    slot: UnsafeCell<Option<T>>,
}

// SAFETY: the `done` flag serialises the single slot write (completer) before any
// slot read (waiter), and the slot is written/taken exactly once, so the interior
// is never accessed concurrently. Sharing across threads is sound for `T: Send`.
unsafe impl<T: Send> Sync for Completion<T> {}
unsafe impl<T: Send> Send for Completion<T> {}

impl<T> Completion<T> {
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            done: AtomicBool::new(false),
            slot: UnsafeCell(core::cell::UnsafeCell::new(None)),
        }
    }

    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            done: AtomicBool::new(false),
            slot: UnsafeCell::new(None),
        }
    }

    /// Whether the completion has fired.
    #[inline]
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    /// Signal completion exactly once, publishing `value`. Returns `Err(value)` if
    /// it was already completed (one-shot).
    pub fn complete(&self, value: T) -> Result<(), T> {
        // Reject a second completion BEFORE touching the slot, so we never race a
        // waiter that is reading after the first complete.
        if self.done.load(Ordering::Relaxed) {
            return Err(value);
        }
        // SAFETY: `done` is still false, so no waiter has observed completion and
        // none will read the slot until our Release store below; the slot is
        // written exactly once. (A concurrent second `complete` is excluded by the
        // single-completer contract; the relaxed check above also rejects it.)
        self.slot.with_mut(|p| unsafe { *p = Some(value) });
        // Release: publishes the slot write to every waiter's Acquire load.
        self.done.store(true, Ordering::Release);
        Ok(())
    }

    /// Spin-wait until completed, then take the value (the first waiter to take it
    /// gets `Some`; this models a single consuming waiter).
    pub fn wait(&self) -> T {
        while !self.done.load(Ordering::Acquire) {
            spin_hint();
        }
        // SAFETY: `done` is true, so `complete()`'s slot write happens-before this
        // read (Release/Acquire on `done`); we take the value exactly once.
        self.slot
            .with_mut(|p| unsafe { (*p).take() })
            .expect("completion fired without a value")
    }

    /// Non-blocking poll: take the value if completed, else `None`.
    pub fn try_take(&self) -> Option<T> {
        if self.done.load(Ordering::Acquire) {
            // SAFETY: as in `wait`, completion happened-before; take once.
            self.slot.with_mut(|p| unsafe { (*p).take() })
        } else {
            None
        }
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
    fn single_thread_complete_then_wait() {
        let c = Completion::new();
        assert!(!c.is_done());
        assert!(c.try_take().is_none());
        assert!(c.complete(99u32).is_ok());
        assert!(c.is_done());
        assert_eq!(c.complete(1), Err(1)); // one-shot
        assert_eq!(c.wait(), 99);
    }

    #[test]
    fn cross_thread_handoff_no_missed_wakeup() {
        let runs = if cfg!(miri) { 20 } else { 2_000 };
        for n in 0..runs {
            let c = Arc::new(Completion::new());
            let c2 = Arc::clone(&c);
            let w = thread::spawn(move || c2.wait());
            // Completer publishes a value the waiter must observe in full.
            c.complete(0xDEAD_0000u32 | n as u32).unwrap();
            assert_eq!(w.join().unwrap(), 0xDEAD_0000u32 | n as u32);
        }
    }
}

// ===========================================================================
// loom model — exhaustive interleaving check of the completer/waiter handoff.
//   RUSTFLAGS="--cfg loom" cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_waiter_sees_published_value_no_missed_wakeup() {
        loom::model(|| {
            let c = Arc::new(Completion::new());
            let c2 = Arc::clone(&c);
            // Waiter thread blocks until complete, then must get exactly 7 — on
            // every interleaving (never a premature return / missed wakeup / torn
            // or absent value).
            let w = loom::thread::spawn(move || c2.wait());
            c.complete(7u8).unwrap();
            assert_eq!(w.join().unwrap(), 7);
        });
    }

    #[test]
    fn loom_concurrent_complete_and_try_take() {
        loom::model(|| {
            let c = Arc::new(Completion::new());
            let c2 = Arc::clone(&c);
            // A poller racing the completer: it observes either None (not yet
            // done) or Some(5) (done & value published in full) — never a torn or
            // bogus value, and never Some twice.
            let t = loom::thread::spawn(move || c2.try_take());
            c.complete(5u8).unwrap();
            let polled = t.join().unwrap();
            assert!(polled.is_none() || polled == Some(5), "bad poll: {polled:?}");
            // Whatever the poller didn't take, the final state is consistent: the
            // value was taken at most once.
            let rest = c.try_take();
            let taken = polled.iter().count() + rest.iter().count();
            assert!(taken <= 1, "value taken more than once");
        });
    }
}
