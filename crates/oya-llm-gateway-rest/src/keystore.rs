//! Key sourcing. **Secrets come from OpenBao, never from a plaintext file/env.**
//!
//! The gateway reads NO pooled provider key from a file or environment
//! variable. The only secret it reads from the environment is the OpenBao
//! token (`BAO_TOKEN`), which the platform injects from a k8s Secret at
//! deploy. Pooled keys are fetched from OpenBao KV v2 at startup and on a
//! periodic refresh.
//!
//! # Stores
//! - [`OpenBaoKeyStore`] — the default, production store. Reads KV v2 at
//!   `<mount>/data/<path>` from the `oya-kms` OpenBao, authenticating with the
//!   `BAO_TOKEN`. Encryption-at-rest is OpenBao's responsibility — the gateway
//!   embeds no KDF/AEAD and holds keys in memory only.
//! - [`InMemoryKeyStore`] — a single-node/test store seeded with keys already
//!   in hand. Keys live in process memory (NOT encrypted at rest locally);
//!   there is intentionally no local encrypt-at-rest path (OpenBao owns
//!   encryption-at-rest). It is fail-closed: a group with no seeded keys
//!   returns an explicit error rather than silently serving an empty pool.
//!
//! The [`KeyStore`] trait is the seam the runtime depends on; both impls
//! return [`KeyMaterial`] (a provider-tagged set of `(label, key)` pairs).

use std::collections::BTreeMap;

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use oya_llm_gateway_kernel::ProviderChannel;

/// A pooled set of keys for one provider channel: label → raw key.
///
/// SECURITY: this is the only type that holds live key bytes in memory. It is
/// never serialized to a log; only fingerprints derived from it
/// ([`crate::fingerprint_key`]) ever leave the process toward observability.
#[derive(Clone)]
pub struct KeyMaterial {
    channel: ProviderChannel,
    /// Insertion-ordered for stable round-robin: label → raw key.
    keys: Vec<(String, String)>,
}

/// Redacting `Debug`: never prints key bytes — only the channel and count.
/// This guarantees an accidental `{:?}` on a secret-bearing value cannot leak
/// keys into a log.
impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("channel", &self.channel)
            .field("keys", &format_args!("<{} redacted>", self.keys.len()))
            .finish()
    }
}

impl KeyMaterial {
    /// Build from a label→key map for `channel`. Order is taken from the
    /// sorted label order so reloads are deterministic.
    #[must_use]
    pub fn from_map(channel: ProviderChannel, map: BTreeMap<String, String>) -> Self {
        KeyMaterial {
            channel,
            keys: map.into_iter().collect(),
        }
    }

    /// The provider channel these keys serve.
    #[must_use]
    pub fn channel(&self) -> ProviderChannel {
        self.channel
    }

    /// Number of keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// `true` if no keys are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The raw keys in pool order. SECURITY: callers must only forward these
    /// to upstream auth headers, never log them.
    #[must_use]
    pub fn raw_keys(&self) -> Vec<&str> {
        self.keys.iter().map(|(_, k)| k.as_str()).collect()
    }

    /// Hash-only fingerprints in pool order (safe for logs/metrics).
    #[must_use]
    pub fn fingerprints(&self) -> Vec<String> {
        self.keys
            .iter()
            .map(|(_, k)| crate::fingerprint_key(k))
            .collect()
    }
}

/// Errors from key sourcing.
#[derive(Debug)]
pub enum KeyStoreError {
    /// The `BAO_TOKEN` environment variable was missing/empty.
    MissingBaoToken,
    /// The in-memory store was asked for a group it holds no keys for. This
    /// keeps the store fail-closed: it never silently returns an empty pool.
    MissingKeys(String),
    /// The OpenBao HTTP request failed (transport/status).
    Upstream(String),
    /// The OpenBao response JSON did not match the expected KV v2 shape.
    MalformedResponse(String),
}

impl std::fmt::Display for KeyStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyStoreError::MissingBaoToken => {
                write!(f, "BAO_TOKEN env var is required and must be non-empty")
            }
            KeyStoreError::MissingKeys(path) => write!(
                f,
                "in-memory key store holds no keys for path {path:?}; refusing to serve an empty pool"
            ),
            KeyStoreError::Upstream(m) => write!(f, "openbao request failed: {m}"),
            KeyStoreError::MalformedResponse(m) => write!(f, "openbao response malformed: {m}"),
        }
    }
}

impl std::error::Error for KeyStoreError {}

/// The async seam the runtime depends on for (re)loading pooled keys.
pub trait KeyStore: Send + Sync {
    /// Load (or reload) the keys for one group, identified by its OpenBao
    /// `path` and the `channel` it serves.
    fn load(
        &self,
        path: &str,
        channel: ProviderChannel,
    ) -> impl std::future::Future<Output = Result<KeyMaterial, KeyStoreError>> + Send;
}

/// Default store: OpenBao KV v2 over HTTP, token-authenticated.
///
/// Reads `GET <address>/v1/<mount>/data/<path>` with header
/// `X-Vault-Token: <BAO_TOKEN>` and extracts `.data.data` as a label→key map.
/// The token is captured once at construction from the environment and held in
/// memory; it is never logged.
pub struct OpenBaoKeyStore {
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>, Empty<Bytes>>,
    address: String,
    kv_mount: String,
    token: String,
}

/// Redacting `Debug`: never prints the OpenBao token.
impl std::fmt::Debug for OpenBaoKeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenBaoKeyStore")
            .field("address", &self.address)
            .field("kv_mount", &self.kv_mount)
            .field("token", &"<redacted>")
            .finish()
    }
}

impl OpenBaoKeyStore {
    /// Build a store for the OpenBao at `address` (e.g.
    /// `http://openbao.oya-kms.svc.cluster.local:8200`) with KV `kv_mount`,
    /// reading the token from the `BAO_TOKEN` env var. Fails closed if the
    /// token is absent or empty.
    pub fn from_env(address: impl Into<String>, kv_mount: impl Into<String>) -> Result<Self, KeyStoreError> {
        let token = std::env::var("BAO_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty())
            .ok_or(KeyStoreError::MissingBaoToken)?;
        Ok(Self::with_token(address, kv_mount, token))
    }

    /// Build a store with an explicit token (used by tests / alternative
    /// secret-injection paths). The token is still never read from a plaintext
    /// key file on disk.
    #[must_use]
    pub fn with_token(
        address: impl Into<String>,
        kv_mount: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        // Build the same hyper-util + rustls (ring) client the proxy uses, but
        // typed for empty-body GET requests. Plain-HTTP OpenBao addresses
        // (in-cluster service URLs) are allowed alongside HTTPS.
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .wrap_connector(http);
        let client = Client::builder(TokioExecutor::new()).build(https);
        OpenBaoKeyStore {
            client,
            address: trim_trailing_slash(address.into()),
            kv_mount: kv_mount.into(),
            token: token.into(),
        }
    }

    /// The KV v2 data URL for `path`.
    #[must_use]
    pub fn data_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/v1/{}/data/{}", self.address, self.kv_mount, path)
    }

    /// Parse a KV v2 read response body into a label→key map.
    ///
    /// Expected shape: `{ "data": { "data": { "<label>": "<key>", ... } } }`.
    /// Non-string values are skipped (KV v2 metadata-style entries are ignored
    /// rather than failing the whole load).
    pub fn parse_kv2_data(body: &str) -> Result<BTreeMap<String, String>, KeyStoreError> {
        let value: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| KeyStoreError::MalformedResponse(format!("not JSON: {e}")))?;
        let data = value
            .get("data")
            .and_then(|d| d.get("data"))
            .and_then(|d| d.as_object())
            .ok_or_else(|| {
                KeyStoreError::MalformedResponse("missing .data.data object".to_string())
            })?;
        let mut out = BTreeMap::new();
        for (label, v) in data {
            if let Some(key) = v.as_str().filter(|k| !k.is_empty()) {
                out.insert(label.clone(), key.to_string());
            }
        }
        Ok(out)
    }
}

impl KeyStore for OpenBaoKeyStore {
    async fn load(&self, path: &str, channel: ProviderChannel) -> Result<KeyMaterial, KeyStoreError> {
        let url = self.data_url(path);
        let request = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(&url)
            .header("X-Vault-Token", &self.token)
            .body(Empty::<Bytes>::new())
            .map_err(|e| KeyStoreError::Upstream(e.to_string()))?;
        let response = self
            .client
            .request(request)
            .await
            .map_err(|e| KeyStoreError::Upstream(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            // Do NOT include the body verbatim (it could echo secret material
            // on some backends); the status code is enough to act on.
            return Err(KeyStoreError::Upstream(format!("status {}", status.as_u16())));
        }
        // The KV read is a small config response (a label→key map), not the
        // proxy hot path, so collecting it fully is correct here.
        let collected = response
            .into_body()
            .collect()
            .await
            .map_err(|e| KeyStoreError::Upstream(e.to_string()))?;
        let bytes = collected.to_bytes();
        let body = std::str::from_utf8(&bytes)
            .map_err(|e| KeyStoreError::MalformedResponse(format!("non-UTF-8 body: {e}")))?;
        let map = Self::parse_kv2_data(body)?;
        Ok(KeyMaterial::from_map(channel, map))
    }
}

/// Single-node / test key store: keys are seeded already in hand and held in
/// process memory.
///
/// There is intentionally **no local encryption-at-rest** — OpenBao owns
/// crypto-at-rest, and the gateway holds keys in memory only (fetched per
/// refresh in production). This store exists for single-node/dev runs and for
/// tests that supply keys directly rather than reaching OpenBao.
///
/// It is **fail-closed**: [`KeyStore::load`] returns
/// [`KeyStoreError::MissingKeys`] for any path that was not seeded, so a
/// misconfiguration surfaces as an error instead of a silently empty pool.
#[derive(Clone, Default)]
pub struct InMemoryKeyStore {
    /// Seeded keys, keyed by the same OpenBao-style path the runtime requests.
    by_path: BTreeMap<String, KeyMaterial>,
}

/// Redacting `Debug`: never prints key bytes — only the seeded path set.
impl std::fmt::Debug for InMemoryKeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryKeyStore")
            .field("paths", &self.by_path.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl InMemoryKeyStore {
    /// An empty store. Loading any path fails closed until keys are seeded.
    #[must_use]
    pub fn new() -> Self {
        InMemoryKeyStore::default()
    }

    /// Seed (or replace) the keys served for `path`. The channel travels with
    /// the [`KeyMaterial`]; `load` returns it verbatim.
    #[must_use]
    pub fn with_keys(mut self, path: impl Into<String>, material: KeyMaterial) -> Self {
        self.by_path.insert(path.into(), material);
        self
    }
}

impl KeyStore for InMemoryKeyStore {
    async fn load(&self, path: &str, channel: ProviderChannel) -> Result<KeyMaterial, KeyStoreError> {
        match self.by_path.get(path) {
            Some(material) if material.channel() == channel => Ok(material.clone()),
            // A path seeded for a different channel is a misconfiguration; treat
            // it as missing rather than serving the wrong dialect's keys.
            _ => Err(KeyStoreError::MissingKeys(path.to_string())),
        }
    }
}

fn trim_trailing_slash(mut url: String) -> String {
    while url.ends_with('/') {
        url.pop();
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_material_orders_and_fingerprints() {
        let mut map = BTreeMap::new();
        map.insert("b".to_string(), "key-b".to_string());
        map.insert("a".to_string(), "key-a".to_string());
        let km = KeyMaterial::from_map(ProviderChannel::OpenAi, map);
        assert_eq!(km.len(), 2);
        // BTreeMap order => a then b.
        assert_eq!(km.raw_keys(), vec!["key-a", "key-b"]);
        let fps = km.fingerprints();
        assert_eq!(fps.len(), 2);
        assert!(fps.iter().all(|f| f.len() == 16));
    }

    #[test]
    fn openbao_data_url_is_kv2_shaped() {
        let store = OpenBaoKeyStore::with_token(
            "http://openbao.oya-kms.svc.cluster.local:8200/",
            "secret",
            "tok",
        );
        assert_eq!(
            store.data_url("agent-gateway/openai"),
            "http://openbao.oya-kms.svc.cluster.local:8200/v1/secret/data/agent-gateway/openai"
        );
        // Leading slash on path is tolerated.
        assert_eq!(
            store.data_url("/agent-gateway/anthropic"),
            "http://openbao.oya-kms.svc.cluster.local:8200/v1/secret/data/agent-gateway/anthropic"
        );
    }

    #[test]
    fn parse_kv2_extracts_string_values_only() {
        let body = r#"
        {
          "data": {
            "data": { "primary": "sk-aaa", "secondary": "sk-bbb", "rotated_at": 12345 },
            "metadata": { "version": 3 }
          }
        }
        "#;
        let map = OpenBaoKeyStore::parse_kv2_data(body).expect("parse");
        assert_eq!(map.get("primary").map(String::as_str), Some("sk-aaa"));
        assert_eq!(map.get("secondary").map(String::as_str), Some("sk-bbb"));
        // Non-string value skipped.
        assert!(!map.contains_key("rotated_at"));
    }

    #[test]
    fn parse_kv2_rejects_wrong_shape() {
        let err = OpenBaoKeyStore::parse_kv2_data(r#"{"data":{"nope":1}}"#).expect_err("bad shape");
        assert!(matches!(err, KeyStoreError::MalformedResponse(_)));
    }

    #[tokio::test]
    async fn in_memory_store_returns_seeded_keys() {
        let mut map = BTreeMap::new();
        map.insert("primary".to_string(), "sk-mem-aaa".to_string());
        let store = InMemoryKeyStore::new().with_keys(
            "agent-gateway/openai",
            KeyMaterial::from_map(ProviderChannel::OpenAi, map),
        );
        let km = store
            .load("agent-gateway/openai", ProviderChannel::OpenAi)
            .await
            .expect("seeded path loads");
        assert_eq!(km.raw_keys(), vec!["sk-mem-aaa"]);
    }

    #[tokio::test]
    async fn in_memory_store_fails_closed_on_unseeded_path() {
        let store = InMemoryKeyStore::new();
        let result = store.load("agent-gateway/openai", ProviderChannel::OpenAi).await;
        assert!(matches!(result.err(), Some(KeyStoreError::MissingKeys(_))));
    }

    #[tokio::test]
    async fn in_memory_store_rejects_channel_mismatch() {
        let mut map = BTreeMap::new();
        map.insert("primary".to_string(), "sk-mem-aaa".to_string());
        let store = InMemoryKeyStore::new().with_keys(
            "agent-gateway/openai",
            KeyMaterial::from_map(ProviderChannel::OpenAi, map),
        );
        // Seeded for OpenAi but requested as Anthropic → fail closed.
        let result = store
            .load("agent-gateway/openai", ProviderChannel::Anthropic)
            .await;
        assert!(matches!(result.err(), Some(KeyStoreError::MissingKeys(_))));
    }
}
