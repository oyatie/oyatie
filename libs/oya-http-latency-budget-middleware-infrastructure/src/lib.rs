//! Per-request latency-budget reporter — Layer 4 infrastructure.
//!
//! Records the wall-clock at chain entry and, when the inner chain returns,
//! compares elapsed time against the configured budget. On budget exceedance
//! the original response is replaced with HTTP 504 (Gateway Timeout) and a
//! structured JSON error.
//!
//! ⚠️ HONEST NAMING (per ADR-0093):
//! This middleware is a POST-HOC LATENCY REPORTER, not a cancellation
//! mechanism. The middleware-kernel chain is sync, so we cannot abort an
//! in-flight handler — that requires async cooperation. What this middleware
//! DOES is enforce post-hoc that the budget was respected, which yields SLO
//! reporting (slow responses get converted to 504s rather than leaking past
//! the SLA). The slow work STILL RAN; side effects already happened.
//!
//! Original name `DeadlineMiddleware` was deceptive — readers reasonably
//! expected real deadline cancellation. Renamed to `LatencyBudgetReporter`.
//! When the async-chain refactor lands (FixupTask F-ASYNCCHAIN-1), a separate
//! `DeadlineMiddleware` may be introduced as the canceling variant; until
//! then, the honest name is the right name.
//!
//! Std-only.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::Instant;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, MiddlewareChain, Next};

#[derive(Clone, Debug)]
pub struct LatencyBudgetReporter {
    budget_ms: u64,
}

impl LatencyBudgetReporter {
    pub fn new(budget_ms: u64) -> Self {
        Self { budget_ms }
    }

    pub fn budget_ms(&self) -> u64 {
        self.budget_ms
    }
}

/// Build the honest post-hoc latency evidence composition.
///
/// The provided observer middleware is registered outside the reporter so it
/// sees the final response after [`LatencyBudgetReporter`] has converted a slow
/// inner 200 into a 504. This is loopback/runtime transcript composition only:
/// it does not cancel in-flight work and it does not constitute measured SLO,
/// production-ready, or hyperscaler-readiness evidence.
pub fn latency_budget_runtime_evidence_chain(
    outcome_observer: Box<dyn Middleware<HttpRequest, HttpResponse>>,
    reporter: LatencyBudgetReporter,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
        .push(outcome_observer)
        .push(Box::new(reporter))
}

/// Public so callers / tests can inspect the budget-exceeded body shape.
pub const LATENCY_BUDGET_EXCEEDED_BODY_PREFIX: &str = "{\"error\":\"latency-budget-exceeded\"";

impl Middleware<HttpRequest, HttpResponse> for LatencyBudgetReporter {
    fn handle(
        &self,
        request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        let start = Instant::now();
        let response = next.run(request);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms > self.budget_ms {
            HttpResponse::new(504)
                .with_header("content-type", "application/json")
                .with_header("x-latency-budget-ms", self.budget_ms.to_string())
                .with_header("x-latency-elapsed-ms", elapsed_ms.to_string())
                .with_body(
                    format!(
                        "{}{}\"budget_ms\":{},\"elapsed_ms\":{}}}",
                        LATENCY_BUDGET_EXCEEDED_BODY_PREFIX, ",", self.budget_ms, elapsed_ms
                    )
                    .into_bytes(),
                )
        } else {
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_middleware_kernel::MiddlewareChain;
    use oya_http_router_kernel::HttpMethod;
    use std::collections::BTreeMap;
    use std::thread::sleep;
    use std::time::Duration;

    fn req() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            path: "/x".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    fn fast_terminal(_req: HttpRequest) -> HttpResponse {
        HttpResponse::new(200).with_body(b"fast".to_vec())
    }

    fn slow_terminal(_req: HttpRequest) -> HttpResponse {
        sleep(Duration::from_millis(20));
        HttpResponse::new(200).with_body(b"slow".to_vec())
    }

    #[test]
    fn within_budget_passes_through() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1_000)));
        let response = chain.execute(req(), fast_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"fast".to_vec());
    }

    #[test]
    fn over_budget_replaced_with_504() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1)));
        let response = chain.execute(req(), slow_terminal);
        assert_eq!(response.status, 504);
        assert_eq!(
            response
                .headers
                .get("x-latency-budget-ms")
                .map(String::as_str),
            Some("1")
        );
        assert!(
            response
                .body
                .windows(LATENCY_BUDGET_EXCEEDED_BODY_PREFIX.len())
                .any(|w| w == LATENCY_BUDGET_EXCEEDED_BODY_PREFIX.as_bytes())
        );
    }

    #[test]
    fn budget_ms_accessor() {
        let mw = LatencyBudgetReporter::new(500);
        assert_eq!(mw.budget_ms(), 500);
    }

    #[test]
    fn body_includes_elapsed_field() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1)));
        let response = chain.execute(req(), slow_terminal);
        let body = std::str::from_utf8(&response.body).unwrap();
        assert!(body.contains("\"elapsed_ms\":"));
        assert!(body.contains("\"budget_ms\":1"));
    }

    #[test]
    fn elapsed_header_set() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1)));
        let response = chain.execute(req(), slow_terminal);
        assert!(response.headers.contains_key("x-latency-elapsed-ms"));
    }

    // F3 adversarial: the honesty test. Slow handler side effects happen
    // BEFORE 504 is returned, because this is a post-hoc reporter, NOT
    // cancellation. The name "LatencyBudgetReporter" reflects this — a
    // type called "DeadlineMiddleware" would lie.
    #[test]
    fn slow_handler_runs_before_504_overwrite_proves_post_hoc_semantics() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_for_terminal = counter.clone();
        let terminal = move |_req: HttpRequest| -> HttpResponse {
            counter_for_terminal.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(20));
            HttpResponse::new(200).with_body(b"work-done".to_vec())
        };
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1)));
        let response = chain.execute(req(), terminal);
        // 504 returned to client.
        assert_eq!(response.status, 504);
        // BUT the slow work DID run exactly once. Side-effects are not
        // prevented — only the response is replaced. THIS is why the name
        // matters: a reader who sees "LatencyBudgetReporter" knows what to
        // expect; a reader who sees "DeadlineMiddleware" expects cancellation
        // that doesn't exist.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // F3 adversarial: body content matches the new prefix (proves the rename
    // landed everywhere — no stragglers from the old "deadline-exceeded"
    // string).
    #[test]
    fn body_contains_latency_budget_exceeded_prefix_not_old_deadline() {
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(LatencyBudgetReporter::new(1)));
        let response = chain.execute(req(), slow_terminal);
        let body = std::str::from_utf8(&response.body).unwrap();
        assert!(body.contains("latency-budget-exceeded"));
        assert!(!body.contains("deadline-exceeded"));
    }

    #[test]
    fn slow_504_is_single_telemetry_red_outcome_without_cancellation_claim() {
        use oya_http_telemetry_middleware_infrastructure::{
            HTTP_REQUEST_COMPLETED_EVENT, InMemoryMetrics, InMemoryWideEvents, TelemetryMiddleware,
        };
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let metrics = Arc::new(InMemoryMetrics::new());
        let wide_events = Arc::new(InMemoryWideEvents::new());
        let telemetry = TelemetryMiddleware::with_wide_events(metrics.clone(), wide_events.clone());
        let handler_side_effects = Arc::new(AtomicUsize::new(0));
        let handler_side_effects_for_terminal = handler_side_effects.clone();
        let terminal = move |_req: HttpRequest| -> HttpResponse {
            handler_side_effects_for_terminal.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(20));
            HttpResponse::new(200).with_body(b"work-done".to_vec())
        };

        let chain = latency_budget_runtime_evidence_chain(
            Box::new(telemetry),
            LatencyBudgetReporter::new(1),
        );
        let response = chain.execute(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/messages/42".into(),
                headers: BTreeMap::new(),
                body: Vec::new(),
                path_captures: BTreeMap::new(),
                matched_template: Some("/messages/{message_id}".into()),
            },
            terminal,
        );

        assert_eq!(response.status, 504);
        assert_eq!(
            response
                .headers
                .get("x-latency-budget-ms")
                .map(String::as_str),
            Some("1")
        );
        let elapsed_ms = response
            .headers
            .get("x-latency-elapsed-ms")
            .and_then(|value| value.parse::<u64>().ok())
            .expect("latency transcript includes numeric elapsed milliseconds");
        assert!(elapsed_ms >= 1);
        let body = std::str::from_utf8(&response.body).unwrap();
        assert!(body.contains("\"error\":\"latency-budget-exceeded\""));
        assert!(body.contains("\"budget_ms\":1"));
        assert!(body.contains("\"elapsed_ms\":"));

        assert_eq!(handler_side_effects.load(Ordering::SeqCst), 1);

        let samples = metrics.snapshot();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].route, "/messages/{message_id}");
        assert_eq!(samples[0].status_class, "5xx");
        assert_eq!(samples[0].count, 1);

        let events = wide_events.snapshot();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_name, HTTP_REQUEST_COMPLETED_EVENT);
        assert_eq!(event.route, "/messages/{message_id}");
        assert_eq!(event.status_code, 504);
        assert_eq!(event.status_class, "5xx");
        assert_eq!(event.red_rate_count, 1);
        assert_eq!(event.red_error_count, 1);
        assert!(event.red_duration_us > 0);
    }
}
