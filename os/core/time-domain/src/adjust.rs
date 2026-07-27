//! Clock discipline: deciding whether to *slew* (gradually adjust frequency) or
//! *step* (jump the clock) given a measured offset.
//!
//! This mirrors the adjtimex/step decision in `siderolabs/talos`'s time
//! controllers, which in turn follows ntpd convention: small offsets are slewed
//! smoothly, while offsets beyond a step threshold are applied as a hard jump.

/// Offsets at or below this many milliseconds are slewed rather than stepped.
/// ntpd's default step threshold is 128 ms.
pub const STEP_THRESHOLD_MS: i64 = 128;

/// Maximum slew rate: how many milliseconds of correction we apply per poll
/// tick when slewing. Models the kernel's bounded frequency adjustment.
pub const MAX_SLEW_MS_PER_TICK: i64 = 16;

/// How the clock should be disciplined for a given offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustMode {
    /// No correction needed (offset within the dead-band).
    Hold,
    /// Gradually adjust by the contained number of milliseconds this tick.
    Slew(i64),
    /// Jump the clock by the contained number of milliseconds immediately.
    Step(i64),
}

/// A computed clock adjustment derived from a measured offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockAdjustment {
    /// The measured offset (local - reference) in milliseconds.
    pub offset_ms: i64,
    /// The discipline decision.
    pub mode: AdjustMode,
}

impl ClockAdjustment {
    /// Decide how to discipline the clock for `offset_ms`.
    ///
    /// `offset_ms` is the RFC 5905 clock offset of the reference relative to the
    /// local clock: a *positive* offset means the local clock is *behind* and
    /// must move *forward*. The correction therefore carries the *same* sign as
    /// the offset.
    ///
    /// * `|offset| <= dead_band_ms` => [`AdjustMode::Hold`].
    /// * `dead_band_ms < |offset| <= STEP_THRESHOLD_MS` => [`AdjustMode::Slew`],
    ///   bounded by [`MAX_SLEW_MS_PER_TICK`].
    /// * `|offset| > STEP_THRESHOLD_MS` => [`AdjustMode::Step`] (full jump).
    pub fn decide(offset_ms: i64, dead_band_ms: i64) -> Self {
        let abs = offset_ms.unsigned_abs();
        let mode = if abs <= dead_band_ms.unsigned_abs() {
            AdjustMode::Hold
        } else if abs <= STEP_THRESHOLD_MS as u64 {
            // Correction moves the clock toward the reference, capped at the
            // slew rate.
            let correction = offset_ms.clamp(-MAX_SLEW_MS_PER_TICK, MAX_SLEW_MS_PER_TICK);
            AdjustMode::Slew(correction)
        } else {
            AdjustMode::Step(offset_ms)
        };
        ClockAdjustment { offset_ms, mode }
    }

    /// Whether this adjustment is a hard step (which resets sync confidence and
    /// is logged as a notable event by Talos).
    pub fn is_step(&self) -> bool {
        matches!(self.mode, AdjustMode::Step(_))
    }

    /// The signed correction this adjustment applies this tick (0 for Hold).
    pub fn correction_ms(&self) -> i64 {
        match self.mode {
            AdjustMode::Hold => 0,
            AdjustMode::Slew(v) | AdjustMode::Step(v) => v,
        }
    }

    /// Apply the correction to a current clock reading, returning the new
    /// Unix-millis value after this tick.
    pub fn apply(&self, clock_unix_millis: i64) -> i64 {
        clock_unix_millis + self.correction_ms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_offset_holds() {
        let adj = ClockAdjustment::decide(5, 10);
        assert_eq!(adj.mode, AdjustMode::Hold);
        assert_eq!(adj.correction_ms(), 0);
        assert!(!adj.is_step());
    }

    #[test]
    fn medium_offset_slews_capped_and_signed() {
        // offset +50 ms (local behind) => correction +16 ms (capped), forward.
        let adj = ClockAdjustment::decide(50, 10);
        assert_eq!(adj.mode, AdjustMode::Slew(16));
        // negative offset (local ahead) slews backward
        let adj2 = ClockAdjustment::decide(-50, 10);
        assert_eq!(adj2.mode, AdjustMode::Slew(-16));
    }

    #[test]
    fn large_offset_steps_full_correction() {
        // local is 5000 ms behind => step the clock forward by 5000.
        let adj = ClockAdjustment::decide(5_000, 10);
        assert_eq!(adj.mode, AdjustMode::Step(5_000));
        assert!(adj.is_step());
        // applying the step corrects the clock fully
        assert_eq!(adj.apply(1_000_000), 1_005_000);
    }

    #[test]
    fn step_threshold_boundary_is_slew() {
        // exactly at the step threshold => still slewed, not stepped.
        let adj = ClockAdjustment::decide(STEP_THRESHOLD_MS, 0);
        assert!(matches!(adj.mode, AdjustMode::Slew(_)));
        // one past => step.
        let adj2 = ClockAdjustment::decide(STEP_THRESHOLD_MS + 1, 0);
        assert!(adj2.is_step());
    }
}
