//! Telemetry middleware — Layer 4.
//!
//! Records per-route counters + total latency for every dispatched request and
//! emits one structured wide event per unit of work. Pure std-only (Mutex +
//! BTreeMap); a production cell wires this output to a Prometheus-compatible
//! exporter / structured log sink via the OTel adapter (separate slice).
//!
//! Counter labels (low-cardinality only per OTel conventions):
//!   - route  (the matched_template from the router — STATIC, NEVER the raw
//!     path; this closes the S6 metric-label-injection class)
//!   - method (GET / POST / ...)
//!   - status_class (2xx / 3xx / 4xx / 5xx)
//!
//! Phase 4 fix (per ADR-0092 + F-MULTI-Q1 + S6 security):
//! Router now sets `HttpRequest::matched_template` to the registered template
//! string (e.g. `/users/{user_id}`). Telemetry reads it directly. The previous
//! heuristic that ran `path.replace(captured_value, "{name}")` is GONE — it
//! both produced wrong labels when a captured value happened to occur as a
//! literal segment elsewhere (Q1 quality bug) AND leaked sensitive captured
//! values into metric labels (S6 security class).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TelemetrySample {
    pub method: String,
    pub route: String,
    pub status_class: String,
    pub count: u64,
    pub total_latency_us: u64,
}

/// Structured per-request wide event emitted by [`TelemetryMiddleware`].
///
/// RED fields are explicit so downstream adapters can project them into
/// counters/histograms without re-inferring semantics from status codes:
/// `red_rate_count` is always 1 for the completed unit of work,
/// `red_error_count` is 1 for server-error responses and 0 otherwise, and
/// `red_duration_us` is the measured request duration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WideEvent {
    pub schema_version: u16,    // data_class: PUBLIC
    pub event_name: String,     // data_class: PUBLIC
    pub tenant_id: String,      // data_class: INTERNAL_ONLY
    pub correlation_id: String, // data_class: INTERNAL_ONLY
    pub method: String,         // data_class: PUBLIC
    pub route: String,          // data_class: PUBLIC (static route template only)
    pub status_code: u16,       // data_class: PUBLIC
    pub status_class: String,   // data_class: PUBLIC
    pub red_rate_count: u64,    // data_class: INTERNAL_ONLY
    pub red_error_count: u64,   // data_class: INTERNAL_ONLY
    pub red_duration_us: u64,   // data_class: INTERNAL_ONLY
}

/// In-memory metrics sink. Production swaps for an OTel adapter.
///
/// ⚠️ Hot-lock note: this Mutex contends on every request. For
/// hyperscaler-tier throughput (per F2 critique) replace with sharded
/// AtomicU64 counters. Tracked as FixupTask F-MULTI-Q2.
#[derive(Debug, Default)]
pub struct InMemoryMetrics {
    by_key: Mutex<BTreeMap<(String, String, String), TelemetrySample>>,
}

impl InMemoryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, method: &str, route: &str, status: u16, latency_us: u64) {
        let class = status_class(status).to_string();
        let key = (method.to_string(), route.to_string(), class.clone());
        // ADR-0083 Tier 1: recover from a poisoned Mutex by taking the inner
        // guard from the `PoisonError` rather than `.expect()`-panicking. The
        // metrics map is monotonically growing aggregates; a writer that
        // panicked previously cannot have left a partially-mutated entry,
        // since insertions are atomic with respect to the Mutex.
        let mut by_key = self
            .by_key
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = by_key.entry(key).or_insert_with(|| TelemetrySample {
            method: method.to_string(),
            route: route.to_string(),
            status_class: class,
            count: 0,
            total_latency_us: 0,
        });
        entry.count += 1;
        entry.total_latency_us = entry.total_latency_us.saturating_add(latency_us);
    }

    pub fn snapshot(&self) -> Vec<TelemetrySample> {
        self.by_key
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect()
    }

    pub fn key_count(&self) -> usize {
        self.by_key
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

/// In-memory wide-event sink. Production swaps for a structured JSON / OTel
/// logs adapter, but this slice gives deterministic request-level evidence.
#[derive(Debug, Default)]
pub struct InMemoryWideEvents {
    events: Mutex<Vec<WideEvent>>,
}

impl InMemoryWideEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, event: WideEvent) {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event);
    }

    pub fn snapshot(&self) -> Vec<WideEvent> {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn count(&self) -> usize {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

fn status_class(status: u16) -> &'static str {
    match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    }
}

/// Fallback label when the dispatch did not set `matched_template` (e.g.,
/// direct-handler invocations in unit tests). Keeps cardinality bounded by
/// using a sentinel rather than the raw path.
pub const UNMATCHED_ROUTE_LABEL: &str = "/_unmatched";
pub const WIDE_EVENT_SCHEMA_VERSION: u16 = 1;
pub const HTTP_REQUEST_COMPLETED_EVENT: &str = "http.request.completed";
pub const TENANT_ID_CAPTURE_KEY: &str = "tenant_id";
pub const TENANT_ID_HEADER: &str = "x-tenant-id";
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";
pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const TRACEPARENT_HEADER: &str = "traceparent";
pub const UNKNOWN_TENANT_ID: &str = "_unknown_tenant";
pub const MISSING_CORRELATION_ID: &str = "_missing_correlation_id";

const TENANT_SLUG_MAX_LEN: usize = 128;
const CORRELATION_ID_MAX_LEN: usize = 128;
const TRACEPARENT_W3C_LEN: usize = 55;

#[derive(Clone, Debug)]
pub struct TelemetryMiddleware {
    metrics: Arc<InMemoryMetrics>,
    wide_events: Arc<InMemoryWideEvents>,
}

impl TelemetryMiddleware {
    pub fn new(metrics: Arc<InMemoryMetrics>) -> Self {
        Self {
            metrics,
            wide_events: Arc::new(InMemoryWideEvents::new()),
        }
    }

    pub fn with_wide_events(
        metrics: Arc<InMemoryMetrics>,
        wide_events: Arc<InMemoryWideEvents>,
    ) -> Self {
        Self {
            metrics,
            wide_events,
        }
    }

    pub fn metrics(&self) -> Arc<InMemoryMetrics> {
        self.metrics.clone()
    }

    pub fn wide_events(&self) -> Arc<InMemoryWideEvents> {
        self.wide_events.clone()
    }
}

fn route_label(request: &HttpRequest) -> String {
    request
        .matched_template
        .clone()
        .unwrap_or_else(|| UNMATCHED_ROUTE_LABEL.to_string())
}

fn valid_tenant_slug(value: &str) -> bool {
    // Keep this crate std-only and avoid Cargo.lock/manifest churn: this mirrors
    // tenancy_kernel::TenantSlug (1..=128 ASCII alphanumeric, dash, underscore)
    // so telemetry refuses forged path/header tenants even if TenantMiddleware is bypassed.
    !value.is_empty()
        && value.len() <= TENANT_SLUG_MAX_LEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn valid_correlation_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= CORRELATION_ID_MAX_LEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

fn header_if_valid(
    request: &HttpRequest,
    name: &str,
    validator: impl Fn(&str) -> bool,
) -> Option<String> {
    request
        .headers
        .get(name)
        .filter(|value| validator(value.as_str()))
        .cloned()
}

fn tenant_slug_value(value: &str) -> Option<String> {
    valid_tenant_slug(value).then(|| value.to_string())
}

fn tenant_id(request: &HttpRequest) -> String {
    request
        .path_captures
        .get(TENANT_ID_CAPTURE_KEY)
        .and_then(|value| tenant_slug_value(value))
        .or_else(|| header_if_valid(request, TENANT_ID_HEADER, valid_tenant_slug))
        .unwrap_or_else(|| UNKNOWN_TENANT_ID.to_string())
}

fn correlation_id(request: &HttpRequest) -> String {
    header_if_valid(request, CORRELATION_ID_HEADER, valid_correlation_token)
        .or_else(|| header_if_valid(request, REQUEST_ID_HEADER, valid_correlation_token))
        .or_else(|| header_if_valid(request, TRACEPARENT_HEADER, valid_traceparent))
        .unwrap_or_else(|| MISSING_CORRELATION_ID.to_string())
}

fn valid_traceparent(value: &str) -> bool {
    if value.len() != TRACEPARENT_W3C_LEN {
        return false;
    }

    let mut parts = value.split('-');
    let Some(version) = parts.next() else {
        return false;
    };
    let Some(trace_id) = parts.next() else {
        return false;
    };
    let Some(parent_id) = parts.next() else {
        return false;
    };
    let Some(flags) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    version == "00"
        && trace_id.len() == 32
        && lower_hex(trace_id)
        && trace_id.bytes().any(|b| b != b'0')
        && parent_id.len() == 16
        && lower_hex(parent_id)
        && parent_id.bytes().any(|b| b != b'0')
        && flags.len() == 2
        && lower_hex(flags)
}

fn lower_hex(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn red_error_count(status: u16) -> u64 {
    if status >= 500 { 1 } else { 0 }
}

impl Middleware<HttpRequest, HttpResponse> for TelemetryMiddleware {
    fn handle(
        &self,
        request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        let method = request.method.name().to_string();
        // Route label: STATIC matched_template if set by dispatch, else
        // sentinel. Never the raw path — that re-introduces S6 (label
        // injection) and unbounded label cardinality. Never reconstruct
        // from path_captures — that's the F-MULTI-Q1 heuristic class.
        let route = route_label(&request);
        let tenant_id = tenant_id(&request);
        let correlation_id = correlation_id(&request);
        let start = Instant::now();
        let response = next.run(request);
        let latency_us = start.elapsed().as_micros() as u64;
        let status_class = status_class(response.status).to_string();
        self.metrics
            .record(&method, &route, response.status, latency_us);
        self.wide_events.record(WideEvent {
            schema_version: WIDE_EVENT_SCHEMA_VERSION,
            event_name: HTTP_REQUEST_COMPLETED_EVENT.to_string(),
            tenant_id,
            correlation_id,
            method,
            route,
            status_code: response.status,
            status_class,
            red_rate_count: 1,
            red_error_count: red_error_count(response.status),
            red_duration_us: latency_us,
        });
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_middleware_kernel::MiddlewareChain;
    use http_router_kernel::HttpMethod;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn req_with_template(
        method: HttpMethod,
        path: &str,
        matched_template: Option<&str>,
    ) -> HttpRequest {
        HttpRequest {
            method,
            path: path.into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: matched_template.map(String::from),
        }
    }

    fn req(method: HttpMethod, path: &str) -> HttpRequest {
        req_with_template(method, path, None)
    }

    fn req_with_headers_and_template(
        method: HttpMethod,
        path: &str,
        headers: &[(&str, &str)],
        matched_template: Option<&str>,
    ) -> HttpRequest {
        let mut request = req_with_template(method, path, matched_template);
        for (key, value) in headers {
            request
                .headers
                .insert((*key).to_string(), (*value).to_string());
        }
        request
    }

    fn wide_event_for(request: HttpRequest) -> WideEvent {
        let metrics = Arc::new(InMemoryMetrics::new());
        let middleware = TelemetryMiddleware::new(metrics);
        let wide_events = middleware.wide_events();
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(middleware));

        let _ = chain.execute(request, terminal_200);

        wide_events
            .snapshot()
            .into_iter()
            .next()
            .expect("one wide event emitted")
    }

    fn terminal_200(_req: HttpRequest) -> HttpResponse {
        HttpResponse::new(200)
    }

    fn terminal_404(_req: HttpRequest) -> HttpResponse {
        HttpResponse::new(404)
    }

    fn terminal_500(_req: HttpRequest) -> HttpResponse {
        HttpResponse::new(500)
    }

    #[test]
    fn records_one_sample_per_request() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(
            req_with_template(HttpMethod::Get, "/workspace", Some("/workspace")),
            terminal_200,
        );
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].count, 1);
        assert_eq!(snap[0].status_class, "2xx");
    }

    #[test]
    fn wide_event_records_tenant_correlation_and_red_fields() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let middleware = TelemetryMiddleware::new(metrics.clone());
        let wide_events = middleware.wide_events();
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(middleware));

        let _ = chain.execute(
            req_with_headers_and_template(
                HttpMethod::Post,
                "/messages/42",
                &[
                    ("x-tenant-id", "tenant-alpha"),
                    ("x-correlation-id", "corr-123_abc.def:456"),
                ],
                Some("/messages/{message_id}"),
            ),
            terminal_200,
        );

        let events = wide_events.snapshot();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.tenant_id, "tenant-alpha");
        assert_eq!(event.correlation_id, "corr-123_abc.def:456");
        assert_eq!(event.method, "POST");
        assert_eq!(event.route, "/messages/{message_id}");
        assert_eq!(event.status_code, 200);
        assert_eq!(event.status_class, "2xx");
        assert_eq!(event.red_rate_count, 1);
        assert_eq!(event.red_error_count, 0);
        assert!(event.red_duration_us < u64::MAX);
    }

    #[test]
    fn wide_event_marks_server_error_and_request_id_fallback() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let middleware = TelemetryMiddleware::new(metrics.clone());
        let wide_events = middleware.wide_events();
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(middleware));

        let _ = chain.execute(
            req_with_headers_and_template(
                HttpMethod::Get,
                "/healthz",
                &[("x-tenant-id", "tenant-beta"), ("x-request-id", "req-99")],
                Some("/healthz"),
            ),
            terminal_500,
        );

        let events = wide_events.snapshot();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.tenant_id, "tenant-beta");
        assert_eq!(event.correlation_id, "req-99");
        assert_eq!(event.status_code, 500);
        assert_eq!(event.status_class, "5xx");
        assert_eq!(event.red_rate_count, 1);
        assert_eq!(event.red_error_count, 1);
    }

    #[test]
    fn wide_event_rejects_invalid_tenant_path_capture_and_header_values() {
        let invalid_values = [
            "tenant\nalpha".to_string(),
            "tenant\talpha".to_string(),
            "tenant alpha".to_string(),
            "tenant/alpha".to_string(),
            "tenant.alpha".to_string(),
            "a".repeat(129),
        ];

        for invalid in invalid_values {
            let mut captured =
                req_with_template(HttpMethod::Get, "/tenants/x", Some("/tenants/{tenant_id}"));
            captured
                .path_captures
                .insert(TENANT_ID_CAPTURE_KEY.to_string(), invalid.clone());
            let captured_event = wide_event_for(captured);
            assert_eq!(captured_event.tenant_id, UNKNOWN_TENANT_ID);
            assert_ne!(captured_event.tenant_id, invalid);

            let header_event = wide_event_for(req_with_headers_and_template(
                HttpMethod::Get,
                "/tenants/x",
                &[(TENANT_ID_HEADER, invalid.as_str())],
                Some("/tenants/{tenant_id}"),
            ));
            assert_eq!(header_event.tenant_id, UNKNOWN_TENANT_ID);
            assert_ne!(header_event.tenant_id, invalid);
        }
    }

    #[test]
    fn wide_event_rejects_invalid_correlation_request_and_traceparent_values() {
        let long_correlation_id = "c".repeat(129);
        let long_request_id = "r".repeat(129);
        let cases = [
            (CORRELATION_ID_HEADER, "corr\n123"),
            (CORRELATION_ID_HEADER, "corr 123"),
            (CORRELATION_ID_HEADER, long_correlation_id.as_str()),
            (REQUEST_ID_HEADER, "req\t123"),
            (REQUEST_ID_HEADER, long_request_id.as_str()),
            (TRACEPARENT_HEADER, "00-not-a-valid-traceparent"),
            (
                TRACEPARENT_HEADER,
                "00-00000000000000000000000000000000-1111111111111111-01",
            ),
            (
                TRACEPARENT_HEADER,
                "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
            ),
            (
                TRACEPARENT_HEADER,
                "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
            ),
            (
                TRACEPARENT_HEADER,
                "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            ),
        ];

        for (header, invalid) in cases {
            let event = wide_event_for(req_with_headers_and_template(
                HttpMethod::Get,
                "/healthz",
                &[(header, invalid)],
                Some("/healthz"),
            ));
            assert_eq!(event.correlation_id, MISSING_CORRELATION_ID);
            assert_ne!(event.correlation_id, invalid);
        }
    }

    #[test]
    fn wide_event_accepts_valid_traceparent_fallback() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let event = wide_event_for(req_with_headers_and_template(
            HttpMethod::Get,
            "/healthz",
            &[(TRACEPARENT_HEADER, traceparent)],
            Some("/healthz"),
        ));

        assert_eq!(event.correlation_id, traceparent);
    }

    #[test]
    fn wide_event_route_uses_template_or_sentinel_and_emits_once_per_request() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let middleware = TelemetryMiddleware::new(metrics);
        let wide_events = middleware.wide_events();
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(middleware));

        let _ = chain.execute(
            req_with_template(
                HttpMethod::Get,
                "/users/42/posts/7",
                Some("/users/{user_id}/posts/{post_id}"),
            ),
            terminal_200,
        );
        let _ = chain.execute(req(HttpMethod::Get, "/raw/dynamic/path"), terminal_404);

        let events = wide_events.snapshot();
        assert_eq!(events.len(), 2);
        assert_eq!(wide_events.count(), 2);
        assert_eq!(events[0].route, "/users/{user_id}/posts/{post_id}");
        assert!(!events[0].route.contains("42"));
        assert!(!events[0].route.contains("/7"));
        assert_eq!(events[1].route, UNMATCHED_ROUTE_LABEL);
        assert!(!events[1].route.contains("/raw/dynamic/path"));
        assert_eq!(events[1].status_code, 404);
        assert_eq!(events[1].status_class, "4xx");
        assert_eq!(events[1].red_rate_count, 1);
        assert_eq!(events[1].red_error_count, 0);
    }

    #[test]
    fn aggregates_same_route_into_one_key() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        for _ in 0..3 {
            let _ = chain.execute(
                req_with_template(HttpMethod::Get, "/workspace", Some("/workspace")),
                terminal_200,
            );
        }
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].count, 3);
    }

    #[test]
    fn separates_by_status_class() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(
            req_with_template(HttpMethod::Get, "/a", Some("/a")),
            terminal_200,
        );
        let _ = chain.execute(
            req_with_template(HttpMethod::Get, "/a", Some("/a")),
            terminal_404,
        );
        let _ = chain.execute(
            req_with_template(HttpMethod::Get, "/a", Some("/a")),
            terminal_500,
        );
        assert_eq!(metrics.key_count(), 3);
    }

    #[test]
    fn separates_by_method() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(
            req_with_template(HttpMethod::Get, "/a", Some("/a")),
            terminal_200,
        );
        let _ = chain.execute(
            req_with_template(HttpMethod::Post, "/a", Some("/a")),
            terminal_200,
        );
        assert_eq!(metrics.key_count(), 2);
    }

    #[test]
    fn route_label_is_matched_template_not_raw_path() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        // Path has dynamic segments; matched_template is the static template.
        let request = req_with_template(
            HttpMethod::Get,
            "/users/42/posts/7",
            Some("/users/{user_id}/posts/{post_id}"),
        );
        let _ = chain.execute(request, terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        // The route label is the static template — the dynamic values 42 and 7
        // never appear.
        assert_eq!(snap[0].route, "/users/{user_id}/posts/{post_id}");
        assert!(!snap[0].route.contains("42"));
        assert!(!snap[0].route.contains("/7"));
    }

    #[test]
    fn status_class_boundaries() {
        assert_eq!(status_class(100), "1xx");
        assert_eq!(status_class(200), "2xx");
        assert_eq!(status_class(304), "3xx");
        assert_eq!(status_class(404), "4xx");
        assert_eq!(status_class(503), "5xx");
        assert_eq!(status_class(700), "5xx"); // out-of-range falls into 5xx
    }

    #[test]
    fn latency_accumulates_per_key() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        for _ in 0..5 {
            let _ = chain.execute(
                req_with_template(HttpMethod::Get, "/x", Some("/x")),
                terminal_200,
            );
        }
        let snap = metrics.snapshot();
        assert_eq!(snap[0].count, 5);
        // total_latency_us is summed (>=0; usually >0 for 5 calls).
    }

    // F3 adversarial + S6 security: a captured value of "5" that ALSO appears
    // as a literal segment elsewhere in the path used to confuse the heuristic
    // reconstruction (Q1 + S6). With matched_template from the router, the
    // captured value never enters the label.
    #[test]
    fn capture_value_appearing_elsewhere_is_not_overrewritten() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let request = req_with_template(
            HttpMethod::Get,
            "/users/5/posts/5",
            Some("/users/{user_id}/posts/{post_id}"),
        );
        let _ = chain.execute(request, terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap[0].route, "/users/{user_id}/posts/{post_id}");
        // The captured "5" MUST NOT appear in the route label (S6 closed).
        assert!(!snap[0].route.contains('5'));
    }

    // F3 adversarial + S6 security: sensitive captured value (looks like API
    // key) never lands in the metric label.
    #[test]
    fn sensitive_capture_value_excluded_from_label() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let sensitive = "sk-live-abc123def456";
        let request = req_with_template(
            HttpMethod::Get,
            &format!("/api/v1/keys/{}", sensitive),
            Some("/api/v1/keys/{key_id}"),
        );
        let _ = chain.execute(request, terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap[0].route, "/api/v1/keys/{key_id}");
        assert!(
            !snap[0].route.contains(sensitive),
            "sensitive capture value MUST NOT appear in metric label (S6)"
        );
    }

    // F3 adversarial: when no matched_template is set (e.g., direct handler
    // invocation in a unit test), label falls back to the sentinel — NOT the
    // raw path. Prevents unbounded label cardinality from test/error flows.
    #[test]
    fn missing_matched_template_falls_back_to_sentinel() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let request = req(HttpMethod::Get, "/some/random/path");
        let _ = chain.execute(request, terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap[0].route, UNMATCHED_ROUTE_LABEL);
        assert!(!snap[0].route.contains("/some"));
    }
}
