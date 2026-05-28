//! The axum reverse-proxy app: SSE streaming passthrough + failover retry.
//!
//! # Streaming contract (load-bearing)
//! On a non-retryable upstream response, the upstream body is piped straight
//! into the axum response via [`axum::body::Body::from_stream`] over the
//! `hyper::body::Incoming` frame stream. The body is NEVER collected, parsed,
//! buffered, or logged — true SSE passthrough that works for both streamed and
//! unary responses.
//!
//! # Upstream client (no reqwest)
//! The upstream HTTP client is built directly on hyper via
//! [`hyper_util::client::legacy::Client`] over a [`hyper_rustls`] HTTPS
//! connector (ring crypto + webpki trust roots). This keeps the hot-path
//! proxy on the blessed hyper backbone with in-process TLS and connection
//! pooling, and avoids the heavier reqwest tree.
//!
//! # Failover loop
//! For each inbound request the proxy:
//! 1. authenticates the ingress proxy-key (constant-time),
//! 2. resolves the target group,
//! 3. up to `max_attempts` times: selects the next pooled key (round-robin),
//!    forwards the request with the channel's injected auth header, and on a
//!    retryable status (e.g. 429/5xx) records a key failure, rotates to the
//!    next key, and backs off with jitter,
//! 4. streams the first non-retryable response back, or returns 503 once
//!    attempts/keys are exhausted.
//!
//! The retry *decision* ([`RetryDecision`]) is a pure function so it is unit
//! tested without a network.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;

use crate::config::RetryPolicyConfig;
use crate::logging::DispatchLog;
use crate::state::{GatewayState, KeyChoice};

/// Outcome of the whole dispatch (for tests/metrics labels).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProxyOutcome {
    /// A response (any status) was streamed back from upstream.
    Streamed,
    /// All keys were blacklisted/in-cooldown at selection time.
    Exhausted,
    /// All attempts were spent on retryable statuses/transport errors.
    RetryExhausted,
    /// The group had no keys loaded.
    NoKeys,
}

impl ProxyOutcome {
    /// Stable metric label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProxyOutcome::Streamed => "streamed",
            ProxyOutcome::Exhausted => "exhausted",
            ProxyOutcome::RetryExhausted => "retry_exhausted",
            ProxyOutcome::NoKeys => "no_keys",
        }
    }
}

/// Errors surfaced as HTTP responses by the proxy handler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProxyError {
    /// Missing/invalid ingress proxy-key.
    Unauthorized,
    /// Unknown group in the path.
    UnknownGroup,
    /// No usable key (exhausted or empty pool).
    NoUsableKey(ProxyOutcome),
    /// All attempts spent.
    RetriesExhausted,
}

impl ProxyError {
    fn status(&self) -> StatusCode {
        match self {
            ProxyError::Unauthorized => StatusCode::UNAUTHORIZED,
            ProxyError::UnknownGroup => StatusCode::NOT_FOUND,
            ProxyError::NoUsableKey(_) | ProxyError::RetriesExhausted => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }

    fn message(&self) -> &'static str {
        match self {
            ProxyError::Unauthorized => "unauthorized",
            ProxyError::UnknownGroup => "unknown group",
            ProxyError::NoUsableKey(_) => "no usable upstream key (pool exhausted)",
            ProxyError::RetriesExhausted => "upstream retries exhausted",
        }
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        (self.status(), self.message()).into_response()
    }
}

/// What the failover loop should do after an attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryDecision {
    /// Stream this response back to the client (terminal).
    ReturnResponse,
    /// Retryable: rotate to the next key and try again.
    Retry,
    /// Retryable but no attempts remain → 503.
    GiveUp,
}

/// Pure failover decision. `attempt` is 1-based; `had_response` is false when
/// the attempt failed at the transport layer (no HTTP status received).
#[must_use]
pub fn decide_retry(
    policy: &RetryPolicyConfig,
    attempt: u32,
    had_response: bool,
    status: Option<u16>,
) -> RetryDecision {
    let attempts = policy.attempts();
    let retryable = match (had_response, status) {
        // A real response with a non-retryable status is terminal.
        (true, Some(code)) => policy.should_retry(code),
        // A response without a readable status: treat as terminal (rare).
        (true, None) => false,
        // Transport error (no response): always retryable.
        (false, _) => true,
    };
    if !retryable {
        return RetryDecision::ReturnResponse;
    }
    if attempt >= attempts {
        RetryDecision::GiveUp
    } else {
        RetryDecision::Retry
    }
}

/// Compute a jittered backoff [`Duration`] for `attempt` (1-based) given a
/// per-call `jitter` value. Pure.
#[must_use]
pub fn backoff_duration(policy: &RetryPolicyConfig, jitter: u64) -> Duration {
    let extra = if policy.backoff_jitter_millis == 0 {
        0
    } else {
        jitter % (policy.backoff_jitter_millis.saturating_add(1))
    };
    Duration::from_millis(policy.backoff_base_millis.saturating_add(extra))
}

/// Shared handler state.
pub type SharedState = Arc<GatewayState>;

/// Headers that must never be forwarded upstream (hop-by-hop + client auth we
/// replace). Lowercased wire names.
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
    // The ingress proxy-key header is consumed by the gateway, not forwarded.
    "x-oya-proxy-key",
];

/// Build the axum router. Routes:
/// - `GET /healthz` — liveness.
/// - `GET /metrics` — Prometheus exposition.
/// - `ANY /proxy/{group}/{*rest}` — the reverse proxy.
pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics_handler))
        .route("/proxy/{group}/{*rest}", any(proxy_handler))
        .with_state(state)
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn metrics_handler(State(state): State<SharedState>) -> Response {
    state.refresh_active_key_gauges();
    match state.metrics().render() {
        Ok(body) => (
            StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4",
            )],
            body,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "metrics render failed").into_response(),
    }
}

/// The reverse-proxy handler.
async fn proxy_handler(
    State(state): State<SharedState>,
    axum::extract::Path((group_name, rest)): axum::extract::Path<(String, String)>,
    request: Request,
) -> Response {
    // 1. Ingress auth (constant-time). The proxy-key rides an explicit header.
    let presented = request
        .headers()
        .get("x-oya-proxy-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !state.auth().verify_ingress(presented) {
        state
            .metrics()
            .record_request(&group_name, "unknown", "unauthorized");
        return ProxyError::Unauthorized.into_response();
    }

    // 2. Resolve group.
    let Some(group) = state.group(&group_name) else {
        state
            .metrics()
            .record_request(&group_name, "unknown", "unknown_group");
        return ProxyError::UnknownGroup.into_response();
    };
    let channel = group.adapter().channel();
    let channel_label = channel.as_str();

    // 3. Read the inbound body ONCE (the request body, not the response). For
    //    a true streaming proxy of large uploads this could itself stream, but
    //    request bodies for LLM calls are small JSON; buffering the *request*
    //    is fine and lets us replay it across failover attempts. We NEVER
    //    buffer the *response* body.
    let method = request.method().clone();
    let inbound_headers = request.headers().clone();
    let body_bytes = match axum::body::to_bytes(request.into_body(), MAX_REQUEST_BODY).await {
        Ok(b) => b,
        Err(_) => {
            return (StatusCode::PAYLOAD_TOO_LARGE, "request body too large").into_response();
        }
    };

    // Build the upstream path tail (path + original query).
    let tail = rest;
    let client = upstream_client();

    // 4. Failover loop.
    let policy = group.retry();
    let attempts = policy.attempts();
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;

        let choice = group.choose_key();
        let chosen = match choice {
            KeyChoice::Chosen(c) => c,
            KeyChoice::Exhausted => {
                state.metrics().record_request(
                    &group_name,
                    channel_label,
                    ProxyOutcome::Exhausted.as_str(),
                );
                return ProxyError::NoUsableKey(ProxyOutcome::Exhausted).into_response();
            }
            KeyChoice::Empty => {
                state.metrics().record_request(
                    &group_name,
                    channel_label,
                    ProxyOutcome::NoKeys.as_str(),
                );
                return ProxyError::NoUsableKey(ProxyOutcome::NoKeys).into_response();
            }
        };

        let url = group.adapter().upstream_url(&tail);
        let upstream_headers =
            build_upstream_headers(&inbound_headers, group.adapter().managed_header_names());

        // Build the upstream request on hyper. Inject pooled auth (the only
        // place a raw key touches a request). The request body is `Full<Bytes>`
        // — the small JSON payload buffered once for failover replay; the
        // *response* body is never buffered (streamed below).
        let send_result = match build_upstream_request(
            &method,
            &url,
            &upstream_headers,
            group.adapter().auth_headers(&chosen.raw_key),
            body_bytes.clone(),
        ) {
            Ok(request) => {
                let started = std::time::Instant::now();
                let result = client.request(request).await;
                let elapsed = started.elapsed().as_secs_f64();
                state
                    .metrics()
                    .observe_upstream_latency(&group_name, channel_label, elapsed);
                result
            }
            Err(_) => {
                // A malformed upstream URI/header is not a transport failure we
                // should retry forever; surface it as a terminal 502-class
                // error after recording the attempt against the key.
                group.record_failure(chosen.id);
                state
                    .metrics()
                    .record_key_failure(&group_name, &chosen.fingerprint);
                state.metrics().record_request(
                    &group_name,
                    channel_label,
                    ProxyOutcome::RetryExhausted.as_str(),
                );
                return ProxyError::RetriesExhausted.into_response();
            }
        };

        let (had_response, status_code) = match &send_result {
            Ok(resp) => (true, Some(resp.status().as_u16())),
            Err(_) => (false, None),
        };

        let decision = decide_retry(policy, attempt, had_response, status_code);

        DispatchLog {
            group: &group_name,
            channel,
            key_fp: &chosen.fingerprint,
            attempt,
            upstream_status: status_code,
            outcome: match decision {
                RetryDecision::ReturnResponse => "return",
                RetryDecision::Retry => "retry",
                RetryDecision::GiveUp => "give_up",
            },
        }
        .emit();

        match decision {
            RetryDecision::ReturnResponse => {
                // Terminal. A real response → success-or-not for the key.
                match send_result {
                    Ok(resp) => {
                        // Any received HTTP response counts as the key
                        // "working" for selection purposes (auth/quota errors
                        // that are retryable were already branched above).
                        group.record_success(chosen.id);
                        state
                            .metrics()
                            .record_key_success(&group_name, &chosen.fingerprint);
                        state.metrics().record_request(
                            &group_name,
                            channel_label,
                            ProxyOutcome::Streamed.as_str(),
                        );
                        return stream_response(resp);
                    }
                    Err(_) => {
                        // had_response was false but decision said return only
                        // when not retryable; transport errors are always
                        // retryable, so this arm is unreachable in practice.
                        return ProxyError::RetriesExhausted.into_response();
                    }
                }
            }
            RetryDecision::Retry | RetryDecision::GiveUp => {
                // Retryable status or transport error: penalize the key.
                group.record_failure(chosen.id);
                state
                    .metrics()
                    .record_key_failure(&group_name, &chosen.fingerprint);
                if matches!(decision, RetryDecision::GiveUp) || attempt >= attempts {
                    state.metrics().record_request(
                        &group_name,
                        channel_label,
                        ProxyOutcome::RetryExhausted.as_str(),
                    );
                    return ProxyError::RetriesExhausted.into_response();
                }
                state.metrics().record_retry(&group_name);
                let backoff = backoff_duration(policy, jitter_now());
                if !backoff.is_zero() {
                    tokio::time::sleep(backoff).await;
                }
                // loop continues → next key
            }
        }
    }
}

/// Max inbound request body we will buffer for failover replay (1 MiB). LLM
/// request payloads are small JSON; this is a safety cap, not a streaming
/// limit on responses (responses are never buffered).
const MAX_REQUEST_BODY: usize = 1024 * 1024;

/// Pipe the upstream response straight through: status + headers + STREAMED
/// body. The body is `Body::from_stream(...)` over the hyper `Incoming` frame
/// stream — never collected, parsed, buffered, or logged.
fn stream_response(upstream: hyper::Response<Incoming>) -> Response {
    let status =
        StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);

    // Copy response headers verbatim except hop-by-hop ones.
    let mut headers = HeaderMap::new();
    for (name, value) in upstream.headers() {
        let lname = name.as_str().to_ascii_lowercase();
        if STRIP_RESPONSE_HEADERS.contains(&lname.as_str()) {
            continue;
        }
        if let (Ok(hn), Ok(hv)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            headers.insert(hn, hv);
        }
    }

    // TRUE STREAMING PASSTHROUGH: the upstream body's DATA frames become the
    // response body's byte stream directly. Each frame is forwarded as soon as
    // it arrives (SSE chunks pass straight through); non-data frames (trailers)
    // are dropped. No buffering, no parsing, no logging of the body.
    let upstream_body = upstream.into_body();
    let byte_stream = futures_util::stream::unfold(upstream_body, |mut body| async move {
        loop {
            match body.frame().await {
                Some(Ok(frame)) => match frame.into_data() {
                    Ok(chunk) => return Some((Ok::<Bytes, std::io::Error>(chunk), body)),
                    // A trailers/non-data frame: skip it and keep reading.
                    Err(_non_data) => continue,
                },
                Some(Err(err)) => {
                    return Some((Err(std::io::Error::other(err.to_string())), body));
                }
                None => return None,
            }
        }
    });
    let body = Body::from_stream(byte_stream);

    let mut response = Response::new(body);
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

/// Response hop-by-hop headers we must not forward (the runtime sets framing).
const STRIP_RESPONSE_HEADERS: &[&str] = &[
    "connection",
    "transfer-encoding",
    "keep-alive",
    "upgrade",
    "trailer",
    "proxy-authenticate",
    // Let the client see content-length only if upstream framed it; for SSE it
    // is absent and we stream chunked, which is correct.
];

/// Build the forwarded request headers: copy the inbound headers minus the
/// strip-list and minus any header the channel will overwrite with pooled
/// auth.
fn build_upstream_headers(inbound: &HeaderMap, managed: &[&str]) -> Vec<(String, String)> {
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

/// The upstream client type: hyper-util legacy client over a rustls HTTPS
/// connector, sending `Full<Bytes>` request bodies and receiving streaming
/// `Incoming` response bodies.
type UpstreamClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>;

/// Build the upstream `hyper::Request`: method + URI + forwarded headers +
/// injected pooled-auth headers + the buffered request body.
///
/// SECURITY: `auth_headers` carry the live pooled key; they are written only
/// into the outbound request here and are never logged.
fn build_upstream_request(
    method: &Method,
    url: &str,
    forwarded_headers: &[(String, String)],
    auth_headers: Vec<(&'static str, String)>,
    body: Bytes,
) -> Result<hyper::Request<Full<Bytes>>, hyper::http::Error> {
    let mut builder = hyper::Request::builder().method(method.as_str()).uri(url);
    if let Some(headers) = builder.headers_mut() {
        for (name, value) in forwarded_headers {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                headers.append(hn, hv);
            }
        }
        // Injected pooled auth overwrites any client-supplied value (the
        // managed-header strip already removed those upstream).
        for (name, value) in auth_headers {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                headers.insert(hn, hv);
            }
        }
    }
    builder.body(Full::new(body))
}

fn upstream_client() -> UpstreamClient {
    // A process-wide client so the legacy connection pool is reused across
    // requests. No total-request timeout: SSE streams are long-lived (the
    // connector applies a connect timeout below to guard dead upstreams).
    use std::sync::OnceLock;
    static CLIENT: OnceLock<UpstreamClient> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let mut http = HttpConnector::new();
            http.enforce_http(false);
            http.set_connect_timeout(Some(Duration::from_secs(10)));
            let https = hyper_rustls::HttpsConnectorBuilder::new()
                .with_webpki_roots()
                .https_or_http()
                .enable_http1()
                .wrap_connector(http);
            Client::builder(TokioExecutor::new()).build(https)
        })
        .clone()
}

fn jitter_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(max_attempts: u32, statuses: Vec<u16>) -> RetryPolicyConfig {
        RetryPolicyConfig {
            retry_on_statuses: statuses,
            max_attempts,
            backoff_base_millis: 0,
            backoff_jitter_millis: 0,
        }
    }

    #[test]
    fn non_retryable_status_returns_response() {
        let p = policy(3, vec![429, 500]);
        assert_eq!(
            decide_retry(&p, 1, true, Some(200)),
            RetryDecision::ReturnResponse
        );
        assert_eq!(
            decide_retry(&p, 1, true, Some(400)),
            RetryDecision::ReturnResponse
        );
        // 401 is NOT in the retry set → terminal (surfaced to caller).
        assert_eq!(
            decide_retry(&p, 1, true, Some(401)),
            RetryDecision::ReturnResponse
        );
    }

    #[test]
    fn retryable_status_retries_until_cap() {
        let p = policy(3, vec![429, 500, 503]);
        assert_eq!(decide_retry(&p, 1, true, Some(429)), RetryDecision::Retry);
        assert_eq!(decide_retry(&p, 2, true, Some(503)), RetryDecision::Retry);
        // Attempt 3 is the last → give up.
        assert_eq!(decide_retry(&p, 3, true, Some(500)), RetryDecision::GiveUp);
    }

    #[test]
    fn transport_error_is_always_retryable() {
        let p = policy(2, vec![429]);
        assert_eq!(decide_retry(&p, 1, false, None), RetryDecision::Retry);
        assert_eq!(decide_retry(&p, 2, false, None), RetryDecision::GiveUp);
    }

    #[test]
    fn single_attempt_policy_never_retries() {
        let p = policy(1, vec![429, 500]);
        // Even a retryable status gives up immediately when cap is 1.
        assert_eq!(decide_retry(&p, 1, true, Some(429)), RetryDecision::GiveUp);
        assert_eq!(decide_retry(&p, 1, false, None), RetryDecision::GiveUp);
    }

    #[test]
    fn backoff_is_bounded_by_base_plus_jitter() {
        let p = RetryPolicyConfig {
            retry_on_statuses: vec![429],
            max_attempts: 3,
            backoff_base_millis: 100,
            backoff_jitter_millis: 50,
        };
        // jitter folds into [0,50]; total in [100,150].
        let d0 = backoff_duration(&p, 0);
        let d_max = backoff_duration(&p, 50);
        let d_wrap = backoff_duration(&p, 51); // 51 % 51 = 0
        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d_max, Duration::from_millis(150));
        assert_eq!(d_wrap, Duration::from_millis(100));
    }

    #[test]
    fn backoff_zero_jitter_is_exactly_base() {
        let p = policy(3, vec![429]); // base 0, jitter 0
        assert_eq!(backoff_duration(&p, 12345), Duration::from_millis(0));
    }

    #[test]
    fn outcome_labels_are_stable() {
        assert_eq!(ProxyOutcome::Streamed.as_str(), "streamed");
        assert_eq!(ProxyOutcome::Exhausted.as_str(), "exhausted");
        assert_eq!(ProxyOutcome::RetryExhausted.as_str(), "retry_exhausted");
        assert_eq!(ProxyOutcome::NoKeys.as_str(), "no_keys");
    }

    #[test]
    fn strip_lists_cover_auth_and_hop_by_hop() {
        assert!(STRIP_REQUEST_HEADERS.contains(&"x-oya-proxy-key"));
        assert!(STRIP_REQUEST_HEADERS.contains(&"host"));
        assert!(STRIP_RESPONSE_HEADERS.contains(&"transfer-encoding"));
    }

    #[test]
    fn build_upstream_headers_drops_strip_and_managed() {
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

        let out = build_upstream_headers(&h, &["authorization"]);
        let names: Vec<&str> = out.iter().map(|(n, _)| n.as_str()).collect();
        // host + proxy-key stripped; authorization is managed → dropped.
        assert!(!names.contains(&"host"));
        assert!(!names.contains(&"x-oya-proxy-key"));
        assert!(!names.contains(&"authorization"));
        // benign headers preserved.
        assert!(names.contains(&"content-type"));
        assert!(names.contains(&"accept"));
    }

    #[test]
    fn build_upstream_request_injects_auth_and_forwards_headers() {
        let req = build_upstream_request(
            &Method::POST,
            "https://api.openai.com/v1/chat/completions",
            &[("content-type".to_string(), "application/json".to_string())],
            vec![("authorization", "Bearer sk-POOLED".to_string())],
            Bytes::from_static(b"{}"),
        )
        .expect("request builds");
        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.uri(), "https://api.openai.com/v1/chat/completions");
        assert_eq!(
            req.headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some("Bearer sk-POOLED")
        );
        assert_eq!(
            req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
    }
}
