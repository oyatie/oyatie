//! # `hal` — Hardware Abstraction Layer (the arch trait)
//!
//! This crate is the contract between the architecture-specific *Frame* (the
//! tiny `unsafe` core that touches real hardware) and everything above it.
//!
//! It contains **only trait definitions and plain data types** — no `unsafe`,
//! no register pokes, no inline assembly. Each supported architecture
//! (`arch-aarch64`, `arch-x86_64`) provides a type implementing [`Arch`], and
//! the safe kernel is written purely against these traits. That is the whole
//! point of the framekernel design: the dangerous code lives behind a narrow,
//! audited, *safe* API.
#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

// ---------------------------------------------------------------------------
// P0 HAL reshape — the full sealed dual-arch capability set.
//
// These modules are **purely additive**: new trait/type *shapes* that nothing
// implements yet (so the build stays green), defined alongside the original
// four traits below. They are the target dual-arch surface from ROADMAP P0; the
// arch backends are rewired onto them in a separate, diff-oracle-verified
// follow-on increment. Everything stays `#![forbid(unsafe_code)]`.
//
// `sealed` is `#[doc(hidden)] pub mod` — the standard "sealed-to-downstream,
// open-to-this-workspace, hidden-from-docs" idiom. In-workspace arch backends
// name the supertrait as `hal::sealed::Sealed` so they can write the real
// `impl Trait for ..` blocks for the sealed capability traits; the seal still
// holds against the external ecosystem because the trait is unnameable in
// rendered docs and only intended for this workspace's Frame backends. (A truly
// private `mod sealed;` would block even the in-workspace impls — `E0603`.)
#[doc(hidden)]
pub mod sealed;

pub mod confidential;
pub mod cpu;
pub mod irq;
pub mod mm;
pub mod qos;
pub mod smp;
pub mod time;
pub mod virtio;

/// Errors that an architecture backend may report from a fallible operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchError {
    /// The requested feature is not implemented on this architecture yet.
    Unsupported,
    /// A memory operation failed (bad range, exhausted region, etc.).
    Memory,
    /// An interrupt/timer operation failed.
    Interrupt,
}

/// A byte-oriented console sink (typically a UART).
///
/// Implementations must be safe to call from the safe kernel. The blanket
/// [`fmt::Write`] support gives `write!`/`writeln!` for free.
pub trait ConsoleWrite {
    /// Emit a single byte to the console.
    fn write_byte(&mut self, byte: u8);

    /// Emit a UTF-8 string to the console. Default implementation writes byte
    /// by byte; backends may override for efficiency.
    fn write_str(&mut self, s: &str) {
        for b in s.as_bytes() {
            self.write_byte(*b);
        }
    }
}

/// Description of a contiguous region of usable physical RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    /// Physical start address of the region.
    pub start: usize,
    /// Length of the region in bytes.
    pub size: usize,
}

impl MemoryRegion {
    /// Construct a region from a start address and size.
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    /// One-past-the-end physical address of the region.
    pub const fn end(&self) -> usize {
        self.start + self.size
    }
}

/// Memory management primitives the arch backend must expose to the Frame.
pub trait MemoryApi {
    /// Initialize the MMU / page tables and return the primary usable RAM
    /// region the heap allocator may carve from.
    fn init_memory(&mut self) -> Result<MemoryRegion, ArchError>;
}

/// Interrupt-controller primitives.
pub trait InterruptApi {
    /// Globally enable IRQ delivery to the CPU.
    fn enable_irq(&mut self);

    /// Globally disable IRQ delivery to the CPU.
    fn disable_irq(&mut self);
}

/// Monotonic timer primitives used for scheduling and timekeeping.
pub trait TimerApi {
    /// Frequency of the monotonic counter in Hz.
    fn timer_frequency(&self) -> u64;

    /// Current value of the monotonic counter (ticks).
    fn timer_now(&self) -> u64;

    /// Program a one-shot timer interrupt `ticks` from now.
    fn set_timer(&mut self, ticks: u64) -> Result<(), ArchError>;
}

/// The umbrella trait an architecture backend implements. It composes the
/// console, memory, interrupt, and timer sub-APIs into a single object the
/// Frame can hold. Keeping it as a supertrait bundle lets the safe kernel
/// depend on exactly the capabilities it needs.
pub trait Arch: MemoryApi + InterruptApi + TimerApi {
    /// The concrete console type for this architecture.
    type Console: ConsoleWrite;

    /// Human-readable architecture name (e.g. "aarch64").
    fn name(&self) -> &'static str;

    /// Borrow the platform console.
    fn console(&mut self) -> &mut Self::Console;

    /// Halt the CPU forever (low-power wait loop). Never returns.
    fn halt(&self) -> !;
}

/// Adapter so any [`ConsoleWrite`] can be used with `core::write!`.
pub struct FmtConsole<'a, C: ConsoleWrite>(pub &'a mut C);

impl<C: ConsoleWrite> fmt::Write for FmtConsole<'_, C> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        ConsoleWrite::write_str(self.0, s);
        Ok(())
    }
}
