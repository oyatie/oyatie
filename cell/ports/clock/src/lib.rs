//! Cell clock port (ADR-0719). Product API is always an interval.
//!
//! Closed adapters: `ntp` (v1, works without hardware), `ptp_phc`, `gnss_atomic`.
//! Callers do not branch on adapter. Unwired plant adapters fail at bind, not at `now`.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{Duration, SystemTime};

/// Default NTP window used when the cell has not measured ε (Cockroach-class NTP).
pub const NTP_DEFAULT_UNCERTAINTY: Duration = Duration::from_millis(250);

/// Closed clock-source set. Deleting a variant without a five-field ADR is born-blocking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockSource {
    Ntp,
    PtpPhc,
    GnssAtomic,
}

impl ClockSource {
    pub const CLOSED: [ClockSource; 3] = [
        ClockSource::Ntp,
        ClockSource::PtpPhc,
        ClockSource::GnssAtomic,
    ];
}

/// `[earliest, latest]` plus a hybrid logical counter inside the interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Interval {
    pub earliest: SystemTime,
    pub latest: SystemTime,
    pub logical: u64,
}

impl Interval {
    pub fn contains(self, instant: SystemTime) -> bool {
        self.earliest <= instant && instant <= self.latest
    }
}

pub trait Clock: Send + Sync {
    fn source(&self) -> ClockSource;
    fn now(&self) -> Interval;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockBindError {
    AdapterNotWired(ClockSource),
}

impl Display for ClockBindError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::AdapterNotWired(src) => {
                write!(f, "clock adapter {src:?} is not wired in this cell")
            }
        }
    }
}

impl Error for ClockBindError {}

#[derive(Clone, Debug)]
pub struct NtpClock {
    uncertainty: Duration,
}

impl NtpClock {
    pub fn with_uncertainty(uncertainty: Duration) -> Self {
        Self { uncertainty }
    }
}

impl Default for NtpClock {
    fn default() -> Self {
        Self::with_uncertainty(NTP_DEFAULT_UNCERTAINTY)
    }
}

impl Clock for NtpClock {
    fn source(&self) -> ClockSource {
        ClockSource::Ntp
    }

    fn now(&self) -> Interval {
        let mid = SystemTime::now();
        Interval {
            earliest: mid
                .checked_sub(self.uncertainty)
                .unwrap_or(SystemTime::UNIX_EPOCH),
            latest: mid + self.uncertainty,
            logical: 0,
        }
    }
}

/// Bind the adapter named in cell IR. NTP is v1. PTP/GNSS fail closed until plant exists.
pub fn bind(source: ClockSource) -> Result<NtpClock, ClockBindError> {
    match source {
        ClockSource::Ntp => Ok(NtpClock::default()),
        ClockSource::PtpPhc | ClockSource::GnssAtomic => {
            Err(ClockBindError::AdapterNotWired(source))
        }
    }
}
