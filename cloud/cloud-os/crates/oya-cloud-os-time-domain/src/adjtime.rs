//! Kernel clock-discipline state, mirroring the `adjtimex(2)` interface that
//! `siderolabs/talos`'s `AdjtimeStatusController` reads to publish the
//! `AdjtimeStatus` resource.
//!
//! Linux exposes the disciplined-clock state through a `struct timex`. Talos
//! does not run a full `ntpd`; instead its time controllers feed measured
//! offsets into the kernel PLL/FLL (via `adjtimex`) and then read the resulting
//! status back out — the frequency offset (`freq`, in scaled ppm), the estimated
//! error, the maximum error, the PLL time constant, and the `STA_*` status
//! flags (notably `STA_UNSYNC`, set whenever the discipline is not locked).
//!
//! This module re-implements the relevant pieces of that interface as pure data
//! so the controller can be exercised offline. The PLL update here is a faithful
//! but simplified model of the kernel's hybrid PLL/FLL loop: each measured
//! offset both produces an immediate offset correction and nudges the running
//! frequency estimate, the same two-term update ntpd and the kernel use.

/// Scaling Linux uses for the frequency field: ppm are stored shifted left 16.
/// `freq` of `1 << SHIFT_USEC` therefore represents 1 ppm.
pub const SHIFT_USEC: i64 = 16;

/// One part-per-million expressed in the kernel's scaled `freq` units.
pub const PPM_SCALE: i64 = 1 << SHIFT_USEC;

/// The maximum frequency adjustment the kernel will accept, in ppm. Linux caps
/// `MAXFREQ` at 500 ppm; offsets implying a larger correction are clamped.
pub const MAX_FREQ_PPM: i64 = 500;

/// Status flags from `struct timex.status` (`STA_*` in `<sys/timex.h>`).
///
/// Only the bits Talos actually inspects/sets are modeled.
pub mod status_flags {
    /// `STA_PLL`: enable phase-locked loop updates.
    pub const STA_PLL: u32 = 0x0001;
    /// `STA_FLL`: enable frequency-locked loop updates.
    pub const STA_FLL: u32 = 0x0008;
    /// `STA_UNSYNC`: clock is unsynchronized (the bit the controller watches).
    pub const STA_UNSYNC: u32 = 0x0040;
    /// `STA_FREQHOLD`: hold frequency, do not adapt it from offsets.
    pub const STA_FREQHOLD: u32 = 0x0080;
    /// `STA_NANO`: report/accept times in nanoseconds rather than microseconds.
    pub const STA_NANO: u32 = 0x2000;
}

/// A faithful-but-simplified model of the kernel `struct timex` discipline
/// state, as read back by the `AdjtimeStatusController`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjtimeState {
    /// Current frequency offset in scaled ppm (`freq` field; `PPM_SCALE` per ppm).
    pub freq_scaled_ppm: i64,
    /// The most recently injected time offset in milliseconds (`offset`).
    pub offset_ms: i64,
    /// Estimated error in milliseconds (`esterror`).
    pub est_error_ms: i64,
    /// Maximum error in milliseconds (`maxerror`).
    pub max_error_ms: i64,
    /// PLL time constant exponent (`constant`; loop bandwidth, larger = slower).
    pub constant: i64,
    /// Status flag bitset (`status`, `STA_*`).
    pub status: u32,
    /// Whether the kernel reports state in nanoseconds (`STA_NANO`).
    pub nano: bool,
}

impl Default for AdjtimeState {
    fn default() -> Self {
        // Fresh boot: PLL enabled but unsynchronized, no learned frequency.
        AdjtimeState {
            freq_scaled_ppm: 0,
            offset_ms: 0,
            est_error_ms: i64::from(i32::MAX),
            max_error_ms: i64::from(i32::MAX),
            constant: 2,
            status: status_flags::STA_PLL | status_flags::STA_UNSYNC,
            nano: false,
        }
    }
}

impl AdjtimeState {
    /// Whether the kernel currently considers the clock unsynchronized.
    pub fn is_unsynchronized(&self) -> bool {
        self.status & status_flags::STA_UNSYNC != 0
    }

    /// The learned frequency offset in whole ppm (rounded toward zero).
    pub fn freq_ppm(&self) -> i64 {
        self.freq_scaled_ppm / PPM_SCALE
    }

    /// Set the synchronized state, clearing or setting `STA_UNSYNC` accordingly.
    pub fn set_synchronized(&mut self, synced: bool) {
        if synced {
            self.status &= !status_flags::STA_UNSYNC;
        } else {
            self.status |= status_flags::STA_UNSYNC;
        }
    }

    /// Whether frequency adaptation is currently held (`STA_FREQHOLD`).
    pub fn frequency_held(&self) -> bool {
        self.status & status_flags::STA_FREQHOLD != 0
    }

    /// Feed a measured offset into the discipline loop.
    ///
    /// This models one `adjtimex` PLL update: the offset is stored for the
    /// immediate phase correction, and (unless frequency is held) a fraction of
    /// the offset is folded into the running frequency estimate, scaled by the
    /// poll interval and the loop time constant. The frequency is clamped to
    /// `MAX_FREQ_PPM`. Applying a correction marks the clock synchronized once
    /// the offset is within `lock_tolerance_ms`.
    pub fn discipline(&mut self, offset_ms: i64, poll_interval_secs: i64, lock_tolerance_ms: i64) {
        self.offset_ms = offset_ms;
        // Estimated error tracks the magnitude of the residual offset.
        self.est_error_ms = offset_ms.abs();

        if !self.frequency_held() {
            // PLL frequency term: df ~= offset / (poll * 2^constant). We work in
            // scaled ppm: a 1 ms offset over a 1 s interval is ~1000 ppm of rate
            // error; we damp by the loop time constant to avoid overshoot.
            let interval = poll_interval_secs.max(1);
            let denom = interval * (1i64 << self.constant.clamp(0, 16));
            // offset_ms / interval gives ms-per-second drift; * 1000 => ppm.
            let raw_ppm = (offset_ms.saturating_mul(1000)) / denom;
            let new_freq = self.freq_scaled_ppm + raw_ppm.saturating_mul(PPM_SCALE);
            self.freq_scaled_ppm = clamp_freq(new_freq);
        }

        self.set_synchronized(offset_ms.abs() <= lock_tolerance_ms.abs());
    }

    /// A hard step (clock jump) resets the discipline: the kernel discards the
    /// phase offset, clears the lock, but keeps the learned frequency.
    pub fn step(&mut self) {
        self.offset_ms = 0;
        self.est_error_ms = 0;
        self.set_synchronized(false);
    }
}

/// Clamp a scaled-ppm frequency to `±MAX_FREQ_PPM`.
fn clamp_freq(scaled: i64) -> i64 {
    let limit = MAX_FREQ_PPM * PPM_SCALE;
    scaled.clamp(-limit, limit)
}

/// The persisted RTC drift file (`/var/lib/talos/adjtime`-style), the three-line
/// format the classic `adjtimex --adjust`/`hwclock` writes: drift rate, last
/// adjustment time, and the UTC-vs-LOCAL mode line.
///
/// Talos always runs the hardware clock in UTC; this models reading/writing the
/// drift so a learned frequency survives reboots.
#[derive(Debug, Clone, PartialEq)]
pub struct AdjtimeFile {
    /// Drift rate in seconds-per-day the RTC gains or loses.
    pub drift_seconds_per_day: f64,
    /// Unix time (seconds) of the last calibration.
    pub last_adjust_unix_secs: i64,
    /// Whether the RTC is kept in UTC (Talos: always true) vs local time.
    pub utc: bool,
}

impl AdjtimeFile {
    /// A fresh, never-calibrated UTC drift file.
    pub fn new() -> Self {
        AdjtimeFile {
            drift_seconds_per_day: 0.0,
            last_adjust_unix_secs: 0,
            utc: true,
        }
    }

    /// Render the classic three-line `/etc/adjtime` format.
    pub fn render(&self) -> String {
        let mode = if self.utc { "UTC" } else { "LOCAL" };
        // The classic format's second number (the "0.0" offset) is unused by
        // modern systems; we keep it zero. The third value is the last-adjust time.
        format!(
            "{:.6} {} 0.000000\n0\n{}\n",
            self.drift_seconds_per_day, self.last_adjust_unix_secs, mode
        )
    }

    /// Parse the classic three-line `/etc/adjtime` format. Tolerant of the
    /// minimal subset Talos writes.
    pub fn parse(text: &str) -> crate::Result<Self> {
        let mut lines = text.lines();
        let first = lines
            .next()
            .ok_or_else(|| crate::TimeError::malformed("empty adjtime file"))?;
        let mut fields = first.split_whitespace();
        let drift = fields
            .next()
            .ok_or_else(|| crate::TimeError::malformed("missing drift field"))?;
        let drift_seconds_per_day: f64 = drift
            .parse()
            .map_err(|_| crate::TimeError::malformed("drift not a number"))?;
        let last_adjust_unix_secs: i64 = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        // Second line is the offset (ignored); third line is the mode.
        let _offset_line = lines.next();
        let mode_line = lines.next().unwrap_or("UTC").trim();
        let utc = !mode_line.eq_ignore_ascii_case("LOCAL");
        Ok(AdjtimeFile {
            drift_seconds_per_day,
            last_adjust_unix_secs,
            utc,
        })
    }

    /// Update the drift estimate from a freshly learned ppm frequency at a given
    /// time. ppm => seconds-per-day is `ppm * 86400 / 1e6`.
    #[allow(clippy::cast_precision_loss)] // ppm frequencies are small (|freq| <= MAX_FREQ_PPM).
    pub fn update_from_ppm(&mut self, freq_ppm: i64, now_unix_secs: i64) {
        self.drift_seconds_per_day = (freq_ppm as f64) * 86_400.0 / 1_000_000.0;
        self.last_adjust_unix_secs = now_unix_secs;
    }
}

impl Default for AdjtimeFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_unsynced_with_pll() {
        let s = AdjtimeState::default();
        assert!(s.is_unsynchronized());
        assert_eq!(s.status & status_flags::STA_PLL, status_flags::STA_PLL);
        assert_eq!(s.freq_ppm(), 0);
    }

    #[test]
    fn set_synchronized_toggles_unsync_bit() {
        let mut s = AdjtimeState::default();
        s.set_synchronized(true);
        assert!(!s.is_unsynchronized());
        s.set_synchronized(false);
        assert!(s.is_unsynchronized());
    }

    #[test]
    fn discipline_locks_within_tolerance_and_learns_frequency() {
        let mut s = AdjtimeState::default();
        // 4 ms offset over a 64s poll, tolerance 16 ms => locks.
        s.discipline(4, 64, 16);
        assert!(!s.is_unsynchronized());
        assert_eq!(s.offset_ms, 4);
        // a positive offset nudges frequency positive (clock was slow).
        assert!(s.freq_scaled_ppm > 0);
    }

    #[test]
    fn discipline_beyond_tolerance_stays_unsynced() {
        let mut s = AdjtimeState::default();
        s.discipline(500, 64, 16);
        assert!(s.is_unsynchronized());
        assert_eq!(s.est_error_ms, 500);
    }

    #[test]
    fn frequency_is_clamped_to_max() {
        let mut s = AdjtimeState::default();
        // Enormous offset over a tiny interval would imply a huge ppm; clamp it.
        s.discipline(1_000_000, 1, 16);
        assert_eq!(s.freq_ppm().abs(), MAX_FREQ_PPM);
    }

    #[test]
    fn frequency_hold_blocks_adaptation() {
        let mut s = AdjtimeState::default();
        s.status |= status_flags::STA_FREQHOLD;
        s.discipline(50, 16, 4);
        assert_eq!(s.freq_scaled_ppm, 0);
        assert!(s.frequency_held());
    }

    #[test]
    fn step_resets_offset_and_lock_but_keeps_freq() {
        let mut s = AdjtimeState::default();
        s.discipline(8, 64, 16);
        let learned = s.freq_scaled_ppm;
        assert!(learned != 0);
        s.step();
        assert_eq!(s.offset_ms, 0);
        assert!(s.is_unsynchronized());
        assert_eq!(s.freq_scaled_ppm, learned);
    }

    #[test]
    fn adjtime_file_roundtrips() {
        let mut f = AdjtimeFile::new();
        f.update_from_ppm(125, 1_700_000_000);
        let rendered = f.render();
        let parsed = AdjtimeFile::parse(&rendered).unwrap();
        assert!(parsed.utc);
        assert_eq!(parsed.last_adjust_unix_secs, 1_700_000_000);
        assert!((parsed.drift_seconds_per_day - f.drift_seconds_per_day).abs() < 1e-6);
    }

    #[test]
    fn adjtime_file_parses_local_mode() {
        let text = "0.500000 1600000000 0.000000\n0\nLOCAL\n";
        let f = AdjtimeFile::parse(text).unwrap();
        assert!(!f.utc);
        assert!((f.drift_seconds_per_day - 0.5).abs() < 1e-9);
    }

    #[test]
    fn adjtime_file_rejects_garbage() {
        assert!(AdjtimeFile::parse("").is_err());
        assert!(AdjtimeFile::parse("notanumber 0\n0\nUTC\n").is_err());
    }

    #[test]
    fn ppm_to_drift_conversion() {
        let mut f = AdjtimeFile::new();
        // 1_000_000 ppm = 1 sec/sec = 86400 sec/day.
        f.update_from_ppm(1_000_000, 0);
        assert!((f.drift_seconds_per_day - 86_400.0).abs() < 1e-6);
    }
}
