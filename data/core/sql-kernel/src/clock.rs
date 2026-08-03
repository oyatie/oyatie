//! Hybrid logical clock behind the TrueTime-shaped [`ClockSource`] trait.
//!
//! Precedent (ADR-0536 D-10): Google Spanner TrueTime exposes time as an
//! `[earliest, latest]` uncertainty interval; CockroachDB and TiKV order
//! transactions with hybrid logical clocks (Kulkarni et al. HLC) bounded by a
//! configured max clock offset. The trait carries the interval shape so a
//! TrueTime-class hardware clock slots in at W5 without any API change, while
//! the [`Hlc`] state machine is the software implementation the transitional
//! engines already prove in production.
//!
//! Pure kernel: physical time is always injected by the caller, never read
//! from the OS, so every ordering property is unit-testable.

use serde::{Deserialize, Serialize};

/// Errors the clock kernel can surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClockError {
    /// A remote timestamp is further ahead of injected physical time than the
    /// configured max offset allows (CockroachDB MaxOffset discipline: the
    /// node must not silently absorb unbounded drift).
    DriftExceeded {
        remote_wall_nanos: u64,
        physical_now_nanos: u64,
        max_offset_nanos: u64,
    },
    /// The logical counter would overflow without physical time advancing.
    LogicalOverflow { wall_nanos: u64 },
    /// A bound was constructed with `earliest` after `latest`.
    InvalidBound {
        earliest: HlcTimestamp,
        latest: HlcTimestamp,
    },
}

impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DriftExceeded {
                remote_wall_nanos,
                physical_now_nanos,
                max_offset_nanos,
            } => write!(
                f,
                "remote wall clock {remote_wall_nanos}ns exceeds local physical time \
                 {physical_now_nanos}ns by more than max offset {max_offset_nanos}ns"
            ),
            Self::LogicalOverflow { wall_nanos } => write!(
                f,
                "logical counter overflow at wall {wall_nanos}ns without physical advance"
            ),
            Self::InvalidBound { earliest, latest } => write!(
                f,
                "clock bound earliest {earliest:?} is after latest {latest:?}"
            ),
        }
    }
}

impl std::error::Error for ClockError {}

/// A hybrid logical timestamp: physical wall nanos plus a logical counter
/// that breaks ties when physical time stands still. Total order is
/// `(wall_nanos, logical)` lexicographic, matching CockroachDB/TiKV HLC.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct HlcTimestamp {
    pub wall_nanos: u64, // data_class: INTERNAL_ONLY
    pub logical: u32,    // data_class: INTERNAL_ONLY
}

impl HlcTimestamp {
    #[must_use]
    pub fn new(wall_nanos: u64, logical: u32) -> Self {
        Self {
            wall_nanos,
            logical,
        }
    }

    /// The zero timestamp (before every real event).
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }
}

/// A TrueTime-shaped uncertainty interval: the true time is guaranteed to be
/// within `[earliest, latest]`. Software HLC produces a bound whose width is
/// the configured max offset; TrueTime-class hardware narrows it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockBound {
    pub earliest: HlcTimestamp, // data_class: INTERNAL_ONLY
    pub latest: HlcTimestamp,   // data_class: INTERNAL_ONLY
}

impl ClockBound {
    pub fn new(earliest: HlcTimestamp, latest: HlcTimestamp) -> Result<Self, ClockError> {
        if earliest > latest {
            return Err(ClockError::InvalidBound { earliest, latest });
        }
        Ok(Self { earliest, latest })
    }

    /// Spanner commit-wait shape: this interval is wholly before `other`
    /// (no overlap), so an event stamped from `self` definitely happened
    /// before one stamped from `other`.
    #[must_use]
    pub fn definitely_before(&self, other: &Self) -> bool {
        self.latest < other.earliest
    }

    /// Whether the two uncertainty intervals overlap (ordering between events
    /// stamped from overlapping bounds is not externally observable).
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        !self.definitely_before(other) && !other.definitely_before(self)
    }
}

/// The owned clock port (ADR-0536 D-10). Implementations: [`Hlc`] wrapped
/// with injected physical time (transitional engines), TrueTime-class
/// hardware at W5. The trait shape does not change at cutover.
pub trait ClockSource {
    fn now_bound(&mut self) -> Result<ClockBound, ClockError>;
}

/// Hybrid logical clock state machine (Kulkarni et al., as deployed by
/// CockroachDB/TiKV). Pure: callers inject physical time on every event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hlc {
    wall_nanos: u64,
    logical: u32,
    max_offset_nanos: u64,
}

impl Hlc {
    #[must_use]
    pub fn new(max_offset_nanos: u64) -> Self {
        Self {
            wall_nanos: 0,
            logical: 0,
            max_offset_nanos,
        }
    }

    #[must_use]
    pub fn max_offset_nanos(&self) -> u64 {
        self.max_offset_nanos
    }

    /// The last timestamp handed out (zero before any event).
    #[must_use]
    pub fn current(&self) -> HlcTimestamp {
        HlcTimestamp::new(self.wall_nanos, self.logical)
    }

    /// Local/send event: advance the clock with injected physical time and
    /// return a timestamp strictly greater than every prior one.
    pub fn tick(&mut self, physical_now_nanos: u64) -> Result<HlcTimestamp, ClockError> {
        if physical_now_nanos > self.wall_nanos {
            self.wall_nanos = physical_now_nanos;
            self.logical = 0;
        } else {
            self.logical = self
                .logical
                .checked_add(1)
                .ok_or(ClockError::LogicalOverflow {
                    wall_nanos: self.wall_nanos,
                })?;
        }
        Ok(self.current())
    }

    /// Receive event: merge a remote timestamp, rejecting remote wall clocks
    /// beyond the max-offset drift bound. Returns a timestamp strictly
    /// greater than both the local clock and the remote timestamp.
    pub fn observe(
        &mut self,
        remote: HlcTimestamp,
        physical_now_nanos: u64,
    ) -> Result<HlcTimestamp, ClockError> {
        let drift_ceiling = physical_now_nanos.saturating_add(self.max_offset_nanos);
        if remote.wall_nanos > drift_ceiling {
            return Err(ClockError::DriftExceeded {
                remote_wall_nanos: remote.wall_nanos,
                physical_now_nanos,
                max_offset_nanos: self.max_offset_nanos,
            });
        }
        let new_wall = self.wall_nanos.max(remote.wall_nanos).max(physical_now_nanos);
        let logical = if new_wall == self.wall_nanos && new_wall == remote.wall_nanos {
            self.logical.max(remote.logical).checked_add(1)
        } else if new_wall == self.wall_nanos {
            self.logical.checked_add(1)
        } else if new_wall == remote.wall_nanos {
            remote.logical.checked_add(1)
        } else {
            Some(0)
        }
        .ok_or(ClockError::LogicalOverflow {
            wall_nanos: new_wall,
        })?;
        self.wall_nanos = new_wall;
        self.logical = logical;
        Ok(self.current())
    }

    /// Produce a TrueTime-shaped bound around a fresh local tick: the true
    /// time is within `[tick, tick.wall + max_offset]`.
    pub fn now_bound(&mut self, physical_now_nanos: u64) -> Result<ClockBound, ClockError> {
        let earliest = self.tick(physical_now_nanos)?;
        let latest = HlcTimestamp::new(
            earliest.wall_nanos.saturating_add(self.max_offset_nanos),
            earliest.logical,
        );
        ClockBound::new(earliest, latest)
    }
}

/// Test/reference clock source returning a fixed bound on every call, so
/// contract tests can pin commit timestamps deterministically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedClockSource {
    bound: ClockBound,
}

impl FixedClockSource {
    #[must_use]
    pub fn new(bound: ClockBound) -> Self {
        Self { bound }
    }
}

impl ClockSource for FixedClockSource {
    fn now_bound(&mut self) -> Result<ClockBound, ClockError> {
        Ok(self.bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_OFFSET: u64 = 500_000_000; // 500ms, CockroachDB default

    #[test]
    fn timestamps_order_by_wall_then_logical() {
        let a = HlcTimestamp::new(1, 5);
        let b = HlcTimestamp::new(2, 0);
        let c = HlcTimestamp::new(2, 1);
        assert!(a < b && b < c);
        assert_eq!(HlcTimestamp::zero(), HlcTimestamp::new(0, 0));
    }

    #[test]
    fn timestamp_round_trips_and_rejects_unknown_fields() {
        let ts = HlcTimestamp::new(42, 7);
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(serde_json::from_str::<HlcTimestamp>(&json).unwrap(), ts);
        let err = serde_json::from_str::<HlcTimestamp>(
            r#"{"wall_nanos":42,"logical":7,"surprise":1}"#,
        );
        assert!(err.is_err());
    }

    #[test]
    fn bound_rejects_inverted_interval() {
        let early = HlcTimestamp::new(10, 0);
        let late = HlcTimestamp::new(20, 0);
        assert!(ClockBound::new(late, early).is_err());
        let bound = ClockBound::new(early, late).unwrap();
        assert_eq!(bound.earliest, early);
    }

    #[test]
    fn definitely_before_requires_disjoint_intervals() {
        let a = ClockBound::new(HlcTimestamp::new(0, 0), HlcTimestamp::new(10, 0)).unwrap();
        let b = ClockBound::new(HlcTimestamp::new(11, 0), HlcTimestamp::new(20, 0)).unwrap();
        let c = ClockBound::new(HlcTimestamp::new(5, 0), HlcTimestamp::new(15, 0)).unwrap();
        assert!(a.definitely_before(&b));
        assert!(!a.definitely_before(&c));
        assert!(a.overlaps(&c) && c.overlaps(&b));
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn tick_is_strictly_monotonic_even_when_physical_time_stalls() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        let mut prev = HlcTimestamp::zero();
        // Property loop: physical time repeats and even regresses; every
        // handed-out timestamp must still strictly increase.
        for physical in [5u64, 5, 5, 3, 9, 9, 2, 10] {
            let ts = hlc.tick(physical).unwrap();
            assert!(ts > prev, "tick({physical}) produced {ts:?} <= {prev:?}");
            prev = ts;
        }
    }

    #[test]
    fn tick_resets_logical_when_physical_advances() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        hlc.tick(5).unwrap();
        hlc.tick(5).unwrap();
        let advanced = hlc.tick(6).unwrap();
        assert_eq!(advanced, HlcTimestamp::new(6, 0));
    }

    #[test]
    fn observe_outputs_exceed_both_local_and_remote() {
        // Property loop over wall-clock orderings: result > max(local, remote).
        for (local_phys, remote) in [
            (10u64, HlcTimestamp::new(5, 9)),
            (10, HlcTimestamp::new(10, 0)),
            (10, HlcTimestamp::new(15, 3)),
            (1, HlcTimestamp::new(1, 0)),
        ] {
            let mut hlc = Hlc::new(MAX_OFFSET);
            hlc.tick(local_phys).unwrap();
            let local_before = hlc.current();
            let ts = hlc.observe(remote, local_phys).unwrap();
            assert!(
                ts > local_before && ts > remote,
                "observe({remote:?}) at {local_phys} gave {ts:?}"
            );
        }
    }

    #[test]
    fn observe_merges_equal_walls_by_max_logical_plus_one() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        hlc.tick(10).unwrap();
        let ts = hlc.observe(HlcTimestamp::new(10, 7), 10).unwrap();
        assert_eq!(ts, HlcTimestamp::new(10, 8));
    }

    #[test]
    fn observe_rejects_remote_beyond_drift_bound() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        let too_far = HlcTimestamp::new(MAX_OFFSET + 2, 0);
        let err = hlc.observe(too_far, 1).unwrap_err();
        assert!(matches!(err, ClockError::DriftExceeded { .. }));
        // The drift rejection must not corrupt clock state.
        assert_eq!(hlc.current(), HlcTimestamp::zero());
    }

    #[test]
    fn observe_at_exact_drift_ceiling_is_accepted() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        let at_ceiling = HlcTimestamp::new(MAX_OFFSET + 1, 0);
        assert!(hlc.observe(at_ceiling, 1).is_ok());
    }

    #[test]
    fn logical_overflow_is_a_typed_error_not_a_wrap() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        hlc.tick(5).unwrap();
        hlc.observe(HlcTimestamp::new(5, u32::MAX), 5).unwrap_err();
        // tick at stalled physical time after maxing logical also errors.
        let mut stalled = Hlc::new(MAX_OFFSET);
        stalled.tick(5).unwrap();
        let almost = HlcTimestamp::new(5, u32::MAX - 1);
        stalled.observe(almost, 5).unwrap();
        let err = stalled.tick(5).unwrap_err();
        assert!(matches!(err, ClockError::LogicalOverflow { .. }));
    }

    #[test]
    fn now_bound_width_is_max_offset() {
        let mut hlc = Hlc::new(MAX_OFFSET);
        let bound = hlc.now_bound(1_000).unwrap();
        assert_eq!(bound.earliest, HlcTimestamp::new(1_000, 0));
        assert_eq!(bound.latest.wall_nanos, 1_000 + MAX_OFFSET);
        assert!(!bound.definitely_before(&bound));
    }

    #[test]
    fn fixed_clock_source_returns_pinned_bound() {
        let bound =
            ClockBound::new(HlcTimestamp::new(1, 0), HlcTimestamp::new(2, 0)).unwrap();
        let mut source = FixedClockSource::new(bound);
        assert_eq!(source.now_bound().unwrap(), bound);
        assert_eq!(source.now_bound().unwrap(), bound);
    }
}
