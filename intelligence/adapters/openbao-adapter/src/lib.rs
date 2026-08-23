//! Real OpenBao Transit adapter — envelope-encrypted refresh-token storage
//! seam (ADR-0384 D8, Stage-7 production seam).
//!
//! Implements [`SecretProviderStore`] from `intelligence-rest`
//! by calling OpenBao's Transit secrets engine directly over HTTP via a shared
//! [`reqwest::Client`]. No vault SDK dependency — the Transit REST API surface
//! is stable and identical to HashiCorp Vault Transit (the two projects share
//! the API shape; OpenBao is the CNCF fork).
//!
//! Encryption path (store_refresh_token):
//!   1. POST /v1/transit/encrypt/<key_name>  { "plaintext": base64(plaintext) }
//!   2. Extract ciphertext ("vault:v1:<base64>" format)
//!   3. POST /v1/secret/data/<key_path>/<handle>  { "data": { "ciphertext": "..." } }
//!
//! Decryption path (fetch_refresh_token):
//!   1. GET  /v1/secret/data/<key_path>/<handle>  → extract ciphertext
//!   2. POST /v1/transit/decrypt/<key_name>  { "ciphertext": "..." }
//!   3. base64-decode the returned plaintext
//!
//! ADR-0083 Tier-3 panic-free: no `unwrap`, no `expect`, no `panic!` in
//! production paths. Tests carry the cfg_attr exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

pub use intelligence_rest::{RestAdapterError, SecretProviderFuture, SecretProviderStore};

// ---------------------------------------------------------------------------
// Redacting vault-token wrapper (no `secrecy` dep required)
// ---------------------------------------------------------------------------

/// Wrapper that holds the raw vault token string but redacts it in all
/// `Debug` and `Display` output. Satisfies ADR-0083 secret-surfacing rule.
struct RedactedToken(String); // data_class: SECRET

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
// Error type
// ---------------------------------------------------------------------------

/// Errors specific to the OpenBao Transit adapter layer. Mapped to
/// [`RestAdapterError`] at the trait boundary.
#[derive(Debug)]
pub enum OpenBaoError {
    /// OpenBao returned HTTP 401 — vault token expired or revoked.
    TokenExpired,
    /// OpenBao returned HTTP 403 — permission denied.
    Forbidden,
    /// OpenBao returned HTTP 503 — vault sealed.
    VaultSealed,
    /// Unexpected HTTP status from OpenBao.
    UnexpectedStatus { status: u16, body: String },
    /// HTTP transport failure.
    Transport(String),
    /// JSON decode failure.
    Decode(String),
    /// base64 decode failure on plaintext returned from Transit.
    Base64Decode(String),
    /// Secret not found under KV path.
    SecretNotFound,
    /// The async secret-provider operation was polled outside a Tokio runtime.
    RuntimeUnavailable,
}

impl fmt::Display for OpenBaoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenBaoError::TokenExpired => f.write_str("vault token expired (HTTP 401)"),
            OpenBaoError::Forbidden => f.write_str("vault permission denied (HTTP 403)"),
            OpenBaoError::VaultSealed => f.write_str("vault sealed (HTTP 503)"),
            OpenBaoError::UnexpectedStatus { status, body } => {
                write!(f, "vault unexpected HTTP {status}: {body}")
            }
            OpenBaoError::Transport(msg) => write!(f, "vault transport error: {msg}"),
            OpenBaoError::Decode(msg) => write!(f, "vault JSON decode error: {msg}"),
            OpenBaoError::Base64Decode(msg) => write!(f, "transit plaintext base64 error: {msg}"),
            OpenBaoError::SecretNotFound => f.write_str("vault secret not found"),
            OpenBaoError::RuntimeUnavailable => {
                f.write_str("OpenBao secret-provider operation requires a Tokio runtime")
            }
        }
    }
}

impl From<OpenBaoError> for RestAdapterError {
    fn from(e: OpenBaoError) -> Self {
        match e {
            OpenBaoError::SecretNotFound => RestAdapterError::SecretNotFound,
            OpenBaoError::TokenExpired => RestAdapterError::SecretStoreUnavailable(
                "vault token expired (HTTP 401)".to_string(),
            ),
            OpenBaoError::Forbidden => RestAdapterError::SecretStoreUnavailable(
                "vault permission denied (HTTP 403)".to_string(),
            ),
            OpenBaoError::VaultSealed => {
                RestAdapterError::SecretStoreUnavailable("vault sealed (HTTP 503)".to_string())
            }
            other => RestAdapterError::SecretStoreUnavailable(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Wire shapes — OpenBao Transit encrypt/decrypt
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct TransitEncryptRequest<'a> {
    plaintext: &'a str, // data_class: SECRET (base64-encoded ciphertext input)
}

#[derive(Deserialize)]
struct TransitEncryptResponse {
    data: TransitEncryptData, // data_class: SECRET
}

#[derive(Deserialize)]
struct TransitEncryptData {
    ciphertext: String, // data_class: SECRET ("vault:v1:<base64>")
}

#[derive(Serialize)]
struct TransitDecryptRequest<'a> {
    ciphertext: &'a str, // data_class: SECRET
}

#[derive(Deserialize)]
struct TransitDecryptResponse {
    data: TransitDecryptData, // data_class: SECRET
}

#[derive(Deserialize)]
struct TransitDecryptData {
    plaintext: String, // data_class: SECRET (base64-encoded)
}

// ---------------------------------------------------------------------------
// Wire shapes — OpenBao KV-v2 secret store
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct KvWriteRequest<'a> {
    data: KvWriteData<'a>, // data_class: SECRET
}

#[derive(Serialize)]
struct KvWriteData<'a> {
    ciphertext: &'a str, // data_class: SECRET
}

#[derive(Deserialize)]
struct KvReadResponse {
    data: KvReadOuter, // data_class: SECRET
}

#[derive(Deserialize)]
struct KvReadOuter {
    data: KvReadData, // data_class: SECRET
}

#[derive(Deserialize)]
struct KvReadData {
    ciphertext: String, // data_class: SECRET
}

// ---------------------------------------------------------------------------
// OpenBaoTransitStore
// ---------------------------------------------------------------------------

/// Real OpenBao Transit adapter. Envelope-encrypts refresh tokens at rest via
/// OpenBao's Transit secrets engine and stores the ciphertext blobs in KV-v2.
///
/// The `vault_token` field is always redacted in `Debug` output; the raw string
/// is never surfaced in logs or error messages.
pub struct OpenBaoTransitStore {
    base_url: String,         // data_class: INTERNAL_ONLY
    transit_key_name: String, // data_class: INTERNAL_ONLY
    /// Sub-path under /v1/secret/data/ where per-handle blobs are stored.
    /// Defaults to the transit key name so handles are co-located under the
    /// same KV prefix (e.g. /v1/secret/data/intelligence-app-rt/<handle>).
    kv_key_path: String, // data_class: INTERNAL_ONLY
    http: Arc<reqwest::Client>, // data_class: INTERNAL_ONLY
    vault_token: RedactedToken, // data_class: SECRET
}

impl fmt::Debug for OpenBaoTransitStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenBaoTransitStore")
            .field("base_url", &self.base_url)
            .field("transit_key_name", &self.transit_key_name)
            .field("kv_key_path", &self.kv_key_path)
            .field("vault_token", &self.vault_token)
            .finish()
    }
}

impl OpenBaoTransitStore {
    /// Construct with the shared `reqwest::Client` from `AppState`, the OpenBao
    /// base URL, the Transit key name, and the vault token.
    ///
    /// `kv_key_path` defaults to `transit_key_name`; use
    /// [`OpenBaoTransitStore::with_kv_path`] to override.
    pub fn new(
        http: Arc<reqwest::Client>,
        base_url: impl Into<String>,
        transit_key_name: impl Into<String>,
        vault_token: impl Into<String>,
    ) -> Self {
        let key = transit_key_name.into();
        let kv_path = key.clone();
        Self {
            base_url: base_url.into(),
            transit_key_name: key,
            kv_key_path: kv_path,
            http,
            vault_token: RedactedToken(vault_token.into()),
        }
    }

    /// Override the KV-v2 sub-path used when storing/fetching ciphertext blobs.
    pub fn with_kv_path(mut self, kv_key_path: impl Into<String>) -> Self {
        self.kv_key_path = kv_key_path.into();
        self
    }

    // -----------------------------------------------------------------------
    // Internal: HTTP helpers
    // -----------------------------------------------------------------------

    /// Attach the Vault token header to a request builder.
    fn with_vault_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder.header("X-Vault-Token", &self.vault_token.0)
    }

    /// Interpret a non-success HTTP response from OpenBao into a typed error.
    fn map_error_status(status: u16, body: String) -> OpenBaoError {
        match status {
            401 => OpenBaoError::TokenExpired,
            403 => OpenBaoError::Forbidden,
            404 => OpenBaoError::SecretNotFound,
            503 => OpenBaoError::VaultSealed,
            _ => OpenBaoError::UnexpectedStatus { status, body },
        }
    }

    // -----------------------------------------------------------------------
    // Public: Transit encrypt / decrypt
    // -----------------------------------------------------------------------

    /// Envelope-encrypt `plaintext` via Transit and return the opaque ciphertext
    /// string (typically `vault:v1:<base64>`).
    ///
    /// POST /v1/transit/encrypt/<key_name>  { "plaintext": base64(plaintext) }
    pub async fn encrypt_envelope(&self, plaintext: &[u8]) -> Result<String, OpenBaoError> {
        let encoded = BASE64.encode(plaintext);
        let url = format!(
            "{}/v1/transit/encrypt/{}",
            self.base_url, self.transit_key_name
        );
        debug!(key = %self.transit_key_name, "transit encrypt");

        let req = TransitEncryptRequest {
            plaintext: &encoded,
        };
        let resp = self
            .with_vault_auth(self.http.post(&url))
            .json(&req)
            .send()
            .await
            .map_err(|e| OpenBaoError::Transport(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_error_status(status, body));
        }

        let body: TransitEncryptResponse = resp
            .json()
            .await
            .map_err(|e| OpenBaoError::Decode(e.to_string()))?;
        Ok(body.data.ciphertext)
    }

    /// Decrypt an opaque ciphertext string via Transit and return the raw
    /// plaintext bytes.
    ///
    /// POST /v1/transit/decrypt/<key_name>  { "ciphertext": "vault:v1:..." }
    pub async fn decrypt_envelope(&self, ciphertext: &str) -> Result<Vec<u8>, OpenBaoError> {
        let url = format!(
            "{}/v1/transit/decrypt/{}",
            self.base_url, self.transit_key_name
        );
        debug!(key = %self.transit_key_name, "transit decrypt");

        let req = TransitDecryptRequest { ciphertext };
        let resp = self
            .with_vault_auth(self.http.post(&url))
            .json(&req)
            .send()
            .await
            .map_err(|e| OpenBaoError::Transport(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_error_status(status, body));
        }

        let body: TransitDecryptResponse = resp
            .json()
            .await
            .map_err(|e| OpenBaoError::Decode(e.to_string()))?;

        BASE64
            .decode(&body.data.plaintext)
            .map_err(|e| OpenBaoError::Base64Decode(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // Internal: KV-v2 read / write
    // -----------------------------------------------------------------------

    /// Write `ciphertext` blob to KV-v2 under `<kv_key_path>/<handle>`.
    async fn kv_write(&self, handle: &str, ciphertext: &str) -> Result<(), OpenBaoError> {
        let url = format!(
            "{}/v1/secret/data/{}/{}",
            self.base_url, self.kv_key_path, handle
        );
        let req = KvWriteRequest {
            data: KvWriteData { ciphertext },
        };
        let resp = self
            .with_vault_auth(self.http.post(&url))
            .json(&req)
            .send()
            .await
            .map_err(|e| OpenBaoError::Transport(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_error_status(status, body));
        }
        Ok(())
    }

    /// Read the ciphertext blob from KV-v2 under `<kv_key_path>/<handle>`.
    async fn kv_read(&self, handle: &str) -> Result<String, OpenBaoError> {
        let url = format!(
            "{}/v1/secret/data/{}/{}",
            self.base_url, self.kv_key_path, handle
        );
        let resp = self
            .with_vault_auth(self.http.get(&url))
            .send()
            .await
            .map_err(|e| OpenBaoError::Transport(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_error_status(status, body));
        }

        let body: KvReadResponse = resp
            .json()
            .await
            .map_err(|e| OpenBaoError::Decode(e.to_string()))?;
        Ok(body.data.data.ciphertext)
    }

    // -----------------------------------------------------------------------
    // Async trait helpers (called from the boxed-future trait implementation)
    // -----------------------------------------------------------------------

    /// Full store path: encrypt plaintext → kv_write ciphertext.
    async fn store_async(&self, handle: &str, plaintext: &str) -> Result<(), OpenBaoError> {
        if plaintext.is_empty() {
            warn!(handle, "store_refresh_token called with empty plaintext");
            return Err(OpenBaoError::UnexpectedStatus {
                status: 0,
                body: "plaintext is empty".to_string(),
            });
        }
        let ciphertext = self.encrypt_envelope(plaintext.as_bytes()).await?;
        self.kv_write(handle, &ciphertext).await
    }

    /// Full fetch path: kv_read ciphertext → decrypt_envelope → utf8.
    async fn fetch_async(&self, handle: &str) -> Result<String, OpenBaoError> {
        let ciphertext = self.kv_read(handle).await?;
        let plaintext_bytes = self.decrypt_envelope(&ciphertext).await?;
        String::from_utf8(plaintext_bytes)
            .map_err(|e| OpenBaoError::Decode(format!("plaintext utf8 decode: {e}")))
    }

    /// Lightweight OpenBao health probe used by `/readyz`.
    async fn health_async(&self) -> Result<(), OpenBaoError> {
        let url = format!("{}/v1/sys/health", self.base_url);
        let resp = self
            .with_vault_auth(self.http.get(&url))
            .send()
            .await
            .map_err(|e| OpenBaoError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        if resp.status().is_success() {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Self::map_error_status(status, body))
    }
}

// ---------------------------------------------------------------------------
// SecretProviderStore trait impl
// ---------------------------------------------------------------------------

impl SecretProviderStore for OpenBaoTransitStore {
    /// Fetch and decrypt the refresh token identified by `handle`.
    ///
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async move {
            require_tokio_runtime()?;
            self.fetch_async(handle)
                .await
                .map_err(RestAdapterError::from)
        })
    }

    /// Envelope-encrypt `plaintext` and store it under `handle`.
    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async move {
            require_tokio_runtime()?;
            if plaintext.is_empty() {
                return Err(RestAdapterError::InvalidSecret);
            }
            self.store_async(handle, plaintext)
                .await
                .map_err(RestAdapterError::from)
        })
    }

    fn readiness_probe(&self) -> SecretProviderFuture<'_, ()> {
        Box::pin(async move {
            require_tokio_runtime()?;
            self.health_async().await.map_err(RestAdapterError::from)
        })
    }
}

fn require_tokio_runtime() -> Result<(), RestAdapterError> {
    tokio::runtime::Handle::try_current()
        .map(|_| ())
        .map_err(|_| RestAdapterError::from(OpenBaoError::RuntimeUnavailable))
}
