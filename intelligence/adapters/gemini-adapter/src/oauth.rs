//! Gemini subscription-OAuth adapter — the "Antigravity" confidential-client
//! flow that pools Gemini subscription seats (NOT an API-key router).
//!
//! This module lives alongside [`crate::GeminiApiKeyAdapter`]; the API-key path
//! is untouched. Responsibilities, mirroring `AnthropicAdapter` / `CodexAdapter`:
//!
//!   1. [`GeminiOAuthAdapter::refresh_token`] — exchange a long-lived refresh
//!      token for a short-lived access token via the Google OAuth token
//!      endpoint, using a **confidential-client** `refresh_token` grant
//!      (`client_id` + `client_secret`, `application/x-www-form-urlencoded`).
//!      Access tokens are cached per handle with expiry-lead refresh, and
//!      concurrent refreshes for the same handle are coalesced (singleflight).
//!   2. [`GeminiOAuthAdapter::resolve_project`] — resolve the mandatory
//!      `cloudaicompanionProject` via `POST /v1internal:loadCodeAssist`
//!      (`metadata.ideType = ANTIGRAVITY`), falling back to
//!      `POST /v1internal:onboardUser` on the daily host. The project is cached
//!      per handle. Project travels in the request **body**, never a header.
//!   3. [`GeminiOAuthAdapter::proxy`] / [`GeminiOAuthAdapter::proxy_stream`] —
//!      forward a caller-supplied body to the Code Assist data endpoint with a
//!      `Bearer <access_token>` header, stripping hop-by-hop / credential
//!      headers exactly like the API-key path.
//!
//! Fail-closed: any refresh or project-resolution failure returns `Err` — the
//! adapter never serves a request without a valid bearer token and project.
//!
//! Secrets discipline: the `client_secret` and `refresh_token` are passed in by
//! the caller (resolved from OpenBao at the handler seam) and NEVER appear in
//! source. Only the public `client_id` is hard-coded.
//!
//! Every magic constant below is marked `// LIVE-RECONFIRM` — these are
//! reverse-engineered from the Antigravity / gemini-cli wire contract and must
//! be re-verified against a live session before production cutover.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;
use tokio::sync::{broadcast, Mutex};
use tracing::debug;

use crate::{
    connection_tokens, filtered_response_headers, hop_by_hop_set, retry_after_secs,
    GeminiByteStream, GeminiProxyRequest, GeminiProxyResponse,
};

// ---------------------------------------------------------------------------
// Constants — LIVE-RECONFIRM before production cutover.
// ---------------------------------------------------------------------------

/// Antigravity confidential-client OAuth client_id (public half of the pair).
/// LIVE-RECONFIRM
const ANTIGRAVITY_CLIENT_ID: &str =
    "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";

/// Google OAuth2 token endpoint (refresh grant). LIVE-RECONFIRM
const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Code Assist data-plane base. LIVE-RECONFIRM
const DATA_BASE_URL: &str = "https://cloudcode-pa.googleapis.com";

/// Daily Code Assist host used for the `onboardUser` project-resolution
/// fallback. LIVE-RECONFIRM
const ONBOARD_BASE_URL: &str = "https://daily-cloudcode-pa.googleapis.com";

/// OAuth scopes the Antigravity authorization grant is provisioned with. Not
/// sent on the `refresh_token` grant (refresh re-uses the originally granted
/// scopes); retained here for the authorization flow + operator audit.
/// LIVE-RECONFIRM
pub const ANTIGRAVITY_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",
    "https://www.googleapis.com/auth/experimentsandconfigs",
];

/// `metadata.ideType` value the Antigravity flow sends to `loadCodeAssist`.
/// LIVE-RECONFIRM
const IDE_TYPE_ANTIGRAVITY: &str = "ANTIGRAVITY";

/// `loadCodeAssist` method path (project resolution, project-in-body). LIVE-RECONFIRM
const LOAD_CODE_ASSIST_PATH: &str = "/v1internal:loadCodeAssist";

/// `onboardUser` method path (project-resolution fallback). LIVE-RECONFIRM
const ONBOARD_USER_PATH: &str = "/v1internal:onboardUser";

/// Non-streaming data-plane method path. LIVE-RECONFIRM
const GENERATE_CONTENT_PATH: &str = "/v1internal:generateContent";

/// Streaming (SSE) data-plane method path. LIVE-RECONFIRM
const STREAM_GENERATE_CONTENT_PATH: &str = "/v1internal:streamGenerateContent";

/// Refresh access tokens this many seconds *before* their stated expiry so an
/// in-flight request never races the expiry boundary.
const DEFAULT_REFRESH_LEAD_SECS: u64 = 60;

/// Fallback access-token lifetime when the token endpoint omits `expires_in`.
/// Google access tokens are nominally 3600s. LIVE-RECONFIRM
const DEFAULT_ACCESS_TOKEN_TTL_SECS: u64 = 3600;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified error for the Gemini OAuth adapter. Distinct from
/// `GeminiAdapterError` (API-key path) so callers can branch on auth-class
/// failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeminiOAuthError {
    /// Token endpoint rejected the refresh grant (expired/revoked refresh
    /// token, bad client_secret, etc.).
    RefreshFailed(String), // data_class: INTERNAL_ONLY
    /// Neither `loadCodeAssist` nor `onboardUser` yielded a
    /// `cloudaicompanionProject` — the seat cannot be served (fail-closed).
    ProjectResolutionFailed(String), // data_class: INTERNAL_ONLY
    /// Data endpoint returned a non-2xx status.
    UpstreamError {
        status: u16,  // data_class: INTERNAL_ONLY
        body: String, // data_class: INTERNAL_ONLY
    },
    /// HTTP transport-level error (connection refused, TLS failure, etc.).
    TransportError(String), // data_class: INTERNAL_ONLY
    /// Upstream responded 429; includes `Retry-After` seconds if present.
    RateLimited {
        retry_after_secs: Option<u64>, // data_class: INTERNAL_ONLY
    },
}

impl std::fmt::Display for GeminiOAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RefreshFailed(m) => write!(f, "OAuth refresh failed: {m}"),
            Self::ProjectResolutionFailed(m) => write!(f, "project resolution failed: {m}"),
            Self::UpstreamError { status, body } => {
                write!(f, "upstream error: HTTP {status}: {body}")
            }
            Self::TransportError(m) => write!(f, "transport error: {m}"),
            Self::RateLimited { retry_after_secs } => {
                write!(f, "rate limited (retry_after={retry_after_secs:?})")
            }
        }
    }
}

impl std::error::Error for GeminiOAuthError {}

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

/// Successful response from the Google OAuth token endpoint (refresh grant).
#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,    // data_class: INTERNAL_ONLY
    expires_in: Option<u64>, // data_class: INTERNAL_ONLY
}

/// A cached access token plus the unix-second wall-clock time it expires at.
#[derive(Clone, Debug)]
struct CachedAccessToken {
    access_token: String, // data_class: INTERNAL_ONLY
    expires_at: u64,      // data_class: INTERNAL_ONLY (unix seconds)
}

// ---------------------------------------------------------------------------
// Singleflight — coalesce concurrent refreshes for the same handle.
// ---------------------------------------------------------------------------

type RefreshResult = Result<String, String>; // data_class: INTERNAL_ONLY

/// Per-handle refresh coalescer: at most ONE upstream token exchange per handle
/// is in flight at a time; concurrent callers receive the leader's result via a
/// broadcast channel. Mirrors `rest::UpstreamOAuthSingleflight` (kept local to
/// avoid an adapter→rest layering inversion).
struct RefreshSingleflight {
    flights: Mutex<HashMap<String, broadcast::Sender<RefreshResult>>>, // data_class: INTERNAL_ONLY
}

impl RefreshSingleflight {
    fn new() -> Self {
        Self {
            flights: Mutex::new(HashMap::new()),
        }
    }

    async fn refresh_or_wait<F, Fut>(&self, handle: &str, do_refresh: F) -> RefreshResult
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = RefreshResult>,
    {
        let mut rx = {
            let mut map = self.flights.lock().await;
            if let Some(tx) = map.get(handle) {
                tx.subscribe()
            } else {
                let (tx, _rx) = broadcast::channel(64);
                map.insert(handle.to_string(), tx.clone());
                drop(map); // release lock before awaiting upstream
                let result = do_refresh().await;
                let _ = tx.send(result.clone());
                self.flights.lock().await.remove(handle);
                return result;
            }
        };
        rx.recv()
            .await
            .unwrap_or_else(|_| Err("singleflight: leader channel closed unexpectedly".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

/// Gemini subscription-OAuth (Antigravity) adapter.
///
/// Borrows a shared `Arc<reqwest::Client>` (TLS + keep-alive amortized across
/// the process). Holds per-handle access-token and project caches plus a
/// refresh singleflight, all behind `tokio::sync::Mutex` — cheap to `clone` and
/// share across the seat pool.
#[derive(Clone)]
pub struct GeminiOAuthAdapter {
    http: Arc<reqwest::Client>,          // data_class: INTERNAL_ONLY
    client_id: String,                   // data_class: INTERNAL_ONLY (public id)
    token_url: String,                   // data_class: INTERNAL_ONLY
    data_base_url: String,               // data_class: INTERNAL_ONLY
    onboard_base_url: String,            // data_class: INTERNAL_ONLY
    refresh_lead_secs: u64,              // data_class: INTERNAL_ONLY
    token_cache: Arc<Mutex<HashMap<String, CachedAccessToken>>>, // data_class: INTERNAL_ONLY
    project_cache: Arc<Mutex<HashMap<String, String>>>, // data_class: INTERNAL_ONLY
    singleflight: Arc<RefreshSingleflight>, // data_class: INTERNAL_ONLY
}

impl GeminiOAuthAdapter {
    /// Construct with the default production endpoints + Antigravity client_id.
    pub fn new(http: Arc<reqwest::Client>) -> Self {
        Self {
            http,
            client_id: ANTIGRAVITY_CLIENT_ID.to_string(),
            token_url: OAUTH_TOKEN_URL.to_string(),
            data_base_url: DATA_BASE_URL.to_string(),
            onboard_base_url: ONBOARD_BASE_URL.to_string(),
            refresh_lead_secs: DEFAULT_REFRESH_LEAD_SECS,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
            project_cache: Arc::new(Mutex::new(HashMap::new())),
            singleflight: Arc::new(RefreshSingleflight::new()),
        }
    }

    /// Construct with explicit endpoints — used by hermetic tests to point every
    /// upstream at a loopback fake. `data_base_url` and `onboard_base_url` may
    /// differ so the `onboardUser` fallback can be exercised independently.
    pub fn with_endpoints(
        http: Arc<reqwest::Client>,
        token_url: impl Into<String>,
        data_base_url: impl Into<String>,
        onboard_base_url: impl Into<String>,
    ) -> Self {
        Self {
            http,
            client_id: ANTIGRAVITY_CLIENT_ID.to_string(),
            token_url: token_url.into(),
            data_base_url: data_base_url.into(),
            onboard_base_url: onboard_base_url.into(),
            refresh_lead_secs: DEFAULT_REFRESH_LEAD_SECS,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
            project_cache: Arc::new(Mutex::new(HashMap::new())),
            singleflight: Arc::new(RefreshSingleflight::new()),
        }
    }

    /// Override the expiry-lead window (seconds). Mainly for tests.
    pub fn with_refresh_lead_secs(mut self, secs: u64) -> Self {
        self.refresh_lead_secs = secs;
        self
    }

    /// The public OAuth client_id this adapter authenticates with.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Resolve a valid access token for `handle`, refreshing via the
    /// confidential-client `refresh_token` grant if the cached token is absent
    /// or within the expiry-lead window.
    ///
    /// `handle` is an opaque cache key (`<tenant>/<seat>` by convention).
    /// `refresh_token` and `client_secret` are resolved from OpenBao by the
    /// caller and never persisted here.
    ///
    /// Concurrent callers for the same `handle` coalesce into a single upstream
    /// exchange. Fail-closed: a rejected grant returns
    /// [`GeminiOAuthError::RefreshFailed`].
    pub async fn refresh_token(
        &self,
        handle: &str,
        refresh_token: &str,
        client_secret: &str,
    ) -> Result<String, GeminiOAuthError> {
        // Fast path: serve a still-fresh cached token without taking the
        // singleflight lock.
        if let Some(tok) = self.cached_fresh_token(handle).await {
            return Ok(tok);
        }

        let handle_s = handle.to_string();
        let url = self.token_url.clone();
        let client_id = self.client_id.clone();
        let refresh = refresh_token.to_string();
        let secret = client_secret.to_string();
        let http = Arc::clone(&self.http);
        let cache = Arc::clone(&self.token_cache);
        let lead = self.refresh_lead_secs;

        let token = self
            .singleflight
            .refresh_or_wait(handle, move || async move {
                // Double-check: a concurrent leader may have just filled the
                // cache between our fast-path miss and acquiring leadership.
                {
                    let c = cache.lock().await;
                    if let Some(tok) = c.get(&handle_s)
                        && tok.expires_at > now_secs().saturating_add(lead)
                    {
                        return Ok(tok.access_token.clone());
                    }
                }

                // Confidential-client refresh grant, application/x-www-form-urlencoded.
                // The vendored reqwest is built without the `.form()` helper, so
                // we encode the body ourselves (see `form_urlencode`).
                let form_body = form_urlencode(&[
                    ("client_id", client_id.as_str()),
                    ("client_secret", secret.as_str()),
                    ("grant_type", "refresh_token"),
                    ("refresh_token", refresh.as_str()),
                ]);
                let resp = http
                    .post(&url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(form_body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                let status = resp.status().as_u16();
                if !resp.status().is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    return Err(format!("HTTP {status}: {body}"));
                }
                let parsed: OAuthTokenResponse = resp.json().await.map_err(|e| e.to_string())?;
                let expires_at = now_secs()
                    .saturating_add(parsed.expires_in.unwrap_or(DEFAULT_ACCESS_TOKEN_TTL_SECS));
                cache.lock().await.insert(
                    handle_s.clone(),
                    CachedAccessToken {
                        access_token: parsed.access_token.clone(),
                        expires_at,
                    },
                );
                Ok(parsed.access_token)
            })
            .await
            .map_err(GeminiOAuthError::RefreshFailed)?;

        Ok(token)
    }

    async fn cached_fresh_token(&self, handle: &str) -> Option<String> {
        let cache = self.token_cache.lock().await;
        let tok = cache.get(handle)?;
        if tok.expires_at > now_secs().saturating_add(self.refresh_lead_secs) {
            Some(tok.access_token.clone())
        } else {
            None
        }
    }

    /// Resolve the mandatory `cloudaicompanionProject` for `handle`.
    ///
    /// Tries `POST {data}/v1internal:loadCodeAssist` with
    /// `{ "metadata": { "ideType": "ANTIGRAVITY" } }`; on miss (non-2xx or no
    /// project in the response) falls back to
    /// `POST {onboard}/v1internal:onboardUser`. The resolved project id is
    /// cached per handle. Fail-closed: if neither yields a project the seat
    /// cannot be served.
    pub async fn resolve_project(
        &self,
        handle: &str,
        access_token: &str,
    ) -> Result<String, GeminiOAuthError> {
        if let Some(p) = self.project_cache.lock().await.get(handle) {
            return Ok(p.clone());
        }

        let body = json!({ "metadata": { "ideType": IDE_TYPE_ANTIGRAVITY } });

        let project = match self
            .post_internal(&self.data_base_url, LOAD_CODE_ASSIST_PATH, access_token, &body)
            .await
        {
            Ok(value) => extract_project(&value),
            Err(e) => {
                debug!(error = %e, "loadCodeAssist failed; attempting onboardUser fallback");
                None
            }
        };

        let project = match project {
            Some(p) => p,
            None => {
                let value = self
                    .post_internal(
                        &self.onboard_base_url,
                        ONBOARD_USER_PATH,
                        access_token,
                        &body,
                    )
                    .await
                    .map_err(|e| {
                        GeminiOAuthError::ProjectResolutionFailed(format!(
                            "onboardUser fallback failed: {e}"
                        ))
                    })?;
                extract_project(&value).ok_or_else(|| {
                    GeminiOAuthError::ProjectResolutionFailed(
                        "no cloudaicompanionProject in loadCodeAssist or onboardUser response"
                            .to_string(),
                    )
                })?
            }
        };

        self.project_cache
            .lock()
            .await
            .insert(handle.to_string(), project.clone());
        Ok(project)
    }

    /// Bearer-authenticated JSON POST to a `v1internal` control method, returning
    /// the parsed JSON body. Used by project resolution. Non-2xx → `Err`.
    async fn post_internal(
        &self,
        base: &str,
        path: &str,
        access_token: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, GeminiOAuthError> {
        let url = format!("{}{}", base.trim_end_matches('/'), path);
        debug!(url = %url, "POST Gemini v1internal control method");
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .json(body)
            .send()
            .await
            .map_err(|e| GeminiOAuthError::TransportError(e.to_string()))?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GeminiOAuthError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| GeminiOAuthError::TransportError(e.to_string()))?;
        if status >= 400 {
            return Err(GeminiOAuthError::UpstreamError {
                status,
                body: String::from_utf8_lossy(&bytes).to_string(),
            });
        }
        serde_json::from_slice(&bytes).map_err(|e| GeminiOAuthError::UpstreamError {
            status,
            body: format!("invalid JSON from {path}: {e}"),
        })
    }

    /// Forward `request` to the Code Assist `generateContent` data endpoint with
    /// the OAuth bearer token. Hop-by-hop / credential headers are stripped.
    pub async fn proxy(
        &self,
        access_token: &str,
        request: GeminiProxyRequest,
    ) -> Result<GeminiProxyResponse, GeminiOAuthError> {
        let url = format!("{}{}", self.data_base_url, GENERATE_CONTENT_PATH);
        debug!(url = %url, "proxying Gemini OAuth generateContent request");
        let resp = self.send(access_token, url, request, false).await?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GeminiOAuthError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        let body = resp
            .bytes()
            .await
            .map_err(|e| GeminiOAuthError::TransportError(e.to_string()))?
            .to_vec();
        if status >= 400 {
            return Err(GeminiOAuthError::UpstreamError {
                status,
                body: String::from_utf8_lossy(&body).to_string(),
            });
        }
        Ok(GeminiProxyResponse {
            status,
            headers,
            body,
        })
    }

    /// Forward `request` to the Code Assist `streamGenerateContent` data endpoint
    /// as an SSE stream. Returns `(status, headers, byte_stream)`; the caller
    /// frames SSE events from the raw bytes.
    pub async fn proxy_stream(
        &self,
        access_token: &str,
        request: GeminiProxyRequest,
    ) -> Result<(u16, std::collections::BTreeMap<String, String>, GeminiByteStream), GeminiOAuthError>
    {
        let url = format!(
            "{}{}?alt=sse",
            self.data_base_url, STREAM_GENERATE_CONTENT_PATH
        );
        debug!(url = %url, "opening Gemini OAuth streamGenerateContent SSE stream");
        let resp = self.send(access_token, url, request, true).await?;
        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GeminiOAuthError::RateLimited {
                retry_after_secs: retry_after_secs(&resp),
            });
        }
        let headers = filtered_response_headers(resp.headers());
        Ok((status, headers, Box::pin(resp.bytes_stream())))
    }

    /// Shared data-plane send: inject Bearer auth, forward caller headers minus
    /// credential / hop-by-hop / connection-nominated headers.
    async fn send(
        &self,
        access_token: &str,
        url: String,
        request: GeminiProxyRequest,
        stream: bool,
    ) -> Result<reqwest::Response, GeminiOAuthError> {
        let hop_by_hop = hop_by_hop_set();
        let connection_tokens = connection_tokens(&request.extra_headers);
        let mut req_builder = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .body(request.body);
        if stream {
            req_builder = req_builder.header("Accept", "text/event-stream");
        }

        for (k, v) in &request.extra_headers {
            let key_lower = k.to_ascii_lowercase();
            if matches!(
                key_lower.as_str(),
                "authorization"
                    | "host"
                    | "content-length"
                    | "user-agent"
                    | "x-goog-api-key"
                    | "x-google-api-key"
            ) {
                continue;
            }
            if stream && key_lower == "accept" {
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

        req_builder
            .send()
            .await
            .map_err(|e| GeminiOAuthError::TransportError(e.to_string()))
    }
}

/// Unix seconds (monotonic enough for token-expiry math; wall-clock skew is
/// covered by the expiry-lead window).
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Encode key/value pairs as `application/x-www-form-urlencoded`. Reserved per
/// RFC 3986 unreserved set (`A-Z a-z 0-9 - _ . ~`) pass through; space maps to
/// `+`; everything else is percent-encoded. Used for the OAuth refresh grant
/// because the vendored reqwest lacks `RequestBuilder::form`.
fn form_urlencode(pairs: &[(&str, &str)]) -> String {
    fn encode(s: &str, out: &mut String) {
        for &b in s.as_bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                b' ' => out.push('+'),
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
    }
    let mut out = String::new();
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push('&');
        }
        encode(k, &mut out);
        out.push('=');
        encode(v, &mut out);
    }
    out
}

/// Tolerantly extract a `cloudaicompanionProject` id from a `loadCodeAssist` /
/// `onboardUser` response. The field has been observed as a bare string
/// (`"projects/x"` or a project id) and as a nested object (`{ "id": ... }`);
/// `onboardUser` wraps it under `response`. Recurse so we tolerate either shape.
/// LIVE-RECONFIRM
fn extract_project(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(field) = map.get("cloudaicompanionProject")
                && let Some(p) = project_from_field(field)
            {
                return Some(p);
            }
            for v in map.values() {
                if let Some(p) = extract_project(v) {
                    return Some(p);
                }
            }
            None
        }
        serde_json::Value::Array(items) => items.iter().find_map(extract_project),
        _ => None,
    }
}

fn project_from_field(field: &serde_json::Value) -> Option<String> {
    match field {
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        serde_json::Value::Object(o) => o
            .get("id")
            .or_else(|| o.get("name"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_project_handles_bare_string() {
        let v = json!({ "cloudaicompanionProject": "projects/abc-123" });
        assert_eq!(extract_project(&v), Some("projects/abc-123".to_string()));
    }

    #[test]
    fn extract_project_handles_nested_object_id() {
        let v = json!({ "cloudaicompanionProject": { "id": "proj-xyz" } });
        assert_eq!(extract_project(&v), Some("proj-xyz".to_string()));
    }

    #[test]
    fn extract_project_handles_onboard_wrapper() {
        let v = json!({ "done": true, "response": { "cloudaicompanionProject": "p-9" } });
        assert_eq!(extract_project(&v), Some("p-9".to_string()));
    }

    #[test]
    fn extract_project_none_when_absent() {
        let v = json!({ "metadata": { "ideType": "ANTIGRAVITY" } });
        assert_eq!(extract_project(&v), None);
    }

    #[test]
    fn extract_project_empty_string_is_none() {
        let v = json!({ "cloudaicompanionProject": "" });
        assert_eq!(extract_project(&v), None);
    }

    #[test]
    fn error_display_is_stable() {
        let e = GeminiOAuthError::RateLimited {
            retry_after_secs: Some(5),
        };
        assert!(e.to_string().contains("rate limited"));
    }

    #[test]
    fn form_urlencode_escapes_reserved_chars() {
        let body = form_urlencode(&[
            ("grant_type", "refresh_token"),
            ("client_secret", "a/b+c=d e"),
        ]);
        assert_eq!(body, "grant_type=refresh_token&client_secret=a%2Fb%2Bc%3Dd+e");
    }

    #[test]
    fn client_id_is_the_public_antigravity_id() {
        let a = GeminiOAuthAdapter::new(Arc::new(reqwest::Client::new()));
        assert_eq!(a.client_id(), ANTIGRAVITY_CLIENT_ID);
    }
}
