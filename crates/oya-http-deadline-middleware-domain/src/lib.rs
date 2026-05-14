//! Per-request deadline middleware — Layer 4.
//!
//! Records the wall-clock at chain entry and, when the inner chain returns,
//! compares elapsed time against the configured budget. On budget exceedance
//! the original response is replaced with HTTP 504 (Gateway Timeout) and a
//! structured JSON error.
//!
//! The middleware-kernel chain is sync, so we cannot actually abort an
//! in-flight handler here — that requires async cooperation. What this
//! middleware *does* do is enforce post-hoc that the deadline was respected,
//! which gives us the lane-grade SLO enforcement (slow responses get
//! converted to 504s rather than leaking past the SLA).
//!
//! Std-only.

use std::time::Instant;

use bytes::Bytes;
use oya_http_middleware_kernel::{Middleware, Next};
use oya_http_runtime_hyper_adapter::{HyperRequest, HyperResponse};

#[derive(Clone, Debug)]
pub struct DeadlineMiddleware {
    budget_ms: u64,
}

impl DeadlineMiddleware {
    pub fn new(budget_ms: u64) -> Self {
        Self { budget_ms }
    }

    pub fn budget_ms(&self) -> u64 {
        self.budget_ms
    }
}

/// Public so callers / tests can inspect the deadline-exceeded body shape.
pub const DEADLINE_EXCEEDED_BODY_PREFIX: &str = "{\"error\":\"deadline-exceeded\"";

impl Middleware<HyperRequest, HyperResponse> for DeadlineMiddleware {
    fn handle(
        &self,
        request: HyperRequest,
        next: Next<'_, HyperRequest, HyperResponse>,
    ) -> HyperResponse {
        let start = Instant::now();
        let response = next.run(request);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms > self.budget_ms {
            HyperResponse::new(504)
                .with_header("content-type", "application/json")
                .with_header("x-deadline-budget-ms", self.budget_ms.to_string())
                .with_header("x-deadline-elapsed-ms", elapsed_ms.to_string())
                .with_body(Bytes::from(format!(
                    "{}{}\"budget_ms\":{},\"elapsed_ms\":{}}}",
                    DEADLINE_EXCEEDED_BODY_PREFIX, ",", self.budget_ms, elapsed_ms
                )))
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

    fn req() -> HyperRequest {
        HyperRequest {
            method: HttpMethod::Get,
            path: "/x".into(),
            headers: BTreeMap::new(),
            body: Bytes::new(),
            path_captures: BTreeMap::new(),
        }
    }

    fn fast_terminal(_req: HyperRequest) -> HyperResponse {
        HyperResponse::new(200).with_body(Bytes::from_static(b"fast"))
    }

    fn slow_terminal(_req: HyperRequest) -> HyperResponse {
        sleep(Duration::from_millis(20));
        HyperResponse::new(200).with_body(Bytes::from_static(b"slow"))
    }

    #[test]
    fn within_budget_passes_through() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(DeadlineMiddleware::new(1_000)));
        let response = chain.execute(req(), fast_terminal);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, Bytes::from_static(b"fast"));
    }

    #[test]
    fn over_budget_replaced_with_504() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(DeadlineMiddleware::new(1)));
        let response = chain.execute(req(), slow_terminal);
        assert_eq!(response.status, 504);
        assert!(
            response
                .headers
                .get("x-deadline-budget-ms")
                .map(String::as_str)
                == Some("1")
        );
        assert!(response
            .body
            .windows(DEADLINE_EXCEEDED_BODY_PREFIX.len())
            .any(|w| w == DEADLINE_EXCEEDED_BODY_PREFIX.as_bytes()));
    }

    #[test]
    fn budget_ms_accessor() {
        let mw = DeadlineMiddleware::new(500);
        assert_eq!(mw.budget_ms(), 500);
    }

    #[test]
    fn body_includes_elapsed_field() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(DeadlineMiddleware::new(1)));
        let response = chain.execute(req(), slow_terminal);
        let body = std::str::from_utf8(&response.body).unwrap();
        assert!(body.contains("\"elapsed_ms\":"));
        assert!(body.contains("\"budget_ms\":1"));
    }

    #[test]
    fn elapsed_header_set() {
        let chain: MiddlewareChain<HyperRequest, HyperResponse> =
            MiddlewareChain::new().push(Box::new(DeadlineMiddleware::new(1)));
        let response = chain.execute(req(), slow_terminal);
        assert!(response.headers.contains_key("x-deadline-elapsed-ms"));
    }
}
