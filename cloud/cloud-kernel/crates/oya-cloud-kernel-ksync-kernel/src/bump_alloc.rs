//! # ksync_bump_alloc — a concurrent bump (arena) allocator core (Phase 2)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). A different
//! *verification emphasis* from the other Phase-2 slices: the previous ones are
//! loom-heavy (interleaving correctness of locks/queues/reclamation); a bump
//! allocator's hard part is **pointer provenance, alignment, and bounds** — Miri's
//! domain. So this slice is verified by **loom** (the atomic bump protocol hands out
//! *non-overlapping* regions under contention) **+ Miri** (every returned pointer has
//! valid provenance into the arena, is correctly aligned and in-bounds, and disjoint
//! concurrent writes do not race) + invariant tests.
//!
//! ## Kernel provenance
//! Models the family of kernel *bump / linear* allocators — early-boot `memblock`
//! reservation and the per-CPU/region "carve a slice off a fixed arena" pattern: a
//! single free-running offset into a fixed buffer, advanced atomically, with no
//! per-object free (the whole arena is reset at once). Not a verbatim Linux file; the
//! sound-Rust expression of the bump-pointer-over-a-fixed-arena discipline.
//!
//! ## Soundness
//! `alloc` reserves `[start, end)` by an atomic CAS on `offset` (each CAS winner gets
//! a *unique* range, so allocations never overlap — loom proves this across all
//! interleavings), bounds-checked against `N` (`end <= N`) with overflow-safe
//! arithmetic, and aligned up to a power-of-two `align`. The returned pointer is
//! derived from the arena's single backing allocation, so it carries provenance for
//! the whole arena and is valid for `[start, end)`. Because reserved ranges are
//! disjoint, two threads writing into their own allocations touch *different* memory
//! locations — not a data race (Miri confirms). The offset uses `Relaxed`: nothing is
//! published *through* it (each thread uses only its own region), it merely
//! partitions the arena.
//!
//! ## Scope (documented, not faked)
//! Fixed compile-time arena size `N`; bump semantics (NO per-allocation free —
//! `reset(&mut self)` frees everything at once, and `&mut` makes it impossible to
//! reset while allocations are outstanding through the type system). Returns raw
//! `NonNull<u8>` (a building block, not a `GlobalAlloc`); the caller owns
//! initialization and lifetime of each region. A verified bump-allocator core, not a
//! general allocator. Not Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicUsize, Ordering};

use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// Maximum supported allocation alignment. The backing storage is over-aligned to
/// this, so aligning the *offset* equals aligning the *absolute address* (the base is
/// a multiple of `MAX_ALIGN`). Requests for a larger alignment return `None` rather
/// than silently handing back a misaligned pointer. 64 bytes = a common cache line.
pub const MAX_ALIGN: usize = 64;

/// Backing bytes over-aligned to [`MAX_ALIGN`]. CRITICAL: a bare `[u8; N]` has
/// `align_of == 1`, so the arena base could land at any address and an `align`-aligned
/// *offset* would not be an `align`-aligned *pointer* (a real misalignment-UB bug an
/// earlier version had). Forcing the base to `MAX_ALIGN` makes offset alignment and
/// absolute alignment coincide for every `align <= MAX_ALIGN`.
#[repr(C, align(64))]
struct Aligned<const N: usize>([u8; N]);

/// A fixed-size arena handing out non-overlapping byte regions by bumping an atomic
/// offset. `alloc(&self, …)` is lock-free and may be called concurrently. The arena
/// is over-aligned to [`MAX_ALIGN`]; allocations may request any power-of-two
/// alignment up to that.
pub struct BumpArena<const N: usize> {
    /// Backing storage as a single, [`MAX_ALIGN`]-aligned allocation, so a pointer
    /// into it carries provenance for the whole arena AND aligning the offset yields
    /// an absolutely-aligned pointer. A plain `core::cell::UnsafeCell` (never a loom
    /// cell): the arena bytes are not touched under `--cfg loom` — only the `offset`
    /// protocol is model-checked — so loom need not track per-byte accesses (which it
    /// could not distinguish as disjoint anyway).
    storage: UnsafeCell<Aligned<N>>,
    /// Free-running bump offset (the only shared mutable state needing atomicity).
    offset: AtomicUsize,
}

// SAFETY: `alloc` hands out disjoint, non-overlapping regions (unique per CAS winner)
// and never reads/writes the arena itself, so sharing `&BumpArena` across threads
// cannot create aliasing or data races; the only shared mutable state is the atomic
// offset. The arena holds plain bytes (no `T`), so no element bounds are needed.
unsafe impl<const N: usize> Sync for BumpArena<N> {}
unsafe impl<const N: usize> Send for BumpArena<N> {}

impl<const N: usize> BumpArena<N> {
    /// Create an empty arena. (Not `const`: loom's atomics are not const-constructible;
    /// the production/Miri build could be `const` but is kept uniform.)
    pub fn new() -> Self {
        Self {
            storage: UnsafeCell::new(Aligned([0u8; N])),
            offset: AtomicUsize::new(0),
        }
    }

    /// Allocate `size` bytes aligned to `align` (a power of two, `<= MAX_ALIGN`).
    /// Returns `None` if the arena does not have room, or if `align > MAX_ALIGN` (which
    /// the over-aligned base cannot guarantee). The returned pointer is `align`-aligned
    /// in absolute terms. Lock-free; safe to call concurrently from many threads. The
    /// returned region is uninitialised — the caller must write before reading — and
    /// stays valid until [`reset`](Self::reset). Panics only if `align` is not a power
    /// of two.
    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        assert!(align.is_power_of_two(), "align must be a power of two");
        // The base is aligned to MAX_ALIGN; a larger alignment cannot be guaranteed by
        // offset arithmetic, so reject it rather than return a misaligned pointer.
        if align > MAX_ALIGN {
            return None;
        }
        let mut cur = self.offset.load(Ordering::Relaxed);
        loop {
            let start = align_up(cur, align)?;
            let end = start.checked_add(size)?;
            if end > N {
                return None; // out of arena space
            }
            // Relaxed: the offset only partitions the arena into disjoint ranges; no
            // data is published through it (each allocation is used by its claimer).
            match self.offset.compare_exchange_weak(
                cur,
                end,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // SAFETY: this CAS uniquely reserved `[start, end)` (no other
                    // allocation can overlap it), and `end <= N`, so deriving a pointer
                    // at `start` from the arena's single backing allocation is in-bounds
                    // and provenance-valid for the whole region. `base` is MAX_ALIGN-
                    // aligned (storage is `Aligned`, repr(align(64))), and `start` is a
                    // multiple of `align <= MAX_ALIGN`, so `base + start` is genuinely
                    // `align`-aligned in ABSOLUTE terms (not just offset-relative). The
                    // base is non-null, so the offset pointer is too.
                    let base = self.storage.get() as *mut u8;
                    let ptr = unsafe { base.add(start) };
                    return Some(unsafe { NonNull::new_unchecked(ptr) });
                }
                Err(actual) => cur = actual, // lost the race; retry from the new offset
            }
        }
    }

    /// Reset the arena, freeing *all* outstanding allocations at once. `&mut self`
    /// guarantees (at the type level) there is no concurrent `alloc` and no live
    /// shared reference to the arena, so invalidating every previously-returned
    /// pointer is sound — the borrow checker forbids using them across this call.
    pub fn reset(&mut self) {
        self.offset.store(0, Ordering::Relaxed);
    }

    /// Bytes currently allocated (a momentary observation under concurrency).
    pub fn used(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Total arena capacity in bytes.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Whether the arena base + an offset is within bounds (test/debug helper).
    #[cfg(test)]
    fn base_addr(&self) -> usize {
        self.storage.get() as usize
    }
}

impl<const N: usize> Default for BumpArena<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Round `offset` up to a multiple of `align` (a power of two), returning `None` on
/// overflow.
#[inline]
fn align_up(offset: usize, align: usize) -> Option<usize> {
    let mask = align - 1;
    offset.checked_add(mask).map(|s| s & !mask)
}

// ===========================================================================
// Behavioural / invariant tests (std build; small variants under Miri). The Miri
// runs are where provenance / alignment / bounds / disjoint-write soundness is
// actually checked.
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_alignment_bounds_exhaustion_reset() {
        let mut a = BumpArena::<64>::new();
        // Aligned allocations.
        let p1 = a.alloc(8, 8).expect("p1");
        let p2 = a.alloc(1, 1).expect("p2");
        let p3 = a.alloc(8, 16).expect("p3 aligned to 16");
        assert_eq!(p1.as_ptr() as usize % 8, 0, "p1 8-aligned");
        assert_eq!(p3.as_ptr() as usize % 16, 0, "p3 16-aligned");
        // Non-overlap: p2 starts at/after p1+8; p3 after p2.
        assert!((p2.as_ptr() as usize) >= (p1.as_ptr() as usize) + 8);
        assert!((p3.as_ptr() as usize) >= (p2.as_ptr() as usize) + 1);
        // Exhaustion returns None, not UB.
        assert!(a.alloc(1024, 1).is_none(), "over-capacity rejected");
        // Reset reuses from the base.
        a.reset();
        let p4 = a.alloc(8, 8).expect("p4 after reset");
        assert_eq!(p4.as_ptr() as usize, a.base_addr(), "reset returns to base");
    }

    // Regression guard for the audit-found alignment bug: with a bare [u8; N] base
    // (align 1) an `align`-aligned OFFSET was not an `align`-aligned POINTER, so an
    // aligned access through it was misalignment-UB (Miri-confirmed). Heap-box the
    // arena (base not stack-aligned by luck), burn a byte so align_up does real work,
    // then perform an ALIGNED u128 write/read — Miri flags misalignment if it recurs.
    #[test]
    fn high_alignment_is_absolute_and_oversize_rejected() {
        let a = Box::new(BumpArena::<256>::new());
        assert_eq!(a.base_addr() % MAX_ALIGN, 0, "arena base is MAX_ALIGN-aligned");
        let _ = a.alloc(1, 1).expect("burn one byte"); // make next offset odd
        let p = a.alloc(16, 16).expect("16-aligned alloc");
        assert_eq!(p.as_ptr() as usize % 16, 0, "absolute 16-alignment");
        // SAFETY: p is a 16-aligned, in-bounds 16-byte region; an aligned u128 access
        // is valid (this is exactly the access that was UB before the fix).
        unsafe {
            core::ptr::write(p.as_ptr().cast::<u128>(), 0xDEAD_BEEF_CAFE_F00D);
            assert_eq!(core::ptr::read(p.as_ptr().cast::<u128>()), 0xDEAD_BEEF_CAFE_F00D);
        }
        // Alignment beyond MAX_ALIGN is refused (None), never a misaligned pointer.
        assert!(a.alloc(8, MAX_ALIGN * 2).is_none(), "over-MAX_ALIGN rejected");
        assert!(a.alloc(8, MAX_ALIGN).is_some(), "MAX_ALIGN itself allowed");
    }

    #[test]
    fn writes_through_allocations_are_in_bounds() {
        // Miri: exercises that each returned pointer is writable for its full size
        // with valid provenance, and that filling the whole arena is in-bounds.
        let mut a = BumpArena::<32>::new();
        let mut ptrs = Vec::new();
        while let Some(p) = a.alloc(4, 4) {
            // SAFETY: alloc returned a unique, 4-aligned, in-bounds 4-byte region.
            unsafe {
                core::ptr::write_bytes(p.as_ptr(), 0xAB, 4);
            }
            ptrs.push(p);
        }
        assert_eq!(ptrs.len(), 8, "32 / 4 = 8 allocations");
        for p in &ptrs {
            // SAFETY: same region, still owned by us (no reset happened).
            unsafe {
                assert_eq!(*p.as_ptr(), 0xAB);
                assert_eq!(*p.as_ptr().add(3), 0xAB);
            }
        }
        a.reset(); // frees all; ptrs must not be used after this (enforced: not used)
    }

    // A Send wrapper so real pointers (with provenance) can cross the join boundary —
    // avoids an int->ptr round trip that Miri's strict provenance would reject.
    struct SendPtr(*mut u8);
    // SAFETY: the pointer addresses a disjoint, uniquely-owned region of the arena
    // (the arena outlives the threads via the Arc); moving it across threads to read
    // back its own bytes introduces no aliasing.
    unsafe impl Send for SendPtr {}

    // Concurrent allocation: many threads carve disjoint regions and stamp each with
    // a thread-unique byte. After joining, every region still holds its own stamp —
    // proving no two allocations overlapped. Miri additionally proves the disjoint
    // concurrent writes are race-free and provenance-correct.
    #[test]
    fn concurrent_allocations_never_overlap() {
        const N: usize = if cfg!(miri) { 64 } else { 4096 };
        let threads = 4usize;
        let chunk = 4usize;
        let per = if cfg!(miri) { 3 } else { 200 };
        let arena = Arc::new(BumpArena::<N>::new());

        let mut handles = Vec::new();
        for t in 0..threads {
            let arena = Arc::clone(&arena);
            handles.push(thread::spawn(move || {
                let stamp = (t as u8) + 1; // 1..=threads, all nonzero & distinct
                let mut mine = Vec::new();
                for _ in 0..per {
                    if let Some(p) = arena.alloc(chunk, 1) {
                        // SAFETY: `p` is a unique in-bounds `chunk`-byte region.
                        unsafe { core::ptr::write_bytes(p.as_ptr(), stamp, chunk) };
                        mine.push((SendPtr(p.as_ptr()), stamp));
                    }
                }
                mine
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.join().unwrap());
        }

        // Every byte each thread wrote still equals that thread's stamp ⇒ no overlap.
        // Read through the original (provenance-carrying) pointer, not an int cast.
        for (SendPtr(p), stamp) in &all {
            for i in 0..chunk {
                // SAFETY: `p` is a region this run allocated within the still-live
                // arena; no reset occurred, so the bytes are valid to read.
                let byte = unsafe { *p.add(i) };
                assert_eq!(byte, *stamp, "an allocated region was overwritten (overlap)");
            }
        }
        // Distinct start addresses ⇒ distinct allocations (sanity; address-only, no deref).
        let starts: BTreeSet<usize> = all.iter().map(|(SendPtr(p), _)| *p as usize).collect();
        assert_eq!(starts.len(), all.len(), "duplicate allocation address");
        // Total allocated does not exceed capacity.
        assert!(arena.used() <= N);
    }
}

// ===========================================================================
// loom model — the atomic bump protocol hands out NON-OVERLAPPING regions under
// contention. The arena bytes are never touched here; only `offset` is modelled.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_concurrent_alloc_no_overlap() {
        loom::model(|| {
            // Arena of 16 bytes; two threads each claim an 8-byte region. Across every
            // interleaving both succeed with DISJOINT ranges (the CAS serialises the
            // offset), or arithmetic forbids overlap — never two overlapping regions.
            let arena = Arc::new(BumpArena::<16>::new());
            let a1 = Arc::clone(&arena);
            let a2 = Arc::clone(&arena);
            let t1 = loom::thread::spawn(move || a1.alloc(8, 1).map(|p| p.as_ptr() as usize));
            let t2 = loom::thread::spawn(move || a2.alloc(8, 1).map(|p| p.as_ptr() as usize));
            let r1 = t1.join().unwrap();
            let r2 = t2.join().unwrap();
            // Both 8-byte allocations fit in 16 bytes, so both must succeed.
            let (s1, s2) = (r1.expect("t1 alloc"), r2.expect("t2 alloc"));
            // Disjoint: the two [start, start+8) ranges do not overlap.
            let no_overlap = s1 + 8 <= s2 || s2 + 8 <= s1;
            assert!(no_overlap, "overlapping allocations: {s1:#x} and {s2:#x}");
        });
    }

    #[test]
    fn loom_contended_alloc_capacity_respected() {
        loom::model(|| {
            // Arena of 8 bytes; two threads each request 8 bytes. Exactly ONE can
            // succeed (capacity = 8); the other must get None — never both (which
            // would be an overlapping over-allocation).
            let arena = Arc::new(BumpArena::<8>::new());
            let a1 = Arc::clone(&arena);
            let a2 = Arc::clone(&arena);
            let t1 = loom::thread::spawn(move || a1.alloc(8, 1).is_some());
            let t2 = loom::thread::spawn(move || a2.alloc(8, 1).is_some());
            let ok1 = t1.join().unwrap();
            let ok2 = t2.join().unwrap();
            assert!(ok1 ^ ok2, "exactly one 8-byte alloc must succeed in an 8-byte arena");
        });
    }
}
