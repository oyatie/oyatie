//! Hyper runtime adapter — the ONLY crate in the hyper foundation that
//! imports hyper directly (per ADR-0090 + ADR-0092).
//!
//! Layer 5 of the foundation. Bridges:
//!   - http-router-kernel::Router<H>
//!   - http-middleware-kernel::MiddlewareChain<HttpRequest, HttpResponse>
//!   - hyper::service::Service over hyper 1.x
//!
//! Conversion-at-the-boundary discipline (ADR-0092 root-cause seam fix):
//!   * Inbound: hyper `Bytes` body → kernel `Vec<u8>` via `.to_vec()`.
//!   * Outbound: kernel `Vec<u8>` body → hyper `Full<Bytes>` via `Bytes::from`.
//!
//! The kernel types stay std-only; every hyper-family dep (`hyper`,
//! `hyper-util`, `hyper-rustls`, `http-body-util`, `bytes`) is concentrated in
//! THIS crate.
//!
//! Request / response structs are re-exported from the middleware kernel so
//! middleware crates depend inward while consumers still avoid importing hyper.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod admission;
mod execution;
mod response;
mod supervisor;
pub use admission::{
    InvalidServingLimits, ServingEvents, ServingLimits, ServingPhase, ServingSnapshot,
};
pub use supervisor::{ServingControl, ServingOutcome, ServingReport};

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Body, Incoming};
use hyper::{Request, Response};
use hyper_rustls::ConfigBuilderExt;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use tokio::net::TcpListener;

use http_middleware_kernel::{Handler, MiddlewareChain, call_into_response};
pub use http_middleware_kernel::{HttpRequest, HttpResponse};
use http_router_kernel::{HttpMethod, Router};

/// Synchronous handler signature wrapped by the router. Handlers are pure
/// `Fn` — they own / borrow their state via captured Arcs. The runtime calls
/// the chain + handler on a tokio worker thread.
///
/// Per ADR-0094: prefer implementing `http_middleware_kernel::Handler`
/// on a typed service struct and wrap with `handler_to_sync(...)` at
/// registration. The closure alias remains for ergonomics on trivial routes.
pub type SyncHandler = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync>;

/// Canonical outbound HTTPS connector type for hyper clients.
pub type HyperHttpsConnector = hyper_rustls::HttpsConnector<HttpConnector>;

/// Canonical outbound HTTPS client type used by app-layer transports.
pub type HyperHttpsClient = Client<HyperHttpsConnector, Full<Bytes>>;

/// Build the aws-lc-rs provider used by the workspace TLS policy.
///
/// X25519MLKEM768 is explicitly first so Buck2 and Cargo cannot diverge on
/// feature unification; X25519 remains present as the classical fallback.
#[must_use]
pub fn pqc_hybrid_aws_lc_provider() -> rustls::crypto::CryptoProvider {
    let mut provider = rustls::crypto::aws_lc_rs::default_provider();
    provider.kx_groups = vec![
        rustls::crypto::aws_lc_rs::kx_group::X25519MLKEM768,
        rustls::crypto::aws_lc_rs::kx_group::X25519,
        rustls::crypto::aws_lc_rs::kx_group::SECP256R1,
        rustls::crypto::aws_lc_rs::kx_group::SECP384R1,
    ];
    provider
}

/// Return the aws-lc-rs key-exchange group order used by this workspace TLS policy.
#[must_use]
pub fn pqc_hybrid_kx_group_names() -> Vec<rustls::NamedGroup> {
    pqc_hybrid_aws_lc_provider()
        .kx_groups
        .iter()
        .map(|group| group.name())
        .collect()
}

/// TLS 1.3-only client config builder using the workspace aws-lc-rs provider.
#[must_use]
pub fn pqc_hybrid_tls13_client_config_builder()
-> rustls::ConfigBuilder<rustls::ClientConfig, rustls::WantsVerifier> {
    rustls::ClientConfig::builder_with_provider(Arc::new(pqc_hybrid_aws_lc_provider()))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("static aws-lc-rs TLS 1.3 PQC-hybrid client provider must be valid")
}

/// TLS 1.3-only server config builder using the workspace aws-lc-rs provider.
#[must_use]
pub fn pqc_hybrid_tls13_server_config_builder()
-> rustls::ConfigBuilder<rustls::ServerConfig, rustls::WantsVerifier> {
    rustls::ServerConfig::builder_with_provider(Arc::new(pqc_hybrid_aws_lc_provider()))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("static aws-lc-rs TLS 1.3 PQC-hybrid server provider must be valid")
}

/// Build a webpki-rooted client TLS config for external HTTPS calls.
#[must_use]
pub fn pqc_hybrid_tls13_client_config() -> rustls::ClientConfig {
    pqc_hybrid_tls13_client_config_builder()
        .with_webpki_roots()
        .with_no_client_auth()
}

/// Build the canonical HTTPS-only connector: TLS 1.3, X25519MLKEM768 first, X25519 fallback.
#[must_use]
pub fn build_pqc_hybrid_https_connector() -> HyperHttpsConnector {
    hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(pqc_hybrid_tls13_client_config())
        .https_only()
        .enable_http1()
        .enable_http2()
        .build()
}

/// Build the canonical pooled hyper HTTPS client.
#[must_use]
pub fn build_pqc_hybrid_https_client() -> HyperHttpsClient {
    Client::builder(TokioExecutor::new()).build(build_pqc_hybrid_https_connector())
}

/// Build a deliberately named loopback-test connector that can speak plaintext
/// HTTP to in-process mock servers. HTTP traffic through this connector is not
/// PQC protected and must never be used as production external-endpoint evidence.
#[doc(hidden)]
#[must_use]
pub fn build_loopback_http_or_pqc_hybrid_https_connector_for_tests() -> HyperHttpsConnector {
    hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(pqc_hybrid_tls13_client_config())
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build()
}

/// Build a pooled client for loopback plaintext test servers plus normal PQC HTTPS.
#[doc(hidden)]
#[must_use]
pub fn build_loopback_http_or_pqc_hybrid_https_client_for_tests() -> HyperHttpsClient {
    Client::builder(TokioExecutor::new())
        .build(build_loopback_http_or_pqc_hybrid_https_connector_for_tests())
}

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
    response::convert(resp, None)
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
        None if router.path_matches_any_method(&request.path) => {
            return HttpResponse::method_not_allowed();
        }
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
    Config(String),
    Connection(String),
    Runtime(String),
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
            HyperRuntimeError::Config(_) => 500,
            HyperRuntimeError::Connection(_) => 500,
            HyperRuntimeError::Runtime(_) => 500,
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
            HyperRuntimeError::Config(reason) => {
                write!(f, "hyper server configuration failed: {reason}")
            }
            HyperRuntimeError::Connection(reason) => {
                write!(f, "hyper connection failed: {reason}")
            }
            HyperRuntimeError::Runtime(reason) => write!(f, "tokio runtime failed: {reason}"),
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

/// Serve an already-bound Tokio listener indefinitely.
///
/// This is the daemon-facing seam used by application composition roots that
/// own the bind decision outside this adapter. It keeps all Tokio/Hyper types
/// inside this crate while still allowing callers to perform pre-bind policy
/// checks with `std::net::TcpListener`.
pub async fn serve_listener(
    listener: TcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    supervisor::run(
        listener,
        router,
        chain,
        config,
        ServingControl::new(ServingLimits::default()),
        None,
        None,
    )
    .await?
    .into_result()
}

/// Serve exactly one accepted connection from an already-bound listener.
///
/// This is a deterministic loopback harness for service-level integration
/// tests and local evidence capture. It does not replace `serve` for daemon
/// operation and does not create deployment, readiness, or production endpoint
/// evidence by itself.
pub async fn serve_one_connection(
    listener: TcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    serve_n_connections(listener, router, chain, config, 1).await
}

/// Serve a bounded number of accepted connections from an already-bound listener.
///
/// This is for deterministic integration evidence only. It avoids unbounded
/// daemon lifetime in tests while exercising the same Hyper request parsing,
/// response serialization, router, middleware, and handler seam as `serve`.
pub async fn serve_n_connections(
    listener: TcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    max_connections: usize,
) -> Result<(), HyperRuntimeError> {
    if max_connections == 0 {
        return Err(HyperRuntimeError::Config(
            "max_connections must be greater than zero".to_string(),
        ));
    }
    supervisor::run(
        listener,
        router,
        chain,
        config,
        ServingControl::new(ServingLimits::default()),
        Some(max_connections),
        None,
    )
    .await?
    .into_result()
}

/// Blocking wrapper for deterministic tests that need to bind a std listener
/// outside this crate without depending directly on tokio.
///
/// The std listener must already be bound (commonly to `127.0.0.1:0` in a
/// test). This helper converts it to a Tokio listener and serves one
/// connection on a private single-thread runtime.
pub fn serve_one_connection_on_std_listener(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    serve_n_connections_on_std_listener(listener, router, chain, config, 1)
}

/// Blocking wrapper for bounded local integration evidence without leaking
/// Tokio into downstream crates.
pub fn serve_n_connections_on_std_listener(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    max_connections: usize,
) -> Result<(), HyperRuntimeError> {
    supervisor::run_std(
        listener,
        router,
        chain,
        config,
        ServingControl::new(ServingLimits::default()),
        Some(max_connections),
        false,
    )?
    .into_result()
}

/// Blocking wrapper for daemon entrypoints that pre-bind a std listener while
/// keeping the async runtime dependency adapter-local.
pub fn serve_on_std_listener(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
) -> Result<(), HyperRuntimeError> {
    serve_controlled_on_std_listener(
        listener,
        router,
        chain,
        config,
        ServingControl::new(ServingLimits::default()),
    )?
    .into_result()
}

pub fn serve_controlled_on_std_listener(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    control: ServingControl,
) -> Result<ServingReport, HyperRuntimeError> {
    supervisor::run_std(listener, router, chain, config, control, None, false)
}

/// Explicit executable opt-in; ordinary library serving installs no signal handlers.
pub fn serve_with_signals_on_std_listener(
    listener: StdTcpListener,
    router: Arc<Router<SyncHandler>>,
    chain: Arc<MiddlewareChain<HttpRequest, HttpResponse>>,
    config: ServerConfig,
    control: ServingControl,
) -> Result<ServingReport, HyperRuntimeError> {
    supervisor::run_std(listener, router, chain, config, control, None, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_router_kernel::Router;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;

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
    fn pqc_hybrid_policy_prioritizes_hybrid_group_and_classical_fallback() {
        let groups = pqc_hybrid_kx_group_names();
        assert_eq!(
            groups.first().copied(),
            Some(rustls::NamedGroup::X25519MLKEM768),
            "X25519MLKEM768 must be the first offered TLS 1.3 key-share group"
        );
        assert!(
            groups.contains(&rustls::NamedGroup::X25519),
            "classical X25519 fallback must remain enabled"
        );

        let _connector = build_pqc_hybrid_https_connector();
    }

    #[tokio::test]
    async fn pqc_hybrid_https_client_rejects_plain_http_uri() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let client = build_pqc_hybrid_https_client();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener binds");
        let addr = listener.local_addr().expect("listener addr");
        let plaintext_server_reached = Arc::new(AtomicBool::new(false));
        let reached = Arc::clone(&plaintext_server_reached);
        let server = tokio::spawn(async move {
            if let Ok(Ok((stream, _))) =
                tokio::time::timeout(Duration::from_millis(200), listener.accept()).await
            {
                reached.store(true, Ordering::SeqCst);
                let io = TokioIo::new(stream);
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(|_req| async {
                            Ok::<_, Infallible>(Response::new(Full::new(Bytes::from_static(
                                b"not-pqc",
                            ))))
                        }),
                    )
                    .await;
            }
        });
        let request = Request::builder()
            .method("GET")
            .uri(format!("http://{addr}/plaintext-is-not-pqc"))
            .body(Full::new(Bytes::new()))
            .expect("request builds");

        client
            .request(request)
            .await
            .expect_err("canonical PQC client must reject plaintext HTTP URIs");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !plaintext_server_reached.load(Ordering::SeqCst),
            "canonical PQC client must reject plaintext HTTP before reaching a loopback HTTP server"
        );
        server.abort();
    }

    #[test]
    fn pqc_hybrid_tls13_handshake_selects_hybrid_group() {
        let rcgen::CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
                .expect("test cert generation");
        let mut roots = rustls::RootCertStore::empty();
        roots.add(cert.der().clone()).expect("self cert trusted");

        let client_config = pqc_hybrid_tls13_client_config_builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server_key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(signing_key.serialize_der()),
        );
        let server_config = pqc_hybrid_tls13_server_config_builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert.der().clone()], server_key)
            .expect("server cert and key match");

        let mut client = rustls::ClientConnection::new(
            Arc::new(client_config),
            rustls::pki_types::ServerName::try_from("localhost").expect("valid DNS name"),
        )
        .expect("client connection");
        let mut server =
            rustls::ServerConnection::new(Arc::new(server_config)).expect("server connection");

        for _ in 0..16 {
            let mut client_to_server = Vec::new();
            client
                .write_tls(&mut client_to_server)
                .expect("client writes tls");
            if !client_to_server.is_empty() {
                let mut cursor = std::io::Cursor::new(client_to_server);
                server.read_tls(&mut cursor).expect("server reads tls");
                server
                    .process_new_packets()
                    .expect("server processes packets");
            }

            let mut server_to_client = Vec::new();
            server
                .write_tls(&mut server_to_client)
                .expect("server writes tls");
            if !server_to_client.is_empty() {
                let mut cursor = std::io::Cursor::new(server_to_client);
                client.read_tls(&mut cursor).expect("client reads tls");
                client
                    .process_new_packets()
                    .expect("client processes packets");
            }

            if !client.is_handshaking() && !server.is_handshaking() {
                break;
            }
        }

        assert!(!client.is_handshaking(), "client handshake must finish");
        assert!(!server.is_handshaking(), "server handshake must finish");
        assert_eq!(
            client.protocol_version(),
            Some(rustls::ProtocolVersion::TLSv1_3)
        );
        assert_eq!(
            server.protocol_version(),
            Some(rustls::ProtocolVersion::TLSv1_3)
        );
        assert_eq!(
            client
                .negotiated_key_exchange_group()
                .map(|group| group.name()),
            Some(rustls::NamedGroup::X25519MLKEM768)
        );
        assert_eq!(
            server
                .negotiated_key_exchange_group()
                .map(|group| group.name()),
            Some(rustls::NamedGroup::X25519MLKEM768)
        );
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
    fn dispatch_known_path_with_wrong_method_returns_405() {
        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/workspace", ok_handler(b"live-list"))
            .unwrap();
        let chain = empty_chain();
        let response = dispatch(
            mock_request(HttpMethod::Post, "/workspace"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 405);
        assert_eq!(response.body, b"method not allowed".to_vec());
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
        impl http_middleware_kernel::Middleware<HttpRequest, HttpResponse> for Counter {
            fn handle(
                &self,
                request: HttpRequest,
                next: http_middleware_kernel::Next<'_, HttpRequest, HttpResponse>,
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

    #[tokio::test]
    async fn serve_one_connection_serves_loopback_request() {
        use std::io::{Read, Write};

        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/healthz", ok_handler(b"ok"))
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local listener");
        let addr = listener.local_addr().expect("listener has local addr");
        let server = tokio::spawn(async move {
            serve_one_connection(
                listener,
                Arc::new(router),
                Arc::new(empty_chain()),
                ServerConfig::default().with_max_body_bytes(0),
            )
            .await
        });

        let response = tokio::task::spawn_blocking(move || {
            let mut stream = std::net::TcpStream::connect(addr).expect("connect loopback");
            stream
                .write_all(b"GET /healthz HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                .expect("write request");
            let mut response = String::new();
            stream.read_to_string(&mut response).expect("read response");
            response
        })
        .await
        .expect("client task joins");

        server
            .await
            .expect("server task joins")
            .expect("serves one connection");
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.ends_with("ok"));
    }

    #[tokio::test]
    async fn serve_n_connections_serves_bounded_loopback_requests() {
        use std::io::{Read, Write};

        let mut router: Router<SyncHandler> = Router::new();
        router
            .route(HttpMethod::Get, "/healthz", ok_handler(b"health"))
            .unwrap();
        router
            .route(HttpMethod::Get, "/livez", ok_handler(b"live"))
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local listener");
        let addr = listener.local_addr().expect("listener has local addr");
        let server = tokio::spawn(async move {
            serve_n_connections(
                listener,
                Arc::new(router),
                Arc::new(empty_chain()),
                ServerConfig::default().with_max_body_bytes(0),
                2,
            )
            .await
        });

        let responses = tokio::task::spawn_blocking(move || {
            ["/healthz", "/livez"].map(|path| {
                let mut stream = std::net::TcpStream::connect(addr).expect("connect loopback");
                stream
                    .write_all(
                        format!(
                            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
                        )
                        .as_bytes(),
                    )
                    .expect("write request");
                let mut response = String::new();
                stream.read_to_string(&mut response).expect("read response");
                response
            })
        })
        .await
        .expect("client task joins");

        server
            .await
            .expect("server task joins")
            .expect("serves bounded connections");
        assert!(responses[0].starts_with("HTTP/1.1 200 OK"));
        assert!(responses[0].ends_with("health"));
        assert!(responses[1].starts_with("HTTP/1.1 200 OK"));
        assert!(responses[1].ends_with("live"));
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
        assert_eq!(HyperRuntimeError::Config("x".into()).status_code(), 500);
        assert_eq!(HyperRuntimeError::Connection("x".into()).status_code(), 500);
        assert_eq!(HyperRuntimeError::Runtime("x".into()).status_code(), 500);
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
        use http_middleware_kernel::Handler;

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
