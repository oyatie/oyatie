//! HTTP middleware kernel — middleware-chain abstraction plus the
//! transport-neutral request/response structs used by runtime adapters.
//!
//! Layer 2 of the hyper foundation. Generic over `Req` + `Resp` so consuming
//! crates can plug in any concrete request/response type. The default
//! `HttpRequest` / `HttpResponse` shapes stay here so middleware crates
//! depend inward on kernels instead of sideways on the Hyper adapter.
//!
//! Kernel body type is `Vec<u8>` (std-only). The hyper adapter converts to
//! and from `hyper::body::Bytes` at the outer boundary. This keeps every
//! middleware/consumer dep-free of `bytes` and concentrates all hyper-family
//! deps in `oya-http-runtime-hyper-adapter` per ADR-0090 + ADR-0092.
//!
//! `HttpRequest` carries `matched_template: Option<String>` (ADR-0092 Phase 4)
//! so middlewares such as telemetry can use the registered template as a
//! low-cardinality metric label instead of reconstructing it from the raw
//! path — eliminating the S6 metric-label-injection class and the F-MULTI-Q1
//! approximate-heuristic quality bug.
//!
//! Chain semantics: each `Middleware::handle(&self, req, next)` may either
//!   - call `next.run(req)` to continue down the chain (returning that result
//!     optionally transformed), or
//!   - short-circuit by returning its own `Resp` without calling `next`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use http_router_kernel::HttpMethod;

/// HTTP request as seen by middlewares + handlers.
///
/// Transport-neutral: the hyper runtime adapter converts concrete
/// `hyper::Request<Incoming>` values into this shape at the outer boundary.
/// Body bytes are owned `Vec<u8>` so this kernel pulls no hyper-family deps.
///
/// `matched_template` is set by the dispatch boundary when a router match
/// succeeds; it's `None` for direct-handler invocations (e.g., unit tests
/// constructing requests without going through Router).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpRequest {
    pub method: HttpMethod,                      // data_class: INTERNAL_ONLY
    pub path: String,                            // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>,       // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                           // data_class: INTERNAL_ONLY
    pub path_captures: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub matched_template: Option<String>, // data_class: INTERNAL_ONLY (static template, not raw path)
}

/// HTTP response shape materialized as bytes for non-streaming routes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,                       // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
}

impl HttpResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }

    /// Insert a response header.
    ///
    /// Per ADR-0092 Phase 10 (S10 + S1 defense):
    ///   * Header name is lowercased on insert to keep `HttpRequest`/
    ///     `HttpResponse` header maps case-canonical (S1 closed).
    ///   * Header VALUE is sanitized: CR, LF, and NUL are stripped. These
    ///     bytes would otherwise enable header-injection / response-
    ///     splitting attacks once the hyper adapter serializes the
    ///     response. Stripping is defense-in-depth — if a caller passes
    ///     attacker-controlled input here without prior validation, the
    ///     on-wire response is still safe.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into().to_ascii_lowercase();
        let value: String = value
            .into()
            .chars()
            .filter(|c| *c != '\r' && *c != '\n' && *c != '\0')
            .collect();
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn not_found() -> Self {
        Self::new(404).with_body(b"not found".to_vec())
    }

    pub fn method_not_allowed() -> Self {
        Self::new(405).with_body(b"method not allowed".to_vec())
    }
}

/// A chain handler that can be called by middleware to continue down the stack.
pub struct Next<'a, Req, Resp> {
    chain: &'a [Box<dyn Middleware<Req, Resp>>],
    terminal: &'a dyn Fn(Req) -> Resp,
}

impl<Req, Resp> Next<'_, Req, Resp> {
    pub fn run(self, request: Req) -> Resp {
        match self.chain.split_first() {
            None => (self.terminal)(request),
            Some((head, tail)) => head.handle(
                request,
                Next {
                    chain: tail,
                    terminal: self.terminal,
                },
            ),
        }
    }
}

/// Trait every middleware implements. The chain composes these in registered
/// order; the terminal handler runs last when every middleware calls `next.run`.
pub trait Middleware<Req, Resp>: Send + Sync {
    fn handle(&self, request: Req, next: Next<'_, Req, Resp>) -> Resp;
}

/// Typed handler trait — preferred over raw `Fn(HttpRequest) -> HttpResponse`
/// closures (per ADR-0094).
///
/// `type Error: Into<HttpResponse>` lets handler authors return structured
/// errors that render to a response at the framework boundary, instead of
/// constructing an `HttpResponse` for every error path inside the handler
/// body. This separates "happy path computation" from "error-to-response
/// mapping" — the latter belongs in one place (the `From<MyError> for
/// HttpResponse` impl), not scattered through handlers.
///
/// Existing closure-based handlers continue to work via the
/// `oya-http-runtime-hyper-adapter::SyncHandler` alias. To migrate one
/// handler, implement `Handler` on your service type and wrap with
/// `handler_to_sync(...)` at the registration site.
///
/// Why no blanket `impl<F: Fn(...)-> HttpResponse> Handler for F`: that
/// would conflict with a future blanket `impl<F: Fn(...)-> Result<...>>
/// Handler for F` under Rust's coherence rules. Explicit wrappers are
/// clearer and avoid trait-ambiguity errors.
pub trait Handler: Send + Sync {
    type Error: Into<HttpResponse>;
    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>;
}

/// Call a `Handler`, collapsing Err into a rendered `HttpResponse`. Useful
/// when bridging a typed `Handler` into the closure-shaped `SyncHandler`
/// alias the router holds today.
pub fn call_into_response<H: Handler>(handler: &H, req: HttpRequest) -> HttpResponse {
    match handler.call(req) {
        Ok(r) => r,
        Err(e) => e.into(),
    }
}

/// Composable chain of middlewares + a terminal handler.
pub struct MiddlewareChain<Req, Resp> {
    middlewares: Vec<Box<dyn Middleware<Req, Resp>>>,
}

impl<Req, Resp> Default for MiddlewareChain<Req, Resp> {
    fn default() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }
}

impl<Req, Resp> MiddlewareChain<Req, Resp> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, middleware: Box<dyn Middleware<Req, Resp>>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn count(&self) -> usize {
        self.middlewares.len()
    }

    /// Execute the chain against a terminal handler. The terminal handler is
    /// what runs when every middleware has called `next.run(req)`.
    pub fn execute<F>(&self, request: Req, terminal: F) -> Resp
    where
        F: Fn(Req) -> Resp,
    {
        let next = Next {
            chain: &self.middlewares,
            terminal: &terminal,
        };
        next.run(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Counter(Arc<AtomicUsize>);
    impl<Req, Resp> Middleware<Req, Resp> for Counter
    where
        Req: Send + Sync + 'static,
        Resp: Send + Sync + 'static,
    {
        fn handle(&self, request: Req, next: Next<'_, Req, Resp>) -> Resp {
            self.0.fetch_add(1, Ordering::SeqCst);
            next.run(request)
        }
    }

    struct ShortCircuit<R>(R);
    impl<Req, Resp> Middleware<Req, Resp> for ShortCircuit<Resp>
    where
        Req: Send + Sync + 'static,
        Resp: Clone + Send + Sync + 'static,
    {
        fn handle(&self, _request: Req, _next: Next<'_, Req, Resp>) -> Resp {
            self.0.clone()
        }
    }

    #[test]
    fn empty_chain_runs_terminal() {
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new();
        let response = chain.execute("hello", |req| format!("handled:{req}"));
        assert_eq!(response, "handled:hello");
    }

    #[test]
    fn single_middleware_invokes_next() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> =
            MiddlewareChain::new().push(Box::new(Counter(counter.clone())));
        let response = chain.execute("x", |req| format!("handled:{req}"));
        assert_eq!(response, "handled:x");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn multiple_middleware_all_invoked() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(Counter(counter.clone())));
        let _ = chain.execute("x", |_| String::from("done"));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn short_circuit_skips_terminal() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(ShortCircuit(String::from("denied"))))
            .push(Box::new(Counter(counter.clone())));
        let response = chain.execute("x", |_| String::from("should-not-run"));
        assert_eq!(response, "denied");
        // First counter ran (called next), short-circuit ran, third counter did NOT run.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn count_reflects_pushed_middleware() {
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(Arc::new(AtomicUsize::new(0)))))
            .push(Box::new(Counter(Arc::new(AtomicUsize::new(0)))));
        assert_eq!(chain.count(), 2);
    }

    #[test]
    fn middleware_runs_in_registered_order() {
        struct Tag {
            tag: &'static str,
            log: Arc<std::sync::Mutex<Vec<&'static str>>>,
        }
        impl Middleware<&'static str, String> for Tag {
            fn handle(&self, req: &'static str, next: Next<'_, &'static str, String>) -> String {
                self.log.lock().unwrap().push(self.tag);
                next.run(req)
            }
        }
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Tag {
                tag: "a",
                log: log.clone(),
            }))
            .push(Box::new(Tag {
                tag: "b",
                log: log.clone(),
            }))
            .push(Box::new(Tag {
                tag: "c",
                log: log.clone(),
            }));
        let _ = chain.execute("x", |_| String::from("end"));
        let recorded = log.lock().unwrap().clone();
        assert_eq!(recorded, vec!["a", "b", "c"]);
    }

    // ---- HttpRequest / HttpResponse construction tests ----
    // F3 adversarial: prove the kernel types can be built with only std types
    // (no bytes, no hyper). If these tests compile, the seam is intact for
    // the kernel surface.

    #[test]
    fn http_request_buildable_with_std_only() {
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/x".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        };
        assert_eq!(req.method, HttpMethod::Get);
        assert!(req.body.is_empty());
        assert!(req.matched_template.is_none());
    }

    #[test]
    fn http_request_with_matched_template() {
        let req = HttpRequest {
            method: HttpMethod::Get,
            path: "/users/42".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: Some("/users/{user_id}".into()),
        };
        assert_eq!(req.matched_template.as_deref(), Some("/users/{user_id}"));
    }

    #[test]
    fn http_response_buildable_with_std_only() {
        let resp = HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(b"{\"ok\":true}".to_vec());
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"{\"ok\":true}".to_vec());
    }

    #[test]
    fn not_found_carries_canonical_body() {
        let resp = HttpResponse::not_found();
        assert_eq!(resp.status, 404);
        assert_eq!(resp.body, b"not found".to_vec());
    }

    #[test]
    fn method_not_allowed_carries_canonical_body() {
        let resp = HttpResponse::method_not_allowed();
        assert_eq!(resp.status, 405);
        assert_eq!(resp.body, b"method not allowed".to_vec());
    }

    #[test]
    fn with_body_accepts_vec_from_string() {
        let resp = HttpResponse::new(200).with_body(b"hello".to_vec());
        assert_eq!(resp.body, b"hello");
    }

    // ---- Handler trait tests (ADR-0094 + F-MULTI-Q4) ----

    /// Example Error type used by tests. Authors who want typed handler
    /// errors implement `From<MyErr> for HttpResponse` to control rendering.
    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestErr {
        NotFound,
        BadInput(String),
        InternalBoom,
    }

    impl From<TestErr> for HttpResponse {
        fn from(e: TestErr) -> Self {
            match e {
                TestErr::NotFound => HttpResponse::new(404).with_body(b"resource missing".to_vec()),
                TestErr::BadInput(msg) => HttpResponse::new(400).with_body(msg.into_bytes()),
                TestErr::InternalBoom => {
                    HttpResponse::new(500).with_body(b"server malfunctioned".to_vec())
                }
            }
        }
    }

    struct EchoHandler;
    impl Handler for EchoHandler {
        type Error = TestErr;
        fn call(&self, req: HttpRequest) -> Result<HttpResponse, TestErr> {
            Ok(HttpResponse::new(200).with_body(req.body))
        }
    }

    struct AlwaysFails(TestErr);
    impl Handler for AlwaysFails {
        type Error = TestErr;
        fn call(&self, _req: HttpRequest) -> Result<HttpResponse, TestErr> {
            Err(self.0.clone())
        }
    }

    fn build_request(body: Vec<u8>) -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Post,
            path: "/echo".into(),
            headers: BTreeMap::new(),
            body,
            path_captures: BTreeMap::new(),
            matched_template: Some("/echo".into()),
        }
    }

    #[test]
    fn handler_ok_path_returns_response() {
        let h = EchoHandler;
        let resp = call_into_response(&h, build_request(b"hi".to_vec()));
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"hi".to_vec());
    }

    // F3 adversarial: typed Err renders via From<TestErr> for HttpResponse —
    // the error-mapping lives in one place, not inside the handler body.
    #[test]
    fn handler_err_path_renders_via_into_response_not_found() {
        let h = AlwaysFails(TestErr::NotFound);
        let resp = call_into_response(&h, build_request(Vec::new()));
        assert_eq!(resp.status, 404);
        assert_eq!(resp.body, b"resource missing".to_vec());
    }

    #[test]
    fn handler_err_path_renders_via_into_response_bad_input() {
        let h = AlwaysFails(TestErr::BadInput("missing 'x' field".into()));
        let resp = call_into_response(&h, build_request(Vec::new()));
        assert_eq!(resp.status, 400);
        assert_eq!(resp.body, b"missing 'x' field".to_vec());
    }

    #[test]
    fn handler_err_path_renders_via_into_response_internal() {
        let h = AlwaysFails(TestErr::InternalBoom);
        let resp = call_into_response(&h, build_request(Vec::new()));
        assert_eq!(resp.status, 500);
        assert_eq!(resp.body, b"server malfunctioned".to_vec());
    }

    // F3 adversarial: the SAME handler can produce different rendered
    // responses depending on its Error variant — proves the typed-error
    // path is meaningfully distinct from a closure that always builds the
    // HttpResponse itself.
    // ---- Phase 10 (S10 + S1 header hardening) F3 adversarial fixtures ----

    #[test]
    fn with_header_strips_cr_in_value() {
        let resp = HttpResponse::new(200).with_header("x-test", "value\rinjected");
        assert_eq!(
            resp.headers.get("x-test").map(String::as_str),
            Some("valueinjected")
        );
    }

    #[test]
    fn with_header_strips_lf_in_value() {
        let resp = HttpResponse::new(200).with_header("x-test", "value\nset-cookie: pwned=yes");
        // After LF strip, the smuggled header collapses into the value:
        assert_eq!(
            resp.headers.get("x-test").map(String::as_str),
            Some("valueset-cookie: pwned=yes")
        );
        // Critically: there is NO `set-cookie` header — it was never set as
        // a separate header, just merged into the original value (and the
        // hyper adapter will treat the whole string as one header value).
        assert!(!resp.headers.contains_key("set-cookie"));
    }

    #[test]
    fn with_header_strips_null_in_value() {
        let resp = HttpResponse::new(200).with_header("x-test", "value\0null");
        assert_eq!(
            resp.headers.get("x-test").map(String::as_str),
            Some("valuenull")
        );
    }

    #[test]
    fn with_header_lowercases_key() {
        let resp = HttpResponse::new(200)
            .with_header("X-Custom-Header", "value")
            .with_header("Content-Type", "application/json");
        assert_eq!(
            resp.headers.get("x-custom-header").map(String::as_str),
            Some("value")
        );
        assert_eq!(
            resp.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        // Original-case lookups MUST miss.
        assert!(!resp.headers.contains_key("X-Custom-Header"));
        assert!(!resp.headers.contains_key("Content-Type"));
    }

    // F3 defense-in-depth: combined attack (mixed case key + CRLF in value)
    // is fully neutralized.
    #[test]
    fn with_header_combined_attack_neutralized() {
        let resp = HttpResponse::new(200)
            .with_header("X-Forwarded-For", "1.2.3.4\r\nAuthorization: Bearer attack");
        assert_eq!(
            resp.headers.get("x-forwarded-for").map(String::as_str),
            Some("1.2.3.4Authorization: Bearer attack")
        );
        assert!(!resp.headers.contains_key("authorization"));
    }

    #[test]
    fn handler_err_variants_render_to_distinct_responses() {
        let nf = call_into_response(&AlwaysFails(TestErr::NotFound), build_request(Vec::new()));
        let bi = call_into_response(
            &AlwaysFails(TestErr::BadInput("nope".into())),
            build_request(Vec::new()),
        );
        let ib = call_into_response(
            &AlwaysFails(TestErr::InternalBoom),
            build_request(Vec::new()),
        );
        assert_ne!(nf.status, bi.status);
        assert_ne!(bi.status, ib.status);
        assert_ne!(nf.body, ib.body);
    }
}
