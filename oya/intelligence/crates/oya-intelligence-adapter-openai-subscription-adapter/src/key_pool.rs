//! Round-robin API-key pool with failure-count blacklist, jittered cooldown, and
//! success-restore. Lifted from gpt-load / one-api circuit-breaker pattern.
//!
//! # Circuit-breaker states
//! - `Active` — eligible; selected in round-robin order.
//! - `Cooling { until_epoch_secs, failure_count }` — skip until cooldown expires.
//! - `Blacklisted` — terminal error; never selected again in this process lifetime.
//!
//! # Parameters
//! - Failure threshold: 3 consecutive transient failures → Cooling.
//! - Cooldown base: 60 s + uniform jitter [0, `jitter_max_secs`).
//! - Terminal error → immediate Blacklisted (no cooldown counter needed).
//! - Success while Cooling → restore to Active + reset failure_count.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
// data_class: INTERNAL_ONLY throughout this module.

use tracing::{debug, info, warn};

use crate::classifier::ResponseClass;
use crate::key_status::{KeyEntry, KeyStatus};

/// Number of consecutive transient failures before a key enters Cooling.
pub const FAILURE_THRESHOLD: u32 = 3;

/// Base cooldown duration in seconds before jitter is applied.
pub const COOLDOWN_BASE_SECS: u64 = 60;

/// Default upper bound for jitter (exclusive). Cooldown = base + [0, jitter_max).
pub const DEFAULT_JITTER_MAX_SECS: u64 = 30;

/// A pool of OpenAI API keys with round-robin selection and circuit-breaker logic.
///
/// SECURITY: KeyPool holds secret-reference paths only, not raw key material.
/// The caller resolves the sref_path to actual key bytes via their secret store.
pub struct KeyPool {
    entries: Vec<KeyEntry>,
    /// Next index to try in round-robin selection.
    next_idx: usize,
    /// Upper bound (exclusive) for jitter in seconds.
    jitter_max_secs: u64,
}

impl KeyPool {
    /// Create a new pool from a list of secret-reference paths.
    pub fn new(sref_paths: Vec<String>) -> Self {
        let entries = sref_paths.into_iter().map(KeyEntry::new).collect();
        Self {
            entries,
            next_idx: 0,
            jitter_max_secs: DEFAULT_JITTER_MAX_SECS,
        }
    }

    /// Override the jitter upper bound (for testing with deterministic cooldown).
    pub fn with_jitter_max(mut self, jitter_max_secs: u64) -> Self {
        self.jitter_max_secs = jitter_max_secs;
        self
    }

    /// Returns `true` if the pool has at least one key.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Select the next eligible key by round-robin.
    ///
    /// Returns `Some(index)` into `entries` for the selected key, or `None` if all
    /// keys are Blacklisted or still cooling.
    pub fn select(&mut self, now_epoch_secs: u64) -> Option<usize> {
        let len = self.entries.len();
        if len == 0 {
            return None;
        }
        // Try each key at most once.
        for i in 0..len {
            let idx = (self.next_idx + i) % len;
            if self.entries[idx].status.is_eligible(now_epoch_secs) {
                // Advance the round-robin cursor past the selected entry.
                self.next_idx = (idx + 1) % len;
                debug!(key_index = idx, "selected OpenAI API key");
                return Some(idx);
            }
        }
        None
    }

    /// Record the result of a call that used key at `key_index`.
    ///
    /// `jitter_secs`: caller-provided jitter (must be in `[0, jitter_max_secs)`).
    /// For production use, derive jitter from a CSPRNG or timestamp-based source.
    pub fn record_result(
        &mut self,
        key_index: usize,
        class: ResponseClass,
        now_epoch_secs: u64,
        jitter_secs: u64,
    ) {
        let entry = &mut self.entries[key_index];
        match class {
            ResponseClass::Success => {
                if matches!(entry.status, KeyStatus::Cooling { .. }) {
                    info!(key_index, "OpenAI API key restored from cooling to active");
                    entry.status = KeyStatus::Active;
                }
                // Already Active: no state change needed.
            }
            ResponseClass::TerminalKeyInvalid | ResponseClass::TerminalQuotaExhausted => {
                warn!(
                    key_index,
                    class = ?class,
                    "OpenAI API key blacklisted (terminal error)"
                );
                entry.status = KeyStatus::Blacklisted;
            }
            ResponseClass::TransientRateLimit
            | ResponseClass::TransientServer
            | ResponseClass::TransientUnknown => {
                let new_count = match &entry.status {
                    KeyStatus::Active => 1,
                    KeyStatus::Cooling { failure_count, .. } => failure_count + 1,
                    KeyStatus::Blacklisted => return, // terminal; ignore
                };
                if new_count >= FAILURE_THRESHOLD {
                    let jitter = jitter_secs.min(self.jitter_max_secs.saturating_sub(1));
                    let until = now_epoch_secs
                        .saturating_add(COOLDOWN_BASE_SECS)
                        .saturating_add(jitter);
                    debug!(
                        key_index,
                        failure_count = new_count,
                        until_epoch_secs = until,
                        "OpenAI API key entering cooling"
                    );
                    entry.status = KeyStatus::Cooling {
                        until_epoch_secs: until,
                        failure_count: new_count,
                    };
                } else {
                    // Increment failure count but stay Active until threshold.
                    entry.status = KeyStatus::Cooling {
                        until_epoch_secs: 0, // eligible immediately (0 ≤ any now)
                        failure_count: new_count,
                    };
                }
            }
        }
    }

    /// Returns the status of the key at `index` (for tests / observability).
    pub fn key_status(&self, index: usize) -> &KeyStatus {
        &self.entries[index].status
    }

    /// Returns the sref_path of the key at `index`.
    pub fn key_sref(&self, index: usize) -> &str {
        &self.entries[index].sref_path
    }

    /// Returns the number of keys in the pool.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::ResponseClass;

    fn pool(keys: &[&str]) -> KeyPool {
        KeyPool::new(keys.iter().map(|s| s.to_string()).collect()).with_jitter_max(30)
    }

    #[test]
    fn empty_pool_returns_none() {
        let mut p = pool(&[]);
        assert!(p.select(1000).is_none());
    }

    #[test]
    fn single_key_selected() {
        let mut p = pool(&["sref://k0"]);
        assert_eq!(p.select(1000), Some(0));
    }

    #[test]
    fn round_robin_cycles() {
        let mut p = pool(&["sref://k0", "sref://k1", "sref://k2"]);
        let a = p.select(1000).unwrap();
        let b = p.select(1000).unwrap();
        let c = p.select(1000).unwrap();
        let d = p.select(1000).unwrap();
        // Should cycle: a=0, b=1, c=2, d=0
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 2);
        assert_eq!(d, 0);
    }

    #[test]
    fn terminal_error_blacklists_key() {
        let mut p = pool(&["sref://k0"]);
        let idx = p.select(1000).unwrap();
        p.record_result(idx, ResponseClass::TerminalKeyInvalid, 1000, 0);
        assert_eq!(*p.key_status(0), KeyStatus::Blacklisted);
        assert!(p.select(1000).is_none());
    }

    #[test]
    fn quota_exhausted_blacklists_key() {
        let mut p = pool(&["sref://k0"]);
        p.record_result(0, ResponseClass::TerminalQuotaExhausted, 1000, 0);
        assert_eq!(*p.key_status(0), KeyStatus::Blacklisted);
    }

    #[test]
    fn three_transients_enter_cooling() {
        let now = 1_000_000u64;
        let mut p = pool(&["sref://k0"]).with_jitter_max(30);
        p.record_result(0, ResponseClass::TransientRateLimit, now, 0);
        p.record_result(0, ResponseClass::TransientRateLimit, now, 0);
        p.record_result(0, ResponseClass::TransientRateLimit, now, 0);
        match p.key_status(0) {
            KeyStatus::Cooling {
                until_epoch_secs,
                failure_count,
            } => {
                // cooldown = now + 60 + jitter(0) = now + 60
                assert_eq!(*until_epoch_secs, now + 60);
                assert_eq!(*failure_count, 3);
            }
            s => panic!("expected Cooling, got {s:?}"),
        }
    }

    #[test]
    fn cooling_jitter_range() {
        let now = 1_000_000u64;
        // Test with jitter = 29 (max - 1)
        let mut p = pool(&["sref://k0"]).with_jitter_max(30);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 29);
        match p.key_status(0) {
            KeyStatus::Cooling {
                until_epoch_secs, ..
            } => {
                // cooldown = now + 60 + 29 = now + 89 (max jitter capped at jitter_max - 1 = 29)
                assert_eq!(*until_epoch_secs, now + 89);
                // Must be in [now+60, now+90)
                assert!(*until_epoch_secs >= now + 60);
                assert!(*until_epoch_secs < now + 90);
            }
            s => panic!("expected Cooling, got {s:?}"),
        }
    }

    #[test]
    fn cooling_key_skipped_until_expiry() {
        let now = 1_000_000u64;
        let mut p = pool(&["sref://k0"]).with_jitter_max(0);
        // Force into cooling immediately (threshold=3, but jitter_max=0 means until=now+60)
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        // Key is cooling; select before expiry returns None
        assert!(p.select(now + 59).is_none());
        // Select at expiry succeeds
        assert_eq!(p.select(now + 60), Some(0));
    }

    #[test]
    fn success_restores_cooling_key() {
        let now = 1_000_000u64;
        let mut p = pool(&["sref://k0"]).with_jitter_max(0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        // Key is cooling; record success restores it
        p.record_result(0, ResponseClass::Success, now + 100, 0);
        assert_eq!(*p.key_status(0), KeyStatus::Active);
        assert_eq!(p.select(now + 100), Some(0));
    }

    #[test]
    fn all_keys_blacklisted_returns_none() {
        let mut p = pool(&["sref://k0", "sref://k1"]);
        p.record_result(0, ResponseClass::TerminalKeyInvalid, 1000, 0);
        p.record_result(1, ResponseClass::TerminalKeyInvalid, 1000, 0);
        assert!(p.select(1000).is_none());
    }

    #[test]
    fn second_key_used_when_first_blacklisted() {
        let mut p = pool(&["sref://k0", "sref://k1"]);
        p.record_result(0, ResponseClass::TerminalKeyInvalid, 1000, 0);
        let idx = p.select(1000).unwrap();
        assert_eq!(idx, 1);
    }

    #[test]
    fn two_transients_do_not_trigger_cooling_start() {
        let now = 1_000u64;
        let mut p = pool(&["sref://k0"]);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        p.record_result(0, ResponseClass::TransientServer, now, 0);
        // 2 failures < threshold of 3; key still immediately eligible
        assert_eq!(p.select(now), Some(0));
    }
}
