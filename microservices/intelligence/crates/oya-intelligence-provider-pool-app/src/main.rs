//! `oya-intelligence-provider-pool` binary entry point — the composition root
//! the pooling lineage lacked (pooling-convergence campaign slice 1,
//! `.omc/pooling-convergence.json`).
//!
//! Composes the bespoke hyper backbone
//! (`oya_http_runtime_hyper_adapter::{ServerConfig, serve}` + the
//! `oya-http-router-kernel` `Router` + `oya-http-middleware-kernel`
//! `MiddlewareChain`) with the anthropic/openai compat-api ingress route
//! surfaces into a runnable process, wiring real handlers through to the
//! existing [`dispatch_to_pool`] use-case over the in-memory reference adapters
//! ([`InMemoryPoolRepository`] / [`InMemoryUsageSnapshotSource`] /
//! [`InMemoryAccountHealthStore`]) and the in-memory mock transport.
//!
//! Lifts the `AppConfig::from_env -> build_app -> serve` shape from
//! `microservices/cloud-intelligence/crates/oya-cloud-intelligence-app/main.rs`
//! (which serves an axum router — this binary serves the doctrine-compliant
//! bespoke hyper backbone instead, no axum, no reqwest).
//!
//! ## Scope of THIS increment
//!
//! The upstream transport is the in-memory **mock** transport: `/v1/messages`
//! and `/v1/chat/completions` route ingress -> kernel routing decision ->
//! dispatch -> health-record over the in-memory adapters and return a mocked
//! provider response. The real `hyper-util` legacy-client + `hyper-rustls`
//! transport is a LATER campaign slice (the workspace dep-seam for it lands in
//! the same PR as this binary, but is not yet consumed here). The SSE relay
//! path is stubbed (streaming requests get a non-streaming mocked body).
//!
//! ## Start-up posture (ADR-0083 Tier 3 — panic-free)
//!
//! `from_env` + `build_app` are fallible and surfaced as a non-zero exit code
//! with a structured log line. There is no `unwrap`/`expect`/`panic` on the
//! start-up path. A misconfigured environment fails closed at bind time.

// ADR-0083 Tier 3: production stays panic-free; tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use oya_http_router_kernel::{HttpMethod, Router};
use oya_http_runtime_hyper_adapter::{ServerConfig, SyncHandler, serve};

use oya_intelligence_provider_pool_app::{
    DispatchError, InMemoryAccountHealthStore, InMemoryPoolRepository,
    InMemoryProviderInvocationTransport, InMemorySecretResolver, InMemoryUsageSnapshotSource,
    NoOpMetricsSink, PoolId, PoolRepository, PoolRoutingStrategy, ProviderAccountId,
    ProviderAccountPool, ProviderFamily, ProviderResponse, ProviderTier, RequestMetadata, TenantId,
    TransportError, TransportScript, UnixMillis, dispatch_to_pool,
};
use oya_intelligence_provider_pool_kernel::DurationMs;

// =====================================================================
// Config (env-driven, fail-closed)
// =====================================================================

/// Environment-driven configuration for the pooling composition root.
///
/// All fields have safe defaults except the ones that must be operator-set for
/// the in-memory bring-up pool to resolve. Reading is fallible — a malformed
/// value fails closed rather than silently defaulting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    /// TCP bind address (default `127.0.0.1:8089`).
    pub listen_addr: SocketAddr,
    /// Tenant id the single bring-up pool is scoped to.
    pub tenant_id: String,
    /// Pool id of the single bring-up pool.
    pub pool_id: String,
    /// Provider family for the bring-up pool.
    pub provider: ProviderFamily,
    /// Member account ids seeded into the bring-up pool.
    pub member_account_ids: Vec<String>,
    /// Per-request body cap for the hyper server.
    pub max_body_bytes: usize,
}

/// Failure reading [`AppConfig`] from the environment. Fail-closed: every
/// variant maps to a non-zero process exit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// `OYA_POOL_LISTEN_ADDR` did not parse as a `SocketAddr`.
    InvalidListenAddr { value: String },
    /// An unknown provider family was supplied.
    InvalidProvider { value: String },
    /// `OYA_POOL_MAX_BODY_BYTES` did not parse as a `usize`.
    InvalidMaxBodyBytes { value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr { value } => {
                write!(f, "OYA_POOL_LISTEN_ADDR is not a valid socket address: {value}")
            }
            Self::InvalidProvider { value } => write!(
                f,
                "OYA_POOL_PROVIDER must be one of claude|openai|gemini, got: {value}"
            ),
            Self::InvalidMaxBodyBytes { value } => {
                write!(f, "OYA_POOL_MAX_BODY_BYTES is not a valid usize: {value}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Default bind address used when `OYA_POOL_LISTEN_ADDR` is unset.
    pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:8089";

    /// Read configuration from the process environment, failing closed on any
    /// malformed value.
    ///
    /// Environment variables:
    /// - `OYA_POOL_LISTEN_ADDR`    — bind address (default `127.0.0.1:8089`)
    /// - `OYA_POOL_TENANT_ID`      — tenant id (default `ten_local`)
    /// - `OYA_POOL_POOL_ID`        — pool id (default `pool_local`)
    /// - `OYA_POOL_PROVIDER`       — `claude|openai|gemini` (default `claude`)
    /// - `OYA_POOL_MEMBER_IDS`     — comma-separated account ids (default `seat-local-1`)
    /// - `OYA_POOL_MAX_BODY_BYTES` — per-request body cap (default 1 MiB)
    ///
    /// # Errors
    /// Returns [`ConfigError`] when a supplied value is malformed.
    pub fn from_env() -> Result<Self, ConfigError> {
        let listen_addr_raw = std::env::var("OYA_POOL_LISTEN_ADDR")
            .unwrap_or_else(|_| Self::DEFAULT_LISTEN_ADDR.to_string());
        let listen_addr = listen_addr_raw
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::InvalidListenAddr {
                value: listen_addr_raw.clone(),
            })?;

        let tenant_id = std::env::var("OYA_POOL_TENANT_ID").unwrap_or_else(|_| "ten_local".into());
        let pool_id = std::env::var("OYA_POOL_POOL_ID").unwrap_or_else(|_| "pool_local".into());

        let provider_raw =
            std::env::var("OYA_POOL_PROVIDER").unwrap_or_else(|_| "claude".to_string());
        let provider = parse_provider(&provider_raw).ok_or(ConfigError::InvalidProvider {
            value: provider_raw.clone(),
        })?;

        let member_account_ids = std::env::var("OYA_POOL_MEMBER_IDS")
            .unwrap_or_else(|_| "seat-local-1".into())
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();

        let max_body_bytes = match std::env::var("OYA_POOL_MAX_BODY_BYTES") {
            Ok(raw) => raw
                .parse::<usize>()
                .map_err(|_| ConfigError::InvalidMaxBodyBytes { value: raw.clone() })?,
            Err(_) => oya_http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES,
        };

        Ok(Self {
            listen_addr,
            tenant_id,
            pool_id,
            provider,
            member_account_ids,
            max_body_bytes,
        })
    }
}

fn parse_provider(raw: &str) -> Option<ProviderFamily> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "claude" | "anthropic" => Some(ProviderFamily::Claude),
        "openai" | "codex" | "openaiorcodex" => Some(ProviderFamily::OpenAiOrCodex),
        "gemini" => Some(ProviderFamily::Gemini),
        _ => None,
    }
}

// =====================================================================
// App state + composition (build_app)
// =====================================================================

/// Shared, process-lifetime state mounted behind the route handlers.
///
/// The health store is the only mutable port, so it is guarded by a `Mutex`
/// (the in-memory reference store is cheap to lock; production swaps in a
/// sharded async store behind the same port). The repository + usage source +
/// transport are immutable snapshots / scripts and held in `Arc`.
struct AppState {
    pool_repo: InMemoryPoolRepository,
    usage_source: InMemoryUsageSnapshotSource,
    health_store: Mutex<InMemoryAccountHealthStore>,
    transport: InMemoryProviderInvocationTransport,
    /// Secret resolver port. For slice 1 no `SecretReference` is passed to the
    /// dispatch (`secret_ref_opt = None`), so this resolver is never invoked;
    /// the live OpenBao resolver lands in a later campaign slice.
    secret_res: InMemorySecretResolver,
    /// Metrics sink port — no-op for single-node bring-up; the OTel sink lands
    /// in the seat-observability slice.
    metrics: NoOpMetricsSink,
    tenant_id: TenantId,
    pool_id: PoolId,
}

impl AppState {
    /// Drive one dispatch through the kernel + in-memory adapters + mock
    /// transport, returning the verbatim mocked provider response or a typed
    /// dispatch error. Default-deny: never panics.
    fn dispatch(&self, model: &str, body: Bytes) -> Result<ProviderResponse, DispatchError> {
        let request = RequestMetadata::new(model.to_owned());
        let now = UnixMillis(now_unix_millis());
        // The health store is the single mutable port. A poisoned lock is
        // surfaced as a default-deny dispatch error rather than a panic.
        let mut health = self
            .health_store
            .lock()
            .map_err(|_| DispatchError::NonRetryableTransport(TransportError::NonRetryable {
                detail: "health store lock poisoned".into(),
            }))?;
        let fut = dispatch_to_pool(
            &self.pool_repo,
            &self.usage_source,
            &mut *health,
            &self.transport,
            &self.secret_res,
            &self.metrics,
            // slice 1: no per-dispatch SecretReference resolution yet — the
            // mock transport needs no credential. Live secret-fetch is a later
            // slice (Unimplemented::OpenBaoSecretResolution).
            None,
            &self.tenant_id,
            &self.pool_id,
            &request,
            now,
            body,
        );
        // The in-memory mock transport returns an immediately-ready future, so
        // a single-poll executor resolves it without yielding. This keeps the
        // sync router handler from needing a nested async runtime for THIS
        // increment; the real hyper-client transport slice moves dispatch onto
        // the async path proper.
        block_on_ready(fut).map(|outcome| outcome.response)
    }
}

/// The composed, ready-to-serve app: the route table, the middleware chain,
/// and the security-critical server config the hyper adapter needs.
struct ComposedApp {
    router: Router<SyncHandler>,
    chain: MiddlewareChain<HttpRequest, HttpResponse>,
    server_config: ServerConfig,
}

/// Build the composed router + middleware chain + server config from config.
///
/// Seeds a single in-memory pool from `config`, mounts the anthropic + openai
/// compat-api route surfaces plus `GET /healthz`, and wires every compat route
/// to a handler that drives [`AppState::dispatch`]. Fallible so a route-table
/// construction error (e.g. a duplicate template) fails closed at start-up.
///
/// # Errors
/// Returns [`BuildError`] when the route table cannot be assembled.
fn build_app(config: &AppConfig) -> Result<ComposedApp, BuildError> {
    let tenant_id = TenantId(config.tenant_id.clone());
    let pool_id = PoolId(config.pool_id.clone());

    let mut members: BTreeSet<ProviderAccountId> = BTreeSet::new();
    for id in &config.member_account_ids {
        members.insert(ProviderAccountId(id.clone()));
    }
    let pool = ProviderAccountPool::new(
        pool_id.clone(),
        config.provider,
        ProviderTier::Pro,
        tenant_id.clone(),
        members,
        PoolRoutingStrategy::RoundRobin,
        DurationMs(60_000),
    );
    let mut pool_repo = InMemoryPoolRepository::new();
    pool_repo
        .save(&pool)
        .map_err(|e| BuildError::Seed(e.to_string()))?;

    // Mock transport: echoes a 200 JSON envelope tagged with the chosen seat.
    // This is the in-memory bring-up path; the real upstream transport is a
    // later slice. Returns an immediately-ready response (no socket).
    let script: TransportScript = Arc::new(|account_id, provider, _body| {
        let body = format!(
            r#"{{"object":"pool.mock","provider":"{provider:?}","seat":"{}","note":"in-memory mock transport (pooling-convergence slice 1); real hyper-client transport is a later slice"}}"#,
            account_id.0
        );
        Ok(ProviderResponse {
            status: 200,
            headers: vec![("content-type".into(), "application/json".into())],
            body: Bytes::from(body),
            retry_after_seconds: None,
            provider_account_id: account_id.clone(),
        })
    });

    let state = Arc::new(AppState {
        pool_repo,
        usage_source: InMemoryUsageSnapshotSource::new(),
        health_store: Mutex::new(InMemoryAccountHealthStore::new()),
        transport: InMemoryProviderInvocationTransport::new(script),
        secret_res: InMemorySecretResolver::new(),
        metrics: NoOpMetricsSink,
        tenant_id,
        pool_id,
    });

    let mut router: Router<SyncHandler> = Router::new();

    // GET /healthz — liveness, no pool touch.
    router
        .route(
            HttpMethod::Get,
            "/healthz",
            Arc::new(|_req: HttpRequest| {
                HttpResponse::new(200)
                    .with_header("content-type", "application/json")
                    .with_body(br#"{"status":"ok"}"#.to_vec())
            }),
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?;

    // Anthropic-compat ingress: POST /v1/messages + GET /v1/messages/count_tokens.
    let messages_state = state.clone();
    let messages_handler: SyncHandler = Arc::new(move |req: HttpRequest| {
        dispatch_handler(&messages_state, &req)
    });
    let count_tokens_handler: SyncHandler = Arc::new(|req: HttpRequest| {
        // count_tokens is a pure local estimate; no pool dispatch.
        let estimate =
            oya_intelligence_adapter_anthropic_compat_api::count_tokens_handler(&utf8_lossy(
                &req.body,
            ));
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"input_tokens":{estimate}}}"#).into_bytes())
    });
    mount(
        &mut router,
        oya_intelligence_adapter_anthropic_compat_api::build_routes(
            messages_handler,
            count_tokens_handler,
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?,
    )?;

    // OpenAI-compat ingress: POST /v1/chat/completions + /v1/embeddings + GET /v1/models.
    let chat_state = state.clone();
    let chat_handler: SyncHandler =
        Arc::new(move |req: HttpRequest| dispatch_handler(&chat_state, &req));
    let embeddings_handler: SyncHandler = Arc::new(|_req: HttpRequest| {
        // Embeddings dispatch is out of scope for slice 1 (the mock transport
        // models a chat/messages completion). Surface an honest 501 rather than
        // a fake success.
        HttpResponse::new(501)
            .with_header("content-type", "application/json")
            .with_body(
                br#"{"error":"embeddings not wired in pooling-convergence slice 1"}"#.to_vec(),
            )
    });
    let models_handler: SyncHandler = Arc::new(|_req: HttpRequest| {
        HttpResponse::new(200)
            .with_header("content-type", "application/json")
            .with_body(br#"{"object":"list","data":[]}"#.to_vec())
    });
    mount(
        &mut router,
        oya_intelligence_adapter_openai_compat_api::build_routes(
            chat_handler,
            embeddings_handler,
            models_handler,
        )
        .map_err(|e| BuildError::Route(format!("{e:?}")))?,
    )?;

    let chain: MiddlewareChain<HttpRequest, HttpResponse> = MiddlewareChain::new();
    let server_config = ServerConfig::default().with_max_body_bytes(config.max_body_bytes);

    Ok(ComposedApp {
        router,
        chain,
        server_config,
    })
}

/// The shared request->dispatch->response handler body used by both the
/// anthropic `/v1/messages` and openai `/v1/chat/completions` routes. Extracts
/// the model hint from the JSON body (best-effort; the kernel routes on the
/// pool regardless) and drives [`AppState::dispatch`], mapping a
/// [`DispatchError`] to a fail-closed HTTP status.
fn dispatch_handler(state: &AppState, req: &HttpRequest) -> HttpResponse {
    let model = extract_model(&req.body).unwrap_or_else(|| "unknown".to_string());
    let body = Bytes::from(req.body.clone());
    match state.dispatch(&model, body) {
        Ok(resp) => {
            let mut http = HttpResponse::new(resp.status);
            for (name, value) in &resp.headers {
                http = http.with_header(name.clone(), value.clone());
            }
            http.with_body(resp.body.to_vec())
        }
        Err(err) => dispatch_error_to_response(&err),
    }
}

/// Map a typed [`DispatchError`] to a fail-closed HTTP response. Detail strings
/// are operator-facing and never echo credentials or prompts.
fn dispatch_error_to_response(err: &DispatchError) -> HttpResponse {
    let (status, kind) = match err {
        DispatchError::PoolNotFound { .. } => (404, "pool_not_found"),
        DispatchError::Repository(_) => (503, "repository_unavailable"),
        DispatchError::Routing(_) => (502, "routing_failed"),
        DispatchError::AllProvidersExhausted { .. } => (502, "all_providers_exhausted"),
        DispatchError::NonRetryableTransport(_) => (502, "transport_non_retryable"),
        DispatchError::SecretResolutionFailed(_) => (502, "secret_resolution_failed"),
    };
    let detail = json_escape(&err.to_string());
    HttpResponse::new(status)
        .with_header("content-type", "application/json")
        .with_body(format!(r#"{{"error":"{kind}","detail":"{detail}"}}"#).into_bytes())
}

/// Add every route of `source` into `target`. Surfaces a duplicate/parse error
/// as a fail-closed [`BuildError`].
fn mount(
    target: &mut Router<SyncHandler>,
    source: Router<SyncHandler>,
) -> Result<(), BuildError> {
    for (method, template) in source.routes() {
        // Re-resolve the handler from the source for this (method, template).
        let Some((handler, _captures, _t)) = source.match_route(method, template) else {
            // Unreachable: every (method, template) yielded by `routes()` is
            // registered. Fail closed instead of panicking if the invariant
            // is ever violated.
            return Err(BuildError::Route(format!(
                "route {method:?} {template} vanished during mount"
            )));
        };
        target
            .route(method, template, handler.clone())
            .map_err(|e| BuildError::Route(format!("{e:?}")))?;
    }
    Ok(())
}

/// Failure assembling the composed app. Fail-closed at start-up.
#[derive(Clone, Debug, Eq, PartialEq)]
enum BuildError {
    /// Seeding the in-memory pool repository failed.
    Seed(String),
    /// Assembling the route table failed (duplicate or malformed template).
    Route(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seed(d) => write!(f, "failed to seed in-memory pool: {d}"),
            Self::Route(d) => write!(f, "failed to build route table: {d}"),
        }
    }
}

impl std::error::Error for BuildError {}

// =====================================================================
// Small local helpers (no extra deps)
// =====================================================================

/// Current unix time in milliseconds. Falls back to 0 on a pre-epoch clock
/// (never panics).
fn now_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Lossy UTF-8 view of a byte body, for the count_tokens estimate.
fn utf8_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Best-effort extraction of a top-level `"model"` string field from a JSON
/// body without pulling a JSON parser into the hot path. Returns `None` if the
/// field is absent or malformed — the kernel routes on the pool regardless, so
/// the model hint is advisory only here.
fn extract_model(body: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let key_pos = text.find("\"model\"")?;
    let after = &text[key_pos + "\"model\"".len()..];
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Minimal JSON string-escaper for embedding an operator detail into an error
/// envelope. Mirrors the compat-api crates' escaper.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Poll a future to completion assuming it is (or quickly becomes) ready
/// without needing a reactor. The in-memory mock transport returns an
/// immediately-ready future, so this resolves on the first poll. If a future
/// were ever `Pending` here (it is not on the mock path), this busy-polls; the
/// real hyper-client transport slice replaces this with proper async dispatch
/// on the server's own runtime.
fn block_on_ready<F: Future>(fut: F) -> F::Output {
    // The std no-op waker is sound and unsafe-free; the mock future never parks
    // a real waker — it is Ready on first poll.
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

// =====================================================================
// Entry point
// =====================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .with_target(false)
        .init();

    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to read AppConfig from environment");
            std::process::exit(1);
        }
    };

    let listen_addr = config.listen_addr;

    let ComposedApp {
        router,
        chain,
        server_config,
    } = match build_app(&config) {
        Ok(parts) => parts,
        Err(e) => {
            tracing::error!(error = %e, "failed to build provider-pool app");
            std::process::exit(1);
        }
    };

    tracing::info!(
        addr = %listen_addr,
        tenant = %config.tenant_id,
        pool = %config.pool_id,
        provider = ?config.provider,
        seats = config.member_account_ids.len(),
        "oya-intelligence-provider-pool listening (in-memory mock transport; real hyper-client transport is a later slice)"
    );

    if let Err(e) = serve(listen_addr, Arc::new(router), Arc::new(chain), server_config).await {
        tracing::error!(error = %e, "hyper serve error");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> AppConfig {
        AppConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            tenant_id: "ten_local".into(),
            pool_id: "pool_local".into(),
            provider: ProviderFamily::Claude,
            member_account_ids: vec!["seat-local-1".into(), "seat-local-2".into()],
            max_body_bytes: oya_http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES,
        }
    }

    #[test]
    fn default_listen_addr_parses() {
        // The fail-closed default must be a valid SocketAddr (else from_env
        // with an unset OYA_POOL_LISTEN_ADDR would error spuriously).
        assert!(
            AppConfig::DEFAULT_LISTEN_ADDR
                .parse::<SocketAddr>()
                .is_ok()
        );
    }

    #[test]
    fn parse_provider_aliases() {
        assert_eq!(parse_provider("claude"), Some(ProviderFamily::Claude));
        assert_eq!(parse_provider("anthropic"), Some(ProviderFamily::Claude));
        assert_eq!(parse_provider("openai"), Some(ProviderFamily::OpenAiOrCodex));
        assert_eq!(parse_provider("codex"), Some(ProviderFamily::OpenAiOrCodex));
        assert_eq!(parse_provider("Gemini"), Some(ProviderFamily::Gemini));
        assert_eq!(parse_provider("nope"), None);
    }

    #[test]
    fn build_app_mounts_all_routes_panic_free() {
        let cfg = base_config();
        let ComposedApp {
            router,
            server_config,
            ..
        } = build_app(&cfg).expect("build_app succeeds");
        // healthz + 2 anthropic + 3 openai = 6 routes.
        assert_eq!(router.count(), 6);
        assert!(router.match_route(HttpMethod::Get, "/healthz").is_some());
        assert!(router.match_route(HttpMethod::Post, "/v1/messages").is_some());
        assert!(
            router
                .match_route(HttpMethod::Post, "/v1/chat/completions")
                .is_some()
        );
        assert!(router.match_route(HttpMethod::Get, "/v1/models").is_some());
        assert_eq!(
            server_config.max_body_bytes,
            oya_http_runtime_hyper_adapter::DEFAULT_MAX_BODY_BYTES
        );
    }

    #[test]
    fn healthz_returns_ok_json() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let resp = oya_http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Get,
                path: "/healthz".into(),
                headers: Default::default(),
                body: Vec::new(),
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, br#"{"status":"ok"}"#.to_vec());
    }

    #[test]
    fn messages_dispatches_through_pool_to_mock_transport() {
        let cfg = base_config();
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let body = br#"{"model":"claude-sonnet-4-6","messages":[{"role":"user","content":"hi"}],"max_tokens":16}"#.to_vec();
        let resp = oya_http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/messages".into(),
                headers: Default::default(),
                body,
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200, "mock transport returns 200");
        let text = String::from_utf8_lossy(&resp.body);
        assert!(text.contains("pool.mock"), "served by mock transport: {text}");
        assert!(text.contains("seat-local-1"), "routed to a seat: {text}");
    }

    #[test]
    fn chat_completions_dispatches_when_pool_is_openai() {
        let mut cfg = base_config();
        cfg.provider = ProviderFamily::OpenAiOrCodex;
        let ComposedApp { router, chain, .. } = build_app(&cfg).expect("build_app succeeds");
        let body =
            br#"{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}]}"#.to_vec();
        let resp = oya_http_runtime_hyper_adapter::dispatch(
            HttpRequest {
                method: HttpMethod::Post,
                path: "/v1/chat/completions".into(),
                headers: Default::default(),
                body,
                path_captures: Default::default(),
                matched_template: None,
            },
            &router,
            &chain,
        );
        assert_eq!(resp.status, 200);
        assert!(String::from_utf8_lossy(&resp.body).contains("pool.mock"));
    }

    #[test]
    fn extract_model_finds_field() {
        assert_eq!(
            extract_model(br#"{"model":"claude-x","x":1}"#).as_deref(),
            Some("claude-x")
        );
        assert_eq!(extract_model(br#"{"no":"model"}"#), None);
        assert_eq!(extract_model(b"not json"), None);
    }

    #[test]
    fn block_on_ready_resolves_ready_future() {
        let v = block_on_ready(async { 7u32 });
        assert_eq!(v, 7);
    }
}
