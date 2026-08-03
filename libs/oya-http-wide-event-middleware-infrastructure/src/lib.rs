//! Wide-event tower middleware — ADR-0536 D-6 canonical telemetry primitive.
//!
//! # Decision (ADR-0536 D-6)
//!
//! One wide event per unit of work (a single high-dimensional structured
//! event) is the canonical telemetry primitive; RED metrics are DERIVED
//! from it, never collected as a parallel taxonomy; cardinality caps are
//! enforced at ingestion.
//!
//! # Composition (no parallel inventions)
//!
//! - RED counters go through the G001-adjacent canonical families in
//!   `oya-shared-hyperscaler-metrics-kernel` via
//!   [`HyperscalerMetrics::record_request_outcome`] — this crate never
//!   formats a metric name.
//! - Duration is carried ON the wide event (`latency_us`); collectors
//!   aggregate it downstream (Honeycomb/Meta wide-event practice).
//! - Route labels follow the matched-template discipline proven in
//!   `oya-http-telemetry-middleware-infrastructure` (S6 label-injection
//!   class stays closed): the label is a STATIC template from the
//!   [`RouteTemplate`] request extension, or a sentinel — never the raw
//!   path.
//!
//! # Why a request extension instead of axum's `MatchedPath`
//!
//! The workspace pins axum with `default-features = false` (no
//! `matched-path`). Reading a crate-local [`RouteTemplate`] extension
//! keeps this layer pure tower (works for axum AND tonic stacks) and
//! defers the axum feature decision to the service-integration slice.
//!
//! # Precedent
//!
//! Google SRE Workbook (SLI-aligned counters), Google Monarch
//! (cardinality limits as a survival property), Meta/Honeycomb wide-event
//! lineage, AWS Builders' Library instrumentation doctrine.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use futures_util::FutureExt as _;
use futures_util::future::BoxFuture;
use http::{Extensions, Request, Response};
use oya_http_middleware_kernel::HttpRequest as StdHttpRequest;
use oya_shared_hyperscaler_metrics_kernel::{
    HyperscalerMetrics, MetricsContext, RequestTelemetryBinding, RequestTelemetryOutcome,
};
use oya_tenancy_kernel::TenantSlug;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};

/// Route label when no [`RouteTemplate`] extension was set (mirrors the
/// bespoke-stack sentinel; keeps cardinality bounded on error flows).
pub const UNMATCHED_ROUTE_LABEL: &str = "/_unmatched";

/// `operation_id` used for requests without a matched route template.
pub const UNMATCHED_OPERATION_ID: &str = "unmatched";

/// `tenant_id` label for tenantless (platform-scoped) requests. The
/// canonical counters require a non-empty tenant label; this sentinel is
/// a single low-cardinality value, never derived from request data.
pub const PLATFORM_TENANT_LABEL: &str = "platform";

// ---------------------------------------------------------------------------
// Request extensions (inserted by earlier middleware / router glue)
// ---------------------------------------------------------------------------

/// STATIC matched route template (e.g. `/users/{user_id}`). Router glue
/// inserts this; raw paths must never be used as labels (S6).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteTemplate(pub String); // data_class: INTERNAL_ONLY

/// Tenant scope resolved by authn/tenant middleware.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantContext(pub String); // data_class: TENANT_SCOPED

/// Authenticated principal (workload or human id, never a secret).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrincipalContext(pub String); // data_class: TENANT_SCOPED

/// Trace correlation id propagated from the ingress edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext(pub String); // data_class: INTERNAL_ONLY

/// Free-form wide-event dimensions attached by earlier middleware or
/// handlers (e.g. cache outcome, retry count, feature flags). Bounded by
/// [`CardinalityCaps`] at ingestion.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RequestDimensions(pub BTreeMap<String, String>); // data_class: TENANT_SCOPED

pub const TENANT_ID_CAPTURE_KEY: &str = "tenant_id";
pub const PRINCIPAL_ID_HEADER: &str = "x-principal-id";
pub const TRACEPARENT_HEADER: &str = "traceparent";
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";
pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const DIMENSION_HEADER_PREFIX: &str = "x-oya-dim-";
pub const MAX_CORRELATION_CONTEXT_BYTES: usize = 128;
pub const MAX_PRINCIPAL_CONTEXT_BYTES: usize = 128;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StdHttpRequestExtensionBridgeReport {
    pub inserted_route_template: bool,
    pub inserted_tenant: bool,
    pub inserted_principal: bool,
    pub inserted_trace: bool,
    pub inserted_dimensions: bool,
    pub dimension_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StdHttpRequestExtensionBridgeError {
    MissingContext,
    InvalidTenantContext,
    InvalidTraceContext,
    InvalidCorrelationContext,
    InvalidPrincipalContext,
}

pub fn insert_std_http_request_extensions(
    request: &StdHttpRequest,
    extensions: &mut Extensions,
) -> Result<StdHttpRequestExtensionBridgeReport, StdHttpRequestExtensionBridgeError> {
    let mut report = StdHttpRequestExtensionBridgeReport::default();
    let route_template = request
        .matched_template
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let tenant_id = validated_tenant_id(request)?;
    let principal = validated_principal_context(request)?;
    let trace = validated_trace_context(request)?;
    let dimensions = request_dimensions_from_headers(request);

    if let Some(route_template) = route_template {
        extensions.insert(RouteTemplate(route_template.to_owned()));
        report.inserted_route_template = true;
    }

    if let Some(tenant_id) = tenant_id {
        extensions.insert(TenantContext(tenant_id));
        report.inserted_tenant = true;
    }

    if let Some(principal) = principal {
        extensions.insert(PrincipalContext(principal.to_owned()));
        report.inserted_principal = true;
    }

    if let Some(trace) = trace {
        extensions.insert(TraceContext(trace));
        report.inserted_trace = true;
    }

    if !dimensions.is_empty() {
        report.dimension_count = dimensions.len();
        report.inserted_dimensions = true;
        extensions.insert(RequestDimensions(dimensions));
    }

    Ok(report)
}

fn validated_tenant_id(
    request: &StdHttpRequest,
) -> Result<Option<String>, StdHttpRequestExtensionBridgeError> {
    let Some(tenant_id) = request
        .path_captures
        .get(TENANT_ID_CAPTURE_KEY)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    TenantSlug::try_new(tenant_id)
        .map(TenantSlug::into_inner)
        .map(Some)
        .map_err(|_| StdHttpRequestExtensionBridgeError::InvalidTenantContext)
}

fn validated_trace_context(
    request: &StdHttpRequest,
) -> Result<Option<String>, StdHttpRequestExtensionBridgeError> {
    if let Some(traceparent) = non_empty_header(request, TRACEPARENT_HEADER) {
        if is_valid_traceparent(traceparent) {
            return Ok(Some(traceparent.to_owned()));
        }
        return Err(StdHttpRequestExtensionBridgeError::InvalidTraceContext);
    }
    let Some(correlation) = non_empty_header(request, CORRELATION_ID_HEADER)
        .or_else(|| non_empty_header(request, REQUEST_ID_HEADER))
    else {
        return Ok(None);
    };
    if is_valid_bounded_correlation_context(correlation) {
        Ok(Some(correlation.to_owned()))
    } else {
        Err(StdHttpRequestExtensionBridgeError::InvalidCorrelationContext)
    }
}

fn is_valid_bounded_correlation_context(value: &str) -> bool {
    is_valid_bounded_context(value, MAX_CORRELATION_CONTEXT_BYTES)
}

fn validated_principal_context(
    request: &StdHttpRequest,
) -> Result<Option<&str>, StdHttpRequestExtensionBridgeError> {
    let Some(principal) = non_empty_header(request, PRINCIPAL_ID_HEADER) else {
        return Ok(None);
    };
    if is_valid_bounded_context(principal, MAX_PRINCIPAL_CONTEXT_BYTES) {
        Ok(Some(principal))
    } else {
        Err(StdHttpRequestExtensionBridgeError::InvalidPrincipalContext)
    }
}

fn is_valid_bounded_context(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn is_valid_traceparent(value: &str) -> bool {
    let mut parts = value.split('-');
    let (Some(version), Some(trace_id), Some(parent_id), Some(flags), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return false;
    };
    version == "00"
        && trace_id.len() == 32
        && parent_id.len() == 16
        && flags.len() == 2
        && is_lower_hex(trace_id)
        && is_lower_hex(parent_id)
        && is_lower_hex(flags)
        && !trace_id.bytes().all(|byte| byte == b'0')
        && !parent_id.bytes().all(|byte| byte == b'0')
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn request_dimensions_from_headers(request: &StdHttpRequest) -> BTreeMap<String, String> {
    let mut dimensions = BTreeMap::new();
    for (name, value) in &request.headers {
        let Some(key) = name.strip_prefix(DIMENSION_HEADER_PREFIX) else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        dimensions.insert(key.to_owned(), value.to_owned());
    }
    dimensions
}

fn non_empty_header<'a>(request: &'a StdHttpRequest, name: &str) -> Option<&'a str> {
    request
        .headers
        .get(name)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

// ---------------------------------------------------------------------------
// Cardinality caps (enforced at ingestion, D-6)
// ---------------------------------------------------------------------------

/// Ingestion-time bounds on the free-form dimension map. Truncation is
/// deterministic (BTreeMap order) and RECORDED on the event — never
/// silent (no-silent-caps doctrine).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CardinalityCaps {
    pub max_dimensions: usize,  // data_class: INTERNAL_ONLY
    pub max_key_bytes: usize,   // data_class: INTERNAL_ONLY
    pub max_value_bytes: usize, // data_class: INTERNAL_ONLY
}

impl Default for CardinalityCaps {
    fn default() -> Self {
        Self {
            max_dimensions: 32,
            max_key_bytes: 64,
            max_value_bytes: 256,
        }
    }
}

impl CardinalityCaps {
    /// Apply the caps to `dimensions`. Returns the bounded map and
    /// whether anything was truncated or dropped.
    #[must_use]
    pub fn apply(&self, dimensions: BTreeMap<String, String>) -> (BTreeMap<String, String>, bool) {
        let mut truncated = dimensions.len() > self.max_dimensions;
        let mut bounded = BTreeMap::new();
        for (key, value) in dimensions.into_iter().take(self.max_dimensions) {
            let mut key = key;
            let mut value = value;
            if key.len() > self.max_key_bytes {
                key = truncate_to_boundary(&key, self.max_key_bytes);
                truncated = true;
            }
            if value.len() > self.max_value_bytes {
                value = truncate_to_boundary(&value, self.max_value_bytes);
                truncated = true;
            }
            bounded.insert(key, value);
        }
        (bounded, truncated)
    }
}

/// Truncate at a UTF-8 character boundary at or below `max_bytes`.
fn truncate_to_boundary(s: &str, max_bytes: usize) -> String {
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}

// ---------------------------------------------------------------------------
// The wide event
// ---------------------------------------------------------------------------

/// One high-dimensional structured event per unit of work. Closed core
/// fields; the bounded `dimensions` map carries service-specific context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WideEvent {
    pub service: String, // data_class: INTERNAL_ONLY
    /// STATIC route template label (never the raw path — S6).
    pub route_template: String, // data_class: INTERNAL_ONLY
    pub method: String,  // data_class: INTERNAL_ONLY
    pub operation_id: String, // data_class: INTERNAL_ONLY
    pub status_code: u16, // data_class: INTERNAL_ONLY
    pub latency_us: u64, // data_class: INTERNAL_ONLY
    /// SLI availability outcome (5xx = failure; 4xx counts as success
    /// for the availability SLI per SRE Workbook practice).
    pub sli_success: bool, // data_class: INTERNAL_ONLY
    pub tenant_id: Option<String>, // data_class: TENANT_SCOPED
    pub principal: Option<String>, // data_class: TENANT_SCOPED
    pub trace_id: Option<String>, // data_class: INTERNAL_ONLY
    /// True when [`CardinalityCaps`] truncated or dropped dimensions.
    pub cardinality_truncated: bool, // data_class: INTERNAL_ONLY
    /// True when canonical-counter derivation failed (telemetry errors
    /// never fail the request path; they are recorded here instead).
    pub red_derivation_failed: bool, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub dimensions: BTreeMap<String, String>, // data_class: TENANT_SCOPED
}

/// Sink port for wide events. Emission is infallible by contract:
/// telemetry must never fail the request path, so implementations own
/// their error handling (buffering, drop counters, backpressure).
pub trait WideEventSink: Send + Sync {
    fn emit(&self, event: WideEvent);
}

/// Derive the canonical `operation_id` label from method + template
/// (e.g. `GET /users/{user_id}` → `get.users-user-id`). Output always
/// satisfies `is_valid_operation_id` (lowercase dotted/dashed tokens).
#[must_use]
pub fn operation_id_for(method: &str, route_template: &str) -> String {
    if route_template == UNMATCHED_ROUTE_LABEL {
        return UNMATCHED_OPERATION_ID.to_owned();
    }
    let mut slug = String::with_capacity(route_template.len());
    let mut last_dash = true; // suppress leading dashes
    for c in route_template.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            slug.push(c);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        return UNMATCHED_OPERATION_ID.to_owned();
    }
    format!("{}.{slug}", method.to_ascii_lowercase())
}

// ---------------------------------------------------------------------------
// Tower layer + service
// ---------------------------------------------------------------------------

/// Shared, immutable middleware configuration.
struct WideEventConfig {
    context: MetricsContext,
    metrics: Arc<dyn HyperscalerMetrics>,
    sink: Arc<dyn WideEventSink>,
    caps: CardinalityCaps,
}

/// Tower [`Layer`] emitting one [`WideEvent`] per request and deriving
/// the canonical RED counters from the same outcome.
#[derive(Clone)]
pub struct WideEventLayer {
    config: Arc<WideEventConfig>,
}

impl WideEventLayer {
    pub fn new(
        context: MetricsContext,
        metrics: Arc<dyn HyperscalerMetrics>,
        sink: Arc<dyn WideEventSink>,
    ) -> Self {
        Self::with_caps(context, metrics, sink, CardinalityCaps::default())
    }

    pub fn with_caps(
        context: MetricsContext,
        metrics: Arc<dyn HyperscalerMetrics>,
        sink: Arc<dyn WideEventSink>,
        caps: CardinalityCaps,
    ) -> Self {
        Self {
            config: Arc::new(WideEventConfig {
                context,
                metrics,
                sink,
                caps,
            }),
        }
    }
}

impl<S> Layer<S> for WideEventLayer {
    type Service = WideEventService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WideEventService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// The [`Service`] produced by [`WideEventLayer`].
#[derive(Clone)]
pub struct WideEventService<S> {
    inner: S,
    config: Arc<WideEventConfig>,
}

impl<S, B, RB> Service<Request<B>> for WideEventService<S>
where
    S: Service<Request<B>, Response = Response<RB>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        // Standard tower clone-swap so the boxed future owns a ready inner.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let config = self.config.clone();

        let method = request.method().as_str().to_owned();
        let route_template = request
            .extensions()
            .get::<RouteTemplate>()
            .map_or_else(|| UNMATCHED_ROUTE_LABEL.to_owned(), |r| r.0.clone());
        let tenant_id = request
            .extensions()
            .get::<TenantContext>()
            .map(|t| t.0.clone());
        let principal = request
            .extensions()
            .get::<PrincipalContext>()
            .map(|p| p.0.clone());
        let trace_id = request
            .extensions()
            .get::<TraceContext>()
            .map(|t| t.0.clone());
        let raw_dimensions = request
            .extensions()
            .get::<RequestDimensions>()
            .map(|d| d.0.clone())
            .unwrap_or_default();

        let start = Instant::now();
        async move {
            let result = inner.call(request).await;
            let latency_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            if let Ok(response) = &result {
                let status_code = response.status().as_u16();
                let sli_success = status_code < 500;
                let operation_id = operation_id_for(&method, &route_template);
                let tenant_label = tenant_id
                    .clone()
                    .unwrap_or_else(|| PLATFORM_TENANT_LABEL.to_owned());

                // Derive RED counters from this same outcome. Telemetry
                // failures never fail the request — they are recorded on
                // the wide event for the collector to alert on.
                let red_derivation_failed =
                    RequestTelemetryBinding::new(&config.context, operation_id.clone())
                        .and_then(|binding| {
                            RequestTelemetryOutcome::new(
                                binding,
                                tenant_label,
                                status_code,
                                sli_success,
                            )
                        })
                        .and_then(|outcome| config.metrics.record_request_outcome(&outcome))
                        .is_err();

                let (dimensions, cardinality_truncated) = config.caps.apply(raw_dimensions);
                config.sink.emit(WideEvent {
                    service: config.context.microservice().to_owned(),
                    route_template,
                    method,
                    operation_id,
                    status_code,
                    latency_us,
                    sli_success,
                    tenant_id,
                    principal,
                    trace_id,
                    cardinality_truncated,
                    red_derivation_failed,
                    dimensions,
                });
            }
            result
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_middleware_kernel::HttpRequest as StdHttpRequest;
    use oya_http_router_kernel::HttpMethod;
    use oya_shared_hyperscaler_metrics_kernel::{CircuitState, MetricsError};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Mutex;

    // Minimal single-threaded block_on (std-only; no tokio dependency).
    fn block_on<F: Future>(fut: F) -> F::Output {
        let waker = std::task::Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut fut = std::pin::pin!(fut);
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
            std::thread::yield_now();
        }
    }

    #[derive(Default)]
    struct RecordingMetrics {
        calls: Mutex<Vec<String>>,
        fail: bool,
    }

    impl HyperscalerMetrics for RecordingMetrics {
        fn record_capability_circuit_state(
            &self,
            _capability_id: &str,
            _state: CircuitState,
        ) -> Result<(), MetricsError> {
            Ok(())
        }
        fn record_capability_retry_budget_exhausted(
            &self,
            _capability_id: &str,
        ) -> Result<(), MetricsError> {
            Ok(())
        }
        fn record_responses_429(&self, tenant_id: &str) -> Result<(), MetricsError> {
            self.calls.lock().unwrap().push(format!("429:{tenant_id}"));
            Ok(())
        }
        fn record_responses_5xx(&self) -> Result<(), MetricsError> {
            self.calls.lock().unwrap().push("5xx".into());
            Ok(())
        }
        fn record_responses_total(&self) -> Result<(), MetricsError> {
            self.calls.lock().unwrap().push("responses_total".into());
            Ok(())
        }
        fn record_request_success(&self) -> Result<(), MetricsError> {
            if self.fail {
                return Err(MetricsError::InvalidStatusCode(0));
            }
            self.calls.lock().unwrap().push("request_success".into());
            Ok(())
        }
        fn record_request_total(&self) -> Result<(), MetricsError> {
            if self.fail {
                return Err(MetricsError::InvalidStatusCode(0));
            }
            self.calls.lock().unwrap().push("request_total".into());
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<WideEvent>>,
    }

    impl WideEventSink for RecordingSink {
        fn emit(&self, event: WideEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    #[derive(Clone)]
    struct StaticHandler {
        status: u16,
    }

    fn std_http_request(
        method: HttpMethod,
        path: &str,
        matched_template: Option<&str>,
        headers: &[(&str, &str)],
        captures: &[(&str, &str)],
    ) -> StdHttpRequest {
        let mut request = StdHttpRequest {
            method,
            path: path.to_owned(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: matched_template.map(str::to_owned),
        };
        for (key, value) in headers {
            request
                .headers
                .insert((*key).to_owned(), (*value).to_owned());
        }
        for (key, value) in captures {
            request
                .path_captures
                .insert((*key).to_owned(), (*value).to_owned());
        }
        request
    }

    impl Service<Request<()>> for StaticHandler {
        type Response = Response<String>;
        type Error = std::convert::Infallible;
        type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _request: Request<()>) -> Self::Future {
            futures_util::future::ready(Ok(Response::builder()
                .status(self.status)
                .body("ok".to_owned())
                .expect("static response")))
        }
    }

    fn run_one(
        status: u16,
        decorate: impl FnOnce(&mut Request<()>),
    ) -> (Arc<RecordingMetrics>, Arc<RecordingSink>) {
        run_one_with(status, false, decorate)
    }

    fn run_one_with(
        status: u16,
        fail_metrics: bool,
        decorate: impl FnOnce(&mut Request<()>),
    ) -> (Arc<RecordingMetrics>, Arc<RecordingSink>) {
        let metrics = Arc::new(RecordingMetrics {
            fail: fail_metrics,
            ..RecordingMetrics::default()
        });
        let sink = Arc::new(RecordingSink::default());
        let layer = WideEventLayer::new(
            MetricsContext::new("cloud-observability").expect("slug"),
            metrics.clone(),
            sink.clone(),
        );
        let mut service = layer.layer(StaticHandler { status });
        let mut request = Request::builder()
            .uri("/ignored/raw/path")
            .body(())
            .unwrap();
        decorate(&mut request);
        let response = block_on(service.call(request)).expect("infallible");
        assert_eq!(response.status().as_u16(), status);
        (metrics, sink)
    }

    #[test]
    fn std_http_request_bridge_populates_static_extensions_without_raw_leakage() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let source = std_http_request(
            HttpMethod::Post,
            "/tenants/acme/accounts/secret-account",
            Some("/tenants/{tenant_id}/accounts/{account_id}"),
            &[
                ("x-principal-id", "wl_console"),
                ("traceparent", traceparent),
            ],
            &[("tenant_id", "acme"), ("account_id", "secret-account")],
        );
        let mut tower_request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();

        let report = insert_std_http_request_extensions(&source, tower_request.extensions_mut())
            .expect("valid std request context bridges to extensions");

        assert!(report.inserted_route_template);
        assert!(report.inserted_tenant);
        assert!(report.inserted_principal);
        assert!(report.inserted_trace);
        let extensions = tower_request.extensions();
        assert_eq!(
            extensions.get::<RouteTemplate>(),
            Some(&RouteTemplate(
                "/tenants/{tenant_id}/accounts/{account_id}".to_owned()
            ))
        );
        assert_eq!(
            extensions.get::<TenantContext>(),
            Some(&TenantContext("acme".to_owned()))
        );
        assert_eq!(
            extensions.get::<PrincipalContext>(),
            Some(&PrincipalContext("wl_console".to_owned()))
        );
        assert_eq!(
            extensions.get::<TraceContext>(),
            Some(&TraceContext(traceparent.to_owned()))
        );
        let route = &extensions.get::<RouteTemplate>().unwrap().0;
        assert!(!route.contains("acme"));
        assert!(!route.contains("secret-account"));
    }

    #[test]
    fn bridged_wide_event_records_bounded_dimensions_for_final_response() {
        let traceparent = "00-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bbbbbbbbbbbbbbbb-01";
        let source = std_http_request(
            HttpMethod::Get,
            "/widgets/widget-42",
            Some("/widgets/{widget_id}"),
            &[
                ("traceparent", traceparent),
                ("x-oya-dim-alpha", "123456"),
                ("x-oya-dim-beta", "ok"),
                ("x-oya-dim-gamma", "dropped"),
            ],
            &[("tenant_id", "tenant_alpha"), ("widget_id", "widget-42")],
        );
        let metrics = Arc::new(RecordingMetrics::default());
        let sink = Arc::new(RecordingSink::default());
        let layer = WideEventLayer::with_caps(
            MetricsContext::new("cloud-observability").expect("slug"),
            metrics.clone(),
            sink.clone(),
            CardinalityCaps {
                max_dimensions: 2,
                max_key_bytes: 4,
                max_value_bytes: 4,
            },
        );
        let mut service = layer.layer(StaticHandler { status: 503 });
        let mut request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();
        insert_std_http_request_extensions(&source, request.extensions_mut())
            .expect("valid bridge context");

        let response = block_on(service.call(request)).expect("infallible");

        assert_eq!(response.status().as_u16(), 503);
        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.route_template, "/widgets/{widget_id}");
        assert!(!event.route_template.contains("widget-42"));
        assert_eq!(event.tenant_id.as_deref(), Some("tenant_alpha"));
        assert_eq!(event.trace_id.as_deref(), Some(traceparent));
        assert_eq!(event.status_code, 503);
        assert!(!event.sli_success);
        assert!(event.cardinality_truncated);
        assert_eq!(event.dimensions.len(), 2);
        assert_eq!(
            event.dimensions.get("alph").map(String::as_str),
            Some("1234")
        );
        assert_eq!(event.dimensions.get("beta").map(String::as_str), Some("ok"));
        let calls = metrics.calls.lock().unwrap();
        assert!(calls.contains(&"5xx".to_owned()));
        assert!(!calls.contains(&"request_success".to_owned()));
    }

    #[test]
    fn bridge_rejects_malformed_tenant_before_extension_insertion() {
        let source = std_http_request(
            HttpMethod::Get,
            "/widgets/widget-42",
            Some("/widgets/{widget_id}"),
            &[],
            &[("tenant_id", "bad/tenant"), ("widget_id", "widget-42")],
        );
        let mut request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();

        insert_std_http_request_extensions(&source, request.extensions_mut())
            .expect_err("malformed tenant capture must be rejected before bridge insertion");

        assert!(request.extensions().get::<TenantContext>().is_none());
        assert!(request.extensions().get::<RouteTemplate>().is_none());
    }

    #[test]
    fn bridge_rejects_malformed_traceparent_before_extension_insertion() {
        let source = std_http_request(
            HttpMethod::Get,
            "/widgets/widget-42",
            Some("/widgets/{widget_id}"),
            &[("traceparent", "00-not-a-valid-traceparent")],
            &[("tenant_id", "tenant_alpha"), ("widget_id", "widget-42")],
        );
        let mut request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();

        insert_std_http_request_extensions(&source, request.extensions_mut())
            .expect_err("malformed traceparent must be rejected before bridge insertion");

        assert!(request.extensions().get::<TraceContext>().is_none());
        assert!(request.extensions().get::<RouteTemplate>().is_none());
    }

    #[test]
    fn bridge_rejects_unbounded_correlation_before_extension_insertion() {
        let too_long_correlation = "c".repeat(129);
        let source = std_http_request(
            HttpMethod::Get,
            "/widgets/widget-42",
            Some("/widgets/{widget_id}"),
            &[("x-correlation-id", too_long_correlation.as_str())],
            &[("tenant_id", "tenant_alpha"), ("widget_id", "widget-42")],
        );
        let mut request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();

        insert_std_http_request_extensions(&source, request.extensions_mut())
            .expect_err("oversized correlation id must be rejected before bridge insertion");

        assert!(request.extensions().get::<TraceContext>().is_none());
        assert!(request.extensions().get::<RouteTemplate>().is_none());
    }

    #[test]
    fn bridge_rejects_unbounded_principal_before_extension_insertion() {
        let too_long_principal = "p".repeat(129);
        let source = std_http_request(
            HttpMethod::Get,
            "/widgets/widget-42",
            Some("/widgets/{widget_id}"),
            &[("x-principal-id", too_long_principal.as_str())],
            &[("tenant_id", "tenant_alpha"), ("widget_id", "widget-42")],
        );
        let mut request = Request::builder()
            .uri(source.path.as_str())
            .body(())
            .unwrap();

        insert_std_http_request_extensions(&source, request.extensions_mut())
            .expect_err("oversized principal id must be rejected before bridge insertion");

        assert!(request.extensions().get::<PrincipalContext>().is_none());
        assert!(request.extensions().get::<RouteTemplate>().is_none());
    }

    #[test]
    fn emits_one_wide_event_with_derived_red_counters() {
        let (metrics, sink) = run_one(200, |req| {
            req.extensions_mut()
                .insert(RouteTemplate("/users/{user_id}".into()));
            req.extensions_mut()
                .insert(TenantContext("ten_acme".into()));
            req.extensions_mut()
                .insert(PrincipalContext("wl_console".into()));
            req.extensions_mut().insert(TraceContext("tr-1".into()));
        });
        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 1, "exactly one wide event per unit of work");
        let e = &events[0];
        assert_eq!(e.service, "cloud-observability");
        assert_eq!(e.route_template, "/users/{user_id}");
        assert_eq!(e.operation_id, "get.users-user-id");
        assert_eq!(e.tenant_id.as_deref(), Some("ten_acme"));
        assert_eq!(e.principal.as_deref(), Some("wl_console"));
        assert_eq!(e.trace_id.as_deref(), Some("tr-1"));
        assert!(e.sli_success);
        assert!(!e.red_derivation_failed);
        // RED derivation went through the canonical fan-out.
        let calls = metrics.calls.lock().unwrap();
        assert_eq!(
            *calls,
            vec!["request_total", "responses_total", "request_success"]
        );
    }

    #[test]
    fn raw_path_never_becomes_a_label() {
        let (_, sink) = run_one(200, |_| {});
        let events = sink.events.lock().unwrap();
        assert_eq!(events[0].route_template, UNMATCHED_ROUTE_LABEL);
        assert_eq!(events[0].operation_id, UNMATCHED_OPERATION_ID);
        assert!(!events[0].route_template.contains("ignored"));
    }

    #[test]
    fn five_xx_is_sli_failure_and_increments_5xx() {
        let (metrics, sink) = run_one(503, |req| {
            req.extensions_mut().insert(RouteTemplate("/a".into()));
        });
        let events = sink.events.lock().unwrap();
        assert!(!events[0].sli_success);
        let calls = metrics.calls.lock().unwrap();
        assert!(calls.contains(&"5xx".to_string()));
        assert!(!calls.contains(&"request_success".to_string()));
    }

    #[test]
    fn four_29_counts_against_platform_tenant_when_tenantless() {
        let (metrics, sink) = run_one(429, |req| {
            req.extensions_mut().insert(RouteTemplate("/a".into()));
        });
        let events = sink.events.lock().unwrap();
        // 4xx is availability success but the 429 counter still fires.
        assert!(events[0].sli_success);
        assert_eq!(events[0].tenant_id, None);
        let calls = metrics.calls.lock().unwrap();
        assert!(calls.contains(&format!("429:{PLATFORM_TENANT_LABEL}")));
    }

    #[test]
    fn metrics_failure_never_fails_request_but_is_recorded() {
        let (_, sink) = run_one_with(200, true, |req| {
            req.extensions_mut().insert(RouteTemplate("/a".into()));
        });
        let events = sink.events.lock().unwrap();
        assert!(events[0].red_derivation_failed);
    }

    #[test]
    fn operation_ids_are_canonical() {
        use oya_shared_hyperscaler_metrics_kernel::is_valid_operation_id;
        for (method, template, expected) in [
            ("GET", "/users/{user_id}", "get.users-user-id"),
            ("POST", "/tenants", "post.tenants"),
            ("DELETE", "/a/{id}/b", "delete.a-id-b"),
            ("GET", UNMATCHED_ROUTE_LABEL, UNMATCHED_OPERATION_ID),
        ] {
            let op = operation_id_for(method, template);
            assert_eq!(op, expected);
            assert!(
                is_valid_operation_id(&op),
                "{op:?} must satisfy the kernel operation-id contract"
            );
        }
    }

    #[test]
    fn cardinality_caps_truncate_deterministically_and_loudly() {
        let caps = CardinalityCaps {
            max_dimensions: 2,
            max_key_bytes: 4,
            max_value_bytes: 4,
        };
        let mut dims = BTreeMap::new();
        dims.insert("alpha".to_owned(), "123456".to_owned());
        dims.insert("b".to_owned(), "ok".to_owned());
        dims.insert("c".to_owned(), "dropped".to_owned());
        let (bounded, truncated) = caps.apply(dims);
        assert!(truncated, "truncation must be recorded, never silent");
        assert_eq!(bounded.len(), 2);
        assert_eq!(bounded.get("alph").map(String::as_str), Some("1234"));
        assert_eq!(bounded.get("b").map(String::as_str), Some("ok"));
    }

    #[test]
    fn utf8_truncation_respects_char_boundaries() {
        let caps = CardinalityCaps {
            max_dimensions: 8,
            max_key_bytes: 64,
            max_value_bytes: 5,
        };
        let mut dims = BTreeMap::new();
        dims.insert("k".to_owned(), "héllo".to_owned()); // é is 2 bytes
        let (bounded, truncated) = caps.apply(dims);
        assert!(truncated);
        let v = bounded.get("k").unwrap();
        assert!(v.len() <= 5);
        assert!(std::str::from_utf8(v.as_bytes()).is_ok());
    }

    #[test]
    fn wide_event_serde_round_trips() {
        let (_, sink) = run_one(200, |req| {
            req.extensions_mut().insert(RouteTemplate("/a".into()));
        });
        let events = sink.events.lock().unwrap();
        let json = serde_json::to_string(&events[0]).unwrap();
        let back: WideEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, events[0]);
    }
}
