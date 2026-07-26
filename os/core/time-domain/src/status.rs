//! COSI-style status resources published by the time controllers.
//!
//! Mirrors `siderolabs/talos`'s `TimeStatus` (alias `TimeSync`) and
//! `TimeServerStatus` resources. The `synced` flag on [`TimeSyncStatus`] is the
//! one machined uses as a precondition for cluster bootstrap.

use crate::adjust::ClockAdjustment;

/// Per-server reachability/quality status, mirroring `TimeServerStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeServerStatus {
    /// The configured NTP host this status describes.
    pub host: String,
    /// Whether the last query to this server succeeded.
    pub reachable: bool,
    /// Last observed stratum (0 if never reached).
    pub stratum: u8,
    /// Last measured offset in milliseconds (None if never measured).
    pub last_offset_ms: Option<i64>,
}

impl TimeServerStatus {
    /// A freshly-configured, not-yet-queried server.
    pub fn new(host: impl Into<String>) -> Self {
        TimeServerStatus {
            host: host.into(),
            reachable: false,
            stratum: 0,
            last_offset_ms: None,
        }
    }

    /// Whether this server is currently a usable time source.
    pub fn is_usable(&self) -> bool {
        self.reachable && self.stratum >= 1 && self.stratum < 16
    }
}

/// The overall clock-sync status, mirroring Talos's `TimeStatus`/`TimeSync`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSyncStatus {
    /// Whether the clock is believed correct (the bootstrap precondition).
    pub synced: bool,
    /// The most recent applied offset in milliseconds.
    pub last_offset_ms: i64,
    /// Number of consecutive successful syncs (used for confidence).
    pub sync_epochs: u32,
    /// Human-readable status message (mirrors the resource's `status` field).
    pub message: String,
}

impl Default for TimeSyncStatus {
    fn default() -> Self {
        TimeSyncStatus {
            synced: false,
            last_offset_ms: 0,
            sync_epochs: 0,
            message: String::from("time not yet synced"),
        }
    }
}

impl TimeSyncStatus {
    /// The status reported when sync is disabled: trusted immediately.
    pub fn trusted() -> Self {
        TimeSyncStatus {
            synced: true,
            last_offset_ms: 0,
            sync_epochs: 1,
            message: String::from("time sync disabled; trusting hardware clock"),
        }
    }

    /// Fold an applied adjustment into the status.
    ///
    /// A *step* resets the sync epoch counter (a large jump means we were not
    /// actually synced); a *slew*/*hold* within tolerance increments it and,
    /// once the offset is within `sync_tolerance_ms`, marks the clock synced.
    pub fn record_adjustment(&mut self, adj: ClockAdjustment, sync_tolerance_ms: i64) {
        self.last_offset_ms = adj.offset_ms;
        if adj.is_step() {
            self.sync_epochs = 0;
            self.synced = false;
            self.message = String::from("clock stepped; resyncing");
            return;
        }
        self.sync_epochs = self.sync_epochs.saturating_add(1);
        if adj.offset_ms.unsigned_abs() <= sync_tolerance_ms.unsigned_abs() {
            self.synced = true;
            self.message = String::from("time synced");
        } else {
            self.message = String::from("slewing toward reference");
        }
    }

    /// Whether the clock has been synced (the bootstrap gate).
    pub fn is_bootstrap_ready(&self) -> bool {
        self.synced
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adjust::ClockAdjustment;

    #[test]
    fn default_is_not_synced() {
        let s = TimeSyncStatus::default();
        assert!(!s.synced);
        assert!(!s.is_bootstrap_ready());
    }

    #[test]
    fn trusted_is_immediately_ready() {
        assert!(TimeSyncStatus::trusted().is_bootstrap_ready());
    }

    #[test]
    fn slew_within_tolerance_marks_synced() {
        let mut s = TimeSyncStatus::default();
        // offset 5 ms, dead band 0 => slew; tolerance 10 => synced.
        let adj = ClockAdjustment::decide(5, 0);
        s.record_adjustment(adj, 10);
        assert!(s.synced);
        assert_eq!(s.sync_epochs, 1);
    }

    #[test]
    fn step_resets_sync() {
        let mut s = TimeSyncStatus::trusted();
        let adj = ClockAdjustment::decide(10_000, 0);
        s.record_adjustment(adj, 10);
        assert!(!s.synced);
        assert_eq!(s.sync_epochs, 0);
    }

    #[test]
    fn server_usability() {
        let mut srv = TimeServerStatus::new("time.cloudflare.com");
        assert!(!srv.is_usable());
        srv.reachable = true;
        srv.stratum = 3;
        assert!(srv.is_usable());
        srv.stratum = 16;
        assert!(!srv.is_usable());
    }
}
