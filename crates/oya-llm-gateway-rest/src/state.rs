//! Runtime state: per-group key pools + channel adapters + shared metrics.
//!
//! This is where the pure [`oya_llm_gateway_kernel`] state machine meets the
//! async world. Each group owns:
//! - a [`KeyPool`] (kernel) guarded by a `Mutex` for `&mut` transitions,
//! - the in-memory raw keys aligned by [`KeyId`] index to the pool's slots,
//! - a [`ChannelAdapter`] (auth/URL),
//! - the group's [`RetryPolicyConfig`].
//!
//! The runtime injects the wall clock and a jitter seed into the kernel (the
//! kernel itself reads no clock and no RNG), keeping the kernel deterministic.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use oya_llm_gateway_kernel::{
    KeyFingerprint, KeyId, KeyPool, KeyState, PoolPolicy, ProviderChannel, Selection,
};

use crate::auth::AuthVerifier;
use crate::channel::ChannelAdapter;
use crate::config::{GatewayConfig, RetryPolicyConfig};
use crate::keystore::KeyMaterial;
use crate::metrics::GatewayMetrics;

/// Runtime for one group: pool + keys + adapter + policy.
pub struct GroupRuntime {
    name: String,
    adapter: ChannelAdapter,
    retry: RetryPolicyConfig,
    /// Kernel pool + the raw keys indexed by slot. Guarded together so a
    /// selection's [`KeyId`] always indexes a consistent `raw_keys` snapshot.
    inner: Mutex<GroupInner>,
}

struct GroupInner {
    pool: KeyPool,
    /// `raw_keys[i]` is the live key for `KeyId(i)`. Replaced atomically with
    /// the pool on refresh.
    raw_keys: Vec<String>,
    /// Upstream-`Retry-After`-driven cooldown override per slot, in unix
    /// millis. The kernel's intrinsic cooldown is the primary mechanism; this
    /// map *extends* a slot's effective cooldown when the upstream told us to
    /// wait longer than the kernel's policy. Selection skips a slot while
    /// `now < retry_after_until_millis[id]`. Entries are cleared on success or
    /// on natural expiry; the map only ever grows by `len()` entries (one per
    /// key), so its memory is bounded.
    retry_after_until_millis: BTreeMap<usize, u64>, // data_class: INTERNAL_ONLY
}

/// A key chosen for an attempt: its slot id, hash fingerprint, and raw value.
pub struct ChosenKey {
    /// Slot handle to report success/failure against.
    pub id: KeyId,
    /// Hash-only fingerprint for logs/metrics.
    pub fingerprint: String,
    /// The live key (forward to upstream auth only; never log).
    pub raw_key: String,
}

/// Result of asking a group for a key.
pub enum KeyChoice {
    /// A usable key was selected.
    Chosen(ChosenKey),
    /// Every key is blacklisted/in-cooldown.
    Exhausted,
    /// The group has no keys loaded.
    Empty,
}

impl GroupRuntime {
    /// Build a group runtime from its config-derived parts and initial keys.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        adapter: ChannelAdapter,
        retry: RetryPolicyConfig,
        policy: PoolPolicy,
        material: KeyMaterial,
    ) -> Self {
        let (pool, raw_keys) = build_pool(adapter.channel(), policy, &material);
        GroupRuntime {
            name: name.into(),
            adapter,
            retry,
            inner: Mutex::new(GroupInner {
                pool,
                raw_keys,
                retry_after_until_millis: BTreeMap::new(),
            }),
        }
    }

    /// Group name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The channel adapter.
    #[must_use]
    pub fn adapter(&self) -> &ChannelAdapter {
        &self.adapter
    }

    /// The retry policy.
    #[must_use]
    pub fn retry(&self) -> &RetryPolicyConfig {
        &self.retry
    }

    /// Replace the pooled keys (periodic refresh). Preserves the policy; the
    /// round-robin cursor resets (acceptable: refresh is infrequent). Retry-
    /// After overrides are dropped — a fresh key set starts with a clean
    /// cooldown slate.
    pub fn refresh_keys(&self, policy: PoolPolicy, material: &KeyMaterial) {
        let (pool, raw_keys) = build_pool(self.adapter.channel(), policy, material);
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool = pool;
            inner.raw_keys = raw_keys;
            inner.retry_after_until_millis.clear();
        }
    }

    /// Number of currently-selectable keys at the current wall clock,
    /// considering both the kernel state machine AND the Retry-After overrides.
    #[must_use]
    pub fn active_key_count(&self) -> usize {
        let now = now_unix_millis();
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .pool
                    .active_count(now)
                    .saturating_sub(retry_after_active_block_count(&inner, now))
            })
            .unwrap_or(0)
    }

    /// Select the next key for an attempt, injecting the wall clock.
    ///
    /// Walks the kernel's round-robin selection up to `len()` times so a key
    /// whose Retry-After override has not yet elapsed is skipped (not
    /// returned). When every key is either kernel-blacklisted or under an
    /// active Retry-After override, returns [`KeyChoice::Exhausted`].
    pub fn choose_key(&self) -> KeyChoice {
        let now = now_unix_millis();
        let Ok(mut inner) = self.inner.lock() else {
            return KeyChoice::Empty;
        };
        let total = inner.pool.len();
        if total == 0 {
            return KeyChoice::Empty;
        }
        for _ in 0..total {
            match inner.pool.select(now) {
                Selection::Key { id, fingerprint } => {
                    let blocked = inner
                        .retry_after_until_millis
                        .get(&id.0)
                        .is_some_and(|&until| now < until);
                    if blocked {
                        // The kernel will rotate the cursor onward; we just
                        // loop to look at the next slot.
                        continue;
                    }
                    let raw = inner.raw_keys.get(id.0).cloned().unwrap_or_default();
                    return KeyChoice::Chosen(ChosenKey {
                        id,
                        fingerprint: fingerprint.as_str().to_string(),
                        raw_key: raw,
                    });
                }
                Selection::Exhausted => return KeyChoice::Exhausted,
                Selection::Empty => return KeyChoice::Empty,
            }
        }
        KeyChoice::Exhausted
    }

    /// Report a successful upstream call for `id`. Clears any Retry-After
    /// override so the slot is immediately eligible again.
    pub fn record_success(&self, id: KeyId) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool.record_success(id);
            inner.retry_after_until_millis.remove(&id.0);
        }
    }

    /// Report a failed upstream call for `id`, injecting clock + jitter seed.
    pub fn record_failure(&self, id: KeyId) {
        let now = now_unix_millis();
        let jitter = jitter_seed();
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool.record_failure(id, now, jitter);
        }
    }

    /// Report a failed upstream call for `id`, applying the kernel's failure
    /// counter AND (if `retry_after_seconds` is set) extending the slot's
    /// effective cooldown so it is not re-selected until `now + retry_after`.
    /// This propagates an upstream `Retry-After` directly into the gateway's
    /// rotation logic (PRD §4.3 / AC-4.1).
    pub fn record_failure_with_retry_after(&self, id: KeyId, retry_after_seconds: Option<u64>) {
        let now = now_unix_millis();
        let jitter = jitter_seed();
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool.record_failure(id, now, jitter);
            if let Some(secs) = retry_after_seconds {
                let until = now.saturating_add(secs.saturating_mul(1_000));
                let existing = inner
                    .retry_after_until_millis
                    .get(&id.0)
                    .copied()
                    .unwrap_or(0);
                if until > existing {
                    inner.retry_after_until_millis.insert(id.0, until);
                }
            }
        }
    }

    /// The soonest restore time across all keys (kernel cooldown OR Retry-
    /// After override), in seconds-from-now. Used to set the `Retry-After`
    /// header on a 503 when the pool is exhausted (PRD §4.3 / AC-3.5).
    /// Returns `None` if at least one key is currently selectable.
    #[must_use]
    pub fn soonest_restore_seconds(&self) -> Option<u64> {
        let now = now_unix_millis();
        let Ok(inner) = self.inner.lock() else {
            return None;
        };
        let total = inner.pool.len();
        if total == 0 {
            return None;
        }
        let mut soonest: Option<u64> = None;
        for idx in 0..total {
            let key_id = KeyId(idx);
            let kernel_until = match inner.pool.state_of(key_id) {
                Some(KeyState::Active) => None,
                Some(KeyState::Blacklisted {
                    cooldown_until_millis,
                }) if cooldown_until_millis > now => Some(cooldown_until_millis),
                Some(KeyState::Blacklisted { .. }) | None => None,
            };
            let override_until = inner
                .retry_after_until_millis
                .get(&idx)
                .copied()
                .filter(|&until| until > now);
            let combined = match (kernel_until, override_until) {
                // If neither, this key is selectable right now -> the pool is
                // not exhausted; no Retry-After to compute.
                (None, None) => return None,
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
            };
            if let Some(until) = combined {
                soonest = Some(match soonest {
                    Some(prev) => prev.min(until),
                    None => until,
                });
            }
        }
        soonest.map(|until| {
            // Ceiling division of millis -> seconds, minimum 1 so the caller
            // is told to retry at least a second from now.
            let delta = until.saturating_sub(now);
            delta.div_ceil(1_000).max(1)
        })
    }
}

/// How many slots whose kernel state is `Active` are nonetheless blocked by
/// an unexpired Retry-After override. Used by [`GroupRuntime::active_key_count`]
/// so the gauge reflects effective availability.
fn retry_after_active_block_count(inner: &GroupInner, now: u64) -> usize {
    let mut blocked = 0usize;
    for (&idx, &until) in &inner.retry_after_until_millis {
        if until <= now {
            continue;
        }
        // Only subtract from `active_count` when the kernel side reports the
        // slot as Active. A kernel-blacklisted slot was already excluded.
        if let Some(KeyState::Active) = inner.pool.state_of(KeyId(idx)) {
            blocked += 1;
        }
    }
    blocked
}

fn build_pool(
    channel: ProviderChannel,
    policy: PoolPolicy,
    material: &KeyMaterial,
) -> (KeyPool, Vec<String>) {
    let raw_keys: Vec<String> = material
        .raw_keys()
        .into_iter()
        .map(str::to_string)
        .collect();
    let fingerprints: Vec<KeyFingerprint> = material
        .fingerprints()
        .into_iter()
        .map(KeyFingerprint::from_hex)
        .collect();
    let pool = KeyPool::new(channel, policy, fingerprints);
    (pool, raw_keys)
}

/// The whole gateway runtime: groups by name + auth + metrics.
pub struct GatewayState {
    groups: BTreeMap<String, GroupRuntime>,
    auth: AuthVerifier,
    metrics: GatewayMetrics,
}

impl GatewayState {
    /// Assemble from already-constructed parts.
    #[must_use]
    pub fn new(
        groups: BTreeMap<String, GroupRuntime>,
        auth: AuthVerifier,
        metrics: GatewayMetrics,
    ) -> Self {
        GatewayState {
            groups,
            auth,
            metrics,
        }
    }

    /// Look up a group runtime by name.
    #[must_use]
    pub fn group(&self, name: &str) -> Option<&GroupRuntime> {
        self.groups.get(name)
    }

    /// The auth verifier.
    #[must_use]
    pub fn auth(&self) -> &AuthVerifier {
        &self.auth
    }

    /// The metrics handle.
    #[must_use]
    pub fn metrics(&self) -> &GatewayMetrics {
        &self.metrics
    }

    /// Group names (sorted).
    #[must_use]
    pub fn group_names(&self) -> Vec<&str> {
        self.groups.keys().map(String::as_str).collect()
    }

    /// Refresh every group's active-key gauge from current pool state.
    pub fn refresh_active_key_gauges(&self) {
        for (name, group) in &self.groups {
            self.metrics.set_active_keys(name, group.active_key_count());
        }
    }

    /// Record a failure on a group's key, propagating an upstream Retry-After
    /// into the effective cooldown (`Some(seconds)`) or applying just the
    /// kernel's policy when none was provided. No-op for an unknown group.
    pub fn record_failure_with_retry_after(
        &self,
        group: &str,
        id: KeyId,
        retry_after_seconds: Option<u64>,
    ) {
        if let Some(group) = self.groups.get(group) {
            group.record_failure_with_retry_after(id, retry_after_seconds);
        }
    }

    /// The soonest restore time, in seconds-from-now, across `group`'s pool.
    /// `None` if the group is unknown OR at least one key is selectable right
    /// now. Used by the OpenAI handlers to set the 503 `Retry-After` when the
    /// pool is exhausted (PRD §4.3 / AC-3.5).
    #[must_use]
    pub fn soonest_restore_seconds(&self, group: &str) -> Option<u64> {
        self.groups
            .get(group)
            .and_then(|g| g.soonest_restore_seconds())
    }

    /// Build a [`PoolPolicy`] for each group from a validated config. Helper
    /// for the composition root.
    #[must_use]
    pub fn pool_policy_for(config: &GatewayConfig, group_name: &str) -> Option<PoolPolicy> {
        config
            .groups
            .iter()
            .find(|g| g.name == group_name)
            .map(|g| {
                PoolPolicy::new(
                    g.blacklist_threshold,
                    g.cooldown_base_millis,
                    g.cooldown_jitter_millis,
                )
            })
    }
}

/// Current wall clock in unix milliseconds (saturating to 0 before the epoch).
fn now_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A per-call jitter seed. We fold a monotonic counter into the nanosecond
/// clock so successive failures on the same millisecond still get distinct
/// seeds (the kernel only needs a varying value; it bounds it into range).
fn jitter_seed() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    nanos ^ n.wrapping_mul(2_654_435_761)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RetryPolicyConfig;

    fn material(channel: ProviderChannel, keys: &[(&str, &str)]) -> KeyMaterial {
        let mut map = BTreeMap::new();
        for (label, key) in keys {
            map.insert((*label).to_string(), (*key).to_string());
        }
        KeyMaterial::from_map(channel, map)
    }

    fn group(keys: &[(&str, &str)]) -> GroupRuntime {
        let adapter = ChannelAdapter::new(ProviderChannel::OpenAi, "https://api.openai.com", None);
        GroupRuntime::new(
            "codex",
            adapter,
            RetryPolicyConfig::default(),
            PoolPolicy::new(2, 10_000, 0),
            material(ProviderChannel::OpenAi, keys),
        )
    }

    #[test]
    fn choose_key_returns_aligned_raw_key_and_fingerprint() {
        let g = group(&[("a", "sk-aaa")]);
        match g.choose_key() {
            KeyChoice::Chosen(c) => {
                assert_eq!(c.raw_key, "sk-aaa");
                assert_eq!(c.fingerprint, crate::fingerprint_key("sk-aaa"));
                assert_eq!(c.id.0, 0);
            }
            _ => panic!("expected a chosen key"),
        }
    }

    #[test]
    fn empty_group_reports_empty() {
        let g = group(&[]);
        assert!(matches!(g.choose_key(), KeyChoice::Empty));
        assert_eq!(g.active_key_count(), 0);
    }

    #[test]
    fn failures_blacklist_then_exhaust() {
        let g = group(&[("a", "sk-aaa")]);
        // threshold = 2; two failures blacklist the only key.
        let id = match g.choose_key() {
            KeyChoice::Chosen(c) => c.id,
            _ => panic!("expected key"),
        };
        g.record_failure(id);
        g.record_failure(id);
        // Now the only key is in cooldown (10s) → exhausted.
        assert!(matches!(g.choose_key(), KeyChoice::Exhausted));
    }

    #[test]
    fn success_keeps_key_active_and_counted() {
        let g = group(&[("a", "sk-aaa"), ("b", "sk-bbb")]);
        let id = match g.choose_key() {
            KeyChoice::Chosen(c) => c.id,
            _ => panic!("expected key"),
        };
        g.record_failure(id);
        g.record_success(id); // resets
        assert_eq!(g.active_key_count(), 2);
    }

    #[test]
    fn refresh_replaces_keys() {
        let g = group(&[("a", "sk-old")]);
        g.refresh_keys(
            PoolPolicy::new(2, 10_000, 0),
            &material(
                ProviderChannel::OpenAi,
                &[("a", "sk-new-1"), ("b", "sk-new-2")],
            ),
        );
        assert_eq!(g.active_key_count(), 2);
        match g.choose_key() {
            KeyChoice::Chosen(c) => assert!(c.raw_key.starts_with("sk-new")),
            _ => panic!("expected key"),
        }
    }

    #[test]
    fn gateway_state_lookup_and_gauges() {
        let g = group(&[("a", "sk-aaa")]);
        let mut groups = BTreeMap::new();
        groups.insert("codex".to_string(), g);
        let auth = AuthVerifier::new("admin", vec!["ingress-1".to_string()]);
        let metrics = GatewayMetrics::new().expect("metrics");
        let state = GatewayState::new(groups, auth, metrics);
        assert!(state.group("codex").is_some());
        assert!(state.group("missing").is_none());
        assert_eq!(state.group_names(), vec!["codex"]);
        state.refresh_active_key_gauges();
        let text = state.metrics().render().expect("render");
        assert!(text.contains("oya_llm_gateway_active_keys{group=\"codex\"} 1"));
    }

    #[test]
    fn jitter_seed_varies_across_calls() {
        let a = jitter_seed();
        let b = jitter_seed();
        // The monotonic counter guarantees distinct seeds even within a ms.
        assert_ne!(a, b);
    }
}
