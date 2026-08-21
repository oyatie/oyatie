//! OpenTelemetry OTLP metrics adapter for the shared `HyperscalerMetrics` port.
//!
//! The default local metrics adapter remains Prometheus. This crate adds an
//! explicit, environment-gated OTLP/HTTP exporter harness for cloud or
//! self-hosted OpenTelemetry Collector integration. It does **not** export by
//! default: callers must set [`OTLP_METRICS_ENABLE_ENV`] and provide a collector
//! endpoint through [`OTLP_METRICS_ENDPOINT_ENV`] or
//! [`OTLP_GENERIC_ENDPOINT_ENV`].
//!
//! # Source basis
//!
//! - <https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/> documents
//!   `MetricExporter::builder()`, HTTP/protobuf transport, endpoint/timeout
//!   configuration, and `SdkMeterProvider::with_periodic_exporter`.
//! - <https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Meter.html>
//!   documents synchronous counters and gauges (`add`, `record`).
//! - <https://opentelemetry.io/docs/specs/otel/metrics/api/> documents the
//!   API distinction between counters and gauges.
//!
//! # Non-claims
//!
//! This crate compiles and unit-tests the adapter/configuration seam. Unless an
//! operator explicitly enables the env-gated harness with a reachable collector,
//! no live OTLP export, collector deployment, alert firing, or production SLO is
//! claimed.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, MeterProvider as _};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use shared_hyperscaler_metrics_kernel::{
    CircuitState, HyperscalerMetrics, MetricFamily, MetricsContext, MetricsError, metric_name,
};

/// Explicit enable switch. Export is disabled unless this env var parses true.
pub const OTLP_METRICS_ENABLE_ENV: &str = "OYA_BACKBONE_OTLP_METRICS";
/// Signal-specific collector endpoint. Takes precedence over the generic OTLP endpoint.
pub const OTLP_METRICS_ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Generic collector endpoint used when the metrics-specific endpoint is absent.
pub const OTLP_GENERIC_ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Optional posture guard: when true, enabled endpoints must use `https://`.
pub const OTLP_REQUIRE_TLS_ENV: &str = "OYA_BACKBONE_OTLP_REQUIRE_TLS";
/// Export timeout in milliseconds. Defaults to the OTLP crate's documented 10s timeout.
pub const OTLP_TIMEOUT_MS_ENV: &str = "OYA_BACKBONE_OTLP_TIMEOUT_MS";

const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const MIN_TIMEOUT_MS: u64 = 100;
const MAX_TIMEOUT_MS: u64 = 60_000;

/// Environment-gated OTLP exporter configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OtlpMetricsHarnessConfig {
    pub enabled: bool,            // data_class: INTERNAL_ONLY
    pub microservice: String,     // data_class: INTERNAL_ONLY
    pub endpoint: Option<String>, // data_class: INTERNAL_ONLY
    pub require_tls: bool,        // data_class: INTERNAL_ONLY
    pub timeout_ms: u64,          // data_class: INTERNAL_ONLY
    pub protocol: String,         // data_class: INTERNAL_ONLY
}

impl OtlpMetricsHarnessConfig {
    /// Read env vars for one validated microservice slug.
    pub fn from_env(microservice: impl Into<String>) -> Result<Self, OtlpMetricsInitError> {
        Self::from_env_map(microservice, |key| std::env::var(key).ok())
    }

    /// Build from a supplied key/value map. Tests use this to avoid mutating the
    /// process environment.
    pub fn from_env_map<F>(
        microservice: impl Into<String>,
        mut get: F,
    ) -> Result<Self, OtlpMetricsInitError>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let microservice = microservice.into();
        MetricsContext::new(microservice.clone()).map_err(OtlpMetricsInitError::Metrics)?;

        let enabled =
            parse_bool_env(OTLP_METRICS_ENABLE_ENV, get(OTLP_METRICS_ENABLE_ENV))?.unwrap_or(false);
        let require_tls =
            parse_bool_env(OTLP_REQUIRE_TLS_ENV, get(OTLP_REQUIRE_TLS_ENV))?.unwrap_or(false);
        let timeout_ms = parse_timeout_ms(get(OTLP_TIMEOUT_MS_ENV))?;
        let endpoint = first_present_endpoint(&mut get)?;

        if enabled && endpoint.is_none() {
            return Err(OtlpMetricsInitError::MissingEndpoint);
        }
        if let Some(endpoint) = endpoint.as_deref() {
            validate_endpoint(endpoint, require_tls)?;
        }

        Ok(Self {
            enabled,
            microservice,
            endpoint,
            require_tls,
            timeout_ms,
            protocol: "http/protobuf".to_string(),
        })
    }

    /// Low-cardinality summary safe for evidence logs. It never includes auth
    /// headers and carries only endpoint scheme/host path as supplied by config.
    #[must_use]
    pub fn connection_summary(&self) -> OtlpMetricsConnectionSummary {
        OtlpMetricsConnectionSummary {
            enabled: self.enabled,
            microservice: self.microservice.clone(),
            endpoint_configured: self.endpoint.is_some(),
            require_tls: self.require_tls,
            timeout_ms: self.timeout_ms,
            protocol: self.protocol.clone(),
        }
    }
}

/// Evidence-safe connection summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OtlpMetricsConnectionSummary {
    pub enabled: bool,             // data_class: INTERNAL_ONLY
    pub microservice: String,      // data_class: INTERNAL_ONLY
    pub endpoint_configured: bool, // data_class: INTERNAL_ONLY
    pub require_tls: bool,         // data_class: INTERNAL_ONLY
    pub timeout_ms: u64,           // data_class: INTERNAL_ONLY
    pub protocol: String,          // data_class: INTERNAL_ONLY
}

/// Initialization failures for the env-gated OTLP harness.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OtlpMetricsInitError {
    InvalidBoolean { env: String, value: String },
    InvalidTimeoutMs { value: String },
    MissingEndpoint,
    InvalidEndpoint { reason: String, value: String },
    Metrics(MetricsError),
    ExporterBuildFailure(String),
}

impl fmt::Display for OtlpMetricsInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBoolean { env, value } => {
                write!(f, "invalid boolean env {env}={value:?}")
            }
            Self::InvalidTimeoutMs { value } => {
                write!(f, "invalid OTLP timeout milliseconds: {value:?}")
            }
            Self::MissingEndpoint => write!(f, "OTLP metrics enabled but no endpoint configured"),
            Self::InvalidEndpoint { reason, value } => {
                write!(f, "invalid OTLP endpoint {value:?}: {reason}")
            }
            Self::Metrics(error) => write!(f, "metrics validation failed: {error}"),
            Self::ExporterBuildFailure(error) => {
                write!(f, "OTLP metric exporter build failed: {error}")
            }
        }
    }
}

impl Error for OtlpMetricsInitError {}

/// OTLP implementation of the shared `HyperscalerMetrics` trait.
pub struct OtlpHyperscalerMetrics {
    ctx: MetricsContext,
    provider: SdkMeterProvider,
    capability_circuit_state: Gauge<u64>,
    capability_retry_budget_exhausted: Counter<u64>,
    responses_429: Counter<u64>,
    responses_5xx: Counter<u64>,
    responses_total: Counter<u64>,
    request_success_total: Counter<u64>,
    request_total: Counter<u64>,
}

impl OtlpHyperscalerMetrics {
    /// Construct instruments against an existing SDK provider. Tests and custom
    /// composition roots can use this without creating an exporter.
    pub fn from_provider(provider: SdkMeterProvider, ctx: MetricsContext) -> Self {
        let meter = provider.meter("oyatie.hyperscaler.metrics");
        Self {
            capability_circuit_state: meter
                .u64_gauge(metric_name(&ctx, MetricFamily::CapabilityCircuitState))
                .with_description(
                    "Capability circuit-breaker state; 1=open/active state, 0=inactive state.",
                )
                .build(),
            capability_retry_budget_exhausted: meter
                .u64_counter(metric_name(
                    &ctx,
                    MetricFamily::CapabilityRetryBudgetExhaustedTotal,
                ))
                .with_description("Capability retry-budget exhausted events.")
                .build(),
            responses_429: meter
                .u64_counter(metric_name(&ctx, MetricFamily::Responses429Total))
                .with_description("429 responses per tenant.")
                .build(),
            responses_5xx: meter
                .u64_counter(metric_name(&ctx, MetricFamily::Responses5xxTotal))
                .with_description("5xx responses total.")
                .build(),
            responses_total: meter
                .u64_counter(metric_name(&ctx, MetricFamily::ResponsesTotal))
                .with_description("Total responses.")
                .build(),
            request_success_total: meter
                .u64_counter(metric_name(&ctx, MetricFamily::RequestSuccessTotal))
                .with_description("SLI success numerator.")
                .build(),
            request_total: meter
                .u64_counter(metric_name(&ctx, MetricFamily::RequestTotal))
                .with_description("SLI request denominator.")
                .build(),
            ctx,
            provider,
        }
    }

    /// Bound microservice slug.
    #[must_use]
    pub fn microservice(&self) -> &str {
        self.ctx.microservice()
    }

    /// Flush the underlying SDK provider when a composition root wants a
    /// deterministic shutdown/export barrier.
    pub fn force_flush(&self) -> Result<(), OtlpMetricsInitError> {
        self.provider
            .force_flush()
            .map_err(|error| OtlpMetricsInitError::ExporterBuildFailure(error.to_string()))
    }
}

impl HyperscalerMetrics for OtlpHyperscalerMetrics {
    fn record_capability_circuit_state(
        &self,
        capability_id: &str,
        state: CircuitState,
    ) -> Result<(), MetricsError> {
        validate_label("capability_id", capability_id)?;
        for candidate in [
            CircuitState::Closed,
            CircuitState::HalfOpen,
            CircuitState::Open,
        ] {
            let value = if candidate == state { 1 } else { 0 };
            self.capability_circuit_state.record(
                value,
                &[
                    self.microservice_attr(),
                    KeyValue::new("capability_id", capability_id.to_string()),
                    KeyValue::new("state", candidate.as_label()),
                ],
            );
        }
        Ok(())
    }

    fn record_capability_retry_budget_exhausted(
        &self,
        capability_id: &str,
    ) -> Result<(), MetricsError> {
        validate_label("capability_id", capability_id)?;
        self.capability_retry_budget_exhausted.add(
            1,
            &[
                self.microservice_attr(),
                KeyValue::new("capability_id", capability_id.to_string()),
            ],
        );
        Ok(())
    }

    fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError> {
        validate_label("tenant_id", tenant_id)?;
        self.responses_429.add(
            1,
            &[
                self.microservice_attr(),
                KeyValue::new("tenant_id", tenant_id.to_string()),
            ],
        );
        Ok(())
    }

    fn record_responses_5xx(&self) -> Result<(), MetricsError> {
        self.responses_5xx.add(1, &[self.microservice_attr()]);
        Ok(())
    }

    fn record_responses_total(&self) -> Result<(), MetricsError> {
        self.responses_total.add(1, &[self.microservice_attr()]);
        Ok(())
    }

    fn record_request_success(&self) -> Result<(), MetricsError> {
        self.request_success_total
            .add(1, &[self.microservice_attr()]);
        Ok(())
    }

    fn record_request_total(&self) -> Result<(), MetricsError> {
        self.request_total.add(1, &[self.microservice_attr()]);
        Ok(())
    }
}

impl OtlpHyperscalerMetrics {
    fn microservice_attr(&self) -> KeyValue {
        KeyValue::new("microservice", self.ctx.microservice().to_string())
    }
}

/// Build an OTLP metrics adapter only when the config is explicitly enabled.
pub fn build_otlp_hyperscaler_metrics(
    config: &OtlpMetricsHarnessConfig,
) -> Result<Option<OtlpHyperscalerMetrics>, OtlpMetricsInitError> {
    if !config.enabled {
        return Ok(None);
    }
    let endpoint = config
        .endpoint
        .as_deref()
        .ok_or(OtlpMetricsInitError::MissingEndpoint)?;
    validate_endpoint(endpoint, config.require_tls)?;
    let ctx =
        MetricsContext::new(config.microservice.clone()).map_err(OtlpMetricsInitError::Metrics)?;
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_millis(config.timeout_ms))
        .build()
        .map_err(|error| OtlpMetricsInitError::ExporterBuildFailure(error.to_string()))?;
    let provider = SdkMeterProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(config.microservice.clone())
                .build(),
        )
        .with_periodic_exporter(exporter)
        .build();
    Ok(Some(OtlpHyperscalerMetrics::from_provider(provider, ctx)))
}

fn parse_bool_env(env: &str, value: Option<String>) -> Result<Option<bool>, OtlpMetricsInitError> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(OtlpMetricsInitError::InvalidBoolean {
                env: env.to_string(),
                value: value.to_string(),
            }),
        })
        .transpose()
}

fn parse_timeout_ms(value: Option<String>) -> Result<u64, OtlpMetricsInitError> {
    let Some(raw) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(DEFAULT_TIMEOUT_MS);
    };
    let parsed = raw
        .parse::<u64>()
        .map_err(|_| OtlpMetricsInitError::InvalidTimeoutMs { value: raw.clone() })?;
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&parsed) {
        return Err(OtlpMetricsInitError::InvalidTimeoutMs { value: raw });
    }
    Ok(parsed)
}

fn first_present_endpoint<F>(get: &mut F) -> Result<Option<String>, OtlpMetricsInitError>
where
    F: FnMut(&str) -> Option<String>,
{
    let endpoint = get(OTLP_METRICS_ENDPOINT_ENV).or_else(|| get(OTLP_GENERIC_ENDPOINT_ENV));
    endpoint
        .map(|endpoint| endpoint.trim().to_string())
        .filter(|endpoint| !endpoint.is_empty())
        .map(|endpoint| {
            validate_endpoint(&endpoint, false)?;
            Ok(endpoint)
        })
        .transpose()
}

fn validate_endpoint(endpoint: &str, require_tls: bool) -> Result<(), OtlpMetricsInitError> {
    if endpoint
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(OtlpMetricsInitError::InvalidEndpoint {
            reason: "endpoint contains whitespace or control characters".to_string(),
            value: endpoint.to_string(),
        });
    }
    if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
        return Err(OtlpMetricsInitError::InvalidEndpoint {
            reason: "endpoint must start with http:// or https://".to_string(),
            value: endpoint.to_string(),
        });
    }
    if require_tls && !endpoint.starts_with("https://") {
        return Err(OtlpMetricsInitError::InvalidEndpoint {
            reason: "TLS-required mode requires https://".to_string(),
            value: endpoint.to_string(),
        });
    }
    Ok(())
}

fn validate_label(label: &str, value: &str) -> Result<(), MetricsError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        Err(MetricsError::InvalidLabelValue {
            label: label.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_hyperscaler_metrics_kernel::{
        BackpressureObservation, RequestTelemetryBinding, RequestTelemetryOutcome,
    };

    fn map(values: &[(&str, &str)]) -> HashMap<String, String> {
        values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn config_is_disabled_without_enable_switch() {
        let values = map(&[(OTLP_GENERIC_ENDPOINT_ENV, "http://collector:4318")]);
        let config =
            OtlpMetricsHarnessConfig::from_env_map("messenger", |key| values.get(key).cloned())
                .unwrap();
        assert!(!config.enabled);
        assert_eq!(config.endpoint.as_deref(), Some("http://collector:4318"));
        assert_eq!(config.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn enabled_config_requires_endpoint_and_validates_tls() {
        let values = map(&[(OTLP_METRICS_ENABLE_ENV, "true")]);
        assert!(matches!(
            OtlpMetricsHarnessConfig::from_env_map("messenger", |key| values.get(key).cloned()),
            Err(OtlpMetricsInitError::MissingEndpoint)
        ));

        let values = map(&[
            (OTLP_METRICS_ENABLE_ENV, "true"),
            (OTLP_REQUIRE_TLS_ENV, "true"),
            (OTLP_GENERIC_ENDPOINT_ENV, "http://collector:4318"),
        ]);
        assert!(matches!(
            OtlpMetricsHarnessConfig::from_env_map("messenger", |key| values.get(key).cloned()),
            Err(OtlpMetricsInitError::InvalidEndpoint { .. })
        ));
    }

    #[test]
    fn metrics_endpoint_takes_precedence_over_generic_endpoint() {
        let values = map(&[
            (OTLP_METRICS_ENABLE_ENV, "on"),
            (OTLP_GENERIC_ENDPOINT_ENV, "http://generic:4318"),
            (OTLP_METRICS_ENDPOINT_ENV, "https://metrics:4318/v1/metrics"),
            (OTLP_TIMEOUT_MS_ENV, "2500"),
        ]);
        let config =
            OtlpMetricsHarnessConfig::from_env_map("mail", |key| values.get(key).cloned()).unwrap();
        assert!(config.enabled);
        assert_eq!(
            config.endpoint.as_deref(),
            Some("https://metrics:4318/v1/metrics")
        );
        assert_eq!(config.timeout_ms, 2500);
        assert_eq!(config.connection_summary().protocol, "http/protobuf");
    }

    #[test]
    fn invalid_boolean_and_timeout_values_are_rejected() {
        let values = map(&[(OTLP_METRICS_ENABLE_ENV, "sometimes")]);
        assert!(matches!(
            OtlpMetricsHarnessConfig::from_env_map("social", |key| values.get(key).cloned()),
            Err(OtlpMetricsInitError::InvalidBoolean { .. })
        ));

        let values = map(&[(OTLP_TIMEOUT_MS_ENV, "99")]);
        assert!(matches!(
            OtlpMetricsHarnessConfig::from_env_map("social", |key| values.get(key).cloned()),
            Err(OtlpMetricsInitError::InvalidTimeoutMs { .. })
        ));
    }

    #[test]
    fn adapter_records_canonical_runtime_outcome_without_collector() {
        let ctx = MetricsContext::new("messenger").unwrap();
        let provider = SdkMeterProvider::builder().build();
        let adapter = OtlpHyperscalerMetrics::from_provider(provider, ctx.clone());
        let binding = RequestTelemetryBinding::new(&ctx, "messenger.post_message").unwrap();
        let outcome = RequestTelemetryOutcome::with_backpressure(
            binding,
            "tenant-a",
            429,
            false,
            Some(
                BackpressureObservation::new("messenger.message-stream", CircuitState::Open, true)
                    .unwrap(),
            ),
        )
        .unwrap();
        adapter.record_request_outcome(&outcome).unwrap();
        assert_eq!(adapter.microservice(), "messenger");
    }

    #[test]
    fn build_returns_none_when_env_gate_is_disabled() {
        let config = OtlpMetricsHarnessConfig {
            enabled: false,
            microservice: "community".to_string(),
            endpoint: Some("http://collector:4318".to_string()),
            require_tls: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            protocol: "http/protobuf".to_string(),
        };
        assert!(build_otlp_hyperscaler_metrics(&config).unwrap().is_none());
    }
}
