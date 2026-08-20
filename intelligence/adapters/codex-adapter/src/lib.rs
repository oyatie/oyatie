//! cloud-intelligence Codex adapter — OpenAI Codex OAuth subscription pool (ADR-0384 Path B, v1).
//!
//! Targets the OpenAI Codex Sign-in-with-ChatGPT OAuth flow + data endpoint.
//! Wire format is reverse-engineered from auth2api / CLIProxyAPI (Stage-6 gap #11).
//!
//! IMPORTANT operator notes:
//! - `CLI_VERSION` is hard-coded. Update on every Codex CLI release that changes
//!   the User-Agent contract with chatgpt.com.
//! - The refresh token must be obtained manually from a logged-in ChatGPT browser
//!   session (no automated browser OAuth flow at this stage — Stage-6 deferred per
//!   ADR-0384 §v1-scope).
//! - The data endpoint (`/backend-api/codex/responses`) is undocumented and
//!   reverse-engineered; it may change without notice.
//!
//! # Non-claims
//! - No automated browser OAuth flow (operator manually seeds refresh token).
//! - No Anthropic compat layer.
//! - cli_version is hard-coded (`cli/0.27.0`); operator must bump on Codex CLI bumps.
//! - Data endpoint is undocumented + reverse-engineered.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use serde::Deserialize;
use tracing::debug;

pub use intelligence_kernel::{AgentId, Provider, SeatId, TenantId};

/// Heap-allocated OpenAI-compatible response byte stream.
pub type OpenAiByteStream =
    Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static>>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default ChatGPT base URL.
const CODEX_DEFAULT_BASE_URL: &str = "https://chatgpt.com";

/// Default OpenAI-compatible API base URL for API-key mode.
const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com";

/// CLI version impersonation header value (Stage-6 gap #11).
/// Hard-coded per ADR-0384 non_claims. Operator must update on Codex CLI bumps.
const CLI_VERSION: &str = "cli/0.27.0";

/// Path for the session token refresh endpoint.
const SESSION_PATH: &str = "/api/auth/session";

/// Path for the Codex completions data endpoint.
const CODEX_RESPONSES_PATH: &str = "/backend-api/codex/responses";

/// Path for OpenAI-compatible chat completions in API-key mode.
const OPENAI_CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";

/// Path for OpenAI-compatible embeddings in API-key mode.
const OPENAI_EMBEDDINGS_PATH: &str = "/v1/embeddings";

/// X-OpenAI-Beta header value required by the Codex data plane.
const OPENAI_BETA_CODEX: &str = "codex-runs";

/// RFC 7230 §6.1 hop-by-hop headers — never forwarded upstream or to caller.
pub const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified error type for the Codex adapter layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexAdapterError {
    /// Session endpoint returned an error — refresh token is likely expired or revoked.
    RefreshFailed(String), // data_class: INTERNAL_ONLY
    /// Upstream Codex data endpoint returned an error status.
    UpstreamError {
        status: u16,  // data_class: INTERNAL_ONLY
        body: String, // data_class: INTERNAL_ONLY
    },
    /// HTTP transport-level error (connection refused, TLS failure, etc.).
    TransportError(String), // data_class: INTERNAL_ONLY
    /// Upstream responded with rate-limit (429); includes Retry-After seconds if present.
    RateLimited {
        retry_after_secs: Option<u64>, // data_class: INTERNAL_ONLY
    },
}

// ---------------------------------------------------------------------------
// Wire types — session endpoint
// ---------------------------------------------------------------------------

/// Response body shape for `GET /api/auth/session`.
/// The reverse-engineered shape returns a JSON object with `accessToken` at the
/// top level. Other fields are ignored.
#[derive(Deserialize)]
struct SessionResponse {
    #[serde(rename = "accessToken")]
    access_token: String, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Public value objects
// ---------------------------------------------------------------------------

/// Resolved tokens returned from a successful session refresh.
#[derive(Debug)]
pub struct CodexTokens {
    pub access_token: String, // data_class: INTERNAL_ONLY
}

/// Inbound proxy request forwarded to the Codex data endpoint.
#[derive(Clone, Debug)]
pub struct CodexProxyRequest {
    pub body: Vec<u8>,                           // data_class: INTERNAL_ONLY
    pub extra_headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

/// Response returned from the Codex data endpoint.
#[derive(Clone, Debug)]
pub struct CodexProxyResponse {
    pub status: u16,                       // data_class: INTERNAL_ONLY
    pub headers: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    pub body: Vec<u8>,                     // data_class: INTERNAL_ONLY
}

fn hop_by_hop_set() -> std::collections::HashSet<&'static str> {
    HOP_BY_HOP.iter().copied().collect()
}

fn connection_tokens(headers: &BTreeMap<String, String>) -> std::collections::HashSet<String> {
    // Caller headers arrive in a BTreeMap keyed by the caller's original casing,
    // so an exact `get("connection")` misses `Connection:` (capital C) and any
    // mixed-case variant, leaking the nominated headers upstream. Iterate every
    // key with a case-insensitive match instead.
    let mut tokens = std::collections::HashSet::new();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("connection") {
            tokens.extend(v.split(',').map(|t| t.trim().to_lowercase()));
        }
    }
    tokens
}

/// Caller-controlled provider headers that are forwarded upstream on the
/// API-key paths. Every other `openai-*` / `x-openai-*` header is provider
/// control surface and is stripped so a caller cannot inject provider/beta
/// directives.
const ALLOWED_PROVIDER_CALLER_HEADERS: &[&str] = &["openai-organization", "openai-project"];

/// Return true when `key_lower` is a provider-controlled header that must not be
/// forwarded from caller-supplied headers. Strips any `openai-*` / `x-openai-*`
/// header except the explicit caller-safe allowlist.
fn is_provider_control_header(key_lower: &str) -> bool {
    if ALLOWED_PROVIDER_CALLER_HEADERS.contains(&key_lower) {
        return false;
    }
    key_lower.starts_with("openai-") || key_lower.starts_with("x-openai-")
}

fn retry_after_secs(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

fn filtered_response_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    let hop_by_hop = hop_by_hop_set();
    let response_connection_tokens: std::collections::HashSet<String> = headers
        .get("connection")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
        .unwrap_or_default();

    let mut filtered = BTreeMap::new();
    for (k, v) in headers {
        let key_lower = k.as_str().to_lowercase();
        if hop_by_hop.contains(key_lower.as_str()) {
            continue;
        }
        if response_connection_tokens.contains(&key_lower) {
            continue;
        }
        if let Ok(val) = v.to_str() {
            filtered.insert(k.as_str().to_string(), val.to_string());
        }
    }
    filtered
}

fn allowed_openai_compatible_path(path: &str) -> Option<&'static str> {
    match path {
        OPENAI_CHAT_COMPLETIONS_PATH => Some(OPENAI_CHAT_COMPLETIONS_PATH),
        OPENAI_EMBEDDINGS_PATH => Some(OPENAI_EMBEDDINGS_PATH),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// OpenAiApiKeyAdapter
// ---------------------------------------------------------------------------

/// OpenAI-compatible API-key adapter for Codex/OpenAI tenant subscriptions.
///
/// This path is intentionally separate from [`CodexAdapter`]'s ChatGPT OAuth
/// session flow. It targets the documented OpenAI-compatible API-key shape:
/// `POST /v1/chat/completions` with `Authorization: Bearer <provider_api_key>`.
pub struct OpenAiApiKeyAdapter {
    base_url: String,           // data_class: INTERNAL_ONLY
    http: Arc<reqwest::Client>, // data_class: INTERNAL_ONLY
}

impl OpenAiApiKeyAdapter {
    /// Construct with default OpenAI API base URL.
    pub fn new(http: Arc<reqwest::Client>) -> Self {
        Self {
            base_url: OPENAI_DEFAULT_BASE_URL.to_string(),
            http,
        }
    }

    /// Construct with a custom base URL. Used in tests against a local fake provider.
    pub fn with_base_url(http: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http,
        }
    }

    /// Forward a non-streaming OpenAI-compatible chat-completions request using
    /// a provider API key.
    pub async fn proxy_chat_completions(
        &self,
        api_key: &str,
        request: CodexProxyRequest,
    ) -> Result<CodexProxyResponse, CodexAdapterError> {
        self.proxy_openai_compatible_path(api_key, OPENAI_CHAT_COMPLETIONS_PATH, request)
            .await
    }

    /// Forward a non-streaming OpenAI-compatible request using a provider API
    /// key. The path is exact-match allowlisted to avoid proxying arbitrary
    /// provider URLs through this adapter.
    pub async fn proxy_openai_compatible_path(
        &self,
        api_key: &str,
        path: &str,
        request: CodexProxyRequest,
    ) -> Result<CodexProxyResponse, CodexAdapterError> {
        let Some(path) = allowed_openai_compatible_path(path) else {
            return Err(CodexAdapterError::UpstreamError {
                status: 400,
                body: "unsupported OpenAI-compatible path".to_string(),
            });
        };
        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "proxying OpenAI-compatible API-key request");

        let hop_by_hop = hop_by_hop_set();
        let connection_tokens = connection_tokens(&request.extra_headers);
        let mut req_builder = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .body(request.body);

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length" | "user-agent"
            ) {
                continue;
            }
            if is_provider_control_header(&key_lower) {
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
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(CodexAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        let body = resp
            .bytes()
            .await
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?
            .to_vec();

        Ok(CodexProxyResponse {
            status,
            headers,
            body,
        })
    }

    /// Forward a streaming OpenAI-compatible chat-completions request using a
    /// provider API key and return raw SSE bytes.
    pub async fn proxy_chat_completions_stream(
        &self,
        api_key: &str,
        request: CodexProxyRequest,
    ) -> Result<(u16, BTreeMap<String, String>, OpenAiByteStream), CodexAdapterError> {
        self.proxy_openai_compatible_path_stream(api_key, OPENAI_CHAT_COMPLETIONS_PATH, request)
            .await
    }

    /// Forward a streaming OpenAI-compatible request using a provider API key.
    /// Streaming is currently only allowlisted for chat completions.
    pub async fn proxy_openai_compatible_path_stream(
        &self,
        api_key: &str,
        path: &str,
        request: CodexProxyRequest,
    ) -> Result<(u16, BTreeMap<String, String>, OpenAiByteStream), CodexAdapterError> {
        if path != OPENAI_CHAT_COMPLETIONS_PATH {
            return Err(CodexAdapterError::UpstreamError {
                status: 400,
                body: "unsupported streaming OpenAI-compatible path".to_string(),
            });
        }
        let url = format!("{}{}", self.base_url, OPENAI_CHAT_COMPLETIONS_PATH);
        debug!(url = %url, "opening OpenAI-compatible API-key SSE stream");

        let hop_by_hop = hop_by_hop_set();
        let connection_tokens = connection_tokens(&request.extra_headers);
        let mut req_builder = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Accept", "text/event-stream")
            .body(request.body);

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length" | "user-agent" | "accept"
            ) {
                continue;
            }
            if is_provider_control_header(&key_lower) {
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
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(CodexAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        let byte_stream: OpenAiByteStream = Box::pin(resp.bytes_stream());
        Ok((status, headers, byte_stream))
    }
}

// ---------------------------------------------------------------------------
// CodexAdapter
// ---------------------------------------------------------------------------

/// D3 OpenAI Codex provider adapter.
///
/// Responsible for:
/// 1. Refreshing the ChatGPT session token by POSTing the refresh cookie to
///    `<base>/api/auth/session` and extracting the `accessToken` from the
///    JSON response.
/// 2. Routing proxied requests to `<base>/backend-api/codex/responses` with
///    the correct Bearer token and `User-Agent: cli/<version>` impersonation
///    header (Stage-6 gap #11).
///
/// The adapter borrows `Arc<reqwest::Client>` — it does NOT construct one per
/// request. The shared client lives in the caller's `Arc` so TLS sessions and
/// keep-alive connections are amortized across the full request lifetime.
///
/// NOTE: Codex OAuth uses the session cookie model, NOT a `client_id` grant
/// flow like Anthropic. There is intentionally NO `client_id` field.
pub struct CodexAdapter {
    base_url: String,           // data_class: INTERNAL_ONLY
    http: Arc<reqwest::Client>, // data_class: INTERNAL_ONLY
    cli_version: String,        // data_class: INTERNAL_ONLY
}

impl CodexAdapter {
    /// Construct with a shared reqwest client. Uses default base URL
    /// (`https://chatgpt.com`) and cli-version (`cli/0.27.0`).
    pub fn new(http: Arc<reqwest::Client>) -> Self {
        Self {
            base_url: CODEX_DEFAULT_BASE_URL.to_string(),
            http,
            cli_version: CLI_VERSION.to_string(),
        }
    }

    /// Construct with a custom base URL. Used in tests against a local mock server.
    pub fn with_base_url(http: Arc<reqwest::Client>, base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http,
            cli_version: CLI_VERSION.to_string(),
        }
    }

    /// Return the cli_version string this adapter sends as User-Agent.
    pub fn cli_version(&self) -> &str {
        &self.cli_version
    }

    /// Refresh the ChatGPT session token.
    ///
    /// POSTs `refresh_token` as a cookie value in the `__Secure-next-auth.session-token`
    /// cookie to `<base>/api/auth/session`. The response body is a JSON object
    /// whose `accessToken` field contains the new bearer token.
    ///
    /// Returns [`CodexAdapterError::RefreshFailed`] on non-2xx or JSON parse failure.
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<CodexTokens, CodexAdapterError> {
        let url = format!("{}{}", self.base_url, SESSION_PATH);
        debug!(url = %url, "refreshing Codex session token");

        let resp = self
            .http
            .post(&url)
            .header("User-Agent", &self.cli_version)
            .header(
                "Cookie",
                format!("__Secure-next-auth.session-token={refresh_token}"),
            )
            .send()
            .await
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?;

        let status = resp.status().as_u16();
        if status == 429 {
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            return Err(CodexAdapterError::RateLimited { retry_after_secs });
        }
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CodexAdapterError::RefreshFailed(format!(
                "session refresh failed: HTTP {status}: {body}"
            )));
        }

        let session: SessionResponse = resp
            .json()
            .await
            .map_err(|e| CodexAdapterError::RefreshFailed(e.to_string()))?;

        Ok(CodexTokens {
            access_token: session.access_token,
        })
    }

    /// Forward `request` to the Codex data endpoint using `access_token`.
    ///
    /// Sets:
    /// - `Authorization: Bearer <access_token>`
    /// - `User-Agent: <cli_version>` (Stage-6 gap #11 impersonation)
    /// - `X-OpenAI-Beta: codex-runs`
    ///
    /// Hop-by-hop headers from `request.extra_headers` are stripped before
    /// forwarding. Hop-by-hop headers in the upstream response are stripped
    /// before returning.
    pub async fn proxy(
        &self,
        access_token: &str,
        request: CodexProxyRequest,
    ) -> Result<CodexProxyResponse, CodexAdapterError> {
        let url = format!("{}{}", self.base_url, CODEX_RESPONSES_PATH);
        debug!(url = %url, "proxying request to Codex data endpoint");

        let hop_by_hop = hop_by_hop_set();

        let connection_tokens = connection_tokens(&request.extra_headers);

        let mut req_builder = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("User-Agent", &self.cli_version)
            .header("X-OpenAI-Beta", OPENAI_BETA_CODEX)
            .body(request.body);

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length" | "user-agent"
            ) {
                continue;
            }
            if is_provider_control_header(&key_lower) {
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
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?;

        let status = resp.status().as_u16();

        if status == 429 {
            return Err(CodexAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }

        let headers = filtered_response_headers(resp.headers());
        let body = resp
            .bytes()
            .await
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?
            .to_vec();

        Ok(CodexProxyResponse {
            status,
            headers,
            body,
        })
    }

    /// Forward `request` to the Codex data endpoint and return the raw response
    /// bytes stream for SSE / streaming responses.
    ///
    /// Same header policy as [`CodexAdapter::proxy`] but returns the raw
    /// [`bytes::Bytes`] chunks via the reqwest streaming API. The caller is
    /// responsible for framing SSE events from the byte stream.
    ///
    /// Returns `(status, headers, bytes_stream)` on success.
    pub async fn proxy_stream(
        &self,
        access_token: &str,
        request: CodexProxyRequest,
    ) -> Result<(u16, BTreeMap<String, String>, OpenAiByteStream), CodexAdapterError> {
        let url = format!("{}{}", self.base_url, CODEX_RESPONSES_PATH);
        debug!(url = %url, "opening SSE stream to Codex data endpoint");

        let hop_by_hop = hop_by_hop_set();

        let connection_tokens = connection_tokens(&request.extra_headers);

        let mut req_builder = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("User-Agent", &self.cli_version)
            .header("X-OpenAI-Beta", OPENAI_BETA_CODEX)
            .body(request.body);

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization" | "host" | "content-length" | "user-agent"
            ) {
                continue;
            }
            if is_provider_control_header(&key_lower) {
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
            .map_err(|e| CodexAdapterError::TransportError(e.to_string()))?;

        let status = resp.status().as_u16();

        if status == 429 {
            return Err(CodexAdapterError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }

        let headers = filtered_response_headers(resp.headers());
        let byte_stream: OpenAiByteStream = Box::pin(resp.bytes_stream());
        Ok((status, headers, byte_stream))
    }
}

// ---------------------------------------------------------------------------
// Unit tests for value-object constructors and error variants
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt as _;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    async fn openai_api_key_proxy_injects_provider_auth_and_strips_client_auth() {
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
        let adapter =
            OpenAiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

        let mut headers = BTreeMap::new();
        headers.insert(
            "authorization".to_string(),
            "Bearer caller-token".to_string(),
        );
        headers.insert("connection".to_string(), "x-drop-me".to_string());
        headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());
        headers.insert("host".to_string(), "attacker.example".to_string());
        headers.insert("openai-beta".to_string(), "assistants=v2".to_string());
        headers.insert("x-openai-beta".to_string(), "codex-runs".to_string());
        headers.insert("openai-organization".to_string(), "org_test".to_string());
        headers.insert("openai-project".to_string(), "proj_test".to_string());

        let response = adapter
            .proxy_chat_completions(
                "sk-provider",
                CodexProxyRequest {
                    body: br#"{"model":"gpt-test","messages":[]}"#.to_vec(),
                    extra_headers: headers,
                },
            )
            .await
            .expect("api-key proxy succeeds");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"ok":true}"#);
        assert!(
            !response.headers.contains_key("connection"),
            "hop-by-hop response header leaked"
        );
        assert!(!response.headers.contains_key("x-upstream-hop"));

        let request = upstream_request.await.expect("fake provider request");
        assert!(
            request.starts_with("POST /v1/chat/completions "),
            "unexpected upstream request:\n{request}"
        );
        assert_header(&request, "authorization", "Bearer sk-provider");
        assert_header(&request, "openai-organization", "org_test");
        assert_header(&request, "openai-project", "proj_test");
        assert!(!request.contains("caller-token"));
        assert!(!request.contains("must-not-forward"));
        assert!(!request.contains("attacker.example"));
        assert!(!request.to_ascii_lowercase().contains("openai-beta:"));
        assert!(!request.to_ascii_lowercase().contains("x-openai-beta:"));
    }

    #[tokio::test]
    async fn openai_api_key_proxy_maps_retry_after_rate_limit() {
        let (base_url, _upstream_request) = one_shot_http_server(
            "HTTP/1.1 429 Too Many Requests\r\n\
             Retry-After: 17\r\n\
             Content-Length: 0\r\n\
             \r\n",
        )
        .await;
        let adapter =
            OpenAiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

        let err = adapter
            .proxy_chat_completions(
                "sk-provider",
                CodexProxyRequest {
                    body: br#"{"model":"gpt-test"}"#.to_vec(),
                    extra_headers: BTreeMap::new(),
                },
            )
            .await
            .expect_err("429 should map to rate-limit error");

        assert_eq!(
            err,
            CodexAdapterError::RateLimited {
                retry_after_secs: Some(17)
            }
        );
    }

    #[tokio::test]
    async fn openai_api_key_stream_forces_sse_accept_and_strips_hop_by_hop() {
        let (base_url, upstream_request) = one_shot_http_server(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/event-stream\r\n\
             Content-Length: 10\r\n\
             \r\n\
             data: hi\n\n",
        )
        .await;
        let adapter =
            OpenAiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

        let mut headers = BTreeMap::new();
        headers.insert("accept".to_string(), "application/json".to_string());
        headers.insert("connection".to_string(), "x-drop-me".to_string());
        headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());

        let (status, headers_out, stream) = adapter
            .proxy_chat_completions_stream(
                "sk-provider",
                CodexProxyRequest {
                    body: br#"{"model":"gpt-test","stream":true}"#.to_vec(),
                    extra_headers: headers,
                },
            )
            .await
            .expect("stream proxy opens");

        assert_eq!(status, 200);
        assert_eq!(
            headers_out.get("content-type").map(String::as_str),
            Some("text/event-stream")
        );
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
        assert_header(&request, "authorization", "Bearer sk-provider");
        assert_header(&request, "accept", "text/event-stream");
        assert!(!request.contains("application/json"));
        assert!(!request.contains("must-not-forward"));
    }

    #[test]
    fn codex_adapter_new_uses_default_base_url() {
        let client = Arc::new(reqwest::Client::builder().build().expect("client build"));
        let adapter = CodexAdapter::new(Arc::clone(&client));
        assert_eq!(adapter.base_url, CODEX_DEFAULT_BASE_URL);
        assert_eq!(adapter.cli_version, CLI_VERSION);
    }

    #[test]
    fn codex_adapter_with_base_url_overrides() {
        let client = Arc::new(reqwest::Client::new());
        let adapter = CodexAdapter::with_base_url(Arc::clone(&client), "http://localhost:9999");
        assert_eq!(adapter.base_url, "http://localhost:9999");
        assert_eq!(adapter.cli_version(), CLI_VERSION);
    }

    #[test]
    fn error_variants_are_eq() {
        let e1 = CodexAdapterError::RefreshFailed("boom".to_string());
        let e2 = CodexAdapterError::RefreshFailed("boom".to_string());
        assert_eq!(e1, e2);

        let e3 = CodexAdapterError::UpstreamError {
            status: 500,
            body: "internal".to_string(),
        };
        let e4 = CodexAdapterError::UpstreamError {
            status: 500,
            body: "internal".to_string(),
        };
        assert_eq!(e3, e4);

        let e5 = CodexAdapterError::RateLimited {
            retry_after_secs: Some(30),
        };
        let e6 = CodexAdapterError::RateLimited {
            retry_after_secs: Some(30),
        };
        assert_eq!(e5, e6);

        let e7 = CodexAdapterError::TransportError("conn refused".to_string());
        let e8 = CodexAdapterError::TransportError("conn refused".to_string());
        assert_eq!(e7, e8);
    }

    #[test]
    fn hop_by_hop_constants_cover_rfc7230_set() {
        let expected = [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "te",
            "trailers",
            "transfer-encoding",
            "upgrade",
        ];
        for h in &expected {
            assert!(
                HOP_BY_HOP.contains(h),
                "HOP_BY_HOP missing RFC 7230 header: {h}"
            );
        }
    }

    #[test]
    fn provider_control_header_strips_arbitrary_openai_headers() {
        // Arbitrary provider-control surface must be stripped.
        assert!(is_provider_control_header("openai-beta"));
        assert!(is_provider_control_header("x-openai-beta"));
        assert!(is_provider_control_header("x-openai-foo"));
        assert!(is_provider_control_header("openai-anything"));
        // Caller-safe allowlist is forwarded.
        assert!(!is_provider_control_header("openai-organization"));
        assert!(!is_provider_control_header("openai-project"));
        // Unrelated headers are untouched.
        assert!(!is_provider_control_header("content-type"));
    }

    #[test]
    fn connection_tokens_match_case_insensitively() {
        // Caller key uses capital `C` and the nominated token uses mixed case;
        // both must still be collected so the nominated header is stripped.
        let mut headers = BTreeMap::new();
        headers.insert("Connection".to_string(), "X-Leak, Keep-Alive".to_string());
        let tokens = connection_tokens(&headers);
        assert!(tokens.contains("x-leak"));
        assert!(tokens.contains("keep-alive"));
    }

    #[tokio::test]
    async fn openai_api_key_proxy_strips_arbitrary_provider_control_headers() {
        let (base_url, upstream_request) = one_shot_http_server(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Content-Length: 11\r\n\
             \r\n\
             {\"ok\":true}",
        )
        .await;
        let adapter =
            OpenAiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

        let mut headers = BTreeMap::new();
        // Mixed-case Connection nominating a mixed-case token must be stripped.
        headers.insert("Connection".to_string(), "X-Leak".to_string());
        headers.insert("X-Leak".to_string(), "must-not-forward".to_string());
        // Arbitrary provider-control headers must never reach upstream.
        headers.insert("x-openai-foo".to_string(), "evil".to_string());
        headers.insert("openai-beta".to_string(), "assistants=v2".to_string());
        headers.insert("x-openai-beta".to_string(), "codex-runs".to_string());
        // Caller-safe provider headers must be forwarded.
        headers.insert("openai-organization".to_string(), "org_test".to_string());
        headers.insert("openai-project".to_string(), "proj_test".to_string());

        adapter
            .proxy_chat_completions(
                "sk-provider",
                CodexProxyRequest {
                    body: br#"{"model":"gpt-test"}"#.to_vec(),
                    extra_headers: headers,
                },
            )
            .await
            .expect("api-key proxy succeeds");

        let request = upstream_request.await.expect("fake provider request");
        let lower = request.to_ascii_lowercase();
        assert!(
            !lower.contains("x-openai-foo:"),
            "arbitrary x-openai-* leaked"
        );
        assert!(!lower.contains("openai-beta:"), "openai-beta leaked");
        assert!(!lower.contains("x-openai-beta:"), "x-openai-beta leaked");
        assert!(
            !request.contains("must-not-forward"),
            "mixed-case Connection token leaked"
        );
        assert_header(&request, "openai-organization", "org_test");
        assert_header(&request, "openai-project", "proj_test");
    }

    #[test]
    fn codex_proxy_request_clone() {
        let mut hdrs = BTreeMap::new();
        hdrs.insert("content-type".to_string(), "application/json".to_string());
        let req = CodexProxyRequest {
            body: b"{}".to_vec(),
            extra_headers: hdrs,
        };
        let cloned = req.clone();
        assert_eq!(req.body, cloned.body);
    }
}
