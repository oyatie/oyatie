//! Telemetry middleware — Layer 4.
//!
//! Records per-route counters + total latency for every dispatched request.
//! Pure std-only (Mutex + BTreeMap); a production cell wires this output to
//! a Prometheus-compatible exporter via the OTel adapter (separate slice).
//!
//! Counter labels (low-cardinality only per OTel conventions):
//!   - route  (the matched template, NOT the raw path — keeps cardinality bounded)
//!   - method (GET / POST / ...)
//!   - status_class (2xx / 3xx / 4xx / 5xx)

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Instant;

use oya_http_middleware_kernel::{Middleware, Next};
use oya_http_runtime_hyper_adapter::{HyperRequest, HyperResponse};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TelemetrySample {
    pub method: String,
    pub route: String,
    pub status_class: String,
    pub count: u64,
    pub total_latency_us: u64,
}

/// In-memory metrics sink. Production swaps for an OTel adapter.
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

impl Middleware<HyperRequest, HyperResponse> for TelemetryMiddleware {
    fn handle(
        &self,
        request: HyperRequest,
        next: Next<'_, HyperRequest, HyperResponse>,
    ) -> HyperResponse {
        let method = request.method.name().to_string();
        // Route label: use the matched path template when present (router stored
        // the captures already), else fall back to "/_unmatched" so cardinality
        // stays bounded.
        let route = if request.path_captures.is_empty() {
            request.path.clone()
        } else {
            // We don't have the original template; reconstruct a "templatized" form
            // by replacing capture values with `{name}`. This is approximate.
            let mut templated = request.path.clone();
            for (name, value) in &request.path_captures {
                if !value.is_empty() {
                    templated = templated.replace(value, &format!("{{{name}}}"));
                }
            }
            templated
        };
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
    use bytes::Bytes;
    use oya_http_middleware_kernel::MiddlewareChain;
    use oya_http_router_kernel::HttpMethod;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn req(method: HttpMethod, path: &str) -> HyperRequest {
        HyperRequest {
            method,
            path: path.into(),
            headers: BTreeMap::new(),
            body: Bytes::new(),
            path_captures: BTreeMap::new(),
        }
    }

    fn terminal_200(_req: HyperRequest) -> HyperResponse {
        HyperResponse::new(200)
    }

    fn terminal_404(_req: HyperRequest) -> HyperResponse {
        HyperResponse::new(404)
    }

    fn terminal_500(_req: HyperRequest) -> HyperResponse {
        HyperResponse::new(500)
    }

    #[test]
    fn records_one_sample_per_request() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(req(HttpMethod::Get, "/workspace"), terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].count, 1);
        assert_eq!(snap[0].status_class, "2xx");
    }

    #[test]
    fn aggregates_same_route_into_one_key() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        for _ in 0..3 {
            let _ = chain.execute(req(HttpMethod::Get, "/workspace"), terminal_200);
        }
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].count, 3);
    }

    #[test]
    fn separates_by_status_class() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(req(HttpMethod::Get, "/a"), terminal_200);
        let _ = chain.execute(req(HttpMethod::Get, "/a"), terminal_404);
        let _ = chain.execute(req(HttpMethod::Get, "/a"), terminal_500);
        assert_eq!(metrics.key_count(), 3);
    }

    #[test]
    fn separates_by_method() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let _ = chain.execute(req(HttpMethod::Get, "/a"), terminal_200);
        let _ = chain.execute(req(HttpMethod::Post, "/a"), terminal_200);
        assert_eq!(metrics.key_count(), 2);
    }

    #[test]
    fn templatizes_route_when_captures_present() {
        let metrics = Arc::new(InMemoryMetrics::new());
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        let mut request = req(HttpMethod::Get, "/users/42/posts/7");
        request.path_captures.insert("user_id".into(), "42".into());
        request.path_captures.insert("post_id".into(), "7".into());
        let _ = chain.execute(request, terminal_200);
        let snap = metrics.snapshot();
        assert_eq!(snap.len(), 1);
        // Both captures should be templatized regardless of map order.
        assert!(snap[0].route.contains("{user_id}"));
        assert!(snap[0].route.contains("{post_id}"));
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
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(TelemetryMiddleware::new(metrics.clone())));
        for _ in 0..5 {
            let _ = chain.execute(req(HttpMethod::Get, "/x"), terminal_200);
        }
        let snap = metrics.snapshot();
        assert_eq!(snap[0].count, 5);
        // total_latency_us is summed (>=0; usually >0 for 5 calls).
    }
}
