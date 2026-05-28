//! LLM agent-dispatch gateway — key-pool kernel (ADR-0105 Layer 1, pure).
//!
//! Clean-room reimplementation of the *concept* of an LLM key-pool reverse
//! proxy (round-robin key rotation with failure-driven blacklisting and
//! timed cooldown recovery). No third-party code was read or copied; this is
//! an original state machine expressed in std-only Rust.
//!
//! # What this layer owns
//! - [`ProviderChannel`] — the upstream API dialect enum (OpenAI / Anthropic /
//!   Gemini). The kernel knows *which* channel a pool serves; it does NOT know
//!   how to speak HTTP (that is the rest adapter's job).
//! - [`KeyId`] / key fingerprints — the kernel never stores or sees a raw API
//!   key. It works with opaque [`KeyId`] handles and stable
//!   [`KeyFingerprint`] hashes (the SHA-256-of-key hashing itself is done by
//!   the adapter; the kernel only carries the resulting hash for logging).
//! - [`KeyPool`] — the round-robin selection + per-key failure-count +
//!   blacklist-threshold + jittered-cooldown + success-reset state machine.
//!
//! # What this layer must NEVER do
//! No async, no I/O, no network, no clock reads, no randomness source, no
//! external crate. Time and jitter are *injected* as plain values
//! ([`now_unix_millis`](KeyPool::select) takes a caller-supplied clock reading)
//! so the machine is fully deterministic and unit-testable. This keeps the
//! kernel honest: every transition is a pure function of its inputs.
//!
//! # Selection contract
//! [`KeyPool::select`] returns the next *active* key in round-robin order,
//! skipping keys that are blacklisted or still in cooldown at the supplied
//! timestamp. Cooldown is checked lazily on selection, so an expired-cooldown
//! key is automatically restored to active without a background sweeper.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` / arithmetic that cannot overflow under the cfg(test) exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::atomic::{AtomicUsize, Ordering};

/// Upstream provider dialect a pool serves. The kernel uses this only to tag a
/// pool and to let callers branch on auth/transport in the adapter layer; the
/// kernel performs no provider-specific logic itself.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProviderChannel {
    /// OpenAI / Codex dialect (`Authorization: Bearer <key>`).
    OpenAi,
    /// Anthropic Claude dialect (`x-api-key` + `anthropic-version`).
    Anthropic,
    /// Google Gemini dialect (`X-Goog-Api-Key`).
    Gemini,
}

impl ProviderChannel {
    /// Stable lowercase identifier, used as a metric label and config key.
    /// Stable across releases — treated as part of the wire/label contract.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderChannel::OpenAi => "openai",
            ProviderChannel::Anthropic => "anthropic",
            ProviderChannel::Gemini => "gemini",
        }
    }

    /// Parse a channel from its stable identifier (config / routing input).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" | "codex" => Some(ProviderChannel::OpenAi),
            "anthropic" | "claude" => Some(ProviderChannel::Anthropic),
            "gemini" | "google" => Some(ProviderChannel::Gemini),
            _ => None,
        }
    }
}

/// Opaque handle to a key slot inside a [`KeyPool`]. Index-stable for the
/// lifetime of the pool. Never serialized to a caller as anything but an
/// integer; carries no secret material.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyId(pub usize);

/// A non-secret, hash-only fingerprint of an API key. The kernel stores this
/// purely so logs/metrics can identify *which* pooled key was used WITHOUT
/// ever holding the key itself. Construction is the adapter's responsibility
/// (it hashes the raw key); the kernel only carries the bytes-as-hex string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyFingerprint(String);

impl KeyFingerprint {
    /// Wrap an already-computed hex fingerprint (e.g. first 16 hex chars of a
    /// SHA-256 over the raw key). The kernel does not validate the hashing
    /// scheme; it only guarantees the value it stores is exactly what it was
    /// given and is exposed only through [`KeyFingerprint::as_str`].
    #[must_use]
    pub fn from_hex(hex: impl Into<String>) -> Self {
        KeyFingerprint(hex.into())
    }

    /// The hash-only label. Safe to log and to use as a metric label value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tunable thresholds for the failure → blacklist → cooldown → restore cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolPolicy {
    /// Consecutive failures that trip a key from active → blacklisted.
    /// Must be >= 1 (a value of 0 is clamped to 1 at construction).
    pub blacklist_threshold: u32, // data_class: INTERNAL_ONLY
    /// Base cooldown duration (milliseconds) applied when a key is
    /// blacklisted, before any jitter is added.
    pub cooldown_base_millis: u64, // data_class: INTERNAL_ONLY
    /// Maximum extra jitter (milliseconds) added on top of the base cooldown.
    /// `0` disables jitter (cooldown is exactly `cooldown_base_millis`).
    pub cooldown_jitter_millis: u64, // data_class: INTERNAL_ONLY
}

impl PoolPolicy {
    /// Construct a policy, clamping `blacklist_threshold` to a minimum of 1 so
    /// a misconfigured `0` can never blacklist a key on its very first use in
    /// a way that permanently starves the pool.
    #[must_use]
    pub fn new(
        blacklist_threshold: u32,
        cooldown_base_millis: u64,
        cooldown_jitter_millis: u64,
    ) -> Self {
        PoolPolicy {
            blacklist_threshold: blacklist_threshold.max(1),
            cooldown_base_millis,
            cooldown_jitter_millis,
        }
    }
}

impl Default for PoolPolicy {
    fn default() -> Self {
        // Conservative defaults: 3 strikes, 30s base cooldown, up to 10s jitter.
        PoolPolicy::new(3, 30_000, 10_000)
    }
}

/// Lifecycle state of a single pooled key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    /// Eligible for selection.
    Active,
    /// Tripped by failures; not eligible until `cooldown_until_millis` passes,
    /// at which point selection lazily restores it to [`KeyState::Active`].
    Blacklisted {
        /// Absolute timestamp (caller's clock, ms) when cooldown expires.
        cooldown_until_millis: u64, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KeySlot {
    fingerprint: KeyFingerprint, // data_class: INTERNAL_ONLY
    state: KeyState,             // data_class: INTERNAL_ONLY
    /// Consecutive failures since the last success / restore.
    failure_count: u32, // data_class: INTERNAL_ONLY
}

/// A round-robin pool of API keys for a single [`ProviderChannel`].
///
/// Selection is lock-free for the cursor (an [`AtomicUsize`]); per-slot state
/// transitions are applied through `&mut self` methods so the owning runtime
/// is responsible for synchronizing mutation (typically behind a single
/// `Mutex`/`RwLock` in the adapter while `select` reads the atomic cursor).
///
/// The pool is intentionally *not* `Sync`-mutated internally beyond the
/// cursor: keeping mutation `&mut` makes the state machine trivially
/// race-free to reason about and to test.
#[derive(Debug)]
pub struct KeyPool {
    channel: ProviderChannel, // data_class: INTERNAL_ONLY
    policy: PoolPolicy,       // data_class: INTERNAL_ONLY
    slots: Vec<KeySlot>,      // data_class: INTERNAL_ONLY
    cursor: AtomicUsize,      // data_class: INTERNAL_ONLY
}

/// Outcome of a [`KeyPool::select`] call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selection {
    /// A usable key was found.
    Key {
        /// Stable slot handle to report success/failure against.
        id: KeyId, // data_class: INTERNAL_ONLY
        /// Hash-only fingerprint for logging/metrics.
        fingerprint: KeyFingerprint, // data_class: INTERNAL_ONLY
    },
    /// Every key is currently blacklisted/in-cooldown; caller should fail the
    /// request (typically 503) rather than rotate forever.
    Exhausted,
    /// The pool was constructed with no keys at all.
    Empty,
}

impl KeyPool {
    /// Build a pool for `channel` from a list of key fingerprints, applying
    /// `policy`. Order is preserved and defines the round-robin order.
    #[must_use]
    pub fn new(
        channel: ProviderChannel,
        policy: PoolPolicy,
        fingerprints: Vec<KeyFingerprint>,
    ) -> Self {
        let slots = fingerprints
            .into_iter()
            .map(|fingerprint| KeySlot {
                fingerprint,
                state: KeyState::Active,
                failure_count: 0,
            })
            .collect();
        KeyPool {
            channel,
            policy,
            slots,
            cursor: AtomicUsize::new(0),
        }
    }

    /// The provider channel this pool serves.
    #[must_use]
    pub fn channel(&self) -> ProviderChannel {
        self.channel
    }

    /// Total number of keys (regardless of state).
    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// `true` if the pool holds no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Number of keys that are active (selectable) at `now_unix_millis`,
    /// counting keys whose cooldown has expired (they are restored lazily on
    /// the next `select`, but counted as active here so the metric reflects
    /// true availability). Pure read — does not mutate.
    #[must_use]
    pub fn active_count(&self, now_unix_millis: u64) -> usize {
        self.slots
            .iter()
            .filter(|slot| match slot.state {
                KeyState::Active => true,
                KeyState::Blacklisted {
                    cooldown_until_millis,
                } => now_unix_millis >= cooldown_until_millis,
            })
            .count()
    }

    /// Select the next usable key in round-robin order at `now_unix_millis`.
    ///
    /// Walks at most `len()` slots starting from the atomic cursor. A
    /// blacklisted key whose cooldown has expired is restored to active and
    /// returned. The cursor advances by exactly one per call (mod `len`) so
    /// load spreads evenly across keys even when some are skipped.
    ///
    /// Requires `&mut self` because an expired-cooldown key is restored
    /// in-place (a state transition). Selection performs no I/O.
    pub fn select(&mut self, now_unix_millis: u64) -> Selection {
        let len = self.slots.len();
        if len == 0 {
            return Selection::Empty;
        }
        // Advance the shared cursor once; this is the round-robin step.
        let start = self.cursor.fetch_add(1, Ordering::Relaxed) % len;
        for offset in 0..len {
            let idx = (start + offset) % len;
            let restore = match self.slots[idx].state {
                KeyState::Active => true,
                KeyState::Blacklisted {
                    cooldown_until_millis,
                } => now_unix_millis >= cooldown_until_millis,
            };
            if restore {
                // Lazy restore: clear blacklist + failure history on re-entry.
                if matches!(self.slots[idx].state, KeyState::Blacklisted { .. }) {
                    self.slots[idx].state = KeyState::Active;
                    self.slots[idx].failure_count = 0;
                }
                return Selection::Key {
                    id: KeyId(idx),
                    fingerprint: self.slots[idx].fingerprint.clone(),
                };
            }
        }
        Selection::Exhausted
    }

    /// Record a successful upstream call for `id`: resets the failure counter
    /// and (defensively) restores the key to active. Idempotent and safe to
    /// call on an unknown/out-of-range id (no-op).
    pub fn record_success(&mut self, id: KeyId) {
        if let Some(slot) = self.slots.get_mut(id.0) {
            slot.failure_count = 0;
            slot.state = KeyState::Active;
        }
    }

    /// Record a failed upstream call for `id`. Increments the consecutive
    /// failure counter; if it reaches the blacklist threshold the key is moved
    /// to [`KeyState::Blacklisted`] with cooldown =
    /// `cooldown_base_millis + jitter`, where `jitter` is derived purely from
    /// the caller-supplied `jitter_seed` (no internal RNG). Returns the new
    /// state so the adapter can emit a metric/log without re-reading.
    ///
    /// `jitter_seed` should be a fresh per-call entropy value supplied by the
    /// runtime (e.g. low bits of a monotonic clock or an injected RNG). The
    /// kernel folds it into `[0, cooldown_jitter_millis]` deterministically so
    /// tests can pin exact cooldown windows.
    pub fn record_failure(
        &mut self,
        id: KeyId,
        now_unix_millis: u64,
        jitter_seed: u64,
    ) -> Option<KeyState> {
        let slot = self.slots.get_mut(id.0)?;
        slot.failure_count = slot.failure_count.saturating_add(1);
        if slot.failure_count >= self.policy.blacklist_threshold {
            let jitter = if self.policy.cooldown_jitter_millis == 0 {
                0
            } else {
                jitter_seed % (self.policy.cooldown_jitter_millis.saturating_add(1))
            };
            let cooldown_until_millis = now_unix_millis
                .saturating_add(self.policy.cooldown_base_millis)
                .saturating_add(jitter);
            slot.state = KeyState::Blacklisted {
                cooldown_until_millis,
            };
        }
        Some(slot.state)
    }

    /// Read the current state of a key (testing/observability). `None` if the
    /// id is out of range.
    #[must_use]
    pub fn state_of(&self, id: KeyId) -> Option<KeyState> {
        self.slots.get(id.0).map(|slot| slot.state)
    }

    /// Read the current consecutive failure count of a key. `None` if the id
    /// is out of range.
    #[must_use]
    pub fn failure_count_of(&self, id: KeyId) -> Option<u32> {
        self.slots.get(id.0).map(|slot| slot.failure_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(label: &str) -> KeyFingerprint {
        KeyFingerprint::from_hex(label)
    }

    fn pool_of(n: usize, policy: PoolPolicy) -> KeyPool {
        let fps = (0..n).map(|i| fp(&format!("kf{i}"))).collect();
        KeyPool::new(ProviderChannel::OpenAi, policy, fps)
    }

    #[test]
    fn channel_roundtrips_through_str() {
        for ch in [
            ProviderChannel::OpenAi,
            ProviderChannel::Anthropic,
            ProviderChannel::Gemini,
        ] {
            assert_eq!(ProviderChannel::parse(ch.as_str()), Some(ch));
        }
    }

    #[test]
    fn channel_parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(
            ProviderChannel::parse("Codex"),
            Some(ProviderChannel::OpenAi)
        );
        assert_eq!(
            ProviderChannel::parse("CLAUDE"),
            Some(ProviderChannel::Anthropic)
        );
        assert_eq!(
            ProviderChannel::parse(" google "),
            Some(ProviderChannel::Gemini)
        );
        assert_eq!(ProviderChannel::parse("mistral"), None);
    }

    #[test]
    fn fingerprint_exposes_only_the_hex_label() {
        let f = fp("abc123");
        assert_eq!(f.as_str(), "abc123");
    }

    #[test]
    fn policy_clamps_zero_threshold_to_one() {
        let p = PoolPolicy::new(0, 1000, 0);
        assert_eq!(p.blacklist_threshold, 1);
    }

    #[test]
    fn empty_pool_selects_empty() {
        let mut pool = pool_of(0, PoolPolicy::default());
        assert_eq!(pool.select(0), Selection::Empty);
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn round_robin_cycles_in_order() {
        let mut pool = pool_of(3, PoolPolicy::default());
        let mut seen = Vec::new();
        for _ in 0..6 {
            match pool.select(0) {
                Selection::Key { id, .. } => seen.push(id.0),
                other => panic!("expected key, got {other:?}"),
            }
        }
        // Two full cycles, strictly in insertion order.
        assert_eq!(seen, vec![0, 1, 2, 0, 1, 2]);
    }

    #[test]
    fn selection_returns_matching_fingerprint() {
        let mut pool = pool_of(2, PoolPolicy::default());
        match pool.select(0) {
            Selection::Key { id, fingerprint } => {
                assert_eq!(id.0, 0);
                assert_eq!(fingerprint.as_str(), "kf0");
            }
            other => panic!("expected key, got {other:?}"),
        }
    }

    #[test]
    fn failures_below_threshold_do_not_blacklist() {
        let mut pool = pool_of(1, PoolPolicy::new(3, 1000, 0));
        let id = KeyId(0);
        assert_eq!(pool.record_failure(id, 0, 0), Some(KeyState::Active));
        assert_eq!(pool.record_failure(id, 0, 0), Some(KeyState::Active));
        assert_eq!(pool.failure_count_of(id), Some(2));
        assert_eq!(pool.state_of(id), Some(KeyState::Active));
    }

    #[test]
    fn threshold_failures_blacklist_with_base_cooldown_when_no_jitter() {
        let mut pool = pool_of(1, PoolPolicy::new(2, 5000, 0));
        let id = KeyId(0);
        pool.record_failure(id, 100, 999);
        let state = pool.record_failure(id, 100, 999).unwrap();
        // jitter disabled -> exactly base cooldown from now.
        assert_eq!(
            state,
            KeyState::Blacklisted {
                cooldown_until_millis: 5100
            }
        );
    }

    #[test]
    fn jitter_is_bounded_and_deterministic_from_seed() {
        let mut pool = pool_of(1, PoolPolicy::new(1, 1000, 10));
        // seed 7 -> 7 % 11 = 7 ms jitter on top of 1000 base from now=0.
        let state = pool.record_failure(KeyId(0), 0, 7).unwrap();
        assert_eq!(
            state,
            KeyState::Blacklisted {
                cooldown_until_millis: 1007
            }
        );

        let mut pool2 = pool_of(1, PoolPolicy::new(1, 1000, 10));
        // seed 100 -> 100 % 11 = 1 ms jitter; always within [0, jitter_max].
        let state2 = pool2.record_failure(KeyId(0), 0, 100).unwrap();
        assert_eq!(
            state2,
            KeyState::Blacklisted {
                cooldown_until_millis: 1001
            }
        );
    }

    #[test]
    fn blacklisted_key_is_skipped_until_cooldown_expires() {
        let mut pool = pool_of(2, PoolPolicy::new(1, 1000, 0));
        // Trip key 0 at t=0 -> cooldown until 1000.
        pool.record_failure(KeyId(0), 0, 0);
        assert_eq!(
            pool.state_of(KeyId(0)),
            Some(KeyState::Blacklisted {
                cooldown_until_millis: 1000
            })
        );

        // At t=500 only key 1 is selectable; repeated selects all land on 1.
        for _ in 0..4 {
            match pool.select(500) {
                Selection::Key { id, .. } => assert_eq!(id.0, 1),
                other => panic!("expected key 1, got {other:?}"),
            }
        }
        assert_eq!(pool.active_count(500), 1);
    }

    #[test]
    fn cooldown_expiry_lazily_restores_key_on_select() {
        let mut pool = pool_of(1, PoolPolicy::new(1, 1000, 0));
        pool.record_failure(KeyId(0), 0, 0);
        // Still cooling down at t=999 -> exhausted.
        assert_eq!(pool.select(999), Selection::Exhausted);
        assert_eq!(pool.active_count(999), 0);
        // At t=1000 cooldown has elapsed -> restored + selectable.
        match pool.select(1000) {
            Selection::Key { id, .. } => assert_eq!(id.0, 0),
            other => panic!("expected restored key, got {other:?}"),
        }
        assert_eq!(pool.state_of(KeyId(0)), Some(KeyState::Active));
        assert_eq!(pool.failure_count_of(KeyId(0)), Some(0));
    }

    #[test]
    fn all_blacklisted_yields_exhausted() {
        let mut pool = pool_of(2, PoolPolicy::new(1, 1000, 0));
        pool.record_failure(KeyId(0), 0, 0);
        pool.record_failure(KeyId(1), 0, 0);
        assert_eq!(pool.select(500), Selection::Exhausted);
        assert_eq!(pool.active_count(500), 0);
    }

    #[test]
    fn success_resets_failure_count_and_restores() {
        let mut pool = pool_of(1, PoolPolicy::new(3, 1000, 0));
        pool.record_failure(KeyId(0), 0, 0);
        pool.record_failure(KeyId(0), 0, 0);
        assert_eq!(pool.failure_count_of(KeyId(0)), Some(2));
        pool.record_success(KeyId(0));
        assert_eq!(pool.failure_count_of(KeyId(0)), Some(0));
        assert_eq!(pool.state_of(KeyId(0)), Some(KeyState::Active));
    }

    #[test]
    fn success_on_blacklisted_key_restores_it_immediately() {
        let mut pool = pool_of(1, PoolPolicy::new(1, 10_000, 0));
        pool.record_failure(KeyId(0), 0, 0);
        assert!(matches!(
            pool.state_of(KeyId(0)),
            Some(KeyState::Blacklisted { .. })
        ));
        // An out-of-band success (e.g. a manual health probe) restores it.
        pool.record_success(KeyId(0));
        assert_eq!(pool.state_of(KeyId(0)), Some(KeyState::Active));
    }

    #[test]
    fn out_of_range_id_is_noop() {
        let mut pool = pool_of(1, PoolPolicy::default());
        assert_eq!(pool.record_failure(KeyId(99), 0, 0), None);
        assert_eq!(pool.state_of(KeyId(99)), None);
        assert_eq!(pool.failure_count_of(KeyId(99)), None);
        // record_success on bad id must not panic.
        pool.record_success(KeyId(99));
    }

    #[test]
    fn failure_then_recovery_then_failure_uses_fresh_counter() {
        let mut pool = pool_of(1, PoolPolicy::new(2, 1000, 0));
        pool.record_failure(KeyId(0), 0, 0); // count=1
        pool.record_success(KeyId(0)); // reset
        pool.record_failure(KeyId(0), 0, 0); // count=1 again, not 2
        assert_eq!(pool.state_of(KeyId(0)), Some(KeyState::Active));
        assert_eq!(pool.failure_count_of(KeyId(0)), Some(1));
    }

    #[test]
    fn cursor_advances_even_when_landing_on_blacklisted_slot() {
        // Regression guard: the cursor must advance by exactly one per call so
        // load stays balanced; skipping must not "stick" the cursor.
        let mut pool = pool_of(3, PoolPolicy::new(1, 10_000, 0));
        pool.record_failure(KeyId(1), 0, 0); // blacklist the middle key
        let mut seen = Vec::new();
        for _ in 0..4 {
            if let Selection::Key { id, .. } = pool.select(0) {
                seen.push(id.0);
            }
        }
        // Cursor visits 0,1(skip->2),2,0 ... active keys only are returned, and
        // both surviving keys appear (no starvation of key 2).
        assert!(seen.contains(&0));
        assert!(seen.contains(&2));
        assert!(!seen.contains(&1));
    }

    #[test]
    fn active_count_counts_expired_cooldown_as_available() {
        let mut pool = pool_of(2, PoolPolicy::new(1, 1000, 0));
        pool.record_failure(KeyId(0), 0, 0);
        // Before expiry: 1 active.
        assert_eq!(pool.active_count(500), 1);
        // After expiry: counted as available again even before select restores.
        assert_eq!(pool.active_count(1000), 2);
    }
}
