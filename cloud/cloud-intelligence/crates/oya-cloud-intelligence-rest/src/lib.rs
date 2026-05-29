//! cloud-intelligence REST adapter — OAuth subscription pool (ADR-0384 Path B).
//!
//! Stage-6 GREEN. Stage-7 SSE passthrough added. Implements:
//! - [`OpenBaoSecretStore`] trait — D8 envelope-encrypted refresh-token storage seam.
//! - [`EventSinkFanout`] — D6 fan-out broadcaster.
//! - [`AnthropicAdapter`] — D3 Anthropic OAuth refresh + async reqwest proxy.
//! - [`ProxyRequest`] / [`ProxyResponse`] — D2 axum reverse-proxy wire types.
//! - [`build_router`] — axum router wiring POST /v1/messages, GET /healthz,
//!   GET /metrics.
//!
//! Stage-6 changes (Item 1 + Item 2):
//! - `AnthropicAdapter` migrated from `reqwest::blocking` to async `reqwest::Client`.
//!   The adapter borrows `&reqwest::Client` — it does NOT own one. `AppState` holds
//!   the shared `Arc<reqwest::Client>` so TLS handshakes and keep-alive connections
//!   are pooled across requests.
//! - Singleflight moved INSIDE `AnthropicAdapter::refresh_token` via
//!   `tokio::sync::Mutex<HashMap<String, broadcast::Sender<...>>>`. Concurrent callers
//!   on the same handle now coalesce exactly ONE upstream OAuth call (Item 2).
//! - `handle_proxy` no longer uses `spawn_blocking`.
//!
//! Stage-7 changes (SSE streaming passthrough):
//! - `AnthropicAdapter::proxy_stream` — returns a `BoxStream` of raw SSE `Bytes`
//!   chunks from Anthropic. Detected by `Accept: text/event-stream` request header.
//! - `SseStreamWithLease` — wrapper that holds a `SeatLease` alive for the full
//!   duration of the stream. Lease is completed with `Ok` on clean end-of-stream,
//!   or `Released` (via `Drop`) if the client disconnects mid-stream.
//! - `handle_proxy` branches on `Accept: text/event-stream` and returns a chunked
//!   HTTP/1.1 `text/event-stream` response instead of a buffered JSON body.
//!
//! TODO(codex-adapter): add `CodexAdapter` mirroring `AnthropicAdapter` once
//! the Codex OAuth refresh flow is documented. Tracked as a separate follow-up
//! PR per ADR-0384 §v1-scope.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, HashMap};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Type alias for a heap-allocated, `Send + 'static` SSE byte stream.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

pub use oya_cloud_intelligence_kernel::{
    AgentId, AuthzGate, EventSink, LlmGatewayEvent, Provider, SeatId, SeatLease, SeatOutcome,
    SubscriptionPool, SubscriptionPoolError, TenantId,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified error type for the REST adapter layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestAdapterError {
    /// Secret handle was not found in the store.
    SecretNotFound,
    /// Secret store returned a transport or vault error.
    SecretStoreUnavailable(String),
    /// Supplied plaintext was empty or otherwise invalid.
    InvalidSecret,
    /// Upstream Anthropic API returned an error status.
    UpstreamError { status: u16, body: String },
    /// OAuth refresh token exchange failed.
    OAuthRefreshFailed(String),
    /// Kernel pool returned an error.
    PoolError(SubscriptionPoolError),
    /// Fan-out sink emit failed (logged; non-fatal in proxy path).
    SinkEmitFailed(String),
}

impl From<SubscriptionPoolError> for RestAdapterError {
    fn from(e: SubscriptionPoolError) -> Self {
        RestAdapterError::PoolError(e)
    }
}

impl std::fmt::Display for RestAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SecretNotFound => write!(f, "secret not found"),
            Self::SecretStoreUnavailable(msg) => write!(f, "secret store unavailable: {msg}"),
            Self::InvalidSecret => write!(f, "invalid secret"),
            Self::UpstreamError { status, body } => {
                write!(f, "upstream error {status}: {body}")
            }
            Self::OAuthRefreshFailed(msg) => write!(f, "OAuth refresh failed: {msg}"),
            Self::PoolError(e) => write!(f, "pool error: {e:?}"),
            Self::SinkEmitFailed(msg) => write!(f, "sink emit failed: {msg}"),
        }
    }
}

impl std::error::Error for RestAdapterError {}

// ---------------------------------------------------------------------------
// D8 — OpenBao envelope-encrypted refresh-token storage seam
// ---------------------------------------------------------------------------

/// D8 secret-store seam. Implementors envelope-encrypt/decrypt via OpenBao
/// transit secrets engine. The kernel never sees plaintext tokens; only opaque
/// handles cross the kernel boundary.
///
/// Real implementation ships in `oya-cloud-intelligence-openbao-adapter` (separate
/// crate, follow-up PR).
pub trait OpenBaoSecretStore: Send + Sync {
    /// Fetch and decrypt the refresh token identified by `handle`.
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError>;

    /// Envelope-encrypt `plaintext` and store it under `handle`.
    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError>;
}

// ---------------------------------------------------------------------------
// D6 — EventSink fan-out
// ---------------------------------------------------------------------------

/// D6 fan-out broadcaster. Holds a list of [`EventSink`] impls and emits each
/// [`LlmGatewayEvent`] to every registered sink in registration order.
///
/// Sink failures are recorded in `last_errors` (non-fatal): the proxy path
/// MUST NOT fail a request because a sink is unavailable.
pub struct EventSinkFanout {
    sinks: Vec<Box<dyn EventSink + Send + Sync>>,
}

impl EventSinkFanout {
    /// Construct a fanout with no initial sinks.
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    /// Register an additional sink.
    pub fn add_sink(&mut self, sink: Box<dyn EventSink + Send + Sync>) {
        self.sinks.push(sink);
    }

    /// Broadcast `event` to every registered sink. Returns the count of sinks
    /// that received the event successfully (panicking sinks are skipped via
    /// catch_unwind and their panics are logged but do not propagate).
    pub fn broadcast(&self, event: LlmGatewayEvent) -> usize {
        let mut delivered = 0usize;
        for sink in &self.sinks {
            // Safety: `catch_unwind` requires `UnwindSafe`. The closure captures
            // only a shared ref to `sink` + a clone of `event`; the `AssertUnwindSafe`
            // wrapper is required because `dyn EventSink` is not `UnwindSafe`.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                sink.emit(event.clone());
            }));
            match result {
                Ok(()) => {
                    delivered += 1;
                }
                Err(payload) => {
                    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic payload".to_string()
                    };
                    warn!(sink_panic = %msg, "EventSink panicked during broadcast; continuing to next sink");
                }
            }
        }
        delivered
    }

    /// Number of registered sinks.
    pub fn sink_count(&self) -> usize {
        self.sinks.len()
    }
}

impl Default for EventSinkFanout {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// D2 — proxy wire types
// ---------------------------------------------------------------------------

/// Inbound HTTP request forwarded through the axum reverse proxy (D2).
///
/// Bodies are opaque byte vectors; header maps use `BTreeMap` for
/// deterministic ordering in tests.
#[derive(Clone, Debug)]
pub struct ProxyRequest {
    pub method: String,                    // data_class: INTERNAL_ONLY
    pub path: String,                      // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,               // data_class: INTERNAL_ONLY
}

/// Outbound HTTP response returned by the axum reverse proxy (D2).
#[derive(Clone, Debug)]
pub struct ProxyResponse {
    pub status: u16,                       // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// D3 — Anthropic provider adapter
// ---------------------------------------------------------------------------

/// Anthropic client_id per ADR-0384.
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Default Anthropic base URL.
const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

/// Anthropic API version header value.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic beta header value for OAuth.
const ANTHROPIC_BETA_OAUTH: &str = "oauth-2025-04-20";

/// Wire shape for Anthropic token endpoint request (refresh grant).
#[derive(Serialize)]
struct OAuthRefreshRequest<'a> {
    client_id: &'a str,     // data_class: INTERNAL_ONLY
    grant_type: &'a str,    // data_class: INTERNAL_ONLY
    refresh_token: &'a str, // data_class: INTERNAL_ONLY
}

/// Wire shape for Anthropic token endpoint successful response.
#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,    // data_class: INTERNAL_ONLY
    refresh_token: String,   // data_class: INTERNAL_ONLY
    expires_in: Option<u64>, // data_class: INTERNAL_ONLY
}

/// Resolved tokens returned from a successful refresh.
pub struct AnthropicTokens {
    pub access_token: String,  // data_class: INTERNAL_ONLY
    pub refresh_token: String, // data_class: INTERNAL_ONLY
    pub expires_at: u64,       // data_class: INTERNAL_ONLY (unix seconds)
}

// ---------------------------------------------------------------------------
// Item 2 — upstream OAuth singleflight
// ---------------------------------------------------------------------------

/// Result type broadcast across concurrent waiters for a single token handle.
/// `Ok` carries the new access token string; `Err` carries the error message.
type RefreshResult = Result<String, String>; // data_class: INTERNAL_ONLY

/// D3 upstream OAuth singleflight. Coalesces concurrent `refresh_token` calls
/// for the same handle into exactly ONE upstream call. All concurrent callers
/// receive the same result via a `broadcast::Sender`.
///
/// Pattern: `tokio::sync::Mutex<HashMap<handle, broadcast::Sender<RefreshResult>>>`.
/// When a flight completes the entry is removed so the next caller starts fresh.
pub struct UpstreamOAuthSingleflight {
    /// In-flight map: present = flight in progress. Absence = no flight.
    flights: tokio::sync::Mutex<HashMap<String, broadcast::Sender<RefreshResult>>>, // data_class: INTERNAL_ONLY
}

impl UpstreamOAuthSingleflight {
    pub fn new() -> Self {
        Self {
            flights: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Call `do_refresh` exactly once per in-flight window for `handle`.
    /// All concurrent callers for the same handle wait on the broadcast channel
    /// and receive the leader's result.
    ///
    /// `do_refresh` is an async closure/future that performs the actual upstream
    /// OAuth call. It is awaited only by the leader task.
    pub async fn refresh_or_wait<F, Fut>(
        &self,
        handle: &str,
        do_refresh: F,
    ) -> Result<String, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        let mut rx = {
            let mut map = self.flights.lock().await;
            if let Some(tx) = map.get(handle) {
                // A flight is in progress — subscribe and wait below.
                tx.subscribe()
            } else {
                // We are the leader. Create the broadcast channel (capacity 64 is
                // ample; all subscribers will receive the single message sent).
                let (tx, _rx) = broadcast::channel(64);
                map.insert(handle.to_string(), tx.clone());
                drop(map); // release lock before awaiting upstream

                let result = do_refresh().await;
                // Notify all waiters (ignore send errors: no active receivers is ok).
                let _ = tx.send(result.clone());

                // Remove flight so the next caller starts fresh.
                self.flights.lock().await.remove(handle);

                return result;
            }
        };

        // Waiter path: await the broadcast result.
        rx.recv()
            .await
            .unwrap_or_else(|_| Err("singleflight: leader channel closed unexpectedly".to_string()))
    }
}

impl Default for UpstreamOAuthSingleflight {
    fn default() -> Self {
        Self::new()
    }
}

/// D3 Anthropic provider adapter. Responsible for:
/// 1. Refreshing tokens before expiry (or on 401) via the Anthropic OAuth
///    endpoint (coalesced via per-adapter [`UpstreamOAuthSingleflight`]).
/// 2. Routing proxied requests to `https://api.anthropic.com` with the
///    bearer token fetched from [`OpenBaoSecretStore`].
///
/// The adapter borrows `&reqwest::Client` — it does NOT own one. The shared
/// client lives in [`AppState`] so TLS sessions and keep-alive connections are
/// amortized across the full request lifetime of the process.
///
/// TODO(codex-adapter): `CodexAdapter` will mirror this struct for the OpenAI
/// Codex OAuth flow. Deferred to a follow-up PR per ADR-0384 §v1-scope.
pub struct AnthropicAdapter<S: OpenBaoSecretStore> {
    secret_store: S,                              // data_class: INTERNAL_ONLY
    singleflight: Arc<UpstreamOAuthSingleflight>, // data_class: INTERNAL_ONLY
    base_url: String,                             // data_class: INTERNAL_ONLY
    client_id: String,                            // data_class: INTERNAL_ONLY
}

impl<S: OpenBaoSecretStore> AnthropicAdapter<S> {
    /// Construct with a concrete [`OpenBaoSecretStore`] implementation.
    /// Uses the default Anthropic base URL and client_id from ADR-0384.
    pub fn new(secret_store: S) -> Self {
        Self::with_base_url(secret_store, ANTHROPIC_BASE_URL.to_string())
    }

    /// Construct with a custom base URL (for testing against a local mock server).
    pub fn with_base_url(secret_store: S, base_url: String) -> Self {
        Self {
            secret_store,
            singleflight: Arc::new(UpstreamOAuthSingleflight::new()),
            base_url,
            client_id: ANTHROPIC_CLIENT_ID.to_string(),
        }
    }

    /// Exchange an OAuth authorization code for access + refresh tokens, then
    /// store the refresh token via the secret store and return the opaque
    /// handle.
    ///
    /// The handle is `<tenant_id>/<seat_id>` by convention so the caller can
    /// reconstruct it without extra state.
    pub async fn exchange_authorization_code(
        &self,
        http_client: &reqwest::Client,
        tenant_id: &TenantId,
        seat_id: &SeatId,
        authorization_code: &str,
    ) -> Result<String, RestAdapterError> {
        #[derive(Serialize)]
        struct CodeRequest<'a> {
            client_id: &'a str,
            grant_type: &'a str,
            code: &'a str,
        }
        let url = format!("{}/v1/oauth/token", self.base_url);
        let body = CodeRequest {
            client_id: &self.client_id,
            grant_type: "authorization_code",
            code: authorization_code,
        };
        debug!(tenant = %tenant_id.as_str(), seat = %seat_id.as_str(), "exchanging authorization code");
        let resp = http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(RestAdapterError::OAuthRefreshFailed(format!(
                "code exchange failed: HTTP {status}: {text}"
            )));
        }
        let token_resp: OAuthTokenResponse = resp
            .json()
            .await
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        let handle = format!("{}/{}", tenant_id.as_str(), seat_id.as_str());
        self.secret_store
            .store_refresh_token(&handle, &token_resp.refresh_token)?;
        Ok(handle)
    }

    /// Refresh the token identified by `refresh_token_handle`. Fetches the
    /// current refresh token from the secret store, exchanges it with
    /// Anthropic via the upstream singleflight coalescer, stores the new
    /// refresh token, and returns the bearer access token to use for this
    /// request.
    ///
    /// Concurrent callers on the same handle see exactly ONE upstream call;
    /// followers wait on the broadcast channel (Item 2).
    pub async fn refresh_token(
        &self,
        http_client: &reqwest::Client,
        refresh_token_handle: &str,
    ) -> Result<String, RestAdapterError> {
        let current_refresh = self
            .secret_store
            .fetch_refresh_token(refresh_token_handle)?;

        let url = format!("{}/v1/oauth/token", self.base_url);
        let client_id = self.client_id.clone();
        let handle = refresh_token_handle.to_string();
        let secret_store_ref: &S = &self.secret_store;

        debug!(
            handle = refresh_token_handle,
            "refreshing Anthropic OAuth token via singleflight"
        );

        let result = self
            .singleflight
            .refresh_or_wait(refresh_token_handle, || {
                let url = url.clone();
                let client_id = client_id.clone();
                let current_refresh = current_refresh.clone();
                let http_client = http_client.clone();
                async move {
                    let body = OAuthRefreshRequest {
                        client_id: &client_id,
                        grant_type: "refresh_token",
                        refresh_token: &current_refresh,
                    };
                    let resp = http_client
                        .post(&url)
                        .json(&body)
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    let status = resp.status().as_u16();
                    if !resp.status().is_success() {
                        let text = resp.text().await.unwrap_or_default();
                        return Err(format!("token refresh failed: HTTP {status}: {text}"));
                    }
                    let token_resp: OAuthTokenResponse =
                        resp.json().await.map_err(|e| e.to_string())?;
                    Ok(token_resp.access_token + "\x00" + &token_resp.refresh_token)
                }
            })
            .await
            .map_err(RestAdapterError::OAuthRefreshFailed)?;

        // Result is encoded as "access_token\x00new_refresh_token".
        let mut parts = result.splitn(2, '\x00');
        let access_token = parts.next().unwrap_or("").to_string();
        let new_refresh = parts.next().unwrap_or("");

        if !new_refresh.is_empty() {
            // Rotate refresh token in secret store (best-effort for followers — leader
            // already stored it; followers store again which is idempotent).
            let _ = secret_store_ref.store_refresh_token(&handle, new_refresh);
        }

        Ok(access_token)
    }

    /// Forward `request` to the Anthropic API using the bearer token obtained
    /// (or refreshed) for the selected seat. Returns the upstream response.
    ///
    /// Headers forwarded: all except `authorization`, `host`, `content-length`
    /// (those are set or managed by reqwest).
    pub async fn proxy(
        &self,
        http_client: &reqwest::Client,
        request: &ProxyRequest,
        refresh_token_handle: &str,
    ) -> Result<ProxyResponse, RestAdapterError> {
        let access_token = self
            .refresh_token(http_client, refresh_token_handle)
            .await?;
        let url = format!("{}{}", self.base_url, request.path);

        // RFC 7230 §6.1 hop-by-hop headers that must never be forwarded upstream
        // or propagated back to the caller.
        let hop_by_hop: std::collections::HashSet<&str> = [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "te",
            "trailers",
            "transfer-encoding",
            "upgrade",
        ]
        .iter()
        .copied()
        .collect();

        // Collect connection-header-nominated tokens from the request.
        let connection_tokens: std::collections::HashSet<String> = request
            .headers
            .get("connection")
            .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let mut req_builder = http_client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("anthropic-beta", ANTHROPIC_BETA_OAUTH)
            .body(request.body.clone());

        // Forward caller headers, skipping hop-by-hop and auth headers.
        for (k, v) in &request.headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length"
            ) {
                continue;
            }
            if hop_by_hop.contains(key_lower.as_str()) {
                continue;
            }
            if connection_tokens.contains(&key_lower) {
                continue;
            }
            req_builder = req_builder.header(k.as_str(), v.as_str());
        }

        let resp = req_builder
            .send()
            .await
            .map_err(|e| RestAdapterError::UpstreamError {
                status: 502,
                body: e.to_string(),
            })?;

        let status = resp.status().as_u16();

        // Filter hop-by-hop headers from the upstream response before returning.
        let response_connection_tokens: std::collections::HashSet<String> = resp
            .headers()
            .get("connection")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let mut headers = BTreeMap::new();
        for (k, v) in resp.headers() {
            let key_lower = k.as_str().to_lowercase();
            if hop_by_hop.contains(key_lower.as_str()) {
                continue;
            }
            if response_connection_tokens.contains(&key_lower) {
                continue;
            }
            if let Ok(val) = v.to_str() {
                headers.insert(k.as_str().to_string(), val.to_string());
            }
        }
        let body = resp
            .bytes()
            .await
            .map_err(|e| RestAdapterError::UpstreamError {
                status: 502,
                body: e.to_string(),
            })?
            .to_vec();

        Ok(ProxyResponse {
            status,
            headers,
            body,
        })
    }

    /// Forward `request` to the Anthropic API as an SSE stream.
    ///
    /// Detects streaming intent via `Accept: text/event-stream` in `request.headers`.
    /// Returns the upstream HTTP status alongside a `BoxStream` of raw `Bytes` chunks
    /// (the SSE wire bytes — not parsed or re-framed).
    ///
    /// The caller is responsible for attaching `Content-Type: text/event-stream`,
    /// `Cache-Control: no-cache`, and `Connection: keep-alive` to the HTTP response,
    /// and for driving the stream to completion before releasing the associated
    /// [`SeatLease`].
    ///
    /// ADR-0083 Tier-3: this method never panics on the request path. All errors
    /// that occur before the first byte is received are returned as `Err(...)`; errors
    /// that occur mid-stream are surfaced as `Err` items in the returned `BoxStream`.
    pub async fn proxy_stream(
        &self,
        http_client: &reqwest::Client,
        access_token: &str,
        request: ProxyRequest,
    ) -> Result<(u16, BoxStream<Result<Bytes, RestAdapterError>>), RestAdapterError> {
        let url = format!("{}{}", self.base_url, request.path);

        // RFC 7230 §6.1 hop-by-hop headers — never forwarded.
        let hop_by_hop: std::collections::HashSet<&str> = [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "te",
            "trailers",
            "transfer-encoding",
            "upgrade",
        ]
        .iter()
        .copied()
        .collect();

        let connection_tokens: std::collections::HashSet<String> = request
            .headers
            .get("connection")
            .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let mut req_builder = http_client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("anthropic-beta", ANTHROPIC_BETA_OAUTH)
            // Always set the SSE accept header for stream requests.
            .header("Accept", "text/event-stream")
            .body(request.body);

        for (k, v) in &request.headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length" | "accept"
            ) {
                continue;
            }
            if hop_by_hop.contains(key_lower.as_str()) {
                continue;
            }
            if connection_tokens.contains(&key_lower) {
                continue;
            }
            req_builder = req_builder.header(k.as_str(), v.as_str());
        }

        let resp = req_builder
            .send()
            .await
            .map_err(|e| RestAdapterError::UpstreamError {
                status: 502,
                body: e.to_string(),
            })?;

        let status = resp.status().as_u16();

        // Map the reqwest byte stream errors to RestAdapterError.
        use futures::StreamExt as _;
        let byte_stream: BoxStream<Result<Bytes, RestAdapterError>> =
            Box::pin(resp.bytes_stream().map(|r| {
                r.map_err(|e| RestAdapterError::UpstreamError {
                    status: 502,
                    body: e.to_string(),
                })
            }));

        Ok((status, byte_stream))
    }
}

// ---------------------------------------------------------------------------
// Stage-7 — SseStreamWithLease
// ---------------------------------------------------------------------------

/// Wraps a raw SSE `BoxStream` and holds the [`SeatLease`] alive until the
/// stream completes or the client disconnects.
///
/// - When the inner stream signals `Poll::Ready(None)` (clean end), the lease
///   is completed with [`SeatOutcome::Ok`].
/// - When a mid-stream error is observed, the lease is completed with
///   [`SeatOutcome::ServerError5xx`].
/// - When the struct is dropped before `Poll::Ready(None)` (client disconnect
///   or stream abandoned), the [`SeatLease`] `Drop` impl fires with the
///   `Released` fallback, satisfying the kernel reservation invariant.
pub struct SseStreamWithLease {
    inner: BoxStream<Result<Bytes, RestAdapterError>>, // data_class: INTERNAL_ONLY
    lease: Option<SeatLease>,                          // data_class: INTERNAL_ONLY
    errored: bool,                                     // data_class: INTERNAL_ONLY
}

impl SseStreamWithLease {
    /// Construct with a stream and its associated lease.
    pub fn new(inner: BoxStream<Result<Bytes, RestAdapterError>>, lease: SeatLease) -> Self {
        Self {
            inner,
            lease: Some(lease),
            errored: false,
        }
    }

    /// Complete the lease with the given outcome and drop it so the kernel
    /// releases the seat immediately.
    fn complete_lease(&mut self, outcome: SeatOutcome) {
        if let Some(lease) = self.lease.take() {
            let _ = lease.complete(outcome, Instant::now());
        }
    }
}

impl Stream for SseStreamWithLease {
    type Item = Result<Bytes, RestAdapterError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                // Clean end of stream — complete with Ok.
                if !self.errored {
                    self.complete_lease(SeatOutcome::Ok);
                }
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => {
                // Mid-stream error — complete with ServerError5xx.
                self.errored = true;
                self.complete_lease(SeatOutcome::ServerError5xx);
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
        }
    }
}

// ---------------------------------------------------------------------------
// D2 — axum router
// ---------------------------------------------------------------------------

/// Cached access token with its expiry wall-clock time.
#[derive(Clone, Debug)]
pub struct CachedToken {
    pub access_token: String, // data_class: INTERNAL_ONLY
    pub expires_at: Instant,  // data_class: INTERNAL_ONLY
}

/// Singleflight coalescer (legacy sync version, kept for backward compat with
/// existing tests in d3_refresh_singleflight.rs). New code uses
/// [`UpstreamOAuthSingleflight`] which is fully async.
///
/// In-flight refresh state for a single token handle.
enum RefreshState {
    /// Refresh is in progress; waiters block on the [`std::sync::Condvar`].
    InFlight,
    /// Refresh completed successfully.
    Done(CachedToken),
    /// Refresh failed with this error message.
    Failed(String),
}

/// Shared state slot for a single in-flight token refresh.
type FlightSlot = Arc<(Mutex<RefreshState>, std::sync::Condvar)>; // data_class: INTERNAL_ONLY

/// Singleflight coalescer for token refreshes (sync, blocking). Ensures at
/// most one in-flight token exchange per handle at any time. Kept for
/// backward-compatibility with the Stage-5 singleflight tests.
pub struct TokenRefreshSingleflight {
    flights: Mutex<HashMap<String, FlightSlot>>, // data_class: INTERNAL_ONLY
}

impl TokenRefreshSingleflight {
    pub fn new() -> Self {
        Self {
            flights: Mutex::new(HashMap::new()),
        }
    }

    /// Refresh (or wait for an in-flight refresh) for `handle`.
    ///
    /// `do_refresh` is called exactly once per handle per flight. All other
    /// concurrent callers for the same handle block until that call returns and
    /// then receive the same result.
    pub fn refresh_or_wait<F>(&self, handle: &str, do_refresh: F) -> Result<CachedToken, String>
    where
        F: FnOnce() -> Result<CachedToken, String>,
    {
        let (slot, is_leader) = {
            let mut map = self.flights.lock().unwrap();
            if let Some(existing) = map.get(handle) {
                (Arc::clone(existing), false)
            } else {
                let slot: Arc<(Mutex<RefreshState>, std::sync::Condvar)> = Arc::new((
                    Mutex::new(RefreshState::InFlight),
                    std::sync::Condvar::new(),
                ));
                map.insert(handle.to_string(), Arc::clone(&slot));
                (slot, true)
            }
        };

        let (state_lock, cvar) = slot.as_ref();

        if is_leader {
            let result = do_refresh();
            {
                let mut state = state_lock.lock().unwrap();
                *state = match &result {
                    Ok(tok) => RefreshState::Done(tok.clone()),
                    Err(e) => RefreshState::Failed(e.clone()),
                };
            }
            cvar.notify_all();
            self.flights.lock().unwrap().remove(handle);
            result
        } else {
            let state = cvar
                .wait_while(state_lock.lock().unwrap(), |s| {
                    matches!(s, RefreshState::InFlight)
                })
                .unwrap();
            match &*state {
                RefreshState::Done(tok) => Ok(tok.clone()),
                RefreshState::Failed(e) => Err(e.clone()),
                RefreshState::InFlight => {
                    Err("singleflight: unexpected InFlight after wait".to_string())
                }
            }
        }
    }
}

impl Default for TokenRefreshSingleflight {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared application state threaded through the axum router handlers.
pub struct AppState {
    /// Pool is Arc<Mutex<...>> so SeatLease can hold a weak back-reference.
    pub pool: Arc<Mutex<SubscriptionPool>>, // data_class: INTERNAL_ONLY
    pub gate: Arc<dyn AuthzGate + Send + Sync>, // data_class: INTERNAL_ONLY
    pub sink: Arc<dyn EventSink + Send + Sync>, // data_class: INTERNAL_ONLY
    pub secret_store: Arc<dyn OpenBaoSecretStore>, // data_class: INTERNAL_ONLY
    pub anthropic_base_url: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                    // data_class: INTERNAL_ONLY
    /// D3 singleflight coalescer (sync, legacy). Kept for test compat.
    pub token_singleflight: Arc<TokenRefreshSingleflight>, // data_class: INTERNAL_ONLY
    /// Shared async reqwest::Client — amortizes TLS handshakes + keep-alive
    /// across all proxy requests (Item 1). ADR-0090 blessed HTTP backbone.
    pub http_client: Arc<reqwest::Client>, // data_class: INTERNAL_ONLY
}

impl AppState {
    /// Build an `AppState` with a freshly constructed shared `reqwest::Client`.
    pub fn new(
        pool: Arc<Mutex<SubscriptionPool>>,
        gate: Arc<dyn AuthzGate + Send + Sync>,
        sink: Arc<dyn EventSink + Send + Sync>,
        secret_store: Arc<dyn OpenBaoSecretStore>,
        anthropic_base_url: String,
        tenant_id: TenantId,
    ) -> Result<Self, reqwest::Error> {
        let http_client = Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
        );
        Ok(Self {
            pool,
            gate,
            sink,
            secret_store,
            anthropic_base_url,
            tenant_id,
            token_singleflight: Arc::new(TokenRefreshSingleflight::new()),
            http_client,
        })
    }
}

/// Maximum request body size: 1 MiB. Requests exceeding this limit receive
/// HTTP 413 Payload Too Large (enforced by axum [`DefaultBodyLimit`]).
const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB

/// Build the axum [`Router`] for the cloud-intelligence REST adapter.
///
/// Routes:
/// - `POST /v1/messages` — OAuth-gated reverse proxy to Anthropic API.
/// - `GET  /healthz`     — liveness probe.
/// - `GET  /metrics`     — placeholder Prometheus text exposition.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/messages", post(handle_proxy))
        .route("/healthz", get(handle_healthz))
        .route("/metrics", get(handle_metrics))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}

/// POST /v1/messages handler.
async fn handle_proxy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Extract agent-id from x-agent-id header.
    let agent_id_str = match headers.get("x-agent-id").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => {
            return (StatusCode::BAD_REQUEST, "missing x-agent-id header").into_response();
        }
    };
    let agent_id = match AgentId::new(agent_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "invalid x-agent-id header").into_response();
        }
    };

    // Acquire a SeatLease (Fix-1: prevents same-seat double-allocation).
    let lease = match SubscriptionPool::lease(
        &state.pool,
        &agent_id,
        state.gate.as_ref(),
        Instant::now(),
    ) {
        Ok(l) => l,
        Err(SubscriptionPoolError::ForbiddenByPolicy) => {
            debug!(agent = %agent_id.as_str(), "seat selection forbidden by policy");
            return StatusCode::FORBIDDEN.into_response();
        }
        Err(SubscriptionPoolError::NoEligibleSeat) => {
            warn!(agent = %agent_id.as_str(), "no eligible seat available");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        Err(e) => {
            warn!(error = ?e, "pool lease error");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let seat_id = lease.seat_id().clone();

    // Build a ProxyRequest from the incoming axum request.
    let mut proxy_headers = BTreeMap::new();
    for (k, v) in headers.iter() {
        if let Ok(val) = v.to_str() {
            proxy_headers.insert(k.as_str().to_string(), val.to_string());
        }
    }
    let proxy_request = ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers: proxy_headers,
        body: body.to_vec(),
        tenant_id: state.tenant_id.clone(),
    };

    // Convention: handle = "<tenant_id>/<seat_id>".
    let refresh_handle = format!("{}/{}", state.tenant_id.as_str(), seat_id.as_str());

    // Build the async adapter (no spawn_blocking — Item 1).
    let adapter = AnthropicAdapter::with_base_url(
        ArcSecretStore {
            inner: Arc::clone(&state.secret_store),
        },
        state.anthropic_base_url.clone(),
    );

    // Detect SSE streaming intent from the Accept header.
    let wants_sse = proxy_request
        .headers
        .get("accept")
        .map(|v| v.to_lowercase().contains("text/event-stream"))
        .unwrap_or(false);

    if wants_sse {
        // --- SSE streaming path ---
        // Obtain access token first (refresh may fail before we stream anything).
        let access_token = match adapter
            .refresh_token(&state.http_client, &refresh_handle)
            .await
        {
            Ok(t) => t,
            Err(_) => {
                let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        match adapter
            .proxy_stream(&state.http_client, &access_token, proxy_request)
            .await
        {
            Ok((upstream_status, byte_stream)) => {
                // Wrap the stream with the lease so the seat is held until the
                // response body is fully consumed (or client disconnects).
                let lease_stream = SseStreamWithLease::new(byte_stream, lease);
                let body = Body::from_stream(lease_stream);
                axum::response::Response::builder()
                    .status(upstream_status)
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .header("connection", "keep-alive")
                    .body(body)
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Err(RestAdapterError::OAuthRefreshFailed(_)) => {
                let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Err(e) => {
                warn!(error = ?e, "sse proxy_stream error before first byte");
                let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
    } else {
        // --- Non-streaming (one-shot JSON) path — unchanged from Stage-6 ---
        let result = adapter
            .proxy(&state.http_client, &proxy_request, &refresh_handle)
            .await;

        match result {
            Ok(resp) => {
                // Success — complete lease with Ok outcome.
                let _ = lease.complete(SeatOutcome::Ok, Instant::now());

                // Emit event (non-fatal).
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let event = LlmGatewayEvent {
                    request_id: format!("req-{now_ms}"),
                    tenant_id: state.tenant_id.clone(),
                    agent_id,
                    seat_id: seat_id.clone(),
                    provider: Provider::Anthropic,
                    model: String::new(),
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    ms_latency: 0,
                    status: oya_cloud_intelligence_kernel::EventStatus::Ok,
                    timestamp_unix_ms: now_ms,
                };
                state.sink.emit(event);

                // Build axum response.
                let mut builder = axum::response::Response::builder().status(resp.status);
                for (k, v) in &resp.headers {
                    if let (Ok(name), Ok(value)) = (
                        HeaderName::from_bytes(k.as_bytes()),
                        HeaderValue::from_str(v),
                    ) {
                        builder = builder.header(name, value);
                    }
                }
                builder
                    .body(Body::from(resp.body))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Err(RestAdapterError::OAuthRefreshFailed(_)) => {
                let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Err(RestAdapterError::UpstreamError { status: 429, .. }) => {
                let _ = lease.complete(SeatOutcome::RateLimited429, Instant::now());
                StatusCode::TOO_MANY_REQUESTS.into_response()
            }
            Err(RestAdapterError::UpstreamError {
                status,
                body: err_body,
            }) => {
                let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
                (
                    StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
                    err_body,
                )
                    .into_response()
            }
            Err(e) => {
                warn!(error = ?e, "proxy error");
                let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
    }
}

/// GET /healthz handler — liveness probe.
async fn handle_healthz() -> impl IntoResponse {
    StatusCode::OK
}

/// GET /metrics handler — placeholder Prometheus text exposition.
async fn handle_metrics() -> impl IntoResponse {
    const METRICS_BODY: &str = "\
# HELP oya_cloud_intelligence_up Gateway up\n\
# TYPE oya_cloud_intelligence_up gauge\n\
oya_cloud_intelligence_up 1\n";
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        METRICS_BODY,
    )
}

// ---------------------------------------------------------------------------
// Internal: ArcSecretStore adaptor
// ---------------------------------------------------------------------------
// Wraps an Arc<dyn OpenBaoSecretStore> so AnthropicAdapter can own it.

struct ArcSecretStore {
    inner: Arc<dyn OpenBaoSecretStore>, // data_class: INTERNAL_ONLY
}

impl OpenBaoSecretStore for ArcSecretStore {
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError> {
        self.inner.fetch_refresh_token(handle)
    }

    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError> {
        self.inner.store_refresh_token(handle, plaintext)
    }
}
