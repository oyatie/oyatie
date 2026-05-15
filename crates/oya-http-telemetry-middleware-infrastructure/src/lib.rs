//! Telemetry middleware — Layer 4.
//!
//! Records per-route counters + total latency for every dispatched request.
//! Pure std-only (Mutex + BTreeMap); a production cell wires this output to
//! a Prometheus-compatible exporter via the OTel adapter (separate slice).
//!
//! Counter labels (low-cardinality only per OTel conventions):
//!   - route  (the matched_template from the router — STATIC, NEVER the raw
//!             path; this closes the S6 metric-label-injection class)
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

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Instant;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TelemetrySample {
    pub method: String,
    pub route: String,
    pub status_class: String,
    pub count: u64,
    pub total_latency_us: u64,
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
        let mut by_key = self.by_key.lock().expect("metrics poisoned");
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
            .expect("metrics poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn key_count(&self) -> usize {
        self.by_key.lock().expect("metrics poisoned").len()
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

#[derive(Clone, Debug)]
pub struct TelemetryMiddleware {
    metrics: std::sync::Arc<InMemoryMetrics>,
}

impl TelemetryMiddleware {
    pub fn new(metrics: std::sync::Arc<InMemoryMetrics>) -> Self {
        Self { metrics }
    }

    pub fn metrics(&self) -> std::sync::Arc<InMemoryMetrics> {
        self.metrics.clone()
    }
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
        let route = request
            .matched_template
            .clone()
            .unwrap_or_else(|| UNMATCHED_ROUTE_LABEL.to_string());
        let start = Instant::now();
        let response = next.run(request);
        let latency_us = start.elapsed().as_micros() as u64;
        self.metrics
            .record(&method, &route, response.status, latency_us);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_middleware_kernel::MiddlewareChain;
    use oya_http_router_kernel::HttpMethod;
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
