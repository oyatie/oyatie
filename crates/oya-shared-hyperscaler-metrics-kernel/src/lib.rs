//! Hyperscaler metric emission trait — the canonical surface every oyatie
//! µservice must implement to satisfy
//! `microservices/observability/contracts/metric-naming-convention.md`.
//!
//! # Why this crate exists (PERF-143-002)
//!
//! Before this kernel, the canonical PrometheusRule
//! `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
//! fires alerts against metric families like
//! `oya_<ms>_capability_circuit_state` and `oya_<ms>_responses_429_total`,
//! but no Rust code emitted these metrics. Result: alerts would fire
//! absent-series ("no data") on every µservice — the canonical-base
//! contract was unenforced in the data plane.
//!
//! This kernel declares the shape of the emitter surface. The adapter
//! crate `oya-shared-hyperscaler-metrics-adapter-prometheus` wires it to
//! the `prometheus` client library. Each µservice depends on the kernel
//! (the trait), accepts an `Arc<dyn HyperscalerMetrics>` at startup, and
//! invokes the per-event methods from its capability execution loop,
//! HTTP/gRPC handlers, retry/circuit-breaker harness, and SLO success-
//! counter sites.
//!
//! # Layer enum
//!
//! Layer `domain` (port-in-kernel, ADR-0056): pure trait + value types,
//! no I/O, no Prometheus dependency. Adapters live one layer out.
//!
//! # Naming justification
//!
//! `oya-shared-hyperscaler-metrics-kernel` follows BNF v4.1:
//! `oya-<vertical:shared>-<topic:hyperscaler-metrics>-<layer:kernel>`.
//! Maps to the 13-layer enum (ADR-0105) `domain` layer; the `kernel`
//! suffix is the canonical alias for `domain` per the shared-substrate
//! convention used by `oya-shared-bounded-contexts-check-cli`, etc.
//!
//! # Trait surface — one method per canonical metric family
//!
//! The methods below cover every metric named in
//! `microservices/observability/contracts/metric-naming-convention.md`:
//!
//! | Trait method                                     | Metric (templated)                                  | INV                              |
//! |--------------------------------------------------|-----------------------------------------------------|----------------------------------|
//! | `record_capability_circuit_state`                | `oya_<ms>_capability_circuit_state`                 | INV-CIRCUIT-BREAKER-BULKHEAD     |
//! | `record_capability_retry_budget_exhausted`       | `oya_<ms>_capability_retry_budget_exhausted_total`  | INV-CIRCUIT-BREAKER-BULKHEAD     |
//! | `record_responses_429`                           | `oya_<ms>_responses_429_total`                      | INV-SHUFFLE-SHARDING             |
//! | `record_responses_5xx`                           | `oya_<ms>_responses_5xx_total`                      | INV-FOUR-GOLDEN-SIGNALS          |
//! | `record_responses_total`                         | `oya_<ms>_responses_total`                          | INV-FOUR-GOLDEN-SIGNALS          |
//! | `record_request_success`                         | `oya_<ms>_request_success_total`                    | INV-SLO-ERROR-BUDGET             |
//! | `record_request_total`                           | `oya_<ms>_request_total`                            | INV-SLO-ERROR-BUDGET             |
//!
//! Every method carries the `microservice` label automatically (the
//! constructor binds it once); per-event labels (`capability_id`,
//! `tenant_id`, `state`) are passed explicitly.
//!
//! # References
//!
//! - `microservices/observability/contracts/metric-naming-convention.md`
//! - `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
//! - `specs/hyperscaler-architecture-invariants.json`
//! - ADR-0064 — canonical-base-and-localization-packs.
//! - ADR-0128 — hyperscaler architecture invariants.
//! - ADR-0139 — agentic SLO-gated promotion.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;

/// The three circuit-breaker states tracked by `oya_<ms>_capability_circuit_state`.
///
/// Each value renders to the canonical `state` label literal expected by
/// the canonical PrometheusRule (`state="open"`, `state="half_open"`,
/// `state="closed"`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CircuitState {
    Closed,
    HalfOpen,
    Open,
}

impl CircuitState {
    /// Canonical label literal — must match the PrometheusRule's
    /// `state="<value>"` matcher exactly.
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            CircuitState::Closed => "closed",
            CircuitState::HalfOpen => "half_open",
            CircuitState::Open => "open",
        }
    }
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_label())
    }
}

/// Error type returned by adapters when label values violate the
/// canonical naming convention. Kernel callers see this when, e.g., a
/// capability_id contains characters disallowed by Prometheus label
/// rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetricsError {
    /// A label value was rejected by the adapter (empty string,
    /// non-UTF-8 control chars, etc.). Carries the offending label name.
    InvalidLabelValue { label: String, value: String },
    /// An adapter-level registry error occurred during construction
    /// (duplicate metric registration, invalid metric name).
    RegistryFailure(String),
}

impl fmt::Display for MetricsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricsError::InvalidLabelValue { label, value } => {
                write!(f, "invalid label value for {label:?}: {value:?}")
            }
            MetricsError::RegistryFailure(msg) => {
                write!(f, "metrics registry failure: {msg}")
            }
        }
    }
}

impl std::error::Error for MetricsError {}

/// The shared metric emission trait. Every oyatie µservice depends on
/// this trait and accepts an implementation at startup. The adapter
/// crate `oya-shared-hyperscaler-metrics-adapter-prometheus` provides
/// the reference impl.
///
/// All methods are `&self` (interior mutability via the adapter's
/// `prometheus::Registry` + atomic counters) so impls can be shared via
/// `Arc<dyn HyperscalerMetrics + Send + Sync>` across async tasks.
///
/// **The `microservice` label is bound once at construction** (see
/// `MetricsContext`); callers never pass it explicitly, which closes
/// the "wrong microservice label" foot-gun.
pub trait HyperscalerMetrics: Send + Sync + 'static {
    /// Set the circuit-breaker state gauge for `capability_id`. The
    /// canonical PrometheusRule `OyaCapabilityCircuitOpen` alerts on any
    /// `oya_<ms>_capability_circuit_state{state="open"} > 0`.
    ///
    /// Adapter contract: writes `1.0` to the gauge for the supplied
    /// `state` and `0.0` to the gauge for the other two states (so the
    /// alert's `state="open"` filter sees a stable "1=open, 0=not open"
    /// signal).
    fn record_capability_circuit_state(
        &self,
        capability_id: &str,
        state: CircuitState,
    ) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_capability_retry_budget_exhausted_total{capability_id=…}` by 1.
    /// Canonical alert: `OyaCapabilityRetryBudgetExhausted`.
    fn record_capability_retry_budget_exhausted(
        &self,
        capability_id: &str,
    ) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_responses_429_total{tenant_id=…}` by 1.
    /// Canonical alert: `OyaTenantRateLimit429Surge`.
    fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_responses_5xx_total` by 1.
    /// Canonical alert: `OyaErrors5xxRateSpike`.
    fn record_responses_5xx(&self) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_responses_total` by 1. Every HTTP/gRPC
    /// response — success or failure — must increment this counter.
    /// Canonical alerts: `OyaErrors5xxRateSpike` denominator,
    /// `OyaTrafficDrop90pct`.
    fn record_responses_total(&self) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_request_success_total` by 1. Increment only
    /// when the request satisfied the µservice's SLI definition (e.g.
    /// HTTP 2xx within latency budget).
    /// Canonical alerts: `OyaErrorBudgetFastBurn1h14x`,
    /// `OyaErrorBudgetSlowBurn6h6x` (numerator).
    fn record_request_success(&self) -> Result<(), MetricsError>;

    /// Increment `oya_<ms>_request_total` by 1. Every request that
    /// counts toward the SLO denominator (the "valid event" definition
    /// in the OpenSLO manifest) must increment this counter.
    fn record_request_total(&self) -> Result<(), MetricsError>;
}

/// Value-typed binding of the `microservice` label, plus a sanity check
/// that the slug matches the canonical regex `^[a-z][a-z0-9-]*$`
/// (mirrors `manifest.json#microservice`).
///
/// Adapters store this once at construction so per-event calls cannot
/// stamp the wrong label.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MetricsContext {
    microservice: String,
}

impl MetricsContext {
    /// Construct after validating the slug. Returns
    /// `Err(MetricsError::InvalidLabelValue)` if the slug is empty or
    /// contains characters outside `[a-z0-9-]` (or starts with a digit
    /// or hyphen).
    pub fn new(microservice: impl Into<String>) -> Result<Self, MetricsError> {
        let microservice = microservice.into();
        if !is_canonical_microservice_slug(&microservice) {
            return Err(MetricsError::InvalidLabelValue {
                label: "microservice".to_string(),
                value: microservice,
            });
        }
        Ok(MetricsContext { microservice })
    }

    /// The validated slug; safe to use as both metric-name interpolation
    /// (`oya_<slug>_*`) and as the `microservice=` label value.
    #[must_use]
    pub fn microservice(&self) -> &str {
        &self.microservice
    }
}

/// Canonical microservice slug check: `^[a-z][a-z0-9-]*$`.
#[must_use]
pub fn is_canonical_microservice_slug(slug: &str) -> bool {
    let mut chars = slug.chars();
    match chars.next() {
        None => return false,
        Some(c) if !c.is_ascii_lowercase() => return false,
        Some(_) => {}
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Canonical metric names — adapters must construct registry entries
/// from `metric_name(<ms>, <family>)` so the templated form
/// `oya_<ms>_<family>` is the single source of truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MetricFamily {
    CapabilityCircuitState,
    CapabilityRetryBudgetExhaustedTotal,
    Responses429Total,
    Responses5xxTotal,
    ResponsesTotal,
    RequestSuccessTotal,
    RequestTotal,
}

impl MetricFamily {
    /// The trailing fragment of the canonical metric name (after
    /// `oya_<ms>_`).
    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            MetricFamily::CapabilityCircuitState => "capability_circuit_state",
            MetricFamily::CapabilityRetryBudgetExhaustedTotal => {
                "capability_retry_budget_exhausted_total"
            }
            MetricFamily::Responses429Total => "responses_429_total",
            MetricFamily::Responses5xxTotal => "responses_5xx_total",
            MetricFamily::ResponsesTotal => "responses_total",
            MetricFamily::RequestSuccessTotal => "request_success_total",
            MetricFamily::RequestTotal => "request_total",
        }
    }

    /// All seven canonical families in deterministic order. Adapters use
    /// this to register every metric at startup.
    #[must_use]
    pub const fn canonical_set() -> &'static [MetricFamily] {
        &[
            MetricFamily::CapabilityCircuitState,
            MetricFamily::CapabilityRetryBudgetExhaustedTotal,
            MetricFamily::Responses429Total,
            MetricFamily::Responses5xxTotal,
            MetricFamily::ResponsesTotal,
            MetricFamily::RequestSuccessTotal,
            MetricFamily::RequestTotal,
        ]
    }
}

/// Build the canonical templated metric name `oya_<ms>_<suffix>`. This
/// is the **only** sanctioned source of metric name strings; adapters
/// MUST NOT format names by hand.
#[must_use]
pub fn metric_name(ctx: &MetricsContext, family: MetricFamily) -> String {
    let slug_underscored = ctx.microservice().replace('-', "_");
    format!("oya_{slug_underscored}_{}", family.suffix())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_state_labels_match_prometheus_rule() {
        // The canonical PrometheusRule filters `state="open"`; verify
        // we render exactly that token.
        assert_eq!(CircuitState::Open.as_label(), "open");
        assert_eq!(CircuitState::HalfOpen.as_label(), "half_open");
        assert_eq!(CircuitState::Closed.as_label(), "closed");
    }

    #[test]
    fn metrics_context_accepts_canonical_slug() {
        let ctx = MetricsContext::new("messenger").unwrap();
        assert_eq!(ctx.microservice(), "messenger");

        let ctx = MetricsContext::new("audit-chain").unwrap();
        assert_eq!(ctx.microservice(), "audit-chain");
    }

    #[test]
    fn metrics_context_rejects_invalid_slug() {
        assert!(MetricsContext::new("").is_err());
        assert!(MetricsContext::new("-bad").is_err());
        assert!(MetricsContext::new("1bad").is_err());
        assert!(MetricsContext::new("Bad").is_err());
        assert!(MetricsContext::new("a_b").is_err());
        assert!(MetricsContext::new("a.b").is_err());
    }

    #[test]
    fn metric_name_renders_canonical_template() {
        let ctx = MetricsContext::new("messenger").unwrap();
        assert_eq!(
            metric_name(&ctx, MetricFamily::CapabilityCircuitState),
            "oya_messenger_capability_circuit_state"
        );
        assert_eq!(
            metric_name(&ctx, MetricFamily::Responses429Total),
            "oya_messenger_responses_429_total"
        );
        assert_eq!(
            metric_name(&ctx, MetricFamily::RequestSuccessTotal),
            "oya_messenger_request_success_total"
        );
    }

    #[test]
    fn metric_name_underscores_hyphenated_slug() {
        // Prometheus metric names cannot contain '-'. The kernel
        // converts hyphens to underscores so `audit-chain` becomes
        // `oya_audit_chain_responses_total` — matching the canonical
        // PrometheusRule's `{__name__=~"oya_.+_responses_total"}` regex.
        let ctx = MetricsContext::new("audit-chain").unwrap();
        assert_eq!(
            metric_name(&ctx, MetricFamily::ResponsesTotal),
            "oya_audit_chain_responses_total"
        );
    }

    #[test]
    fn canonical_set_contains_all_seven_families() {
        assert_eq!(MetricFamily::canonical_set().len(), 7);
    }

    #[test]
    fn canonical_microservice_slug_regex() {
        assert!(is_canonical_microservice_slug("ontology"));
        assert!(is_canonical_microservice_slug("ops-portal"));
        assert!(is_canonical_microservice_slug("ms123"));
        assert!(!is_canonical_microservice_slug(""));
        assert!(!is_canonical_microservice_slug("UPPER"));
        assert!(!is_canonical_microservice_slug("with_underscore"));
        assert!(!is_canonical_microservice_slug("9start-digit"));
        assert!(!is_canonical_microservice_slug("-start-hyphen"));
    }
}
