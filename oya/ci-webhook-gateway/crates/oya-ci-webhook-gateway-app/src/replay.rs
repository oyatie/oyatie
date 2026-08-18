//! # replay — delivery-replay / dedup guard
//!
//! Pure, time-injected, std-only idempotency guard for the CI webhook receiver.
//!
//! ## Design
//!
//! [`DeliveryGuard`] holds an in-memory `HashMap` of seen delivery keys.  It is
//! wrapped in a `Mutex` so a single `Arc<Mutex<DeliveryGuard>>` can live inside
//! the `Clone`-able [`super::AppState`] and be shared safely across concurrent
//! axum handler invocations.
//!
//! ## Key-derivation precedence
//!
//! 1. **Primary**: `delivery_id` when it is non-empty and not the sentinel
//!    `"unknown"` (the value injected by the handler when the header is absent).
//! 2. **Fallback**: `(head_sha, pr_number, CiAction)` — all three fields
//!    present on [`oya_ci_webhook_gateway_kernel::CiTriggerEvent`] at the
//!    wiring point (after Step 4 route, before Step 5 jenkins.trigger).
//!
//! Using `Option`-based detection of the absent-header case ensures the
//! fallback branch is genuinely reachable and that distinct events with no
//! delivery-id header do NOT collide on the single sentinel key `"unknown"`.
//!
//! ## Record-on-receipt policy
//!
//! [`DeliveryGuard::record_and_check`] records the key **before** the Jenkins
//! trigger fires.  A concurrent replay of the same delivery is therefore
//! deduped even while the first delivery is still in-flight.
//!
//! Implication: if the first delivery subsequently fails at Step 5 (Jenkins
//! returns 502), the key remains recorded and a legitimate retry within the TTL
//! will be treated as a replay.  This is a deliberate, documented trade-off:
//! the guard is **best-effort single-instance dedup**, not a correctness
//! guarantee.  Distributed / shared-store dedup (sticky routing or Redis) is
//! named as follow-up item task #62.
//!
//! ## TTL
//!
//! Default TTL is 300 000 ms (5 minutes).  GitHub's default re-delivery
//! window is 24 h, but duplicate re-deliveries in normal operation arrive
//! within seconds; 5 min is long enough to catch all realistic duplicates
//! while keeping memory bounded in practice.
//!
//! ## Scope
//!
//! This guard is **single-instance only**.  A horizontally-scaled deployment
//! (multiple pods) cannot share state via this module; replays routed to a
//! different pod bypass the guard.  See task #62.

use oya_ci_webhook_gateway_kernel::CiAction;
use std::collections::HashMap;

/// Map `CiAction` to a stable `u8` discriminant for use as a `Hash`-able key.
/// `CiAction` itself does not derive `Hash`; this avoids touching the kernel crate.
fn ci_action_discriminant(action: CiAction) -> u8 {
    match action {
        CiAction::PrOpened => 0,
        CiAction::PrSynchronized => 1,
        CiAction::PrClosed => 2,
        CiAction::Ping => 3,
    }
}

/// Default TTL in milliseconds (5 minutes).
pub const DEFAULT_TTL_MS: u64 = 300_000;

/// The canonical key for a delivery.
///
/// Constructed via [`DeliveryKey::from_parts`]; not built directly by callers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DeliveryKey {
    /// Primary key: unique GitHub delivery ID.
    DeliveryId(String),
    /// Fallback key: `(head_sha, pr_number, action_discriminant)` when
    /// delivery-id is absent.  The action is stored as a `u8` discriminant
    /// (see [`ci_action_discriminant`]) because [`CiAction`] does not
    /// implement [`Hash`].
    ContentHash {
        head_sha: String,
        pr_number: u64,
        /// Stable discriminant of the [`CiAction`] variant (0=PrOpened,
        /// 1=PrSynchronized, 2=PrClosed, 3=Ping).
        action_disc: u8,
    },
}

impl DeliveryKey {
    /// Derive the delivery key for a routed event.
    ///
    /// Uses the primary `delivery_id` path when the id is non-empty and not
    /// the sentinel `"unknown"` inserted by the handler on a missing header.
    /// Falls back to `(head_sha, pr_number, action_discriminant)` otherwise.
    pub fn from_parts(delivery_id: &str, head_sha: &str, pr_number: u64, action: CiAction) -> Self {
        let id = delivery_id.trim();
        if !id.is_empty() && id != "unknown" {
            DeliveryKey::DeliveryId(id.to_owned())
        } else {
            DeliveryKey::ContentHash {
                head_sha: head_sha.to_owned(),
                pr_number,
                action_disc: ci_action_discriminant(action),
            }
        }
    }
}

/// Verdict returned by [`DeliveryGuard::record_and_check`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// This key has not been seen before; proceed with dispatch.
    FirstSeen,
    /// This key was already recorded within the TTL; short-circuit with an
    /// idempotent acknowledgement.
    Replay,
}

/// In-memory delivery-dedup guard.
///
/// All mutable state is inside the struct; callers wrap it in
/// `Arc<Mutex<DeliveryGuard>>` to share it across concurrent axum handlers.
///
/// Time is fully injected — the struct never reads the wall clock.
pub struct DeliveryGuard {
    /// Map of delivery key → timestamp (unix millis) when first seen.
    seen: HashMap<DeliveryKey, u64>,
    /// Window within which a repeated key is classified as a replay.
    ttl_ms: u64,
}

impl DeliveryGuard {
    /// Construct a guard with the given TTL.
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            seen: HashMap::new(),
            ttl_ms,
        }
    }

    /// Construct a guard with [`DEFAULT_TTL_MS`].
    pub fn with_default_ttl() -> Self {
        Self::new(DEFAULT_TTL_MS)
    }

    /// Record `key` at time `now_unix_millis` and return the verdict.
    ///
    /// - If `key` was never seen, or was seen but its entry has expired
    ///   (i.e. `now - recorded_at >= ttl_ms`), records the key and returns
    ///   [`Verdict::FirstSeen`].
    /// - If `key` was seen within the TTL, returns [`Verdict::Replay`] without
    ///   updating the recorded timestamp.
    pub fn record_and_check(&mut self, key: DeliveryKey, now_unix_millis: u64) -> Verdict {
        if let Some(&recorded_at) = self.seen.get(&key) {
            let elapsed = now_unix_millis.saturating_sub(recorded_at);
            if elapsed < self.ttl_ms {
                return Verdict::Replay;
            }
            // TTL expired — treat as fresh.
        }
        self.seen.insert(key, now_unix_millis);
        Verdict::FirstSeen
    }

    /// Remove all entries whose recorded timestamp is older than `ttl_ms`
    /// relative to `now_unix_millis`.
    ///
    /// Pruning is not required for correctness — [`record_and_check`] already
    /// treats expired entries as fresh.  It only bounds memory.  Exposed `pub`
    /// so the handler can call it before each `record_and_check`, or on a
    /// background timer.
    pub fn prune(&mut self, now_unix_millis: u64) {
        self.seen.retain(|_, &mut recorded_at| {
            now_unix_millis.saturating_sub(recorded_at) < self.ttl_ms
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ci_webhook_gateway_kernel::CiAction;

    fn key_id(id: &str) -> DeliveryKey {
        DeliveryKey::DeliveryId(id.to_owned())
    }

    fn key_content(sha: &str, pr: u64, action: CiAction) -> DeliveryKey {
        DeliveryKey::ContentHash {
            head_sha: sha.to_owned(),
            pr_number: pr,
            action_disc: ci_action_discriminant(action),
        }
    }

    // T3.1 — FirstSeen then Replay for identical keys.
    #[test]
    fn first_seen_then_replay_identical_key() {
        let mut guard = DeliveryGuard::new(60_000);
        let key = key_id("del-abc-123");
        let t0 = 1_000_000u64;

        assert_eq!(guard.record_and_check(key.clone(), t0), Verdict::FirstSeen);
        // Same key, 1 second later — still within TTL.
        assert_eq!(
            guard.record_and_check(key.clone(), t0 + 1_000),
            Verdict::Replay
        );
    }

    // T3.2 — Two distinct keys are both FirstSeen.
    #[test]
    fn two_distinct_keys_both_first_seen() {
        let mut guard = DeliveryGuard::new(60_000);
        let t0 = 2_000_000u64;
        let k1 = key_id("del-aaa");
        let k2 = key_id("del-bbb");

        assert_eq!(guard.record_and_check(k1, t0), Verdict::FirstSeen);
        assert_eq!(guard.record_and_check(k2, t0), Verdict::FirstSeen);
    }

    // T3.3 — TTL expiry restores FirstSeen via injected now.
    #[test]
    fn ttl_expiry_restores_first_seen() {
        let ttl_ms = 5_000u64;
        let mut guard = DeliveryGuard::new(ttl_ms);
        let key = key_id("del-ttl-test");
        let t0 = 10_000_000u64;

        assert_eq!(guard.record_and_check(key.clone(), t0), Verdict::FirstSeen);
        // Within TTL — replay.
        assert_eq!(
            guard.record_and_check(key.clone(), t0 + ttl_ms - 1),
            Verdict::Replay
        );
        // Exactly at TTL boundary — FirstSeen again (elapsed >= ttl_ms).
        assert_eq!(
            guard.record_and_check(key.clone(), t0 + ttl_ms),
            Verdict::FirstSeen
        );
    }

    // T3.4 — prune() removes only expired entries, retains fresh ones.
    #[test]
    fn prune_removes_only_expired_entries() {
        let ttl_ms = 10_000u64;
        let mut guard = DeliveryGuard::new(ttl_ms);
        let t0 = 20_000_000u64;

        let fresh = key_id("del-fresh");
        let stale = key_id("del-stale");

        guard.record_and_check(stale.clone(), t0);
        // Record fresh one later so it won't expire.
        let t1 = t0 + ttl_ms + 1; // stale entry is now past TTL
        guard.record_and_check(fresh.clone(), t1);

        // Prune relative to t1 + 1 — stale should be gone, fresh remains.
        guard.prune(t1 + 1);

        assert_eq!(guard.seen.len(), 1);
        assert!(guard.seen.contains_key(&fresh));
        assert!(!guard.seen.contains_key(&stale));
    }

    // T3.5 — delivery-id-vs-(head_sha, pr_number, CiAction) key-derivation
    //         precedence: non-empty non-sentinel id wins.
    #[test]
    fn key_derivation_delivery_id_takes_precedence() {
        // Non-empty, non-sentinel delivery_id → DeliveryId variant.
        let k = DeliveryKey::from_parts("del-xyz", "sha-abc", 7, CiAction::PrOpened);
        assert_eq!(k, key_id("del-xyz"));

        // Empty delivery_id → ContentHash fallback.
        let k2 = DeliveryKey::from_parts("", "sha-abc", 7, CiAction::PrOpened);
        assert_eq!(k2, key_content("sha-abc", 7, CiAction::PrOpened));

        // Sentinel "unknown" → ContentHash fallback (no collision on sentinel).
        let k3 = DeliveryKey::from_parts("unknown", "sha-abc", 7, CiAction::PrSynchronized);
        assert_eq!(k3, key_content("sha-abc", 7, CiAction::PrSynchronized));

        // Two distinct events with "unknown" delivery_id do NOT collide.
        let k4 = DeliveryKey::from_parts("unknown", "sha-def", 8, CiAction::PrOpened);
        assert_ne!(k3, k4);
    }
}
