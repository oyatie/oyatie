//! The `TimeService` gRPC-style API surface, modeled as a trait.
//!
//! Mirrors `siderolabs/talos`'s `machine.TimeService` (the `Time` and
//! `TimeCheck` RPCs the apid exposes), which let a client ask a node for its
//! current time and the offset it measures against a given NTP server.

use crate::Result;
use crate::status::TimeSyncStatus;

/// A `Time`/`TimeCheck` request: which NTP server to consult.
///
/// An empty `server` means "use the node's configured server".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TimeRequest {
    /// NTP server to check against, or empty for the node default.
    pub server: String,
}

impl TimeRequest {
    /// A request against the node's configured server.
    pub fn default_server() -> Self {
        TimeRequest::default()
    }

    /// A request against an explicit server.
    pub fn against(server: impl Into<String>) -> Self {
        TimeRequest {
            server: server.into(),
        }
    }

    /// The effective server, falling back to `fallback` when unset.
    pub fn effective_server<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.server.is_empty() {
            fallback
        } else {
            self.server.as_str()
        }
    }
}

/// A `Time`/`TimeCheck` reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeReply {
    /// The server that was consulted.
    pub server: String,
    /// The node's local time as Unix milliseconds.
    pub local_unix_millis: i64,
    /// The reference (server) time as Unix milliseconds.
    pub remote_unix_millis: i64,
}

impl TimeReply {
    /// The measured offset (local - remote) in milliseconds.
    pub fn offset_ms(&self) -> i64 {
        self.local_unix_millis - self.remote_unix_millis
    }
}

/// The node-side `TimeService` API surface.
///
/// Production wires this to apid; the controller in [`crate::controller`]
/// provides an in-memory implementation backed by its last measurement.
pub trait TimeService {
    /// `Time` RPC: report the node's current local and reference time.
    fn time(&self, req: &TimeRequest) -> Result<TimeReply>;

    /// `TimeCheck` RPC: like [`TimeService::time`] but explicitly against a
    /// given server without disciplining the local clock.
    fn time_check(&self, req: &TimeRequest) -> Result<TimeReply>;

    /// Report the current sync status (used by health/readiness checks).
    fn sync_status(&self) -> TimeSyncStatus;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubService {
        local: i64,
        remote: i64,
        default_server: String,
    }

    impl TimeService for StubService {
        fn time(&self, req: &TimeRequest) -> Result<TimeReply> {
            Ok(TimeReply {
                server: req.effective_server(&self.default_server).to_string(),
                local_unix_millis: self.local,
                remote_unix_millis: self.remote,
            })
        }
        fn time_check(&self, req: &TimeRequest) -> Result<TimeReply> {
            self.time(req)
        }
        fn sync_status(&self) -> TimeSyncStatus {
            TimeSyncStatus::trusted()
        }
    }

    #[test]
    fn request_effective_server_falls_back() {
        let req = TimeRequest::default_server();
        assert_eq!(req.effective_server("pool.ntp.org"), "pool.ntp.org");
        let req2 = TimeRequest::against("time.cloudflare.com");
        assert_eq!(req2.effective_server("pool.ntp.org"), "time.cloudflare.com");
    }

    #[test]
    fn reply_offset_computation() {
        let r = TimeReply {
            server: "s".into(),
            local_unix_millis: 1_005,
            remote_unix_millis: 1_000,
        };
        assert_eq!(r.offset_ms(), 5);
    }

    #[test]
    fn stub_service_uses_default_and_reports_status() {
        let svc = StubService {
            local: 2_050,
            remote: 2_000,
            default_server: "default.ntp".into(),
        };
        let reply = svc.time(&TimeRequest::default_server()).unwrap();
        assert_eq!(reply.server, "default.ntp");
        assert_eq!(reply.offset_ms(), 50);
        assert!(svc.sync_status().is_bootstrap_ready());
    }

    #[test]
    fn stub_time_check_against_explicit_server() {
        let svc = StubService {
            local: 0,
            remote: 0,
            default_server: "d".into(),
        };
        let reply = svc
            .time_check(&TimeRequest::against("explicit.ntp"))
            .unwrap();
        assert_eq!(reply.server, "explicit.ntp");
    }
}
