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
    KeyFingerprint, KeyId, KeyPool, PoolPolicy, ProviderChannel, Selection,
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
            inner: Mutex::new(GroupInner { pool, raw_keys }),
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
    /// round-robin cursor resets (acceptable: refresh is infrequent).
    pub fn refresh_keys(&self, policy: PoolPolicy, material: &KeyMaterial) {
        let (pool, raw_keys) = build_pool(self.adapter.channel(), policy, material);
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool = pool;
            inner.raw_keys = raw_keys;
        }
    }

    /// Number of currently-selectable keys at the current wall clock.
    #[must_use]
    pub fn active_key_count(&self) -> usize {
        let now = now_unix_millis();
        self.inner
            .lock()
            .map(|inner| inner.pool.active_count(now))
            .unwrap_or(0)
    }

    /// Select the next key for an attempt, injecting the wall clock.
    pub fn choose_key(&self) -> KeyChoice {
        let now = now_unix_millis();
        let Ok(mut inner) = self.inner.lock() else {
            return KeyChoice::Empty;
        };
        match inner.pool.select(now) {
            Selection::Key { id, fingerprint } => {
                let raw = inner.raw_keys.get(id.0).cloned().unwrap_or_default();
                KeyChoice::Chosen(ChosenKey {
                    id,
                    fingerprint: fingerprint.as_str().to_string(),
                    raw_key: raw,
                })
            }
            Selection::Exhausted => KeyChoice::Exhausted,
            Selection::Empty => KeyChoice::Empty,
        }
    }

    /// Report a successful upstream call for `id`.
    pub fn record_success(&self, id: KeyId) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.pool.record_success(id);
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
