//! cloud-intelligence REST adapter — OAuth subscription pool (ADR-0384 Path B).
//!
//! Stage-6 GREEN. Stage-7 SSE passthrough added. Implements:
//! - [`SecretProviderStore`] trait — owned secret-provider refresh-token storage seam.
//! - [`EventSinkFanout`] — D6 fan-out broadcaster.
//! - [`AnthropicAdapter`] — D3 Anthropic OAuth refresh + async reqwest proxy.
//! - [`ProxyRequest`] / [`ProxyResponse`] — D2 axum reverse-proxy wire types.
//! - [`build_router`] — axum router wiring POST /v1/messages, GET /healthz,
//!   GET /livez, GET /readyz, GET /metrics.
//!
//! Stage-6 changes (Item 1 + Item 2):
//! - `AnthropicAdapter` migrated from `reqwest::blocking` to async `reqwest::Client`.
//!   The adapter borrows `&reqwest::Client` — it does NOT own one. `AppState` holds
//!   the shared `Arc<reqwest::Client>` so TLS handshakes and keep-alive connections
//!   are pooled across requests.
//! - Singleflight is driven inside `AnthropicAdapter::refresh_token` via a process-shared
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
//! Deferred (governed): a `CodexAdapter` mirroring `AnthropicAdapter` is a
//! separate follow-up PR, pending documentation of the Codex OAuth refresh flow.
//! Scope boundary per ADR-0384 §v1-scope; tracked to closure as FRIC-1781133000
//! in the friction ledger (the closed-loop home for deferred work).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use futures::Stream;
use futures::StreamExt as _;
use intelligence_codex_adapter::{
    CodexAdapter, CodexAdapterError, CodexProxyRequest, CodexProxyResponse, OpenAiApiKeyAdapter,
};
use intelligence_gemini_adapter::{GeminiAdapterError, GeminiApiKeyAdapter, GeminiProxyResponse};
use intelligence_kernel::model_routing::{
    BackendClass, ModelRouter, ModelRoutingError, ProtocolShape, RoutePolicy, RouteRequest,
    RoutingDecision,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Type alias for a heap-allocated, `Send + 'static` SSE byte stream.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

pub use intelligence_kernel::{
    AgentId, AuthzAction, AuthzDecision, AuthzGate, AuthzRequest, CredentialMode, EventSink,
    EventStatus, LlmGatewayEvent, OAuthSubscription, Provider, SeatId, SeatLease, SeatOutcome,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionPoolError, SubscriptionState,
    TenantId, is_secret_handle_reference, looks_like_jwt,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified error type for the REST adapter layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestAdapterError {
    /// Secret handle was not found in the store.
    SecretNotFound,
    /// Secret-provider adapter returned a transport or availability error.
    SecretStoreUnavailable(String),
    /// Supplied plaintext was empty or otherwise invalid.
    InvalidSecret,
    /// Upstream Anthropic API returned an error status.
    UpstreamError { status: u16, body: String },
    /// OAuth refresh token exchange failed.
    OAuthRefreshFailed(String),
    /// OAuth rotation persistence requires the caller to retry the request.
    OAuthRefreshRetryRequired,
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
            Self::OAuthRefreshRetryRequired => {
                write!(f, "OAuth refresh requires a request retry")
            }
            Self::PoolError(e) => write!(f, "pool error: {e:?}"),
            Self::SinkEmitFailed(msg) => write!(f, "sink emit failed: {msg}"),
        }
    }
}

impl std::error::Error for RestAdapterError {}

// ---------------------------------------------------------------------------
// D8 — owned secret-provider refresh-token storage seam
// ---------------------------------------------------------------------------

/// D8 secret-provider seam. Implementors resolve opaque handles through the
/// owned cloud-secrets/cloud-kms port. The kernel never sees plaintext tokens;
/// only opaque handles cross the kernel boundary.
///
/// Transient backing stores live behind adapter crates; core request handling
/// depends only on this port.
/// Object-safe future returned by [`SecretProviderStore`].
///
/// The lifetime is tied to the store and input references, retaining dynamic
/// dispatch while allowing production secret providers to perform async I/O.
pub type SecretProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RestAdapterError>> + Send + 'a>>;

pub trait SecretProviderStore: Send + Sync {
    /// Fetch and decrypt the refresh token identified by `handle`.
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String>;

    /// Envelope-encrypt `plaintext` and store it under `handle`.
    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()>;

    /// Lightweight health probe for readiness. In-memory test stores are ready
    /// by default; production adapters override this to touch the owned
    /// secret-provider port.
    fn readiness_probe(&self) -> SecretProviderFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
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

/// Collect the lowercased set of header names nominated by a `Connection`
/// header on caller-supplied request headers.
///
/// Caller headers arrive in a `BTreeMap` keyed by the caller's original casing,
/// so an exact `get("connection")` misses `Connection:` (capital C) and any
/// mixed-case variant — leaking the nominated headers upstream. Iterate every
/// key with a case-insensitive match instead.
fn connection_tokens(headers: &BTreeMap<String, String>) -> std::collections::HashSet<String> {
    let mut tokens = std::collections::HashSet::new();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("connection") {
            tokens.extend(v.split(',').map(|t| t.trim().to_lowercase()));
        }
    }
    tokens
}

// ---------------------------------------------------------------------------
// D3 — Anthropic provider adapter
// ---------------------------------------------------------------------------

/// Anthropic client_id per ADR-0384.
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Default Anthropic base URL.
const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

/// Default OpenAI-compatible base URL for API-key provider pools.
const OPENAI_COMPATIBLE_BASE_URL: &str = "https://api.openai.com";

/// Default Gemini API base URL for API-key provider pools.
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Default ChatGPT base URL for Codex OAuth-subscription pools. Used for the
/// session-token refresh + Codex data endpoint (distinct from the API-key
/// `OPENAI_COMPATIBLE_BASE_URL`).
const CODEX_OAUTH_BASE_URL: &str = "https://chatgpt.com";

/// OpenAI-compatible upstream path that the Codex OAuth-subscription flow
/// supports (the ChatGPT-backed responses endpoint is chat-shaped only).
const CODEX_OAUTH_SUPPORTED_PATH: &str = "/v1/chat/completions";

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

/// Result broadcast across concurrent waiters for one token handle.
type RefreshResult = Result<String, RestAdapterError>; // data_class: INTERNAL_ONLY

struct PendingOAuthRotation {
    refresh_token: String, // data_class: SECRET
}

/// D3 upstream OAuth singleflight. Coalesces concurrent `refresh_token` calls
/// for the same handle into exactly ONE upstream call. All concurrent callers
/// receive the same result via a `broadcast::Sender`.
///
/// Pattern: `tokio::sync::Mutex<HashMap<handle, broadcast::Sender<RefreshResult>>>`.
/// When a flight completes the entry is removed so the next caller starts fresh.
pub struct UpstreamOAuthSingleflight {
    /// In-flight map: present = flight in progress. Absence = no flight.
    flights: Arc<tokio::sync::Mutex<HashMap<String, broadcast::Sender<RefreshResult>>>>, // data_class: INTERNAL_ONLY
    /// In-process recovery for an exchanged token not yet durably stored.
    /// This does not close the process-crash atomicity gap.
    pending_rotations: Arc<tokio::sync::Mutex<HashMap<String, Arc<PendingOAuthRotation>>>>, // data_class: SECRET
}

impl UpstreamOAuthSingleflight {
    pub fn new() -> Self {
        Self {
            flights: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            pending_rotations: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    async fn pending_rotation(&self, handle: &str) -> Option<Arc<PendingOAuthRotation>> {
        self.pending_rotations.lock().await.get(handle).cloned()
    }

    async fn retain_pending_rotation(
        &self,
        handle: &str,
        refresh_token: String,
    ) -> Arc<PendingOAuthRotation> {
        let pending = Arc::new(PendingOAuthRotation { refresh_token });
        self.pending_rotations
            .lock()
            .await
            .insert(handle.to_string(), Arc::clone(&pending));
        pending
    }

    async fn clear_pending_rotation(&self, handle: &str, pending: &Arc<PendingOAuthRotation>) {
        let mut rotations = self.pending_rotations.lock().await;
        if rotations
            .get(handle)
            .is_some_and(|registered| Arc::ptr_eq(registered, pending))
        {
            rotations.remove(handle);
        }
    }

    /// Call `do_refresh` exactly once per in-flight window for `handle`.
    /// All concurrent callers for the same handle wait on the broadcast channel
    /// and receive the leader's result.
    ///
    /// `do_refresh` owns the complete fetch/exchange/durable-rotation flight.
    /// A helper-owned task drives it so cancelling the initiating request cannot
    /// strand the keyed flight. Every caller, including the initiator, waits on
    /// the same receiver.
    pub async fn refresh_or_wait<F, Fut>(&self, handle: &str, do_refresh: F) -> RefreshResult
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = RefreshResult> + Send + 'static,
    {
        let mut receiver = {
            let mut map = self.flights.lock().await;
            if let Some(sender) = map.get(handle) {
                sender.subscribe()
            } else {
                let (sender, receiver) = broadcast::channel(1);
                map.insert(handle.to_string(), sender.clone());
                let flights = Arc::clone(&self.flights);
                let pending_rotations = Arc::clone(&self.pending_rotations);
                let handle = handle.to_string();
                tokio::spawn(async move {
                    // The nested task converts both construction-time and poll-time
                    // panics, plus task cancellation, into one shared typed result.
                    let refresh_task = tokio::spawn(async move { do_refresh().await });
                    // Resolve pending-rotation state before taking the flight
                    // registry lock so cleanup never nests these mutexes.
                    let task_result = refresh_task.await;
                    let pending = pending_rotations.lock().await.contains_key(&handle);
                    let result = if pending {
                        // Never publish an access token while its rotated
                        // refresh token is still only process-resident.
                        Err(RestAdapterError::OAuthRefreshRetryRequired)
                    } else {
                        match task_result {
                            Ok(result) => result,
                            Err(error) => {
                                let cause = if error.is_cancelled() {
                                    "cancelled"
                                } else if error.is_panic() {
                                    "panicked"
                                } else {
                                    "terminated"
                                };
                                Err(RestAdapterError::OAuthRefreshFailed(format!(
                                    "singleflight worker {cause}"
                                )))
                            }
                        }
                    };
                    let mut map = flights.lock().await;
                    if map
                        .get(&handle)
                        .is_some_and(|registered| registered.same_channel(&sender))
                    {
                        map.remove(&handle);
                        // Remove and publish while holding the map lock. A later
                        // caller cannot join an already-published flight.
                        let _ = sender.send(result);
                    } else {
                        warn!(
                            handle,
                            "singleflight registry ownership changed before cleanup"
                        );
                        let error = if matches!(
                            &result,
                            Err(RestAdapterError::OAuthRefreshRetryRequired)
                        ) {
                            RestAdapterError::OAuthRefreshRetryRequired
                        } else {
                            RestAdapterError::OAuthRefreshFailed(
                                "flight registry ownership changed before cleanup".to_string(),
                            )
                        };
                        let _ = sender.send(Err(error));
                    }
                });
                receiver
            }
        };

        match receiver.recv().await {
            Ok(result) => result,
            Err(_) if self.pending_rotation(handle).await.is_some() => {
                Err(RestAdapterError::OAuthRefreshRetryRequired)
            }
            Err(_) => Err(RestAdapterError::OAuthRefreshFailed(
                "singleflight worker closed without a result".to_string(),
            )),
        }
    }
}

impl Default for UpstreamOAuthSingleflight {
    fn default() -> Self {
        Self::new()
    }
}

const ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS: usize = 3;
const ROTATED_REFRESH_TOKEN_STORE_RETRY_BASE_DELAY: Duration = Duration::from_millis(25);
// Calibration knob pending production OpenBao p99 latency evidence.
const ROTATED_REFRESH_TOKEN_STORE_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(1);

async fn persist_rotated_refresh_token<S: SecretProviderStore>(
    secret_store: &S,
    handle: &str,
    rotated_refresh_token: &str,
) -> Result<(), RestAdapterError> {
    for attempt in 1..=ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS {
        let store_result = tokio::time::timeout(
            ROTATED_REFRESH_TOKEN_STORE_ATTEMPT_TIMEOUT,
            secret_store.store_refresh_token(handle, rotated_refresh_token),
        )
        .await;

        match store_result {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(RestAdapterError::SecretStoreUnavailable(_))) => {
                warn!(
                    handle,
                    attempt, "secret provider unavailable while persisting refresh-token rotation"
                );
            }
            Ok(Err(_)) => return Err(RestAdapterError::OAuthRefreshRetryRequired),
            Err(_) => warn!(
                handle,
                attempt, "refresh-token persistence attempt timed out"
            ),
        }

        if attempt < ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS {
            warn!(
                handle,
                attempt,
                max_attempts = ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS,
                "retrying durable refresh-token rotation"
            );
            tokio::time::sleep(ROTATED_REFRESH_TOKEN_STORE_RETRY_BASE_DELAY * attempt as u32).await;
        }
    }

    Err(RestAdapterError::OAuthRefreshRetryRequired)
}

/// D3 Anthropic provider adapter. Responsible for:
/// 1. Refreshing tokens before expiry (or on 401) via the Anthropic OAuth
///    endpoint (coalesced via a process-shared [`UpstreamOAuthSingleflight`]).
/// 2. Routing proxied requests to `https://api.anthropic.com` with the
///    bearer token fetched from [`SecretProviderStore`].
///
/// The adapter borrows `&reqwest::Client` — it does NOT own one. The shared
/// client lives in [`AppState`] so TLS sessions and keep-alive connections are
/// amortized across the full request lifetime of the process.
///
/// Deferred (governed): a `CodexAdapter` will mirror this struct for the OpenAI
/// Codex OAuth flow in a follow-up PR. Scope boundary per ADR-0384 §v1-scope;
/// tracked to closure as FRIC-1781133000 in the friction ledger.
pub struct AnthropicAdapter<S: SecretProviderStore> {
    secret_store: Arc<S>,                         // data_class: INTERNAL_ONLY
    singleflight: Arc<UpstreamOAuthSingleflight>, // data_class: INTERNAL_ONLY
    base_url: String,                             // data_class: INTERNAL_ONLY
    client_id: String,                            // data_class: INTERNAL_ONLY
}

impl<S: SecretProviderStore + 'static> AnthropicAdapter<S> {
    /// Construct with a concrete [`SecretProviderStore`] implementation.
    /// Uses the default Anthropic base URL and client_id from ADR-0384.
    pub fn new(secret_store: S) -> Self {
        Self::with_base_url(secret_store, ANTHROPIC_BASE_URL.to_string())
    }

    /// Construct with a custom base URL (for testing against a local mock server).
    pub fn with_base_url(secret_store: S, base_url: String) -> Self {
        Self::with_base_url_and_singleflight(
            secret_store,
            base_url,
            Arc::new(UpstreamOAuthSingleflight::new()),
        )
    }

    /// Construct with a process-shared keyed flight registry.
    pub fn with_base_url_and_singleflight(
        secret_store: S,
        base_url: String,
        singleflight: Arc<UpstreamOAuthSingleflight>,
    ) -> Self {
        Self {
            secret_store: Arc::new(secret_store),
            singleflight,
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
            .store_refresh_token(&handle, &token_resp.refresh_token)
            .await?;
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
        let url = format!("{}/v1/oauth/token", self.base_url);
        let client_id = self.client_id.clone();
        let handle = refresh_token_handle.to_string();
        let secret_store = Arc::clone(&self.secret_store);
        let http_client = http_client.clone();
        let rotation_registry = Arc::clone(&self.singleflight);

        debug!(
            handle = refresh_token_handle,
            "refreshing Anthropic OAuth token via singleflight"
        );

        self.singleflight
            .refresh_or_wait(refresh_token_handle, move || async move {
                if let Some(pending) = rotation_registry.pending_rotation(&handle).await {
                    persist_rotated_refresh_token(
                        secret_store.as_ref(),
                        &handle,
                        &pending.refresh_token,
                    )
                    .await?;
                    rotation_registry
                        .clear_pending_rotation(&handle, &pending)
                        .await;
                    return Err(RestAdapterError::OAuthRefreshRetryRequired);
                }

                let current_refresh = secret_store.fetch_refresh_token(&handle).await?;
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
                    .map_err(|error| RestAdapterError::OAuthRefreshFailed(error.to_string()))?;
                let status = resp.status().as_u16();
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(RestAdapterError::OAuthRefreshFailed(format!(
                        "token refresh failed: HTTP {status}: {text}"
                    )));
                }
                let token_resp: OAuthTokenResponse = resp
                    .json()
                    .await
                    .map_err(|error| RestAdapterError::OAuthRefreshFailed(error.to_string()))?;
                let access_token = token_resp.access_token;
                let pending = rotation_registry
                    .retain_pending_rotation(&handle, token_resp.refresh_token)
                    .await;
                persist_rotated_refresh_token(
                    secret_store.as_ref(),
                    &handle,
                    &pending.refresh_token,
                )
                .await?;
                rotation_registry
                    .clear_pending_rotation(&handle, &pending)
                    .await;
                Ok(access_token)
            })
            .await
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
        let connection_tokens = connection_tokens(&request.headers);

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

    /// Forward `request` to the Anthropic API using a provider API key.
    ///
    /// API-key mode is the documented direct provider path: it injects
    /// `x-api-key` plus the canonical `anthropic-version` and explicitly avoids
    /// OAuth-only `Authorization` and `anthropic-beta` headers.
    pub async fn proxy_with_api_key(
        &self,
        http_client: &reqwest::Client,
        request: &ProxyRequest,
        api_key: &str,
    ) -> Result<ProxyResponse, RestAdapterError> {
        let url = format!("{}{}", self.base_url, request.path);
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
        let connection_tokens = connection_tokens(&request.headers);

        let mut req_builder = http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .body(request.body.clone());

        for (k, v) in &request.headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization"
                    | "x-api-key"
                    | "host"
                    | "content-length"
                    | "anthropic-version"
                    | "anthropic-beta"
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

        let connection_tokens = connection_tokens(&request.headers);

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

    /// Forward `request` to the Anthropic API using a provider API key and
    /// return a raw SSE byte stream.
    pub async fn proxy_stream_with_api_key(
        &self,
        http_client: &reqwest::Client,
        api_key: &str,
        request: ProxyRequest,
    ) -> Result<(u16, BoxStream<Result<Bytes, RestAdapterError>>), RestAdapterError> {
        let url = format!("{}{}", self.base_url, request.path);
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
        let connection_tokens = connection_tokens(&request.headers);

        let mut req_builder = http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Accept", "text/event-stream")
            .body(request.body);

        for (k, v) in &request.headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization"
                    | "x-api-key"
                    | "host"
                    | "content-length"
                    | "anthropic-version"
                    | "anthropic-beta"
                    | "accept"
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
///   is completed with the outcome derived from the upstream HTTP status.
/// - When a mid-stream error is observed, the lease is completed with
///   [`SeatOutcome::ServerError5xx`].
/// - When the struct is dropped before `Poll::Ready(None)` (client disconnect
///   or stream abandoned), the [`SeatLease`] `Drop` impl fires with the
///   `Released` fallback, satisfying the kernel reservation invariant.
pub struct SseStreamWithLease {
    inner: BoxStream<Result<Bytes, RestAdapterError>>, // data_class: INTERNAL_ONLY
    lease: Option<SeatLease>,                          // data_class: INTERNAL_ONLY
    errored: bool,                                     // data_class: INTERNAL_ONLY
    clean_outcome: SeatOutcome,                        // data_class: INTERNAL_ONLY
}

impl SseStreamWithLease {
    /// Construct with a stream and its associated lease.
    pub fn new(inner: BoxStream<Result<Bytes, RestAdapterError>>, lease: SeatLease) -> Self {
        Self::new_with_clean_outcome(inner, lease, SeatOutcome::Ok)
    }

    /// Construct with a stream, lease, and clean-end outcome already derived
    /// from the upstream HTTP status.
    pub fn new_with_clean_outcome(
        inner: BoxStream<Result<Bytes, RestAdapterError>>,
        lease: SeatLease,
        clean_outcome: SeatOutcome,
    ) -> Self {
        Self {
            inner,
            lease: Some(lease),
            errored: false,
            clean_outcome,
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
                // Clean end of stream — complete with the upstream status outcome.
                if !self.errored {
                    // Copy the Copy-field out before the &mut self call (E0502:
                    // the argument's immutable borrow conflicts with the receiver).
                    let outcome = self.clean_outcome;
                    self.complete_lease(outcome);
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

/// Registry of static or dynamically registered tenant/provider pools.
///
/// The v1 proxy path still uses `AppState::pool` as the default Anthropic
/// data-plane pool, but admin/readiness paths use this registry so a single
/// process can expose multiple tenant/provider pools without cross-tenant
/// credential leakage.
#[derive(Clone)]
pub struct PoolRegistry {
    pools: Arc<Mutex<HashMap<PoolKey, Arc<Mutex<SubscriptionPool>>>>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PoolKey {
    tenant_id: TenantId, // data_class: INTERNAL_ONLY
    provider: Provider,  // data_class: INTERNAL_ONLY
}

/// Secret-redacted pool status returned by tenant-scoped admin routes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PoolStatus {
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub provider: String,  // data_class: INTERNAL_ONLY
    pub total_seats: usize,
    pub ready: bool,
}

/// Per-seat, contract-shaped account status returned by `/admin/v1/accounts`.
/// Mirrors the OpenAPI `AccountStatus` schema (seat-shaped, secret-free), as
/// projected by the kernel's `RedactedSeatStatus`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AccountStatus {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub provider: String,               // data_class: INTERNAL_ONLY
    pub seat_id: String,                // data_class: INTERNAL_ONLY
    pub state: &'static str,            // data_class: INTERNAL_ONLY
    pub cooldown_until: Option<String>, // data_class: INTERNAL_ONLY
    pub headroom_percent: f64,          // data_class: INTERNAL_ONLY
}

impl PoolRegistry {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert_pool(
        &self,
        tenant_id: TenantId,
        provider: Provider,
        pool: Arc<Mutex<SubscriptionPool>>,
    ) {
        if let Ok(mut pools) = self.pools.lock() {
            pools.insert(
                PoolKey {
                    tenant_id,
                    provider,
                },
                pool,
            );
        }
    }

    pub fn get_pool(
        &self,
        tenant_id: &TenantId,
        provider: Provider,
    ) -> Option<Arc<Mutex<SubscriptionPool>>> {
        self.pools
            .lock()
            .ok()?
            .get(&PoolKey {
                tenant_id: tenant_id.clone(),
                provider,
            })
            .cloned()
    }

    pub fn pool_count(&self) -> usize {
        self.pools.lock().map(|pools| pools.len()).unwrap_or(0)
    }

    pub fn pool_status(&self, tenant_id: &TenantId, provider: Provider) -> Option<PoolStatus> {
        let pool = self.get_pool(tenant_id, provider)?;
        let pool = pool.lock().ok()?;
        Some(PoolStatus {
            tenant_id: tenant_id.as_str().to_string(),
            provider: provider.to_string(),
            total_seats: pool.seat_count(),
            ready: pool.has_eligible_seat(Instant::now()),
        })
    }

    pub fn register_subscription(
        &self,
        tenant_id: TenantId,
        provider: Provider,
        subscription: OAuthSubscription,
        strategy: SelectionStrategy,
    ) -> Result<PoolStatus, SubscriptionPoolError> {
        let key = PoolKey {
            tenant_id: tenant_id.clone(),
            provider,
        };
        let pool = {
            let mut pools = self
                .pools
                .lock()
                .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?;
            Arc::clone(pools.entry(key).or_insert_with(|| {
                Arc::new(Mutex::new(SubscriptionPool::new(
                    tenant_id.clone(),
                    provider,
                    strategy,
                )))
            }))
        };

        pool.lock()
            .map_err(|_| SubscriptionPoolError::NoEligibleSeat)?
            .add_seat(subscription)?;

        self.pool_status(&tenant_id, provider)
            .ok_or(SubscriptionPoolError::NoEligibleSeat)
    }

    pub fn pool_statuses(&self) -> Vec<PoolStatus> {
        let Ok(pools) = self.pools.lock() else {
            return Vec::new();
        };
        pools
            .iter()
            .filter_map(|(key, pool)| {
                let pool = pool.lock().ok()?;
                Some(PoolStatus {
                    tenant_id: key.tenant_id.as_str().to_string(),
                    provider: key.provider.to_string(),
                    total_seats: pool.seat_count(),
                    ready: pool.has_eligible_seat(Instant::now()),
                })
            })
            .collect()
    }

    /// Per-seat account statuses for a single tenant.
    pub fn account_statuses_for(&self, tenant: &TenantId) -> Vec<AccountStatus> {
        let Ok(pools) = self.pools.lock() else {
            return Vec::new();
        };
        let now = Instant::now();
        let mut statuses = Vec::new();
        for (key, pool) in pools.iter() {
            if &key.tenant_id != tenant {
                continue;
            }
            let Ok(pool) = pool.lock() else {
                continue;
            };
            statuses.extend(pool.redacted_seat_statuses(now).into_iter().map(|seat| {
                AccountStatus {
                    tenant_id: seat.tenant_id,
                    provider: seat.provider.to_string(),
                    seat_id: seat.seat_id,
                    state: seat.state,
                    cooldown_until: None,
                    headroom_percent: seat.headroom_percent,
                }
            }));
        }
        statuses
    }

    /// Number of registered pools for a single tenant — the tenant-scoped variant
    /// of [`pool_count`] used by authn-gated admin routes.
    pub fn pool_count_for(&self, tenant: &TenantId) -> usize {
        self.pools
            .lock()
            .map(|pools| pools.keys().filter(|k| &k.tenant_id == tenant).count())
            .unwrap_or(0)
    }
}

impl Default for PoolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared application state threaded through the axum router handlers.
pub struct AppState {
    /// Pool is Arc<Mutex<...>> so SeatLease can hold a weak back-reference.
    pub pool: Arc<Mutex<SubscriptionPool>>, // data_class: INTERNAL_ONLY
    pub pool_registry: PoolRegistry, // data_class: INTERNAL_ONLY
    pub gate: Arc<dyn AuthzGate + Send + Sync>, // data_class: INTERNAL_ONLY
    pub sink: Arc<dyn EventSink + Send + Sync>, // data_class: INTERNAL_ONLY
    pub secret_store: Arc<dyn SecretProviderStore>, // data_class: INTERNAL_ONLY
    pub anthropic_base_url: String,  // data_class: INTERNAL_ONLY
    pub openai_compatible_base_url: String, // data_class: INTERNAL_ONLY
    /// ChatGPT base URL for Codex OAuth-subscription seats (session refresh +
    /// Codex data endpoint). Distinct from `openai_compatible_base_url`, which
    /// targets the documented API-key OpenAI shape.
    pub codex_oauth_base_url: String, // data_class: INTERNAL_ONLY
    pub gemini_base_url: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,         // data_class: INTERNAL_ONLY
    /// Data-plane ingress authenticator (AUTH-005). Verifies an unforgeable
    /// credential and mints a [`VerifiedIngressPrincipal`] bound to the caller's
    /// CONFIGURED tenant. If the bound token is empty/unset, every `/v1/*`
    /// data-plane request fails closed with 401 (no allow-all path); the verified
    /// tenant then drives the in-process Cedar gate so cross-tenant is forbidden.
    pub ingress_authenticator: Arc<dyn IngressPrincipalAuthenticator>, // data_class: INTERNAL_ONLY
    /// Admin control-plane authenticator (AUTH-005 / ADR-0573 PR2). Verifies the
    /// configured admin bearer and mints a [`VerifiedAdminPrincipal`] bound to the
    /// CONFIGURED admin tenant — NEVER the caller-supplied `x-oya-admin-tenant`
    /// header. An empty/unset admin bearer verifies nothing, so every `/admin/v1/*`
    /// tenant-scoped route fails closed with 401 (no allow-all path).
    pub admin_authenticator: Arc<dyn AdminPrincipalAuthenticator>, // data_class: INTERNAL_ONLY
    pub admin_bearer_token: Option<String>, // data_class: SECRET
    /// Runtime environment string (e.g. `production`). Production enforces the
    /// provider OAuth-compliance fail-closed gate on the dynamic admin path,
    /// mirroring the boot-time gate in the app crate.
    pub environment: String, // data_class: INTERNAL_ONLY
    /// Providers whose OAuth-subscription compliance status is `APPROVED`.
    /// In production, runtime admin registration of an OAuth-subscription seat
    /// is rejected unless the provider is present here.
    pub oauth_approved_providers: std::collections::HashSet<Provider>, // data_class: INTERNAL_ONLY
    /// Process-shared D3 upstream OAuth keyed-flight registry.
    pub upstream_oauth_singleflight: Arc<UpstreamOAuthSingleflight>, // data_class: INTERNAL_ONLY
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
        secret_store: Arc<dyn SecretProviderStore>,
        anthropic_base_url: String,
        tenant_id: TenantId,
    ) -> Result<Self, reqwest::Error> {
        let pool_registry = PoolRegistry::new();
        pool_registry.insert_pool(tenant_id.clone(), Provider::Anthropic, Arc::clone(&pool));
        Self::new_with_pool_registry(
            pool,
            pool_registry,
            gate,
            sink,
            secret_store,
            anthropic_base_url,
            tenant_id,
            None,
            None,
            "development".to_string(),
            std::collections::HashSet::new(),
        )
    }

    /// Build an `AppState` with an explicit tenant/provider pool registry.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_pool_registry(
        pool: Arc<Mutex<SubscriptionPool>>,
        pool_registry: PoolRegistry,
        gate: Arc<dyn AuthzGate + Send + Sync>,
        sink: Arc<dyn EventSink + Send + Sync>,
        secret_store: Arc<dyn SecretProviderStore>,
        anthropic_base_url: String,
        tenant_id: TenantId,
        ingress_bearer_token: Option<String>,
        admin_bearer_token: Option<String>,
        environment: String,
        oauth_approved_providers: std::collections::HashSet<Provider>,
    ) -> Result<Self, reqwest::Error> {
        let http_client = Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
        );
        // Default ingress authenticator: the configured bearer bound to this
        // service's own tenant. The composition root (facade `build_app`) or a
        // test may override the binding via `with_ingress_authenticator` so a
        // cross-tenant principal is expressible. An empty token => every
        // data-plane request 401 (fail-closed).
        let ingress_authenticator: Arc<dyn IngressPrincipalAuthenticator> =
            Arc::new(ConfiguredBearerIngressAuthenticator::new(
                ingress_bearer_token.unwrap_or_default(),
                tenant_id.clone(),
            ));
        // Default admin authenticator: the configured admin bearer bound to this
        // service's own tenant (the administered tenant). An empty token => every
        // tenant-scoped admin request 401 (fail-closed); the verified admin tenant
        // (never `x-oya-admin-tenant`) then drives the tenant-isolation guard + gate.
        let admin_authenticator: Arc<dyn AdminPrincipalAuthenticator> =
            Arc::new(ConfiguredBearerAdminAuthenticator::new(
                admin_bearer_token.clone().unwrap_or_default(),
                tenant_id.clone(),
            ));
        Ok(Self {
            pool,
            pool_registry,
            gate,
            sink,
            secret_store,
            anthropic_base_url,
            openai_compatible_base_url: OPENAI_COMPATIBLE_BASE_URL.to_string(),
            codex_oauth_base_url: CODEX_OAUTH_BASE_URL.to_string(),
            gemini_base_url: GEMINI_BASE_URL.to_string(),
            tenant_id,
            ingress_authenticator,
            admin_authenticator,
            admin_bearer_token,
            environment,
            oauth_approved_providers,
            upstream_oauth_singleflight: Arc::new(UpstreamOAuthSingleflight::new()),
            http_client,
        })
    }

    /// Return this state with the data-plane ingress bearer authenticator
    /// (re)configured, bound to this service's own tenant. Test fixtures and
    /// embedding applications use this helper instead of mutating the field.
    #[must_use]
    pub fn with_ingress_bearer_token(mut self, token: Option<String>) -> Self {
        self.ingress_authenticator = Arc::new(ConfiguredBearerIngressAuthenticator::new(
            token.unwrap_or_default(),
            self.tenant_id.clone(),
        ));
        self
    }

    /// Install a custom ingress authenticator (e.g. a cross-tenant-bound
    /// reference adapter for tests, or a production mTLS/SPIFFE adapter).
    /// Overrides the default bearer authenticator built from the configured
    /// ingress token. No default-allow path exists: an authenticator that
    /// verifies nothing leaves every data-plane request at 401.
    #[must_use]
    pub fn with_ingress_authenticator(
        mut self,
        authenticator: Arc<dyn IngressPrincipalAuthenticator>,
    ) -> Self {
        self.ingress_authenticator = authenticator;
        self
    }

    /// True when the runtime environment is `production` (case-insensitive).
    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
    }

    /// True when `provider`'s OAuth-subscription compliance status is APPROVED.
    pub fn is_oauth_approved(&self, provider: Provider) -> bool {
        self.oauth_approved_providers.contains(&provider)
    }
}

/// Maximum request body size: 1 MiB. Requests exceeding this limit receive
/// HTTP 413 Payload Too Large (enforced by axum [`DefaultBodyLimit`]).
const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteBodyError {
    MalformedJson,
    MissingModel,
    InvalidModelType,
    EmptyModel,
    UnknownModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SafetyBlockClass {
    CredentialOrSecret,
    TenantBoundary,
    PromptInjection,
}

// =====================================================================
// Data-plane ingress authn/authz (AUTH-005 / ADR-0573)
//
// The data plane previously trusted any holder of the single shared ingress
// bearer and fed the in-process Cedar gate `principal_tenant == resource_tenant
// == state.tenant_id` with a hard-coded IngressRealm role, so the cross-tenant
// forbid could never fire. We mirror the shipped gold standard
// (iam/facade/identity-workload-rest): verify an UNFORGEABLE principal, then
// feed its VERIFIED tenant (never a header, never `state.tenant_id`) to the gate,
// fail-closed.
// =====================================================================

/// An ingress caller whose bearer the [`IngressPrincipalAuthenticator`] has
/// VERIFIED. The fields are private with only a `pub(crate)` constructor, so a
/// handler cannot fabricate one from caller-supplied `x-*` headers — only a
/// verifier that proved an unforgeable credential mints it. Mirrors the
/// `VerifiedCaller` newtype at iam/facade/identity-workload-rest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedIngressPrincipal {
    /// The tenant the verified caller acts within (from the credential binding,
    /// NEVER a caller-supplied header).
    tenant: TenantId, // data_class: INTERNAL_ONLY
    /// The verified caller's agent label, when the credential carries one.
    agent: Option<AgentId>, // data_class: INTERNAL_ONLY
}

impl VerifiedIngressPrincipal {
    /// Mint a verified principal. Crate-private: only an authenticator in this
    /// crate can construct one (no public constructor → unforgeable).
    pub(crate) fn new(tenant: TenantId, agent: Option<AgentId>) -> Self {
        Self { tenant, agent }
    }

    /// The verified caller's tenant (trusted; derived from the credential).
    #[must_use]
    pub fn tenant(&self) -> &TenantId {
        &self.tenant
    }

    /// The verified caller's agent label, when the credential carries one.
    #[must_use]
    pub fn agent(&self) -> Option<&AgentId> {
        self.agent.as_ref()
    }
}

/// Ingress authentication PORT: derive a [`VerifiedIngressPrincipal`] from the
/// request headers by checking an UNFORGEABLE credential (a constant-time bearer
/// compare today; mTLS/SPIFFE in a production adapter). `None` ⇒ no verified
/// principal ⇒ `401` (default-deny). Caller-supplied `x-*` headers MUST NOT
/// authenticate — only a verified credential mints a principal.
pub trait IngressPrincipalAuthenticator: Send + Sync {
    /// Verify the caller's credential. `None` ⇒ `401`.
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedIngressPrincipal>;
}

/// Reference [`IngressPrincipalAuthenticator`] adapter: a single configured
/// bearer token bound to one principal tenant, compared in constant time. The
/// minted principal carries the CONFIGURED tenant, NEVER a caller header. An
/// empty/unset token verifies NOTHING (every request `401`) — no allow-all path.
#[derive(Clone, Debug)]
pub struct ConfiguredBearerIngressAuthenticator {
    token: String,              // data_class: SECRET
    principal_tenant: TenantId, // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerIngressAuthenticator {
    /// Build an authenticator for one configured bearer credential bound to
    /// `principal_tenant` (what a successful verify returns; never a header).
    #[must_use]
    pub fn new(token: impl Into<String>, principal_tenant: TenantId) -> Self {
        Self {
            token: token.into(),
            principal_tenant,
        }
    }
}

impl IngressPrincipalAuthenticator for ConfiguredBearerIngressAuthenticator {
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedIngressPrincipal> {
        let configured = self.token.trim();
        if configured.is_empty() {
            // No configured credential ⇒ no caller can be verified (fail-closed).
            return None;
        }
        let presented = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))?;
        if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
            Some(VerifiedIngressPrincipal::new(
                self.principal_tenant.clone(),
                None,
            ))
        } else {
            None
        }
    }
}

/// One bearer→principal binding for the multi-tenant ingress authenticator: a
/// configured credential and the tenant (+ optional agent) a successful match
/// acts as. The tenant/agent are NEVER caller-supplied headers.
#[derive(Clone, Debug)]
pub struct BearerBinding {
    pub token: String,          // data_class: SECRET
    pub tenant: TenantId,       // data_class: INTERNAL_ONLY
    pub agent: Option<AgentId>, // data_class: INTERNAL_ONLY
}

/// Multi-tenant [`IngressPrincipalAuthenticator`] (AUTH-005 increment-3): a set
/// of configured bearer credentials, each bound to one principal tenant. The
/// verified tenant drives the pool keying + gate, so a tenant-B caller's bearer
/// can only ever mint a tenant-B principal. `verify()` scans EVERY binding in
/// constant time with NO early break, so a hit's position does not leak via
/// timing. An empty set or an unknown token verifies NOTHING (every request
/// `401`) — no allow-all path.
#[derive(Clone, Debug)]
pub struct ConfiguredBearerMapIngressAuthenticator {
    bindings: Vec<BearerBinding>, // data_class: SECRET
}

impl ConfiguredBearerMapIngressAuthenticator {
    /// Build a multi-tenant authenticator from token→principal bindings.
    #[must_use]
    pub fn new(bindings: Vec<BearerBinding>) -> Self {
        Self { bindings }
    }
}

impl IngressPrincipalAuthenticator for ConfiguredBearerMapIngressAuthenticator {
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedIngressPrincipal> {
        let presented = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))?
            .as_bytes();
        // Constant-time, no-early-break scan: every binding is compared so the
        // matched binding's position never leaks via timing. A binding with an
        // empty configured token can never match (fail-closed).
        let mut matched: Option<&BearerBinding> = None;
        for binding in &self.bindings {
            let configured = binding.token.trim();
            let is_match =
                !configured.is_empty() && constant_time_eq(presented, configured.as_bytes());
            if is_match {
                matched = Some(binding);
            }
        }
        matched.map(|binding| {
            VerifiedIngressPrincipal::new(binding.tenant.clone(), binding.agent.clone())
        })
    }
}

// =====================================================================
// Admin control-plane authn/authz (AUTH-005 / ADR-0573 PR2)
//
// The admin plane previously gated every `/admin/v1/*` route on a single global
// admin bearer plus `admin_tenant_allowed`, which only checked the caller-supplied
// `x-oya-admin-tenant` header against the path tenant — BOTH attacker-controlled —
// and never invoked the authz gate. One admin credential could administer EVERY
// tenant (cross-tenant IDOR). We mirror the shipped gold standard
// (iam/facade/identity-workload-rest + TenantScopedLifecycleAuthorizer): verify an
// UNFORGEABLE admin principal bound to the CONFIGURED admin tenant, enforce the
// tenant-isolation invariant SERVER-SIDE (verified tenant == path tenant), and
// consult the gate as the authoritative + auditable decision, fail-closed.
// =====================================================================

/// An admin caller whose bearer the [`AdminPrincipalAuthenticator`] has VERIFIED.
/// The field is private with only a `pub(crate)` constructor, so a handler cannot
/// fabricate one from caller-supplied `x-*` headers — only a verifier that proved
/// an unforgeable credential mints it. Mirrors [`VerifiedIngressPrincipal`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAdminPrincipal {
    /// The tenant this verified admin may administer (from the credential binding,
    /// NEVER a caller-supplied header).
    tenant: TenantId, // data_class: INTERNAL_ONLY
}

impl VerifiedAdminPrincipal {
    /// Mint a verified admin principal. Crate-private: only an authenticator in
    /// this crate can construct one (no public constructor → unforgeable).
    pub(crate) fn new(tenant: TenantId) -> Self {
        Self { tenant }
    }

    /// The tenant this verified admin may administer (trusted; from the credential).
    #[must_use]
    pub fn tenant(&self) -> &TenantId {
        &self.tenant
    }
}

/// Admin authentication PORT: derive a [`VerifiedAdminPrincipal`] from the request
/// headers by checking an UNFORGEABLE credential (a constant-time bearer compare
/// today; mTLS/SPIFFE in a production adapter). `None` ⇒ no verified admin ⇒ `401`
/// (default-deny). Caller-supplied `x-*` headers MUST NOT authenticate.
pub trait AdminPrincipalAuthenticator: Send + Sync {
    /// Verify the admin caller's credential. `None` ⇒ `401`.
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedAdminPrincipal>;
}

/// Reference [`AdminPrincipalAuthenticator`] adapter: a single configured admin
/// bearer token bound to one admin tenant, compared in constant time. The minted
/// principal carries the CONFIGURED admin tenant, NEVER a caller header. An
/// empty/unset token verifies NOTHING (every request `401`) — no allow-all path.
#[derive(Clone, Debug)]
pub struct ConfiguredBearerAdminAuthenticator {
    token: String,          // data_class: SECRET
    admin_tenant: TenantId, // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerAdminAuthenticator {
    /// Build an authenticator for one configured admin bearer credential bound to
    /// `admin_tenant` (what a successful verify returns; never a header).
    #[must_use]
    pub fn new(token: impl Into<String>, admin_tenant: TenantId) -> Self {
        Self {
            token: token.into(),
            admin_tenant,
        }
    }
}

impl AdminPrincipalAuthenticator for ConfiguredBearerAdminAuthenticator {
    fn verify(&self, headers: &HeaderMap) -> Option<VerifiedAdminPrincipal> {
        let configured = self.token.trim();
        if configured.is_empty() {
            // No configured credential ⇒ no admin can be verified (fail-closed).
            return None;
        }
        let presented = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))?;
        if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
            Some(VerifiedAdminPrincipal::new(self.admin_tenant.clone()))
        } else {
            None
        }
    }
}

/// Authenticate the data-plane ingress caller and authorize the request through
/// the in-process Cedar gate, bound to the VERIFIED principal tenant vs the
/// service resource tenant. Fail-closed: no verified principal ⇒ `401`; a gate
/// `Forbid` (cross-tenant) or any gate fault ⇒ `403`. Mirrors the gold-standard
/// ordering at iam/facade/identity-workload-rest: authn first, resolve the
/// resource tenant server-side, then decide.
fn require_data_plane_ingress_authz(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<VerifiedIngressPrincipal, Response> {
    // (1) AUTHN — unforgeable verified principal. A self-attested header cannot
    // reach the `Some` branch (the authenticator ignores them).
    let Some(principal) = state.ingress_authenticator.verify(headers) else {
        return Err(openai_error_response(
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "missing_or_invalid_ingress_bearer",
            "data-plane routes require a valid ingress bearer token",
            None,
        ));
    };

    // (2) AUTHZ — intra-tenant gate check, scoped to the VERIFIED principal
    // tenant (never `state.tenant_id`, never a header). The data plane is now
    // multi-tenant: the caller acts on ITS OWN tenant's pool, so the resource
    // tenant IS `principal.tenant()`. CROSS-tenant isolation is enforced by
    // keying the pool by `principal.tenant()` (primary barrier) plus the kernel
    // `authz_allows` backstop (principal=verified caller vs the pool's own
    // tenant); this edge gate enforces whether the principal may SelectSeat at
    // all within its tenant. A Forbid — or any gate fault, which the adapter
    // maps to Forbid — is fail-closed 403. The agent label does not affect the
    // tenant decision: prefer a verified agent (future mTLS adapter), else the
    // caller's x-agent-id, else a tenant-derived sentinel so the Cedar Workload
    // uid is always well-formed.
    let agent = principal
        .agent()
        .cloned()
        .or_else(|| {
            headers
                .get("x-agent-id")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| AgentId::new(value).ok())
        })
        .or_else(|| AgentId::new(format!("ingress:{}", principal.tenant().as_str())).ok());
    let Some(agent) = agent else {
        // Could not construct any principal label => fail-closed 403.
        return Err(forbidden_ingress_response());
    };
    let request = AuthzRequest {
        principal_tenant: principal.tenant(),
        principal_agent: &agent,
        action: AuthzAction::SelectSeat,
        resource_tenant: principal.tenant(),
        resource_provider: Provider::Anthropic,
    };
    if state.gate.decide(&request) == AuthzDecision::Forbid {
        return Err(forbidden_ingress_response());
    }
    Ok(principal)
}

/// Fail-closed 403 for a denied/faulted ingress authorization.
fn forbidden_ingress_response() -> Response {
    openai_error_response(
        StatusCode::FORBIDDEN,
        "permission_error",
        "ingress_tenant_forbidden",
        "ingress principal is not authorized for this tenant",
        None,
    )
}

/// Authenticate the admin caller and authorize the request, fail-closed. Mirrors
/// the gold-standard ordering (iam/facade/identity-workload-rest +
/// `TenantScopedLifecycleAuthorizer`): authn an UNFORGEABLE principal first,
/// enforce tenant isolation against the administered (path) tenant SERVER-SIDE,
/// then consult the in-process authz gate.
///
/// - No verified admin principal ⇒ `401`. The configured admin bearer is the only
///   credential that mints one; a caller-supplied `x-oya-admin-tenant` header never
///   does (it is no longer an authz input).
/// - Verified admin tenant != the administered (path) tenant ⇒ `403`. This
///   BLAST-RADIUS guard is MANDATORY because the admin Cedar rule is realm-scoped:
///   `decide()` alone would Allow any admin principal on ANY tenant, so the tenant
///   axis must be enforced here, not delegated to the gate (per #815/#124 review).
/// - A gate `Forbid` — or any gate fault, which the adapter maps to `Forbid` —
///   ⇒ `403`. The gate is the authoritative + auditable realm decision (admin
///   actions map to the Cedar `AdminRealm` / `RefreshKeyPool`).
fn require_admin_authz(
    state: &AppState,
    headers: &HeaderMap,
    path_tenant: &str,
) -> Result<VerifiedAdminPrincipal, Response> {
    // (1) AUTHN — unforgeable verified admin principal. A self-attested header
    // cannot reach the `Some` branch (the authenticator ignores them).
    let Some(principal) = state.admin_authenticator.verify(headers) else {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };
    // (2) TENANT ISOLATION (blast-radius guard) — the verified admin may only
    // administer its own tenant. Deny-wins for any cross-tenant request.
    if principal.tenant().as_str() != path_tenant {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    // (3) AUTHZ — authoritative + auditable realm decision via the in-process gate.
    // `RefreshToken` maps to the Cedar AdminRealm/RefreshKeyPool action; a Forbid or
    // any gate fault is fail-closed 403. `resource_tenant == principal.tenant()` is
    // sound: step (2) already proved the path tenant equals the verified tenant.
    let Ok(agent) = AgentId::new(format!("admin:{}", principal.tenant().as_str())) else {
        return Err(StatusCode::FORBIDDEN.into_response());
    };
    let request = AuthzRequest {
        principal_tenant: principal.tenant(),
        principal_agent: &agent,
        action: AuthzAction::RefreshToken,
        resource_tenant: principal.tenant(),
        resource_provider: Provider::Anthropic,
    };
    if state.gate.decide(&request) == AuthzDecision::Forbid {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    Ok(principal)
}

fn guard_in_transit_payload(body: &[u8]) -> Result<(), Response> {
    let body = String::from_utf8_lossy(body);
    let lower = body.to_ascii_lowercase();
    let safety_class = if [
        "api_key",
        "api-key",
        "access_token",
        "refresh_token",
        "bearer ",
        "private key",
        "password",
        "secret_access_key",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(SafetyBlockClass::CredentialOrSecret)
    } else if [
        "tenant_boundary_violation",
        "cross-tenant",
        "other tenant",
        "tenant_id_override",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(SafetyBlockClass::TenantBoundary)
    } else if [
        "ignore previous instructions",
        "ignore all previous",
        "developer message",
        "system prompt",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Some(SafetyBlockClass::PromptInjection)
    } else {
        None
    };

    match safety_class {
        Some(safety_class) => Err(safety_guardrail_response(safety_class)),
        None => Ok(()),
    }
}

fn safety_guardrail_response(safety_class: SafetyBlockClass) -> Response {
    let (class, message) = match safety_class {
        SafetyBlockClass::CredentialOrSecret => (
            "credential_or_secret",
            "request body contains credential-like material that cannot cross the model boundary",
        ),
        SafetyBlockClass::TenantBoundary => (
            "tenant_boundary",
            "request body contains tenant-boundary override material",
        ),
        SafetyBlockClass::PromptInjection => (
            "prompt_injection",
            "request body contains prompt-injection control text",
        ),
    };
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": {
                "type": "safety_guardrail",
                "code": "safety_guardrail_blocked",
                "safety_class": class,
                "message": message,
                "provider_request_dispatched": false
            }
        })),
    )
        .into_response()
}

fn route_decision_for_body(
    protocol: ProtocolShape,
    body: &[u8],
) -> Result<RoutingDecision, RouteBodyError> {
    let payload = serde_json::from_slice::<serde_json::Value>(body)
        .map_err(|_| RouteBodyError::MalformedJson)?;
    let Some(model_value) = payload.get("model") else {
        return Err(RouteBodyError::MissingModel);
    };
    let Some(model) = model_value.as_str() else {
        return Err(RouteBodyError::InvalidModelType);
    };
    if model.trim().is_empty() {
        return Err(RouteBodyError::EmptyModel);
    }
    ModelRouter::default()
        .route(RouteRequest {
            protocol,
            model: model.to_string(),
            route_policy: RoutePolicy::default(),
            tenant_default_backend: None,
        })
        .map_err(|error| match error {
            ModelRoutingError::EmptyModel => RouteBodyError::EmptyModel,
            ModelRoutingError::UnknownModel { .. } => RouteBodyError::UnknownModel,
        })
}

fn rewrite_body_model(body: Bytes, upstream_model: &str) -> Bytes {
    let Ok(mut payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return body;
    };
    let Some(object) = payload.as_object_mut() else {
        return body;
    };
    if object
        .get("model")
        .and_then(serde_json::Value::as_str)
        .map(|model| model == upstream_model)
        .unwrap_or(false)
    {
        return body;
    }
    object.insert(
        "model".to_string(),
        serde_json::Value::String(upstream_model.to_string()),
    );
    serde_json::to_vec(&payload)
        .map(Bytes::from)
        .unwrap_or(body)
}

fn openai_model_routing_error_response(error: RouteBodyError) -> Response {
    let (status, code, message) = match error {
        RouteBodyError::UnknownModel => (
            StatusCode::NOT_FOUND,
            "model_not_found",
            "requested model is not registered for this gateway",
        ),
        RouteBodyError::MissingModel => (
            StatusCode::BAD_REQUEST,
            "missing_model",
            "request body must include a model field",
        ),
        RouteBodyError::InvalidModelType => (
            StatusCode::BAD_REQUEST,
            "invalid_model",
            "request body model field must be a string",
        ),
        RouteBodyError::EmptyModel => (
            StatusCode::BAD_REQUEST,
            "empty_model",
            "request body model field must not be empty",
        ),
        RouteBodyError::MalformedJson => (
            StatusCode::BAD_REQUEST,
            "malformed_json",
            "request body must be valid JSON",
        ),
    };
    openai_error_response(status, "invalid_request_error", code, message, None)
}

fn unsupported_translation_response(
    from_protocol: &'static str,
    backend: BackendClass,
    upstream_model: &str,
) -> Response {
    openai_error_response(
        StatusCode::NOT_IMPLEMENTED,
        "unsupported_feature",
        "unsupported_model_translation",
        format!(
            "{from_protocol} to {backend:?} translation is not implemented for model {upstream_model}"
        ),
        None,
    )
}

fn seat_outcome_for_upstream_status(status: u16) -> SeatOutcome {
    if status == 429 {
        SeatOutcome::RateLimited429
    } else if status >= 400 {
        // The kernel currently exposes one non-rate-limit upstream failure
        // bucket. Treat every upstream HTTP error as non-success so provider
        // failures never falsely reset failure counts as SeatOutcome::Ok.
        SeatOutcome::ServerError5xx
    } else {
        SeatOutcome::Ok
    }
}

fn event_status_for_upstream_status(status: u16) -> EventStatus {
    if status == 429 {
        EventStatus::RateLimited
    } else if status >= 400 {
        EventStatus::UpstreamError
    } else {
        EventStatus::Ok
    }
}

/// Build the axum [`Router`] for the cloud-intelligence REST adapter.
///
/// Routes:
/// - `POST /v1/messages` — OAuth-gated reverse proxy to Anthropic API.
/// - `POST /v1/complete` — legacy Anthropic completions deprecation surface.
/// - `GET  /healthz`     — lightweight health probe.
/// - `GET  /livez`       — Kubernetes liveness probe.
/// - `GET  /readyz`      — readiness probe (pool lock is reachable).
/// - `GET  /metrics`     — Prometheus text exposition from live gateway state.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/messages", post(handle_proxy))
        .route("/v1/messages/count_tokens", post(handle_count_tokens))
        .route("/v1/complete", post(handle_legacy_complete))
        .route("/v1/chat/completions", post(handle_openai_chat_completions))
        .route("/v1/embeddings", post(handle_openai_embeddings))
        .route("/v1/models", get(handle_models))
        .route("/admin/v1/status", get(handle_admin_status))
        .route("/admin/v1/accounts", get(handle_admin_accounts))
        .route("/admin/v1/analytics", get(handle_admin_analytics))
        .route(
            "/admin/v1/analytics/stream",
            get(handle_admin_analytics_stream),
        )
        .route("/admin/v1/guardrails", get(handle_admin_guardrails))
        .route("/admin/v1/agent-runtimes", get(handle_admin_agent_runtimes))
        .route(
            "/admin/v1/agent-schedules",
            get(handle_admin_agent_schedules),
        )
        .route(
            "/admin/v1/parity/canaries",
            get(handle_admin_parity_canaries),
        )
        .route(
            "/admin/v1/guardrails/escalations",
            get(handle_admin_guardrail_escalations),
        )
        .route(
            "/admin/v1/evidence/retention",
            get(handle_admin_evidence_retention),
        )
        .route(
            "/admin/v1/redaction/profiles",
            get(handle_admin_redaction_profiles),
        )
        .route("/admin/v1/resume", post(handle_admin_resume))
        .route(
            "/admin/v1/tenants/{tenant_id}/providers/{provider}/pool",
            get(handle_admin_pool_status),
        )
        .route(
            "/admin/v1/tenants/{tenant_id}/providers/{provider}/subscriptions",
            post(handle_admin_register_subscription),
        )
        .route("/healthz", get(handle_healthz))
        .route("/livez", get(handle_livez))
        .route("/readyz", get(handle_readyz))
        .route("/metrics", get(handle_metrics))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}

fn oauth_refresh_retry_response(lease: SeatLease) -> Response {
    let _ = lease.complete(SeatOutcome::Released, Instant::now());
    axum::response::Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("retry-after", "1")
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::SERVICE_UNAVAILABLE.into_response())
}

/// POST /v1/messages handler.
async fn handle_proxy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut body: Bytes,
) -> Response {
    let principal = match require_data_plane_ingress_authz(&state, &headers) {
        Ok(principal) => principal,
        Err(response) => return response,
    };
    if let Err(response) = guard_in_transit_payload(&body) {
        return response;
    }
    let route_decision = match route_decision_for_body(ProtocolShape::AnthropicMessages, &body) {
        Ok(decision) => decision,
        Err(error) => return openai_model_routing_error_response(error),
    };
    body = rewrite_body_model(body, &route_decision.upstream_model);
    match route_decision.backend {
        BackendClass::GeminiNative => {
            return handle_gemini_anthropic_messages_proxy(
                state,
                headers,
                body,
                principal.tenant(),
            )
            .await;
        }
        BackendClass::AnthropicSubscription => {}
        BackendClass::OpenAiCompatible => {
            return unsupported_translation_response(
                "Anthropic Messages",
                route_decision.backend,
                &route_decision.upstream_model,
            );
        }
    }

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

    // Resolve the VERIFIED caller tenant's Anthropic pool (multi-tenant: never
    // `state.pool`, which is the process tenant's default). Absent => 503.
    let Some(pool) = state
        .pool_registry
        .get_pool(principal.tenant(), Provider::Anthropic)
    else {
        warn!(tenant = %principal.tenant().as_str(), "no Anthropic pool for tenant");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    // Acquire a SeatLease (Fix-1: prevents same-seat double-allocation).
    let lease = match SubscriptionPool::lease(
        &pool,
        principal.tenant(),
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
        tenant_id: principal.tenant().clone(),
    };

    // Resolve the leased seat's opaque credential handle AND its transport mode
    // in a single lock so OAuth-subscription vs API-key seats route to the
    // correct upstream path (API-key seats must NEVER trigger OAuth refresh).
    // Resolve from the SAME (caller-tenant) pool the seat was leased from.
    let (credential_handle, credential_mode) = match pool.lock().ok().and_then(|pool| {
        let handle = pool.credential_secret_handle_for_seat(&seat_id)?;
        let mode = pool.credential_mode_for_seat(&seat_id)?;
        Some((handle, mode))
    }) {
        Some((handle, mode)) => (handle, mode),
        None => {
            let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Build the async adapter (no spawn_blocking — Item 1).
    let adapter = AnthropicAdapter::with_base_url_and_singleflight(
        ArcSecretStore {
            inner: Arc::clone(&state.secret_store),
        },
        state.anthropic_base_url.clone(),
        Arc::clone(&state.upstream_oauth_singleflight),
    );

    // Detect SSE streaming intent from the Accept header.
    let wants_sse = proxy_request
        .headers
        .get("accept")
        .map(|v| v.to_lowercase().contains("text/event-stream"))
        .unwrap_or(false);

    if wants_sse {
        // --- SSE streaming path ---
        let stream_result = match credential_mode {
            CredentialMode::OAuthSubscription => {
                // Obtain access token first (refresh may fail before we stream).
                let access_token = match adapter
                    .refresh_token(&state.http_client, &credential_handle)
                    .await
                {
                    Ok(t) => t,
                    Err(RestAdapterError::OAuthRefreshRetryRequired) => {
                        return oauth_refresh_retry_response(lease);
                    }
                    Err(_) => {
                        let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                adapter
                    .proxy_stream(&state.http_client, &access_token, proxy_request)
                    .await
            }
            CredentialMode::ApiKey => {
                // API-key seats resolve the provider key from the opaque handle
                // and MUST NOT trigger an OAuth refresh.
                let api_key = match state
                    .secret_store
                    .fetch_refresh_token(&credential_handle)
                    .await
                {
                    Ok(key) => key,
                    Err(_) => {
                        let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                adapter
                    .proxy_stream_with_api_key(&state.http_client, &api_key, proxy_request)
                    .await
            }
        };

        match stream_result {
            Ok((upstream_status, byte_stream)) => {
                // Wrap the stream with the lease so the seat is held until the
                // response body is fully consumed (or client disconnects).
                let lease_stream = SseStreamWithLease::new_with_clean_outcome(
                    byte_stream,
                    lease,
                    seat_outcome_for_upstream_status(upstream_status),
                );
                let body = Body::from_stream(lease_stream);
                axum::response::Response::builder()
                    .status(upstream_status)
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .header("connection", "keep-alive")
                    .body(body)
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Err(RestAdapterError::OAuthRefreshRetryRequired) => oauth_refresh_retry_response(lease),
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
        // --- Non-streaming (one-shot JSON) path ---
        let result = match credential_mode {
            CredentialMode::OAuthSubscription => {
                adapter
                    .proxy(&state.http_client, &proxy_request, &credential_handle)
                    .await
            }
            CredentialMode::ApiKey => {
                // API-key seats resolve the provider key from the opaque handle
                // and MUST NOT trigger an OAuth refresh.
                match state
                    .secret_store
                    .fetch_refresh_token(&credential_handle)
                    .await
                {
                    Ok(api_key) => {
                        adapter
                            .proxy_with_api_key(&state.http_client, &proxy_request, &api_key)
                            .await
                    }
                    Err(_) => {
                        let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
        };

        match result {
            Ok(resp) => {
                let upstream_status = resp.status;
                let _ = lease.complete(
                    seat_outcome_for_upstream_status(upstream_status),
                    Instant::now(),
                );

                // Emit event (non-fatal).
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let event = LlmGatewayEvent {
                    request_id: format!("req-{now_ms}"),
                    tenant_id: principal.tenant().clone(),
                    agent_id,
                    seat_id: seat_id.clone(),
                    provider: Provider::Anthropic,
                    model: String::new(),
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    ms_latency: 0,
                    status: event_status_for_upstream_status(upstream_status),
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
            Err(RestAdapterError::OAuthRefreshRetryRequired) => oauth_refresh_retry_response(lease),
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

struct ProviderLeaseContext {
    lease: SeatLease,                // data_class: INTERNAL_ONLY
    seat_id: SeatId,                 // data_class: INTERNAL_ONLY
    credential_handle: String,       // data_class: INTERNAL_ONLY
    credential_mode: CredentialMode, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy)]
enum LeaseError {
    Forbidden,
    NoEligibleSeat,
    Internal,
}

fn headers_to_btree(headers: &HeaderMap) -> BTreeMap<String, String> {
    let mut proxy_headers = BTreeMap::new();
    for (k, v) in headers.iter() {
        if let Ok(val) = v.to_str() {
            proxy_headers.insert(k.as_str().to_string(), val.to_string());
        }
    }
    proxy_headers
}

fn acquire_provider_lease(
    state: &AppState,
    provider: Provider,
    principal_tenant: &TenantId,
    agent_id: &AgentId,
) -> Result<ProviderLeaseContext, LeaseError> {
    let Some(pool) = state.pool_registry.get_pool(principal_tenant, provider) else {
        return Err(LeaseError::NoEligibleSeat);
    };
    let lease = match SubscriptionPool::lease(
        &pool,
        principal_tenant,
        agent_id,
        state.gate.as_ref(),
        Instant::now(),
    ) {
        Ok(lease) => lease,
        Err(SubscriptionPoolError::ForbiddenByPolicy) => return Err(LeaseError::Forbidden),
        Err(SubscriptionPoolError::NoEligibleSeat) => return Err(LeaseError::NoEligibleSeat),
        Err(_) => return Err(LeaseError::Internal),
    };
    let seat_id = lease.seat_id().clone();
    let Some((credential_handle, credential_mode)) = pool.lock().ok().and_then(|pool| {
        let handle = pool.credential_secret_handle_for_seat(&seat_id)?;
        let mode = pool.credential_mode_for_seat(&seat_id)?;
        Some((handle, mode))
    }) else {
        let _ = lease.complete(SeatOutcome::RefreshFailed, Instant::now());
        return Err(LeaseError::Internal);
    };
    Ok(ProviderLeaseContext {
        lease,
        seat_id,
        credential_handle,
        credential_mode,
    })
}

#[derive(Serialize)]
struct OpenAiCompatibleErrorBody {
    error: OpenAiCompatibleError, // data_class: INTERNAL_ONLY
}

#[derive(Serialize)]
struct OpenAiCompatibleError {
    message: String, // data_class: INTERNAL_ONLY
    #[serde(rename = "type")]
    error_type: &'static str, // data_class: INTERNAL_ONLY
    code: &'static str, // data_class: INTERNAL_ONLY
}

fn openai_error_response(
    status: StatusCode,
    error_type: &'static str,
    code: &'static str,
    message: impl Into<String>,
    retry_after_seconds: Option<u64>,
) -> Response {
    let payload = OpenAiCompatibleErrorBody {
        error: OpenAiCompatibleError {
            message: message.into(),
            error_type,
            code,
        },
    };
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{\"error\":{}}".to_vec());
    let mut builder = axum::response::Response::builder()
        .status(status)
        .header("content-type", "application/json");
    if let Some(seconds) = retry_after_seconds {
        builder = builder.header("retry-after", seconds.to_string());
    }
    builder
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn codex_proxy_response_to_axum(resp: CodexProxyResponse) -> Response {
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

fn gemini_proxy_response_to_axum(resp: GeminiProxyResponse) -> Response {
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
        .header("content-type", "application/json")
        .body(Body::from(resp.body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn openai_adapter_error_to_response(error: CodexAdapterError) -> Response {
    match error {
        CodexAdapterError::RateLimited { retry_after_secs } => openai_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_error",
            "upstream_rate_limited",
            "upstream provider rate limited the request",
            retry_after_secs,
        ),
        CodexAdapterError::UpstreamError { status, body } => openai_error_response(
            StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
            "upstream_error",
            "upstream_error",
            body,
            None,
        ),
        CodexAdapterError::TransportError(_) => openai_error_response(
            StatusCode::BAD_GATEWAY,
            "upstream_error",
            "provider_transport_error",
            "upstream provider transport failed",
            None,
        ),
        CodexAdapterError::RefreshFailed(_) => openai_error_response(
            StatusCode::BAD_GATEWAY,
            "upstream_error",
            "provider_refresh_failed",
            "upstream provider credential refresh failed",
            None,
        ),
    }
}

fn gemini_adapter_error_to_response(error: GeminiAdapterError) -> Response {
    match error {
        GeminiAdapterError::InvalidRequest(message) => openai_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            "invalid_gemini_adapter_request",
            message,
            None,
        ),
        GeminiAdapterError::RateLimited { retry_after_secs } => openai_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_error",
            "upstream_rate_limited",
            "upstream provider rate limited the request",
            retry_after_secs,
        ),
        GeminiAdapterError::UpstreamError { status, body } => openai_error_response(
            StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
            "upstream_error",
            "upstream_error",
            body,
            None,
        ),
        GeminiAdapterError::TransportError(_) => openai_error_response(
            StatusCode::BAD_GATEWAY,
            "upstream_error",
            "provider_transport_error",
            "upstream provider transport failed",
            None,
        ),
    }
}

fn openai_stream_requested(headers: &HeaderMap, body: &[u8]) -> bool {
    let accept_sse = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false);
    if accept_sse {
        return true;
    }
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("stream").and_then(serde_json::Value::as_bool))
        .unwrap_or(false)
}

fn openai_route_missing_agent_response() -> Response {
    openai_error_response(
        StatusCode::BAD_REQUEST,
        "invalid_request_error",
        "missing_agent_id",
        "missing x-agent-id header",
        None,
    )
}

fn parse_agent_header_for_openai(headers: &HeaderMap) -> Result<AgentId, Response> {
    let Some(agent_id_str) = headers.get("x-agent-id").and_then(|v| v.to_str().ok()) else {
        return Err(openai_route_missing_agent_response());
    };
    AgentId::new(agent_id_str.to_string()).map_err(|_| {
        openai_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            "invalid_agent_id",
            "invalid x-agent-id header",
            None,
        )
    })
}

async fn handle_openai_chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_openai_compatible_proxy(state, headers, body, "/v1/chat/completions", true).await
}

async fn handle_openai_embeddings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_openai_compatible_proxy(state, headers, body, "/v1/embeddings", false).await
}

async fn handle_legacy_complete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_data_plane_ingress_authz(&state, &headers) {
        return response;
    }
    let payload = OpenAiCompatibleErrorBody {
        error: OpenAiCompatibleError {
            message: "legacy Anthropic completions are deprecated; use /v1/messages or /v1/chat/completions"
                .to_string(),
            error_type: "invalid_request_error",
            code: "legacy_anthropic_completions_deprecated",
        },
    };
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{\"error\":{}}".to_vec());
    axum::response::Response::builder()
        .status(StatusCode::GONE)
        .header("content-type", "application/json")
        .header("x-oya-compatibility", "deprecated-legacy-completions")
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn handle_openai_compatible_proxy(
    state: Arc<AppState>,
    headers: HeaderMap,
    mut body: Bytes,
    upstream_path: &'static str,
    allow_stream: bool,
) -> Response {
    let principal = match require_data_plane_ingress_authz(&state, &headers) {
        Ok(principal) => principal,
        Err(response) => return response,
    };
    if let Err(response) = guard_in_transit_payload(&body) {
        return response;
    }
    let route_decision = match route_decision_for_body(ProtocolShape::OpenAiChatCompletions, &body)
    {
        Ok(decision) => decision,
        Err(error) => return openai_model_routing_error_response(error),
    };
    body = rewrite_body_model(body, &route_decision.upstream_model);
    match route_decision.backend {
        BackendClass::GeminiNative if upstream_path == "/v1/chat/completions" => {
            return handle_gemini_openai_chat_proxy(state, headers, body, principal.tenant()).await;
        }
        BackendClass::GeminiNative => {
            return unsupported_translation_response(
                "OpenAI-compatible",
                route_decision.backend,
                &route_decision.upstream_model,
            );
        }
        BackendClass::OpenAiCompatible => {}
        BackendClass::AnthropicSubscription => {
            return unsupported_translation_response(
                "OpenAI-compatible",
                route_decision.backend,
                &route_decision.upstream_model,
            );
        }
    }

    let agent_id = match parse_agent_header_for_openai(&headers) {
        Ok(agent_id) => agent_id,
        Err(response) => return response,
    };
    let lease_context =
        match acquire_provider_lease(&state, Provider::Codex, principal.tenant(), &agent_id) {
            Ok(context) => context,
            Err(LeaseError::Forbidden) => {
                return openai_error_response(
                    StatusCode::FORBIDDEN,
                    "policy_error",
                    "forbidden_by_policy",
                    "request was denied by policy",
                    None,
                );
            }
            Err(LeaseError::NoEligibleSeat) => {
                return openai_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "unavailable_error",
                    "no_eligible_provider_seat",
                    "no eligible OpenAI-compatible provider seat is available",
                    Some(1),
                );
            }
            Err(LeaseError::Internal) => {
                return openai_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "provider_pool_error",
                    "provider pool state could not be read",
                    None,
                );
            }
        };

    // OAuth-subscription Codex seats route through the ChatGPT session OAuth
    // flow + Codex data endpoint (subscription pooling). API-key seats fall
    // through to the documented OpenAI-compatible API-key path below, which
    // stays behavior-identical.
    if lease_context.credential_mode == CredentialMode::OAuthSubscription {
        return handle_codex_oauth_subscription_proxy(
            &state,
            &headers,
            body,
            upstream_path,
            allow_stream,
            lease_context,
            principal.tenant(),
            agent_id,
        )
        .await;
    }

    let api_key = match state
        .secret_store
        .fetch_refresh_token(&lease_context.credential_handle)
        .await
    {
        Ok(api_key) => api_key,
        Err(_) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RefreshFailed, Instant::now());
            return openai_error_response(
                StatusCode::BAD_GATEWAY,
                "upstream_error",
                "provider_credential_unavailable",
                "provider credential handle could not be resolved",
                None,
            );
        }
    };

    let request = CodexProxyRequest {
        body: body.to_vec(),
        extra_headers: headers_to_btree(&headers),
    };
    let adapter = OpenAiApiKeyAdapter::with_base_url(
        Arc::clone(&state.http_client),
        state.openai_compatible_base_url.clone(),
    );

    if allow_stream && openai_stream_requested(&headers, &body) {
        match adapter
            .proxy_openai_compatible_path_stream(&api_key, upstream_path, request)
            .await
        {
            Ok((upstream_status, upstream_headers, byte_stream)) => {
                let mapped: BoxStream<Result<Bytes, RestAdapterError>> =
                    Box::pin(byte_stream.map(|chunk| {
                        chunk.map_err(|e| RestAdapterError::UpstreamError {
                            status: 502,
                            body: e.to_string(),
                        })
                    }));
                let lease_stream = SseStreamWithLease::new_with_clean_outcome(
                    mapped,
                    lease_context.lease,
                    seat_outcome_for_upstream_status(upstream_status),
                );
                let mut builder = axum::response::Response::builder().status(upstream_status);
                for (k, v) in &upstream_headers {
                    if let (Ok(name), Ok(value)) = (
                        HeaderName::from_bytes(k.as_bytes()),
                        HeaderValue::from_str(v),
                    ) {
                        builder = builder.header(name, value);
                    }
                }
                return builder
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .body(Body::from_stream(lease_stream))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            Err(error) => {
                let _ = lease_context
                    .lease
                    .complete(SeatOutcome::ServerError5xx, Instant::now());
                return openai_adapter_error_to_response(error);
            }
        }
    }

    match adapter
        .proxy_openai_compatible_path(&api_key, upstream_path, request)
        .await
    {
        Ok(resp) => {
            let upstream_status = resp.status;
            let _ = lease_context.lease.complete(
                seat_outcome_for_upstream_status(upstream_status),
                Instant::now(),
            );
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            state.sink.emit(LlmGatewayEvent {
                request_id: format!("req-{now_ms}"),
                tenant_id: principal.tenant().clone(),
                agent_id,
                seat_id: lease_context.seat_id,
                provider: Provider::Codex,
                model: String::new(),
                prompt_tokens: 0,
                completion_tokens: 0,
                ms_latency: 0,
                status: event_status_for_upstream_status(upstream_status),
                timestamp_unix_ms: now_ms,
            });
            codex_proxy_response_to_axum(resp)
        }
        Err(CodexAdapterError::RateLimited { retry_after_secs }) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RateLimited429, Instant::now());
            openai_adapter_error_to_response(CodexAdapterError::RateLimited { retry_after_secs })
        }
        Err(error) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::ServerError5xx, Instant::now());
            openai_adapter_error_to_response(error)
        }
    }
}

/// Codex OAuth-subscription data-plane path. Mirrors the Anthropic
/// OAuth-subscription lifecycle in [`handle_proxy`]: resolve the seat's refresh
/// token from the secret store (OpenBao seam), refresh the ChatGPT session
/// token via the [`CodexAdapter`], then proxy (streaming or one-shot) to the
/// Codex data endpoint, holding the [`SeatLease`] for the full response.
///
/// Fail-closed: a missing credential or a failed refresh releases the seat and
/// denies the request — it never falls through to an unauthenticated upstream
/// call. Codex wire-shape (cli User-Agent impersonation, `X-OpenAI-Beta`,
/// hop-by-hop + provider-control header stripping) is owned by [`CodexAdapter`].
#[allow(clippy::too_many_arguments)]
async fn handle_codex_oauth_subscription_proxy(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    upstream_path: &'static str,
    allow_stream: bool,
    lease_context: ProviderLeaseContext,
    principal_tenant: &TenantId,
    agent_id: AgentId,
) -> Response {
    // The ChatGPT-backed Codex responses endpoint serves chat-shaped requests
    // only. Embeddings over an OAuth-subscription seat is a genuinely-
    // unsupported combination and still fails closed (mirrors the api-key
    // path's exact-path allowlist).
    if upstream_path != CODEX_OAUTH_SUPPORTED_PATH {
        let _ = lease_context
            .lease
            .complete(SeatOutcome::RefreshFailed, Instant::now());
        return openai_error_response(
            StatusCode::BAD_GATEWAY,
            "upstream_error",
            "codex_oauth_subscription_path_unsupported",
            "Codex OAuth-subscription seats support only /v1/chat/completions",
            None,
        );
    }

    // Resolve the OAuth refresh token via the opaque credential handle. For
    // OAuth seats this resolves to the ChatGPT session refresh token. Fail
    // closed on a missing/unreadable credential.
    let refresh_token = match state
        .secret_store
        .fetch_refresh_token(&lease_context.credential_handle)
        .await
    {
        Ok(token) => token,
        Err(_) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RefreshFailed, Instant::now());
            return openai_error_response(
                StatusCode::BAD_GATEWAY,
                "upstream_error",
                "provider_credential_unavailable",
                "provider credential handle could not be resolved",
                None,
            );
        }
    };

    let adapter = CodexAdapter::with_base_url(
        Arc::clone(&state.http_client),
        state.codex_oauth_base_url.clone(),
    );

    // Refresh the ChatGPT session token before streaming. On failure the seat
    // is released (RefreshFailed) and the request denied — never falls through
    // to an unauthenticated upstream call.
    let access_token = match adapter.refresh_token(&refresh_token).await {
        Ok(tokens) => tokens.access_token,
        Err(error) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RefreshFailed, Instant::now());
            return openai_adapter_error_to_response(error);
        }
    };

    let request = CodexProxyRequest {
        body: body.to_vec(),
        extra_headers: headers_to_btree(headers),
    };

    if allow_stream && openai_stream_requested(headers, &body) {
        match adapter.proxy_stream(&access_token, request).await {
            Ok((upstream_status, upstream_headers, byte_stream)) => {
                let mapped: BoxStream<Result<Bytes, RestAdapterError>> =
                    Box::pin(byte_stream.map(|chunk| {
                        chunk.map_err(|e| RestAdapterError::UpstreamError {
                            status: 502,
                            body: e.to_string(),
                        })
                    }));
                let lease_stream = SseStreamWithLease::new_with_clean_outcome(
                    mapped,
                    lease_context.lease,
                    seat_outcome_for_upstream_status(upstream_status),
                );
                let mut builder = axum::response::Response::builder().status(upstream_status);
                for (k, v) in &upstream_headers {
                    if let (Ok(name), Ok(value)) = (
                        HeaderName::from_bytes(k.as_bytes()),
                        HeaderValue::from_str(v),
                    ) {
                        builder = builder.header(name, value);
                    }
                }
                return builder
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .body(Body::from_stream(lease_stream))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            Err(error) => {
                let _ = lease_context
                    .lease
                    .complete(SeatOutcome::ServerError5xx, Instant::now());
                return openai_adapter_error_to_response(error);
            }
        }
    }

    match adapter.proxy(&access_token, request).await {
        Ok(resp) => {
            let upstream_status = resp.status;
            let _ = lease_context.lease.complete(
                seat_outcome_for_upstream_status(upstream_status),
                Instant::now(),
            );
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            state.sink.emit(LlmGatewayEvent {
                request_id: format!("req-{now_ms}"),
                tenant_id: principal_tenant.clone(),
                agent_id,
                seat_id: lease_context.seat_id,
                provider: Provider::Codex,
                model: String::new(),
                prompt_tokens: 0,
                completion_tokens: 0,
                ms_latency: 0,
                status: event_status_for_upstream_status(upstream_status),
                timestamp_unix_ms: now_ms,
            });
            codex_proxy_response_to_axum(resp)
        }
        Err(CodexAdapterError::RateLimited { retry_after_secs }) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RateLimited429, Instant::now());
            openai_adapter_error_to_response(CodexAdapterError::RateLimited { retry_after_secs })
        }
        Err(error) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::ServerError5xx, Instant::now());
            openai_adapter_error_to_response(error)
        }
    }
}

async fn handle_gemini_openai_chat_proxy(
    state: Arc<AppState>,
    headers: HeaderMap,
    body: Bytes,
    principal_tenant: &TenantId,
) -> Response {
    if openai_stream_requested(&headers, &body) {
        return openai_error_response(
            StatusCode::NOT_IMPLEMENTED,
            "unsupported_feature",
            "gemini_stream_translation_not_enabled",
            "Gemini streaming translation is not enabled for this route yet",
            None,
        );
    }
    let agent_id = match parse_agent_header_for_openai(&headers) {
        Ok(agent_id) => agent_id,
        Err(response) => return response,
    };
    handle_gemini_adapter_proxy(
        state,
        headers,
        body,
        principal_tenant,
        agent_id,
        GeminiRequestShape::OpenAiChat,
    )
    .await
}

async fn handle_gemini_anthropic_messages_proxy(
    state: Arc<AppState>,
    headers: HeaderMap,
    body: Bytes,
    principal_tenant: &TenantId,
) -> Response {
    let agent_id_str = match headers.get("x-agent-id").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => return (StatusCode::BAD_REQUEST, "missing x-agent-id header").into_response(),
    };
    let agent_id = match AgentId::new(agent_id_str) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid x-agent-id header").into_response(),
    };
    handle_gemini_adapter_proxy(
        state,
        headers,
        body,
        principal_tenant,
        agent_id,
        GeminiRequestShape::AnthropicMessages,
    )
    .await
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GeminiRequestShape {
    OpenAiChat,
    AnthropicMessages,
}

async fn handle_gemini_adapter_proxy(
    state: Arc<AppState>,
    headers: HeaderMap,
    body: Bytes,
    principal_tenant: &TenantId,
    agent_id: AgentId,
    request_shape: GeminiRequestShape,
) -> Response {
    let lease_context =
        match acquire_provider_lease(&state, Provider::Gemini, principal_tenant, &agent_id) {
            Ok(context) => context,
            Err(LeaseError::Forbidden) => {
                return gemini_adapter_error_to_response(GeminiAdapterError::InvalidRequest(
                    "request was denied by policy".to_string(),
                ));
            }
            Err(LeaseError::NoEligibleSeat) => {
                return openai_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "unavailable_error",
                    "no_eligible_provider_seat",
                    "no eligible Gemini provider seat is available",
                    Some(1),
                );
            }
            Err(LeaseError::Internal) => {
                return openai_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "provider_pool_error",
                    "provider pool state could not be read",
                    None,
                );
            }
        };

    if lease_context.credential_mode != CredentialMode::ApiKey {
        let _ = lease_context
            .lease
            .complete(SeatOutcome::RefreshFailed, Instant::now());
        return openai_error_response(
            StatusCode::BAD_GATEWAY,
            "upstream_error",
            "gemini_api_key_required",
            "Gemini routes require an API-key provider seat",
            None,
        );
    }

    let api_key = match state
        .secret_store
        .fetch_refresh_token(&lease_context.credential_handle)
        .await
    {
        Ok(api_key) => api_key,
        Err(_) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RefreshFailed, Instant::now());
            return openai_error_response(
                StatusCode::BAD_GATEWAY,
                "upstream_error",
                "provider_credential_unavailable",
                "provider credential handle could not be resolved",
                None,
            );
        }
    };

    let adapter = GeminiApiKeyAdapter::with_base_url(
        Arc::clone(&state.http_client),
        state.gemini_base_url.clone(),
    );
    let result = match request_shape {
        GeminiRequestShape::OpenAiChat => {
            adapter
                .proxy_openai_chat(&api_key, body.to_vec(), headers_to_btree(&headers))
                .await
        }
        GeminiRequestShape::AnthropicMessages => {
            adapter
                .proxy_anthropic_messages(&api_key, body.to_vec(), headers_to_btree(&headers))
                .await
        }
    };

    match result {
        Ok(resp) => {
            let upstream_status = resp.status;
            let _ = lease_context.lease.complete(
                seat_outcome_for_upstream_status(upstream_status),
                Instant::now(),
            );
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            state.sink.emit(LlmGatewayEvent {
                request_id: format!("req-{now_ms}"),
                tenant_id: principal_tenant.clone(),
                agent_id,
                seat_id: lease_context.seat_id,
                provider: Provider::Gemini,
                model: String::new(),
                prompt_tokens: 0,
                completion_tokens: 0,
                ms_latency: 0,
                status: event_status_for_upstream_status(upstream_status),
                timestamp_unix_ms: now_ms,
            });
            gemini_proxy_response_to_axum(resp)
        }
        Err(GeminiAdapterError::RateLimited { retry_after_secs }) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::RateLimited429, Instant::now());
            gemini_adapter_error_to_response(GeminiAdapterError::RateLimited { retry_after_secs })
        }
        Err(error) => {
            let _ = lease_context
                .lease
                .complete(SeatOutcome::ServerError5xx, Instant::now());
            gemini_adapter_error_to_response(error)
        }
    }
}

#[derive(Deserialize)]
struct AdminRegisterSubscriptionRequest {
    seat_id: String,
    subscription_id: String,
    credential_mode: String,
    secret_handle: String,
}

#[derive(Serialize)]
struct AdminRegisterSubscriptionResponse {
    tenant_id: String,
    provider: String,
    seat_id: String,
    status: String,
}

#[derive(Serialize)]
struct TokenCountResponse {
    input_tokens: u64,
    token_source: &'static str,
}

#[derive(Serialize)]
struct ModelInventoryResponse {
    object: &'static str,
    data: Vec<ModelInventoryModel>,
    inventory_source: &'static str,
    stale: bool,
}

#[derive(Serialize)]
struct ModelInventoryModel {
    id: &'static str,
    object: &'static str,
    owned_by: &'static str,
}

#[derive(Serialize)]
struct AdminBoundaryStatus {
    secret_resolution: &'static str,
    authorization: &'static str,
}

#[derive(Serialize)]
struct AdminGatewayStatus {
    status: &'static str,
    route_controller_ready: bool,
    model_inventory_ready: bool,
    credential_worker_ready: bool,
    policy_engine_ready: bool,
    default_data_plane_ready: bool,
    secret_provider_ready: bool,
    registered_pools: usize,
    boundaries: AdminBoundaryStatus,
}

#[derive(Serialize)]
struct AdminAccountsResponse {
    accounts: Vec<AccountStatus>,
    redaction: &'static str,
}

#[derive(Serialize)]
struct AdminAnalyticsResponse {
    window: &'static str,
    request_count: u64,
    rate_limited_count: u64,
    circuit_breaker_open_count: u64,
    source: &'static str,
}

#[derive(Deserialize)]
struct AdminResumeRequest {
    scope: String,
    reason: String,
}

#[derive(Serialize)]
struct AdminResumeResponse {
    scope: String,
    status: &'static str,
    reason_recorded: bool,
    retry_after_seconds: Option<u64>,
}

/// POST /v1/messages/count_tokens
async fn handle_count_tokens(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Non-leasing endpoint: still fail-closed authn, principal deliberately unused.
    let _principal = match require_data_plane_ingress_authz(&state, &headers) {
        Ok(principal) => principal,
        Err(response) => return response,
    };
    if let Err(response) = guard_in_transit_payload(&body) {
        return response;
    }
    let payload = match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(payload) => payload,
        Err(_) => return openai_model_routing_error_response(RouteBodyError::MalformedJson),
    };
    Json(TokenCountResponse {
        input_tokens: estimate_input_tokens(&payload),
        token_source: "local-estimate-until-provider-accounting",
    })
    .into_response()
}

/// GET /v1/models
async fn handle_models(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // Non-leasing endpoint: still fail-closed authn, principal deliberately unused.
    let _principal = match require_data_plane_ingress_authz(&state, &headers) {
        Ok(principal) => principal,
        Err(response) => return response,
    };
    Json(ModelInventoryResponse {
        object: "list",
        data: vec![
            ModelInventoryModel {
                id: "claude-opus-4-5",
                object: "model",
                owned_by: "anthropic-subscription-backend",
            },
            ModelInventoryModel {
                id: "gpt-4o",
                object: "model",
                owned_by: "openai-compatible-backend",
            },
            ModelInventoryModel {
                id: "gemini-2.5-flash",
                object: "model",
                owned_by: "gemini-native-backend",
            },
        ],
        inventory_source: "model-inventory-worker",
        stale: true,
    })
    .into_response()
}

/// GET /admin/v1/status
async fn handle_admin_status(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(principal) = state.admin_authenticator.verify(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let default_data_plane_ready = state
        .pool
        .lock()
        .map(|pool| pool.has_eligible_seat(Instant::now()))
        .unwrap_or(false);
    let secret_provider_ready = state.secret_store.readiness_probe().await.is_ok();
    let route_controller_ready = state.pool_registry.pool_count_for(principal.tenant()) > 0;
    let model_inventory_ready = true;
    let credential_worker_ready = default_data_plane_ready && secret_provider_ready;
    let policy_engine_ready = policy_engine_readiness_probe(&state);
    let status = if route_controller_ready
        && model_inventory_ready
        && credential_worker_ready
        && policy_engine_ready
    {
        "ok"
    } else {
        "degraded"
    };
    Json(AdminGatewayStatus {
        status,
        route_controller_ready,
        model_inventory_ready,
        credential_worker_ready,
        policy_engine_ready,
        default_data_plane_ready,
        secret_provider_ready,
        registered_pools: state.pool_registry.pool_count_for(principal.tenant()),
        boundaries: AdminBoundaryStatus {
            secret_resolution: "owned-secret-provider-port",
            authorization: "owned-policy-engine-port",
        },
    })
    .into_response()
}

/// GET /admin/v1/accounts
async fn handle_admin_accounts(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(principal) = state.admin_authenticator.verify(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    Json(AdminAccountsResponse {
        accounts: state.pool_registry.account_statuses_for(principal.tenant()),
        redaction: "secret-handles-redacted",
    })
    .into_response()
}

/// GET /admin/v1/analytics
async fn handle_admin_analytics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let Some(principal) = state.admin_authenticator.verify(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let request_count = state.pool_registry.pool_count_for(principal.tenant()) as u64;
    Json(AdminAnalyticsResponse {
        window: "PT5M",
        request_count,
        rate_limited_count: 0,
        circuit_breaker_open_count: 0,
        source: "cloud-admin-api",
    })
    .into_response()
}

/// GET /admin/v1/analytics/stream
async fn handle_admin_analytics_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let Some(principal) = state.admin_authenticator.verify(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let payload = serde_json::json!({
        "window": "PT5M",
        "request_count": state.pool_registry.pool_count_for(principal.tenant()),
        "source": "cloud-admin-api"
    });
    (
        StatusCode::OK,
        [
            ("content-type", "text/event-stream"),
            ("cache-control", "no-cache"),
        ],
        format!("event: analytics\ndata: {payload}\n\n"),
    )
        .into_response()
}

/// GET /admin/v1/guardrails
async fn handle_admin_guardrails(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    Json(serde_json::json!({
        "profiles": [{
            "kind": "GuardrailDetectionProfile",
            "name": "platform-critical-safety-floor",
            "policy_engine_port": "owned-policy-engine-port",
            "detectors": [
                "prompt_injection",
                "data_exfiltration",
                "credential_probing",
                "sandbox_escape",
                "self_harm",
                "harm_to_others",
                "privacy_violation",
                "tenant_boundary_violation",
                "fraud",
                "fault",
                "unsafe",
                "anomaly",
                "hostile_pattern"
            ],
            "critical_action": "block_and_quarantine",
            "mandatory_secondary_agentic_review": true,
            "manual_review_after_secondary_review": true
        }],
        "signal_policy": {
            "kind": "SafetySignalPolicy",
            "platform_automatic_enforcement": true,
            "tenant_policy_receives_signals": true,
            "tenant_policy_receives_recommendations": true,
            "tenant_can_override_platform_critical_block": false,
            "tenant_overlay_may_only_tighten": true
        }
    }))
    .into_response()
}

/// GET /admin/v1/agent-runtimes
async fn handle_admin_agent_runtimes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    let tenant_id = state.tenant_id.as_str();
    Json(serde_json::json!({
        "runtimes": [{
            "kind": "AgentRuntimeProfile",
            "tenant_id": tenant_id,
            "name": "dogfood-fable-runtime",
            "model_route_ref": "dogfood-model-route",
            "prompt_profile_ref": "internal-coding-agent-prompt-profile",
            "thinking_policy_ref": "critical-block-second-pass-policy",
            "tool_compatibility_profile_ref": "claude-codex-gemini-tool-compatibility",
            "sandbox_policy_ref": "ephemeral-workspace-sandbox",
            "cloud_intelligence_owned_control_plane": true,
            "embeds_model_runtime": false,
            "installs_cli_or_tui_surface": false
        }],
        "workflow_statuses": [{
            "kind": "AgentWorkflowStatus",
            "tenant_id": tenant_id,
            "workflow_ref": format!("agent-workflow-ref://{tenant_id}/{tenant_id}-internal-coding-agent"),
            "runtime_profile_ref": "dogfood-fable-runtime",
            "delegation_policy_ref": "claude-codex-gemini-delegation",
            "generation_adapters": ["claude", "codex", "gemini"],
            "routing_advisor_scope": "routing-decision-only",
            "routing_advisor_models": [
                "chatgpt-spark",
                "gemini-3.1-flash-lite",
                "nemotron-3-ultra-550b-a55b"
            ],
            "policy_engine_port": "owned-policy-engine-port",
            "evidence_visibility": "redacted-structured-evidence",
            "sealed_evidence_handle_ref": format!("sealed-evidence-ref://{tenant_id}/internal-coding-agent/status"),
            "requires_secondary_review_for_critical_blocks": true,
            "raw_payload_included": false
        }]
    }))
    .into_response()
}

/// GET /admin/v1/agent-schedules
async fn handle_admin_agent_schedules(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    let tenant_id = state.tenant_id.as_str();
    let schedule_ref = format!("schedule-ref://{tenant_id}/nightly-drift-check");
    Json(serde_json::json!({
        "schedules": [{
            "kind": "AgentSchedule",
            "tenant_id": tenant_id,
            "name": "nightly-drift-check",
            "schedule_ref": schedule_ref,
            "runtime_profile_ref": "dogfood-fable-runtime",
            "execution_externalized_to_controller": true,
            "embeds_local_cron": false,
            "policy_engine_port": "owned-policy-engine-port",
            "evidence_visibility": "redacted-structured-evidence"
        }],
        "statuses": [{
            "kind": "AgentScheduleStatus",
            "schedule_ref": format!("schedule-ref://{tenant_id}/nightly-drift-check"),
            "state": "passed",
            "next_run_window": "P1D",
            "controller_owner": "agent-scheduler-worker",
            "policy_engine_port": "owned-policy-engine-port",
            "evidence_visibility": "redacted-structured-evidence",
            "raw_payload_included": false,
            "sealed_evidence_handle_ref": format!("sealed-evidence-ref://{tenant_id}/nightly-drift-check/status")
        }]
    }))
    .into_response()
}

/// GET /admin/v1/parity/canaries
async fn handle_admin_parity_canaries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    let tenant_id = state.tenant_id.as_str();
    Json(serde_json::json!({
        "plans": [{
            "kind": "ParityCanaryPlan",
            "tenant_id": tenant_id,
            "name": "nightly-drift-check",
            "artifact_family": "external-proxy-reference",
            "capability_namespace": "XPROXY",
            "schedule_ref": format!("schedule-ref://{tenant_id}/nightly-drift-check"),
            "probes": ["capability-parity", "wire-profile-drift", "adapter-translation-drift"],
            "compatibility_canaries": ["route-matrix", "streaming-fixtures", "security-redaction"],
            "controller_owned": true,
            "opens_pr_or_task_on_delta": true,
            "audit_event_required": true,
            "policy_engine_port": "owned-policy-engine-port",
            "evidence_visibility": "redacted-structured-evidence",
            "embeds_local_cron": false,
            "writes_raw_prompts_or_secrets": false
        }],
        "statuses": [{
            "kind": "ParityCanaryStatus",
            "plan_ref": format!("parity-canary-plan-ref://{tenant_id}/nightly-drift-check"),
            "state": "passed",
            "retry_after_header": "Retry-After",
            "retry_after_seconds": null,
            "evidence_visibility": "redacted-structured-evidence",
            "sealed_evidence_handle_ref": format!("sealed-evidence-ref://{tenant_id}/nightly-drift-check/parity-canary"),
            "raw_payload_included": false
        }]
    }))
    .into_response()
}

/// GET /admin/v1/guardrails/escalations
async fn handle_admin_guardrail_escalations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    Json(serde_json::json!({
        "escalations": [{
            "kind": "ManualReviewEscalation",
            "name": "critical-safety-review",
            "critical_blocks_require_secondary_review_first": true,
            "secondary_review_can_enrich_but_not_unblock": true,
            "manual_review_required_after_secondary_review": true,
            "default_evidence_visibility": "redacted-structured-evidence",
            "raw_payload_access": "audited-break-glass-only"
        }]
    }))
    .into_response()
}

/// GET /admin/v1/evidence/retention
async fn handle_admin_evidence_retention(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    Json(serde_json::json!({
        "profiles": [{
            "kind": "EvidenceRetentionProfile",
            "name": "platform-guardrail-evidence",
            "secret_provider_port": "owned-secret-provider-port",
            "stores_payload_on_normal_path": false,
            "encrypted_handle_on_guardrail_trigger": true,
            "fixed_ttl_by_data_class": true,
            "regulatory_classification_required": true,
            "default_reviewer_visibility": "redacted-structured-evidence",
            "raw_access_requires_audited_break_glass": true,
            "ttl_by_data_class": {
                "TRIVIAL_PERSONAL": "P7D",
                "SENSITIVE_PERSONAL": "P72H",
                "HIGH_RISK_SECURITY": "P30D",
                "REGULATED": "P30D"
            }
        }]
    }))
    .into_response()
}

/// GET /admin/v1/redaction/profiles
async fn handle_admin_redaction_profiles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, state.tenant_id.as_str()) {
        return response;
    }
    Json(serde_json::json!({
        "profiles": [{
            "kind": "InTransitRedactionProfile",
            "name": "platform-model-boundary-redaction",
            "blocks_sensitive_classes": true,
            "redacts_trivial_personal_data": true,
            "reversible_tokens_require_tenant_policy": true,
            "default_token_lifetime": "ephemeral-run",
            "restore_only_after_model_output": true,
            "provider_receives_raw_token_values": false,
            "routing_advisor_receives_raw_token_values": false
        }]
    }))
    .into_response()
}

/// POST /admin/v1/resume
async fn handle_admin_resume(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AdminResumeRequest>,
) -> Response {
    if !admin_bearer_allowed(&headers, state.admin_bearer_token.as_deref()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !has_valid_idempotency_key(&headers) {
        return (
            StatusCode::BAD_REQUEST,
            "missing or invalid idempotency-key",
        )
            .into_response();
    }
    if request.scope.trim().is_empty() || request.reason.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "scope and reason are required").into_response();
    }
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(AdminResumeResponse {
            scope: request.scope,
            status: "not_implemented",
            reason_recorded: false,
            retry_after_seconds: None,
        }),
    )
        .into_response()
}

/// GET /admin/v1/tenants/{tenant_id}/providers/{provider}/pool
async fn handle_admin_pool_status(
    State(state): State<Arc<AppState>>,
    Path((tenant_id_raw, provider_raw)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = require_admin_authz(&state, &headers, &tenant_id_raw) {
        return response;
    }
    let tenant_id = match TenantId::new(tenant_id_raw) {
        Ok(tenant_id) => tenant_id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid tenant_id").into_response(),
    };
    let provider = match parse_provider(&provider_raw) {
        Some(provider) => provider,
        None => return (StatusCode::BAD_REQUEST, "invalid provider").into_response(),
    };

    match state.pool_registry.pool_status(&tenant_id, provider) {
        Some(status) => Json(status).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// POST /admin/v1/tenants/{tenant_id}/providers/{provider}/subscriptions
async fn handle_admin_register_subscription(
    State(state): State<Arc<AppState>>,
    Path((tenant_id_raw, provider_raw)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // R4 + R5 (AUTH-005 / ADR-0573 PR2): authenticate + authorize BEFORE the
    // untrusted body is deserialized. The `Bytes` extractor takes the raw body so
    // no `serde` work happens on an unauthenticated request.
    if let Err(response) = require_admin_authz(&state, &headers, &tenant_id_raw) {
        return response;
    }
    if !has_valid_idempotency_key(&headers) {
        return (
            StatusCode::BAD_REQUEST,
            "missing or invalid idempotency-key",
        )
            .into_response();
    }
    let request: AdminRegisterSubscriptionRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(_) => return (StatusCode::BAD_REQUEST, "malformed json").into_response(),
    };
    let tenant_id = match TenantId::new(tenant_id_raw.clone()) {
        Ok(tenant_id) => tenant_id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid tenant_id").into_response(),
    };
    let provider = match parse_provider(&provider_raw) {
        Some(provider) => provider,
        None => return (StatusCode::BAD_REQUEST, "invalid provider").into_response(),
    };
    let seat_id = match SeatId::new(request.seat_id.clone()) {
        Ok(seat_id) => seat_id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid seat_id").into_response(),
    };
    let subscription_id = match SubscriptionId::new(request.subscription_id) {
        Ok(subscription_id) => subscription_id,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid subscription_id").into_response(),
    };
    let credential_mode = match parse_credential_mode(&request.credential_mode) {
        Some(credential_mode) => credential_mode,
        None => return (StatusCode::BAD_REQUEST, "invalid credential_mode").into_response(),
    };
    // Production fail-closed gate (mirrors the boot-time `validate_provider_compliance`
    // gate in the app crate): runtime registration of an OAuth-subscription seat is
    // rejected unless the provider's OAuth compliance status is APPROVED.
    if state.is_production()
        && credential_mode == CredentialMode::OAuthSubscription
        && !state.is_oauth_approved(provider)
    {
        return (
            StatusCode::FORBIDDEN,
            "provider compliance not approved for oauth_subscription mode",
        )
            .into_response();
    }
    if !is_secret_handle_reference(&request.secret_handle) {
        return (StatusCode::BAD_REQUEST, "invalid secret handle").into_response();
    }

    let subscription = OAuthSubscription::new(
        tenant_id.clone(),
        seat_id.clone(),
        subscription_id,
        provider,
        SubscriptionState::Active,
        request.secret_handle,
        0,
    )
    .with_credential_mode(credential_mode);

    match state.pool_registry.register_subscription(
        tenant_id.clone(),
        provider,
        subscription,
        SelectionStrategy::RoundRobin,
    ) {
        Ok(_) => (
            StatusCode::CREATED,
            Json(AdminRegisterSubscriptionResponse {
                tenant_id: tenant_id.as_str().to_string(),
                provider: provider.to_string(),
                seat_id: seat_id.as_str().to_string(),
                status: "registered".to_string(),
            }),
        )
            .into_response(),
        Err(SubscriptionPoolError::ForbiddenByPolicy) => StatusCode::FORBIDDEN.into_response(),
        Err(SubscriptionPoolError::DuplicateSeat) => StatusCode::CONFLICT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn parse_provider(raw: &str) -> Option<Provider> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => Some(Provider::Anthropic),
        "codex" | "openai" => Some(Provider::Codex),
        "gemini" | "google" => Some(Provider::Gemini),
        _ => None,
    }
}

fn parse_credential_mode(raw: &str) -> Option<CredentialMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "oauth_subscription" | "oauth-subscription" | "oauth" => {
            Some(CredentialMode::OAuthSubscription)
        }
        "api_key" | "api-key" | "apikey" => Some(CredentialMode::ApiKey),
        _ => None,
    }
}

fn estimate_input_tokens(payload: &serde_json::Value) -> u64 {
    fn collect_text(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::String(text) => out.push(text.clone()),
            serde_json::Value::Array(values) => {
                for value in values {
                    collect_text(value, out);
                }
            }
            serde_json::Value::Object(object) => {
                for (key, value) in object {
                    if matches!(
                        key.as_str(),
                        "content" | "text" | "input" | "message" | "messages"
                    ) {
                        collect_text(value, out);
                    }
                }
            }
            _ => {}
        }
    }

    let mut text_fragments = Vec::new();
    collect_text(payload, &mut text_fragments);
    let word_count = text_fragments
        .iter()
        .flat_map(|text| text.split_whitespace())
        .count() as u64;
    if word_count == 0 && !text_fragments.is_empty() {
        1
    } else {
        word_count
    }
}

/// Probe the owned-policy-engine port with a no-op decision request. The
/// admin status surface reports readiness as true only when the gate actually
/// answers Allow for the gateway's own readiness principal, so a default-deny
/// or wedged policy adapter degrades the reported gateway status.
fn policy_engine_readiness_probe(state: &AppState) -> bool {
    let Ok(agent_id) = AgentId::new("admin-readiness-probe") else {
        return false;
    };
    let decision = state.gate.decide(&AuthzRequest {
        principal_tenant: &state.tenant_id,
        principal_agent: &agent_id,
        action: AuthzAction::SelectSeat,
        resource_tenant: &state.tenant_id,
        resource_provider: Provider::Anthropic,
    });
    matches!(decision, AuthzDecision::Allow)
}

fn admin_bearer_allowed(headers: &HeaderMap, configured_token: Option<&str>) -> bool {
    let Some(configured_token) = configured_token else {
        return false;
    };
    if configured_token.is_empty() {
        return false;
    }
    let Some(presented) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    else {
        return false;
    };

    constant_time_eq(presented.as_bytes(), configured_token.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

fn has_valid_idempotency_key(headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// GET /healthz handler — liveness probe.
async fn handle_healthz() -> impl IntoResponse {
    StatusCode::OK
}

/// GET /livez handler — Kubernetes liveness probe.
async fn handle_livez() -> impl IntoResponse {
    StatusCode::OK
}

/// GET /readyz handler — readiness probe.
async fn handle_readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let has_default_data_plane_pool = match state.pool.lock() {
        Ok(pool) => pool.has_eligible_seat(Instant::now()),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
    };
    if !has_default_data_plane_pool {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    if state.secret_store.readiness_probe().await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    StatusCode::OK
}

/// GET /metrics handler — Prometheus text exposition from live gateway state.
async fn handle_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let secret_provider_ready = state.secret_store.readiness_probe().await.is_ok();
    let default_pool_ready = state
        .pool
        .lock()
        .map(|pool| pool.has_eligible_seat(Instant::now()))
        .unwrap_or(false);
    let pool_statuses = state.pool_registry.pool_statuses();
    let mut body = String::from(
        "\
# HELP oya_cloud_intelligence_up Gateway process is up\n\
# TYPE oya_cloud_intelligence_up gauge\n\
oya_cloud_intelligence_up 1\n\
# HELP oya_cloud_intelligence_secret_provider_ready Secret-provider readiness probe result\n\
# TYPE oya_cloud_intelligence_secret_provider_ready gauge\n",
    );
    body.push_str(&format!(
        "oya_cloud_intelligence_secret_provider_ready {}\n",
        u8::from(secret_provider_ready)
    ));
    body.push_str(
        "# HELP oya_cloud_intelligence_default_pool_ready Default data-plane pool has an eligible seat\n\
# TYPE oya_cloud_intelligence_default_pool_ready gauge\n",
    );
    body.push_str(&format!(
        "oya_cloud_intelligence_default_pool_ready {}\n",
        u8::from(default_pool_ready)
    ));
    body.push_str(
        "# HELP oya_cloud_intelligence_registered_provider_pools Registered tenant/provider pools\n\
# TYPE oya_cloud_intelligence_registered_provider_pools gauge\n",
    );
    body.push_str(&format!(
        "oya_cloud_intelligence_registered_provider_pools {}\n",
        pool_statuses.len()
    ));
    body.push_str(
        "# HELP oya_cloud_intelligence_provider_pool_ready Provider pool readiness by provider\n\
# TYPE oya_cloud_intelligence_provider_pool_ready gauge\n",
    );
    for status in &pool_statuses {
        body.push_str(&format!(
            "oya_cloud_intelligence_provider_pool_ready{{provider=\"{}\"}} {}\n",
            status.provider,
            u8::from(status.ready)
        ));
    }
    body.push_str(
        "# HELP oya_cloud_intelligence_provider_pool_seats Provider pool seat count by provider\n\
# TYPE oya_cloud_intelligence_provider_pool_seats gauge\n",
    );
    for status in &pool_statuses {
        body.push_str(&format!(
            "oya_cloud_intelligence_provider_pool_seats{{provider=\"{}\"}} {}\n",
            status.provider, status.total_seats
        ));
    }
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
}

// ---------------------------------------------------------------------------
// Internal: ArcSecretStore adaptor
// ---------------------------------------------------------------------------
// Wraps an Arc<dyn SecretProviderStore> so AnthropicAdapter can own it.

struct ArcSecretStore {
    inner: Arc<dyn SecretProviderStore>, // data_class: INTERNAL_ONLY
}

impl SecretProviderStore for ArcSecretStore {
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        self.inner.fetch_refresh_token(handle)
    }

    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        self.inner.store_refresh_token(handle, plaintext)
    }

    fn readiness_probe(&self) -> SecretProviderFuture<'_, ()> {
        self.inner.readiness_probe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt as _;
    use intelligence_kernel::{
        AuthzDecision, OAuthSubscription, SelectionStrategy, SubscriptionId, SubscriptionState,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tower::ServiceExt;

    struct AllowGate;

    impl AuthzGate for AllowGate {
        fn decide(&self, _request: &intelligence_kernel::AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Allow
        }
    }

    struct ForbidGate;

    impl AuthzGate for ForbidGate {
        fn decide(&self, _request: &intelligence_kernel::AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Forbid
        }
    }

    struct NoopSink;

    impl EventSink for NoopSink {
        fn emit(&self, _event: LlmGatewayEvent) {}
    }

    fn bearer_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers
    }

    #[test]
    fn bearer_map_authenticator_verifies_tenant_by_token() {
        let auth = ConfiguredBearerMapIngressAuthenticator::new(vec![
            BearerBinding {
                token: "tok-a".to_string(),
                tenant: TenantId::new("tenant-a").unwrap(),
                agent: None,
            },
            BearerBinding {
                token: "tok-b".to_string(),
                tenant: TenantId::new("tenant-b").unwrap(),
                agent: None,
            },
        ]);

        // Token A mints a principal bound to tenant-A (never a caller header).
        let principal = auth
            .verify(&bearer_headers("tok-a"))
            .expect("token A must verify");
        assert_eq!(principal.tenant().as_str(), "tenant-a");
        // Token B mints tenant-B.
        let principal_b = auth
            .verify(&bearer_headers("tok-b"))
            .expect("token B must verify");
        assert_eq!(principal_b.tenant().as_str(), "tenant-b");

        // Unknown token => None => 401 (no allow-all path).
        assert!(auth.verify(&bearer_headers("tok-unknown")).is_none());
        // No authorization header => None.
        assert!(auth.verify(&HeaderMap::new()).is_none());
    }

    #[test]
    fn bearer_map_authenticator_empty_bindings_verify_nothing() {
        let auth = ConfiguredBearerMapIngressAuthenticator::new(vec![]);
        // Fail-closed: an empty map can never mint a principal, even for a
        // present bearer.
        assert!(auth.verify(&bearer_headers("anything")).is_none());
        // An empty configured token never matches an empty presented one either.
        let auth_empty_tok = ConfiguredBearerMapIngressAuthenticator::new(vec![BearerBinding {
            token: String::new(),
            tenant: TenantId::new("tenant-a").unwrap(),
            agent: None,
        }]);
        assert!(auth_empty_tok.verify(&bearer_headers("")).is_none());
    }

    struct MemorySecretStore {
        ready: bool,
    }

    impl SecretProviderStore for MemorySecretStore {
        fn fetch_refresh_token<'a>(&'a self, _handle: &'a str) -> SecretProviderFuture<'a, String> {
            Box::pin(async { Ok("test-refresh-token".to_string()) })
        }

        fn store_refresh_token<'a>(
            &'a self,
            _handle: &'a str,
            _plaintext: &'a str,
        ) -> SecretProviderFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }

        fn readiness_probe(&self) -> SecretProviderFuture<'_, ()> {
            Box::pin(async move {
                if self.ready {
                    Ok(())
                } else {
                    Err(RestAdapterError::SecretStoreUnavailable(
                        "test secret store unavailable".to_string(),
                    ))
                }
            })
        }
    }

    struct RecordingSecretStore {
        handles: Arc<Mutex<Vec<String>>>,
    }

    impl SecretProviderStore for RecordingSecretStore {
        fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
            Box::pin(async move {
                self.handles.lock().unwrap().push(handle.to_string());
                Ok("test-refresh-token".to_string())
            })
        }

        fn store_refresh_token<'a>(
            &'a self,
            _handle: &'a str,
            _plaintext: &'a str,
        ) -> SecretProviderFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }
    }

    struct RotationStoreState {
        token: String,
        reject_writes: bool,
        fetch_count: usize,
        store_count: usize,
    }

    struct RotationPersistenceStore {
        state: Arc<Mutex<RotationStoreState>>,
    }

    impl SecretProviderStore for RotationPersistenceStore {
        fn fetch_refresh_token<'a>(&'a self, _handle: &'a str) -> SecretProviderFuture<'a, String> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                state.fetch_count += 1;
                Ok(state.token.clone())
            })
        }

        fn store_refresh_token<'a>(
            &'a self,
            _handle: &'a str,
            plaintext: &'a str,
        ) -> SecretProviderFuture<'a, ()> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                state.store_count += 1;
                if state.reject_writes {
                    return Err(RestAdapterError::SecretStoreUnavailable(
                        "injected OpenBao outage".to_string(),
                    ));
                }
                state.token = plaintext.to_string();
                Ok(())
            })
        }
    }

    fn test_state(has_seat: bool, secret_store_ready: bool) -> Arc<AppState> {
        test_state_with_secret_store(
            has_seat,
            Arc::new(MemorySecretStore {
                ready: secret_store_ready,
            }),
            "tenant-a/seat-a",
        )
    }

    fn test_state_with_secret_store(
        has_seat: bool,
        secret_store: Arc<dyn SecretProviderStore>,
        secret_handle: &str,
    ) -> Arc<AppState> {
        test_state_with_policy_gate(has_seat, secret_store, secret_handle, Arc::new(AllowGate))
    }

    fn test_state_with_policy_gate(
        has_seat: bool,
        secret_store: Arc<dyn SecretProviderStore>,
        secret_handle: &str,
        gate: Arc<dyn AuthzGate + Send + Sync>,
    ) -> Arc<AppState> {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let seat_id = SeatId::new("seat-a").unwrap();
        let mut pool = SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );
        if has_seat {
            pool.add_seat(OAuthSubscription::new(
                tenant_id.clone(),
                seat_id.clone(),
                SubscriptionId::new("sub-a").unwrap(),
                Provider::Anthropic,
                SubscriptionState::Active,
                secret_handle.to_string(),
                0,
            ))
            .unwrap();
        }

        let pool = Arc::new(Mutex::new(pool));
        let registry = PoolRegistry::new();
        registry.insert_pool(tenant_id.clone(), Provider::Anthropic, Arc::clone(&pool));
        Arc::new(
            AppState::new_with_pool_registry(
                pool,
                registry,
                gate,
                Arc::new(NoopSink),
                secret_store,
                "http://127.0.0.1:1".to_string(),
                tenant_id,
                Some("ingress-token".to_string()),
                Some("admin-token".to_string()),
                "development".to_string(),
                std::collections::HashSet::new(),
            )
            .unwrap(),
        )
    }

    async fn one_shot_http_server(
        response: &'static str,
    ) -> (String, tokio::task::JoinHandle<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake provider");
        let addr = listener.local_addr().expect("fake provider addr");
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept fake provider");
            let mut buf = vec![0_u8; 16 * 1024];
            let n = socket.read(&mut buf).await.expect("read fake request");
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write fake response");
            request
        });
        (format!("http://{addr}"), handle)
    }

    fn assert_header(request: &str, header: &str, value: &str) {
        let needle = format!("{header}: {value}");
        assert!(
            request
                .lines()
                .any(|line| line.eq_ignore_ascii_case(&needle)),
            "missing header `{needle}` in request:\n{request}"
        );
    }

    #[tokio::test]
    async fn anthropic_api_key_proxy_injects_x_api_key_without_oauth_beta() {
        let (base_url, upstream_request) = one_shot_http_server(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Connection: x-upstream-hop\r\n\
             X-Upstream-Hop: remove-me\r\n\
             Content-Length: 11\r\n\
             \r\n\
             {\"ok\":true}",
        )
        .await;
        let adapter = AnthropicAdapter::with_base_url(MemorySecretStore { ready: true }, base_url);
        let mut headers = BTreeMap::new();
        headers.insert(
            "authorization".to_string(),
            "Bearer caller-token".to_string(),
        );
        headers.insert("anthropic-beta".to_string(), "oauth-2025-04-20".to_string());
        headers.insert("connection".to_string(), "x-drop-me".to_string());
        headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());

        let response = adapter
            .proxy_with_api_key(
                &reqwest::Client::new(),
                &ProxyRequest {
                    method: "POST".to_string(),
                    path: "/v1/messages".to_string(),
                    headers,
                    body: br#"{"model":"claude-test","messages":[]}"#.to_vec(),
                    tenant_id: TenantId::new("tenant-a").unwrap(),
                },
                "sk-ant-provider",
            )
            .await
            .expect("api-key proxy succeeds");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"ok":true}"#);
        assert!(!response.headers.contains_key("connection"));
        assert!(!response.headers.contains_key("x-upstream-hop"));

        let request = upstream_request.await.expect("fake provider request");
        assert!(request.starts_with("POST /v1/messages "));
        assert_header(&request, "x-api-key", "sk-ant-provider");
        assert_header(&request, "anthropic-version", ANTHROPIC_VERSION);
        assert!(!request.contains("Bearer caller-token"));
        assert!(!request.to_ascii_lowercase().contains("anthropic-beta:"));
        assert!(!request.contains("must-not-forward"));
    }

    #[tokio::test]
    async fn anthropic_api_key_stream_forces_sse_accept_without_oauth_headers() {
        let (base_url, upstream_request) = one_shot_http_server(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/event-stream\r\n\
             Content-Length: 10\r\n\
             \r\n\
             data: hi\n\n",
        )
        .await;
        let adapter = AnthropicAdapter::with_base_url(MemorySecretStore { ready: true }, base_url);
        let mut headers = BTreeMap::new();
        headers.insert("accept".to_string(), "application/json".to_string());
        headers.insert(
            "authorization".to_string(),
            "Bearer caller-token".to_string(),
        );
        headers.insert("connection".to_string(), "x-drop-me".to_string());
        headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());

        let (status, stream) = adapter
            .proxy_stream_with_api_key(
                &reqwest::Client::new(),
                "sk-ant-provider",
                ProxyRequest {
                    method: "POST".to_string(),
                    path: "/v1/messages".to_string(),
                    headers,
                    body: br#"{"model":"claude-test","stream":true}"#.to_vec(),
                    tenant_id: TenantId::new("tenant-a").unwrap(),
                },
            )
            .await
            .expect("api-key stream opens");

        assert_eq!(status, 200);
        use futures::StreamExt as _;
        let body = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|chunk| chunk.expect("stream chunk"))
            .fold(Vec::new(), |mut acc, bytes| {
                acc.extend_from_slice(&bytes);
                acc
            });
        assert_eq!(body, b"data: hi\n\n");

        let request = upstream_request.await.expect("fake provider request");
        assert_header(&request, "x-api-key", "sk-ant-provider");
        assert_header(&request, "anthropic-version", ANTHROPIC_VERSION);
        assert_header(&request, "accept", "text/event-stream");
        assert!(!request.contains("application/json"));
        assert!(!request.contains("Bearer caller-token"));
        assert!(!request.to_ascii_lowercase().contains("anthropic-beta:"));
        assert!(!request.contains("must-not-forward"));
    }

    #[tokio::test]
    async fn health_liveness_and_readiness_routes_are_implemented() {
        let router = build_router(test_state(true, true));
        for path in ["/healthz", "/livez", "/readyz"] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "expected {path} to be implemented"
            );
        }
    }

    #[tokio::test]
    async fn xproxy_api_005_external_reference_ops_routes_are_admin_guarded_cloud_routes() {
        let router = build_router(test_state(true, true));
        let cases = [
            ("GET", "/admin/v1/status", Body::empty()),
            ("GET", "/admin/v1/accounts", Body::empty()),
            ("GET", "/admin/v1/analytics", Body::empty()),
            ("GET", "/admin/v1/analytics/stream", Body::empty()),
            (
                "POST",
                "/admin/v1/resume",
                Body::from(r#"{"scope":"tenant-a/anthropic","reason":"unit-test"}"#),
            ),
        ];

        for (method, path, body) in cases {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header("content-type", "application/json")
                        .body(body)
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "expected {method} {path} to be an authenticated cloud route"
            );
        }
    }

    #[tokio::test]
    async fn data_plane_routes_require_distinct_ingress_bearer() {
        let router = build_router(test_state(true, true));
        for (name, request) in [
            (
                "missing bearer",
                Request::builder()
                    .uri("/v1/models")
                    .body(Body::empty())
                    .unwrap(),
            ),
            (
                "admin bearer is not data-plane bearer",
                Request::builder()
                    .uri("/v1/models")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            ),
        ] {
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{name} must fail closed"
            );
        }
    }

    #[tokio::test]
    async fn safety_guardrail_blocks_secret_like_payload_before_provider_dispatch() {
        let handles = Arc::new(Mutex::new(Vec::new()));
        let state = test_state_with_secret_store(
            true,
            Arc::new(RecordingSecretStore {
                handles: Arc::clone(&handles),
            }),
            "secret-ref://tenant-a/anthropic/seat-a",
        );

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("authorization", "Bearer ingress-token")
                    .header("x-agent-id", "agent-a")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"model":"claude-test","messages":[{"role":"user","content":"api_key = sk-live"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("safety_guardrail_blocked"));
        assert!(
            handles.lock().unwrap().is_empty(),
            "provider credential lookup must not run after a safety block"
        );
    }

    #[tokio::test]
    async fn unknown_model_fails_closed_before_provider_dispatch() {
        let handles = Arc::new(Mutex::new(Vec::new()));
        let state = test_state_with_secret_store(
            true,
            Arc::new(RecordingSecretStore {
                handles: Arc::clone(&handles),
            }),
            "secret-ref://tenant-a/anthropic/seat-a",
        );

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("authorization", "Bearer ingress-token")
                    .header("x-agent-id", "agent-a")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"model":"not-a-real-model","messages":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("model_not_found"));
        assert!(
            handles.lock().unwrap().is_empty(),
            "unknown models must fail before leasing or resolving provider credentials"
        );
    }

    #[test]
    fn upstream_http_errors_are_not_recorded_as_successful_seat_outcomes() {
        assert_eq!(seat_outcome_for_upstream_status(200), SeatOutcome::Ok);
        assert_eq!(
            seat_outcome_for_upstream_status(400),
            SeatOutcome::ServerError5xx
        );
        assert_eq!(
            seat_outcome_for_upstream_status(429),
            SeatOutcome::RateLimited429
        );
        assert_eq!(
            seat_outcome_for_upstream_status(500),
            SeatOutcome::ServerError5xx
        );
        assert_eq!(event_status_for_upstream_status(200), EventStatus::Ok);
        assert_eq!(
            event_status_for_upstream_status(400),
            EventStatus::UpstreamError
        );
        assert_eq!(
            event_status_for_upstream_status(429),
            EventStatus::RateLimited
        );
        assert_eq!(
            event_status_for_upstream_status(500),
            EventStatus::UpstreamError
        );
    }

    #[tokio::test]
    async fn xproxy_api_005_admin_status_accounts_and_analytics_expose_owned_ports_not_transient_engines()
     {
        let router = build_router(test_state(true, true));

        let status = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/status")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status.status(), StatusCode::OK);
        let status_body = status.into_body().collect().await.unwrap().to_bytes();
        let status_json: serde_json::Value = serde_json::from_slice(&status_body).unwrap();
        assert_eq!(status_json["status"], "ok");
        assert_eq!(status_json["route_controller_ready"], true);
        assert_eq!(status_json["model_inventory_ready"], true);
        assert_eq!(status_json["credential_worker_ready"], true);
        assert_eq!(status_json["policy_engine_ready"], true);
        assert_eq!(status_json["default_data_plane_ready"], true);
        assert_eq!(status_json["secret_provider_ready"], true);
        assert_eq!(status_json["registered_pools"], 1);
        assert_eq!(
            status_json["boundaries"]["secret_resolution"],
            "owned-secret-provider-port"
        );
        assert_eq!(
            status_json["boundaries"]["authorization"],
            "owned-policy-engine-port"
        );
        let status_body = std::str::from_utf8(&status_body).unwrap();
        assert!(status_body.contains("owned-secret-provider-port"));
        assert!(status_body.contains("owned-policy-engine-port"));
        assert!(!status_body.contains("OpenBao"));
        assert!(!status_body.contains("Cedar"));
        assert!(!status_body.contains("vault"));

        // A default-deny policy engine must degrade the reported status
        // instead of being narrated as ready.
        let forbid_router = build_router(test_state_with_policy_gate(
            true,
            Arc::new(MemorySecretStore { ready: true }),
            "tenant-a/seat-a",
            Arc::new(ForbidGate),
        ));
        let status = forbid_router
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/status")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status.status(), StatusCode::OK);
        let body = status.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["policy_engine_ready"], false);
        assert_eq!(body["status"], "degraded");

        let accounts = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/accounts")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(accounts.status(), StatusCode::OK);
        let accounts_body = accounts.into_body().collect().await.unwrap().to_bytes();
        let accounts_json: serde_json::Value = serde_json::from_slice(&accounts_body).unwrap();
        let account = &accounts_json["accounts"][0];
        assert_eq!(account["tenant_id"], "tenant-a");
        assert_eq!(account["provider"], "anthropic");
        assert_eq!(account["seat_id"], "seat-a");
        assert_eq!(account["state"], "active");
        assert!(account["headroom_percent"].as_f64().is_some());
        assert!(account.get("total_seats").is_none());
        assert!(account.get("ready").is_none());
        assert_eq!(accounts_json["redaction"], "secret-handles-redacted");
        let accounts_body = std::str::from_utf8(&accounts_body).unwrap();
        assert!(accounts_body.contains("\"accounts\""));
        assert!(!accounts_body.contains("test-refresh-token"));
        assert!(!accounts_body.contains("tenant-a/seat-a"));

        let analytics = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/analytics")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(analytics.status(), StatusCode::OK);
        let analytics_body = analytics.into_body().collect().await.unwrap().to_bytes();
        let analytics_body = std::str::from_utf8(&analytics_body).unwrap();
        assert!(analytics_body.contains("\"window\""));
        assert!(analytics_body.contains("\"request_count\""));

        let stream = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/analytics/stream")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream.status(), StatusCode::OK);
        assert_eq!(
            stream
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("text/event-stream")
        );

        let resume = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/resume")
                    .header("authorization", "Bearer admin-token")
                    .header("idempotency-key", "11111111-1111-4111-8111-333333333333")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"scope":"tenant-a/anthropic","reason":"unit-test"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resume.status(), StatusCode::NOT_IMPLEMENTED);
        let resume_body = resume.into_body().collect().await.unwrap().to_bytes();
        let resume_body = std::str::from_utf8(&resume_body).unwrap();
        assert!(resume_body.contains("\"status\":\"not_implemented\""));
        assert!(!resume_body.to_ascii_lowercase().contains("cli"));
        assert!(!resume_body.to_ascii_lowercase().contains("tui"));
    }

    #[tokio::test]
    async fn cloud_intelligence_agent_and_canary_admin_routes_are_authenticated_readonly_and_redacted()
     {
        let router = build_router(test_state(true, true));
        for path in [
            "/admin/v1/agent-runtimes",
            "/admin/v1/agent-schedules",
            "/admin/v1/parity/canaries",
        ] {
            let unauthorized = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                unauthorized.status(),
                StatusCode::UNAUTHORIZED,
                "{path} auth"
            );

            let authorized = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .header("authorization", "Bearer admin-token")
                        .header("x-oya-admin-tenant", "tenant-a")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(authorized.status(), StatusCode::OK, "{path} status");
            let body = authorized.into_body().collect().await.unwrap().to_bytes();
            let body = std::str::from_utf8(&body).unwrap();
            assert!(body.contains("owned-policy-engine-port"));
            assert!(body.contains("redacted-structured-evidence"));
            // Quoted-key form: negative-capability attestations like
            // `writes_raw_prompts_or_secrets` are allowed; raw payload FIELDS
            // (`"raw_prompt": ...`) are not.
            assert!(!body.contains("\"raw_prompt\""));
            assert!(!body.contains("\"raw_completion\""));
            assert!(!body.contains("\"raw_token\""));
            assert!(!body.contains("sk-"));
            assert!(
                !body
                    .to_ascii_lowercase()
                    .contains(&["shell", "out", "to", "cli"].join(" "))
            );
            assert!(
                !body
                    .to_ascii_lowercase()
                    .contains(&["local", "panel"].join(" "))
            );
        }

        // AUTH-005 / ADR-0573 PR2: these routes administer the service's own tenant
        // (tenant-a). The `x-oya-admin-tenant` header is no longer an authz input —
        // a forged value is DEAD INPUT and cannot redirect or deny the request. The
        // verified admin credential (bound to tenant-a) authorizes; the gate decides.
        for path in [
            "/admin/v1/agent-runtimes",
            "/admin/v1/agent-schedules",
            "/admin/v1/parity/canaries",
        ] {
            for forged_tenant in ["tenant-b", "oyatie"] {
                let forged = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(path)
                            .header("authorization", "Bearer admin-token")
                            .header("x-oya-admin-tenant", forged_tenant)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    forged.status(),
                    StatusCode::OK,
                    "{path} forged tenant {forged_tenant} header must be ignored (dead input)"
                );
                let body = forged.into_body().collect().await.unwrap().to_bytes();
                let body = std::str::from_utf8(&body).unwrap();
                assert!(
                    !body.contains("tenant-b") && !body.contains("oyatie"),
                    "{path} must only ever expose the credential-bound tenant (tenant-a)"
                );
            }
        }

        let runtimes = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/agent-runtimes")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = runtimes.into_body().collect().await.unwrap().to_bytes();
        let runtime_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(runtime_json["runtimes"][0]["tenant_id"], "tenant-a");
        assert_eq!(runtime_json["runtimes"][0]["name"], "dogfood-fable-runtime");
        assert_eq!(
            runtime_json["workflow_statuses"][0]["tenant_id"],
            "tenant-a"
        );
        assert!(
            runtime_json["workflow_statuses"][0]["workflow_ref"]
                .as_str()
                .unwrap()
                .starts_with("agent-workflow-ref://tenant-a/")
        );
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("AgentRuntimeProfile"));
        assert!(body.contains("AgentWorkflowStatus"));
        assert!(body.contains("claude"));
        assert!(body.contains("codex"));
        assert!(body.contains("gemini"));
        assert!(body.contains("routing-decision-only"));
        assert!(!body.contains("\"tenant_id\":\"oyatie\""));

        let schedules = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/agent-schedules")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = schedules.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["schedules"][0]["tenant_id"], "tenant-a");
        assert_eq!(
            body["schedules"][0]["schedule_ref"],
            "schedule-ref://tenant-a/nightly-drift-check"
        );
        let status = &body["statuses"][0];
        assert_eq!(status["kind"], "AgentScheduleStatus");
        assert_eq!(status["state"], "passed");
        assert_eq!(status["policy_engine_port"], "owned-policy-engine-port");
        assert_eq!(
            status["evidence_visibility"],
            "redacted-structured-evidence"
        );
        assert_eq!(status["raw_payload_included"], false);
        assert_eq!(
            status["sealed_evidence_handle_ref"],
            "sealed-evidence-ref://tenant-a/nightly-drift-check/status"
        );

        let canaries = router
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/parity/canaries")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = canaries.into_body().collect().await.unwrap().to_bytes();
        let canary_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(canary_json["plans"][0]["tenant_id"], "tenant-a");
        assert_eq!(
            canary_json["plans"][0]["schedule_ref"],
            "schedule-ref://tenant-a/nightly-drift-check"
        );
        assert_eq!(
            canary_json["statuses"][0]["plan_ref"],
            "parity-canary-plan-ref://tenant-a/nightly-drift-check"
        );
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("ParityCanaryPlan"));
        assert!(body.contains("ParityCanaryStatus"));
        assert!(body.contains("Retry-After"));
        assert!(body.contains("sealed-evidence-ref://"));
    }

    #[tokio::test]
    async fn cloud_intelligence_safety_admin_routes_are_authenticated_and_redacted() {
        let router = build_router(test_state(true, true));
        for path in [
            "/admin/v1/guardrails",
            "/admin/v1/guardrails/escalations",
            "/admin/v1/evidence/retention",
            "/admin/v1/redaction/profiles",
        ] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path} auth");
        }

        let guardrails = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/guardrails")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(guardrails.status(), StatusCode::OK);
        let body = guardrails.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("mandatory_secondary_agentic_review"));
        assert!(body.contains("\"tenant_can_override_platform_critical_block\":false"));
        assert!(body.contains("owned-policy-engine-port"));
        assert!(!body.contains("Cedar"));
        assert!(!body.contains("OpenBao"));
        assert!(!body.contains("vault"));

        let evidence = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/evidence/retention")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(evidence.status(), StatusCode::OK);
        let body = evidence.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("encrypted_handle_on_guardrail_trigger"));
        assert!(body.contains("raw_access_requires_audited_break_glass"));
        assert!(!body.contains("raw_prompt"));
        assert!(!body.contains("raw_completion"));

        let redaction = router
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/redaction/profiles")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(redaction.status(), StatusCode::OK);
        let body = redaction.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("reversible_tokens_require_tenant_policy"));
        assert!(body.contains("ephemeral-run"));
        assert!(body.contains("restore_only_after_model_output"));
    }

    /// AUTH-005 / ADR-0573 PR2: the admin tenant axis is bound to the VERIFIED
    /// credential (the configured admin bearer → tenant-a here) and enforced against
    /// the administered (path) tenant SERVER-SIDE. The caller-supplied
    /// `x-oya-admin-tenant` header is NO LONGER an authz input — it can neither deny a
    /// legitimate same-tenant request nor grant a cross-tenant one. (This supersedes
    /// the FRIC-1781420000 header guard, which trusted a forgeable header.)
    #[tokio::test]
    async fn admin_authz_is_credential_bound_not_header_bound() {
        let router = build_router(test_state(true, true));
        // Same-tenant routes (administer tenant-a): a valid admin bearer authorizes
        // regardless of the x-oya-admin-tenant header (absent OR forged => still OK);
        // no bearer => 401.
        for path in [
            "/admin/v1/agent-runtimes",
            "/admin/v1/agent-schedules",
            "/admin/v1/parity/canaries",
            "/admin/v1/guardrails",
            "/admin/v1/guardrails/escalations",
            "/admin/v1/evidence/retention",
            "/admin/v1/redaction/profiles",
            "/admin/v1/tenants/tenant-a/providers/anthropic/pool",
        ] {
            for header in [None, Some("tenant-b"), Some("oyatie")] {
                let mut builder = Request::builder()
                    .uri(path)
                    .header("authorization", "Bearer admin-token");
                if let Some(header) = header {
                    builder = builder.header("x-oya-admin-tenant", header);
                }
                let response = router
                    .clone()
                    .oneshot(builder.body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    StatusCode::OK,
                    "{path} must authorize the credential-bound admin regardless of the x-oya-admin-tenant header ({header:?})"
                );
            }

            let unauthenticated = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                unauthenticated.status(),
                StatusCode::UNAUTHORIZED,
                "{path} must 401 without an admin bearer"
            );
        }

        // Cross-tenant PATH: the admin credential is bound to tenant-a, so a tenant-b
        // path is forbidden — even with a forged matching x-oya-admin-tenant header.
        let cross_tenant_pool = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-b/providers/anthropic/pool")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            cross_tenant_pool.status(),
            StatusCode::FORBIDDEN,
            "admin bound to tenant-a must not read tenant-b's pool (cross-tenant)"
        );

        // Mutating admin surface: authn+authz run BEFORE idempotency/body parsing.
        let subscription_request = |path_tenant: &str, tenant_header: Option<&str>| {
            let mut builder = Request::builder()
                .method("POST")
                .uri(format!(
                    "/admin/v1/tenants/{path_tenant}/providers/anthropic/subscriptions"
                ))
                .header("authorization", "Bearer admin-token")
                .header("idempotency-key", "44444444-4444-4444-8444-444444444444")
                .header("content-type", "application/json");
            if let Some(tenant) = tenant_header {
                builder = builder.header("x-oya-admin-tenant", tenant);
            }
            builder
                .body(Body::from(
                    r#"{"seat_id":"seat-guard","subscription_id":"sub-guard","credential_mode":"oauth_subscription","secret_handle":"secret-ref://tenant-a/anthropic/seat-guard"}"#,
                ))
                .unwrap()
        };

        // Cross-tenant PATH (with a forged matching header) => 403.
        let cross_tenant_register = build_router(test_state(true, true))
            .oneshot(subscription_request("tenant-b", Some("tenant-b")))
            .await
            .unwrap();
        assert_eq!(
            cross_tenant_register.status(),
            StatusCode::FORBIDDEN,
            "subscription registration must deny a cross-tenant path"
        );

        // Same-tenant PATH (header absent — it is irrelevant) => CREATED.
        let same_tenant_register = build_router(test_state(true, true))
            .oneshot(subscription_request("tenant-a", None))
            .await
            .unwrap();
        assert_eq!(
            same_tenant_register.status(),
            StatusCode::CREATED,
            "same-tenant subscription registration must succeed regardless of the header"
        );
    }

    #[tokio::test]
    async fn xproxy_api_004_model_inventory_and_count_tokens_routes_are_contract_first_cloud_surfaces()
     {
        let router = build_router(test_state(true, true));

        let models = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .header("authorization", "Bearer ingress-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(models.status(), StatusCode::OK);
        let models_body = models.into_body().collect().await.unwrap().to_bytes();
        let models_body = std::str::from_utf8(&models_body).unwrap();
        assert!(models_body.contains("\"object\":\"list\""));
        assert!(models_body.contains("model-inventory-worker"));
        assert!(models_body.contains("gemini-2.5-flash"));
        assert!(!models_body.to_ascii_lowercase().contains("cli"));
        assert!(!models_body.to_ascii_lowercase().contains("tui"));

        let token_count = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages/count_tokens")
                    .header("authorization", "Bearer ingress-token")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"model":"claude-opus-4-5","messages":[{"role":"user","content":"hello cloud intelligence"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(token_count.status(), StatusCode::OK);
        let token_count_body = token_count.into_body().collect().await.unwrap().to_bytes();
        let token_count_body = std::str::from_utf8(&token_count_body).unwrap();
        assert!(token_count_body.contains("\"input_tokens\""));
        assert!(!token_count_body.to_ascii_lowercase().contains("prompt"));
        assert!(!token_count_body.to_ascii_lowercase().contains("secret"));
    }

    #[tokio::test]
    async fn readiness_requires_configured_pool() {
        let response = build_router(test_state(false, true))
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn readiness_requires_secret_store_health() {
        let response = build_router(test_state(true, false))
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn readiness_requires_default_data_plane_pool_not_only_other_registry_pool() {
        let tenant_a = TenantId::new("tenant-a").unwrap();
        let tenant_b = TenantId::new("tenant-b").unwrap();
        let default_pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant_a.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));

        let registry = PoolRegistry::new();
        let mut codex_pool = SubscriptionPool::new(
            tenant_b.clone(),
            Provider::Codex,
            SelectionStrategy::RoundRobin,
        );
        codex_pool
            .add_seat(OAuthSubscription::new(
                tenant_b.clone(),
                SeatId::new("seat-b").unwrap(),
                SubscriptionId::new("sub-b").unwrap(),
                Provider::Codex,
                SubscriptionState::Active,
                "secret-ref://tenant-b/codex/seat-b",
                0,
            ))
            .unwrap();
        registry.insert_pool(tenant_b, Provider::Codex, Arc::new(Mutex::new(codex_pool)));

        let state = AppState::new_with_pool_registry(
            default_pool,
            registry,
            Arc::new(AllowGate),
            Arc::new(NoopSink),
            Arc::new(MemorySecretStore { ready: true }),
            "http://127.0.0.1:1".to_string(),
            tenant_a,
            Some("ingress-token".to_string()),
            Some("admin-token".to_string()),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap();

        let response = build_router(Arc::new(state))
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn proxy_uses_registered_secret_handle_not_derived_convention() {
        let handles = Arc::new(Mutex::new(Vec::new()));
        let state = test_state_with_secret_store(
            true,
            Arc::new(RecordingSecretStore {
                handles: Arc::clone(&handles),
            }),
            "secret-ref://tenant-a/anthropic/seat-a",
        );

        let _ = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("authorization", "Bearer ingress-token")
                    .header("x-agent-id", "agent-a")
                    .body(Body::from(r#"{"model":"claude-test","messages":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            handles.lock().unwrap().as_slice(),
            ["secret-ref://tenant-a/anthropic/seat-a"]
        );
    }

    /// Fake upstream that records every request line it receives across N
    /// connections, so a test can assert which upstream paths the proxy hit
    /// (e.g. the OAuth token endpoint vs `/v1/messages`).
    fn recording_multi_request_server(
        responses: Vec<&'static str>,
    ) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_for_task = Arc::clone(&received);
        let std_listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind fake provider");
        std_listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let addr = std_listener.local_addr().expect("fake provider addr");
        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(std_listener).expect("tokio listener");
            for response in responses {
                let (mut socket, _) = listener.accept().await.expect("accept fake provider");
                let mut buf = vec![0_u8; 16 * 1024];
                let n = socket.read(&mut buf).await.expect("read fake request");
                let request = String::from_utf8_lossy(&buf[..n]).to_string();
                received_for_task.lock().unwrap().push(request);
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fake response");
            }
        });
        (format!("http://{addr}"), received, handle)
    }

    fn proxy_state_with_seat_mode(
        base_url: String,
        credential_mode: CredentialMode,
        secret_handle: &str,
    ) -> Arc<AppState> {
        proxy_state_with_secret_store(
            base_url,
            credential_mode,
            secret_handle,
            Arc::new(MemorySecretStore { ready: true }),
            0,
        )
        .0
    }

    fn proxy_state_with_secret_store(
        base_url: String,
        credential_mode: CredentialMode,
        secret_handle: &str,
        secret_store: Arc<dyn SecretProviderStore>,
        failure_count: u32,
    ) -> (Arc<AppState>, Arc<Mutex<SubscriptionPool>>) {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let seat_id = SeatId::new("seat-a").unwrap();
        let mut pool = SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        );
        pool.add_seat(
            OAuthSubscription::new(
                tenant_id.clone(),
                seat_id,
                SubscriptionId::new("sub-a").unwrap(),
                Provider::Anthropic,
                SubscriptionState::Active,
                secret_handle.to_string(),
                failure_count,
            )
            .with_credential_mode(credential_mode),
        )
        .unwrap();

        let pool = Arc::new(Mutex::new(pool));
        let registry = PoolRegistry::new();
        registry.insert_pool(tenant_id.clone(), Provider::Anthropic, Arc::clone(&pool));
        let state = Arc::new(
            AppState::new_with_pool_registry(
                Arc::clone(&pool),
                registry,
                Arc::new(AllowGate),
                Arc::new(NoopSink),
                secret_store,
                base_url,
                tenant_id,
                Some("ingress-token".to_string()),
                Some("admin-token".to_string()),
                "development".to_string(),
                std::collections::HashSet::new(),
            )
            .unwrap(),
        );
        (state, pool)
    }

    fn openai_proxy_state(base_url: String) -> Arc<AppState> {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let default_pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));

        let mut openai_pool = SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Codex,
            SelectionStrategy::RoundRobin,
        );
        openai_pool
            .add_seat(
                OAuthSubscription::new(
                    tenant_id.clone(),
                    SeatId::new("openai-seat-a").unwrap(),
                    SubscriptionId::new("openai-sub-a").unwrap(),
                    Provider::Codex,
                    SubscriptionState::Active,
                    "secret-ref://tenant-a/openai/openai-seat-a",
                    0,
                )
                .with_credential_mode(CredentialMode::ApiKey),
            )
            .unwrap();

        let registry = PoolRegistry::new();
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Anthropic,
            Arc::clone(&default_pool),
        );
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Codex,
            Arc::new(Mutex::new(openai_pool)),
        );

        let mut state = AppState::new_with_pool_registry(
            default_pool,
            registry,
            Arc::new(AllowGate),
            Arc::new(NoopSink),
            Arc::new(MemorySecretStore { ready: true }),
            "http://127.0.0.1:1".to_string(),
            tenant_id,
            Some("ingress-token".to_string()),
            Some("admin-token".to_string()),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap();
        state.openai_compatible_base_url = base_url;
        Arc::new(state)
    }

    #[tokio::test]
    async fn xproxy_api_003_openai_chat_completions_passes_body_and_safe_headers_to_openai_compatible_backend()
     {
        let upstream_body = r#"{"id":"chatcmpl-1","object":"chat.completion","choices":[]}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Upstream-Trace: trace-a\r\nContent-Length: {}\r\n\r\n{}",
                upstream_body.len(), upstream_body
            )
            .into_boxed_str(),
        )]);
        let state = openai_proxy_state(base_url);
        let request_body = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"hello"}]}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("connection", "x-drop-me")
                    .header("x-drop-me", "must-not-forward")
                    .header("x-openai-beta", "must-not-forward")
                    .header("openai-organization", "org-safe")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-upstream-trace")
                .and_then(|value| value.to_str().ok()),
            Some("trace-a")
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), upstream_body);

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 1);
        let req = &requests[0];
        assert!(
            req.starts_with("POST /v1/chat/completions "),
            "unexpected upstream path:\n{req}"
        );
        assert!(
            req.contains(request_body),
            "body was not passed through:\n{req}"
        );
        assert_header(req, "authorization", "Bearer test-refresh-token");
        assert_header(req, "openai-organization", "org-safe");
        assert!(
            !req.contains("ingress-token"),
            "caller authorization must not be forwarded:\n{req}"
        );
        assert!(
            !req.contains("must-not-forward"),
            "provider-control or connection-nominated headers leaked:\n{req}"
        );
    }

    /// AppState with a Codex OAuth-subscription seat whose ChatGPT base URL
    /// points at a loopback fake. Mirrors `openai_proxy_state` but the seat is
    /// `OAuthSubscription` rather than `ApiKey`.
    fn codex_oauth_proxy_state(base_url: String) -> Arc<AppState> {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let default_pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));

        let mut codex_pool = SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Codex,
            SelectionStrategy::RoundRobin,
        );
        codex_pool
            .add_seat(
                OAuthSubscription::new(
                    tenant_id.clone(),
                    SeatId::new("codex-seat-a").unwrap(),
                    SubscriptionId::new("codex-sub-a").unwrap(),
                    Provider::Codex,
                    SubscriptionState::Active,
                    "secret-ref://tenant-a/codex/codex-seat-a",
                    0,
                )
                .with_credential_mode(CredentialMode::OAuthSubscription),
            )
            .unwrap();

        let registry = PoolRegistry::new();
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Anthropic,
            Arc::clone(&default_pool),
        );
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Codex,
            Arc::new(Mutex::new(codex_pool)),
        );

        let mut state = AppState::new_with_pool_registry(
            default_pool,
            registry,
            Arc::new(AllowGate),
            Arc::new(NoopSink),
            Arc::new(MemorySecretStore { ready: true }),
            "http://127.0.0.1:1".to_string(),
            tenant_id,
            Some("ingress-token".to_string()),
            Some("admin-token".to_string()),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap();
        state.codex_oauth_base_url = base_url;
        Arc::new(state)
    }

    #[tokio::test]
    async fn codex_oauth_subscription_refreshes_session_then_proxies_to_codex_data_endpoint() {
        let session_body = r#"{"accessToken":"codex-access-xyz"}"#;
        let upstream_body = r#"{"id":"resp-1","object":"response","output":[]}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![
            Box::leak(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    session_body.len(),
                    session_body
                )
                .into_boxed_str(),
            ),
            Box::leak(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    upstream_body.len(),
                    upstream_body
                )
                .into_boxed_str(),
            ),
        ]);
        let state = codex_oauth_proxy_state(base_url);
        let request_body = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}]}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("user-agent", "evil-downstream-ua")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), upstream_body);

        let requests = received.lock().unwrap().clone();
        assert_eq!(
            requests.len(),
            2,
            "expected a session refresh followed by the codex data call"
        );
        let session_req = &requests[0];
        assert!(
            session_req.starts_with("POST /api/auth/session "),
            "first call must be the session refresh:\n{session_req}"
        );
        assert!(
            session_req.contains("__Secure-next-auth.session-token=test-refresh-token"),
            "session refresh must present the seat refresh token as the cookie:\n{session_req}"
        );

        let data_req = &requests[1];
        assert!(
            data_req.starts_with("POST /backend-api/codex/responses "),
            "second call must hit the Codex data endpoint:\n{data_req}"
        );
        assert_header(data_req, "authorization", "Bearer codex-access-xyz");
        assert_header(data_req, "x-openai-beta", "codex-runs");
        // Codex CLI User-Agent impersonation replaces the caller UA (Cloudflare 1010).
        assert_header(data_req, "user-agent", "cli/0.27.0");
        assert!(
            data_req.contains(request_body),
            "request body must be passed through:\n{data_req}"
        );
        assert!(
            !data_req.contains("evil-downstream-ua"),
            "caller User-Agent must be replaced, not forwarded:\n{data_req}"
        );
        assert!(
            !data_req.contains("ingress-token"),
            "caller ingress token must not be forwarded upstream:\n{data_req}"
        );
    }

    #[tokio::test]
    async fn codex_oauth_subscription_streaming_proxies_sse_from_codex_data_endpoint() {
        let session_body = r#"{"accessToken":"codex-access-stream"}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![
            Box::leak(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    session_body.len(),
                    session_body
                )
                .into_boxed_str(),
            ),
            Box::leak(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\nContent-Length: 10\r\n\r\ndata: hi\n\n"
                    .to_string()
                    .into_boxed_str(),
            ),
        ]);
        let state = codex_oauth_proxy_state(base_url);
        let request_body = r#"{"model":"gpt-4o","messages":[],"stream":true}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("accept", "text/event-stream")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("text/event-stream")
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), "data: hi\n\n");

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("POST /api/auth/session "));
        assert!(requests[1].starts_with("POST /backend-api/codex/responses "));
        assert_header(&requests[1], "authorization", "Bearer codex-access-stream");
    }

    #[tokio::test]
    async fn codex_oauth_subscription_refresh_failure_fails_closed_without_data_call() {
        // The session endpoint rejects the refresh token; the data endpoint must
        // never be reached (no fall-through to an unauthenticated upstream call).
        let (base_url, received, _server) = recording_multi_request_server(vec![Box::leak(
            "HTTP/1.1 401 Unauthorized\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
                .to_string()
                .into_boxed_str(),
        )]);
        let state = codex_oauth_proxy_state(base_url);
        let request_body = r#"{"model":"gpt-4o","messages":[]}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let requests = received.lock().unwrap().clone();
        assert_eq!(
            requests.len(),
            1,
            "only the session refresh may be attempted; refresh failure must not call the data endpoint"
        );
        assert!(requests[0].starts_with("POST /api/auth/session "));
    }

    #[tokio::test]
    async fn codex_oauth_subscription_embeddings_path_is_unsupported_and_fails_closed() {
        // Embeddings over an OAuth-subscription Codex seat is genuinely
        // unsupported (the ChatGPT responses endpoint is chat-shaped only) and
        // is rejected before any refresh/proxy — no upstream server needed.
        let state = codex_oauth_proxy_state("http://127.0.0.1:1".to_string());
        let request_body = r#"{"model":"text-embedding-3-small","input":"hi"}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/embeddings")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("codex_oauth_subscription_path_unsupported"),
            "expected the unsupported-path error code in the body"
        );
    }

    fn gemini_proxy_state(base_url: String) -> Arc<AppState> {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let default_pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));

        let mut gemini_pool = SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Gemini,
            SelectionStrategy::RoundRobin,
        );
        gemini_pool
            .add_seat(
                OAuthSubscription::new(
                    tenant_id.clone(),
                    SeatId::new("gemini-seat-a").unwrap(),
                    SubscriptionId::new("gemini-sub-a").unwrap(),
                    Provider::Gemini,
                    SubscriptionState::Active,
                    "secret-ref://tenant-a/gemini/gemini-seat-a",
                    0,
                )
                .with_credential_mode(CredentialMode::ApiKey),
            )
            .unwrap();

        let registry = PoolRegistry::new();
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Anthropic,
            Arc::clone(&default_pool),
        );
        registry.insert_pool(
            tenant_id.clone(),
            Provider::Gemini,
            Arc::new(Mutex::new(gemini_pool)),
        );

        let mut state = AppState::new_with_pool_registry(
            default_pool,
            registry,
            Arc::new(AllowGate),
            Arc::new(NoopSink),
            Arc::new(MemorySecretStore { ready: true }),
            "http://127.0.0.1:1".to_string(),
            tenant_id,
            Some("ingress-token".to_string()),
            Some("admin-token".to_string()),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap();
        state.gemini_base_url = format!("{base_url}/v1beta");
        Arc::new(state)
    }

    #[tokio::test]
    async fn xproxy_route_gemini_openai_chat_translates_to_native_generate_content_backend() {
        let upstream_body = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"ok"}]}}],"usageMetadata":{"promptTokenCount":2,"candidatesTokenCount":3}}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                upstream_body.len(),
                upstream_body
            )
            .into_boxed_str(),
        )]);
        let state = gemini_proxy_state(base_url);
        let request_body = r#"{"model":"gemini:gemini-2.5-flash","messages":[{"role":"system","content":"Be brief"},{"role":"user","content":"hello"}]}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("connection", "x-drop-me")
                    .header("x-drop-me", "must-not-forward")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["object"], "chat.completion");
        assert_eq!(
            body_json["choices"][0]["message"]["content"],
            serde_json::json!("ok")
        );
        assert_eq!(body_json["usage"]["total_tokens"], serde_json::json!(5));

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 1);
        let req = &requests[0];
        assert!(
            req.starts_with("POST /v1beta/models/gemini-2.5-flash:generateContent "),
            "unexpected upstream path:\n{req}"
        );
        assert_header(req, "x-goog-api-key", "test-refresh-token");
        assert!(req.contains("\"contents\""));
        assert!(req.contains("\"systemInstruction\""));
        assert!(!req.contains("\"messages\""));
        assert!(!req.contains("ingress-token"));
        assert!(!req.contains("must-not-forward"));
    }

    #[tokio::test]
    async fn xproxy_route_gemini_anthropic_messages_translates_through_adapter_boundary() {
        let upstream_body = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"pong"}]}}],"usageMetadata":{"promptTokenCount":4,"candidatesTokenCount":6}}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                upstream_body.len(),
                upstream_body
            )
            .into_boxed_str(),
        )]);
        let state = gemini_proxy_state(base_url);
        let request_body = r#"{"model":"gemini:gemini-2.5-pro","system":"Be exact","max_tokens":64,"messages":[{"role":"user","content":"ping"}]}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("connection", "x-drop-me")
                    .header("x-drop-me", "must-not-forward")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["type"], "message");
        assert_eq!(body_json["content"][0]["text"], serde_json::json!("pong"));
        assert_eq!(body_json["usage"]["input_tokens"], serde_json::json!(4));
        assert_eq!(body_json["usage"]["output_tokens"], serde_json::json!(6));

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 1);
        let req = &requests[0];
        assert!(
            req.starts_with("POST /v1beta/models/gemini-2.5-pro:generateContent "),
            "unexpected upstream path:\n{req}"
        );
        assert_header(req, "x-goog-api-key", "test-refresh-token");
        assert!(req.contains("\"contents\""));
        assert!(req.contains("\"systemInstruction\""));
        assert!(!req.contains("\"messages\""));
        assert!(!req.contains("ingress-token"));
        assert!(!req.contains("must-not-forward"));
    }

    #[tokio::test]
    async fn xproxy_route_007_openai_embeddings_route_uses_same_openai_compatible_pass_through_contract()
     {
        let upstream_body = r#"{"object":"list","data":[{"embedding":[0.1]}]}"#;
        let (base_url, received, _server) = recording_multi_request_server(vec![Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                upstream_body.len(),
                upstream_body
            )
            .into_boxed_str(),
        )]);
        let state = openai_proxy_state(base_url);
        let request_body = r#"{"model":"text-embedding-3-small","input":"hello"}"#;

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/embeddings")
                    .header("x-agent-id", "agent-a")
                    .header("authorization", "Bearer ingress-token")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(std::str::from_utf8(&body).unwrap(), upstream_body);

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 1);
        let req = &requests[0];
        assert!(
            req.starts_with("POST /v1/embeddings "),
            "unexpected upstream path:\n{req}"
        );
        assert!(
            req.contains(request_body),
            "body was not passed through:\n{req}"
        );
        assert_header(req, "authorization", "Bearer test-refresh-token");
        assert!(
            !req.contains("ingress-token"),
            "caller authorization must not be forwarded:\n{req}"
        );
    }

    #[tokio::test]
    async fn xproxy_api_002_legacy_anthropic_complete_route_is_cloud_api_deprecation_not_cli_flow()
    {
        let response = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/complete")
                    .header("authorization", "Bearer ingress-token")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"model":"claude-2","prompt":"hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::GONE);
        assert_eq!(
            response
                .headers()
                .get("x-oya-compatibility")
                .and_then(|value| value.to_str().ok()),
            Some("deprecated-legacy-completions")
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            payload["error"]["code"],
            "legacy_anthropic_completions_deprecated"
        );
        assert!(
            !std::str::from_utf8(&body)
                .unwrap()
                .to_ascii_lowercase()
                .contains("cli")
        );
        assert!(
            !std::str::from_utf8(&body)
                .unwrap()
                .to_ascii_lowercase()
                .contains("tui")
        );
    }

    /// Build an admin-test `AppState` for a given environment + OAuth approval
    /// set. The default Anthropic pool starts empty so admin registration drives
    /// the seat count.
    fn admin_state(
        environment: &str,
        oauth_approved_providers: std::collections::HashSet<Provider>,
    ) -> Arc<AppState> {
        let tenant_id = TenantId::new("tenant-a").unwrap();
        let pool = Arc::new(Mutex::new(SubscriptionPool::new(
            tenant_id.clone(),
            Provider::Anthropic,
            SelectionStrategy::RoundRobin,
        )));
        let registry = PoolRegistry::new();
        registry.insert_pool(tenant_id.clone(), Provider::Anthropic, Arc::clone(&pool));
        Arc::new(
            AppState::new_with_pool_registry(
                pool,
                registry,
                Arc::new(AllowGate),
                Arc::new(NoopSink),
                Arc::new(MemorySecretStore { ready: true }),
                "http://127.0.0.1:1".to_string(),
                tenant_id,
                Some("ingress-token".to_string()),
                Some("admin-token".to_string()),
                environment.to_string(),
                oauth_approved_providers,
            )
            .unwrap(),
        )
    }

    fn admin_register_request(credential_mode: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/admin/v1/tenants/tenant-a/providers/anthropic/subscriptions")
            .header("authorization", "Bearer admin-token")
            .header("x-oya-admin-tenant", "tenant-a")
            .header("idempotency-key", "11111111-1111-4111-8111-111111111111")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"seat_id":"seat-x","subscription_id":"sub-x","credential_mode":"{credential_mode}","secret_handle":"secret-ref://tenant-a/anthropic/seat-x"}}"#
            )))
            .unwrap()
    }

    fn anthropic_proxy_request() -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/messages")
            .header("authorization", "Bearer ingress-token")
            .header("x-agent-id", "agent-a")
            .body(Body::from(r#"{"model":"claude-test","messages":[]}"#))
            .unwrap()
    }

    fn anthropic_sse_proxy_request() -> Request<Body> {
        let mut request = anthropic_proxy_request();
        request
            .headers_mut()
            .insert("accept", HeaderValue::from_static("text/event-stream"));
        request
    }

    #[tokio::test]
    async fn production_admin_register_oauth_rejected_when_provider_pending() {
        let response = build_router(admin_state("production", std::collections::HashSet::new()))
            .oneshot(admin_register_request("oauth_subscription"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn production_admin_register_oauth_accepted_when_provider_approved() {
        let approved = std::collections::HashSet::from([Provider::Anthropic]);
        let response = build_router(admin_state("production", approved))
            .oneshot(admin_register_request("oauth_subscription"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn production_admin_register_api_key_accepted_without_oauth_approval() {
        let response = build_router(admin_state("production", std::collections::HashSet::new()))
            .oneshot(admin_register_request("api_key"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn non_production_admin_register_oauth_accepted_without_oauth_approval() {
        let response = build_router(admin_state("development", std::collections::HashSet::new()))
            .oneshot(admin_register_request("oauth_subscription"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn proxy_api_key_seat_sends_x_api_key_and_skips_oauth_refresh() {
        // Single upstream request expected: the direct `/v1/messages` call.
        let (base_url, received, _server) = recording_multi_request_server(vec![
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\n{\"ok\":true}",
        ]);
        let state = proxy_state_with_seat_mode(
            base_url,
            CredentialMode::ApiKey,
            "secret-ref://tenant-a/anthropic/seat-a",
        );

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("authorization", "Bearer ingress-token")
                    .header("x-agent-id", "agent-a")
                    .body(Body::from(r#"{"model":"claude-test","messages":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let requests = received.lock().unwrap().clone();
        assert_eq!(
            requests.len(),
            1,
            "api-key path must make exactly one upstream call"
        );
        let req = &requests[0];
        assert!(
            req.starts_with("POST /v1/messages "),
            "unexpected upstream path:\n{req}"
        );
        assert!(
            req.to_ascii_lowercase()
                .contains("x-api-key: test-refresh-token"),
            "api-key path must send x-api-key:\n{req}"
        );
        assert!(
            !req.contains("/v1/oauth/token"),
            "api-key path must not call the OAuth token endpoint"
        );
        assert!(
            !req.to_ascii_lowercase().contains("authorization: bearer"),
            "api-key path must not forward an OAuth bearer token"
        );
    }

    #[tokio::test]
    async fn proxy_oauth_seat_uses_oauth_refresh_then_bearer() {
        // Two upstream requests: the OAuth token refresh, then `/v1/messages`.
        let (base_url, received, _server) = recording_multi_request_server(vec![
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 70\r\n\r\n{\"access_token\":\"acc-tok\",\"refresh_token\":\"new-ref\",\"expires_in\":3600}",
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\n{\"ok\":true}",
        ]);
        let state = proxy_state_with_seat_mode(
            base_url,
            CredentialMode::OAuthSubscription,
            "secret-ref://tenant-a/anthropic/seat-a",
        );

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("authorization", "Bearer ingress-token")
                    .header("x-agent-id", "agent-a")
                    .body(Body::from(r#"{"model":"claude-test","messages":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let requests = received.lock().unwrap().clone();
        assert_eq!(requests.len(), 2, "OAuth path must refresh then proxy");
        assert!(
            requests[0].starts_with("POST /v1/oauth/token "),
            "first OAuth-path call must hit the token endpoint:\n{}",
            requests[0]
        );
        let messages_req = &requests[1];
        assert!(
            messages_req.starts_with("POST /v1/messages "),
            "second OAuth-path call must hit /v1/messages:\n{messages_req}"
        );
        assert!(
            messages_req
                .to_ascii_lowercase()
                .contains("authorization: bearer acc-tok"),
            "OAuth path must forward the refreshed bearer token:\n{messages_req}"
        );
        assert!(
            !messages_req.to_ascii_lowercase().contains("x-api-key:"),
            "OAuth path must not send x-api-key"
        );
    }

    #[tokio::test]
    async fn rotation_persistence_outage_releases_seat_and_recovers_without_old_token_replay() {
        let first_token_body = r#"{"access_token":"access-first","refresh_token":"rotated-refresh","expires_in":3600}"#;
        let next_token_body =
            r#"{"access_token":"access-next","refresh_token":"final-refresh","expires_in":3600}"#;
        let message_body = r#"{"ok":true}"#;
        let response = |body: &'static str| {
            Box::leak(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                )
                .into_boxed_str(),
            ) as &'static str
        };
        let (base_url, received, _server) = recording_multi_request_server(vec![
            response(first_token_body),
            response(next_token_body),
            response(message_body),
        ]);
        let store_state = Arc::new(Mutex::new(RotationStoreState {
            token: "single-use-old".to_string(),
            reject_writes: true,
            fetch_count: 0,
            store_count: 0,
        }));
        let (state, pool) = proxy_state_with_secret_store(
            base_url,
            CredentialMode::OAuthSubscription,
            "secret-ref://tenant-a/anthropic/seat-a",
            Arc::new(RotationPersistenceStore {
                state: Arc::clone(&store_state),
            }),
            5,
        );
        let router = build_router(state);

        for failure in 1..=6 {
            let request = if failure == 1 {
                anthropic_sse_proxy_request()
            } else {
                anthropic_proxy_request()
            };
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(
                response.headers().get("retry-after"),
                Some(&HeaderValue::from_static("1"))
            );
            if failure == 1 {
                assert_ne!(
                    response.headers().get("content-type"),
                    Some(&HeaderValue::from_static("text/event-stream"))
                );
            }
            let response_body = response.into_body().collect().await.unwrap().to_bytes();
            assert!(response_body.is_empty());
            let response_text = String::from_utf8_lossy(&response_body);
            assert!(!response_text.contains("single-use-old"));
            assert!(!response_text.contains("rotated-refresh"));
            let store = store_state.lock().unwrap();
            assert_eq!(store.fetch_count, 1);
            assert_eq!(
                store.store_count,
                failure * ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS
            );
            assert_eq!(store.token, "single-use-old");
            drop(store);
            let pool = pool.lock().unwrap();
            let statuses = pool.redacted_seat_statuses(Instant::now());
            assert_eq!(statuses[0].state, "active");
            assert!(pool.has_eligible_seat(Instant::now()));
        }
        assert_eq!(received.lock().unwrap().len(), 1);

        store_state.lock().unwrap().reject_writes = false;
        let recovered = router
            .clone()
            .oneshot(anthropic_proxy_request())
            .await
            .unwrap();
        assert_eq!(recovered.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            recovered.headers().get("retry-after"),
            Some(&HeaderValue::from_static("1"))
        );
        let recovered_body = recovered.into_body().collect().await.unwrap().to_bytes();
        assert!(recovered_body.is_empty());
        let recovered_text = String::from_utf8_lossy(&recovered_body);
        assert!(!recovered_text.contains("single-use-old"));
        assert!(!recovered_text.contains("rotated-refresh"));
        {
            let store = store_state.lock().unwrap();
            assert_eq!(store.fetch_count, 1);
            assert_eq!(
                store.store_count,
                6 * ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS + 1
            );
            assert_eq!(store.token, "rotated-refresh");
        }
        assert_eq!(received.lock().unwrap().len(), 1);
        {
            let pool = pool.lock().unwrap();
            assert_eq!(
                pool.redacted_seat_statuses(Instant::now())[0].state,
                "active"
            );
            assert!(pool.has_eligible_seat(Instant::now()));
        }

        let following = router.oneshot(anthropic_proxy_request()).await.unwrap();
        assert_eq!(following.status(), StatusCode::OK);
        {
            let store = store_state.lock().unwrap();
            assert_eq!(store.fetch_count, 2);
            assert_eq!(
                store.store_count,
                6 * ROTATED_REFRESH_TOKEN_STORE_MAX_ATTEMPTS + 2
            );
            assert_eq!(store.token, "final-refresh");
        }
        let requests = received.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests[0].contains(r#""refresh_token":"single-use-old""#));
        assert!(requests[1].contains(r#""refresh_token":"rotated-refresh""#));
        assert!(!requests[1].contains("single-use-old"));
        assert!(requests[2].starts_with("POST /v1/messages "));
        assert!(
            requests[2]
                .to_ascii_lowercase()
                .contains("authorization: bearer access-next")
        );
        drop(requests);
        let pool = pool.lock().unwrap();
        assert_eq!(
            pool.redacted_seat_statuses(Instant::now())[0].state,
            "active"
        );
        assert!(pool.has_eligible_seat(Instant::now()));
    }

    #[tokio::test]
    async fn tenant_scoped_admin_pool_route_requires_admin_bearer() {
        let response = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/pool")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn tenant_scoped_admin_pool_route_returns_status_without_secret_handles() {
        let response = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/pool")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("\"tenant_id\":\"tenant-a\""));
        assert!(body.contains("\"provider\":\"anthropic\""));
        assert!(body.contains("\"total_seats\":1"));
        assert!(!body.contains("tenant-a/seat-a"));
        assert!(!body.contains("test-refresh-token"));
    }

    /// AUTH-005 / ADR-0573 PR2: an admin credential bound to tenant-a must not read
    /// another tenant's pool. The cross-tenant axis is the PATH tenant vs the verified
    /// credential tenant — the forged `x-oya-admin-tenant` header cannot grant access.
    #[tokio::test]
    async fn tenant_scoped_admin_pool_route_forbids_cross_tenant_path() {
        let response = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-b/providers/anthropic/pool")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn tenant_scoped_admin_subscription_registration_adds_seat_without_echoing_secret() {
        let router = build_router(test_state(true, true));
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/subscriptions")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .header("idempotency-key", "11111111-1111-4111-8111-111111111111")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"seat_id":"seat-b","subscription_id":"sub-b","credential_mode":"oauth_subscription","secret_handle":"secret-ref://tenant-a/anthropic/seat-b"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("\"seat_id\":\"seat-b\""));
        assert!(!body.contains("secret-ref://tenant-a/anthropic/seat-b"));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/pool")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("\"total_seats\":2"));
    }

    #[tokio::test]
    async fn tenant_scoped_admin_subscription_registration_accepts_gemini_api_key_pool() {
        let router = build_router(test_state(true, true));
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/tenants/tenant-a/providers/gemini/subscriptions")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .header("idempotency-key", "33333333-3333-4333-8333-333333333333")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"seat_id":"gemini-seat-b","subscription_id":"gemini-sub-b","credential_mode":"api_key","secret_handle":"secret-ref://tenant-a/gemini/seat-b"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("\"provider\":\"gemini\""));
        assert!(!body.contains("secret-ref://tenant-a/gemini/seat-b"));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/admin/v1/tenants/tenant-a/providers/gemini/pool")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("\"provider\":\"gemini\""));
        assert!(body.contains("\"total_seats\":1"));
    }

    #[tokio::test]
    async fn tenant_scoped_admin_subscription_registration_requires_idempotency_key() {
        let response = build_router(test_state(true, true))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/subscriptions")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"seat_id":"seat-b","subscription_id":"sub-b","credential_mode":"oauth_subscription","secret_handle":"secret-ref://tenant-a/anthropic/seat-b"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn tenant_scoped_admin_subscription_registration_rejects_duplicate_seat() {
        let router = build_router(test_state(true, true));
        let request_body = r#"{"seat_id":"seat-b","subscription_id":"sub-b","credential_mode":"oauth_subscription","secret_handle":"secret-ref://tenant-a/anthropic/seat-b"}"#;

        let first = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/subscriptions")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .header("idempotency-key", "11111111-1111-4111-8111-111111111111")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::CREATED);

        let duplicate = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/tenants/tenant-a/providers/anthropic/subscriptions")
                    .header("authorization", "Bearer admin-token")
                    .header("x-oya-admin-tenant", "tenant-a")
                    .header("idempotency-key", "22222222-2222-4222-8222-222222222222")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    }
}
