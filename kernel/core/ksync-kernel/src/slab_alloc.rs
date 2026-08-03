//! # ksync_slab_alloc — a lock-free fixed-block slab allocator (Phase 2, hard tier)
//!
//! Phase-2 subsystem-core port (`docs/context/phase2-context.md`). A *synthesis* of
//! the two verification emphases Phase 2 has exercised:
//! - the **reclamation / lock-free-list** discipline of `treiber_stack` /
//!   `hazard_stack` / `ebr_stack` — but here blocks are genuinely **recycled**
//!   (alloc → free → alloc the same block), so the free list is squarely in **ABA
//!   territory**; and
//! - the **provenance / alignment / bounds** discipline of `bump_alloc` (Miri's
//!   domain) for the block storage.
//!
//! ## Kernel provenance
//! Models the kernel `slab`/`kmem_cache` idea at its core: a fixed pool of same-size
//! objects with a free list, `alloc` popping a free object and `free` returning it.
//! The kernel guards its per-CPU free lists with locking/`this_cpu` magic; this slice
//! ports the *lock-free* fixed-block free list. Not a verbatim Linux file.
//!
//! ## The ABA problem (why a versioned head)
//! `alloc` = pop the free-list head; `free` = push onto it. Because blocks are
//! recycled, a naive pointer/index head has the **ABA bug**: thread A reads
//! head = block i and its saved `next`, B then pops i, pops `next`, and frees i
//! (head = i again but the list changed), and A's CAS wrongly succeeds with a stale
//! `next` — corrupting the list (a block gets double-allocated). The fix is a
//! **versioned head**: `head: AtomicU64` packs `(tag: u32, index: u32)`, and every
//! pop/push bumps `tag`, so a stale CAS is rejected even when the index repeats.
//!
//! ## Why it is data-race free (separate metadata)
//! The free-list links live in a **separate `[AtomicU32; N]` array**, NOT inside the
//! block bytes. So reading a block's "next free" index is always an atomic load of
//! valid metadata and never races a user's (non-atomic) writes to the block data they
//! own — the classic intrusive-free-list pitfall (overlaying the link on user data,
//! which is a data race in the Rust model) is avoided by construction. A block is in
//! the free list XOR owned by exactly one caller (the versioned CAS makes each pop
//! unique), so user writes to a block never race. loom checks the free-list protocol
//! (no double-allocation, no lost/duplicated block, ABA-safe); Miri checks the real
//! block pointers (provenance, alignment, bounds, no use-after-free of recycled
//! blocks, race-free disjoint writes).
//!
//! ## Scope (documented, not faked)
//! Fixed `N` blocks of `BLOCK` bytes (both compile-time; `BLOCK` a power of two
//! `<= 64`, so each block is `BLOCK`-aligned off the 64-aligned base); `free` is
//! `unsafe` (the caller must pass a pointer from this slab, exactly once); the 32-bit
//! tag wraps after 2^32 pop/push pairs (unreachable in any realistic run; not in
//! loom/Miri). A verified lock-free slab core, not a production `kmem_cache`. Not
//! Linux parity.

#[cfg(loom)]
use loom::sync::atomic::{AtomicU32, AtomicU64, Ordering};
#[cfg(not(loom))]
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// Free-list sentinel meaning "empty" (no free block).
const EMPTY: u32 = u32::MAX;

/// Max supported block alignment (the storage is over-aligned to this; a power-of-two
/// `BLOCK <= MAX_ALIGN` is therefore `BLOCK`-aligned at every slot).
const MAX_ALIGN: usize = 64;

#[inline]
fn pack(tag: u32, idx: u32) -> u64 {
    ((tag as u64) << 32) | (idx as u64)
}
#[inline]
fn unpack(h: u64) -> (u32, u32) {
    ((h >> 32) as u32, h as u32)
}

/// Over-aligned backing storage: `N` blocks of `BLOCK` bytes. `#[repr(align(64))]`
/// forces the base to `MAX_ALIGN`, so block `i` at `base + i*BLOCK` is `BLOCK`-aligned
/// (for power-of-two `BLOCK <= MAX_ALIGN`) — see `bump_alloc` for why offset alignment
/// must equal absolute alignment.
#[repr(C, align(64))]
struct Slab<const N: usize, const BLOCK: usize>([[u8; BLOCK]; N]);

/// A lock-free fixed-block slab allocator. `alloc(&self)` and `free(&self, …)` are
/// lock-free and may be called concurrently from any number of threads.
pub struct SlabAllocator<const N: usize, const BLOCK: usize> {
    storage: UnsafeCell<Slab<N, BLOCK>>,
    /// Free-list links: `next[i]` is the index of the block after `i` in the free
    /// list (or `EMPTY`). Separate from block data so reads never race user writes.
    next: [AtomicU32; N],
    /// Versioned free-list head: `(tag, index)`; `index == EMPTY` means no free block.
    head: AtomicU64,
}

// SAFETY: a block is in the free list XOR exclusively owned by one caller (the
// versioned CAS makes each pop unique), so user writes to a block never race; the
// only shared mutable state is the atomic `head` and the atomic `next` metadata
// array (never aliased with block data). Sharing `&SlabAllocator` across threads is
// therefore sound. The slab holds plain bytes (no `T`), so no element bounds apply.
unsafe impl<const N: usize, const BLOCK: usize> Sync for SlabAllocator<N, BLOCK> {}
unsafe impl<const N: usize, const BLOCK: usize> Send for SlabAllocator<N, BLOCK> {}

impl<const N: usize, const BLOCK: usize> SlabAllocator<N, BLOCK> {
    /// Create a slab with all `N` blocks free. (Not `const`: loom atomics are not
    /// const-constructible.)
    pub fn new() -> Self {
        assert!(N >= 1, "slab needs at least one block");
        assert!(BLOCK.is_power_of_two(), "BLOCK must be a power of two");
        assert!(BLOCK <= MAX_ALIGN, "BLOCK must be <= MAX_ALIGN");
        assert!(N < EMPTY as usize, "N must be < u32::MAX (index space)");
        // Free list initially threads every block: 0 -> 1 -> ... -> N-1 -> EMPTY.
        let next = core::array::from_fn(|i| {
            AtomicU32::new(if i + 1 < N { (i + 1) as u32 } else { EMPTY })
        });
        Self {
            storage: UnsafeCell::new(Slab([[0u8; BLOCK]; N])),
            next,
            head: AtomicU64::new(pack(0, 0)), // head = block 0
        }
    }

    /// Allocate one `BLOCK`-byte block (lock-free). Returns `None` if the slab is
    /// exhausted. The region is uninitialised; the caller owns it exclusively until
    /// it is passed to [`free`](Self::free). The pointer is `BLOCK`-aligned.
    pub fn alloc(&self) -> Option<NonNull<u8>> {
        loop {
            let h = self.head.load(Ordering::Acquire);
            let (tag, idx) = unpack(h);
            if idx == EMPTY {
                return None; // exhausted
            }
            // Atomic load of valid metadata (never races user block writes).
            let nxt = self.next[idx as usize].load(Ordering::Acquire);
            // Pop `idx`; bump the tag so a stale (ABA) CAS cannot succeed.
            let new = pack(tag.wrapping_add(1), nxt);
            if self
                .head
                .compare_exchange_weak(h, new, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // SAFETY: we uniquely popped block `idx` (idx < N, the CAS winner owns
                // it), so deriving its pointer from the slab's single backing
                // allocation is in-bounds (idx*BLOCK + BLOCK <= N*BLOCK) and
                // provenance-valid; base is MAX_ALIGN-aligned and BLOCK is a power of
                // two <= MAX_ALIGN, so the block pointer is BLOCK-aligned.
                let base = self.storage.get() as *mut u8;
                let ptr = unsafe { base.add(idx as usize * BLOCK) };
                return Some(unsafe { NonNull::new_unchecked(ptr) });
            }
            // Lost the race (or spurious failure / tag changed); retry.
        }
    }

    /// Return a block to the free list (lock-free).
    ///
    /// # Safety
    /// `ptr` must be a pointer previously returned by [`alloc`](Self::alloc) on *this*
    /// slab and not yet freed (no double-free). After this call the caller must not
    /// use `ptr` or the block it points to.
    pub unsafe fn free(&self, ptr: NonNull<u8>) {
        let base = self.storage.get() as *mut u8;
        // Index from the byte offset. Address arithmetic (no deref) — `ptr` came from
        // `base.add(idx*BLOCK)`, so this recovers `idx` exactly.
        let off = (ptr.as_ptr() as usize) - (base as usize);
        let idx_usize = off / BLOCK;
        debug_assert!(idx_usize * BLOCK == off && idx_usize < N, "ptr not a slab block");
        let idx = idx_usize as u32;
        loop {
            let h = self.head.load(Ordering::Acquire);
            let (tag, hidx) = unpack(h);
            // Link the freed block to the current head (atomic store of metadata).
            self.next[idx as usize].store(hidx, Ordering::Release);
            // Push `idx`; bump the tag (ABA defence + publishes the next-store).
            let new = pack(tag.wrapping_add(1), idx);
            if self
                .head
                .compare_exchange_weak(h, new, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return;
            }
        }
    }

    /// Total number of blocks.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Size of each block in bytes.
    pub const fn block_size(&self) -> usize {
        BLOCK
    }
}

impl<const N: usize, const BLOCK: usize> Default for SlabAllocator<N, BLOCK> {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Behavioural / invariant tests (std build; small variants under Miri). The Miri
// runs check provenance / alignment / bounds / use-after-free of recycled blocks.
// ===========================================================================
#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn single_thread_alloc_free_recycle() {
        let s = SlabAllocator::<4, 8>::new();
        // Allocate all 4 blocks; they are distinct and 8-aligned.
        let mut ptrs = Vec::new();
        for _ in 0..4 {
            let p = s.alloc().expect("block");
            assert_eq!(p.as_ptr() as usize % 8, 0, "block 8-aligned");
            ptrs.push(p);
        }
        let addrs: BTreeSet<usize> = ptrs.iter().map(|p| p.as_ptr() as usize).collect();
        assert_eq!(addrs.len(), 4, "all 4 blocks distinct");
        assert!(s.alloc().is_none(), "exhausted");
        // Free one and re-alloc: must recycle (return a previously-seen block).
        let freed = ptrs.pop().unwrap();
        let freed_addr = freed.as_ptr() as usize;
        unsafe { s.free(freed) };
        let re = s.alloc().expect("recycled block");
        assert_eq!(re.as_ptr() as usize, freed_addr, "recycled the freed block");
        // Free all; the slab can hand out exactly 4 again (no leak/corruption).
        ptrs.push(re);
        for p in ptrs {
            unsafe { s.free(p) };
        }
        let mut count = 0;
        while s.alloc().is_some() {
            count += 1;
        }
        assert_eq!(count, 4, "all blocks recovered after free");
    }

    #[test]
    fn writes_to_distinct_blocks_are_isolated() {
        // Miri: write a distinct pattern into each allocated block, then verify each
        // still holds it (blocks do not overlap) and provenance/alignment are valid.
        let s = SlabAllocator::<8, 4>::new();
        let mut ptrs = Vec::new();
        for i in 0..8u8 {
            let p = s.alloc().expect("block");
            // SAFETY: p is a unique, in-bounds 4-byte block we own.
            unsafe { core::ptr::write_bytes(p.as_ptr(), i + 1, 4) };
            ptrs.push((p, i + 1));
        }
        for (p, val) in &ptrs {
            // SAFETY: still owned (not freed); read its 4 bytes.
            unsafe {
                assert_eq!(*p.as_ptr(), *val);
                assert_eq!(*p.as_ptr().add(3), *val);
            }
        }
        for (p, _) in ptrs {
            unsafe { s.free(p) };
        }
    }

    // A Send wrapper so real block pointers (with provenance) can cross the join
    // boundary for post-hoc verification, without an int->ptr round trip.
    struct SendPtr(NonNull<u8>);
    // SAFETY: each pointer is a uniquely-owned slab block (the slab outlives the
    // threads via the Arc); moving it across threads introduces no aliasing.
    unsafe impl Send for SendPtr {}

    // Concurrent alloc/free stress with REAL recycling (ABA territory): threads
    // repeatedly alloc a block, stamp every byte with a thread id, check the stamp
    // survived (exclusive ownership), then free it. Afterwards the slab must hand out
    // exactly N blocks — proving the free list was never corrupted (no block lost or
    // duplicated). Miri additionally proves no use-after-free of recycled blocks and
    // no data race.
    #[test]
    fn concurrent_alloc_free_preserves_freelist_integrity() {
        const N: usize = 8;
        const BLOCK: usize = 4;
        let threads = 4usize;
        let rounds = if cfg!(miri) { 4 } else { 2_000 };
        let s = Arc::new(SlabAllocator::<N, BLOCK>::new());

        let mut handles = Vec::new();
        for t in 0..threads {
            let s = Arc::clone(&s);
            handles.push(thread::spawn(move || {
                let stamp = (t as u8) + 1;
                for _ in 0..rounds {
                    if let Some(p) = s.alloc() {
                        // SAFETY: we exclusively own this block until we free it.
                        unsafe { core::ptr::write_bytes(p.as_ptr(), stamp, BLOCK) };
                        // Verify no other thread wrote our block (exclusive ownership).
                        for i in 0..BLOCK {
                            assert_eq!(unsafe { *p.as_ptr().add(i) }, stamp, "block aliased!");
                        }
                        // SAFETY: ptr from this slab, freed exactly once here.
                        unsafe { s.free(p) };
                    }
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // Free-list integrity: exactly N blocks recoverable, all distinct.
        let mut got = Vec::new();
        while let Some(p) = s.alloc() {
            got.push(SendPtr(p));
        }
        assert_eq!(got.len(), N, "free list lost or duplicated blocks");
        let addrs: BTreeSet<usize> = got.iter().map(|SendPtr(p)| p.as_ptr() as usize).collect();
        assert_eq!(addrs.len(), N, "duplicate block handed out");
    }
}

// ===========================================================================
// loom model — the lock-free free-list protocol (no double-allocation, ABA-safe,
// no lost/duplicated block) under bounded preemption.
//   RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test loom_
// ===========================================================================
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;

    #[test]
    fn loom_concurrent_alloc_distinct_blocks() {
        loom::model(|| {
            // 2-block slab; two threads alloc concurrently. Both must succeed with
            // DIFFERENT blocks — never the same block handed to both (which the
            // versioned CAS prevents).
            let s = Arc::new(SlabAllocator::<2, 8>::new());
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            let t1 = loom::thread::spawn(move || s1.alloc().map(|p| p.as_ptr() as usize));
            let t2 = loom::thread::spawn(move || s2.alloc().map(|p| p.as_ptr() as usize));
            let a = t1.join().unwrap().expect("t1 block");
            let b = t2.join().unwrap().expect("t2 block");
            assert_ne!(a, b, "two threads got the SAME block (double-allocation)");
        });
    }

    // ADVERSARIAL (reviewer-added): classic 3-deep ABA. List starts head -> 0 -> 1 -> 2.
    // Thread A allocs (reads head=X, nxt=Y) and may be preempted before its CAS.
    // Thread B pops X, pops Y, then frees X back (so the head index == X again but Y is
    // now live in B's hands). If A's stale CAS (head: X -> Y) succeeds, Y is double-
    // handed. With the versioned tag this MUST be impossible; without it loom finds a
    // double-allocation or a lost/duplicated block. (Validated: this test FAILS when the
    // tag bump is removed, so it genuinely exercises the ABA defence.)
    #[test]
    fn loom_adv_classic_three_deep_aba() {
        loom::model(|| {
            let s = Arc::new(SlabAllocator::<3, 8>::new());
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            let t1 = loom::thread::spawn(move || s1.alloc().map(|p| p.as_ptr() as usize));
            let t2 = loom::thread::spawn(move || {
                let x = s2.alloc();
                let _y = s2.alloc();
                if let Some(px) = x {
                    unsafe { s2.free(px) };
                }
                _y.map(|p| p.as_ptr() as usize)
            });
            let a = t1.join().unwrap();
            let b_y = t2.join().unwrap();
            let mut live = std::collections::BTreeSet::new();
            if let Some(a) = a {
                assert!(live.insert(a), "A's block already counted");
            }
            if let Some(y) = b_y {
                assert!(
                    live.insert(y),
                    "DOUBLE-ALLOCATION: same block live in two threads (ABA!)"
                );
            }
            while let Some(p) = s.alloc() {
                let addr = p.as_ptr() as usize;
                assert!(
                    live.insert(addr),
                    "DUPLICATED block: free list handed out a live block again (ABA!)"
                );
            }
            assert_eq!(live.len(), 3, "block LOST or DUPLICATED (3-deep ABA)");
        });
    }

    #[test]
    fn loom_alloc_free_realloc_no_corruption() {
        loom::model(|| {
            // ABA stress: a 2-block slab, two threads each do alloc->free->alloc->free.
            // Whatever the interleaving, afterwards the free list must still yield
            // EXACTLY 2 distinct blocks — proving no block was lost or duplicated by
            // an ABA on the versioned head.
            let s = Arc::new(SlabAllocator::<2, 8>::new());
            let s1 = Arc::clone(&s);
            let s2 = Arc::clone(&s);
            let work = move |slab: &SlabAllocator<2, 8>| {
                if let Some(p) = slab.alloc() {
                    // SAFETY: owned block from this slab, freed once.
                    unsafe { slab.free(p) };
                }
            };
            let t1 = loom::thread::spawn(move || work(&s1));
            let t2 = loom::thread::spawn(move || work(&s2));
            t1.join().unwrap();
            t2.join().unwrap();
            // Drain: the slab must give back exactly 2 distinct blocks.
            let p0 = s.alloc().expect("block 0 recoverable");
            let p1 = s.alloc().expect("block 1 recoverable");
            assert!(s.alloc().is_none(), "more than 2 blocks (duplicated!)");
            assert_ne!(
                p0.as_ptr() as usize,
                p1.as_ptr() as usize,
                "free list duplicated a block (ABA corruption)"
            );
        });
    }

    // -----------------------------------------------------------------------
    // H3 — allocator-under-SMP (P4·SMP·S2, Part C). The slab free-list IS the
    // kernel's lock-free free-list primitive; S4's fork/cow_fault/exit hit it
    // (and `Arc` refcounts) concurrently. H3 adds (1) an EXPLICIT double-free
    // scenario and (2) the COW-frame `Arc` refcount model on top of the existing
    // ABA/conservation models above.
    // -----------------------------------------------------------------------

    // H3(1) EXPLICIT double-free / re-alloc race: thread A frees a block while
    // thread B (and the post-join drain) re-allocates. The versioned head must
    // never hand the SAME live block to two owners and must conserve exactly the
    // 2 blocks. A racing alloc sees either the freed block or `None`, never a
    // duplicate; after joining, the slab yields exactly 2 DISTINCT blocks.
    #[test]
    fn loom_alloc_explicit_double_free_conservation() {
        loom::model(|| {
            let s = Arc::new(SlabAllocator::<2, 8>::new());
            // Pre-allocate both blocks; record their addresses.
            let a = s.alloc().expect("block a");
            let b = s.alloc().expect("block b");
            assert!(s.alloc().is_none(), "slab is full");

            let s_free = Arc::clone(&s);
            let s_alloc = Arc::clone(&s);
            let a_addr = a.as_ptr() as usize;

            // Thread 1 frees block `a`.
            let t_free = loom::thread::spawn(move || {
                // SAFETY: `a` came from this slab and is freed exactly once here.
                unsafe { s_free.free(a) };
            });
            // Thread 2 races to re-allocate. It gets `a` back (the only free
            // block) or `None` (if it ran before the free) — never block `b`
            // (still live) and never a duplicate of `a`.
            let t_alloc = loom::thread::spawn(move || {
                s_alloc.alloc().map(|p| p.as_ptr() as usize)
            });

            t_free.join().unwrap();
            let got = t_alloc.join().unwrap();
            if let Some(addr) = got {
                assert_eq!(addr, a_addr, "re-alloc handed out a block other than the freed one");
            }

            // Conservation: account for every block exactly once. `b` is still
            // held; `a` is either held by the racing alloc or back on the free
            // list. Drain whatever is free and assert the live set is exactly
            // {a, b}, all distinct (no lost, no duplicated block).
            use std::collections::BTreeSet;
            let mut live = BTreeSet::new();
            assert!(live.insert(b.as_ptr() as usize), "b counted twice");
            if let Some(addr) = got {
                assert!(live.insert(addr), "DOUBLE-ALLOCATION: re-alloc duplicated a live block");
            }
            while let Some(p) = s.alloc() {
                assert!(
                    live.insert(p.as_ptr() as usize),
                    "DUPLICATED block: free list handed out a live block (double-free corruption)"
                );
            }
            assert_eq!(live.len(), 2, "block LOST or DUPLICATED across the double-free race");
            // SAFETY: tidy up `b` (owned, freed once) so loom sees no leak.
            unsafe { s.free(b) };
        });
    }

    // H3(2) COW-frame `Arc` refcount model: S4 shares `Arc<Box<Frame>>` across
    // CPUs on a copy-on-write fault; the refcount ordering must drop the payload
    // EXACTLY ONCE (no premature free, no leak) under concurrent clone/drop.
    // loom's `Arc` is instrumented, so this directly checks that ordering. A
    // loom `AtomicUsize` drop-counter confirms exactly-once destruction.
    #[test]
    fn loom_cow_arc_refcount_drops_exactly_once() {
        use loom::sync::atomic::{AtomicUsize, Ordering as O};

        loom::model(|| {
            // A payload that bumps a shared counter when dropped. The counter
            // itself is shared via a second Arc so both the model and the payload
            // can reach it.
            struct CowFrame {
                drops: Arc<AtomicUsize>,
            }
            impl Drop for CowFrame {
                fn drop(&mut self) {
                    self.drops.fetch_add(1, O::Release);
                }
            }

            let drops = Arc::new(AtomicUsize::new(0));
            // The shared COW frame, refcount = 1.
            let frame = Arc::new(CowFrame {
                drops: Arc::clone(&drops),
            });

            // Two CPUs each clone the shared frame (cow fault), use it, then drop.
            let f1 = Arc::clone(&frame);
            let f2 = Arc::clone(&frame);
            let t1 = loom::thread::spawn(move || {
                let _local = Arc::clone(&f1); // a further clone, then both drop
                drop(_local);
                drop(f1);
            });
            let t2 = loom::thread::spawn(move || {
                drop(f2);
            });
            t1.join().unwrap();
            t2.join().unwrap();
            // The original handle drops last here.
            drop(frame);

            // Exactly-once destruction: the payload's Drop ran a single time —
            // no premature free (would be >1 if a clone double-dropped) and no
            // leak (would be 0 if a strong ref lingered).
            assert_eq!(
                drops.load(O::Acquire),
                1,
                "COW frame payload dropped {} times (must be exactly once)",
                drops.load(O::Acquire)
            );
        });
    }
}
