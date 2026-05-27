//! LLM-gateway service core (composition / usecase layer per ADR-0105).
//!
//! This crate is the *application* that wires the kernel + rest adapter into
//! the end-to-end LLM-gateway pipeline described by
//! `microservices/llm-gateway/PRD.md`:
//!
//! ```text
//! caller (OpenAI SDK) --bearer--> ingress auth ─resolve group─▶ KeyPool::select
//!                                                                     │
//!                                                                     ▼
//!                                            UpstreamTransport (port) ──▶ provider
//!                                                                     │
//!                          ◀── byte-passthrough SSE / Retry-After ─────┘
//! ```
//!
//! It owns **no** policy algorithm, **no** crypto, and **no** state-machine
//! rules of its own — those live inward:
//! - [`oya_llm_gateway_kernel`] — the pure round-robin / failure / cooldown
//!   state machine (no I/O, no async).
//! - [`oya_llm_gateway_rest`] — the OpenAI-canonical REST surface, byte-
//!   passthrough SSE, constant-time auth realms, and per-channel adapters.
//!
//! ## Layering invariant (ADR-0131 / layered-architecture discipline)
//!
//! This is the `application`/usecase ring. Path-deps inward on `-kernel`
//! and `-rest`; the only NEW seam this crate owns is the upstream-transport
//! port ([`UpstreamTransport`] re-exported from the rest crate) and the
//! `hyper`-backed adapter that satisfies it. The reference
//! [`InMemoryUpstreamAdapter`] keeps the gateway runnable in tests / bring-up
//! without a network.
//!
//! ## Hot-path posture (ADR-0083 Tier 3 — panic-free; PRD §4.3 / §5)
//!
//! [`run_gateway`] never `.unwrap()`/`.expect()`/`panic!()` on the request
//! path; configuration errors are returned as [`GatewayBootError`] before any
//! socket binds. The upstream adapter is fail-closed (a transport error is
//! retryable, NOT a panic) so a misbehaving provider can never crash the
//! process — the kernel's blacklist + cooldown handle the credit recovery.
//!
//! ## Honest boundaries (PRD §5 deferred items)
//!
//! Where a downstream is not yet wired, this crate surfaces a typed
//! [`Unimplemented`] code (e.g. `OpenBaoResolution`, `BedrockAuditEmission`,
//! `PerTenantRateLimit`) and is tracked at
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-*`.
//! No stubbed `Ok(())` for paths the gateway claims but does not implement.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` and integration tests may use unwrap/expect/panic under cfg(test).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;

use axum::Router;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use oya_llm_gateway_kernel::{PoolPolicy, ProviderChannel};
use oya_llm_gateway_rest::{
    AuthVerifier, ChannelAdapter, GatewayConfig, GatewayMetrics, GroupRuntime, KeyMaterial,
    OpenAiAppState, UpstreamBody, UpstreamError, UpstreamResponse, UpstreamTransport,
    build_openai_router,
};

pub use oya_llm_gateway_rest::Unimplemented;

// =====================================================================
// Ports
// =====================================================================

/// Persistence port for non-secret routing configuration (the ConfigMap-style
/// JSON described by [`GatewayConfig`]).
///
/// This is the seam the runtime depends on for loading the declarative
/// gateway config. Implementations: [`InMemoryGatewayConfigRepository`] for
/// tests / single-node bring-up; the `main.rs` binary loads from a file at
/// `$GATEWAY_CONFIG`. Errors are surfaced as [`RepositoryError`] so a config-
/// load failure is fail-closed rather than panicking.
pub trait GatewayConfigRepository {
    /// Load the gateway config. Returns an explicit error on missing/parse.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing source cannot be read or
    /// parsed.
    fn load(&self) -> Result<GatewayConfig, RepositoryError>;
}

/// Persistence port for per-group pooled key material. The runtime depends on
/// this for both initial key load and periodic refresh; production uses an
/// OpenBao-backed implementation (tracked at
/// `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-openbao-wire-in`),
/// dev/test use [`InMemoryKeyMaterialRepository`].
pub trait KeyMaterialRepository {
    /// Load (or reload) the keys for one group, identified by its OpenBao-
    /// style path and the channel it serves.
    ///
    /// # Errors
    /// Returns [`RepositoryError`] when the backing store cannot be read or
    /// the path is not seeded.
    fn load(&self, path: &str, channel: ProviderChannel) -> Result<KeyMaterial, RepositoryError>;
}

/// An opaque backing-store failure from a [`GatewayConfigRepository`] or
/// [`KeyMaterialRepository`]. Carries a human-facing detail for logs without
/// leaking store internals into the typed control flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryError {
    detail: String, // data_class: INTERNAL_ONLY
}

impl RepositoryError {
    /// Construct a store error with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "llm-gateway store error: {}", self.detail)
    }
}

impl std::error::Error for RepositoryError {}

// =====================================================================
// In-memory reference adapters
// =====================================================================

/// In-memory [`GatewayConfigRepository`] backed by a single [`GatewayConfig`].
/// The reference adapter for tests and single-node bring-up; production wires
/// `main.rs` to read from `$GATEWAY_CONFIG`.
#[derive(Clone, Debug)]
pub struct InMemoryGatewayConfigRepository {
    config: GatewayConfig, // data_class: INTERNAL_ONLY
}

impl InMemoryGatewayConfigRepository {
    /// Seed the repository with a fully-validated config.
    #[must_use]
    pub fn new(config: GatewayConfig) -> Self {
        Self { config }
    }
}

impl GatewayConfigRepository for InMemoryGatewayConfigRepository {
    fn load(&self) -> Result<GatewayConfig, RepositoryError> {
        // Validate eagerly so a misconfigured repository fails closed BEFORE
        // we touch the network.
        self.config
            .validate()
            .map_err(|e| RepositoryError::new(e.to_string()))?;
        Ok(self.config.clone())
    }
}

/// In-memory [`KeyMaterialRepository`] backed by a `BTreeMap` keyed by the
/// OpenBao-style path. The reference adapter; production swaps in an OpenBao-
/// backed store behind the same port (PRD §4.5).
///
/// **Fail-closed**: [`KeyMaterialRepository::load`] returns
/// [`RepositoryError`] for any path that was not seeded, so a misconfiguration
/// surfaces as an error instead of a silently empty pool.
#[derive(Clone, Default)]
pub struct InMemoryKeyMaterialRepository {
    by_path: BTreeMap<String, KeyMaterial>, // data_class: INTERNAL_ONLY
}

impl fmt::Debug for InMemoryKeyMaterialRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryKeyMaterialRepository")
            .field("paths", &self.by_path.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl InMemoryKeyMaterialRepository {
    /// An empty repository. Loading any path fails closed until keys are seeded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed (or replace) the keys served for `path`.
    #[must_use]
    pub fn with_keys(mut self, path: impl Into<String>, material: KeyMaterial) -> Self {
        self.by_path.insert(path.into(), material);
        self
    }
}

impl KeyMaterialRepository for InMemoryKeyMaterialRepository {
    fn load(&self, path: &str, channel: ProviderChannel) -> Result<KeyMaterial, RepositoryError> {
        match self.by_path.get(path) {
            Some(material) if material.channel() == channel => Ok(material.clone()),
            _ => Err(RepositoryError::new(format!(
                "no key material seeded for path {path:?} (channel {})",
                channel.as_str()
            ))),
        }
    }
}

// =====================================================================
// In-memory upstream transport (tests / dev)
// =====================================================================

/// A scripted upstream response factory: given the channel + path it returns
/// a fully-formed [`UpstreamResponse`] (or [`UpstreamError`]). Used by tests
/// to drive the failover loop deterministically.
pub type UpstreamScript = Arc<
    dyn Fn(ProviderChannel, &str, &Bytes) -> Result<UpstreamResponse, UpstreamError> + Send + Sync,
>;

/// In-memory [`UpstreamTransport`] used in acceptance tests / single-node
/// bring-up. The script is consulted on every dispatch; no socket is opened.
#[derive(Clone)]
pub struct InMemoryUpstreamAdapter {
    script: UpstreamScript,
}

impl InMemoryUpstreamAdapter {
    /// Build an adapter from a per-call response script.
    #[must_use]
    pub fn new(script: UpstreamScript) -> Self {
        Self { script }
    }
}

impl UpstreamTransport for InMemoryUpstreamAdapter {
    fn dispatch(
        &self,
        channel: ProviderChannel,
        _upstream_base_url: &str,
        _method: &str,
        path_and_query: &str,
        _auth_headers: Vec<(&'static str, String)>,
        _forwarded_headers: Vec<(String, String)>,
        body: Bytes,
        _streaming: bool,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<UpstreamResponse, UpstreamError>> + Send + '_>,
    > {
        let result = (self.script)(channel, path_and_query, &body);
        Box::pin(async move { result })
    }
}

// =====================================================================
// Production hyper-backed upstream transport
// =====================================================================

/// Production [`UpstreamTransport`]: hyper-util legacy client over a rustls
/// HTTPS connector (ring crypto + webpki trust roots). Shares the connection
/// pool across requests for the lifetime of the gateway process.
///
/// The streaming path wraps `hyper::body::Incoming` as an `UpstreamBody::Stream`
/// so the REST handler can `Body::from_stream` it straight into the response
/// (chunk-boundary preservation, PRD §4.2 / AC-2.2).
#[derive(Clone)]
pub struct HyperUpstreamAdapter {
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>,
}

impl HyperUpstreamAdapter {
    /// Build a process-wide hyper client. No total-request timeout: SSE
    /// streams are long-lived (the connector applies a connect timeout to
    /// guard dead upstreams).
    #[must_use]
    pub fn new() -> Self {
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        http.set_connect_timeout(Some(std::time::Duration::from_secs(10)));
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .wrap_connector(http);
        let client = Client::builder(TokioExecutor::new()).build(https);
        HyperUpstreamAdapter { client }
    }
}

impl Default for HyperUpstreamAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl UpstreamTransport for HyperUpstreamAdapter {
    fn dispatch(
        &self,
        _channel: ProviderChannel,
        upstream_base_url: &str,
        method: &str,
        path_and_query: &str,
        auth_headers: Vec<(&'static str, String)>,
        forwarded_headers: Vec<(String, String)>,
        body: Bytes,
        streaming: bool,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<UpstreamResponse, UpstreamError>> + Send + '_>,
    > {
        let client = self.client.clone();
        let url = compose_url(upstream_base_url, path_and_query);
        let method = method.to_string();
        Box::pin(async move {
            let mut builder = hyper::Request::builder().method(method.as_str()).uri(&url);
            if let Some(h) = builder.headers_mut() {
                for (name, value) in &forwarded_headers {
                    if let (Ok(hn), Ok(hv)) = (
                        hyper::header::HeaderName::from_bytes(name.as_bytes()),
                        hyper::header::HeaderValue::from_bytes(value.as_bytes()),
                    ) {
                        h.append(hn, hv);
                    }
                }
                // Pooled auth overwrites any caller-supplied header.
                for (name, value) in auth_headers {
                    if let (Ok(hn), Ok(hv)) = (
                        hyper::header::HeaderName::from_bytes(name.as_bytes()),
                        hyper::header::HeaderValue::from_bytes(value.as_bytes()),
                    ) {
                        h.insert(hn, hv);
                    }
                }
            }
            let request = builder
                .body(Full::new(body))
                .map_err(|e| UpstreamError::BadRequest(e.to_string()))?;
            let response = client
                .request(request)
                .await
                .map_err(|e| UpstreamError::Transport(e.to_string()))?;
            let status = axum::http::StatusCode::from_u16(response.status().as_u16())
                .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);
            // Capture headers verbatim (skip hop-by-hop on the way out is the
            // REST handler's job, not the transport's).
            let mut headers: Vec<(String, String)> = Vec::with_capacity(response.headers().len());
            for (name, value) in response.headers() {
                if let Ok(v) = value.to_str() {
                    headers.push((name.as_str().to_string(), v.to_string()));
                }
            }
            let retry_after_seconds = headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case("retry-after"))
                .and_then(|(_, v)| v.trim().parse::<u64>().ok());

            let body = if streaming {
                let upstream_body = response.into_body();
                let byte_stream =
                    futures_util::stream::unfold(upstream_body, |mut body| async move {
                        loop {
                            match body.frame().await {
                                Some(Ok(frame)) => match frame.into_data() {
                                    Ok(chunk) => {
                                        return Some((Ok::<Bytes, std::io::Error>(chunk), body));
                                    }
                                    Err(_non_data) => continue,
                                },
                                Some(Err(err)) => {
                                    return Some((
                                        Err(std::io::Error::other(err.to_string())),
                                        body,
                                    ));
                                }
                                None => return None,
                            }
                        }
                    });
                UpstreamBody::Stream(Box::new(Box::pin(byte_stream)))
            } else {
                let collected = response
                    .into_body()
                    .collect()
                    .await
                    .map_err(|e| UpstreamError::Transport(e.to_string()))?;
                UpstreamBody::Buffered(collected.to_bytes())
            };

            Ok(UpstreamResponse {
                status,
                retry_after_seconds,
                headers,
                body,
            })
        })
    }
}

/// Join `base` (no trailing slash) and `path` (leading-slash tolerated) with
/// exactly one `/`. Pure.
fn compose_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let tail = path.trim_start_matches('/');
    if tail.is_empty() {
        base.to_string()
    } else {
        format!("{base}/{tail}")
    }
}

// =====================================================================
// Wire-up / lifecycle
// =====================================================================

/// Errors raised by [`build_router_from_config`] / [`run_gateway`] BEFORE the
/// listener binds. All conditions that would cause the gateway to serve an
/// empty pool, an unknown channel, or no groups are surfaced here.
#[derive(Debug)]
pub enum GatewayBootError {
    /// The config repository failed to produce a valid [`GatewayConfig`].
    Repository(RepositoryError),
    /// A group's `channel` string did not resolve to a known provider.
    UnknownChannel { group: String, channel: String },
    /// A group's key material failed to load.
    KeyMaterial {
        group: String,
        error: RepositoryError,
    },
    /// A composition invariant was violated (e.g. zero groups configured).
    NoGroups,
    /// The auth verifier was constructed with no ingress keys (fail-closed).
    NoIngressKeys,
    /// A TCP listener could not be bound.
    Bind { address: String, error: String },
    /// The hyper/axum serve loop exited with an error.
    Serve(String),
}

impl fmt::Display for GatewayBootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(e) => write!(f, "{e}"),
            Self::UnknownChannel { group, channel } => {
                write!(f, "group {group}: unknown provider channel {channel:?}")
            }
            Self::KeyMaterial { group, error } => {
                write!(f, "group {group}: key material load failed: {error}")
            }
            Self::NoGroups => write!(f, "gateway config declares no groups"),
            Self::NoIngressKeys => write!(
                f,
                "auth verifier requires at least one ingress key (fail-closed)"
            ),
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(e) => write!(f, "serve error: {e}"),
        }
    }
}

impl std::error::Error for GatewayBootError {}

impl From<RepositoryError> for GatewayBootError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// Wire the LLM-gateway state from a config repository + key repository +
/// auth verifier. Returns the assembled state ready to mount on a router.
/// Pure of I/O once both repositories have already loaded — this is the
/// composition root's "build the world" step.
///
/// # Errors
/// Returns [`GatewayBootError`] when the config is missing/invalid, a group
/// channel is unrecognized, key material is absent for a configured path, or
/// the auth verifier holds no ingress keys.
pub fn build_gateway_state<C, K>(
    config_repo: &C,
    key_repo: &K,
    auth: AuthVerifier,
    metrics: GatewayMetrics,
) -> Result<Arc<oya_llm_gateway_rest::GatewayState>, GatewayBootError>
where
    C: GatewayConfigRepository,
    K: KeyMaterialRepository,
{
    if auth.ingress_key_count() == 0 {
        return Err(GatewayBootError::NoIngressKeys);
    }
    let config = config_repo.load()?;
    if config.groups.is_empty() {
        return Err(GatewayBootError::NoGroups);
    }
    let mut groups = BTreeMap::new();
    for group_cfg in &config.groups {
        let channel =
            group_cfg
                .parsed_channel()
                .ok_or_else(|| GatewayBootError::UnknownChannel {
                    group: group_cfg.name.clone(),
                    channel: group_cfg.channel.clone(),
                })?;
        let material = key_repo
            .load(&group_cfg.bao_key_path, channel)
            .map_err(|error| GatewayBootError::KeyMaterial {
                group: group_cfg.name.clone(),
                error,
            })?;
        let adapter = ChannelAdapter::new(
            channel,
            group_cfg.upstream_base_url.clone(),
            group_cfg.anthropic_version.clone(),
        );
        let policy = PoolPolicy::new(
            group_cfg.blacklist_threshold,
            group_cfg.cooldown_base_millis,
            group_cfg.cooldown_jitter_millis,
        );
        let runtime = GroupRuntime::new(
            group_cfg.name.clone(),
            adapter,
            group_cfg.retry.clone(),
            policy,
            material,
        );
        groups.insert(group_cfg.name.clone(), runtime);
    }
    let state = Arc::new(oya_llm_gateway_rest::GatewayState::new(
        groups, auth, metrics,
    ));
    Ok(state)
}

/// Build the full axum app: the OpenAI-canonical surface (mounted on the
/// chosen default group) PLUS the existing per-group reverse-proxy router
/// from `oya-llm-gateway-rest`. Both share the same [`oya_llm_gateway_rest::GatewayState`].
///
/// # Errors
/// Returns [`GatewayBootError`] if the chosen default group is not present in
/// the assembled state.
pub fn build_router(
    state: Arc<oya_llm_gateway_rest::GatewayState>,
    transport: Arc<dyn UpstreamTransport>,
    default_group: impl Into<String>,
) -> Result<Router, GatewayBootError> {
    let default_group = default_group.into();
    if state.group(&default_group).is_none() {
        return Err(GatewayBootError::UnknownChannel {
            group: default_group,
            channel: "default-group-missing".to_string(),
        });
    }
    let openai_state = OpenAiAppState::new(Arc::clone(&state), transport, default_group);
    let openai = build_openai_router(openai_state);
    let proxy = oya_llm_gateway_rest::proxy::build_router(state);
    // The OpenAI router defines `/v1/*` while the proxy router defines
    // `/proxy/*`, `/healthz`, `/metrics`. Merge them — axum routes are
    // path-distinct so this is conflict-free.
    Ok(openai.merge(proxy))
}

/// Bind a listener on `listen_addr` and serve the assembled router. Returns
/// when the serve loop exits (typically only on shutdown).
///
/// # Errors
/// Returns [`GatewayBootError::Bind`] if the address cannot be bound, or
/// [`GatewayBootError::Serve`] if axum exits with an error.
pub async fn serve(listen_addr: &str, app: Router) -> Result<(), GatewayBootError> {
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|e| GatewayBootError::Bind {
            address: listen_addr.to_string(),
            error: e.to_string(),
        })?;
    tracing::info!(
        target: "oya_llm_gateway_app::boot",
        addr = listen_addr,
        "llm-gateway listening"
    );
    axum::serve(listener, app)
        .await
        .map_err(|e| GatewayBootError::Serve(e.to_string()))
}

/// One-shot end-to-end runner: build state + router from the repositories +
/// auth + metrics, then bind on `listen_addr` and serve. Returns on serve
/// shutdown.
///
/// # Errors
/// Returns [`GatewayBootError`] on any boot-time failure (see variants).
pub async fn run_gateway<C, K>(
    config_repo: &C,
    key_repo: &K,
    auth: AuthVerifier,
    metrics: GatewayMetrics,
    transport: Arc<dyn UpstreamTransport>,
    listen_addr: &str,
    default_group: impl Into<String>,
) -> Result<(), GatewayBootError>
where
    C: GatewayConfigRepository,
    K: KeyMaterialRepository,
{
    let state = build_gateway_state(config_repo, key_repo, auth, metrics)?;
    let app = build_router(state, transport, default_group)?;
    serve(listen_addr, app).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap as Map;

    fn sample_config() -> GatewayConfig {
        // A minimal valid config with a single OpenAI group.
        let json = r#"
        {
          "listen_addr": "127.0.0.1:0",
          "openbao": { "address": "http://openbao.invalid:8200" },
          "key_refresh_secs": 0,
          "groups": [
            {
              "name": "codex",
              "channel": "openai",
              "upstream_base_url": "https://api.openai.com",
              "bao_key_path": "agent-gateway/openai",
              "blacklist_threshold": 3,
              "cooldown_base_millis": 1000,
              "cooldown_jitter_millis": 0
            }
          ]
        }
        "#;
        GatewayConfig::from_json(json).expect("valid sample config")
    }

    fn material(channel: ProviderChannel, keys: &[(&str, &str)]) -> KeyMaterial {
        let mut map = Map::new();
        for (label, key) in keys {
            map.insert((*label).to_string(), (*key).to_string());
        }
        KeyMaterial::from_map(channel, map)
    }

    fn build_test_repos() -> (
        InMemoryGatewayConfigRepository,
        InMemoryKeyMaterialRepository,
    ) {
        let config = sample_config();
        let cfg_repo = InMemoryGatewayConfigRepository::new(config);
        let key_repo = InMemoryKeyMaterialRepository::new().with_keys(
            "agent-gateway/openai",
            material(ProviderChannel::OpenAi, &[("a", "sk-aaa"), ("b", "sk-bbb")]),
        );
        (cfg_repo, key_repo)
    }

    #[test]
    fn in_memory_config_repo_validates_eagerly() {
        let cfg_repo = InMemoryGatewayConfigRepository::new(sample_config());
        let cfg = cfg_repo.load().expect("valid config loads");
        assert_eq!(cfg.groups.len(), 1);
        assert_eq!(cfg.groups[0].name, "codex");
    }

    #[test]
    fn in_memory_key_repo_fails_closed_on_unseeded_path() {
        let repo = InMemoryKeyMaterialRepository::new();
        let err = repo
            .load("agent-gateway/openai", ProviderChannel::OpenAi)
            .expect_err("unseeded path");
        assert!(err.detail().contains("agent-gateway/openai"));
    }

    #[test]
    fn in_memory_key_repo_rejects_channel_mismatch() {
        let repo = InMemoryKeyMaterialRepository::new().with_keys(
            "agent-gateway/openai",
            material(ProviderChannel::OpenAi, &[("a", "sk-aaa")]),
        );
        // Seeded for OpenAi but requested as Anthropic -> fail closed.
        assert!(
            repo.load("agent-gateway/openai", ProviderChannel::Anthropic)
                .is_err()
        );
    }

    #[test]
    fn build_state_assembles_groups_from_repositories() {
        let (cfg, keys) = build_test_repos();
        let auth = AuthVerifier::new("admin-tok", vec!["ingress-secret".to_string()]);
        let metrics = GatewayMetrics::new().expect("metrics");
        let state = build_gateway_state(&cfg, &keys, auth, metrics).expect("build");
        assert!(state.group("codex").is_some());
        assert_eq!(state.group_names(), vec!["codex"]);
    }

    #[test]
    fn build_state_fails_closed_on_missing_ingress_keys() {
        let (cfg, keys) = build_test_repos();
        let auth = AuthVerifier::new("admin", Vec::new());
        let metrics = GatewayMetrics::new().expect("metrics");
        let result = build_gateway_state(&cfg, &keys, auth, metrics);
        match result {
            Err(GatewayBootError::NoIngressKeys) => {}
            Err(other) => panic!("expected NoIngressKeys, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn build_state_fails_closed_on_missing_key_material() {
        let cfg = InMemoryGatewayConfigRepository::new(sample_config());
        // No keys seeded.
        let keys = InMemoryKeyMaterialRepository::new();
        let auth = AuthVerifier::new("admin", vec!["ingress".to_string()]);
        let metrics = GatewayMetrics::new().expect("metrics");
        let result = build_gateway_state(&cfg, &keys, auth, metrics);
        match result {
            Err(GatewayBootError::KeyMaterial { .. }) => {}
            Err(other) => panic!("expected KeyMaterial, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn compose_url_joins_with_exactly_one_slash() {
        assert_eq!(
            compose_url("https://api.openai.com", "/v1/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            compose_url("https://api.openai.com/", "v1/models"),
            "https://api.openai.com/v1/models"
        );
        assert_eq!(
            compose_url("https://api.openai.com", ""),
            "https://api.openai.com"
        );
        assert_eq!(
            compose_url("https://api.openai.com/", "/"),
            "https://api.openai.com"
        );
    }

    #[tokio::test]
    async fn in_memory_upstream_adapter_calls_script() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let captured_path = Arc::new(std::sync::Mutex::new(String::new()));
        let cap = Arc::clone(&captured_path);
        let count = Arc::clone(&counter);
        let script: UpstreamScript = Arc::new(move |_ch, path, _body| {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            *cap.lock().unwrap() = path.to_string();
            Ok(UpstreamResponse {
                status: axum::http::StatusCode::OK,
                retry_after_seconds: None,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: UpstreamBody::Buffered(Bytes::from_static(b"{\"ok\":true}")),
            })
        });
        let adapter = InMemoryUpstreamAdapter::new(script);
        let resp = adapter
            .dispatch(
                ProviderChannel::OpenAi,
                "https://api.openai.com",
                "POST",
                "/v1/chat/completions",
                vec![("authorization", "Bearer sk-foo".to_string())],
                vec![("content-type".to_string(), "application/json".to_string())],
                Bytes::from_static(b"{}"),
                false,
            )
            .await
            .expect("script ok");
        assert_eq!(resp.status, axum::http::StatusCode::OK);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(*captured_path.lock().unwrap(), "/v1/chat/completions");
    }

    #[test]
    fn unimplemented_codes_are_stable() {
        assert_eq!(
            Unimplemented::OpenBaoResolution.type_slug(),
            "gateway_unimplemented_openbao_resolution"
        );
    }
}
