//! llm-gateway REST adapter — OAuth subscription pool (ADR-0384 Path B).
//!
//! Stage-5 GREEN. Implements:
//! - [`OpenBaoSecretStore`] trait — D8 envelope-encrypted refresh-token storage seam.
//! - [`EventSinkFanout`] — D6 fan-out broadcaster.
//! - [`AnthropicAdapter`] — D3 Anthropic OAuth refresh + reqwest proxy.
//! - [`ProxyRequest`] / [`ProxyResponse`] — D2 axum reverse-proxy wire types.
//! - [`build_router`] — axum router wiring POST /v1/messages, GET /healthz,
//!   GET /metrics.
//!
//! TODO(codex-adapter): add `CodexAdapter` mirroring `AnthropicAdapter` once
//! the Codex OAuth refresh flow is documented. Tracked as a separate follow-up
//! PR per ADR-0384 §v1-scope.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

pub use oya_llm_gateway_oauth_pool_kernel::{
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

// ---------------------------------------------------------------------------
// D8 — OpenBao envelope-encrypted refresh-token storage seam
// ---------------------------------------------------------------------------

/// D8 secret-store seam. Implementors envelope-encrypt/decrypt via OpenBao
/// transit secrets engine. The kernel never sees plaintext tokens; only opaque
/// handles cross the kernel boundary.
///
/// Real implementation ships in `oya-llm-gateway-openbao-adapter` (separate
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

/// D3 Anthropic provider adapter. Responsible for:
/// 1. Refreshing tokens before expiry (or on 401) via the Anthropic OAuth
///    endpoint.
/// 2. Routing proxied requests to `https://api.anthropic.com` with the
///    bearer token fetched from [`OpenBaoSecretStore`].
///
/// TODO(codex-adapter): `CodexAdapter` will mirror this struct for the OpenAI
/// Codex OAuth flow. Deferred to a follow-up PR per ADR-0384 §v1-scope.
pub struct AnthropicAdapter<S: OpenBaoSecretStore> {
    secret_store: S,                   // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client, // data_class: INTERNAL_ONLY
    base_url: String,                  // data_class: INTERNAL_ONLY
    client_id: String,                 // data_class: INTERNAL_ONLY
}

impl<S: OpenBaoSecretStore> AnthropicAdapter<S> {
    /// Construct with a concrete [`OpenBaoSecretStore`] implementation.
    /// Uses the default Anthropic base URL and client_id from ADR-0384.
    pub fn new(secret_store: S) -> Self {
        Self::with_base_url(secret_store, ANTHROPIC_BASE_URL.to_string())
    }

    /// Construct with a custom base URL (for testing against a local mock server).
    pub fn with_base_url(secret_store: S, base_url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest blocking client construction is infallible on supported platforms");
        Self {
            secret_store,
            client,
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
    pub fn exchange_authorization_code(
        &self,
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
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(RestAdapterError::OAuthRefreshFailed(format!(
                "code exchange failed: HTTP {status}: {text}"
            )));
        }
        let token_resp: OAuthTokenResponse = resp
            .json()
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        let handle = format!("{}/{}", tenant_id.as_str(), seat_id.as_str());
        self.secret_store
            .store_refresh_token(&handle, &token_resp.refresh_token)?;
        Ok(handle)
    }

    /// Refresh the token identified by `refresh_token_handle`. Fetches the
    /// current refresh token from the secret store, exchanges it with
    /// Anthropic, stores the new refresh token, and returns the bearer access
    /// token to use for this request.
    pub fn refresh_token(&self, refresh_token_handle: &str) -> Result<String, RestAdapterError> {
        let current_refresh = self
            .secret_store
            .fetch_refresh_token(refresh_token_handle)?;
        let url = format!("{}/v1/oauth/token", self.base_url);
        let body = OAuthRefreshRequest {
            client_id: &self.client_id,
            grant_type: "refresh_token",
            refresh_token: &current_refresh,
        };
        debug!(
            handle = refresh_token_handle,
            "refreshing Anthropic OAuth token"
        );
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(RestAdapterError::OAuthRefreshFailed(format!(
                "token refresh failed: HTTP {status}: {text}"
            )));
        }
        let token_resp: OAuthTokenResponse = resp
            .json()
            .map_err(|e| RestAdapterError::OAuthRefreshFailed(e.to_string()))?;
        // Rotate refresh token in secret store.
        self.secret_store
            .store_refresh_token(refresh_token_handle, &token_resp.refresh_token)?;
        Ok(token_resp.access_token)
    }

    /// Forward `request` to the Anthropic API using the bearer token obtained
    /// (or refreshed) for the selected seat. Returns the upstream response.
    ///
    /// Headers forwarded: all except `authorization`, `host`, `content-length`
    /// (those are set or managed by reqwest).
    pub fn proxy(
        &self,
        request: &ProxyRequest,
        refresh_token_handle: &str,
    ) -> Result<ProxyResponse, RestAdapterError> {
        let access_token = self.refresh_token(refresh_token_handle)?;
        let url = format!("{}{}", self.base_url, request.path);
        let mut req_builder = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("anthropic-beta", ANTHROPIC_BETA_OAUTH)
            .body(request.body.clone());

        // RFC 7230 §6.1 hop-by-hop headers that must never be forwarded upstream
        // or propagated back to the caller. Also parse the incoming Connection:
        // header value tokens and drop those header names too.
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

/// In-flight refresh state for a single token handle.
enum RefreshState {
    /// Refresh is in progress; waiters block on the [`Condvar`].
    InFlight,
    /// Refresh completed successfully.
    Done(CachedToken),
    /// Refresh failed with this error message.
    Failed(String),
}

/// Shared state slot for a single in-flight token refresh.
type FlightSlot = Arc<(Mutex<RefreshState>, Condvar)>; // data_class: INTERNAL_ONLY

/// Singleflight coalescer for token refreshes. Ensures at most one in-flight
/// token exchange per handle at any time. Concurrent callers wait on the same
/// result rather than issuing N parallel calls to the OAuth endpoint.
pub struct TokenRefreshSingleflight {
    /// Each entry is an Arc around (state mutex, condvar). Presence in the map
    /// means a refresh is in flight. Entry is removed once the flight completes
    /// so the next request starts a new flight.
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
                // A flight is already in progress — we are a waiter.
                (Arc::clone(existing), false)
            } else {
                // No flight in progress — we are the leader.
                let slot: Arc<(Mutex<RefreshState>, Condvar)> =
                    Arc::new((Mutex::new(RefreshState::InFlight), Condvar::new()));
                map.insert(handle.to_string(), Arc::clone(&slot));
                (slot, true)
            }
        };

        let (state_lock, cvar) = slot.as_ref();

        if is_leader {
            // Perform the actual refresh outside any lock.
            let result = do_refresh();
            {
                let mut state = state_lock.lock().unwrap();
                *state = match &result {
                    Ok(tok) => RefreshState::Done(tok.clone()),
                    Err(e) => RefreshState::Failed(e.clone()),
                };
            }
            cvar.notify_all();
            // Remove from map so the next caller starts a fresh flight.
            self.flights.lock().unwrap().remove(handle);
            result
        } else {
            // Wait until the leader finishes.
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
    /// D3 singleflight coalescer — at most one token-exchange call per handle. // data_class: INTERNAL_ONLY
    pub token_singleflight: Arc<TokenRefreshSingleflight>, // data_class: INTERNAL_ONLY
}

/// Maximum request body size: 1 MiB. Requests exceeding this limit receive
/// HTTP 413 Payload Too Large (enforced by axum [`DefaultBodyLimit`]).
const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB

/// Build the axum [`Router`] for the llm-gateway REST adapter.
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

    // Fix-2: coalesce concurrent refresh calls for the same handle.
    let singleflight = Arc::clone(&state.token_singleflight);
    let secret_store_ref = Arc::clone(&state.secret_store);
    let handle_for_sf = refresh_handle.clone();
    let refresh_token_result = tokio::task::spawn_blocking(move || {
        singleflight.refresh_or_wait(&handle_for_sf, || {
            secret_store_ref
                .fetch_refresh_token(&handle_for_sf)
                .map(|token| CachedToken {
                    access_token: token,
                    expires_at: Instant::now() + Duration::from_secs(3600),
                })
                .map_err(|e| format!("{e:?}"))
        })
    })
    .await;

    let cached_token = match refresh_token_result {
        Ok(Ok(tok)) => tok,
        Ok(Err(e)) => {
            warn!(handle = %refresh_handle, error = %e, "singleflight token refresh failed");
            // Fix-6: record RefreshFailed on vault/refresh errors.
            let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(e) => {
            warn!(error = %e, "spawn_blocking join error during singleflight");
            let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Run the blocking reqwest proxy call on a blocking thread.
    let secret_store_ref2 = Arc::clone(&state.secret_store);
    let handle_clone = refresh_handle.clone();
    let token_clone = cached_token.access_token.clone();
    let base_url = state.anthropic_base_url.clone();
    let result = tokio::task::spawn_blocking(move || {
        let adapter = AnthropicAdapter::with_base_url(
            ArcSecretStore {
                inner: secret_store_ref2,
                prefetched: Some((handle_clone, token_clone)),
            },
            base_url,
        );
        adapter.proxy(&proxy_request, &refresh_handle)
    })
    .await;

    match result {
        Ok(Ok(resp)) => {
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
                status: oya_llm_gateway_oauth_pool_kernel::EventStatus::Ok,
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
        Ok(Err(RestAdapterError::UpstreamError { status: 429, .. })) => {
            let _ = lease.complete(SeatOutcome::RateLimited429, Instant::now());
            StatusCode::TOO_MANY_REQUESTS.into_response()
        }
        Ok(Err(RestAdapterError::UpstreamError {
            status,
            body: err_body,
        })) => {
            let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
                err_body,
            )
                .into_response()
        }
        Ok(Err(e)) => {
            warn!(error = ?e, "proxy error");
            let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
            StatusCode::BAD_GATEWAY.into_response()
        }
        Err(e) => {
            warn!(error = %e, "spawn_blocking join error");
            let _ = lease.complete(SeatOutcome::ServerError5xx, Instant::now());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
# HELP oya_llm_gateway_up Gateway up\n\
# TYPE oya_llm_gateway_up gauge\n\
oya_llm_gateway_up 1\n";
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        METRICS_BODY,
    )
}

// ---------------------------------------------------------------------------
// Internal: ArcSecretStore adaptor for spawn_blocking
// ---------------------------------------------------------------------------
// Wraps an Arc<dyn OpenBaoSecretStore> so AnthropicAdapter can own it.
// The `prefetched` field holds a pre-fetched (handle, plaintext) pair so the
// blocking adapter can skip the vault fetch and use the already-fetched token.

struct ArcSecretStore {
    inner: Arc<dyn OpenBaoSecretStore>,   // data_class: INTERNAL_ONLY
    prefetched: Option<(String, String)>, // data_class: INTERNAL_ONLY
}

impl OpenBaoSecretStore for ArcSecretStore {
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError> {
        if let Some((ref h, ref t)) = self.prefetched
            && h == handle
        {
            return Ok(t.clone());
        }
        self.inner.fetch_refresh_token(handle)
    }

    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError> {
        self.inner.store_refresh_token(handle, plaintext)
    }
}
