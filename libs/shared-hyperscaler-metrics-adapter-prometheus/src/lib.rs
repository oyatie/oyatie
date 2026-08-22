//! Prometheus 0.13.x reference adapter for `HyperscalerMetrics`.
//!
//! Wires the `shared-hyperscaler-metrics-kernel` trait to a
//! `prometheus::Registry` with the canonical metric families declared in
//! `microservices/observability/contracts/metric-naming-convention.md`.
//!
//! # Layer
//!
//! Layer `adapter` (outside the kernel boundary per ADR-0056). The kernel
//! defines the surface; this crate is the I/O-bearing impl.
//!
//! # Wiring contract
//!
//! Every µservice constructs ONE adapter per process at startup:
//!
//! ```ignore
//! use shared_hyperscaler_metrics_kernel::MetricsContext;
//! use shared_hyperscaler_metrics_adapter_prometheus::PrometheusHyperscalerMetrics;
//! use std::sync::Arc;
//!
//! let registry = prometheus::Registry::new();
//! let ctx = MetricsContext::new("messenger").expect("canonical slug");
//! let adapter = Arc::new(
//!     PrometheusHyperscalerMetrics::register(&registry, ctx)
//!         .expect("adapter registers cleanly")
//! );
//! // pass `adapter` into capability-execution / HTTP handler / SLI sites
//! ```
//!
//! The `prometheus::Registry` is then scraped by the `/metrics` endpoint
//! that Grafana Alloy ingests per ADR-0139 Layer A.
//!
//! # References
//!
//! - <https://docs.rs/prometheus/0.13.4/prometheus/> — client library docs.
//! - `microservices/observability/contracts/metric-naming-convention.md`.
//! - ADR-0064 (canonical-base) + ADR-0128 (INV gates) + ADR-0139 (SLO gate).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_hyperscaler_metrics_kernel::{
    BackpressureObservation, CircuitState, HyperscalerMetrics, MetricFamily, MetricsContext,
    MetricsError, RequestTelemetryBinding, RequestTelemetryOutcome, metric_name,
};
use prometheus::{GaugeVec, IntCounter, IntCounterVec, Opts, Registry};

/// Reference impl: every metric family in
/// `MetricFamily::canonical_set()` is registered against a shared
/// `prometheus::Registry`.
pub struct PrometheusHyperscalerMetrics {
    ctx: MetricsContext,
    /// `<ms>_capability_circuit_state{capability_id, state}` — gauge.
    capability_circuit_state: GaugeVec,
    /// `<ms>_capability_retry_budget_exhausted_total{capability_id}` — counter.
    capability_retry_budget_exhausted: IntCounterVec,
    /// `<ms>_responses_429_total{tenant_id}` — counter.
    responses_429: IntCounterVec,
    /// `<ms>_responses_5xx_total` — counter.
    responses_5xx: IntCounter,
    /// `<ms>_responses_total` — counter.
    responses_total: IntCounter,
    /// `<ms>_request_success_total` — counter.
    request_success_total: IntCounter,
    /// `<ms>_request_total` — counter.
    request_total: IntCounter,
}

impl PrometheusHyperscalerMetrics {
    /// Register every canonical metric family against `registry` and
    /// return a wired adapter. Idempotency: prometheus 0.13 returns
    /// `AlreadyReg` if the metric was already registered — we propagate
    /// that as `MetricsError::RegistryFailure` so callers can detect
    /// duplicate construction at startup (always a programming bug).
    pub fn register(registry: &Registry, ctx: MetricsContext) -> Result<Self, MetricsError> {
        let microservice_label = ctx.microservice().to_string();

        // INV-CIRCUIT-BREAKER-BULKHEAD
        let capability_circuit_state = GaugeVec::new(
            Opts::new(
                metric_name(&ctx, MetricFamily::CapabilityCircuitState),
                "Capability circuit-breaker state per (capability_id, state). \
                 1.0 = state active; 0.0 = state inactive. Canonical alert: \
                 OyaCapabilityCircuitOpen.",
            )
            .const_label("microservice", &microservice_label),
            &["capability_id", "state"],
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("gauge_vec init: {e}")))?;
        registry
            .register(Box::new(capability_circuit_state.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register circuit_state: {e}")))?;

        let capability_retry_budget_exhausted = IntCounterVec::new(
            Opts::new(
                metric_name(&ctx, MetricFamily::CapabilityRetryBudgetExhaustedTotal),
                "Capability retry-budget exhausted events per capability_id. \
                 Canonical alert: OyaCapabilityRetryBudgetExhausted.",
            )
            .const_label("microservice", &microservice_label),
            &["capability_id"],
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter_vec init: {e}")))?;
        registry
            .register(Box::new(capability_retry_budget_exhausted.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register retry_budget: {e}")))?;

        // INV-SHUFFLE-SHARDING
        let responses_429 = IntCounterVec::new(
            Opts::new(
                metric_name(&ctx, MetricFamily::Responses429Total),
                "429 responses per tenant_id. Canonical alert: \
                 OyaTenantRateLimit429Surge.",
            )
            .const_label("microservice", &microservice_label),
            &["tenant_id"],
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter_vec 429 init: {e}")))?;
        registry
            .register(Box::new(responses_429.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register 429: {e}")))?;

        // INV-FOUR-GOLDEN-SIGNALS
        let responses_5xx = IntCounter::with_opts(
            Opts::new(
                metric_name(&ctx, MetricFamily::Responses5xxTotal),
                "5xx responses total. Canonical alert: OyaErrors5xxRateSpike.",
            )
            .const_label("microservice", &microservice_label),
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter 5xx init: {e}")))?;
        registry
            .register(Box::new(responses_5xx.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register 5xx: {e}")))?;

        let responses_total = IntCounter::with_opts(
            Opts::new(
                metric_name(&ctx, MetricFamily::ResponsesTotal),
                "Total responses (denominator for errors and traffic). \
                 Canonical alerts: OyaErrors5xxRateSpike, OyaTrafficDrop90pct.",
            )
            .const_label("microservice", &microservice_label),
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter total init: {e}")))?;
        registry
            .register(Box::new(responses_total.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register total: {e}")))?;

        // INV-SLO-ERROR-BUDGET
        let request_success_total = IntCounter::with_opts(
            Opts::new(
                metric_name(&ctx, MetricFamily::RequestSuccessTotal),
                "SLI numerator: requests satisfying the per-µservice SLI. \
                 Canonical alerts: OyaErrorBudgetFastBurn1h14x, \
                 OyaErrorBudgetSlowBurn6h6x.",
            )
            .const_label("microservice", &microservice_label),
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter success init: {e}")))?;
        registry
            .register(Box::new(request_success_total.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register success: {e}")))?;

        let request_total = IntCounter::with_opts(
            Opts::new(
                metric_name(&ctx, MetricFamily::RequestTotal),
                "SLI denominator: every valid event in scope of the SLO.",
            )
            .const_label("microservice", &microservice_label),
        )
        .map_err(|e| MetricsError::RegistryFailure(format!("counter req init: {e}")))?;
        registry
            .register(Box::new(request_total.clone()))
            .map_err(|e| MetricsError::RegistryFailure(format!("register req: {e}")))?;

        Ok(Self {
            ctx,
            capability_circuit_state,
            capability_retry_budget_exhausted,
            responses_429,
            responses_5xx,
            responses_total,
            request_success_total,
            request_total,
        })
    }

    /// Expose the bound microservice slug (for debug + audit).
    #[must_use]
    pub fn microservice(&self) -> &str {
        self.ctx.microservice()
    }
}

fn require_non_empty(label: &str, value: &str) -> Result<(), MetricsError> {
    if value.is_empty() {
        Err(MetricsError::InvalidLabelValue {
            label: label.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(())
    }
}

impl HyperscalerMetrics for PrometheusHyperscalerMetrics {
    fn record_capability_circuit_state(
        &self,
        capability_id: &str,
        state: CircuitState,
    ) -> Result<(), MetricsError> {
        require_non_empty("capability_id", capability_id)?;
        // Stable signal contract: set the supplied state to 1.0 and the
        // other two to 0.0. The canonical PrometheusRule filters
        // `state="open"` and expects "1=open, 0=not open" — without
        // resetting the others, a flip from open→closed would leave the
        // open gauge at 1.
        for s in [
            CircuitState::Closed,
            CircuitState::HalfOpen,
            CircuitState::Open,
        ] {
            let value = if s == state { 1.0 } else { 0.0 };
            self.capability_circuit_state
                .with_label_values(&[capability_id, s.as_label()])
                .set(value);
        }
        Ok(())
    }

    fn record_capability_retry_budget_exhausted(
        &self,
        capability_id: &str,
    ) -> Result<(), MetricsError> {
        require_non_empty("capability_id", capability_id)?;
        self.capability_retry_budget_exhausted
            .with_label_values(&[capability_id])
            .inc();
        Ok(())
    }

    fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError> {
        require_non_empty("tenant_id", tenant_id)?;
        self.responses_429.with_label_values(&[tenant_id]).inc();
        Ok(())
    }

    fn record_responses_5xx(&self) -> Result<(), MetricsError> {
        self.responses_5xx.inc();
        Ok(())
    }

    fn record_responses_total(&self) -> Result<(), MetricsError> {
        self.responses_total.inc();
        Ok(())
    }

    fn record_request_success(&self) -> Result<(), MetricsError> {
        self.request_success_total.inc();
        Ok(())
    }

    fn record_request_total(&self) -> Result<(), MetricsError> {
        self.request_total.inc();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::core::Collector;

    fn fresh_adapter(slug: &str) -> (Registry, PrometheusHyperscalerMetrics) {
        let registry = Registry::new();
        let ctx = MetricsContext::new(slug).unwrap();
        let adapter = PrometheusHyperscalerMetrics::register(&registry, ctx).unwrap();
        (registry, adapter)
    }

    fn binding(slug: &str, operation_id: &str) -> RequestTelemetryBinding {
        let ctx = MetricsContext::new(slug).unwrap();
        RequestTelemetryBinding::new(&ctx, operation_id).unwrap()
    }

    fn counter_value(registry: &Registry, name: &str) -> f64 {
        registry
            .gather()
            .iter()
            .find(|mf| mf.get_name() == name)
            .and_then(|mf| mf.get_metric().first())
            .map(|metric| metric.get_counter().get_value())
            .unwrap_or(0.0)
    }

    fn labeled_counter_value(
        registry: &Registry,
        name: &str,
        label_name: &str,
        label_value: &str,
    ) -> f64 {
        registry
            .gather()
            .iter()
            .find(|mf| mf.get_name() == name)
            .and_then(|mf| {
                mf.get_metric().iter().find(|metric| {
                    metric.get_label().iter().any(|label| {
                        label.get_name() == label_name && label.get_value() == label_value
                    })
                })
            })
            .map(|metric| metric.get_counter().get_value())
            .unwrap_or(0.0)
    }

    fn circuit_gauge_value(registry: &Registry, capability_id: &str, state: &str) -> f64 {
        registry
            .gather()
            .iter()
            .find(|mf| mf.get_name() == "messenger_capability_circuit_state")
            .and_then(|mf| {
                mf.get_metric().iter().find(|metric| {
                    let has_capability = metric.get_label().iter().any(|label| {
                        label.get_name() == "capability_id" && label.get_value() == capability_id
                    });
                    let has_state = metric
                        .get_label()
                        .iter()
                        .any(|label| label.get_name() == "state" && label.get_value() == state);
                    has_capability && has_state
                })
            })
            .map(|metric| metric.get_gauge().get_value())
            .unwrap_or(0.0)
    }

    #[test]
    fn register_succeeds_for_canonical_slug() {
        let (_registry, adapter) = fresh_adapter("messenger");
        assert_eq!(adapter.microservice(), "messenger");
    }

    #[test]
    fn duplicate_register_against_same_registry_errors() {
        let registry = Registry::new();
        let ctx = MetricsContext::new("messenger").unwrap();
        let _first = PrometheusHyperscalerMetrics::register(&registry, ctx.clone()).unwrap();
        // PrometheusHyperscalerMetrics does NOT impl Debug (the inner
        // prometheus types are not all Debug), so .unwrap_err() is
        // unavailable. Pattern-match the Result instead.
        match PrometheusHyperscalerMetrics::register(&registry, ctx) {
            Err(MetricsError::RegistryFailure(_)) => {}
            Err(other) => panic!("expected RegistryFailure, got {other:?}"),
            Ok(_) => panic!("duplicate register must error"),
        }
    }

    #[test]
    fn circuit_state_gauge_reflects_state_transitions() {
        let (registry, adapter) = fresh_adapter("messenger");
        adapter
            .record_capability_circuit_state("llm-anthropic", CircuitState::Open)
            .unwrap();

        // Scrape the registry: the open gauge should be 1, others 0.
        let gathered = registry.gather();
        let circuit_metric = gathered
            .iter()
            .find(|mf| mf.get_name() == "messenger_capability_circuit_state")
            .expect("metric family present");
        let mut found_open = false;
        let mut found_closed = false;
        for m in circuit_metric.get_metric() {
            let labels: Vec<(&str, &str)> = m
                .get_label()
                .iter()
                .map(|p| (p.get_name(), p.get_value()))
                .collect();
            let state = labels.iter().find(|(k, _)| *k == "state").map(|(_, v)| *v);
            let value = m.get_gauge().get_value();
            if state == Some("open") {
                assert_eq!(value, 1.0);
                found_open = true;
            }
            if state == Some("closed") {
                assert_eq!(value, 0.0);
                found_closed = true;
            }
        }
        assert!(found_open && found_closed);

        // Flip to closed; the open gauge should now report 0.
        adapter
            .record_capability_circuit_state("llm-anthropic", CircuitState::Closed)
            .unwrap();
        let gathered = registry.gather();
        let circuit_metric = gathered
            .iter()
            .find(|mf| mf.get_name() == "messenger_capability_circuit_state")
            .unwrap();
        for m in circuit_metric.get_metric() {
            let labels: Vec<(&str, &str)> = m
                .get_label()
                .iter()
                .map(|p| (p.get_name(), p.get_value()))
                .collect();
            let state = labels.iter().find(|(k, _)| *k == "state").map(|(_, v)| *v);
            let value = m.get_gauge().get_value();
            if state == Some("open") {
                assert_eq!(value, 0.0);
            }
            if state == Some("closed") {
                assert_eq!(value, 1.0);
            }
        }
    }

    #[test]
    fn responses_429_counter_increments_per_tenant() {
        let (registry, adapter) = fresh_adapter("messenger");
        adapter.record_responses_429("tenant-a").unwrap();
        adapter.record_responses_429("tenant-a").unwrap();
        adapter.record_responses_429("tenant-b").unwrap();

        let gathered = registry.gather();
        let metric = gathered
            .iter()
            .find(|mf| mf.get_name() == "messenger_responses_429_total")
            .unwrap();
        let mut tenant_a_value = 0;
        let mut tenant_b_value = 0;
        for m in metric.get_metric() {
            let labels: Vec<(&str, &str)> = m
                .get_label()
                .iter()
                .map(|p| (p.get_name(), p.get_value()))
                .collect();
            let tenant = labels
                .iter()
                .find(|(k, _)| *k == "tenant_id")
                .map(|(_, v)| *v);
            let value = m.get_counter().get_value() as u64;
            if tenant == Some("tenant-a") {
                tenant_a_value = value;
            }
            if tenant == Some("tenant-b") {
                tenant_b_value = value;
            }
        }
        assert_eq!(tenant_a_value, 2);
        assert_eq!(tenant_b_value, 1);
    }

    #[test]
    fn empty_label_value_rejected() {
        let (_registry, adapter) = fresh_adapter("messenger");
        assert!(matches!(
            adapter.record_responses_429(""),
            Err(MetricsError::InvalidLabelValue { .. })
        ));
        assert!(matches!(
            adapter.record_capability_circuit_state("", CircuitState::Open),
            Err(MetricsError::InvalidLabelValue { .. })
        ));
        assert!(matches!(
            adapter.record_capability_retry_budget_exhausted(""),
            Err(MetricsError::InvalidLabelValue { .. })
        ));
    }

    #[test]
    fn all_seven_canonical_families_appear_in_registry() {
        // prometheus::Registry::gather() only surfaces metric families
        // that have at least one observed sample. For the *Vec families
        // (GaugeVec, IntCounterVec) we need to trigger a label
        // observation first; do so for every canonical family so the
        // canonical-base contract is provably wired.
        let (registry, adapter) = fresh_adapter("audit-chain");
        adapter
            .record_capability_circuit_state("probe", CircuitState::Closed)
            .unwrap();
        adapter
            .record_capability_retry_budget_exhausted("probe")
            .unwrap();
        adapter.record_responses_429("tenant-probe").unwrap();
        adapter.record_responses_5xx().unwrap();
        adapter.record_responses_total().unwrap();
        adapter.record_request_success().unwrap();
        adapter.record_request_total().unwrap();

        let gathered = registry.gather();
        let names: Vec<&str> = gathered.iter().map(|mf| mf.get_name()).collect();
        // hyphenated slug becomes underscored
        for expected in [
            "audit_chain_capability_circuit_state",
            "audit_chain_capability_retry_budget_exhausted_total",
            "audit_chain_responses_429_total",
            "audit_chain_responses_5xx_total",
            "audit_chain_responses_total",
            "audit_chain_request_success_total",
            "audit_chain_request_total",
        ] {
            assert!(
                names.contains(&expected),
                "missing canonical family: {expected}; registered: {names:?}"
            );
        }
    }

    #[test]
    fn microservice_label_bound_once() {
        let (registry, adapter) = fresh_adapter("messenger");
        adapter.record_responses_total().unwrap();
        let gathered = registry.gather();
        let metric = gathered
            .iter()
            .find(|mf| mf.get_name() == "messenger_responses_total")
            .unwrap();
        for m in metric.get_metric() {
            let label = m
                .get_label()
                .iter()
                .find(|p| p.get_name() == "microservice")
                .map(|p| p.get_value().to_string());
            assert_eq!(label.as_deref(), Some("messenger"));
        }
        // satisfy unused-import lint on Collector in test scope when only
        // collect() and not desc() is called.
        let _ = adapter.responses_total.desc();
    }

    #[test]
    fn runtime_outcome_default_emitter_records_prometheus_counters() {
        let (registry, adapter) = fresh_adapter("messenger");
        let binding = binding("messenger", "messenger.post_message");
        let outcomes = [
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

        for outcome in &outcomes {
            adapter.record_request_outcome(outcome).unwrap();
        }

        assert_eq!(counter_value(&registry, "messenger_request_total"), 3.0);
        assert_eq!(
            counter_value(&registry, "messenger_request_success_total"),
            1.0
        );
        assert_eq!(
            counter_value(&registry, "messenger_responses_total"),
            3.0
        );
        assert_eq!(
            labeled_counter_value(
                &registry,
                "messenger_responses_429_total",
                "tenant_id",
                "tenant-a"
            ),
            1.0
        );
        assert_eq!(
            counter_value(&registry, "messenger_responses_5xx_total"),
            1.0
        );
        assert_eq!(
            labeled_counter_value(
                &registry,
                "messenger_capability_retry_budget_exhausted_total",
                "capability_id",
                "broker-publish"
            ),
            1.0
        );
        assert_eq!(
            circuit_gauge_value(&registry, "broker-publish", "open"),
            1.0
        );
        assert_eq!(
            circuit_gauge_value(&registry, "broker-publish", "closed"),
            0.0
        );
    }
}
