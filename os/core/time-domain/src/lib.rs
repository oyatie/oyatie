//! # talos-time
//!
//! A `no_std` port of the Talos time-synchronization subsystem, mirroring
//! `siderolabs/talos`'s `internal/app/machined/pkg/controllers/time` and the
//! `pkg/ntp` SNTP client.
//!
//! Talos runs a tiny SNTP/NTP client to discipline the system clock. The
//! machined `time` controllers turn the raw NTP measurements into COSI
//! resources (`TimeSync`, `TimeServerStatus`) whose `synced` flag is one of the
//! preconditions used to gate cluster bootstrap (etcd will not start until the
//! clock is believed correct).
//!
//! This crate models that boundary faithfully but offline:
//!
//! * [`ntp`] — NTP packet (RFC 5905) encode/decode and the 1900-vs-1970 epoch
//!   math, including the leap-indicator / stratum / mode fields.
//! * [`sntp`] — the SNTP query/response state machine and offset/round-trip
//!   computation, driven through a [`sntp::NtpTransport`] trait so the network
//!   syscall is mockable.
//! * [`sync`] — the [`sync::SyncSpec`] config (server list, min/max poll,
//!   bootstrap timeout) and validation rules.
//! * [`time_service`] — the gRPC-style `TimeService` API surface as a trait.
//! * [`status`] — [`status::TimeSyncStatus`] and
//!   [`status::TimeServerStatus`] COSI-style resources.
//! * [`adjust`] — [`adjust::ClockAdjustment`], slew-vs-step discipline logic.
//! * [`adjtime`] — the `adjtimex(2)` kernel discipline state ([`adjtime::AdjtimeState`])
//!   and the persisted RTC drift file ([`adjtime::AdjtimeFile`]).
//! * [`selection`] — the per-peer clock filter ([`selection::ClockFilter`]):
//!   lowest-delay sample selection, jitter, and dispersion accounting.
//! * [`timeserver`] — timeserver config-layer resolution
//!   ([`timeserver::TimeServerSpec`]) merging machine-config / platform /
//!   cmdline / default sources.
//! * [`controller`] — the COSI controller reconcile loop that consumes the
//!   client, applies adjustments, and publishes the sync status.
//!
//! The crate uses the standard library and has zero external dependencies
//! beyond the internal `talos-core` path dependency.

// Pedantic lints intentionally relaxed crate-wide, matching the sibling crates:
// requiring `#[must_use]` and `# Errors` doc sections on every small accessor or
// fallible builder would be noise without improving the API's safety.
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_errors_doc)]

pub mod adjtime;
pub mod adjust;
pub mod controller;
pub mod ntp;
pub mod selection;
pub mod sntp;
pub mod status;
pub mod sync;
pub mod time_service;
pub mod timeserver;

pub use adjtime::{AdjtimeFile, AdjtimeState};
pub use adjust::{AdjustMode, ClockAdjustment};
pub use controller::{TimeController, TimeControllerInput, TimeControllerOutput};
pub use ntp::{LeapIndicator, NtpMode, NtpPacket, NtpTimestamp, Stratum};
pub use selection::{ClockFilter, FilterResult, FilterSample};
pub use sntp::{NtpClient, NtpMeasurement, NtpTransport};
pub use status::{TimeServerStatus, TimeSyncStatus};
pub use sync::SyncSpec;
pub use time_service::{TimeReply, TimeRequest, TimeService};
pub use timeserver::{ConfigLayer, TimeServerLayer, TimeServerSpec};

use std::fmt;

/// Errors produced by the time subsystem.
///
/// Each variant maps onto a [`os_kernel::Error`] kind via [`TimeError::into`]
/// so callers at the crate boundary can fold these into the workspace error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    /// A configuration value failed validation (empty server list, poll
    /// interval out of range, ...).
    InvalidConfig(String),
    /// The NTP wire packet was malformed (bad length, version, mode).
    MalformedPacket(String),
    /// The server returned a kiss-of-death / unsynchronized response
    /// (stratum 0 or leap-indicator "alarm").
    ServerUnsynchronized,
    /// No configured server answered within the bootstrap deadline.
    NoServersReachable,
    /// The clock has not yet reached synced state when one was required.
    NotSynced,
    /// The transport (socket) failed.
    Transport(String),
}

impl TimeError {
    /// Construct an [`TimeError::InvalidConfig`].
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        TimeError::InvalidConfig(msg.into())
    }

    /// Construct a [`TimeError::MalformedPacket`].
    pub fn malformed(msg: impl Into<String>) -> Self {
        TimeError::MalformedPacket(msg.into())
    }

    /// Construct a [`TimeError::Transport`].
    pub fn transport(msg: impl Into<String>) -> Self {
        TimeError::Transport(msg.into())
    }

    /// Short, stable kind string.
    pub fn kind(&self) -> &'static str {
        match self {
            TimeError::InvalidConfig(_) => "invalid_config",
            TimeError::MalformedPacket(_) => "malformed_packet",
            TimeError::ServerUnsynchronized => "server_unsynchronized",
            TimeError::NoServersReachable => "no_servers_reachable",
            TimeError::NotSynced => "not_synced",
            TimeError::Transport(_) => "transport",
        }
    }
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::InvalidConfig(m) => write!(f, "invalid time config: {m}"),
            TimeError::MalformedPacket(m) => write!(f, "malformed ntp packet: {m}"),
            TimeError::ServerUnsynchronized => write!(f, "ntp server is unsynchronized"),
            TimeError::NoServersReachable => write!(f, "no ntp servers reachable"),
            TimeError::NotSynced => write!(f, "clock not yet synced"),
            TimeError::Transport(m) => write!(f, "ntp transport error: {m}"),
        }
    }
}

impl From<TimeError> for os_kernel::Error {
    fn from(e: TimeError) -> Self {
        use os_kernel::Error;
        match e {
            TimeError::InvalidConfig(m) => Error::Invalid(m),
            TimeError::MalformedPacket(m) => Error::Parse(m),
            TimeError::ServerUnsynchronized => {
                Error::InvalidState(String::from("ntp server unsynchronized"))
            }
            TimeError::NoServersReachable => {
                Error::NotFound(String::from("no ntp servers reachable"))
            }
            TimeError::NotSynced => Error::InvalidState(String::from("clock not synced")),
            TimeError::Transport(m) => Error::Other(m),
        }
    }
}

/// Crate-local result alias.
pub type Result<T> = std::result::Result<T, TimeError>;
