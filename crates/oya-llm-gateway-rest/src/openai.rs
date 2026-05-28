//! OpenAI-canonical REST surface (`/v1/chat/completions`, `/v1/embeddings`,
//! `/v1/models`) with byte-passthrough SSE, OpenAI error envelope, and
//! `Retry-After`-honoring backpressure.
//!
//! # What this module owns
//! - The three OpenAI-shaped endpoints required by PRD §5 / AC-1, mounted on a
//!   default-routed group selected from the gateway state.
//! - The OpenAI error envelope shape (`{error:{message,type,param,code}}`) per
//!   PRD §4.1 / AC-1.4.
//! - The upstream `Retry-After` -> kernel cooldown propagation seam
//!   ([`extract_retry_after_seconds`] + [`UpstreamTransport`]).
//! - The hot-path failover loop that calls [`UpstreamTransport`], records key
//!   success/failure on the kernel, and translates exhaustion into a `503` with
//!   `Retry-After`.
//!
//! # What this module does NOT own
//! - Real HTTP transport. [`UpstreamTransport`] is a port; the app composition
//!   root supplies a hyper-backed adapter in production and a fake one in
//!   tests. This keeps the hot path testable without sockets.
//! - Per-tenant rate limits, body inspection, or audit emission — those are
//!   honest-bounded as `Unimplemented` (PRD §5 deferred items) and tracked at
//!   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-*`.
//!
//! # Streaming contract (load-bearing)
//! On `stream: true` the upstream body's data frames are forwarded verbatim
//! through [`axum::body::Body::from_stream`]. The body is NEVER collected,
//! parsed, buffered, or logged — true SSE passthrough with chunk-boundary
//! preservation (PRD §4.2 / AC-2.2).

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_TYPE, RETRY_AFTER};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::Stream;
use serde::{Deserialize, Serialize};

use oya_llm_gateway_kernel::{KeyId, ProviderChannel};

use crate::logging::DispatchLog;
use crate::state::{ChosenKey, GatewayState, KeyChoice};

/// The OpenAI error envelope (PRD §4.1 — verbatim across providers / SDKs).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiError {
    pub error: OpenAiErrorBody, // data_class: INTERNAL_ONLY
}

/// Inner envelope; `param`/`code` are optional per the OpenAI spec.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenAiErrorBody {
    pub message: String, // data_class: INTERNAL_ONLY
    #[serde(rename = "type")]
    pub error_type: String, // data_class: INTERNAL_ONLY
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>, // data_class: INTERNAL_ONLY
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>, // data_class: INTERNAL_ONLY
}

impl OpenAiError {
    /// Build an OpenAI error envelope with the gateway-specific `type` codes
    /// listed in PRD §4.1: `gateway_key_exhausted`, `gateway_provider_unavailable`,
    /// `budget_exceeded`, plus the standard OpenAI codes
    /// (`invalid_request_error`, `authentication_error`, ...).
    #[must_use]
    pub fn new(message: impl Into<String>, error_type: impl Into<String>) -> Self {
        OpenAiError {
            error: OpenAiErrorBody {
                message: message.into(),
                error_type: error_type.into(),
                param: None,
                code: None,
            },
        }
    }

    /// Render this envelope as a JSON response with the supplied status.
    /// `Retry-After` is added when present (PRD §4.1 / AC-4.1).
    #[must_use]
    pub fn into_response_with_retry_after(
        self,
        status: StatusCode,
        retry_after: Option<u64>,
    ) -> Response {
        let mut response = (status, Json(self)).into_response();
        if let Some(seconds) = retry_after
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(RETRY_AFTER, value);
        }
        response
    }
}

/// A typed bound for features the gateway will own but does not yet implement
/// (PRD §5 deferred items). Surfaced as 501 so callers / monitoring distinguish
/// "we know this is wrong" from random transport noise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unimplemented {
    /// OpenBao secret resolution path is not yet wired in this composition.
    /// Tracked at `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-openbao-wire-in`.
    OpenBaoResolution,
    /// Bedrock-shaped audit emission is not yet wired in this composition.
    /// Tracked at `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-bedrock-audit-emission`.
    BedrockAuditEmission,
    /// Per-tenant rate limiting / budget admission is not yet wired.
    /// Tracked at `registry/placeholder-debt/adr-follow-ups.yaml#adr-0373-llm-gateway-per-tenant-rate-limit`.
    PerTenantRateLimit,
}

impl Unimplemented {
    /// Stable error-type slug surfaced in the OpenAI error envelope.
    #[must_use]
    pub fn type_slug(self) -> &'static str {
        match self {
            Unimplemented::OpenBaoResolution => "gateway_unimplemented_openbao_resolution",
            Unimplemented::BedrockAuditEmission => "gateway_unimplemented_bedrock_audit",
            Unimplemented::PerTenantRateLimit => "gateway_unimplemented_per_tenant_rate_limit",
        }
    }

    /// Short human-facing message.
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Unimplemented::OpenBaoResolution => {
                "OpenBao secret resolution downstream is not yet wired in this composition"
            }
            Unimplemented::BedrockAuditEmission => {
                "Bedrock-shaped audit-chain emission downstream is not yet wired"
            }
            Unimplemented::PerTenantRateLimit => {
                "Per-tenant rate limiting / budget admission is not yet wired"
            }
        }
    }
}

/// What the gateway's hot-path observed back from the upstream provider. The
/// adapter trait below returns these so the kernel + REST surface remain
/// independent of any concrete HTTP client.
#[derive(Debug)]
pub struct UpstreamResponse {
    /// HTTP status received from the upstream.
    pub status: StatusCode, // data_class: INTERNAL_ONLY
    /// Optional `Retry-After` header value, parsed to seconds. `None` if absent
    /// or unparseable.
    pub retry_after_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    /// Selected response headers to echo back to the caller (e.g. `content-type`).
    pub headers: Vec<(String, String)>, // data_class: INTERNAL_ONLY
    /// Streaming response body. For SSE/`stream:true` this is the upstream's
    /// byte stream verbatim; for non-stream calls it's the fully-buffered body
    /// (small JSON, OK to materialize).
    pub body: UpstreamBody, // data_class: INTERNAL_ONLY
}

/// Body returned from an [`UpstreamTransport`] call. The streaming variant
/// preserves chunk boundaries (PRD §4.2 / AC-2.2).
pub enum UpstreamBody {
    /// A fully-buffered JSON/text body. Used for non-stream paths.
    Buffered(Bytes), // data_class: INTERNAL_ONLY
    /// A streaming byte body. Each item is one upstream chunk verbatim.
    Stream(Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>), // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for UpstreamBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpstreamBody::Buffered(b) => f.debug_tuple("Buffered").field(&b.len()).finish(),
            UpstreamBody::Stream(_) => f.debug_tuple("Stream").field(&"<stream>").finish(),
        }
    }
}

/// What the upstream transport could fail with (transport-only — HTTP errors
/// surface as a status inside [`UpstreamResponse`], not here).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpstreamError {
    /// Connection / DNS / TLS failure — always retryable.
    Transport(String), // data_class: INTERNAL_ONLY
    /// The request itself could not be built (malformed URL/header).
    BadRequest(String), // data_class: INTERNAL_ONLY
}

impl std::fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpstreamError::Transport(m) => write!(f, "upstream transport error: {m}"),
            UpstreamError::BadRequest(m) => write!(f, "upstream request build error: {m}"),
        }
    }
}

impl std::error::Error for UpstreamError {}

/// Port: the upstream HTTP transport seam. Swap in a hyper-backed real adapter
/// in production and a fake one in tests so the failover loop is unit-testable
/// without a socket.
pub trait UpstreamTransport: Send + Sync {
    /// Issue one upstream request. `path_and_query` is the OpenAI-canonical
    /// tail (e.g. `/v1/chat/completions`). `auth_headers` carry the live
    /// pooled key (caller MUST move them straight into the request and never
    /// log them). `body` is the small JSON request body; the response body is
    /// streamed verbatim.
    ///
    /// The argument list is intentionally wide so a transport adapter can
    /// build the upstream request without an intermediate DTO: the request is
    /// described by its method + URL components + headers + body, plus a
    /// `streaming` flag that selects buffered vs streamed body shape.
    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        channel: ProviderChannel,
        upstream_base_url: &str,
        method: &str,
        path_and_query: &str,
        auth_headers: Vec<(&'static str, String)>,
        forwarded_headers: Vec<(String, String)>,
        body: Bytes,
        streaming: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<UpstreamResponse, UpstreamError>> + Send + '_>,
    >;
}

/// Shared handler state: a [`GatewayState`] plus the upstream transport port.
pub struct OpenAiAppState {
    pub state: Arc<GatewayState>,
    pub transport: Arc<dyn UpstreamTransport>,
    /// Which group to dispatch to (the OpenAI-canonical surface uses a single
    /// default group, chosen at composition time).
    pub default_group: String, // data_class: INTERNAL_ONLY
}

impl OpenAiAppState {
    /// Build the shared handler state.
    #[must_use]
    pub fn new(
        state: Arc<GatewayState>,
        transport: Arc<dyn UpstreamTransport>,
        default_group: impl Into<String>,
    ) -> Arc<Self> {
        Arc::new(OpenAiAppState {
            state,
            transport,
            default_group: default_group.into(),
        })
    }
}

/// Build the OpenAI-canonical router (`/v1/chat/completions`,
/// `/v1/embeddings`, `/v1/models`). Mount this on top of (or alongside) the
/// per-group reverse-proxy router from [`crate::proxy::build_router`].
pub fn build_openai_router(state: Arc<OpenAiAppState>) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/v1/models", get(models))
        .with_state(state)
}

/// The minimal subset of the chat request that the gateway inspects (only to
/// branch on `stream` and to detect the model — body is forwarded verbatim).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatRequestPeek {
    #[serde(default)]
    pub stream: bool, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub model: Option<String>, // data_class: INTERNAL_ONLY
}

/// `POST /v1/chat/completions` — OpenAI-canonical chat completions.
///
/// Branches on `stream:true` for byte-passthrough SSE, else a buffered JSON
/// response. The body is forwarded verbatim to the upstream provider; the
/// gateway only peeks at `stream`/`model` for routing decisions.
async fn chat_completions(
    State(state): State<Arc<OpenAiAppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_openai(state, headers, "POST", "/v1/chat/completions", body, true).await
}

/// `POST /v1/embeddings` — OpenAI-canonical embeddings. Always non-stream.
async fn embeddings(
    State(state): State<Arc<OpenAiAppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_openai(state, headers, "POST", "/v1/embeddings", body, false).await
}

/// `GET /v1/models` — OpenAI-canonical model list. Always non-stream.
async fn models(State(state): State<Arc<OpenAiAppState>>, headers: HeaderMap) -> Response {
    handle_openai(state, headers, "GET", "/v1/models", Bytes::new(), false).await
}

/// Shared OpenAI-handler pipeline:
///   ingress-auth (constant-time) ->
///   group-resolve ->
///   peek `stream` (chat only) ->
///   failover loop: select_key -> dispatch -> record success/failure ->
///   stream/buffer response or 503 + Retry-After when exhausted.
async fn handle_openai(
    state: Arc<OpenAiAppState>,
    headers: HeaderMap,
    method: &str,
    path: &str,
    body: Bytes,
    chat_aware_stream: bool,
) -> Response {
    // 1. Ingress auth (constant-time).
    let presented = headers
        .get("x-oya-proxy-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
        })
        .unwrap_or("");
    if !state.state.auth().verify_ingress(presented) {
        state
            .state
            .metrics()
            .record_request(&state.default_group, "unknown", "unauthorized");
        return OpenAiError::new("invalid api key", "authentication_error")
            .into_response_with_retry_after(StatusCode::UNAUTHORIZED, None);
    }

    // 2. Resolve the default group.
    let Some(group) = state.state.group(&state.default_group) else {
        state
            .state
            .metrics()
            .record_request(&state.default_group, "unknown", "unknown_group");
        return OpenAiError::new("unknown gateway group", "invalid_request_error")
            .into_response_with_retry_after(StatusCode::NOT_FOUND, None);
    };
    let channel = group.adapter().channel();
    let channel_label = channel.as_str();

    // 3. Peek `stream` only on the chat-completions endpoint; embeddings and
    //    /models are always non-streaming.
    let streaming = if chat_aware_stream {
        serde_json::from_slice::<ChatRequestPeek>(&body)
            .map(|peek| peek.stream)
            .unwrap_or(false)
    } else {
        false
    };

    // 4. Failover loop.
    let policy = group.retry().clone();
    let attempts = policy.attempts();
    let mut attempt: u32 = 0;
    // Headers we forward (minus hop-by-hop + ingress credentials).
    let forwarded = build_forwarded_headers(&headers, group.adapter().managed_header_names());

    loop {
        attempt += 1;

        let chosen: ChosenKey = match group.choose_key() {
            KeyChoice::Chosen(c) => c,
            KeyChoice::Exhausted => {
                let retry_after = state.state.soonest_restore_seconds(&state.default_group);
                state.state.metrics().record_request(
                    &state.default_group,
                    channel_label,
                    "exhausted",
                );
                DispatchLog {
                    group: &state.default_group,
                    channel,
                    key_fp: "-",
                    attempt,
                    upstream_status: None,
                    outcome: "exhausted",
                }
                .emit();
                return OpenAiError::new(
                    "all upstream keys are cooling down",
                    "gateway_key_exhausted",
                )
                .into_response_with_retry_after(StatusCode::SERVICE_UNAVAILABLE, retry_after);
            }
            KeyChoice::Empty => {
                state.state.metrics().record_request(
                    &state.default_group,
                    channel_label,
                    "no_keys",
                );
                return OpenAiError::new(
                    "gateway has no upstream keys configured",
                    "gateway_provider_unavailable",
                )
                .into_response_with_retry_after(StatusCode::SERVICE_UNAVAILABLE, None);
            }
        };

        let upstream_base_url = group.adapter().upstream_base_url().to_string();
        let auth_headers = group.adapter().auth_headers(&chosen.raw_key);

        let dispatch_result = state
            .transport
            .dispatch(
                channel,
                &upstream_base_url,
                method,
                path,
                auth_headers,
                forwarded.clone(),
                body.clone(),
                streaming,
            )
            .await;

        match dispatch_result {
            Ok(resp) => {
                let status = resp.status;
                let retry_after = resp.retry_after_seconds;
                let retryable = policy.should_retry(status.as_u16());

                DispatchLog {
                    group: &state.default_group,
                    channel,
                    key_fp: &chosen.fingerprint,
                    attempt,
                    upstream_status: Some(status.as_u16()),
                    outcome: if retryable { "retry" } else { "return" },
                }
                .emit();

                if !retryable {
                    // Terminal response — stream/buffer through.
                    group.record_success(chosen.id);
                    state
                        .state
                        .metrics()
                        .record_key_success(&state.default_group, &chosen.fingerprint);
                    state.state.metrics().record_request(
                        &state.default_group,
                        channel_label,
                        "streamed",
                    );
                    return finish_upstream(resp);
                }

                // Retryable status: penalize the key (which also propagates an
                // upstream `Retry-After` into the kernel cooldown via
                // `record_failure_with_retry_after`).
                state.state.record_failure_with_retry_after(
                    &state.default_group,
                    chosen.id,
                    retry_after,
                );
                state
                    .state
                    .metrics()
                    .record_key_failure(&state.default_group, &chosen.fingerprint);

                if attempt >= attempts {
                    return give_up_with_retry_after(
                        &state,
                        channel_label,
                        retry_after
                            .or_else(|| state.state.soonest_restore_seconds(&state.default_group)),
                    );
                }
                state.state.metrics().record_retry(&state.default_group);
                // Backoff between attempts (respect upstream Retry-After when set).
                let backoff = if let Some(secs) = retry_after {
                    Duration::from_secs(secs.min(30))
                } else {
                    crate::proxy::backoff_duration(&policy, jitter_now())
                };
                if !backoff.is_zero() {
                    tokio::time::sleep(backoff).await;
                }
                // loop continues -> next key.
            }
            Err(UpstreamError::Transport(_)) => {
                DispatchLog {
                    group: &state.default_group,
                    channel,
                    key_fp: &chosen.fingerprint,
                    attempt,
                    upstream_status: None,
                    outcome: "transport_err",
                }
                .emit();
                state
                    .state
                    .record_failure_with_retry_after(&state.default_group, chosen.id, None);
                state
                    .state
                    .metrics()
                    .record_key_failure(&state.default_group, &chosen.fingerprint);
                if attempt >= attempts {
                    return give_up_with_retry_after(
                        &state,
                        channel_label,
                        state.state.soonest_restore_seconds(&state.default_group),
                    );
                }
                state.state.metrics().record_retry(&state.default_group);
                // small backoff before next attempt.
                let backoff = crate::proxy::backoff_duration(&policy, jitter_now());
                if !backoff.is_zero() {
                    tokio::time::sleep(backoff).await;
                }
                // loop continues.
            }
            Err(UpstreamError::BadRequest(_)) => {
                // Malformed request build — non-retryable; treat as a key
                // failure to surface it in metrics but do not loop forever.
                let _ignored: KeyId = chosen.id;
                group.record_failure(chosen.id);
                state
                    .state
                    .metrics()
                    .record_key_failure(&state.default_group, &chosen.fingerprint);
                state.state.metrics().record_request(
                    &state.default_group,
                    channel_label,
                    "retry_exhausted",
                );
                return OpenAiError::new(
                    "upstream request construction failed",
                    "gateway_provider_unavailable",
                )
                .into_response_with_retry_after(StatusCode::BAD_GATEWAY, None);
            }
        }
    }
}

/// Translate an [`UpstreamResponse`] into an axum response, preserving SSE
/// chunk boundaries via [`Body::from_stream`] when streaming.
fn finish_upstream(resp: UpstreamResponse) -> Response {
    let mut response = Response::builder().status(resp.status);
    if let Some(headers) = response.headers_mut() {
        for (name, value) in &resp.headers {
            let lname = name.to_ascii_lowercase();
            if STRIP_RESPONSE_HEADERS.contains(&lname.as_str()) {
                continue;
            }
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                headers.insert(hn, hv);
            }
        }
    }
    let body = match resp.body {
        UpstreamBody::Buffered(bytes) => Body::from(bytes),
        UpstreamBody::Stream(stream) => Body::from_stream(stream),
    };
    match response.body(body) {
        Ok(r) => r,
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(OpenAiError::new(
                "failed to build response",
                "gateway_provider_unavailable",
            )),
        )
            .into_response(),
    }
}

/// Render the terminal 503 with `Retry-After` when retries are exhausted.
fn give_up_with_retry_after(
    state: &Arc<OpenAiAppState>,
    channel_label: &str,
    retry_after: Option<u64>,
) -> Response {
    state
        .state
        .metrics()
        .record_request(&state.default_group, channel_label, "retry_exhausted");
    OpenAiError::new(
        "all upstream keys are cooling down",
        "gateway_key_exhausted",
    )
    .into_response_with_retry_after(StatusCode::SERVICE_UNAVAILABLE, retry_after)
}

/// Build the forwarded request headers: copy the inbound headers minus the
/// strip-list and minus any header the channel will overwrite with pooled
/// auth. Matches [`crate::proxy::build_upstream_headers`] in semantics.
fn build_forwarded_headers(inbound: &HeaderMap, managed: &[&str]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (name, value) in inbound {
        let lname = name.as_str().to_ascii_lowercase();
        if STRIP_REQUEST_HEADERS.contains(&lname.as_str()) {
            continue;
        }
        if managed.iter().any(|m| m.eq_ignore_ascii_case(&lname)) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            out.push((lname, v.to_string()));
        }
    }
    out
}

/// Headers that must never be forwarded upstream (hop-by-hop + client auth we
/// replace).
const STRIP_REQUEST_HEADERS: &[&str] = &[
    "host",
    "connection",
    "proxy-authorization",
    "proxy-authenticate",
    "transfer-encoding",
    "upgrade",
    "keep-alive",
    "te",
    "trailer",
    "x-oya-proxy-key",
    "authorization",
];

/// Response hop-by-hop headers we must not forward (the runtime sets framing).
const STRIP_RESPONSE_HEADERS: &[&str] = &[
    "connection",
    "transfer-encoding",
    "keep-alive",
    "upgrade",
    "trailer",
    "proxy-authenticate",
];

/// Best-effort jitter source for the inter-attempt backoff.
fn jitter_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0)
}

/// Parse an upstream `Retry-After` header (delta-seconds form per RFC 7231 §7.1.3).
///
/// Only the delta-seconds form is honored; HTTP-date `Retry-After` values are
/// ignored (returning `None`) to keep the parser dependency-free. The kernel's
/// jittered-cooldown stands in for HTTP-date values, so this is a safe fallback.
#[must_use]
pub fn extract_retry_after_seconds(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    trimmed.parse::<u64>().ok()
}

/// Pull the `Retry-After` (delta-seconds) value from a header list, if any.
#[must_use]
pub fn retry_after_from_headers(headers: &[(String, String)]) -> Option<u64> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("retry-after"))
        .and_then(|(_, value)| extract_retry_after_seconds(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_serializes_to_openai_shape() {
        let err = OpenAiError::new("nope", "invalid_request_error");
        let json = serde_json::to_value(&err).expect("serialize");
        assert_eq!(json["error"]["message"], "nope");
        assert_eq!(json["error"]["type"], "invalid_request_error");
        // `param` and `code` are omitted when None.
        assert!(json["error"].get("param").is_none());
        assert!(json["error"].get("code").is_none());
    }

    #[test]
    fn error_envelope_attaches_retry_after_when_set() {
        let resp = OpenAiError::new("cooling", "gateway_key_exhausted")
            .into_response_with_retry_after(StatusCode::SERVICE_UNAVAILABLE, Some(7));
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            resp.headers()
                .get(RETRY_AFTER)
                .and_then(|v| v.to_str().ok()),
            Some("7")
        );
    }

    #[test]
    fn error_envelope_omits_retry_after_when_absent() {
        let resp = OpenAiError::new("nope", "invalid_request_error")
            .into_response_with_retry_after(StatusCode::UNAUTHORIZED, None);
        assert!(!resp.headers().contains_key(RETRY_AFTER));
    }

    #[test]
    fn extract_retry_after_seconds_handles_delta_seconds_only() {
        assert_eq!(extract_retry_after_seconds("13"), Some(13));
        assert_eq!(extract_retry_after_seconds(" 42 "), Some(42));
        // HTTP-date forms are not parsed (returns None — kernel jitter stands in).
        assert_eq!(
            extract_retry_after_seconds("Wed, 21 Oct 2026 07:28:00 GMT"),
            None
        );
        assert_eq!(extract_retry_after_seconds(""), None);
    }

    #[test]
    fn retry_after_from_headers_is_case_insensitive() {
        let headers = vec![
            ("Content-Type".to_string(), "text/event-stream".to_string()),
            ("retry-after".to_string(), "5".to_string()),
        ];
        assert_eq!(retry_after_from_headers(&headers), Some(5));

        let upper = vec![("Retry-After".to_string(), "9".to_string())];
        assert_eq!(retry_after_from_headers(&upper), Some(9));
    }

    #[test]
    fn unimplemented_carries_stable_type_slugs() {
        assert_eq!(
            Unimplemented::OpenBaoResolution.type_slug(),
            "gateway_unimplemented_openbao_resolution"
        );
        assert_eq!(
            Unimplemented::BedrockAuditEmission.type_slug(),
            "gateway_unimplemented_bedrock_audit"
        );
        assert_eq!(
            Unimplemented::PerTenantRateLimit.type_slug(),
            "gateway_unimplemented_per_tenant_rate_limit"
        );
    }

    #[test]
    fn chat_request_peek_defaults_to_non_streaming() {
        let peek: ChatRequestPeek = serde_json::from_str(r#"{"model":"gpt-4o"}"#).expect("parse");
        assert!(!peek.stream);
        assert_eq!(peek.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn chat_request_peek_detects_stream_true() {
        let peek: ChatRequestPeek =
            serde_json::from_str(r#"{"stream":true,"model":"gpt-4o"}"#).expect("parse");
        assert!(peek.stream);
    }

    #[test]
    fn forwarded_headers_strip_credentials_and_hop_by_hop() {
        let mut h = HeaderMap::new();
        h.insert("host", HeaderValue::from_static("client.example"));
        h.insert(
            "x-oya-proxy-key",
            HeaderValue::from_static("ingress-secret"),
        );
        h.insert(
            "authorization",
            HeaderValue::from_static("Bearer client-token"),
        );
        h.insert("content-type", HeaderValue::from_static("application/json"));
        h.insert("accept", HeaderValue::from_static("text/event-stream"));

        let out = build_forwarded_headers(&h, &["authorization", "x-api-key"]);
        let names: Vec<&str> = out.iter().map(|(n, _)| n.as_str()).collect();
        assert!(!names.contains(&"host"));
        assert!(!names.contains(&"x-oya-proxy-key"));
        assert!(!names.contains(&"authorization"));
        assert!(names.contains(&"content-type"));
        assert!(names.contains(&"accept"));
    }

    /// Compile-time guard: [`UpstreamResponse`] uses [`Bytes`] for any
    /// buffered body, and the streaming variant is a boxed `Stream<...>`.
    /// This regression-checks the public contract.
    #[test]
    fn upstream_response_buffered_carries_bytes() {
        let resp = UpstreamResponse {
            status: StatusCode::OK,
            retry_after_seconds: None,
            headers: vec![(
                CONTENT_TYPE.as_str().to_string(),
                "application/json".to_string(),
            )],
            body: UpstreamBody::Buffered(Bytes::from_static(b"{}")),
        };
        match resp.body {
            UpstreamBody::Buffered(b) => assert_eq!(b.as_ref(), b"{}"),
            UpstreamBody::Stream(_) => panic!("expected buffered body"),
        }
    }
}
