//! Time shapes: a monotonic [`Clock`] and a **deadline-shaped** [`Timer`].
//!
//! The deadline shape is a consensus Q1 non-negotiable: the trait signature is
//! `now_ns()` / `set_deadline_ns(ns)` / `cancel()`, NOT a periodic
//! `set_timer(ticks)`. On hardware without a deadline timer the arch backend
//! emulates one-shots from a periodic source (the fallback body, P4), but the
//! *signature* never has to change. The legacy periodic [`crate::TimerApi`] is
//! retained untouched as the existing bring-up path. Sealed.

use crate::sealed::Sealed;
use crate::ArchError;

/// A monotonic, never-decreasing clock reported in nanoseconds.
///
/// Backed by the invariant TSC / kvm-clock on x86 or `CNTVCT_EL0` on aarch64.
/// The nanosecond unit is the arch-neutral currency the vDSO `clock_gettime`
/// page (roadmap P3) and the scheduler share.
pub trait Clock: Sealed {
    /// Current monotonic time in nanoseconds since an arbitrary fixed epoch.
    fn now_ns(&self) -> u64;
}

/// A one-shot, deadline-programmed timer.
///
/// `set_deadline_ns` arms a single interrupt for an **absolute** monotonic
/// nanosecond instant (comparable to [`Clock::now_ns`]); `cancel` disarms a
/// pending deadline. This is the x86 TSC-deadline / aarch64 ECV shape; the
/// fallback body programs the nearest periodic tick that covers the deadline.
pub trait Timer: Sealed {
    /// Arm a one-shot interrupt for the absolute monotonic instant
    /// `deadline_ns`. A deadline already in the past should fire promptly.
    fn set_deadline_ns(&mut self, deadline_ns: u64) -> Result<(), ArchError>;

    /// Disarm any pending deadline. Idempotent: cancelling when nothing is
    /// armed is a no-op.
    fn cancel(&mut self);
}
