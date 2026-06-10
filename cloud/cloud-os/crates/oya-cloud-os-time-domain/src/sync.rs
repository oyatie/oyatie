//! Time-sync configuration ([`SyncSpec`]) and its validation rules.
//!
//! Mirrors the machine-config `time:` block consumed by Talos's time
//! controllers: the NTP server list, the poll-interval bounds, and the
//! bootstrap timeout that gates cluster startup on a believed-correct clock.

use crate::{Result, TimeError};

/// Default NTP poll exponents (RFC 5905 recommends 6..=10, i.e. 64s..1024s).
pub const DEFAULT_MIN_POLL: u8 = 6;
/// Default maximum poll exponent.
pub const DEFAULT_MAX_POLL: u8 = 10;
/// Hard lower bound on the poll exponent (16 seconds).
pub const POLL_FLOOR: u8 = 4;
/// Hard upper bound on the poll exponent (~9.1 hours).
pub const POLL_CEILING: u8 = 15;
/// Default time the node will wait for the clock to sync before giving up.
pub const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 70 * 60 * 1000;

/// The time-synchronization configuration for a node.
///
/// Modeled on `siderolabs/talos`'s `TimeConfig` (machine config `machine.time`)
/// plus the controller defaults that fill in unset fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSpec {
    /// Whether time sync is enabled at all. When disabled the node trusts its
    /// RTC and reports `synced = true` immediately.
    pub enabled: bool,
    /// Ordered list of NTP server hosts to query (round-robin).
    pub servers: Vec<String>,
    /// Minimum poll exponent (log2 seconds).
    pub min_poll: u8,
    /// Maximum poll exponent (log2 seconds).
    pub max_poll: u8,
    /// How long to wait for first sync before bootstrap proceeds anyway.
    pub bootstrap_timeout_ms: u64,
}

impl Default for SyncSpec {
    fn default() -> Self {
        SyncSpec {
            enabled: true,
            servers: vec![String::from("time.cloudflare.com")],
            min_poll: DEFAULT_MIN_POLL,
            max_poll: DEFAULT_MAX_POLL,
            bootstrap_timeout_ms: DEFAULT_BOOTSTRAP_TIMEOUT_MS,
        }
    }
}

impl SyncSpec {
    /// Build a spec from an explicit server list, using default poll bounds.
    pub fn with_servers<I, S>(servers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        SyncSpec {
            servers: servers.into_iter().map(Into::into).collect(),
            ..SyncSpec::default()
        }
    }

    /// A spec with time-sync disabled (node trusts its hardware clock).
    pub fn disabled() -> Self {
        SyncSpec {
            enabled: false,
            servers: Vec::new(),
            ..SyncSpec::default()
        }
    }

    /// Validate the configuration, returning the first violation found.
    ///
    /// Rules:
    /// * an *enabled* spec must list at least one server;
    /// * every server string must be non-empty and contain no whitespace;
    /// * poll exponents must fall within `[POLL_FLOOR, POLL_CEILING]`;
    /// * `min_poll <= max_poll`;
    /// * the bootstrap timeout must be non-zero.
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.servers.is_empty() {
            return Err(TimeError::invalid_config(
                "time sync enabled but no NTP servers configured",
            ));
        }
        for s in &self.servers {
            if s.is_empty() {
                return Err(TimeError::invalid_config("empty NTP server host"));
            }
            if s.chars().any(char::is_whitespace) {
                return Err(TimeError::invalid_config(format!(
                    "NTP server host contains whitespace: {s:?}"
                )));
            }
        }
        if self.min_poll < POLL_FLOOR || self.min_poll > POLL_CEILING {
            return Err(TimeError::invalid_config(format!(
                "min_poll {} out of range {}..={}",
                self.min_poll, POLL_FLOOR, POLL_CEILING
            )));
        }
        if self.max_poll < POLL_FLOOR || self.max_poll > POLL_CEILING {
            return Err(TimeError::invalid_config(format!(
                "max_poll {} out of range {}..={}",
                self.max_poll, POLL_FLOOR, POLL_CEILING
            )));
        }
        if self.min_poll > self.max_poll {
            return Err(TimeError::invalid_config(format!(
                "min_poll {} exceeds max_poll {}",
                self.min_poll, self.max_poll
            )));
        }
        if self.bootstrap_timeout_ms == 0 {
            return Err(TimeError::invalid_config(
                "bootstrap timeout must be non-zero",
            ));
        }
        Ok(())
    }

    /// The poll interval (seconds) for the given exponent, clamped to the
    /// configured `[min_poll, max_poll]` window.
    pub fn poll_interval_secs(&self, exponent: u8) -> u64 {
        let e = exponent.clamp(self.min_poll, self.max_poll);
        1u64 << e
    }

    /// Select the next server to query for poll number `attempt`, round-robin.
    pub fn server_for_attempt(&self, attempt: usize) -> Option<&str> {
        if self.servers.is_empty() {
            return None;
        }
        Some(self.servers[attempt % self.servers.len()].as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid() {
        assert!(SyncSpec::default().validate().is_ok());
    }

    #[test]
    fn enabled_without_servers_is_invalid() {
        let spec = SyncSpec {
            enabled: true,
            servers: Vec::new(),
            ..SyncSpec::default()
        };
        assert_eq!(spec.validate().unwrap_err().kind(), "invalid_config");
    }

    #[test]
    fn disabled_without_servers_is_ok() {
        assert!(SyncSpec::disabled().validate().is_ok());
    }

    #[test]
    fn rejects_bad_poll_window_and_whitespace() {
        let spec = SyncSpec {
            min_poll: 11,
            max_poll: 10,
            ..SyncSpec::default()
        };
        assert!(spec.validate().is_err());

        let ws = SyncSpec::with_servers(["bad host"]);
        assert!(ws.validate().is_err());
    }

    #[test]
    fn poll_interval_clamps_and_rotates_servers() {
        let spec = SyncSpec::with_servers(["a", "b", "c"]);
        // exponent below min_poll clamps up to min_poll (6 => 64s)
        assert_eq!(spec.poll_interval_secs(0), 64);
        assert_eq!(spec.poll_interval_secs(10), 1024);
        assert_eq!(spec.server_for_attempt(0), Some("a"));
        assert_eq!(spec.server_for_attempt(4), Some("b"));
    }
}
