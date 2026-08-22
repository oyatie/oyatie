//! Hyperscaler metric emission trait — the canonical surface every oyatie
//! µservice must implement to satisfy
//! `microservices/observability/contracts/metric-naming-convention.md`.
//!
//! # Why this crate exists (PERF-143-002)
//!
//! Before this kernel, the canonical PrometheusRule
//! `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
//! fires alerts against metric families like
//! `<ms>_capability_circuit_state` and `<ms>_responses_429_total`,
//! but no Rust code emitted these metrics. Result: alerts would fire
//! absent-series ("no data") on every µservice — the canonical-base
//! contract was unenforced in the data plane.
//!
//! This kernel declares the shape of the emitter surface. The adapter
//! crate `shared-hyperscaler-metrics-adapter-prometheus` wires it to
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
//! `shared-hyperscaler-metrics-kernel` follows BNF v4.1:
//! `oya-<vertical:shared>-<topic:hyperscaler-metrics>-<layer:kernel>`.
//! Maps to the 13-layer enum (ADR-0105) `domain` layer; the `kernel`
//! suffix is the canonical alias for `domain` per the shared-substrate
//! convention used by `shared-bounded-contexts-check-cli`, etc.
//!
//! # Trait surface — one method per canonical metric family
//!
//! The methods below cover every metric named in
//! `microservices/observability/contracts/metric-naming-convention.md`:
//!
//! | Trait method                                     | Metric (templated)                                  | INV                              |
//! |--------------------------------------------------|-----------------------------------------------------|----------------------------------|
//! | `record_capability_circuit_state`                | `<ms>_capability_circuit_state`                 | INV-CIRCUIT-BREAKER-BULKHEAD     |
//! | `record_capability_retry_budget_exhausted`       | `<ms>_capability_retry_budget_exhausted_total`  | INV-CIRCUIT-BREAKER-BULKHEAD     |
//! | `record_responses_429`                           | `<ms>_responses_429_total`                      | INV-SHUFFLE-SHARDING             |
//! | `record_responses_5xx`                           | `<ms>_responses_5xx_total`                      | INV-FOUR-GOLDEN-SIGNALS          |
//! | `record_responses_total`                         | `<ms>_responses_total`                          | INV-FOUR-GOLDEN-SIGNALS          |
//! | `record_request_success`                         | `<ms>_request_success_total`                    | INV-SLO-ERROR-BUDGET             |
//! | `record_request_total`                           | `<ms>_request_total`                            | INV-SLO-ERROR-BUDGET             |
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

/// The three circuit-breaker states tracked by `<ms>_capability_circuit_state`.
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
    /// A runtime request outcome carried a status code outside the HTTP status
    /// code space used by the REST/gRPC gateway adapters.
    InvalidStatusCode(u16),
    /// A SLO-burn exercise profile was impossible to evaluate.
    InvalidSloProfile { field: String, value: u64 },
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
            MetricsError::InvalidStatusCode(status) => {
                write!(f, "invalid telemetry status code: {status}")
            }
            MetricsError::InvalidSloProfile { field, value } => {
                write!(f, "invalid SLO burn profile field {field:?}: {value}")
            }
            MetricsError::RegistryFailure(msg) => {
                write!(f, "metrics registry failure: {msg}")
            }
        }
    }
}

impl std::error::Error for MetricsError {}

/// A circuit-breaker / retry-budget observation attached to one request
/// outcome.
///
/// This is intentionally low-cardinality: only `capability_id` and canonical
/// [`CircuitState`] are emitted. Tenant/principal/body data never becomes a
/// metric name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackpressureObservation {
    pub capability_id: String,        // data_class: INTERNAL_ONLY
    pub circuit_state: CircuitState,  // data_class: INTERNAL_ONLY
    pub retry_budget_exhausted: bool, // data_class: INTERNAL_ONLY
}

impl BackpressureObservation {
    /// Construct after validating the capability label.
    pub fn new(
        capability_id: impl Into<String>,
        circuit_state: CircuitState,
        retry_budget_exhausted: bool,
    ) -> Result<Self, MetricsError> {
        let capability_id = capability_id.into();
        if capability_id.is_empty() {
            return Err(MetricsError::InvalidLabelValue {
                label: "capability_id".to_string(),
                value: capability_id,
            });
        }
        Ok(Self {
            capability_id,
            circuit_state,
            retry_budget_exhausted,
        })
    }

    /// True when the observation represents active backpressure.
    #[must_use]
    pub const fn is_backpressure(&self) -> bool {
        matches!(self.circuit_state, CircuitState::Open) || self.retry_budget_exhausted
    }
}

/// Runtime request outcome emitted by REST/gRPC gateways.
///
/// The value binds a pre-validated [`RequestTelemetryBinding`] to one tenant
/// and one status outcome. The default [`HyperscalerMetrics::record_request_outcome`]
/// implementation fans this single value out to the canonical counters:
/// request total, request success, response total, 429, 5xx, circuit state, and
/// retry-budget exhaustion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestTelemetryOutcome {
    pub binding: RequestTelemetryBinding, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub status_code: u16,                 // data_class: INTERNAL_ONLY
    pub sli_success: bool,                // data_class: INTERNAL_ONLY
    pub backpressure: Option<BackpressureObservation>, // data_class: INTERNAL_ONLY
}

impl RequestTelemetryOutcome {
    /// Construct a runtime outcome, validating status and tenant labels before
    /// any adapter sees them.
    pub fn new(
        binding: RequestTelemetryBinding,
        tenant_id: impl Into<String>,
        status_code: u16,
        sli_success: bool,
    ) -> Result<Self, MetricsError> {
        Self::with_backpressure(binding, tenant_id, status_code, sli_success, None)
    }

    /// Construct a runtime outcome with optional circuit-breaker/backpressure
    /// metadata.
    pub fn with_backpressure(
        binding: RequestTelemetryBinding,
        tenant_id: impl Into<String>,
        status_code: u16,
        sli_success: bool,
        backpressure: Option<BackpressureObservation>,
    ) -> Result<Self, MetricsError> {
        if !(100..=599).contains(&status_code) {
            return Err(MetricsError::InvalidStatusCode(status_code));
        }
        let tenant_id = tenant_id.into();
        if tenant_id.is_empty() {
            return Err(MetricsError::InvalidLabelValue {
                label: "tenant_id".to_string(),
                value: tenant_id,
            });
        }
        Ok(Self {
            binding,
            tenant_id,
            status_code,
            sli_success,
            backpressure,
        })
    }

    /// True when the outcome should increment the canonical 429 counter.
    #[must_use]
    pub const fn is_429(&self) -> bool {
        self.status_code == 429
    }

    /// True when the outcome should increment the canonical 5xx counter.
    #[must_use]
    pub const fn is_5xx(&self) -> bool {
        self.status_code >= 500 && self.status_code <= 599
    }

    /// True when the outcome contains active backpressure evidence.
    #[must_use]
    pub fn has_backpressure(&self) -> bool {
        self.is_429()
            || self
                .backpressure
                .as_ref()
                .is_some_and(BackpressureObservation::is_backpressure)
    }
}

/// The shared metric emission trait. Every oyatie µservice depends on
/// this trait and accepts an implementation at startup. The adapter
/// crate `shared-hyperscaler-metrics-adapter-prometheus` provides
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
    /// `<ms>_capability_circuit_state{state="open"} > 0`.
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

    /// Increment `<ms>_capability_retry_budget_exhausted_total{capability_id=…}` by 1.
    /// Canonical alert: `OyaCapabilityRetryBudgetExhausted`.
    fn record_capability_retry_budget_exhausted(
        &self,
        capability_id: &str,
    ) -> Result<(), MetricsError>;

    /// Increment `<ms>_responses_429_total{tenant_id=…}` by 1.
    /// Canonical alert: `OyaTenantRateLimit429Surge`.
    fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError>;

    /// Increment `<ms>_responses_5xx_total` by 1.
    /// Canonical alert: `OyaErrors5xxRateSpike`.
    fn record_responses_5xx(&self) -> Result<(), MetricsError>;

    /// Increment `<ms>_responses_total` by 1. Every HTTP/gRPC
    /// response — success or failure — must increment this counter.
    /// Canonical alerts: `OyaErrors5xxRateSpike` denominator,
    /// `OyaTrafficDrop90pct`.
    fn record_responses_total(&self) -> Result<(), MetricsError>;

    /// Increment `<ms>_request_success_total` by 1. Increment only
    /// when the request satisfied the µservice's SLI definition (e.g.
    /// HTTP 2xx within latency budget).
    /// Canonical alerts: `OyaErrorBudgetFastBurn1h14x`,
    /// `OyaErrorBudgetSlowBurn6h6x` (numerator).
    fn record_request_success(&self) -> Result<(), MetricsError>;

    /// Increment `<ms>_request_total` by 1. Every request that
    /// counts toward the SLO denominator (the "valid event" definition
    /// in the OpenSLO manifest) must increment this counter.
    fn record_request_total(&self) -> Result<(), MetricsError>;

    /// Emit the canonical runtime counters for one request outcome.
    ///
    /// This default method is the runtime adapter seam used by REST/gRPC
    /// gateways: it preserves the small trait surface while ensuring every
    /// status outcome increments the same canonical metric families in the same
    /// order.
    fn record_request_outcome(
        &self,
        outcome: &RequestTelemetryOutcome,
    ) -> Result<(), MetricsError> {
        self.record_request_total()?;
        self.record_responses_total()?;
        if outcome.sli_success {
            self.record_request_success()?;
        }
        if outcome.is_429() {
            self.record_responses_429(&outcome.tenant_id)?;
        }
        if outcome.is_5xx() {
            self.record_responses_5xx()?;
        }
        if let Some(backpressure) = &outcome.backpressure {
            self.record_capability_circuit_state(
                &backpressure.capability_id,
                backpressure.circuit_state,
            )?;
            if backpressure.retry_budget_exhausted {
                self.record_capability_retry_budget_exhausted(&backpressure.capability_id)?;
            }
        }
        Ok(())
    }
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

/// Canonical request metric names for a single REST/gRPC operation.
///
/// Runtime adapters use this value as the low-cardinality operation binding
/// before emitting counters through [`HyperscalerMetrics`]. Keeping the metric
/// names in the kernel prevents each REST crate from hand-formatting
/// `<microservice>_*` strings differently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestTelemetryBinding {
    pub microservice: String,           // data_class: INTERNAL_ONLY
    pub operation_id: String,           // data_class: INTERNAL_ONLY
    pub request_total_metric: String,   // data_class: INTERNAL_ONLY
    pub request_success_metric: String, // data_class: INTERNAL_ONLY
    pub responses_total_metric: String, // data_class: INTERNAL_ONLY
    pub responses_5xx_metric: String,   // data_class: INTERNAL_ONLY
    pub responses_429_metric: String,   // data_class: INTERNAL_ONLY
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
    /// (`<slug>_*`) and as the `microservice=` label value.
    #[must_use]
    pub fn microservice(&self) -> &str {
        &self.microservice
    }
}

/// SLO burn exercise profile used by runtime validation tests and pre-prod
/// drills. The default mirrors the common 99.9% request-success SLO and the
/// 14x/6x multi-window burn-rate alert thresholds; it is an exercise contract,
/// not a production SLO claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SloBurnProfile {
    pub target_success_per_million: u32, // data_class: INTERNAL_ONLY
    pub fast_burn_threshold_basis_points: u32, // data_class: INTERNAL_ONLY
    pub slow_burn_threshold_basis_points: u32, // data_class: INTERNAL_ONLY
}

impl Default for SloBurnProfile {
    fn default() -> Self {
        Self {
            target_success_per_million: 999_000,
            fast_burn_threshold_basis_points: 140_000,
            slow_burn_threshold_basis_points: 60_000,
        }
    }
}

impl SloBurnProfile {
    /// Validate target and threshold fields before an exercise report is built.
    pub fn validate(self) -> Result<Self, MetricsError> {
        if self.target_success_per_million >= 1_000_000 {
            return Err(MetricsError::InvalidSloProfile {
                field: "target_success_per_million".to_string(),
                value: u64::from(self.target_success_per_million),
            });
        }
        if self.fast_burn_threshold_basis_points == 0 {
            return Err(MetricsError::InvalidSloProfile {
                field: "fast_burn_threshold_basis_points".to_string(),
                value: 0,
            });
        }
        if self.slow_burn_threshold_basis_points == 0 {
            return Err(MetricsError::InvalidSloProfile {
                field: "slow_burn_threshold_basis_points".to_string(),
                value: 0,
            });
        }
        Ok(self)
    }
}

/// Deterministic SLO burn-rate exercise output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SloBurnAssessment {
    pub total_events: u64,                 // data_class: INTERNAL_ONLY
    pub success_events: u64,               // data_class: INTERNAL_ONLY
    pub error_events: u64,                 // data_class: INTERNAL_ONLY
    pub target_success_per_million: u32,   // data_class: INTERNAL_ONLY
    pub observed_success_per_million: u64, // data_class: INTERNAL_ONLY
    pub observed_error_per_million: u64,   // data_class: INTERNAL_ONLY
    pub budget_error_per_million: u64,     // data_class: INTERNAL_ONLY
    pub burn_rate_basis_points: u64,       // data_class: INTERNAL_ONLY
    pub fast_burn_alert: bool,             // data_class: INTERNAL_ONLY
    pub slow_burn_alert: bool,             // data_class: INTERNAL_ONLY
}

/// Runtime telemetry exercise report emitted by a pre-prod validation run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTelemetryExerciseReport {
    pub microservice: String,               // data_class: INTERNAL_ONLY
    pub operation_id: String,               // data_class: INTERNAL_ONLY
    pub request_total: u64,                 // data_class: INTERNAL_ONLY
    pub request_success: u64,               // data_class: INTERNAL_ONLY
    pub responses_total: u64,               // data_class: INTERNAL_ONLY
    pub responses_429: u64,                 // data_class: INTERNAL_ONLY
    pub responses_5xx: u64,                 // data_class: INTERNAL_ONLY
    pub backpressure_events: u64,           // data_class: INTERNAL_ONLY
    pub circuit_open_events: u64,           // data_class: INTERNAL_ONLY
    pub retry_budget_exhausted_events: u64, // data_class: INTERNAL_ONLY
    pub slo_burn: SloBurnAssessment,        // data_class: INTERNAL_ONLY
}

/// Emit every runtime outcome through a [`HyperscalerMetrics`] implementation
/// and return a deterministic exercise report.
pub fn exercise_runtime_telemetry(
    metrics: &dyn HyperscalerMetrics,
    outcomes: &[RequestTelemetryOutcome],
    profile: SloBurnProfile,
) -> Result<RuntimeTelemetryExerciseReport, MetricsError> {
    let profile = profile.validate()?;
    for outcome in outcomes {
        metrics.record_request_outcome(outcome)?;
    }
    Ok(runtime_telemetry_report(outcomes, profile))
}

/// Build a report without emitting metrics. Useful for validating a captured
/// drill transcript.
#[must_use]
pub fn runtime_telemetry_report(
    outcomes: &[RequestTelemetryOutcome],
    profile: SloBurnProfile,
) -> RuntimeTelemetryExerciseReport {
    let request_total = outcomes.len() as u64;
    let request_success = outcomes
        .iter()
        .filter(|outcome| outcome.sli_success)
        .count() as u64;
    let responses_429 = outcomes.iter().filter(|outcome| outcome.is_429()).count() as u64;
    let responses_5xx = outcomes.iter().filter(|outcome| outcome.is_5xx()).count() as u64;
    let backpressure_events = outcomes
        .iter()
        .filter(|outcome| outcome.has_backpressure())
        .count() as u64;
    let circuit_open_events = outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .backpressure
                .as_ref()
                .is_some_and(|bp| bp.circuit_state == CircuitState::Open)
        })
        .count() as u64;
    let retry_budget_exhausted_events = outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .backpressure
                .as_ref()
                .is_some_and(|bp| bp.retry_budget_exhausted)
        })
        .count() as u64;
    let (microservice, operation_id) = outcomes
        .first()
        .map(|outcome| {
            (
                outcome.binding.microservice.clone(),
                outcome.binding.operation_id.clone(),
            )
        })
        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

    RuntimeTelemetryExerciseReport {
        microservice,
        operation_id,
        request_total,
        request_success,
        responses_total: request_total,
        responses_429,
        responses_5xx,
        backpressure_events,
        circuit_open_events,
        retry_budget_exhausted_events,
        slo_burn: assess_slo_burn(request_total, request_success, profile),
    }
}

/// Raw event counts for one observation window (e.g. 1 h or 6 h).
///
/// Used as an input to [`assess_multiwindow_burn`] so callers can pass
/// independently-windowed counters without re-wrapping them in a
/// [`RequestTelemetryOutcome`] slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowEvents {
    pub total_events: u64,   // data_class: INTERNAL_ONLY
    pub success_events: u64, // data_class: INTERNAL_ONLY
}

/// Result of a multi-window-multi-burn-rate (MWMB) evaluation.
///
/// The canonical Google SRE MWMB pattern fires an alert ONLY when **both**
/// the short window and the long window simultaneously exceed the burn-rate
/// threshold. This AND-gate prevents a short transient spike from producing
/// a false-positive alert.
///
/// `fast_burn_alert` — true IFF both
/// `short_window.burn_rate_basis_points >= profile.fast_burn_threshold_basis_points`
/// AND
/// `long_window.burn_rate_basis_points >= profile.fast_burn_threshold_basis_points`.
///
/// `slow_burn_alert` — analogous for `slow_burn_threshold_basis_points`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiWindowBurnAssessment {
    pub short_window: SloBurnAssessment, // data_class: INTERNAL_ONLY
    pub long_window: SloBurnAssessment,  // data_class: INTERNAL_ONLY
    pub fast_burn_alert: bool,           // data_class: INTERNAL_ONLY
    pub slow_burn_alert: bool,           // data_class: INTERNAL_ONLY
}

/// Evaluate the canonical multi-window-multi-burn-rate (MWMB) alert pattern.
///
/// Computes per-window burn-rates via [`assess_slo_burn`] and combines them
/// with the MWMB AND-gate: an alert fires only when **both** windows exceed
/// the threshold simultaneously.
///
/// # Errors
///
/// Returns [`MetricsError::InvalidSloProfile`] when `profile.validate()` fails.
///
/// # Panics
///
/// Never panics. Zero-traffic windows return `burn_rate_basis_points = 0`
/// (saturating arithmetic — see [`assess_slo_burn`]).
pub fn assess_multiwindow_burn(
    short: WindowEvents,
    long: WindowEvents,
    profile: SloBurnProfile,
) -> Result<MultiWindowBurnAssessment, MetricsError> {
    let profile = profile.validate()?;
    let short_window = assess_slo_burn(short.total_events, short.success_events, profile);
    let long_window = assess_slo_burn(long.total_events, long.success_events, profile);
    let threshold_fast = u64::from(profile.fast_burn_threshold_basis_points);
    let threshold_slow = u64::from(profile.slow_burn_threshold_basis_points);
    let fast_burn_alert = short_window.burn_rate_basis_points >= threshold_fast
        && long_window.burn_rate_basis_points >= threshold_fast;
    let slow_burn_alert = short_window.burn_rate_basis_points >= threshold_slow
        && long_window.burn_rate_basis_points >= threshold_slow;
    Ok(MultiWindowBurnAssessment {
        short_window,
        long_window,
        fast_burn_alert,
        slow_burn_alert,
    })
}

/// Compute an integer burn-rate assessment from request totals.
#[must_use]
pub fn assess_slo_burn(
    total_events: u64,
    success_events: u64,
    profile: SloBurnProfile,
) -> SloBurnAssessment {
    let success_events = success_events.min(total_events);
    let error_events = total_events.saturating_sub(success_events);
    let observed_success_per_million = success_events
        .saturating_mul(1_000_000)
        .checked_div(total_events)
        .unwrap_or(1_000_000);
    let observed_error_per_million = 1_000_000u64.saturating_sub(observed_success_per_million);
    let budget_error_per_million =
        1_000_000u64.saturating_sub(u64::from(profile.target_success_per_million));
    let burn_rate_basis_points = observed_error_per_million
        .saturating_mul(10_000)
        .checked_div(budget_error_per_million)
        .unwrap_or(0);
    SloBurnAssessment {
        total_events,
        success_events,
        error_events,
        target_success_per_million: profile.target_success_per_million,
        observed_success_per_million,
        observed_error_per_million,
        budget_error_per_million,
        burn_rate_basis_points,
        fast_burn_alert: burn_rate_basis_points
            >= u64::from(profile.fast_burn_threshold_basis_points),
        slow_burn_alert: burn_rate_basis_points
            >= u64::from(profile.slow_burn_threshold_basis_points),
    }
}

impl RequestTelemetryBinding {
    /// Build canonical request/response counter names for one low-cardinality
    /// operation id.
    pub fn new(
        context: &MetricsContext,
        operation_id: impl Into<String>,
    ) -> Result<Self, MetricsError> {
        let operation_id = operation_id.into();
        if !is_valid_operation_id(&operation_id) {
            return Err(MetricsError::InvalidLabelValue {
                label: "operation_id".to_string(),
                value: operation_id,
            });
        }
        Ok(Self {
            microservice: context.microservice().to_string(),
            operation_id,
            request_total_metric: metric_name(context, MetricFamily::RequestTotal),
            request_success_metric: metric_name(context, MetricFamily::RequestSuccessTotal),
            responses_total_metric: metric_name(context, MetricFamily::ResponsesTotal),
            responses_5xx_metric: metric_name(context, MetricFamily::Responses5xxTotal),
            responses_429_metric: metric_name(context, MetricFamily::Responses429Total),
        })
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

/// Operation id label check: lowercase dotted or dashed tokens only.
#[must_use]
pub fn is_valid_operation_id(operation_id: &str) -> bool {
    let mut previous_separator = true;
    let mut saw_token = false;
    for character in operation_id.chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            previous_separator = false;
            saw_token = true;
        } else if character == '.' || character == '-' || character == '_' {
            if previous_separator {
                return false;
            }
            previous_separator = true;
        } else {
            return false;
        }
    }
    saw_token && !previous_separator
}

/// Canonical metric names — adapters must construct registry entries
/// from `metric_name(<ms>, <family>)` so the templated form
/// `<ms>_<family>` is the single source of truth.
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
    /// `<ms>_`).
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

/// Build the canonical templated metric name `<ms>_<suffix>`. This
/// is the **only** sanctioned source of metric name strings; adapters
/// MUST NOT format names by hand.
#[must_use]
pub fn metric_name(ctx: &MetricsContext, family: MetricFamily) -> String {
    let slug_underscored = ctx.microservice().replace('-', "_");
    format!("{slug_underscored}_{}", family.suffix())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct RecordingMetrics {
        events: Mutex<Vec<String>>,
    }

    impl RecordingMetrics {
        fn events(&self) -> Vec<String> {
            self.events.lock().unwrap().clone()
        }

        fn push(&self, event: impl Into<String>) {
            self.events.lock().unwrap().push(event.into());
        }
    }

    impl HyperscalerMetrics for RecordingMetrics {
        fn record_capability_circuit_state(
            &self,
            capability_id: &str,
            state: CircuitState,
        ) -> Result<(), MetricsError> {
            self.push(format!("circuit:{capability_id}:{}", state.as_label()));
            Ok(())
        }

        fn record_capability_retry_budget_exhausted(
            &self,
            capability_id: &str,
        ) -> Result<(), MetricsError> {
            self.push(format!("retry_exhausted:{capability_id}"));
            Ok(())
        }

        fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError> {
            self.push(format!("responses_429:{tenant_id}"));
            Ok(())
        }

        fn record_responses_5xx(&self) -> Result<(), MetricsError> {
            self.push("responses_5xx");
            Ok(())
        }

        fn record_responses_total(&self) -> Result<(), MetricsError> {
            self.push("responses_total");
            Ok(())
        }

        fn record_request_success(&self) -> Result<(), MetricsError> {
            self.push("request_success");
            Ok(())
        }

        fn record_request_total(&self) -> Result<(), MetricsError> {
            self.push("request_total");
            Ok(())
        }
    }

    fn runtime_binding() -> RequestTelemetryBinding {
        let ctx = MetricsContext::new("messenger").unwrap();
        RequestTelemetryBinding::new(&ctx, "messenger.post_message").unwrap()
    }

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
            "messenger_capability_circuit_state"
        );
        assert_eq!(
            metric_name(&ctx, MetricFamily::Responses429Total),
            "messenger_responses_429_total"
        );
        assert_eq!(
            metric_name(&ctx, MetricFamily::RequestSuccessTotal),
            "messenger_request_success_total"
        );
    }

    #[test]
    fn metric_name_underscores_hyphenated_slug() {
        // Prometheus metric names cannot contain '-'. The kernel
        // converts hyphens to underscores so `audit-chain` becomes
        // `audit_chain_responses_total` — matching the canonical
        // PrometheusRule's `{__name__=~".+_responses_total"}` regex.
        let ctx = MetricsContext::new("audit-chain").unwrap();
        assert_eq!(
            metric_name(&ctx, MetricFamily::ResponsesTotal),
            "audit_chain_responses_total"
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

    #[test]
    fn request_telemetry_binding_uses_canonical_metric_names() {
        let ctx = MetricsContext::new("message-stream").unwrap();
        let binding = RequestTelemetryBinding::new(&ctx, "messenger.post_message").unwrap();

        assert_eq!(binding.microservice, "message-stream");
        assert_eq!(binding.operation_id, "messenger.post_message");
        assert_eq!(
            binding.request_total_metric,
            "message_stream_request_total"
        );
        assert_eq!(
            binding.request_success_metric,
            "message_stream_request_success_total"
        );
        assert_eq!(
            binding.responses_total_metric,
            "message_stream_responses_total"
        );
        assert_eq!(
            binding.responses_5xx_metric,
            "message_stream_responses_5xx_total"
        );
        assert_eq!(
            binding.responses_429_metric,
            "message_stream_responses_429_total"
        );
    }

    #[test]
    fn operation_id_rejects_empty_uppercase_and_adjacent_separators() {
        assert!(is_valid_operation_id("messenger.post_message"));
        assert!(is_valid_operation_id("community.cast-vote"));
        assert!(!is_valid_operation_id(""));
        assert!(!is_valid_operation_id("Messenger.PostMessage"));
        assert!(!is_valid_operation_id(".messenger"));
        assert!(!is_valid_operation_id("messenger..post"));
        assert!(!is_valid_operation_id("messenger.post."));
    }

    #[test]
    fn runtime_outcome_records_canonical_counters_and_backpressure() {
        let binding = runtime_binding();
        let outcomes = vec![
            RequestTelemetryOutcome::new(binding.clone(), "tenant-a", 202, true).unwrap(),
            RequestTelemetryOutcome::with_backpressure(
                binding.clone(),
                "tenant-a",
                429,
                false,
                Some(
                    BackpressureObservation::new("broker-publish", CircuitState::Open, true)
                        .unwrap(),
                ),
            )
            .unwrap(),
            RequestTelemetryOutcome::new(binding, "tenant-a", 503, false).unwrap(),
        ];
        let metrics = RecordingMetrics::default();

        let report =
            exercise_runtime_telemetry(&metrics, &outcomes, SloBurnProfile::default()).unwrap();

        let events = metrics.events();
        assert_eq!(events.iter().filter(|e| *e == "request_total").count(), 3);
        assert_eq!(events.iter().filter(|e| *e == "responses_total").count(), 3);
        assert_eq!(events.iter().filter(|e| *e == "request_success").count(), 1);
        assert!(events.contains(&"responses_429:tenant-a".to_string()));
        assert!(events.contains(&"responses_5xx".to_string()));
        assert!(events.contains(&"circuit:broker-publish:open".to_string()));
        assert!(events.contains(&"retry_exhausted:broker-publish".to_string()));

        assert_eq!(report.microservice, "messenger");
        assert_eq!(report.operation_id, "messenger.post_message");
        assert_eq!(report.request_total, 3);
        assert_eq!(report.request_success, 1);
        assert_eq!(report.responses_429, 1);
        assert_eq!(report.responses_5xx, 1);
        assert_eq!(report.backpressure_events, 1);
        assert_eq!(report.circuit_open_events, 1);
        assert_eq!(report.retry_budget_exhausted_events, 1);
        assert!(report.slo_burn.fast_burn_alert);
        assert!(report.slo_burn.slow_burn_alert);
    }

    #[test]
    fn runtime_outcome_rejects_invalid_status_and_empty_labels() {
        let binding = runtime_binding();
        assert_eq!(
            RequestTelemetryOutcome::new(binding.clone(), "tenant-a", 99, true),
            Err(MetricsError::InvalidStatusCode(99))
        );
        assert!(matches!(
            RequestTelemetryOutcome::new(binding.clone(), "", 200, true),
            Err(MetricsError::InvalidLabelValue { .. })
        ));
        assert!(matches!(
            BackpressureObservation::new("", CircuitState::Open, false),
            Err(MetricsError::InvalidLabelValue { .. })
        ));
    }

    #[test]
    fn slo_burn_assessment_is_deterministic_for_zero_and_fast_burn_traffic() {
        let zero = assess_slo_burn(0, 0, SloBurnProfile::default());
        assert_eq!(zero.observed_success_per_million, 1_000_000);
        assert_eq!(zero.burn_rate_basis_points, 0);
        assert!(!zero.fast_burn_alert);

        let burn = assess_slo_burn(100, 95, SloBurnProfile::default());
        assert_eq!(burn.error_events, 5);
        assert_eq!(burn.observed_error_per_million, 50_000);
        assert_eq!(burn.burn_rate_basis_points, 500_000);
        assert!(burn.fast_burn_alert);
        assert!(burn.slow_burn_alert);
    }

    #[test]
    fn slo_burn_profile_rejects_impossible_thresholds() {
        assert!(matches!(
            SloBurnProfile {
                target_success_per_million: 1_000_000,
                ..SloBurnProfile::default()
            }
            .validate(),
            Err(MetricsError::InvalidSloProfile { .. })
        ));
        assert!(matches!(
            SloBurnProfile {
                fast_burn_threshold_basis_points: 0,
                ..SloBurnProfile::default()
            }
            .validate(),
            Err(MetricsError::InvalidSloProfile { .. })
        ));
    }

    // -----------------------------------------------------------------------
    // Multi-window-multi-burn-rate (MWMB) tests
    // -----------------------------------------------------------------------

    /// Helper: build WindowEvents where `error_rate_per_million` errors are
    /// injected into `total` requests (saturating).
    fn window_with_error_rate(total: u64, error_rate_per_million: u64) -> WindowEvents {
        let error_events = total
            .saturating_mul(error_rate_per_million)
            .saturating_div(1_000_000);
        let success_events = total.saturating_sub(error_events);
        WindowEvents {
            total_events: total,
            success_events,
        }
    }

    /// 5% error rate, 999_000 target => error_rate / budget = 50_000 / 1_000 = 50x burn.
    /// Both thresholds (14x fast = 140_000 bp, 6x slow = 60_000 bp) are exceeded.
    #[test]
    fn mwmb_both_windows_hot_fires_fast_and_slow_alert() {
        // 5 % error rate = 50x burn on a 99.9 % SLO => exceeds both 14x and 6x
        let hot = window_with_error_rate(10_000, 50_000);
        let result = assess_multiwindow_burn(hot, hot, SloBurnProfile::default()).unwrap();
        assert!(
            result.fast_burn_alert,
            "fast_burn_alert must fire when both windows are hot"
        );
        assert!(
            result.slow_burn_alert,
            "slow_burn_alert must fire when both windows are hot"
        );
    }

    /// (b) Only the short window is hot; long window is healthy.
    /// MWMB AND-gate must suppress the alert.
    #[test]
    fn mwmb_only_short_window_hot_suppresses_alert() {
        let hot = window_with_error_rate(10_000, 50_000); // 50x burn
        let healthy = window_with_error_rate(10_000, 100); // ~0.1x burn (well under threshold)
        let result = assess_multiwindow_burn(hot, healthy, SloBurnProfile::default()).unwrap();
        assert!(
            !result.fast_burn_alert,
            "fast_burn_alert must be suppressed when only short window is hot"
        );
        assert!(
            !result.slow_burn_alert,
            "slow_burn_alert must be suppressed when only short window is hot"
        );
    }

    /// (c) Only the long window is hot; short window is healthy.
    /// MWMB AND-gate must suppress the alert.
    #[test]
    fn mwmb_only_long_window_hot_suppresses_alert() {
        let healthy = window_with_error_rate(10_000, 100); // ~0.1x burn
        let hot = window_with_error_rate(10_000, 50_000); // 50x burn
        let result = assess_multiwindow_burn(healthy, hot, SloBurnProfile::default()).unwrap();
        assert!(
            !result.fast_burn_alert,
            "fast_burn_alert must be suppressed when only long window is hot"
        );
        assert!(
            !result.slow_burn_alert,
            "slow_burn_alert must be suppressed when only long window is hot"
        );
    }

    /// (d) Zero-traffic windows must produce no alert and must not panic.
    #[test]
    fn mwmb_zero_traffic_windows_produce_no_alert() {
        let zero = WindowEvents {
            total_events: 0,
            success_events: 0,
        };
        let result = assess_multiwindow_burn(zero, zero, SloBurnProfile::default()).unwrap();
        assert!(
            !result.fast_burn_alert,
            "fast_burn_alert must not fire on zero traffic"
        );
        assert!(
            !result.slow_burn_alert,
            "slow_burn_alert must not fire on zero traffic"
        );
        // burn_rate_basis_points is 0 for zero-traffic windows
        assert_eq!(result.short_window.burn_rate_basis_points, 0);
        assert_eq!(result.long_window.burn_rate_basis_points, 0);
    }

    /// (e) An invalid profile must return MetricsError::InvalidSloProfile.
    #[test]
    fn mwmb_invalid_profile_returns_error() {
        let invalid_profile = SloBurnProfile {
            target_success_per_million: 1_000_000, // invalid: >= 1M
            ..SloBurnProfile::default()
        };
        let window = WindowEvents {
            total_events: 100,
            success_events: 99,
        };
        assert!(
            matches!(
                assess_multiwindow_burn(window, window, invalid_profile),
                Err(MetricsError::InvalidSloProfile { .. })
            ),
            "assess_multiwindow_burn must propagate InvalidSloProfile"
        );
    }

    /// Extra: verify the short_window and long_window assessments are
    /// independently computed (different inputs, different outputs).
    #[test]
    fn mwmb_per_window_assessments_are_independent() {
        let short = WindowEvents {
            total_events: 1_000,
            success_events: 950, // 5% error => 50x burn on 99.9% SLO
        };
        let long = WindowEvents {
            total_events: 6_000,
            success_events: 5_994, // 0.1% error => ~1x burn on 99.9% SLO
        };
        let result = assess_multiwindow_burn(short, long, SloBurnProfile::default()).unwrap();
        // short window is hot
        assert!(result.short_window.fast_burn_alert);
        // long window is not hot
        assert!(!result.long_window.fast_burn_alert);
        // combined MWMB alert is suppressed
        assert!(!result.fast_burn_alert);
    }
}
