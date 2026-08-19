//! Live OpenBao KV-v2 secret-fetch adapter.
//!
//! Implements [`CredentialHandleIssuerPort`] by calling
//! `GET /v1/secret/data/<seat-path>` directly over a bare hyper HTTP client.
//! No vault SDK dependency — the KV-v2 REST API is stable and identical to
//! HashiCorp Vault KV-v2 (OpenBao is the CNCF fork).
//!
//! Fetch path:
//!   1. Derive seat-path from `SecretReference::canonical_ref()` by stripping
//!      the `openbao://` prefix.
//!   2. `GET /v1/secret/data/<seat-path>` with `X-Vault-Token: <BAO_TOKEN>`.
//!   3. Parse JSON `{ "data": { "data": { "api_key" | "oauth_access_token": "..." } } }`.
//!   4. Validate credential is non-empty.
//!   5. Issue `CredentialHandle` (raw material never stored on the handle).
//!
//! ADR-0083 Tier-3 panic-free: no `unwrap`, no `expect`, no `panic!` in
//! production paths.  Tests carry the cfg_attr exemption inherited from lib.rs.
//!
//! Security properties:
//!   - `RedactedToken` wraps the raw vault token; `Debug`/`Display` prints `<REDACTED>`.
//!   - Credential material (api_key, oauth_access_token) never stored beyond
//!     the local call frame where it is validated for emptiness.
//!   - No raw material in `CredentialHandle`, `CredentialHandleIssueFailure`,
//!     or log output.

use std::fmt;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;
use tracing::{debug, warn};

pub use intelligence_credential_resolver_usecase::{
    CredentialHandleIssueFailure, CredentialHandleIssuerPort, CredentialHandleRequest,
};

use intelligence_credential_resolver_domain::{
    CredentialHandle, CredentialHandleIssueRequest, MAX_CREDENTIAL_HANDLE_TTL_SECONDS,
};

// ---------------------------------------------------------------------------
// Redacting vault-token wrapper (ADR-0083 secret-surfacing rule)
// ---------------------------------------------------------------------------

/// Holds the raw vault token string but redacts it in all `Debug` / `Display`
/// output so the token never leaks into logs or error messages.
pub struct RedactedToken(pub(crate) String); // data_class: SECRET

impl fmt::Debug for RedactedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<REDACTED>")
    }
}

impl fmt::Display for RedactedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<REDACTED>")
    }
}

// ---------------------------------------------------------------------------
// Config error
// ---------------------------------------------------------------------------

/// Errors that prevent a valid [`OpenBaoKvAdapterConfig`] from being built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenBaoKvConfigError {
    /// `BAO_TOKEN` environment variable is absent or empty.
    MissingBaoToken,
    /// `base_url` is empty or whitespace-only.
    EmptyBaseUrl,
}

impl fmt::Display for OpenBaoKvConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenBaoKvConfigError::MissingBaoToken => {
                f.write_str("BAO_TOKEN env var is absent or empty")
            }
            OpenBaoKvConfigError::EmptyBaseUrl => f.write_str("base_url is empty"),
        }
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for [`OpenBaoKvAdapter`].
///
/// Construct via [`OpenBaoKvAdapterConfig::from_env`] so the vault token is
/// sourced from the `BAO_TOKEN` environment variable at construction time only.
pub struct OpenBaoKvAdapterConfig {
    /// OpenBao base URL, e.g. `http://openbao.svc:8200`.
    pub base_url: String, // data_class: INTERNAL_ONLY
    /// Vault token — always redacted in Debug/Display output.
    pub(crate) vault_token: RedactedToken, // data_class: SECRET
}

impl fmt::Debug for OpenBaoKvAdapterConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenBaoKvAdapterConfig")
            .field("base_url", &self.base_url)
            .field("vault_token", &self.vault_token)
            .finish()
    }
}

impl OpenBaoKvAdapterConfig {
    /// Build config from `base_url` and the `BAO_TOKEN` environment variable.
    pub fn from_env(base_url: impl Into<String>) -> Result<Self, OpenBaoKvConfigError> {
        let base_url = base_url.into();
        if base_url.trim().is_empty() {
            return Err(OpenBaoKvConfigError::EmptyBaseUrl);
        }
        let token = std::env::var("BAO_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty())
            .ok_or(OpenBaoKvConfigError::MissingBaoToken)?;
        Ok(Self {
            base_url,
            vault_token: RedactedToken(token),
        })
    }

    /// Build config from explicit values (for testing).
    pub fn new(
        base_url: impl Into<String>,
        vault_token: impl Into<String>,
    ) -> Result<Self, OpenBaoKvConfigError> {
        let base_url = base_url.into();
        let vault_token = vault_token.into();
        if base_url.trim().is_empty() {
            return Err(OpenBaoKvConfigError::EmptyBaseUrl);
        }
        if vault_token.trim().is_empty() {
            return Err(OpenBaoKvConfigError::MissingBaoToken);
        }
        Ok(Self {
            base_url,
            vault_token: RedactedToken(vault_token),
        })
    }
}

// ---------------------------------------------------------------------------
// Wire shapes — OpenBao KV-v2 read response
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct KvReadResponse {
    data: KvReadOuter, // data_class: SECRET
}

#[derive(Deserialize)]
struct KvReadOuter {
    data: KvSecretData, // data_class: SECRET
}

#[derive(Deserialize)]
struct KvSecretData {
    api_key: Option<String>,            // data_class: SECRET
    oauth_access_token: Option<String>, // data_class: SECRET
}

impl KvSecretData {
    /// Extract the first non-empty credential field.
    /// The returned string is a raw secret value — callers MUST NOT log it.
    fn credential(&self) -> Option<&str> {
        self.api_key
            .as_deref()
            .filter(|v| !v.is_empty())
            .or_else(|| self.oauth_access_token.as_deref().filter(|v| !v.is_empty()))
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Internal errors from the OpenBao KV fetch layer.
/// Mapped to [`CredentialHandleIssueFailure`] at the port boundary —
/// raw material NEVER appears in any mapped value.
#[derive(Debug)]
enum OpenBaoKvError {
    TokenExpired,
    Forbidden,
    SecretNotFound,
    VaultSealed,
    UnexpectedStatus { status: u16 },
    Transport(String),
    Decode(String),
    EmptyCredential,
}

impl OpenBaoKvError {
    /// Sanitized, non-secret reason string safe for `CredentialHandleIssueFailure`.
    fn reason(&self) -> &'static str {
        match self {
            OpenBaoKvError::TokenExpired => "openbao:token-expired",
            OpenBaoKvError::Forbidden => "openbao:forbidden",
            OpenBaoKvError::SecretNotFound => "openbao:secret-not-found",
            OpenBaoKvError::VaultSealed => "openbao:vault-sealed",
            OpenBaoKvError::UnexpectedStatus { .. } => "openbao:unexpected-status",
            OpenBaoKvError::Transport(_) => "openbao:transport-error",
            OpenBaoKvError::Decode(_) => "openbao:decode-error",
            OpenBaoKvError::EmptyCredential => "openbao:empty-credential",
        }
    }
}

const EVIDENCE_REF: &str = "openbao:kv:fetch";

fn map_status_error(status: u16) -> OpenBaoKvError {
    match status {
        401 => OpenBaoKvError::TokenExpired,
        403 => OpenBaoKvError::Forbidden,
        404 => OpenBaoKvError::SecretNotFound,
        503 => OpenBaoKvError::VaultSealed,
        s => OpenBaoKvError::UnexpectedStatus { status: s },
    }
}

fn to_issue_failure(err: OpenBaoKvError) -> CredentialHandleIssueFailure {
    CredentialHandleIssueFailure {
        reason: err.reason().to_owned(),
        evidence_ref: EVIDENCE_REF.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

/// Live OpenBao KV-v2 adapter that implements [`CredentialHandleIssuerPort`].
///
/// Makes a single `GET /v1/secret/data/<seat-path>` per `issue_handle` call.
/// Uses a plain HTTP connector (TLS is handled by the sidecar / mTLS mesh in
/// production; the test mock uses plain HTTP on localhost).
pub struct OpenBaoKvAdapter {
    config: OpenBaoKvAdapterConfig, // data_class: INTERNAL_ONLY (token redacted in Debug)
    client: Client<HttpConnector, Full<Bytes>>, // data_class: INTERNAL_ONLY
}

impl fmt::Debug for OpenBaoKvAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenBaoKvAdapter")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl OpenBaoKvAdapter {
    /// Construct with an [`OpenBaoKvAdapterConfig`].
    pub fn new(config: OpenBaoKvAdapterConfig) -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();
        Self { config, client }
    }

    /// Derive the OpenBao KV seat-path from a canonical secret reference.
    ///
    /// `openbao://secret/ten_a/intelligence/provider/openai`
    /// → `secret/ten_a/intelligence/provider/openai`
    fn seat_path(canonical_ref: &str) -> &str {
        canonical_ref
            .strip_prefix("openbao://")
            .unwrap_or(canonical_ref)
    }

    /// Perform the HTTP GET and return the raw response body bytes.
    async fn kv_get(&self, seat_path: &str) -> Result<Bytes, OpenBaoKvError> {
        let url = format!("{}/v1/secret/data/{}", self.config.base_url, seat_path);
        debug!(seat_path, "openbao kv fetch");

        let req = Request::builder()
            .method("GET")
            .uri(&url)
            .header("X-Vault-Token", &self.config.vault_token.0)
            .header("Accept", "application/json")
            .body(Full::new(Bytes::new()))
            .map_err(|e| OpenBaoKvError::Transport(e.to_string()))?;

        let resp = self
            .client
            .request(req)
            .await
            .map_err(|e| OpenBaoKvError::Transport(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            warn!(status, seat_path, "openbao kv fetch non-success");
            return Err(map_status_error(status));
        }

        resp.into_body()
            .collect()
            .await
            .map(|c| c.to_bytes())
            .map_err(|e| OpenBaoKvError::Transport(e.to_string()))
    }

    /// Parse the KV-v2 JSON body and extract the credential field.
    /// Returns the raw credential string — callers MUST NOT log it.
    fn parse_credential(body: &[u8]) -> Result<String, OpenBaoKvError> {
        let parsed: KvReadResponse =
            serde_json::from_slice(body).map_err(|e| OpenBaoKvError::Decode(e.to_string()))?;
        let cred = parsed
            .data
            .data
            .credential()
            .ok_or(OpenBaoKvError::EmptyCredential)?;
        Ok(cred.to_owned())
    }

    /// Full fetch: GET + parse. Returns raw credential string.
    async fn fetch_credential(&self, seat_path: &str) -> Result<String, OpenBaoKvError> {
        let body = self.kv_get(seat_path).await?;
        Self::parse_credential(&body)
    }

    /// Async implementation of issue_handle; drives tokio::task::block_in_place.
    async fn issue_handle_async(
        &self,
        request: &CredentialHandleRequest,
    ) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
        let canonical = request.secret_reference.canonical_ref();
        let seat_path = Self::seat_path(canonical);

        // Fetch credential material — raw value used only to validate non-empty,
        // then discarded. It is never stored on the returned CredentialHandle.
        let _credential = self
            .fetch_credential(seat_path)
            .await
            .map_err(to_issue_failure)?;

        // Validate that the credential is non-empty (already done inside fetch_credential
        // via EmptyCredential, but belt-and-suspenders: _credential is non-empty here).

        let handle_id = format!("handle://openbao/{seat_path}/gen-1");
        let expires = request
            .now_epoch_seconds
            .saturating_add(MAX_CREDENTIAL_HANDLE_TTL_SECONDS);

        CredentialHandle::issue(CredentialHandleIssueRequest {
            handle_id,
            tenant_id: request.tenant_id.clone(),
            provider: request.provider,
            audience: request.audience,
            issued_at_epoch_seconds: request.now_epoch_seconds,
            expires_at_epoch_seconds: expires,
            generation: 1,
            sidecar_signature_ref: "openbao://kv/fetched".to_owned(),
        })
        .map_err(|e| CredentialHandleIssueFailure {
            reason: format!("openbao:handle-issue-error:{e:?}"),
            evidence_ref: EVIDENCE_REF.to_owned(),
        })
    }
}

impl CredentialHandleIssuerPort for OpenBaoKvAdapter {
    /// Issue a `CredentialHandle` by fetching live credential material from
    /// OpenBao KV-v2.
    ///
    /// Uses `tokio::task::block_in_place` so the sync trait method can call the
    /// async HTTP internals from within a multi-threaded Tokio runtime.
    fn issue_handle(
        &mut self,
        request: CredentialHandleRequest,
    ) -> Result<CredentialHandle, CredentialHandleIssueFailure> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.issue_handle_async(&request))
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};

    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;

    use intelligence_credential_resolver_domain::{
        CredentialAudience, CredentialProvider, SecretReference,
    };

    type CapturedRequest = (String, Option<String>);
    type CapturedRequestSlot = Arc<Mutex<Option<CapturedRequest>>>;

    // -----------------------------------------------------------------------
    // In-process mock server helpers
    // -----------------------------------------------------------------------

    /// Starts a one-shot hyper HTTP/1 server on 127.0.0.1:0.
    /// Returns the bound address and a task handle that resolves after the first
    /// request is served.
    async fn start_mock_server<F, Fut>(handler: F) -> SocketAddr
    where
        F: Fn(Request<hyper::body::Incoming>) -> Fut + Send + Clone + 'static,
        Fut: std::future::Future<Output = Response<Full<Bytes>>> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let handler = handler.clone();
                let svc = service_fn(move |req| {
                    let handler = handler.clone();
                    async move { Ok::<_, Infallible>(handler(req).await) }
                });
                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                        let _ = e; // test-only: ignore connection errors
                    }
                });
            }
        });
        addr
    }

    fn json_ok_api_key(key: &str) -> Response<Full<Bytes>> {
        let body = format!(r#"{{"data":{{"data":{{"api_key":"{key}"}}}}}}"#);
        Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    }

    fn json_ok_oauth_token(token: &str) -> Response<Full<Bytes>> {
        let body = format!(r#"{{"data":{{"data":{{"oauth_access_token":"{token}"}}}}}}"#);
        Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    }

    fn status_response(status: u16) -> Response<Full<Bytes>> {
        Response::builder()
            .status(status)
            .body(Full::new(Bytes::new()))
            .unwrap()
    }

    fn make_adapter(base_url: &str) -> OpenBaoKvAdapter {
        let config = OpenBaoKvAdapterConfig::new(base_url, "s.test-vault-token").unwrap();
        OpenBaoKvAdapter::new(config)
    }

    fn valid_request() -> CredentialHandleRequest {
        CredentialHandleRequest {
            tenant_id: "ten_a".to_owned(),
            provider: CredentialProvider::OpenAi,
            audience: CredentialAudience::ProviderDispatch,
            secret_reference: SecretReference::parse(
                "${openbao:secret/ten_a/intelligence/provider/openai}",
                "ten_a",
                CredentialProvider::OpenAi,
            )
            .unwrap(),
            request_evidence_ref: "req:openbao:1".to_owned(),
            now_epoch_seconds: 1_000,
        }
    }

    // -----------------------------------------------------------------------
    // Unit tests: JSON parse + error mapping
    // -----------------------------------------------------------------------

    #[test]
    fn parse_credential_extracts_api_key() {
        let body = br#"{"data":{"data":{"api_key":"sk-test-key"}}}"#;
        let cred = OpenBaoKvAdapter::parse_credential(body).unwrap();
        assert_eq!(cred, "sk-test-key");
    }

    #[test]
    fn parse_credential_extracts_oauth_access_token() {
        let body = br#"{"data":{"data":{"oauth_access_token":"ya29.test-token"}}}"#;
        let cred = OpenBaoKvAdapter::parse_credential(body).unwrap();
        assert_eq!(cred, "ya29.test-token");
    }

    #[test]
    fn parse_credential_prefers_api_key_when_both_present() {
        let body = br#"{"data":{"data":{"api_key":"sk-pref","oauth_access_token":"ya29.second"}}}"#;
        let cred = OpenBaoKvAdapter::parse_credential(body).unwrap();
        assert_eq!(cred, "sk-pref");
    }

    #[test]
    fn parse_credential_empty_fields_returns_empty_credential_error() {
        let body = br#"{"data":{"data":{}}}"#;
        let err = OpenBaoKvAdapter::parse_credential(body).unwrap_err();
        assert_eq!(err.reason(), "openbao:empty-credential");
    }

    #[test]
    fn parse_credential_malformed_json_returns_decode_error() {
        let body = b"not-json";
        let err = OpenBaoKvAdapter::parse_credential(body).unwrap_err();
        assert_eq!(err.reason(), "openbao:decode-error");
    }

    #[test]
    fn map_status_error_covers_all_sentinel_codes() {
        assert_eq!(map_status_error(401).reason(), "openbao:token-expired");
        assert_eq!(map_status_error(403).reason(), "openbao:forbidden");
        assert_eq!(map_status_error(404).reason(), "openbao:secret-not-found");
        assert_eq!(map_status_error(503).reason(), "openbao:vault-sealed");
        assert_eq!(map_status_error(500).reason(), "openbao:unexpected-status");
    }

    #[test]
    fn seat_path_strips_openbao_prefix() {
        assert_eq!(
            OpenBaoKvAdapter::seat_path("openbao://secret/ten_a/intelligence/provider/openai"),
            "secret/ten_a/intelligence/provider/openai"
        );
    }

    #[test]
    fn config_rejects_empty_base_url() {
        let err = OpenBaoKvAdapterConfig::new("", "s.token").unwrap_err();
        assert_eq!(err, OpenBaoKvConfigError::EmptyBaseUrl);
    }

    #[test]
    fn config_rejects_empty_vault_token() {
        let err = OpenBaoKvAdapterConfig::new("http://openbao:8200", "").unwrap_err();
        assert_eq!(err, OpenBaoKvConfigError::MissingBaoToken);
    }

    #[test]
    fn debug_output_redacts_vault_token() {
        let config =
            OpenBaoKvAdapterConfig::new("http://openbao:8200", "s.supersecret-token").unwrap();
        let debug = format!("{config:?}");
        assert!(!debug.contains("s.supersecret-token"));
        assert!(debug.contains("<REDACTED>"));
    }

    // -----------------------------------------------------------------------
    // Integration tests: in-process mock server
    // -----------------------------------------------------------------------

    #[tokio::test(flavor = "multi_thread")]
    async fn happy_path_api_key_issues_handle() {
        let addr = start_mock_server(|_req| async { json_ok_api_key("sk-live-key") }).await;
        let adapter = make_adapter(&format!("http://{addr}"));
        let mut adapter = adapter;

        let handle = adapter
            .issue_handle(valid_request())
            .expect("handle issued");

        assert_eq!(handle.bound_tenant(), "ten_a");
        assert_eq!(handle.bound_provider(), CredentialProvider::OpenAi);
        assert_eq!(handle.generation(), 1);
        assert!(handle.expires_at_epoch_seconds() > 1_000);
        // Verify raw key not in handle Debug
        let debug = format!("{handle:?}");
        assert!(!debug.contains("sk-live-key"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn happy_path_oauth_token_issues_handle() {
        let addr = start_mock_server(|_req| async { json_ok_oauth_token("ya29.live-oauth") }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let handle = adapter
            .issue_handle(valid_request())
            .expect("handle issued");

        assert_eq!(handle.bound_tenant(), "ten_a");
        let debug = format!("{handle:?}");
        assert!(!debug.contains("ya29.live-oauth"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_sends_correct_path_and_vault_token_header() {
        let captured: CapturedRequestSlot = Arc::new(Mutex::new(None));
        let captured_clone = captured.clone();

        let addr = start_mock_server(move |req| {
            let captured = captured_clone.clone();
            async move {
                let path = req.uri().path().to_owned();
                let token = req
                    .headers()
                    .get("x-vault-token")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_owned());
                *captured.lock().unwrap() = Some((path, token));
                json_ok_api_key("sk-test")
            }
        })
        .await;

        let mut adapter = make_adapter(&format!("http://{addr}"));
        adapter
            .issue_handle(valid_request())
            .expect("handle issued");

        let (path, token) = captured.lock().unwrap().take().unwrap();
        assert_eq!(
            path,
            "/v1/secret/data/secret/ten_a/intelligence/provider/openai"
        );
        assert_eq!(token.as_deref(), Some("s.test-vault-token"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_401_maps_to_token_expired_failure() {
        let addr = start_mock_server(|_req| async { status_response(401) }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("401 → failure");
        assert_eq!(failure.reason, "openbao:token-expired");
        assert_eq!(failure.evidence_ref, EVIDENCE_REF);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_403_maps_to_forbidden_failure() {
        let addr = start_mock_server(|_req| async { status_response(403) }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("403 → failure");
        assert_eq!(failure.reason, "openbao:forbidden");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_404_maps_to_secret_not_found_failure() {
        let addr = start_mock_server(|_req| async { status_response(404) }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("404 → failure");
        assert_eq!(failure.reason, "openbao:secret-not-found");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_503_maps_to_vault_sealed_failure() {
        let addr = start_mock_server(|_req| async { status_response(503) }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("503 → failure");
        assert_eq!(failure.reason, "openbao:vault-sealed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_500_maps_to_unexpected_status_failure() {
        let addr = start_mock_server(|_req| async { status_response(500) }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("500 → failure");
        assert_eq!(failure.reason, "openbao:unexpected-status");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn malformed_json_maps_to_decode_error() {
        let addr = start_mock_server(|_req| async {
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(b"not-json".as_slice())))
                .unwrap()
        })
        .await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let failure = adapter
            .issue_handle(valid_request())
            .expect_err("bad json → failure");
        assert_eq!(failure.reason, "openbao:decode-error");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn debug_output_never_contains_raw_secret_material() {
        let addr = start_mock_server(|_req| async { json_ok_api_key("sk-super-secret") }).await;
        let mut adapter = make_adapter(&format!("http://{addr}"));

        let handle = adapter.issue_handle(valid_request()).expect("handle");
        let debug = format!("{adapter:?}{handle:?}");

        assert!(!debug.contains("sk-super-secret"));
        assert!(!debug.contains("s.test-vault-token"));
    }
}
