//! # `frame` — safe abstractions over the arch HAL
//!
//! The framekernel is split into two halves:
//!
//!   * the **arch backends** (`arch-aarch64`, `arch-x86_64`) — the `unsafe`
//!     code that touches real hardware, behind the [`hal`] traits; and
//!   * **this crate + the safe kernel** — everything above, which is `unsafe`-
//!     free.
//!
//! `frame` is the seam: it consumes the safe [`hal`] API and offers the safe
//! kernel higher-level services (memory regions, the heap, trap/IRQ
//! orchestration). At this skeleton stage it defines the *shapes* of those
//! services; the implementations grow alongside the arch backends.
//!
//! Because the dangerous work is already encapsulated behind `hal`, this crate
//! can — and does — forbid `unsafe` entirely.
#![no_std]
#![forbid(unsafe_code)]

// NOTE: `extern crate alloc;` is intentionally NOT enabled yet. Pulling in
// `alloc` requires a registered `#[global_allocator]`, which in turn needs a
// live RAM region from the arch backend. That heap wiring is a later milestone
// (see `HeapPlan` below and ALLOWED_CRATES.md); enabling `alloc` before then
// would force a placeholder allocator into the skeleton.

use hal::MemoryRegion;

/// Bookkeeping for the kernel heap region.
///
/// The actual global allocator wiring (installing a `#[global_allocator]` over
/// a real RAM range) requires `unsafe` and a live [`MemoryRegion`] from the
/// arch backend, so it is deferred to bring-up. This type holds the safe,
/// inspectable description of where the heap will live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapPlan {
    region: MemoryRegion,
}

impl HeapPlan {
    /// Record the region the heap allocator will manage.
    pub const fn new(region: MemoryRegion) -> Self {
        Self { region }
    }

    /// The region backing the heap.
    pub const fn region(&self) -> MemoryRegion {
        self.region
    }

    /// Total heap capacity in bytes.
    pub const fn capacity(&self) -> usize {
        self.region.size
    }
}

/// Categories of CPU trap the safe kernel may want to reason about. The arch
/// backend maps real exception/interrupt sources onto these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    /// A hardware interrupt (IRQ).
    Irq,
    /// A synchronous CPU fault (page fault, illegal instruction, ...).
    Synchronous,
    /// A timer tick.
    Timer,
}

/// A safe summary of a trap, handed up from the Frame to the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapInfo {
    /// What kind of trap occurred.
    pub kind: TrapKind,
}

impl TrapInfo {
    /// Construct a trap summary.
    pub const fn new(kind: TrapKind) -> Self {
        Self { kind }
    }
}
