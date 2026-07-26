//! The `TimeService` API surface (NTP time check).
//!
//! Mirrors `pkg/machinery/api/time/time.proto`: `Time` and `TimeCheck` which
//! query an NTP server and report the local time alongside the server's,
//! computing the offset machined uses to decide whether the clock is in sync.

use crate::common::{ApiError, Code, RequestContext};
use os_kernel::role::Role;

/// A wall-clock instant expressed as whole seconds + nanoseconds since the Unix
/// epoch (the proto uses `google.protobuf.Timestamp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    /// Seconds since the Unix epoch.
    pub seconds: i64,
    /// Nanosecond fraction in `[0, 1_000_000_000)`.
    pub nanos: u32,
}

impl Timestamp {
    /// Construct, normalizing nanos that overflow a second.
    pub fn new(seconds: i64, nanos: u32) -> Self {
        let extra = (nanos / 1_000_000_000) as i64;
        Timestamp {
            seconds: seconds + extra,
            nanos: nanos % 1_000_000_000,
        }
    }

    /// Total nanoseconds since epoch as an `i128` (avoids overflow).
    pub fn as_nanos(self) -> i128 {
        self.seconds as i128 * 1_000_000_000 + self.nanos as i128
    }

    /// Signed offset `self - other` in nanoseconds.
    pub fn offset_nanos(self, other: Timestamp) -> i128 {
        self.as_nanos() - other.as_nanos()
    }
}

/// The result of a `Time`/`TimeCheck` call, mirroring `time.TimeResponse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeResponse {
    /// The NTP server queried.
    pub server: String,
    /// The local node's time.
    pub localtime: Timestamp,
    /// The time reported by the server.
    pub remotetime: Timestamp,
}

impl TimeResponse {
    /// The clock offset (local - remote) in nanoseconds. Positive means the
    /// local clock is ahead of the server.
    pub fn offset_nanos(&self) -> i128 {
        self.localtime.offset_nanos(self.remotetime)
    }

    /// Whether the clock is in sync within the given tolerance (nanoseconds).
    /// machined treats anything beyond a few hundred ms as out-of-sync.
    pub fn in_sync(&self, tolerance_nanos: i128) -> bool {
        self.offset_nanos().abs() <= tolerance_nanos
    }
}

/// The NTP source consulted by the time service, behind a trait so tests can
/// inject a deterministic server/local clock.
pub trait TimeSource {
    /// The default NTP server name.
    fn default_server(&self) -> String;

    /// The local node's current time.
    fn local_now(&self) -> Timestamp;

    /// Query the given NTP server for its current time.
    fn query(&self, server: &str) -> Result<Timestamp, ApiError>;
}

/// The `TimeService`.
pub struct TimeService<S: TimeSource> {
    source: S,
    /// In-sync tolerance in nanoseconds (default 400ms, matching machined).
    pub tolerance_nanos: i128,
}

impl<S: TimeSource> TimeService<S> {
    /// Wrap a source with the default 400ms tolerance.
    pub fn new(source: S) -> Self {
        TimeService {
            source,
            tolerance_nanos: 400_000_000,
        }
    }

    /// `Time`: query the default server.
    pub fn time(&self, ctx: &RequestContext) -> Result<TimeResponse, ApiError> {
        let server = self.source.default_server();
        self.time_check(ctx, &server)
    }

    /// `TimeCheck`: query a specific server and report local + remote time.
    pub fn time_check(&self, ctx: &RequestContext, server: &str) -> Result<TimeResponse, ApiError> {
        ctx.authorize(Role::Reader)?;
        if server.trim().is_empty() {
            return Err(ApiError::new(
                Code::InvalidArgument,
                "NTP server is required",
            ));
        }
        let remotetime = self.source.query(server)?;
        Ok(TimeResponse {
            server: server.to_string(),
            localtime: self.source.local_now(),
            remotetime,
        })
    }

    /// Whether the node's clock is currently in sync against the default server.
    pub fn is_in_sync(&self, ctx: &RequestContext) -> Result<bool, ApiError> {
        Ok(self.time(ctx)?.in_sync(self.tolerance_nanos))
    }
}

/// An in-memory NTP source for tests.
#[derive(Debug, Clone)]
pub struct FixedTimeSource {
    /// The default server name.
    pub server: String,
    /// The local clock value.
    pub local: Timestamp,
    /// The server clock value (returned for any reachable server).
    pub remote: Timestamp,
    /// Servers that should fail to resolve.
    pub unreachable: Vec<String>,
}

impl FixedTimeSource {
    /// A source where local and remote agree exactly.
    pub fn synced(seconds: i64) -> Self {
        FixedTimeSource {
            server: "time.cloudflare.com".to_string(),
            local: Timestamp::new(seconds, 0),
            remote: Timestamp::new(seconds, 0),
            unreachable: Vec::new(),
        }
    }
}

impl TimeSource for FixedTimeSource {
    fn default_server(&self) -> String {
        self.server.clone()
    }
    fn local_now(&self) -> Timestamp {
        self.local
    }
    fn query(&self, server: &str) -> Result<Timestamp, ApiError> {
        if self.unreachable.iter().any(|s| s == server) {
            return Err(ApiError::new(
                Code::Unavailable,
                format!("{server} unreachable"),
            ));
        }
        Ok(self.remote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_normalizes_nanos() {
        let t = Timestamp::new(10, 1_500_000_000);
        assert_eq!(t.seconds, 11);
        assert_eq!(t.nanos, 500_000_000);
    }

    #[test]
    fn offset_computation() {
        let local = Timestamp::new(100, 0);
        let remote = Timestamp::new(99, 500_000_000);
        assert_eq!(local.offset_nanos(remote), 500_000_000);
    }

    #[test]
    fn synced_clock_is_in_sync() {
        let svc = TimeService::new(FixedTimeSource::synced(1_700_000_000));
        let resp = svc.time(&RequestContext::admin_local()).unwrap();
        assert_eq!(resp.offset_nanos(), 0);
        assert!(resp.in_sync(svc.tolerance_nanos));
        assert!(svc.is_in_sync(&RequestContext::admin_local()).unwrap());
    }

    #[test]
    fn skewed_clock_out_of_sync() {
        let mut src = FixedTimeSource::synced(1_700_000_000);
        src.local = Timestamp::new(1_700_000_002, 0); // 2s ahead
        let svc = TimeService::new(src);
        let resp = svc.time(&RequestContext::admin_local()).unwrap();
        assert_eq!(resp.offset_nanos(), 2_000_000_000);
        assert!(!resp.in_sync(svc.tolerance_nanos));
    }

    #[test]
    fn unreachable_server_errors() {
        let mut src = FixedTimeSource::synced(1_700_000_000);
        src.unreachable.push("bad.ntp".to_string());
        let svc = TimeService::new(src);
        let err = svc
            .time_check(&RequestContext::admin_local(), "bad.ntp")
            .unwrap_err();
        assert_eq!(err.code, Code::Unavailable);
    }

    #[test]
    fn empty_server_rejected_and_read_gated() {
        let svc = TimeService::new(FixedTimeSource::synced(1));
        assert_eq!(
            svc.time_check(&RequestContext::admin_local(), "  ")
                .unwrap_err()
                .code,
            Code::InvalidArgument
        );
        let nobody = RequestContext::with_roles(os_kernel::role::RoleSet::new());
        assert_eq!(svc.time(&nobody).unwrap_err().code, Code::PermissionDenied);
    }
}
