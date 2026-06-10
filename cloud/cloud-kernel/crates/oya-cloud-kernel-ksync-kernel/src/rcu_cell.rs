//! # ksync_rcu_cell — a verified RCU-protected pointer cell (Phase 2, hard tier)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). A capstone slice
//! porting **RCU** (Read-Copy-Update) — *the* signature Linux read-mostly mechanism —
//! reduced to its essential, verifiable core: a single published pointer with
//! wait-free reads and grace-period reclamation.
//!
//! ## Kernel provenance
//! `include/linux/rcupdate.h` / `kernel/rcu/`: readers run `rcu_read_lock()` /
//! `rcu_read_unlock()` (cheap, never block writers) and dereference shared pointers
//! via `rcu_dereference()`; a writer publishes a new version with
//! `rcu_assign_pointer()` (a Release store) and then calls `synchronize_rcu()` — which
//! blocks until every pre-existing read-side critical section has finished (a *grace
//! period*) — before freeing the old version. This crate ports exactly that shape for
//! one `Box<T>`-sized datum: read / copy-update / synchronize / free.
//!
//! ## How the grace period is implemented (and why it is sound)
//! A global epoch counter; each reader announces the current epoch when it pins
//! (`rcu_read_lock`) and `UNPINNED` when it unpins. `update` swaps in the new pointer
//! (Release), then `synchronize` bumps the epoch to `e+1` and **waits until no reader
//! is still pinned at an epoch `<= e`** — i.e. until every reader that might hold the
//! old pointer has exited. Only then is the old `Box` freed. A reader that pins *after*
//! the bump announces `> e` and, because its `rcu_dereference` Acquire-load pairs with
//! the writer's swap Release, observes the *new* pointer — so it never holds the old
//! one. A reader that holds the old pointer is pinned at `<= e`, so `synchronize`
//! waits for it: **no use-after-free**.
//!
//! ## The ordering crux (StoreLoad, same shape as ebr_stack)
//! A reader does `announce(epoch); deref(ptr)` and `synchronize` does
//! `bump(epoch); scan(readers)`. That is the Dekker / StoreLoad pattern: with only
//! Release/Acquire a reader could load the *old* pointer while `synchronize`'s scan
//! fails to observe its announce, and free the pointer under it. The fix is a
//! **`SeqCst` fence on each side** — one after the reader's announce-store (before its
//! deref) and one after `synchronize`'s epoch bump (before its scan). The two fences
//! are ordered in the single total order of `SeqCst` fences, so a reader holding the
//! old pointer is always observed by the scan (and waited for). loom checks the
//! grace-period protocol; **Miri checks the real frees for use-after-free**.
//!
//! ## Scope (documented, not faked)
//! Single updater (`synchronize_rcu` callers serialize in the kernel too; multiple
//! concurrent updaters here would need an external writer lock — out of scope, would
//! compose with `ksync_spinlock`). A bounded number of concurrent readers (`SLOTS`,
//! recycled). `synchronize` *spins* (Phase 2 has no scheduler to block on). The epoch
//! is a free-running `usize` that does not wrap in any realistic run. A verified RCU
//! core (one published pointer), not the full kernel RCU (no call_rcu/softirq/
//! per-CPU/expedited machinery). Not Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};

#[cfg(not(loom))]
use alloc::boxed::Box;
#[cfg(loom)]
use std::boxed::Box;

use core::marker::PhantomData;
use core::ops::Deref;

/// Maximum number of concurrently-registered readers (slots recycled on drop).
const SLOTS: usize = 2;
/// Sentinel: this reader slot is not in a read-side critical section.
const UNPINNED: usize = usize::MAX;

/// An RCU-protected cell holding one `T`. Readers obtain a lightweight [`Reader`]
/// handle and `read()` a [`ReadGuard`] (wait-free `rcu_dereference`); the single
/// updater calls [`update`](RcuCell::update) (publish + grace period + reclaim).
pub struct RcuCell<T> {
    /// The currently-published `Box<T>` (as a raw pointer). `rcu_assign_pointer` swaps
    /// it (Release); `rcu_dereference` loads it (Acquire).
    ptr: AtomicPtr<T>,
    /// Global grace-period epoch.
    global_epoch: AtomicUsize,
    /// Per-reader announced epoch (or `UNPINNED`).
    readers: [AtomicUsize; SLOTS],
    /// Bitset of in-use reader slots.
    reader_mask: AtomicUsize,
}

// SAFETY: `ptr` is published via Release swap and read via Acquire load only while the
// reader is epoch-pinned, which forces `synchronize` to wait for that reader before
// freeing the version it may hold — so no read ever races a free. Readers expose `&T`
// from several threads at once (needs `T: Sync`); the updater moves `T` in and frees
// old `T` (needs `T: Send`).
unsafe impl<T: Send + Sync> Sync for RcuCell<T> {}
unsafe impl<T: Send> Send for RcuCell<T> {}

impl<T> RcuCell<T> {
    /// Create a cell initially publishing `value`.
    pub fn new(value: T) -> Self {
        Self {
            ptr: AtomicPtr::new(Box::into_raw(Box::new(value))),
            global_epoch: AtomicUsize::new(0),
            readers: core::array::from_fn(|_| AtomicUsize::new(UNPINNED)),
            reader_mask: AtomicUsize::new(0),
        }
    }

    /// Register a reader handle (claims a slot). Panics if more than `SLOTS` readers
    /// are concurrently registered. Each handle is used by one thread.
    pub fn reader(&self) -> Reader<'_, T> {
        loop {
            let mask = self.reader_mask.load(Ordering::Acquire);
            let idx = (0..SLOTS)
                .find(|&i| mask & (1 << i) == 0)
                .expect("ksync_rcu_cell: more than SLOTS concurrently-registered readers");
            if self
                .reader_mask
                .compare_exchange(mask, mask | (1 << idx), Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Reader {
                    cell: self,
                    idx,
                    _not_sync: PhantomData,
                };
            }
        }
    }

    /// Publish `value` as the new version, wait a grace period, then free the old
    /// version (read-copy-**update**).
    ///
    /// # Deadlock warning (the kernel's "no `synchronize_rcu` inside `rcu_read_lock`")
    /// `update` must NOT be called by a thread that is **currently holding a
    /// [`ReadGuard`]** (i.e. is itself pinned): `synchronize` spins until every reader
    /// pinned at the old epoch unpins, and the calling thread's own guard can never
    /// unpin while it is blocked here — a self-deadlock. This mirrors the kernel rule
    /// that `synchronize_rcu()` may not be called from within an RCU read-side
    /// critical section. (A liveness hazard, never a memory-safety bug.)
    ///
    /// # Updaters
    /// Concurrent updaters do not corrupt memory: `swap` is an atomic RMW, so each
    /// concurrent `update` receives a *distinct* displaced pointer and reclaims exactly
    /// that one (no double-free), running its own independent grace period. But each
    /// pays a full `synchronize`, so serializing updaters (a single writer, or an
    /// external lock) is the efficient, recommended usage — as the kernel drives
    /// `synchronize_rcu`.
    pub fn update(&self, value: T) {
        let new = Box::into_raw(Box::new(value));
        // rcu_assign_pointer: publish the new version (Release pairs with a reader's
        // Acquire rcu_dereference).
        let old = self.ptr.swap(new, Ordering::Release);
        // synchronize_rcu: wait until no reader can still hold `old`.
        self.synchronize();
        // SAFETY: the grace period guarantees every reader that could have loaded
        // `old` has exited its read-side critical section, so no one references it.
        // The atomic `swap` makes us the UNIQUE reclaimer of this displaced version
        // (each concurrent update gets a distinct `old`), so freeing it (and dropping
        // its T) here happens exactly once.
        drop(unsafe { Box::from_raw(old) });
    }

    /// Wait for a grace period: every reader pinned at the pre-bump epoch must exit.
    fn synchronize(&self) {
        // Bump the epoch: readers pinned at `<= e` are the ones that might hold the
        // just-replaced version; new pins read `e+1` and observe the new pointer.
        let e = self.global_epoch.fetch_add(1, Ordering::AcqRel);
        // StoreLoad barrier pairing with each reader's pin fence: ensures a reader
        // holding the old pointer (pinned at `<= e`) is observed by the scan below.
        fence(Ordering::SeqCst);
        loop {
            let mut waiting = false;
            for slot in &self.readers {
                let r = slot.load(Ordering::Acquire);
                // A reader still pinned at an epoch `<= e` may hold the old version, so
                // we must wait for it to unpin. (Epochs are monotone and do not wrap in
                // any realistic run, so a plain `r <= e` comparison is correct.)
                if r != UNPINNED && r <= e {
                    waiting = true;
                    break;
                }
            }
            if !waiting {
                return; // grace period elapsed: no reader can hold the old version
            }
            spin_hint();
        }
    }

    /// Whether the cell currently has no published version (only true mid-`Drop`-style
    /// teardown; always `false` in normal use).
    pub fn is_empty(&self) -> bool {
        self.ptr.load(Ordering::Acquire).is_null()
    }
}

impl<T> Drop for RcuCell<T> {
    fn drop(&mut self) {
        // Exclusive access: free the currently-published version (drops its T). No
        // reader can exist (they borrow &self).
        let p = self.ptr.load(Ordering::Relaxed);
        if !p.is_null() {
            // SAFETY: `p` is the live published Box we own; exclusive at drop.
            drop(unsafe { Box::from_raw(p) });
        }
    }
}

/// A per-thread reader handle owning one RCU reader slot.
pub struct Reader<'a, T> {
    cell: &'a RcuCell<T>,
    idx: usize,
    // Make `Reader` `!Sync`: one thread per handle (the slot + read-side state are
    // single-owner). `read(&mut self)` already enforces one live guard at a time.
    _not_sync: PhantomData<core::cell::Cell<()>>,
}

// SAFETY: a Reader may move to another thread (Send when T allows sharing the cell);
// it is !Sync (above) so it is never shared by reference across threads.
unsafe impl<T: Send + Sync> Send for Reader<'_, T> {}

impl<T> Reader<'_, T> {
    /// Enter an RCU read-side critical section and dereference the current version.
    /// `&mut self` ensures one live guard per reader at a time. Wait-free.
    pub fn read(&mut self) -> ReadGuard<'_, T> {
        // rcu_read_lock: announce the current epoch (Release), then a SeqCst fence so a
        // concurrent `synchronize` either observes this pin or we observe its bump (and
        // thus dereference the new pointer) — the StoreLoad barrier.
        let e = self.cell.global_epoch.load(Ordering::Acquire);
        self.cell.readers[self.idx].store(e, Ordering::Release);
        fence(Ordering::SeqCst);
        // rcu_dereference: Acquire-load the published pointer (pairs with update's swap
        // Release). Protected: if this is the old version, we pinned at `<= e` so
        // `synchronize` will wait for us before freeing it.
        let ptr = self.cell.ptr.load(Ordering::Acquire);
        ReadGuard {
            cell: self.cell,
            idx: self.idx,
            ptr,
        }
    }
}

impl<T> Drop for Reader<'_, T> {
    fn drop(&mut self) {
        // Ensure unpinned (defensive; a live guard borrows &mut self, so normally we
        // are already unpinned here) and release the slot.
        self.cell.readers[self.idx].store(UNPINNED, Ordering::Release);
        self.cell
            .reader_mask
            .fetch_and(!(1 << self.idx), Ordering::Release);
    }
}

/// An RCU read guard: derefs to `&T` (the version observed at `rcu_dereference`).
/// Dropping it ends the read-side critical section (`rcu_read_unlock`).
pub struct ReadGuard<'a, T> {
    cell: &'a RcuCell<T>,
    idx: usize,
    ptr: *const T,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: we are epoch-pinned (rcu_read_lock), so the version `ptr` points to
        // cannot be freed until we unpin — `synchronize` waits for our slot. The
        // pointer was published with a Release swap observed by our Acquire load, so
        // the pointee is fully initialised. Hence `&*ptr` is valid for our lifetime.
        unsafe { &*self.ptr }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // rcu_read_unlock: leave the critical section, letting a waiting grace period
        // complete. Release so a reclaiming `synchronize`'s Acquire scan sees it.
        self.cell.readers[self.idx].store(UNPINNED, Ordering::Release);
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
    use std::sync::atomic::{AtomicBool, Ordering as O};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_read_update() {
        let cell = RcuCell::new(10u64);
        {
            let mut r = cell.reader();
            assert_eq!(*r.read(), 10);
        }
        cell.update(20);
        {
            let mut r = cell.reader();
            assert_eq!(*r.read(), 20);
        }
        assert!(!cell.is_empty());
    }

    // Concurrent readers + a single updater doing many updates with REAL reclamation.
    // A reader's guard always derefs a valid published value (never a freed version),
    // and the value is monotone (each update increments). Miri proves no use-after-free
    // on the reclaimed Boxes.
    #[test]
    fn concurrent_readers_single_updater_no_uaf() {
        let updates: u64 = if cfg!(miri) { 8 } else { 5_000 };
        // Up to SLOTS concurrent reader threads (each holds one slot until it stops);
        // the updater is the main thread and needs no reader slot.
        let nreaders = SLOTS;
        let cell = Arc::new(RcuCell::new(0u64));
        let stop = Arc::new(AtomicBool::new(false));

        let mut hs = Vec::new();
        for _ in 0..nreaders {
            let cell = Arc::clone(&cell);
            let stop = Arc::clone(&stop);
            hs.push(thread::spawn(move || {
                let mut r = cell.reader();
                let mut last = 0u64;
                while !stop.load(O::Acquire) {
                    let g = r.read();
                    let v = *g; // deref the published version — must be valid, not freed
                    assert!(v >= last, "version went backwards: {v} < {last}");
                    last = v;
                }
            }));
        }
        // Single updater.
        for i in 1..=updates {
            cell.update(i);
        }
        stop.store(true, O::Release);
        for h in hs {
            h.join().unwrap();
        }
        let mut r = cell.reader();
        assert_eq!(*r.read(), updates);
    }
}

// ===========================================================================
// loom model — the grace-period protocol: a reader holding the old version is never
// freed under it (synchronize waits), and synchronize terminates.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// Coverage note: `loom_reader_holds_old_version_across_update` establishes the
// deterministic "reader holds the old version while the updater synchronizes" case
// (pinning before the spawn); the announce-vs-bump StoreLoad race is explored by
// `loom_reader_after_update_sees_new_version`, where the reader pins concurrently with
// the update and loom enumerates every interleaving (asserting it observes a valid
// version, never freed/torn). Together they cover both the wait and the race.
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_reader_holds_old_version_across_update() {
        loom::model(|| {
            // Main pins and dereferences the OLD version (0) BEFORE spawning the
            // updater — establishing that main is an in-grace-period reader holding the
            // old Box. The updater publishes 1 and calls synchronize, which MUST wait
            // for main's read-side section: while main holds the guard, *guard == 0 and
            // the old Box is not freed (Miri would flag a UAF if it were). Once main
            // drops the guard, synchronize completes and frees the old Box.
            let cell = Arc::new(RcuCell::new(0u32));
            let mut r = cell.reader();
            let g = r.read();
            assert_eq!(*g, 0, "reader holds the old version");
            let c2 = Arc::clone(&cell);
            let updater = loom::thread::spawn(move || c2.update(1));
            // Still holding the guard: the old version must remain valid (== 0).
            assert_eq!(*g, 0, "old version freed while reader holds it (UAF)!");
            drop(g); // rcu_read_unlock -> lets synchronize finish
            updater.join().unwrap();
            // New version is published; a fresh read sees it.
            let mut r2 = cell.reader();
            assert_eq!(*r2.read(), 1);
        });
    }

    #[test]
    fn loom_reader_after_update_sees_new_version() {
        loom::model(|| {
            // A reader that pins concurrently with / after an update observes either the
            // old or the new version (a valid published value), never freed memory.
            let cell = Arc::new(RcuCell::new(0u32));
            let c2 = Arc::clone(&cell);
            let updater = loom::thread::spawn(move || c2.update(1));
            let mut r = cell.reader();
            let v = *r.read();
            assert!(v == 0 || v == 1, "reader saw an invalid version {v}");
            updater.join().unwrap();
            assert_eq!(*cell.reader().read(), 1);
        });
    }
}
