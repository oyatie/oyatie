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
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub use oya_cloud_intelligence_kernel::{AgentId, Provider, SeatId, TenantId};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default ChatGPT base URL.
const CODEX_DEFAULT_BASE_URL: &str = "https://chatgpt.com";

/// CLI version impersonation header value (Stage-6 gap #11).
/// Hard-coded per ADR-0384 non_claims. Operator must update on Codex CLI bumps.
const CLI_VERSION: &str = "cli/0.27.0";

/// Path for the session token refresh endpoint.
const SESSION_PATH: &str = "/api/auth/session";

/// Path for the Codex completions data endpoint.
const CODEX_RESPONSES_PATH: &str = "/backend-api/codex/responses";

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

        let hop_by_hop: std::collections::HashSet<&str> = HOP_BY_HOP.iter().copied().collect();

        let connection_tokens: std::collections::HashSet<String> = request
            .extra_headers
            .get("connection")
            .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
            .unwrap_or_default();

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
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            return Err(CodexAdapterError::RateLimited { retry_after_secs });
        }

        // Filter hop-by-hop from response headers.
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
    ) -> Result<
        (
            u16,
            BTreeMap<String, String>,
            impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>,
        ),
        CodexAdapterError,
    > {
        let url = format!("{}{}", self.base_url, CODEX_RESPONSES_PATH);
        debug!(url = %url, "opening SSE stream to Codex data endpoint");

        let hop_by_hop: std::collections::HashSet<&str> = HOP_BY_HOP.iter().copied().collect();

        let connection_tokens: std::collections::HashSet<String> = request
            .extra_headers
            .get("connection")
            .map(|v| v.split(',').map(|t| t.trim().to_lowercase()).collect())
            .unwrap_or_default();

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
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            return Err(CodexAdapterError::RateLimited { retry_after_secs });
        }

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

        let byte_stream = resp.bytes_stream();
        Ok((status, headers, byte_stream))
    }
}

// ---------------------------------------------------------------------------
// Unit tests for value-object constructors and error variants
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
