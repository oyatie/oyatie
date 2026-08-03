// Pure, arch-neutral split-virtqueue ring-index/offset arithmetic (VIRTIO 1.x,
// "split virtqueue" layout, §2.7 of the VIRTIO spec).
//
// This file is the body of the `user_layout::virtqueue` module (it is `include!`d
// from `lib.rs`, alongside `layout.rs`/`signal.rs`/`timekeep.rs`/`vfs.rs`/
// `netlink.rs`, which supply the crate-level `#![no_std]`). Like those it carries
// **no inner attributes** (`#![...]`) or `//!` module docs, because it is
// `include!`d both into this crate and into the out-of-workspace host harness's
// module body, where inner attributes are not permitted.
//
// It is the **single source of truth** for the parts of a split virtqueue that
// are a *pure function* of their inputs and therefore identical on every arch:
// the three rings' byte layout (descriptor table / available ring / used ring),
// the per-element byte offsets, the available/used index publish + consume with
// wrap-at-`2^16` of the producer/consumer indices and wrap-at-`SIZE` of the ring
// *slots*, and the free-descriptor chain (a `Vec`-free intrusive freelist that
// lives inside the descriptor `next` fields). It depends on **nothing** outside
// `core` and contains **zero `unsafe`**, so the `check-tcb.sh` ratchet stays
// green and the arithmetic is exhaustively host-tested (see `mod
// virtqueue_tests` at the bottom).
//
// Keeping this logic pure keeps the `unsafe` arch Frame thin: the Frame (a later
// item, K-P5-*) does ONLY the things that MUST be unsafe — the volatile MMIO
// reads/writes of the published indices, the DMA-coherent physaddr translation,
// and the memory barriers. It asks *this* module "what byte offset is descriptor
// `i`'s `flags` field at?" / "which slot does avail index `idx` publish into?"
// and never does ring arithmetic itself. So there are deliberately NO volatile
// reads, NO MMIO, and NO physical-address translation here.
//
// ## Split-virtqueue memory layout (VIRTIO 1.x §2.7)
// A split virtqueue of `SIZE` (a.k.a. "queue size", a power of two) descriptors
// is three contiguous, independently-aligned regions:
//
//   Descriptor Table:  SIZE × 16-byte `struct virtq_desc { le64 addr; le32 len;
//                      le16 flags; le16 next; }`
//   Available Ring:    le16 flags; le16 idx; le16 ring[SIZE]; (+ optional
//                      le16 used_event)  — driver -> device.
//   Used Ring:         le16 flags; le16 idx; struct { le32 id; le32 len; }
//                      ring[SIZE]; (+ optional le16 avail_event) — device ->
//                      driver.
//
// This module computes the byte offsets of every field above (relative to each
// ring's own base) and tracks the in-memory producer/consumer indices; it never
// dereferences them.

// ---------------------------------------------------------------------------
// virtq_desc flag bits (VIRTIO 1.x §2.7.5).
// ---------------------------------------------------------------------------

/// `VIRTQ_DESC_F_NEXT` — the descriptor chains to another via its `next` field.
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
/// `VIRTQ_DESC_F_WRITE` — the buffer is device-write-only (else driver-read-only).
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
/// `VIRTQ_DESC_F_INDIRECT` — the buffer contains a list of indirect descriptors.
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

// ---------------------------------------------------------------------------
// Fixed element sizes (bytes). These are wire-format constants, identical on
// every arch (the rings are little-endian by spec; both target arches are LE).
// ---------------------------------------------------------------------------

/// Size of one `struct virtq_desc`: `addr`(8) + `len`(4) + `flags`(2) + `next`(2).
pub const DESC_SIZE: usize = 16;
/// Byte offset of `addr` within a `virtq_desc`.
pub const DESC_ADDR_OFF: usize = 0;
/// Byte offset of `len` within a `virtq_desc`.
pub const DESC_LEN_OFF: usize = 8;
/// Byte offset of `flags` within a `virtq_desc`.
pub const DESC_FLAGS_OFF: usize = 12;
/// Byte offset of `next` within a `virtq_desc`.
pub const DESC_NEXT_OFF: usize = 14;

/// Byte offset of `flags` within the available ring header.
pub const AVAIL_FLAGS_OFF: usize = 0;
/// Byte offset of `idx` within the available ring header.
pub const AVAIL_IDX_OFF: usize = 2;
/// Byte offset of the first `ring[0]` entry within the available ring (after the
/// 2-byte `flags` + 2-byte `idx` header). Each entry is a `le16` descriptor id.
pub const AVAIL_RING_OFF: usize = 4;
/// Size of one available-ring entry (a `le16` descriptor-table index).
pub const AVAIL_RING_ENTRY_SIZE: usize = 2;

/// Byte offset of `flags` within the used ring header.
pub const USED_FLAGS_OFF: usize = 0;
/// Byte offset of `idx` within the used ring header.
pub const USED_IDX_OFF: usize = 2;
/// Byte offset of the first `ring[0]` entry within the used ring (after the
/// 2-byte `flags` + 2-byte `idx` header). Each entry is a `struct virtq_used_elem`.
pub const USED_RING_OFF: usize = 4;
/// Size of one `struct virtq_used_elem`: `id`(le32) + `len`(le32).
pub const USED_RING_ENTRY_SIZE: usize = 8;
/// Byte offset of `id` within a `virtq_used_elem`.
pub const USED_ELEM_ID_OFF: usize = 0;
/// Byte offset of `len` within a `virtq_used_elem`.
pub const USED_ELEM_LEN_OFF: usize = 4;

// ---------------------------------------------------------------------------
// One consumed used-ring element, returned by `VirtQueue::take_used`.
// ---------------------------------------------------------------------------

/// A used-ring element the device wrote: the head descriptor `id` of a completed
/// chain and the number of `len` bytes the device wrote into it. Pure data; the
/// arch Frame reads these from the used ring via the offsets this module gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UsedElem {
    /// Descriptor-table index of the head of the completed chain (`0..SIZE`).
    pub id: u16,
    /// Bytes the device wrote into the chain (device-controlled; not validated).
    pub len: u32,
}

// ---------------------------------------------------------------------------
// The pure split-virtqueue index/offset machine.
// ---------------------------------------------------------------------------

/// Pure ring-index/offset arithmetic for a split virtqueue of `SIZE` descriptors
/// (`SIZE` must be a power of two in `1..=32768`, as the VIRTIO spec requires;
/// `new()` debug-asserts this). It owns NO buffer memory and performs NO I/O: it
/// tracks the producer index we will next publish into the available ring
/// (`avail_idx`), the consumer index we have consumed up to in the used ring
/// (`used_last`), and the head of an intrusive free-descriptor chain — and it
/// answers byte-offset queries for every ring field. The arch Frame owns the
/// actual DMA memory and does the volatile reads/writes at the offsets this
/// returns.
///
/// All ring-*slot* indexing wraps at `SIZE` (`idx % SIZE`); the 16-bit
/// producer/consumer indices themselves wrap at `2^16` exactly like the
/// hardware's `le16 idx` fields (so `used_last`/`avail_idx` are compared modulo
/// `2^16` via wrapping subtraction). `SIZE` being a power of two makes the
/// `% SIZE` a mask, but we keep the explicit `%` for clarity; the const generic
/// guarantees `SIZE > 0`, so there is no divide-by-zero.
#[derive(Clone, Copy, Debug)]
pub struct VirtQueue<const SIZE: usize> {
    /// Next available-ring producer index (mod `2^16`); published to `avail.idx`.
    avail_idx: u16,
    /// Last used-ring index we have consumed (mod `2^16`); compared against the
    /// device-published `used.idx` to find newly-completed chains.
    used_last: u16,
    /// Head of the free-descriptor chain, or `SIZE` (== none) when exhausted.
    /// Free descriptors are threaded through their own `next` field, which the
    /// Frame writes from `free_next_target` / reads back into `push_free`.
    free_head: usize,
    /// Number of descriptors currently on the free chain (`0..=SIZE`).
    free_count: usize,
}

impl<const SIZE: usize> VirtQueue<SIZE> {
    /// A fresh virtqueue with all `SIZE` descriptors on the free chain (head =
    /// descriptor 0, threaded 0 -> 1 -> ... -> SIZE-1), both ring indices at 0.
    ///
    /// Debug-asserts the VIRTIO `SIZE` invariant (power of two, `1..=32768`); in
    /// release builds a bad `SIZE` simply yields a queue whose `% SIZE` masking
    /// still never panics (no divide-by-zero, since `SIZE > 0` is required to
    /// even instantiate a non-empty free chain).
    pub fn new() -> Self {
        debug_assert!(SIZE > 0, "virtqueue SIZE must be > 0");
        debug_assert!(SIZE <= 32768, "virtqueue SIZE must be <= 32768");
        debug_assert!(SIZE.is_power_of_two(), "virtqueue SIZE must be a power of two");
        VirtQueue {
            avail_idx: 0,
            used_last: 0,
            // All descriptors start free: head = 0, the Frame initialises each
            // descriptor's `next` to i+1 (see `initial_free_next`).
            free_head: if SIZE == 0 { SIZE } else { 0 },
            free_count: SIZE,
        }
    }

    /// The queue size (`SIZE`), as a runtime value.
    pub fn size(&self) -> usize {
        SIZE
    }

    // ---- Per-ring base byte offsets (relative to a region base) -----------

    /// Total byte size of the descriptor table (`SIZE × 16`).
    pub const fn desc_table_size() -> usize {
        SIZE * DESC_SIZE
    }

    /// Total byte size of the available ring *without* the optional
    /// `used_event` suffix: `flags`(2) + `idx`(2) + `SIZE × 2`.
    pub const fn avail_ring_size() -> usize {
        AVAIL_RING_OFF + SIZE * AVAIL_RING_ENTRY_SIZE
    }

    /// Total byte size of the used ring *without* the optional `avail_event`
    /// suffix: `flags`(2) + `idx`(2) + `SIZE × 8`.
    pub const fn used_ring_size() -> usize {
        USED_RING_OFF + SIZE * USED_RING_ENTRY_SIZE
    }

    // ---- Descriptor-table field offsets -----------------------------------

    /// Byte offset of descriptor `index`'s base within the descriptor table.
    /// `index` is taken modulo `SIZE` so a wrapped chain pointer never indexes
    /// out of the table.
    pub fn desc_offset(&self, index: usize) -> usize {
        (index % SIZE) * DESC_SIZE
    }

    /// Byte offset of descriptor `index`'s `addr` (le64) field.
    pub fn desc_addr_offset(&self, index: usize) -> usize {
        self.desc_offset(index) + DESC_ADDR_OFF
    }

    /// Byte offset of descriptor `index`'s `len` (le32) field.
    pub fn desc_len_offset(&self, index: usize) -> usize {
        self.desc_offset(index) + DESC_LEN_OFF
    }

    /// Byte offset of descriptor `index`'s `flags` (le16) field.
    pub fn desc_flags_offset(&self, index: usize) -> usize {
        self.desc_offset(index) + DESC_FLAGS_OFF
    }

    /// Byte offset of descriptor `index`'s `next` (le16) field.
    pub fn desc_next_offset(&self, index: usize) -> usize {
        self.desc_offset(index) + DESC_NEXT_OFF
    }

    // ---- Available-ring offsets + publish ---------------------------------

    /// Byte offset of available-ring slot `idx`'s `le16` descriptor-id entry.
    /// `idx` is the raw 16-bit producer index; the slot wraps at `SIZE`.
    pub fn avail_ring_entry_offset(&self, idx: u16) -> usize {
        AVAIL_RING_OFF + (idx as usize % SIZE) * AVAIL_RING_ENTRY_SIZE
    }

    /// The producer index value we will next publish (the current `avail.idx`,
    /// before incrementing). Use this both as the `le16` to write to `avail.idx`
    /// (after a publish) and to compute the ring slot for the *next* entry.
    pub fn avail_idx(&self) -> u16 {
        self.avail_idx
    }

    /// Publish descriptor-chain head `head` into the available ring: returns the
    /// byte offset of the ring slot the Frame must write `head` into, and bumps
    /// the producer index (wrapping at `2^16`). The Frame then writes the new
    /// [`avail_idx`](Self::avail_idx) to the ring's `idx` field (with the
    /// required memory barrier) to make the entry visible to the device.
    ///
    /// Returns `(slot_byte_offset, head)`; `head` is echoed back so the Frame
    /// has the exact `le16` to store. The slot uses the *pre-increment* index, so
    /// the device, reading `avail.idx`, sees entries `[old_idx, new_idx)`.
    pub fn publish_avail(&mut self, head: u16) -> (usize, u16) {
        let slot = self.avail_ring_entry_offset(self.avail_idx);
        self.avail_idx = self.avail_idx.wrapping_add(1);
        (slot, head)
    }

    // ---- Used-ring offsets + consume --------------------------------------

    /// Byte offset of used-ring slot `idx`'s `struct virtq_used_elem`. `idx` is
    /// the raw 16-bit consumer index; the slot wraps at `SIZE`.
    pub fn used_ring_entry_offset(&self, idx: u16) -> usize {
        USED_RING_OFF + (idx as usize % SIZE) * USED_RING_ENTRY_SIZE
    }

    /// Byte offset of the `id` (le32) field of used-ring slot `idx`.
    pub fn used_elem_id_offset(&self, idx: u16) -> usize {
        self.used_ring_entry_offset(idx) + USED_ELEM_ID_OFF
    }

    /// Byte offset of the `len` (le32) field of used-ring slot `idx`.
    pub fn used_elem_len_offset(&self, idx: u16) -> usize {
        self.used_ring_entry_offset(idx) + USED_ELEM_LEN_OFF
    }

    /// The last used-ring index we have consumed up to (our private consumer
    /// cursor; the device publishes its own `used.idx`).
    pub fn used_last(&self) -> u16 {
        self.used_last
    }

    /// Number of newly-completed used-ring entries given the device's published
    /// `used.idx`. Computed with wrapping subtraction so it is correct across the
    /// `2^16` wrap of either index. The result never exceeds `SIZE` for a
    /// well-behaved device (the spec forbids more than `SIZE` in-flight), but we
    /// do not clamp: the caller (Frame) treats a value `> SIZE` as a protocol
    /// error from a misbehaving device.
    pub fn used_available(&self, device_used_idx: u16) -> u16 {
        device_used_idx.wrapping_sub(self.used_last)
    }

    /// The byte offset of the next used-ring slot to consume (the slot for the
    /// current `used_last`), or `None` if no new entry is available given
    /// `device_used_idx`. Does NOT advance the cursor — call [`take_used`] for
    /// that. This lets the Frame read the `id`/`len` at the returned offset (via
    /// [`used_elem_id_offset`]/[`used_elem_len_offset`] applied to
    /// [`used_last`]) before committing the consume.
    pub fn next_used_offset(&self, device_used_idx: u16) -> Option<usize> {
        if self.used_available(device_used_idx) == 0 {
            None
        } else {
            Some(self.used_ring_entry_offset(self.used_last))
        }
    }

    /// Consume one used-ring entry: the Frame has read `(id, len)` from the slot
    /// at [`next_used_offset`]; this records it as an [`UsedElem`], advances the
    /// consumer cursor (wrapping at `2^16`), returns the descriptor `id`'s chain
    /// head to the free chain accounting via the returned [`UsedElem`], and
    /// yields `Some(elem)`. Returns `None` (and does not advance) if no new entry
    /// is available given `device_used_idx`.
    ///
    /// The `id`/`len` arguments are the values the Frame read from the device's
    /// used ring; this module trusts them only as opaque data (it masks `id` into
    /// `0..SIZE` when the caller later threads it back onto the free chain via
    /// [`push_free`]).
    pub fn take_used(&mut self, device_used_idx: u16, id: u16, len: u32) -> Option<UsedElem> {
        if self.used_available(device_used_idx) == 0 {
            return None;
        }
        self.used_last = self.used_last.wrapping_add(1);
        Some(UsedElem { id, len })
    }

    // ---- Free-descriptor chain (intrusive, lives in `desc.next`) ----------

    /// Number of free descriptors currently available to allocate.
    pub fn free_count(&self) -> usize {
        self.free_count
    }

    /// True when no descriptors are free (the queue is full of in-flight chains).
    pub fn is_exhausted(&self) -> bool {
        self.free_count == 0
    }

    /// The current free-chain head descriptor index, or `None` if exhausted.
    /// (`SIZE` is the in-band "none" sentinel internally.)
    pub fn free_head(&self) -> Option<usize> {
        if self.free_head >= SIZE {
            None
        } else {
            Some(self.free_head)
        }
    }

    /// The descriptor index `desc`'s `next` field should point at when `desc` is
    /// the head being freshly pushed onto the free chain: the *old* free head, or
    /// `SIZE` (the none-sentinel) if the chain was empty. The Frame writes this
    /// value into descriptor `desc`'s `next` field before/at the moment it calls
    /// [`push_free`]. Pure read of current state; does not mutate.
    pub fn free_next_target(&self) -> usize {
        self.free_head
    }

    /// Pop the head descriptor off the free chain, returning its index, and set
    /// the new head to `next_of_head` (the value the Frame read from the popped
    /// descriptor's `next` field; pass `SIZE` or any `>= SIZE` value to mean "the
    /// chain is now empty"). Returns `None` (and mutates nothing) when exhausted.
    ///
    /// This is the allocate half: the Frame reads `desc[head].next`, calls this
    /// with that value, then fills in `desc[head]`'s `addr`/`len`/`flags` for the
    /// new buffer. The returned index is always in `0..SIZE`.
    pub fn pop_free(&mut self, next_of_head: usize) -> Option<usize> {
        if self.free_head >= SIZE {
            return None;
        }
        let head = self.free_head;
        // Normalise any out-of-range "none" sentinel to exactly SIZE.
        self.free_head = if next_of_head >= SIZE { SIZE } else { next_of_head };
        self.free_count -= 1;
        // `free_head` is only ever set from `0..SIZE` (here) or `push_free`'s
        // masked index, so `head < SIZE` already; the `% SIZE` is belt-and-braces.
        Some(head % SIZE)
    }

    /// Push descriptor `desc` back onto the free chain as the new head. The Frame
    /// must have already written `desc`'s `next` field to [`free_next_target`]
    /// (the old head / none-sentinel) so the chain stays intact. `desc` is masked
    /// into `0..SIZE`. Saturates `free_count` at `SIZE` (a double-free of the same
    /// index is the Frame's bug, not this module's panic).
    pub fn push_free(&mut self, desc: usize) {
        let d = desc % SIZE;
        self.free_head = d;
        if self.free_count < SIZE {
            self.free_count += 1;
        }
    }

    /// The value descriptor `i`'s `next` field should be initialised to when the
    /// whole table is laid out as one free chain `0 -> 1 -> ... -> SIZE-1 -> none`:
    /// `i + 1` for `i < SIZE-1`, else `SIZE` (the none-sentinel). The Frame loops
    /// `0..SIZE` writing this into each `desc[i].next` at queue setup so the
    /// intrusive free chain `new()` assumes is actually present in memory.
    pub fn initial_free_next(i: usize) -> usize {
        if i + 1 < SIZE {
            i + 1
        } else {
            SIZE
        }
    }
}

impl<const SIZE: usize> Default for VirtQueue<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Host unit tests (run via the out-of-workspace tests-host harness; they
// `include!` this exact file, so they exercise the production arithmetic).
// Cover: descriptor-table index math, avail-ring head/idx publish, used-ring
// consume + wrap at SIZE and at 2^16, free-descriptor chain push/pop, and
// bounds-safety for the const SIZE (every offset stays inside its ring).
// ===========================================================================
#[cfg(test)]
mod virtqueue_tests {
    use super::*;

    /// A small queue used by most tests (power of two, as the spec requires).
    const N: usize = 4;

    #[test]
    fn ring_region_sizes_match_the_split_layout() {
        // Descriptor table: SIZE * 16.
        assert_eq!(VirtQueue::<N>::desc_table_size(), N * 16);
        // Avail ring: flags(2) + idx(2) + SIZE * 2.
        assert_eq!(VirtQueue::<N>::avail_ring_size(), 4 + N * 2);
        // Used ring: flags(2) + idx(2) + SIZE * 8.
        assert_eq!(VirtQueue::<N>::used_ring_size(), 4 + N * 8);
        // A larger queue scales linearly.
        assert_eq!(VirtQueue::<256>::desc_table_size(), 256 * 16);
        assert_eq!(VirtQueue::<256>::avail_ring_size(), 4 + 256 * 2);
        assert_eq!(VirtQueue::<256>::used_ring_size(), 4 + 256 * 8);
    }

    #[test]
    fn descriptor_field_offsets_are_within_the_table_and_correctly_spaced() {
        let q = VirtQueue::<N>::new();
        for i in 0..N {
            let base = q.desc_offset(i);
            assert_eq!(base, i * DESC_SIZE);
            assert_eq!(q.desc_addr_offset(i), base + 0);
            assert_eq!(q.desc_len_offset(i), base + 8);
            assert_eq!(q.desc_flags_offset(i), base + 12);
            assert_eq!(q.desc_next_offset(i), base + 14);
            // Every field lies strictly inside the descriptor table.
            assert!(q.desc_next_offset(i) + 2 <= VirtQueue::<N>::desc_table_size());
        }
    }

    #[test]
    fn descriptor_index_wraps_at_size() {
        let q = VirtQueue::<N>::new();
        // A chain `next` pointer past SIZE wraps back into the table.
        assert_eq!(q.desc_offset(N), q.desc_offset(0));
        assert_eq!(q.desc_offset(N + 1), q.desc_offset(1));
        assert_eq!(q.desc_offset(2 * N + 3), q.desc_offset(3));
    }

    #[test]
    fn avail_publish_advances_idx_and_targets_the_right_slot() {
        let mut q = VirtQueue::<N>::new();
        assert_eq!(q.avail_idx(), 0);
        // Publish heads 10, 11, 12 ... and check each lands in the next slot.
        let (slot0, h0) = q.publish_avail(10);
        assert_eq!(slot0, AVAIL_RING_OFF + 0 * AVAIL_RING_ENTRY_SIZE);
        assert_eq!(h0, 10);
        assert_eq!(q.avail_idx(), 1);

        let (slot1, h1) = q.publish_avail(11);
        assert_eq!(slot1, AVAIL_RING_OFF + 1 * AVAIL_RING_ENTRY_SIZE);
        assert_eq!(h1, 11);
        assert_eq!(q.avail_idx(), 2);

        // Every avail-ring slot is inside the avail ring.
        for idx in 0..(N as u16) {
            assert!(
                q.avail_ring_entry_offset(idx) + AVAIL_RING_ENTRY_SIZE
                    <= VirtQueue::<N>::avail_ring_size()
            );
        }
    }

    #[test]
    fn avail_ring_slot_wraps_at_size_while_idx_keeps_counting() {
        let mut q = VirtQueue::<N>::new();
        // Publish SIZE+1 entries; slot for the SIZE-th wraps back to slot 0.
        let mut slots = [0usize; 5];
        for (k, s) in slots.iter_mut().enumerate() {
            let (slot, _) = q.publish_avail(k as u16);
            *s = slot;
        }
        // slot[N] (the 5th publish on a size-4 queue) wraps to slot[0].
        assert_eq!(slots[N], slots[0]);
        // But the producer idx kept counting (no wrap at 4; only at 2^16).
        assert_eq!(q.avail_idx(), (N as u16) + 1);
    }

    #[test]
    fn avail_idx_wraps_at_2_pow_16() {
        let mut q = VirtQueue::<N>::new();
        // Force the producer index up against the 16-bit boundary.
        for _ in 0..u16::MAX {
            q.publish_avail(0);
        }
        assert_eq!(q.avail_idx(), u16::MAX);
        // The next publish wraps 0xFFFF -> 0x0000.
        q.publish_avail(0);
        assert_eq!(q.avail_idx(), 0);
    }

    #[test]
    fn used_offsets_are_within_the_used_ring_and_correctly_spaced() {
        let q = VirtQueue::<N>::new();
        for idx in 0..(N as u16) {
            let base = q.used_ring_entry_offset(idx);
            assert_eq!(base, USED_RING_OFF + (idx as usize) * USED_RING_ENTRY_SIZE);
            assert_eq!(q.used_elem_id_offset(idx), base + 0);
            assert_eq!(q.used_elem_len_offset(idx), base + 4);
            assert!(base + USED_RING_ENTRY_SIZE <= VirtQueue::<N>::used_ring_size());
        }
        // Used slot wraps at SIZE just like avail.
        assert_eq!(q.used_ring_entry_offset(N as u16), q.used_ring_entry_offset(0));
    }

    #[test]
    fn used_available_counts_new_entries_across_wrap() {
        let q = VirtQueue::<N>::new();
        // No entries yet.
        assert_eq!(q.used_available(0), 0);
        // Device published 3 entries.
        assert_eq!(q.used_available(3), 3);
        // Wrapping case: consumer at default 0, device idx near the top then
        // wrapped — a fresh queue with used_last=0 and device idx=2 sees 2.
        let mut q2 = VirtQueue::<N>::new();
        // Advance our cursor to 0xFFFF by taking entries (device always ahead).
        for _ in 0..u16::MAX {
            assert!(q2.take_used(u16::MAX, 0, 0).is_some());
        }
        assert_eq!(q2.used_last(), u16::MAX);
        // Device wraps to 1 (published one more across the boundary).
        assert_eq!(q2.used_available(1), 2, "0xFFFF -> 0x0001 spans 2 entries");
    }

    #[test]
    fn take_used_consumes_in_order_and_advances_cursor() {
        let mut q = VirtQueue::<N>::new();
        // Device says 2 entries ready; consume both with their (id,len).
        let e0 = q.take_used(2, 3, 100).expect("first used entry");
        assert_eq!(e0, UsedElem { id: 3, len: 100 });
        assert_eq!(q.used_last(), 1);

        let e1 = q.take_used(2, 1, 64).expect("second used entry");
        assert_eq!(e1, UsedElem { id: 1, len: 64 });
        assert_eq!(q.used_last(), 2);

        // No third entry available; cursor does not move.
        assert_eq!(q.take_used(2, 0, 0), None);
        assert_eq!(q.used_last(), 2);
    }

    #[test]
    fn next_used_offset_points_at_the_pending_slot_without_advancing() {
        let mut q = VirtQueue::<N>::new();
        // Nothing pending -> None.
        assert_eq!(q.next_used_offset(0), None);
        // One pending -> the slot for used_last (0), cursor unchanged.
        assert_eq!(q.next_used_offset(1), Some(q.used_ring_entry_offset(0)));
        assert_eq!(q.used_last(), 0, "next_used_offset must not advance");
        // Consume it, then the next pending slot is slot 1.
        q.take_used(2, 0, 0).unwrap();
        assert_eq!(q.next_used_offset(2), Some(q.used_ring_entry_offset(1)));
    }

    #[test]
    fn used_ring_slot_wraps_at_size_during_consume() {
        let mut q = VirtQueue::<N>::new();
        // Consume SIZE+1 entries; the (SIZE)-th lands in slot 0 again.
        let off0 = q.used_ring_entry_offset(q.used_last());
        for _ in 0..N {
            q.take_used(u16::MAX, 0, 0).unwrap();
        }
        // After SIZE consumes, used_last == SIZE, whose slot wraps to slot 0.
        assert_eq!(q.used_last() as usize, N);
        let off_wrapped = q.used_ring_entry_offset(q.used_last());
        assert_eq!(off_wrapped, off0, "used slot wraps at SIZE");
    }

    #[test]
    fn fresh_queue_has_all_descriptors_free() {
        let q = VirtQueue::<N>::new();
        assert_eq!(q.free_count(), N);
        assert!(!q.is_exhausted());
        assert_eq!(q.free_head(), Some(0));
        assert_eq!(q.size(), N);
    }

    #[test]
    fn initial_free_next_threads_the_whole_table() {
        // 0 -> 1 -> 2 -> 3 -> none(SIZE) for a size-4 queue.
        assert_eq!(VirtQueue::<N>::initial_free_next(0), 1);
        assert_eq!(VirtQueue::<N>::initial_free_next(1), 2);
        assert_eq!(VirtQueue::<N>::initial_free_next(2), 3);
        assert_eq!(VirtQueue::<N>::initial_free_next(N - 1), N, "last points at none-sentinel");
    }

    #[test]
    fn pop_free_walks_the_initial_chain_then_exhausts() {
        let mut q = VirtQueue::<N>::new();
        // The Frame, popping head h, reads desc[h].next == initial_free_next(h).
        // head=0, next=1
        assert_eq!(q.pop_free(VirtQueue::<N>::initial_free_next(0)), Some(0));
        assert_eq!(q.free_count(), N - 1);
        assert_eq!(q.free_head(), Some(1));
        // head=1, next=2
        assert_eq!(q.pop_free(VirtQueue::<N>::initial_free_next(1)), Some(1));
        // head=2, next=3
        assert_eq!(q.pop_free(VirtQueue::<N>::initial_free_next(2)), Some(2));
        // head=3, next=SIZE (none)
        assert_eq!(q.pop_free(VirtQueue::<N>::initial_free_next(3)), Some(3));
        // Now exhausted.
        assert!(q.is_exhausted());
        assert_eq!(q.free_count(), 0);
        assert_eq!(q.free_head(), None);
        // A pop on an exhausted chain yields None and mutates nothing.
        assert_eq!(q.pop_free(0), None);
        assert_eq!(q.free_count(), 0);
        assert_eq!(q.free_head(), None);
    }

    #[test]
    fn push_free_returns_descriptors_as_the_new_head() {
        let mut q = VirtQueue::<N>::new();
        // Drain the whole chain.
        let mut next = VirtQueue::<N>::initial_free_next(0);
        while q.pop_free(next).is_some() {
            // The Frame would read the new head's `next`; emulate with the
            // initial threading for whatever head is now current.
            next = match q.free_head() {
                Some(h) => VirtQueue::<N>::initial_free_next(h),
                None => N,
            };
        }
        assert!(q.is_exhausted());

        // Push descriptor 2 back: it becomes the head, count 1.
        // The Frame first writes desc[2].next = free_next_target() (== N here).
        assert_eq!(q.free_next_target(), N, "empty chain -> none-sentinel");
        q.push_free(2);
        assert_eq!(q.free_head(), Some(2));
        assert_eq!(q.free_count(), 1);

        // Push descriptor 0: desc[0].next must be set to the old head (2).
        assert_eq!(q.free_next_target(), 2);
        q.push_free(0);
        assert_eq!(q.free_head(), Some(0));
        assert_eq!(q.free_count(), 2);

        // Pop them back in LIFO order: 0 (with next=2), then 2 (with next=N).
        assert_eq!(q.pop_free(2), Some(0));
        assert_eq!(q.pop_free(N), Some(2));
        assert!(q.is_exhausted());
    }

    #[test]
    fn push_free_masks_index_and_saturates_count() {
        let mut q = VirtQueue::<N>::new();
        // A full queue: pushing more never exceeds SIZE.
        // (Start full; push an in-range index — count stays clamped at SIZE.)
        assert_eq!(q.free_count(), N);
        q.push_free(0);
        assert_eq!(q.free_count(), N, "free_count saturates at SIZE");
        // An out-of-range desc index is masked into 0..SIZE (N + 1 -> 1).
        q.push_free(N + 1);
        assert_eq!(q.free_head(), Some(1));
    }

    #[test]
    fn pop_then_push_round_trips_a_full_alloc_free_cycle() {
        let mut q = VirtQueue::<2>::new();
        // Allocate both descriptors (size-2 queue: 0 -> 1 -> none).
        let a = q.pop_free(VirtQueue::<2>::initial_free_next(0)).unwrap();
        let b = q.pop_free(VirtQueue::<2>::initial_free_next(1)).unwrap();
        assert_eq!((a, b), (0, 1));
        assert!(q.is_exhausted());
        // Publish each as an avail head, then complete + free them.
        q.publish_avail(a as u16);
        q.publish_avail(b as u16);
        assert_eq!(q.avail_idx(), 2);
        // Device completes b then a (out of order is legal).
        let ub = q.take_used(2, b as u16, 8).unwrap();
        let ua = q.take_used(2, a as u16, 16).unwrap();
        assert_eq!(ub.id, b as u16);
        assert_eq!(ua.id, a as u16);
        // Free both back (Frame writes each desc.next to free_next_target first).
        q.push_free(ub.id as usize);
        q.push_free(ua.id as usize);
        assert_eq!(q.free_count(), 2);
        assert!(!q.is_exhausted());
    }

    #[test]
    fn all_offsets_stay_in_bounds_for_a_large_power_of_two_size() {
        // Bounds-safety for the const SIZE: pick the spec's max-ish size and
        // assert every per-element offset stays inside its region.
        const BIG: usize = 1024;
        let q = VirtQueue::<BIG>::new();
        // Last descriptor's last field is inside the table.
        assert!(q.desc_next_offset(BIG - 1) + 2 <= VirtQueue::<BIG>::desc_table_size());
        // Last avail slot inside the avail ring.
        assert!(
            q.avail_ring_entry_offset((BIG - 1) as u16) + AVAIL_RING_ENTRY_SIZE
                <= VirtQueue::<BIG>::avail_ring_size()
        );
        // Last used elem inside the used ring.
        assert!(
            q.used_ring_entry_offset((BIG - 1) as u16) + USED_RING_ENTRY_SIZE
                <= VirtQueue::<BIG>::used_ring_size()
        );
        assert_eq!(q.free_count(), BIG);
    }
}
