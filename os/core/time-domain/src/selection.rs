//! NTP sample filtering and clock selection, mirroring the sample-quality logic
//! in `siderolabs/talos`'s `pkg/ntp` (which in turn follows RFC 5905 §10's clock
//! filter and §11's jitter/dispersion accounting).
//!
//! A raw [`crate::sntp::NtpMeasurement`] is noisy: the round-trip delay varies
//! per packet and a single sample can be badly skewed by a queueing spike. Talos
//! keeps a small shift-register of recent samples per server and, on each poll,
//! selects the sample with the *lowest delay* as the best offset estimate, while
//! computing the *jitter* (RMS difference of the other samples from the best)
//! and the peer *dispersion* (an aging penalty for stale samples). The selected
//! sample is only trusted when its delay and dispersion are within bounds.
//!
//! This module implements that filter over fixed-point milliseconds so it stays
//! deterministic and dependency-free.

use crate::sntp::NtpMeasurement;

/// Depth of the per-peer clock-filter shift register (RFC 5905 uses 8).
pub const FILTER_DEPTH: usize = 8;

/// Maximum dispersion (ms) a peer may accumulate before its samples are
/// considered unusable (RFC 5905 `MAXDISP` is 16 s).
pub const MAX_DISPERSION_MS: i64 = 16_000;

/// A timestamped sample held in the clock filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterSample {
    /// Measured clock offset in milliseconds.
    pub offset_ms: i64,
    /// Round-trip delay in milliseconds.
    pub delay_ms: i64,
    /// Local time the sample was taken (Unix millis), used for dispersion aging.
    pub taken_unix_millis: i64,
}

impl FilterSample {
    /// Build a sample from a measurement taken at `taken_unix_millis`.
    pub fn from_measurement(m: &NtpMeasurement, taken_unix_millis: i64) -> Self {
        FilterSample {
            offset_ms: m.offset_ms,
            delay_ms: m.delay_ms,
            taken_unix_millis,
        }
    }
}

/// The result of running the clock filter over the current shift register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterResult {
    /// The selected (lowest-delay) offset in milliseconds.
    pub offset_ms: i64,
    /// The delay of the selected sample in milliseconds.
    pub delay_ms: i64,
    /// RMS jitter of the remaining samples about the selected offset (ms).
    pub jitter_ms: i64,
    /// Accumulated dispersion of the selected sample (ms), aged to `now`.
    pub dispersion_ms: i64,
}

impl FilterResult {
    /// Whether the selected estimate is trustworthy: a non-negative delay within
    /// `max_delay_ms` and a dispersion under [`MAX_DISPERSION_MS`].
    pub fn is_trustworthy(&self, max_delay_ms: i64) -> bool {
        self.delay_ms >= 0
            && self.delay_ms <= max_delay_ms
            && self.dispersion_ms < MAX_DISPERSION_MS
    }
}

/// A per-peer clock filter: a bounded shift register of recent samples plus the
/// selection/jitter logic over them.
#[derive(Debug, Clone, Default)]
pub struct ClockFilter {
    samples: Vec<FilterSample>,
}

impl ClockFilter {
    /// A fresh, empty filter.
    pub fn new() -> Self {
        ClockFilter {
            samples: Vec::new(),
        }
    }

    /// Number of samples currently held.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether the filter holds no samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Push a new sample, evicting the oldest once [`FILTER_DEPTH`] is exceeded.
    pub fn push(&mut self, sample: FilterSample) {
        if self.samples.len() == FILTER_DEPTH {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    /// Run the filter as of local time `now_unix_millis`.
    ///
    /// Returns `None` when empty. Otherwise selects the lowest-delay sample as
    /// the best offset, computes RMS jitter of the other samples about it, and
    /// the dispersion of the selected sample (aging penalty since it was taken,
    /// modeled at the conventional 15 ppm clock-frequency tolerance).
    pub fn evaluate(&self, now_unix_millis: i64) -> Option<FilterResult> {
        let best = self
            .samples
            .iter()
            .filter(|s| s.delay_ms >= 0)
            .min_by_key(|s| s.delay_ms)?;

        // Jitter: RMS of (offset_i - best_offset) over the other samples.
        let mut sumsq: i128 = 0;
        let mut count: i128 = 0;
        for s in &self.samples {
            if std::ptr::eq(s, best) {
                continue;
            }
            let d = i128::from(s.offset_ms - best.offset_ms);
            sumsq += d * d;
            count += 1;
        }
        let jitter_ms = if count > 0 {
            // The RMS of bounded-millisecond offsets fits comfortably in i64.
            #[allow(clippy::cast_possible_truncation)]
            let j = isqrt_i128(sumsq / count) as i64;
            j
        } else {
            0
        };

        // Dispersion: PHI (15 ppm) times the age of the selected sample, in ms.
        let age_ms = (now_unix_millis - best.taken_unix_millis).max(0);
        // 15 ppm => age_ms * 15 / 1_000_000.
        let dispersion_ms = (age_ms.saturating_mul(15)) / 1_000_000;

        Some(FilterResult {
            offset_ms: best.offset_ms,
            delay_ms: best.delay_ms,
            jitter_ms,
            dispersion_ms,
        })
    }

    /// Clear all samples (e.g. after a clock step invalidates history).
    pub fn reset(&mut self) {
        self.samples.clear();
    }
}

/// Integer square root for non-negative `i128`, rounding down.
fn isqrt_i128(n: i128) -> i128 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.midpoint(1);
    while y < x {
        x = y;
        y = x.midpoint(n / x);
    }
    x
}

/// Select the best peer among several evaluated filters: the one with the lowest
/// combined `delay + dispersion + jitter` (a simplified RFC 5905 "synchronization
/// distance"). Returns the index and its result. Untrustworthy peers are skipped.
pub fn select_best_peer(
    results: &[(usize, FilterResult)],
    max_delay_ms: i64,
) -> Option<(usize, FilterResult)> {
    results
        .iter()
        .filter(|(_, r)| r.is_trustworthy(max_delay_ms))
        .min_by_key(|(_, r)| r.delay_ms + r.dispersion_ms + r.jitter_ms)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(offset: i64, delay: i64, taken: i64) -> FilterSample {
        FilterSample {
            offset_ms: offset,
            delay_ms: delay,
            taken_unix_millis: taken,
        }
    }

    #[test]
    fn empty_filter_evaluates_to_none() {
        assert!(ClockFilter::new().evaluate(0).is_none());
    }

    #[test]
    fn selects_lowest_delay_sample() {
        let mut f = ClockFilter::new();
        f.push(sample(100, 80, 0));
        f.push(sample(40, 10, 0)); // lowest delay -> best offset
        f.push(sample(60, 50, 0));
        let r = f.evaluate(0).unwrap();
        assert_eq!(r.offset_ms, 40);
        assert_eq!(r.delay_ms, 10);
    }

    #[test]
    fn shift_register_bounded_to_depth() {
        let mut f = ClockFilter::new();
        for i in 0..i64::try_from(FILTER_DEPTH + 4).unwrap() {
            f.push(sample(i, 100 + i, 0));
        }
        assert_eq!(f.len(), FILTER_DEPTH);
    }

    #[test]
    fn jitter_is_zero_for_single_sample() {
        let mut f = ClockFilter::new();
        f.push(sample(42, 10, 0));
        let r = f.evaluate(0).unwrap();
        assert_eq!(r.jitter_ms, 0);
    }

    #[test]
    fn jitter_rms_of_spread() {
        let mut f = ClockFilter::new();
        // best is the delay-10 sample at offset 0; others differ by +/-3, +6.
        f.push(sample(0, 10, 0));
        f.push(sample(3, 50, 0));
        f.push(sample(-3, 60, 0));
        f.push(sample(6, 70, 0));
        let r = f.evaluate(0).unwrap();
        // RMS of {3, -3, 6} = sqrt((9+9+36)/3) = sqrt(18) = 4 (floored).
        assert_eq!(r.jitter_ms, 4);
    }

    #[test]
    fn dispersion_ages_with_time() {
        let mut f = ClockFilter::new();
        f.push(sample(0, 10, 0));
        // 1e9 ms later at 15 ppm => 1e9 * 15 / 1e6 = 15000 ms dispersion.
        let r = f.evaluate(1_000_000_000).unwrap();
        assert_eq!(r.dispersion_ms, 15_000);
        assert!(r.is_trustworthy(5_000)); // 15000 < MAX_DISPERSION_MS (16000)

        // Age further so dispersion exceeds MAX_DISPERSION_MS.
        let r2 = f.evaluate(2_000_000_000).unwrap();
        assert_eq!(r2.dispersion_ms, 30_000);
        assert!(!r2.is_trustworthy(5_000));
    }

    #[test]
    fn trustworthy_requires_bounded_delay() {
        let mut f = ClockFilter::new();
        f.push(sample(5, 9_000, 0));
        let r = f.evaluate(0).unwrap();
        assert!(!r.is_trustworthy(5_000));
        assert!(r.is_trustworthy(10_000));
    }

    #[test]
    fn negative_delay_samples_are_ignored() {
        let mut f = ClockFilter::new();
        f.push(sample(1, -5, 0)); // bogus, ignored in selection
        f.push(sample(2, 30, 0));
        let r = f.evaluate(0).unwrap();
        assert_eq!(r.delay_ms, 30);
    }

    #[test]
    fn reset_clears_history() {
        let mut f = ClockFilter::new();
        f.push(sample(1, 10, 0));
        f.reset();
        assert!(f.is_empty());
        assert!(f.evaluate(0).is_none());
    }

    #[test]
    fn select_best_peer_prefers_lowest_distance() {
        let r0 = FilterResult {
            offset_ms: 5,
            delay_ms: 100,
            jitter_ms: 2,
            dispersion_ms: 1,
        };
        let r1 = FilterResult {
            offset_ms: 6,
            delay_ms: 20,
            jitter_ms: 1,
            dispersion_ms: 1,
        };
        let chosen = select_best_peer(&[(0, r0), (1, r1)], 5_000).unwrap();
        assert_eq!(chosen.0, 1);
    }

    #[test]
    fn select_best_peer_skips_untrustworthy() {
        let bad = FilterResult {
            offset_ms: 0,
            delay_ms: 9_000,
            jitter_ms: 0,
            dispersion_ms: 0,
        };
        assert!(select_best_peer(&[(0, bad)], 5_000).is_none());
    }

    #[test]
    fn from_measurement_carries_fields() {
        let m = NtpMeasurement {
            offset_ms: 7,
            delay_ms: 21,
            stratum: 2,
        };
        let s = FilterSample::from_measurement(&m, 1234);
        assert_eq!(s.offset_ms, 7);
        assert_eq!(s.delay_ms, 21);
        assert_eq!(s.taken_unix_millis, 1234);
    }

    #[test]
    fn isqrt_basic() {
        assert_eq!(isqrt_i128(0), 0);
        assert_eq!(isqrt_i128(1), 1);
        assert_eq!(isqrt_i128(15), 3);
        assert_eq!(isqrt_i128(16), 4);
        assert_eq!(isqrt_i128(17), 4);
        assert_eq!(isqrt_i128(1_000_000), 1_000);
    }
}
