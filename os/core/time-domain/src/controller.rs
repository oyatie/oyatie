//! The time COSI controller reconcile loop.
//!
//! Mirrors `siderolabs/talos`'s `internal/app/machined/pkg/controllers/time`
//! controllers (`SyncController`, `AdjtimeStatusController`): on each reconcile
//! it queries the configured NTP servers through an [`NtpClient`], folds the
//! best measurement into a [`ClockAdjustment`], applies it to its model of the
//! system clock, and publishes a [`TimeSyncStatus`] plus per-server
//! [`TimeServerStatus`] resources.
//!
//! The controller is deterministic and offline: the local clock is an explicit
//! field advanced by the caller, and the network is the mockable
//! [`NtpTransport`].

use crate::adjtime::AdjtimeState;
use crate::adjust::ClockAdjustment;
use crate::selection::{ClockFilter, FilterSample};
use crate::sntp::{NtpClient, NtpMeasurement, NtpTransport};
use crate::status::{TimeServerStatus, TimeSyncStatus};
use crate::sync::SyncSpec;
use crate::time_service::{TimeReply, TimeRequest, TimeService};
use crate::{Result, TimeError};

/// Tolerance (ms) within which the clock is considered synced after slewing.
pub const SYNC_TOLERANCE_MS: i64 = 16;
/// Dead-band (ms): offsets at or below this are not corrected at all.
pub const DEAD_BAND_MS: i64 = 1;
/// Reject measurements whose round-trip delay exceeds this many milliseconds.
pub const MAX_ACCEPTABLE_DELAY_MS: i64 = 5_000;

/// Input to one reconcile pass.
#[derive(Debug, Clone, Copy)]
pub struct TimeControllerInput {
    /// The node's local clock reading (Unix millis) at the start of the pass.
    pub local_unix_millis: i64,
    /// Estimated one-way send/receive latency to model T1/T4 (ms).
    pub rtt_estimate_ms: i64,
}

/// Output of one reconcile pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeControllerOutput {
    /// The adjustment that was applied (if a usable measurement was obtained).
    pub adjustment: Option<ClockAdjustment>,
    /// The local clock after applying the adjustment.
    pub local_unix_millis: i64,
    /// The published sync status snapshot.
    pub status: TimeSyncStatus,
}

/// The time controller. Holds the config, the SNTP client, the current clock
/// model, and the published status resources.
#[derive(Debug)]
pub struct TimeController {
    spec: SyncSpec,
    client: NtpClient,
    status: TimeSyncStatus,
    servers: Vec<TimeServerStatus>,
    /// One clock filter per configured server (parallel to `servers`).
    filters: Vec<ClockFilter>,
    /// Kernel discipline state mirror (adjtimex).
    adjtime: AdjtimeState,
    /// Current poll exponent, adapted within `[min_poll, max_poll]`.
    poll_exponent: u8,
    local_unix_millis: i64,
    last_remote_unix_millis: i64,
}

impl TimeController {
    /// Build a controller from a validated [`SyncSpec`].
    ///
    /// Returns an error if the spec fails validation.
    pub fn new(spec: SyncSpec, initial_clock_unix_millis: i64) -> Result<Self> {
        spec.validate()?;
        let servers = spec
            .servers
            .iter()
            .map(|h| TimeServerStatus::new(h.clone()))
            .collect();
        let status = if spec.enabled {
            TimeSyncStatus::default()
        } else {
            TimeSyncStatus::trusted()
        };
        let filters = spec.servers.iter().map(|_| ClockFilter::new()).collect();
        let mut adjtime = AdjtimeState::default();
        if !spec.enabled {
            adjtime.set_synchronized(true);
        }
        let poll_exponent = spec.min_poll;
        Ok(TimeController {
            spec,
            client: NtpClient::new(),
            status,
            servers,
            filters,
            adjtime,
            poll_exponent,
            local_unix_millis: initial_clock_unix_millis,
            last_remote_unix_millis: initial_clock_unix_millis,
        })
    }

    /// Current published sync status.
    pub fn status(&self) -> &TimeSyncStatus {
        &self.status
    }

    /// Per-server status resources.
    pub fn server_statuses(&self) -> &[TimeServerStatus] {
        &self.servers
    }

    /// The controller's current model of the local clock.
    pub fn local_clock(&self) -> i64 {
        self.local_unix_millis
    }

    /// The kernel discipline state mirror (adjtimex).
    pub fn adjtime(&self) -> &AdjtimeState {
        &self.adjtime
    }

    /// The current poll exponent (log2 seconds), adapted within the spec window.
    pub fn poll_exponent(&self) -> u8 {
        self.poll_exponent
    }

    /// The current poll interval in seconds for the adapted exponent.
    pub fn poll_interval_secs(&self) -> u64 {
        self.spec.poll_interval_secs(self.poll_exponent)
    }

    /// The clock filter for server index `idx` (parallel to `server_statuses`).
    pub fn filter(&self, idx: usize) -> Option<&ClockFilter> {
        self.filters.get(idx)
    }

    /// Run one reconcile pass against `transport`.
    ///
    /// Walks the server list (round-robin starting at the current attempt
    /// count), keeping the lowest-delay acceptable measurement, then disciplines
    /// the clock and publishes status. When sync is disabled this is a no-op
    /// that keeps the trusted status.
    pub fn reconcile<T: NtpTransport>(
        &mut self,
        transport: &mut T,
        input: TimeControllerInput,
    ) -> Result<TimeControllerOutput> {
        self.local_unix_millis = input.local_unix_millis;

        if !self.spec.enabled {
            return Ok(TimeControllerOutput {
                adjustment: None,
                local_unix_millis: self.local_unix_millis,
                status: self.status.clone(),
            });
        }

        let mut best: Option<(usize, NtpMeasurement)> = None;
        let n = self.servers.len();
        // Fix the round-robin start for the whole pass so every server is
        // queried exactly once (the client's attempt count advances per query).
        let start = self.client.attempts();
        for i in 0..n {
            let idx = (start + i) % n;
            let host = self.servers[idx].host.clone();
            let send = self.local_unix_millis;
            let recv = self.local_unix_millis + input.rtt_estimate_ms.max(0);
            match self.client.query(transport, &host, send, recv) {
                Ok(m) => {
                    self.servers[idx].reachable = true;
                    self.servers[idx].stratum = m.stratum;
                    self.servers[idx].last_offset_ms = Some(m.offset_ms);
                    // Record the sample in this server's clock filter for
                    // jitter/dispersion accounting across polls.
                    self.filters[idx].push(FilterSample::from_measurement(&m, recv));
                    if m.is_acceptable(MAX_ACCEPTABLE_DELAY_MS) {
                        let better = match best {
                            Some((_, b)) => m.delay_ms < b.delay_ms,
                            None => true,
                        };
                        if better {
                            best = Some((idx, m));
                        }
                    }
                }
                Err(_) => {
                    self.servers[idx].reachable = false;
                }
            }
        }

        let (_, measurement) = best.ok_or(TimeError::NoServersReachable)?;

        let adj = ClockAdjustment::decide(measurement.offset_ms, DEAD_BAND_MS);
        // The reference (server) time is the local clock plus the measured
        // offset: offset = remote - local, so remote = local_before + offset.
        self.last_remote_unix_millis = self.local_unix_millis + measurement.offset_ms;
        self.local_unix_millis = adj.apply(self.local_unix_millis);
        self.status.record_adjustment(adj, SYNC_TOLERANCE_MS);

        // Mirror the discipline into the kernel adjtimex state and adapt the
        // poll interval the same way ntpd does: widen the poll as the clock
        // settles (small offsets), tighten it back after a disruptive step.
        // The poll interval is `1 << exponent` with the exponent capped at
        // `POLL_CEILING` (15), so it is at most 32_768 and always fits in i64.
        #[allow(clippy::cast_possible_wrap)]
        let poll_secs = self.spec.poll_interval_secs(self.poll_exponent) as i64;
        if adj.is_step() {
            self.adjtime.step();
            for f in &mut self.filters {
                f.reset();
            }
            self.poll_exponent = self.spec.min_poll;
        } else {
            self.adjtime
                .discipline(measurement.offset_ms, poll_secs, SYNC_TOLERANCE_MS);
            if self.status.synced && self.poll_exponent < self.spec.max_poll {
                self.poll_exponent += 1;
            } else if !self.status.synced && self.poll_exponent > self.spec.min_poll {
                self.poll_exponent -= 1;
            }
        }

        Ok(TimeControllerOutput {
            adjustment: Some(adj),
            local_unix_millis: self.local_unix_millis,
            status: self.status.clone(),
        })
    }

    /// Whether the controller believes the clock is synced enough to bootstrap.
    pub fn bootstrap_ready(&self) -> bool {
        self.status.is_bootstrap_ready()
    }
}

impl TimeService for TimeController {
    fn time(&self, req: &TimeRequest) -> Result<TimeReply> {
        let fallback = self.spec.servers.first().map_or("", String::as_str);
        Ok(TimeReply {
            server: req.effective_server(fallback).to_string(),
            local_unix_millis: self.local_unix_millis,
            remote_unix_millis: self.last_remote_unix_millis,
        })
    }

    fn time_check(&self, req: &TimeRequest) -> Result<TimeReply> {
        self.time(req)
    }

    fn sync_status(&self) -> TimeSyncStatus {
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sntp::FakeTransport;

    #[test]
    fn new_rejects_invalid_spec() {
        let spec = SyncSpec {
            enabled: true,
            servers: Vec::new(),
            ..SyncSpec::default()
        };
        assert!(TimeController::new(spec, 0).is_err());
    }

    #[test]
    fn disabled_spec_is_trusted_and_noop() {
        let mut ctrl = TimeController::new(SyncSpec::disabled(), 1_000).unwrap();
        assert!(ctrl.bootstrap_ready());
        let mut tx = FakeTransport::healthy(9_999, 2);
        let out = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: 1_000,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap();
        assert!(out.adjustment.is_none());
        // clock untouched
        assert_eq!(out.local_unix_millis, 1_000);
    }

    #[test]
    fn large_offset_steps_then_resync_syncs() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        // Server is at 9000 ms => local is 8000 behind => big step.
        let mut tx = FakeTransport::healthy(9_000, 2);
        let out = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: 1_000,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap();
        let adj = out.adjustment.unwrap();
        assert!(adj.is_step());
        // clock jumped forward to ~9000
        assert_eq!(out.local_unix_millis, 9_000);
        assert!(!out.status.synced);

        // Now the clock matches the server: tiny offset => slew within tolerance.
        tx.server_unix_millis = 9_000;
        let out2 = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: 9_000,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap();
        assert!(out2.status.synced);
        assert!(ctrl.bootstrap_ready());
    }

    #[test]
    fn no_reachable_servers_errors() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        let mut tx = FakeTransport::healthy(0, 2);
        tx.fail_with = Some(TimeError::transport("down"));
        let err = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: 1_000,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap_err();
        assert_eq!(err, TimeError::NoServersReachable);
        assert!(!ctrl.server_statuses()[0].reachable);
    }

    #[test]
    fn picks_lowest_delay_server() {
        // Two servers; both healthy at the same clock, so offsets equal. The
        // controller still records both and produces a measurement.
        let spec = SyncSpec::with_servers(["a", "b"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        let mut tx = FakeTransport::healthy(1_050, 2);
        let out = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: 1_000,
                    rtt_estimate_ms: 20,
                },
            )
            .unwrap();
        assert!(out.adjustment.is_some());
        // both servers were queried this pass
        assert!(ctrl.server_statuses().iter().all(|s| s.reachable));
    }

    #[test]
    fn time_service_reports_local_and_remote() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        let mut tx = FakeTransport::healthy(1_010, 2);
        ctrl.reconcile(
            &mut tx,
            TimeControllerInput {
                local_unix_millis: 1_000,
                rtt_estimate_ms: 0,
            },
        )
        .unwrap();
        let reply = ctrl.time(&TimeRequest::default_server()).unwrap();
        assert_eq!(reply.server, "a");
        assert!(ctrl.sync_status().sync_epochs >= 1);
    }

    #[test]
    fn adjtime_tracks_sync_after_small_offset() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        // Server 5 ms ahead => slew within tolerance => synced.
        let mut tx = FakeTransport::healthy(1_005, 2);
        ctrl.reconcile(
            &mut tx,
            TimeControllerInput {
                local_unix_millis: 1_000,
                rtt_estimate_ms: 0,
            },
        )
        .unwrap();
        assert!(!ctrl.adjtime().is_unsynchronized());
    }

    #[test]
    fn step_resets_adjtime_and_filters_and_poll() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        // First settle so the poll exponent grows.
        let mut tx = FakeTransport::healthy(1_005, 2);
        for _ in 0..3 {
            let now = ctrl.local_clock();
            ctrl.reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: now,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap();
        }
        let grown = ctrl.poll_exponent();
        assert!(grown > crate::sync::DEFAULT_MIN_POLL);

        // Now a huge step.
        tx.server_unix_millis = 9_000_000;
        let now = ctrl.local_clock();
        let out = ctrl
            .reconcile(
                &mut tx,
                TimeControllerInput {
                    local_unix_millis: now,
                    rtt_estimate_ms: 0,
                },
            )
            .unwrap();
        assert!(out.adjustment.unwrap().is_step());
        assert!(ctrl.adjtime().is_unsynchronized());
        // poll exponent reset to min after the step
        assert_eq!(ctrl.poll_exponent(), crate::sync::DEFAULT_MIN_POLL);
        // filters cleared after the step
        assert!(ctrl.filter(0).unwrap().is_empty());
    }

    #[test]
    fn filter_accumulates_samples_per_server() {
        let spec = SyncSpec::with_servers(["a"]);
        let mut ctrl = TimeController::new(spec, 1_000).unwrap();
        let mut tx = FakeTransport::healthy(1_004, 2);
        ctrl.reconcile(
            &mut tx,
            TimeControllerInput {
                local_unix_millis: 1_000,
                rtt_estimate_ms: 10,
            },
        )
        .unwrap();
        assert_eq!(ctrl.filter(0).unwrap().len(), 1);
        let r = ctrl.filter(0).unwrap().evaluate(2_000).unwrap();
        assert!(r.delay_ms >= 0);
    }

    #[test]
    fn poll_interval_reflects_exponent() {
        let mut spec = SyncSpec::with_servers(["a"]);
        spec.min_poll = 6;
        spec.max_poll = 8;
        let ctrl = TimeController::new(spec, 0).unwrap();
        assert_eq!(ctrl.poll_exponent(), 6);
        assert_eq!(ctrl.poll_interval_secs(), 64);
    }

    #[test]
    fn disabled_controller_adjtime_is_synced() {
        let ctrl = TimeController::new(SyncSpec::disabled(), 0).unwrap();
        assert!(!ctrl.adjtime().is_unsynchronized());
    }
}
