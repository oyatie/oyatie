//! Gateway metrics — dependency-free, tracing/OTel-aligned.
//!
//! Metric families (all prefixed `oya_llm_gateway_`):
//! - `requests_total{group, channel, outcome}` — inbound dispatch outcomes.
//! - `key_success_total{group, key_fp}` — per-key upstream successes.
//! - `key_failure_total{group, key_fp}` — per-key upstream failures.
//! - `retries_total{group}` — failover/next-key rotations performed.
//! - `upstream_latency_seconds{group, channel}` — upstream call latency.
//! - `active_keys{group}` — gauge of currently-selectable keys per group.
//!
//! # Why no `prometheus` dependency
//! Per `docs/ideas/llm-gateway-best-of-both.md` ("would a hyperscaler use this
//! dependency?"): a hyperscaler does NOT add a raw per-service `prometheus`
//! client — it reuses the shared metrics seam or emits via OTel. The shared
//! `oya-shared-hyperscaler-metrics-kernel` trait, however, models a FIXED set
//! of seven canonical families keyed by `microservice`/`tenant_id`/
//! `capability_id`; it cannot express this gateway's per-key-fingerprint and
//! per-group-channel families without distorting them. So this module keeps
//! the gateway-specific families but records them with `std`-only atomic
//! counters + structured `tracing` events (the OTel pipe), and renders a
//! Prometheus-text-compatible exposition from the atomics. No third-party
//! metrics crate is pulled in.
//!
//! TODO(metrics-phase): once the gateway's per-key/per-channel families are
//! folded into the shared OTel/observability pipeline (ADR-0130), emit through
//! an `Arc<dyn HyperscalerMetrics>` for the canonical request/response counters
//! and drop the bespoke text rendering here. Tracked with the quota/usage/
//! model-map phase, NOT this dep-rework.
//!
//! SECURITY: the only key-identifying label is `key_fp`, a non-reversible
//! SHA-256-derived fingerprint (see [`crate::fingerprint_key`]). The raw key,
//! prompt, and response body are NEVER used as labels or otherwise recorded.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Owns the gateway's metric families as `std`-only atomic counters/gauges.
///
/// Cloning shares the same underlying counters (the inner state is `Arc`-wrapped)
/// so a `GatewayMetrics` handed to multiple subsystems aggregates consistently.
#[derive(Clone)]
pub struct GatewayMetrics {
    inner: std::sync::Arc<MetricsInner>,
}

#[derive(Default)]
struct MetricsInner {
    /// `requests_total{group, channel, outcome}` → count.
    requests_total: Mutex<BTreeMap<(String, String, String), u64>>,
    /// `key_success_total{group, key_fp}` → count.
    key_success_total: Mutex<BTreeMap<(String, String), u64>>,
    /// `key_failure_total{group, key_fp}` → count.
    key_failure_total: Mutex<BTreeMap<(String, String), u64>>,
    /// `retries_total{group}` → count.
    retries_total: Mutex<BTreeMap<String, u64>>,
    /// `upstream_latency_seconds{group, channel}` → (sum, count) for an
    /// average + a Prometheus-style `_sum`/`_count` pair.
    upstream_latency: Mutex<BTreeMap<(String, String), LatencyAgg>>,
    /// `active_keys{group}` → gauge value.
    active_keys: Mutex<BTreeMap<String, u64>>,
    /// Monotonic guard so `Default` is observably initialized (kept for parity
    /// with future OTel meter wiring).
    _generation: AtomicU64,
}

#[derive(Clone, Copy, Default)]
struct LatencyAgg {
    sum_seconds: f64,
    count: u64,
}

/// Errors constructing/registering metrics. Retained for API compatibility;
/// the dependency-free implementation never fails to construct.
#[derive(Debug)]
pub struct MetricsError(pub String);

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "metrics error: {}", self.0)
    }
}

impl std::error::Error for MetricsError {}

impl GatewayMetrics {
    /// Construct a fresh metrics handle. Infallible (kept `Result` for API
    /// stability with the composition root / tests).
    pub fn new() -> Result<Self, MetricsError> {
        Ok(GatewayMetrics {
            inner: std::sync::Arc::new(MetricsInner::default()),
        })
    }

    /// Record an inbound dispatch outcome
    /// (`outcome` ∈ `streamed|exhausted|retry_exhausted|no_keys|unauthorized|unknown_group`).
    pub fn record_request(&self, group: &str, channel: &str, outcome: &str) {
        if let Ok(mut map) = self.inner.requests_total.lock() {
            *map.entry((group.to_string(), channel.to_string(), outcome.to_string()))
                .or_insert(0) += 1;
        }
        tracing::info!(
            target: "oya_llm_gateway::metrics",
            metric = "oya_llm_gateway_requests_total",
            group,
            channel,
            outcome,
            "dispatch outcome"
        );
    }

    /// Record a per-key upstream success (label is a hash, never the raw key).
    pub fn record_key_success(&self, group: &str, key_fp: &str) {
        if let Ok(mut map) = self.inner.key_success_total.lock() {
            *map.entry((group.to_string(), key_fp.to_string())).or_insert(0) += 1;
        }
        tracing::debug!(
            target: "oya_llm_gateway::metrics",
            metric = "oya_llm_gateway_key_success_total",
            group,
            key_fp,
            "key success"
        );
    }

    /// Record a per-key upstream failure (label is a hash, never the raw key).
    pub fn record_key_failure(&self, group: &str, key_fp: &str) {
        if let Ok(mut map) = self.inner.key_failure_total.lock() {
            *map.entry((group.to_string(), key_fp.to_string())).or_insert(0) += 1;
        }
        tracing::debug!(
            target: "oya_llm_gateway::metrics",
            metric = "oya_llm_gateway_key_failure_total",
            group,
            key_fp,
            "key failure"
        );
    }

    /// Record one failover next-key rotation.
    pub fn record_retry(&self, group: &str) {
        if let Ok(mut map) = self.inner.retries_total.lock() {
            *map.entry(group.to_string()).or_insert(0) += 1;
        }
        tracing::debug!(
            target: "oya_llm_gateway::metrics",
            metric = "oya_llm_gateway_retries_total",
            group,
            "failover retry"
        );
    }

    /// Observe an upstream latency sample (seconds).
    pub fn observe_upstream_latency(&self, group: &str, channel: &str, seconds: f64) {
        if let Ok(mut map) = self.inner.upstream_latency.lock() {
            let agg = map
                .entry((group.to_string(), channel.to_string()))
                .or_default();
            agg.sum_seconds += seconds;
            agg.count += 1;
        }
        tracing::debug!(
            target: "oya_llm_gateway::metrics",
            metric = "oya_llm_gateway_upstream_latency_seconds",
            group,
            channel,
            seconds,
            "upstream latency"
        );
    }

    /// Set the active-key gauge for a group.
    pub fn set_active_keys(&self, group: &str, count: usize) {
        if let Ok(mut map) = self.inner.active_keys.lock() {
            map.insert(group.to_string(), count as u64);
        }
    }

    /// Render the current metrics in Prometheus text exposition format (the
    /// body of the `/metrics` response). Built purely from the atomic state —
    /// no third-party metrics crate.
    pub fn render(&self) -> Result<String, MetricsError> {
        let mut out = String::new();

        out.push_str("# HELP oya_llm_gateway_requests_total Inbound dispatch requests by group, channel, and outcome.\n");
        out.push_str("# TYPE oya_llm_gateway_requests_total counter\n");
        if let Ok(map) = self.inner.requests_total.lock() {
            for ((group, channel, outcome), v) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_requests_total{{group=\"{}\",channel=\"{}\",outcome=\"{}\"}} {}\n",
                    esc(group), esc(channel), esc(outcome), v
                ));
            }
        }

        out.push_str("# HELP oya_llm_gateway_key_success_total Per-key upstream successes (key_fp is a non-reversible hash).\n");
        out.push_str("# TYPE oya_llm_gateway_key_success_total counter\n");
        if let Ok(map) = self.inner.key_success_total.lock() {
            for ((group, key_fp), v) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_key_success_total{{group=\"{}\",key_fp=\"{}\"}} {}\n",
                    esc(group), esc(key_fp), v
                ));
            }
        }

        out.push_str("# HELP oya_llm_gateway_key_failure_total Per-key upstream failures (key_fp is a non-reversible hash).\n");
        out.push_str("# TYPE oya_llm_gateway_key_failure_total counter\n");
        if let Ok(map) = self.inner.key_failure_total.lock() {
            for ((group, key_fp), v) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_key_failure_total{{group=\"{}\",key_fp=\"{}\"}} {}\n",
                    esc(group), esc(key_fp), v
                ));
            }
        }

        out.push_str("# HELP oya_llm_gateway_retries_total Failover next-key rotations performed.\n");
        out.push_str("# TYPE oya_llm_gateway_retries_total counter\n");
        if let Ok(map) = self.inner.retries_total.lock() {
            for (group, v) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_retries_total{{group=\"{}\"}} {}\n",
                    esc(group), v
                ));
            }
        }

        out.push_str("# HELP oya_llm_gateway_upstream_latency_seconds Upstream call latency in seconds (sum/count).\n");
        out.push_str("# TYPE oya_llm_gateway_upstream_latency_seconds summary\n");
        if let Ok(map) = self.inner.upstream_latency.lock() {
            for ((group, channel), agg) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_upstream_latency_seconds_sum{{group=\"{}\",channel=\"{}\"}} {}\n",
                    esc(group), esc(channel), agg.sum_seconds
                ));
                out.push_str(&format!(
                    "oya_llm_gateway_upstream_latency_seconds_count{{group=\"{}\",channel=\"{}\"}} {}\n",
                    esc(group), esc(channel), agg.count
                ));
            }
        }

        out.push_str("# HELP oya_llm_gateway_active_keys Currently-selectable keys per group.\n");
        out.push_str("# TYPE oya_llm_gateway_active_keys gauge\n");
        if let Ok(map) = self.inner.active_keys.lock() {
            for (group, v) in map.iter() {
                out.push_str(&format!(
                    "oya_llm_gateway_active_keys{{group=\"{}\"}} {}\n",
                    esc(group), v
                ));
            }
        }

        Ok(out)
    }
}

/// Escape a Prometheus label value (`\`, `"`, newline) per the exposition
/// format. Group/channel/outcome/fingerprint values are already constrained,
/// but escaping keeps the output well-formed for arbitrary group names.
fn esc(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_construct_independently() {
        let m = GatewayMetrics::new().expect("construct");
        let _m2 = GatewayMetrics::new().expect("second independent handle");
        drop(m);
    }

    #[test]
    fn render_includes_incremented_series() {
        let m = GatewayMetrics::new().expect("construct");
        m.record_request("codex", "openai", "streamed");
        m.record_key_success("codex", "deadbeefdeadbeef");
        m.record_key_failure("codex", "feedfacefeedface");
        m.record_retry("codex");
        m.observe_upstream_latency("codex", "openai", 0.42);
        m.set_active_keys("codex", 3);

        let text = m.render().expect("render");
        assert!(text.contains("oya_llm_gateway_requests_total"));
        assert!(text.contains("oya_llm_gateway_key_success_total"));
        assert!(text.contains("oya_llm_gateway_retries_total"));
        assert!(text.contains("oya_llm_gateway_upstream_latency_seconds"));
        assert!(text.contains("oya_llm_gateway_active_keys"));
        // Active-keys gauge reflects the set value.
        assert!(text.contains("oya_llm_gateway_active_keys{group=\"codex\"} 3"));
    }

    #[test]
    fn render_never_contains_raw_key_only_fingerprint() {
        let m = GatewayMetrics::new().expect("construct");
        // Simulate using a fingerprint label, never the raw key.
        let fp = crate::fingerprint_key("sk-supersecret-key");
        m.record_key_success("codex", &fp);
        let text = m.render().expect("render");
        assert!(text.contains(&fp));
        assert!(!text.contains("supersecret"));
    }

    #[test]
    fn counters_accumulate_across_calls() {
        let m = GatewayMetrics::new().expect("construct");
        m.record_request("g", "openai", "streamed");
        m.record_request("g", "openai", "streamed");
        let text = m.render().expect("render");
        assert!(text.contains(
            "oya_llm_gateway_requests_total{group=\"g\",channel=\"openai\",outcome=\"streamed\"} 2"
        ));
    }
}
