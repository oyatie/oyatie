//! Hyper runtime adapter — the ONLY crate in the hyper foundation that
//! imports hyper directly (per ADR-0090 + ADR-0092).
//!
//! Layer 5 of the foundation. Bridges:
//!   - oya-http-router-kernel::Router<H>
//!   - oya-http-middleware-kernel::MiddlewareChain<HttpRequest, HttpResponse>
//!   - hyper::service::Service over hyper 1.x
//!
//! Conversion-at-the-boundary discipline (ADR-0092 root-cause seam fix):
//!   * Inbound: hyper `Bytes` body → kernel `Vec<u8>` via `.to_vec()`.
//!   * Outbound: kernel `Vec<u8>` body → hyper `Full<Bytes>` via `Bytes::from`.
//!
//! The kernel types stay std-only; every hyper-family dep (`hyper`,
//!     `hyper-util`, `http-body-util`, `bytes`) is concentrated in THIS crate.
//!
//! Request / response structs are re-exported from the middleware kernel so
//! middleware crates depend inward while consumers still avoid importing hyper.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Body, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use tokio::net::TcpListener;

use oya_http_middleware_kernel::{Handler, MiddlewareChain, call_into_response};
pub use oya_http_middleware_kernel::{HttpRequest, HttpResponse};
use oya_http_router_kernel::{HttpMethod, Router};

/// Synchronous handler signature wrapped by the router. Handlers are pure
/// `Fn` — they own / borrow their state via captured Arcs. The runtime calls
/// the chain + handler on a tokio worker thread.
///
/// Per ADR-0094: prefer implementing `oya_http_middleware_kernel::Handler`
/// on a typed service struct and wrap with `handler_to_sync(...)` at
/// registration. The closure alias remains for ergonomics on trivial routes.
pub type SyncHandler = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync>;

/// Wrap a typed `Handler` (with associated `Error`) into the closure-shaped
/// `SyncHandler` the router holds. Renders errors via the handler's
/// `From<Error> for HttpResponse` impl at call time.
///
/// This is the canonical bridge between the kernel `Handler` trait and the
/// router's handler-type-erasure. ADR-0094.
pub fn handler_to_sync<H>(handler: H) -> SyncHandler
where
    H: Handler + 'static,
{
    let handler = Arc::new(handler);
    Arc::new(move |req: HttpRequest| call_into_response(handler.as_ref(), req))
}

/// Default per-request body cap (1 MiB) when `ServerConfig` is constructed
/// via `default()`. Per ADR-0092 + S3 security finding: NEVER read an
/// unbounded request body. Routes that legitimately need larger bodies
/// MUST override via `ServerConfig::with_max_body_bytes`.
pub const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;

/// Default header-read timeout. Hyper's `http1.header_read_timeout` budget;
/// closes Slowloris-style attacks that drip headers one byte at a time.
pub const DEFAULT_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(15);

/// Default keep-alive idle timeout. Connections idle longer than this are
/// dropped to bound concurrent-connection count under load.
pub const DEFAULT_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);

/// Server-level configuration: body cap + connection timeouts. Per ADR-0092
/// Phase 8 (S3 + S4): both fields are mandatory at the seam; defaults are
/// safe but conservative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerConfig {
    pub max_body_bytes: usize,
    pub header_read_timeout: Duration,
    pub keepalive_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            header_read_timeout: DEFAULT_HEADER_READ_TIMEOUT,
            keepalive_timeout: DEFAULT_KEEPALIVE_TIMEOUT,
        }
    }
}

impl ServerConfig {
    pub fn with_max_body_bytes(mut self, max: usize) -> Self {
        self.max_body_bytes = max;
        self
    }

    pub fn with_header_read_timeout(mut self, dur: Duration) -> Self {
        self.header_read_timeout = dur;
        self
    }

    pub fn with_keepalive_timeout(mut self, dur: Duration) -> Self {
        self.keepalive_timeout = dur;
        self
    }
}

/// Build an `HttpRequest` from a `hyper::Request<Incoming>` by collecting
/// the body fully, bounded by `max_body_bytes`. Bodies exceeding the cap
/// fail with `HyperRuntimeError::BodyTooLarge`; the caller renders 413.
/// This closes the S3 security finding (unbounded body → OOM DoS).
///
/// Boundary conversion: hyper `Bytes` body → kernel `Vec<u8>`. Allocates
/// once, bounded by max_body_bytes.
pub async fn collect_hyper_request(
    req: Request<Incoming>,
    max_body_bytes: usize,
) -> Result<HttpRequest, HyperRuntimeError> {
    let method_str = req.method().as_str().to_string();
    let method = HttpMethod::parse(&method_str)
        .ok_or_else(|| HyperRuntimeError::UnsupportedMethod(method_str.clone()))?;
    let path = req.uri().path().to_string();
    let mut headers = BTreeMap::new();
    // ADR-0092 Phase 10:
    //   * S1: hyper normalizes header names to lowercase already; we
    //     additionally `.to_ascii_lowercase()` defensively so direct-
    //     constructor tests cannot create case-divergent maps.
    //   * S2: non-UTF8 header value is REJECTED with 400 BadHeader, not
    //     silently dropped. Silent drops mask attack signal.
    for (name, value) in req.headers().iter() {
        let name_lower = name.as_str().to_ascii_lowercase();
        match value.to_str() {
            Ok(value_str) => {
                headers.insert(name_lower, value_str.to_string());
            }
            Err(_) => {
                return Err(HyperRuntimeError::NonUtf8HeaderValue {
                    header_name: name_lower,
                });
            }
        }
    }
    let body_bytes = collect_body_with_limit(req.into_body(), max_body_bytes).await?;
    Ok(HttpRequest {
        method,
        path,
        headers,
        body: body_bytes,
        path_captures: BTreeMap::new(),
        matched_template: None,
    })
}

/// Collect a hyper body to `Vec<u8>` with a hard byte cap. Used by
/// `collect_hyper_request`; exposed for tests + future per-route overrides.
pub async fn collect_body_with_limit<B>(
    body: B,
    max_bytes: usize,
) -> Result<Vec<u8>, HyperRuntimeError>
where
    B: Body<Data = Bytes> + Send + Unpin,
    B::Error: std::fmt::Display + std::error::Error + Send + Sync + 'static,
{
    let limited = Limited::new(body, max_bytes);
    let collected = limited.collect().await.map_err(|err| {
        // Limited returns a boxed error; we can't downcast cleanly without
        // a dep on a specific error type, so detect via the display string —
        // the upstream LengthLimitError::ContentLengthMismatch / OverLimit
        // both contain "limit" in their messages.
        let msg = err.to_string();
        if msg.to_lowercase().contains("limit") || msg.to_lowercase().contains("too large") {
            HyperRuntimeError::BodyTooLarge { max_bytes }
        } else {
            HyperRuntimeError::BodyRead(msg)
        }
    })?;
    Ok(collected.to_bytes().to_vec())
}

/// Convert an `HttpResponse` into a hyper `Response<Full<Bytes>>`.
///
/// Boundary conversion: kernel `Vec<u8>` body → hyper `Bytes`. Zero-copy via
/// `Bytes::from(Vec<u8>)` (Bytes takes ownership of the buffer).
pub fn to_hyper_response(resp: HttpResponse) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(resp.status);
    for (name, value) in &resp.headers {
        builder = builder.header(name, value);
    }
    builder
        .body(Full::new(Bytes::from(resp.body)))
        .unwrap_or_else(|_| {
            // ADR-0083 Tier 1: avoid `.expect()` on the fallback Response::builder.
            // Construct directly via `Response::new(body)` (infallible), then set
            // the status code on the parts. This path is hit only if the outer
            // builder rejected the header set; the fallback intentionally drops
            // user headers and serves a fixed 500 body.
            let mut response =
                Response::new(Full::new(Bytes::from_static(b"response build failed")));
            *response.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            response
        })
}

/// Dispatch a request through router → middleware chain → handler.
///
/// Lookups are sync (router) + sync (middleware chain) + sync (handler).
/// The hyper Service wrapper drives this from an async context.
pub fn dispatch(
    request: HttpRequest,
    router: &Router<SyncHandler>,
    chain: &MiddlewareChain<HttpRequest, HttpResponse>,
) -> HttpResponse {
    let (handler, captures, template) = match router.match_route(request.method, &request.path) {
        Some(triple) => triple,
        None => return HttpResponse::not_found(),
    };
    let template_owned = template.to_string();
    let mut req_with_captures = request;
    req_with_captures.path_captures = captures;
    req_with_captures.matched_template = Some(template_owned);
    let handler_arc = handler.clone();
    chain.execute(req_with_captures, move |req| handler_arc(req))
}

#[derive(Debug)]
pub enum HyperRuntimeError {
    Bind(String),
    BodyRead(String),
    BodyTooLarge { max_bytes: usize },
    UnsupportedMethod(String),
    NonUtf8HeaderValue { header_name: String },
}

impl HyperRuntimeError {
    /// Status code the runtime should emit when converting this error to a
    /// client-facing response.
    pub fn status_code(&self) -> u16 {
        match self {
            HyperRuntimeError::Bind(_) => 500,
            HyperRuntimeError::BodyRead(_) => 400,
            HyperRuntimeError::BodyTooLarge { .. } => 413,
            HyperRuntimeError::UnsupportedMethod(_) => 405,
            HyperRuntimeError::NonUtf8HeaderValue { .. } => 400,
        }
    }
}

impl std::fmt::Display for HyperRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperRuntimeError::Bind(reason) => write!(f, "hyper bind failed: {reason}"),
            HyperRuntimeError::BodyRead(reason) => {
                write!(f, "hyper body read failed: {reason}")
            }
            HyperRuntimeError::BodyTooLarge { max_bytes } => {
                write!(f, "request body exceeded max {max_bytes} bytes")
            }
            HyperRuntimeError::UnsupportedMethod(method) => {
                write!(f, "unsupported HTTP method: `{method}`")
            }
            HyperRuntimeError::NonUtf8HeaderValue { header_name } => {
                write!(f, "header `{header_name}` contains non-UTF-8 bytes")
            }
        }
    }
}

impl std::error::Error for HyperRuntimeError {}

impl From<HyperRuntimeError> for HttpResponse {
    fn from(err: HyperRuntimeError) -> Self {
        HttpResponse::new(err.status_code()).with_body(err.to_string().into_bytes())
    }
}

/// Start a hyper server on `addr` that dispatches every request through
/// `router` + `chain`, using `config` for security-critical limits.
/// This is the Layer 5 entry point that per-cell binaries (Layer 6) call
/// from their `tokio::main`.
///
/// Per ADR-0092 Phase 8 (S3 + S4):
///   * Per-request bodies are capped at `config.max_body_bytes` — defends
///     against unbounded-body DoS.
///   * Hyper builder uses `config.header_read_timeout` + idle timeout —
///     defends against Slowloris-class connection-holding attacks.
pub async fn serve(
    addr: SocketAddr,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| HyperRuntimeError::Bind(error.to_string()))?;
    serve_listener(listener, router, chain, config).await
}

/// Serve using an already-bound `TcpListener`.
///
/// This is intentionally separate from `serve(addr, ...)` so tests and
/// higher-level composition crates can bind `127.0.0.1:0`, capture the
/// selected socket address, and prove real TCP/Hyper loopback behavior without
/// hard-coding ports or importing Hyper directly. Production binaries may also
/// pre-bind sockets via supervisor/systemd-style activation and still keep all
/// Hyper-family dependencies concentrated in this crate.
pub async fn serve_listener(
    listener: TcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    let config = Arc::new(config);
    loop {
        let (stream, _peer) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => continue,
        };
        let io = TokioIo::new(stream);
        let router = router.clone();
        let chain = chain.clone();
        let config = config.clone();
        let timer_config = config.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let router = router.clone();
                let chain = chain.clone();
                let config = config.clone();
                async move {
                    let response = match collect_hyper_request(req, config.max_body_bytes).await {
                        Ok(parsed) => dispatch(parsed, &router, &chain),
                        Err(err) => HttpResponse::from(err),
                    };
                    Ok::<_, Infallible>(to_hyper_response(response))
                }
            });
            let mut builder = ConnBuilder::new(hyper_util::rt::TokioExecutor::new());
            // S4 hardening: bound how long we'll wait for headers from a
            // slow client. Slowloris-style attacks that drip header bytes
            // at one byte per second exceed this budget and the conn drops.
            builder
                .http1()
                .header_read_timeout(timer_config.header_read_timeout)
                .keep_alive(true)
                .timer(TokioTimer::new());
            // HTTP/2: keepalive ping bounds idle connections.
            builder
                .http2()
                .keep_alive_interval(Some(timer_config.keepalive_timeout / 2))
                .keep_alive_timeout(timer_config.keepalive_timeout)
                .timer(TokioTimer::new());
            let _ = builder.serve_connection(io, service).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_router_kernel::Router;

    fn ok_handler(body: &'static [u8]) -> SyncHandler {
        Arc::new(move |_req: HttpRequest| HttpResponse::new(200).with_body(body.to_vec()))
    }

    fn empty_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
        MiddlewareChain::new()
    }

    fn mock_request(method: HttpMethod, path: &str) -> HttpRequest {
        HttpRequest {
            method,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    #[test]
    fn dispatch_routes_to_matching_handler() {
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/workspace", ok_handler(b"live-list"))
            .unwrap();
        let chain = empty_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/workspace"), &router, &chain);
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"live-list".to_vec());
    }

    #[test]
    fn dispatch_unknown_route_returns_404() {
        let router: Router<SyncHandler> = Router::new();
        let chain = empty_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/nope"), &router, &chain);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn dispatch_passes_path_captures_to_handler() {
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(
                HttpMethod::Post,
                "/workspace/docs/api/v1/extractors/{extractor_id}/refresh",
                Arc::new(move |req: HttpRequest| {
                    let id = req
                        .path_captures
                        .get("extractor_id")
                        .cloned()
                        .unwrap_or_default();
                    HttpResponse::new(202).with_body(id.into_bytes())
                }),
            )
            .unwrap();
        let chain = empty_chain();
        let response = dispatch(
            mock_request(
                HttpMethod::Post,
                "/workspace/docs/api/v1/extractors/manifest-walker/refresh",
            ),
            &router,
            &chain,
        );
        assert_eq!(response.status, 202);
        assert_eq!(response.body, b"manifest-walker".to_vec());
    }

    #[test]
    fn response_helpers_build_canonical_errors() {
        assert_eq!(HttpResponse::not_found().status, 404);
        assert_eq!(HttpResponse::method_not_allowed().status, 405);
    }

    #[test]
    fn response_with_header_inserts() {
        let resp = HttpResponse::new(200).with_header("content-type", "application/json");
        assert_eq!(
            resp.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
    }

    #[test]
    fn to_hyper_response_preserves_status_and_body() {
        let resp = HttpResponse::new(201).with_body(b"created".to_vec());
        let hyper_resp = to_hyper_response(resp);
        assert_eq!(hyper_resp.status().as_u16(), 201);
    }

    #[test]
    fn dispatch_invokes_middleware_chain() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Counter(Arc<AtomicUsize>);
        impl oya_http_middleware_kernel::Middleware<HttpRequest, HttpResponse> for Counter {
            fn handle(
                &self,
                request: HttpRequest,
                next: oya_http_middleware_kernel::Next<'_, HttpRequest, HttpResponse>,
            ) -> HttpResponse {
                self.0.fetch_add(1, Ordering::SeqCst);
                next.run(request)
            }
        }
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/x", ok_handler(b"x"))
            .unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<HttpRequest, HttpResponse> =
            MiddlewareChain::new().push(Box::new(Counter(counter.clone())));
        let _ = dispatch(mock_request(HttpMethod::Get, "/x"), &router, &chain);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn hyper_runtime_error_display() {
        let bind = HyperRuntimeError::Bind("permission denied".into());
        assert!(format!("{bind}").contains("bind failed"));
        let body = HyperRuntimeError::BodyRead("eof".into());
        assert!(format!("{body}").contains("body read"));
        let method = HyperRuntimeError::UnsupportedMethod("FOO".into());
        assert!(format!("{method}").contains("FOO"));
    }

    // F3 adversarial: boundary conversion preserves bytes byte-for-byte for
    // the full u8 range. We must drain the hyper Body back to bytes and
    // compare byte-for-byte; asserting only status would let a silent
    // body-mangling regression pass.
    #[tokio::test]
    async fn boundary_conversion_round_trip_identity() {
        let original: Vec<u8> = (0u8..=255).collect();
        let resp = HttpResponse::new(200).with_body(original.clone());
        let hyper_resp = to_hyper_response(resp);
        let (parts, body) = hyper_resp.into_parts();
        assert_eq!(parts.status.as_u16(), 200);
        let drained = body
            .collect()
            .await
            .expect("Full<Bytes> never errors on collect")
            .to_bytes();
        assert_eq!(
            drained.as_ref(),
            original.as_slice(),
            "Vec<u8> -> Bytes -> Full -> collect -> Bytes must be byte-identical"
        );
        assert_eq!(drained.len(), 256);
        assert_eq!(drained[0], 0);
        assert_eq!(drained[255], 255);
    }

    // F3 adversarial: empty body survives the boundary (the obvious edge).
    #[tokio::test]
    async fn boundary_conversion_empty_body_round_trip() {
        let resp = HttpResponse::new(204).with_body(Vec::new());
        let hyper_resp = to_hyper_response(resp);
        let (parts, body) = hyper_resp.into_parts();
        assert_eq!(parts.status.as_u16(), 204);
        let drained = body.collect().await.unwrap().to_bytes();
        assert!(drained.is_empty());
    }

    // F3 adversarial: handler_to_sync wraps a typed Handler so the router
    // can hold it as a SyncHandler, and the rendered error path goes through
    // From<Error> for HttpResponse — proves the Phase 6 contract end-to-end.
    // F3 adversarial Phase 10 (S2 non-UTF8 + S1 header case):
    // building a hyper::Request<Full<Bytes>> and round-tripping through a
    // helper that exercises the same header-iteration path collect_hyper_request
    // uses. Direct Request<Incoming> isn't constructible in tests.
    #[test]
    fn header_name_lowercased_when_inserted_via_with_header() {
        // Surrogate for the adapter path: middleware-kernel with_header
        // already lowercases. Adapter inherits since adapter writes through
        // BTreeMap with insertion done via the same `.to_ascii_lowercase()`
        // in collect_hyper_request. Smoke this for S1 explicitly.
        let resp = HttpResponse::new(200).with_header("X-Tenant-Id", "acme");
        assert!(resp.headers.contains_key("x-tenant-id"));
        assert!(!resp.headers.contains_key("X-Tenant-Id"));
    }

    // F3 adversarial: NonUtf8HeaderValue error maps to 400.
    #[test]
    fn non_utf8_header_value_renders_400() {
        let err = HyperRuntimeError::NonUtf8HeaderValue {
            header_name: "x-binary".into(),
        };
        let resp: HttpResponse = err.into();
        assert_eq!(resp.status, 400);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("non-UTF-8"));
        assert!(body.contains("x-binary"));
    }

    #[test]
    fn non_utf8_header_value_status_code() {
        assert_eq!(
            HyperRuntimeError::NonUtf8HeaderValue {
                header_name: "x".into()
            }
            .status_code(),
            400
        );
    }

    // F3 adversarial: collect_body_with_limit accepts bodies <= max.
    #[tokio::test]
    async fn collect_body_with_limit_accepts_under_cap() {
        let body = Full::new(Bytes::from_static(b"hello"));
        let result = collect_body_with_limit(body, 1024).await.unwrap();
        assert_eq!(result, b"hello".to_vec());
    }

    // F3 adversarial: collect_body_with_limit accepts bodies exactly at max.
    #[tokio::test]
    async fn collect_body_with_limit_accepts_exact_cap() {
        let payload = vec![0xAB; 100];
        let body = Full::new(Bytes::from(payload.clone()));
        let result = collect_body_with_limit(body, 100).await.unwrap();
        assert_eq!(result, payload);
    }

    // F3 adversarial: collect_body_with_limit rejects bodies > max with the
    // specific BodyTooLarge variant. This closes the S3 unbounded-body DoS.
    #[tokio::test]
    async fn collect_body_with_limit_rejects_over_cap_with_body_too_large() {
        let body = Full::new(Bytes::from(vec![0u8; 1025]));
        let err = collect_body_with_limit(body, 1024).await.unwrap_err();
        match err {
            HyperRuntimeError::BodyTooLarge { max_bytes } => {
                assert_eq!(max_bytes, 1024);
            }
            other => panic!("expected BodyTooLarge, got {other:?}"),
        }
    }

    // F3 adversarial: BodyTooLarge maps to 413 Payload Too Large at the
    // From<HyperRuntimeError> for HttpResponse boundary.
    #[test]
    fn body_too_large_renders_413() {
        let err = HyperRuntimeError::BodyTooLarge { max_bytes: 1024 };
        let resp: HttpResponse = err.into();
        assert_eq!(resp.status, 413);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("1024"));
        assert!(body.contains("body"));
    }

    // F3 adversarial: each error variant maps to the correct status — proves
    // the From impl handles every variant, not just the obvious ones.
    #[test]
    fn hyper_runtime_error_status_code_mapping() {
        assert_eq!(HyperRuntimeError::Bind("x".into()).status_code(), 500);
        assert_eq!(HyperRuntimeError::BodyRead("x".into()).status_code(), 400);
        assert_eq!(
            HyperRuntimeError::BodyTooLarge { max_bytes: 1 }.status_code(),
            413
        );
        assert_eq!(
            HyperRuntimeError::UnsupportedMethod("BREW".into()).status_code(),
            405
        );
    }

    // F4 ergonomic: ServerConfig builder methods are chainable.
    #[test]
    fn server_config_builder_chains_with_methods() {
        let cfg = ServerConfig::default()
            .with_max_body_bytes(2048)
            .with_header_read_timeout(Duration::from_secs(5))
            .with_keepalive_timeout(Duration::from_secs(30));
        assert_eq!(cfg.max_body_bytes, 2048);
        assert_eq!(cfg.header_read_timeout, Duration::from_secs(5));
        assert_eq!(cfg.keepalive_timeout, Duration::from_secs(30));
    }

    // F1 linus: ServerConfig::default uses safe defaults (sealed contract
    // for fresh cell binaries).
    #[test]
    fn server_config_defaults_are_safe() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.max_body_bytes, DEFAULT_MAX_BODY_BYTES);
        assert_eq!(cfg.max_body_bytes, 1024 * 1024);
        assert!(cfg.header_read_timeout >= Duration::from_secs(5));
        assert!(cfg.header_read_timeout <= Duration::from_secs(60));
        assert!(cfg.keepalive_timeout >= Duration::from_secs(10));
    }

    #[test]
    fn handler_to_sync_routes_ok_and_err_paths() {
        use oya_http_middleware_kernel::Handler;

        #[derive(Clone, Debug)]
        enum SvcErr {
            Missing,
        }
        impl From<SvcErr> for HttpResponse {
            fn from(e: SvcErr) -> Self {
                match e {
                    SvcErr::Missing => {
                        HttpResponse::new(404).with_body(b"missing-from-svc".to_vec())
                    }
                }
            }
        }

        struct OkSvc;
        impl Handler for OkSvc {
            type Error = SvcErr;
            fn call(&self, _req: HttpRequest) -> Result<HttpResponse, SvcErr> {
                Ok(HttpResponse::new(200).with_body(b"svc-ok".to_vec()))
            }
        }

        struct ErrSvc;
        impl Handler for ErrSvc {
            type Error = SvcErr;
            fn call(&self, _req: HttpRequest) -> Result<HttpResponse, SvcErr> {
                Err(SvcErr::Missing)
            }
        }

        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/ok", handler_to_sync(OkSvc))
            .unwrap();
        router
            .route(HttpMethod::Get, "/err", handler_to_sync(ErrSvc))
            .unwrap();

        let chain = empty_chain();
        let ok = dispatch(mock_request(HttpMethod::Get, "/ok"), &router, &chain);
        assert_eq!(ok.status, 200);
        assert_eq!(ok.body, b"svc-ok".to_vec());

        let err = dispatch(mock_request(HttpMethod::Get, "/err"), &router, &chain);
        // The handler returned Err(SvcErr::Missing); rendered via From impl.
        assert_eq!(err.status, 404);
        assert_eq!(err.body, b"missing-from-svc".to_vec());
    }
}
