//! M02-P01 — OpenAI API-key pool adapter.
//!
//! Replaces the in-memory mock with a live pool: round-robin key selection,
//! failure-count blacklist, jittered cooldown, success-restore, and correct
//! `Authorization: Bearer` header injection.
//!
//! OpenAI API keys do NOT expire (distinct from Anthropic OAuth tokens).
//! Pool circuit-breaker lifted from gpt-load / one-api patterns.
//!
//! No raw key material is ever logged or exposed — SecretReference paths only
//! until the secret store resolves the material at call time.
//!
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod classifier;
pub mod key_pool;
pub mod key_status;
pub mod outbound_headers;

pub use classifier::{ResponseClass, classify_response};
pub use key_pool::KeyPool;
pub use key_status::KeyStatus;
pub use outbound_headers::openai_auth_headers;

use intelligence_account_domain::{SecretMaterial, SecretReference, SecretStorePort};
use intelligence_account_kernel::{
    AuthError, AuthMode, AuthToken, ProviderAuthPort, ProviderFamily,
};
use std::sync::Mutex;

// ── Wire types for OpenAI error response body parsing ────────────────────────
// data_class: INTERNAL_ONLY

#[derive(serde::Deserialize)]
struct OpenAiErrorBody {
    error: OpenAiErrorDetail,
}

#[derive(serde::Deserialize)]
struct OpenAiErrorDetail {
    #[serde(rename = "type")]
    error_type: Option<String>,
}

// ── OpenAiApiKeyPoolAdapter ───────────────────────────────────────────────────

/// Live API-key pool adapter for OpenAI.
///
/// Maintains a `KeyPool` of secret-reference paths. On each `authenticate` call:
/// 1. Selects the next eligible key via round-robin.
/// 2. Resolves the key bytes from the provided `SecretStorePort`.
/// 3. Returns an `AuthToken` whose `token_id_redacted` encodes the pool index.
///
/// Callers must call `record_call_result` after using the token to drive
/// circuit-breaker state transitions.
pub struct OpenAiApiKeyPoolAdapter<S: SecretStorePort> {
    pool: Mutex<KeyPool>,
    secret_store: S,
    clock_fn: Box<dyn Fn() -> u64 + Send + Sync>,
    /// Jitter supplier for cooldown; returns value in [0, jitter_max_secs).
    jitter_fn: Box<dyn Fn() -> u64 + Send + Sync>,
}

impl<S: SecretStorePort + Send> OpenAiApiKeyPoolAdapter<S> {
    /// Create with a real clock (seconds since UNIX epoch via `std::time`).
    pub fn new(pool: KeyPool, secret_store: S) -> Self {
        Self {
            pool: Mutex::new(pool),
            secret_store,
            clock_fn: Box::new(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            }),
            jitter_fn: Box::new(|| {
                // Simple low-cost jitter from the low bits of the current nanosecond clock.
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() as u64 % 30)
                    .unwrap_or(0)
            }),
        }
    }

    /// Override clock (for tests).
    pub fn with_clock(mut self, clock_fn: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        self.clock_fn = Box::new(clock_fn);
        self
    }

    /// Override jitter supplier (for tests — use deterministic value).
    pub fn with_jitter(mut self, jitter_fn: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        self.jitter_fn = Box::new(jitter_fn);
        self
    }

    /// Record the HTTP result of a call that used the token at `pool_index`.
    ///
    /// `http_status`: actual HTTP response status.
    /// `error_type_json`: raw JSON body bytes (used to extract `error.type` if present).
    pub fn record_call_result(
        &self,
        pool_index: usize,
        http_status: u16,
        error_type_json: Option<&[u8]>,
    ) {
        let error_type: Option<String> = error_type_json.and_then(|body| {
            serde_json::from_slice::<OpenAiErrorBody>(body)
                .ok()
                .and_then(|b| b.error.error_type)
        });
        let class = classify_response(http_status, error_type.as_deref());
        let now = (self.clock_fn)();
        let jitter = (self.jitter_fn)();
        self.pool
            .lock()
            .unwrap()
            .record_result(pool_index, class, now, jitter);
    }

    /// Returns the `Authorization: Bearer` header value for the key at `pool_index`.
    ///
    /// SECURITY: resolves raw key material; do not log the returned string.
    pub fn auth_headers_for(&self, pool_index: usize) -> Result<Vec<(String, String)>, AuthError> {
        let sref = {
            let guard = self.pool.lock().unwrap();
            let path = guard.key_sref(pool_index).to_owned();
            SecretReference::new(path).map_err(|_| AuthError::InvalidSecretReference)?
        };
        let material: SecretMaterial = self
            .secret_store
            .get(&sref)
            .map_err(|_| AuthError::InvalidSecretReference)?;
        let key_bytes = material.expose_for_provider_call();
        let key_str = std::str::from_utf8(key_bytes).map_err(|_| AuthError::InvalidToken)?;
        Ok(openai_auth_headers(key_str))
    }
}

impl<S: SecretStorePort + Send> ProviderAuthPort for OpenAiApiKeyPoolAdapter<S> {
    const PROVIDER_FAMILY: ProviderFamily = ProviderFamily::OpenAiOrCodex;
    const AUTH_MODE: AuthMode = AuthMode::Subscription;

    fn authenticate(&self, _sref: &SecretReference) -> Result<AuthToken, AuthError> {
        let now = (self.clock_fn)();
        let idx = self
            .pool
            .lock()
            .unwrap()
            .select(now)
            .ok_or(AuthError::NetworkUnavailable)?;

        // token_id encodes the pool index for the caller to use with record_call_result.
        // SECURITY: This does NOT contain any key material.
        let token_id = format!("openai-pool-key-{idx}");

        // API keys do not expire; set a 24h nominal lifetime for the AuthToken wrapper.
        AuthToken::new(now, now + 86_400, ProviderFamily::OpenAiOrCodex, token_id)
    }

    fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        // Parse the pool index from the token_id.
        let id = token.token_id_redacted();
        let idx: usize = id
            .strip_prefix("openai-pool-key-")
            .and_then(|s| s.parse().ok())
            .ok_or(AuthError::InvalidToken)?;
        let now = (self.clock_fn)();
        let jitter = (self.jitter_fn)();
        self.pool.lock().unwrap().record_result(
            idx,
            ResponseClass::TerminalKeyInvalid,
            now,
            jitter,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_account_domain::{
        ProviderFamily, SecretMaterial, SecretReference, SecretStoreError, SecretStorePort,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── In-memory secret store for tests ────────────────────────────────────

    struct FakeStore(Mutex<HashMap<SecretReference, Vec<u8>>>);

    impl FakeStore {
        /// `entries`: list of (sref_path_string, key_bytes_string)
        fn new(entries: Vec<(String, String)>) -> Self {
            let map: HashMap<_, _> = entries
                .into_iter()
                .map(|(k, v)| (SecretReference::new(k).unwrap(), v.into_bytes()))
                .collect();
            Self(Mutex::new(map))
        }
    }

    impl SecretStorePort for FakeStore {
        fn put(
            &mut self,
            sref: &SecretReference,
            material: SecretMaterial,
        ) -> Result<(), SecretStoreError> {
            self.0
                .lock()
                .unwrap()
                .insert(sref.clone(), material.expose_for_provider_call().to_vec());
            Ok(())
        }

        fn get(&self, sref: &SecretReference) -> Result<SecretMaterial, SecretStoreError> {
            self.0
                .lock()
                .unwrap()
                .get(sref)
                .map(|v| SecretMaterial::new(v.clone()))
                .ok_or(SecretStoreError::NotFound)
        }

        fn rotate(
            &mut self,
            sref: &SecretReference,
            new_material: SecretMaterial,
        ) -> Result<(), SecretStoreError> {
            self.0.lock().unwrap().insert(
                sref.clone(),
                new_material.expose_for_provider_call().to_vec(),
            );
            Ok(())
        }

        fn delete(&mut self, sref: &SecretReference) -> Result<(), SecretStoreError> {
            self.0.lock().unwrap().remove(sref);
            Ok(())
        }
    }

    fn make_adapter(keys: &[&str]) -> OpenAiApiKeyPoolAdapter<FakeStore> {
        let sref_paths: Vec<String> = keys.iter().map(|k| format!("sref://{k}")).collect();
        // Store maps sref_path -> key_value (using the key name as mock material)
        let store_entries: Vec<(String, String)> = sref_paths
            .iter()
            .zip(keys.iter())
            .map(|(p, k)| (p.clone(), (*k).to_owned()))
            .collect();
        let store = FakeStore::new(store_entries);
        let pool = KeyPool::new(sref_paths).with_jitter_max(0);
        OpenAiApiKeyPoolAdapter::new(pool, store)
            .with_clock(|| 1_000_000u64)
            .with_jitter(|| 0u64)
    }

    fn sref(s: &str) -> SecretReference {
        SecretReference::new(s.to_owned()).unwrap()
    }

    #[test]
    fn adapter_reports_correct_mode_and_family() {
        let a = make_adapter(&["sk-key0"]);
        assert_eq!(a.auth_mode(), AuthMode::Subscription);
        assert_eq!(a.provider_family(), ProviderFamily::OpenAiOrCodex);
        assert_eq!(
            <OpenAiApiKeyPoolAdapter<FakeStore> as ProviderAuthPort>::AUTH_MODE,
            AuthMode::Subscription
        );
    }

    #[test]
    fn authenticate_returns_token_with_pool_index() {
        let a = make_adapter(&["sk-key0"]);
        let token = a.authenticate(&sref("sref://unused")).unwrap();
        assert_eq!(token.provider_family(), ProviderFamily::OpenAiOrCodex);
        assert!(token.token_id_redacted().starts_with("openai-pool-key-"));
    }

    #[test]
    fn token_debug_is_redacted() {
        let a = make_adapter(&["sk-key0"]);
        let t = a.authenticate(&sref("sref://unused")).unwrap();
        let dbg = format!("{t:?}");
        assert!(dbg.contains("[REDACTED]"), "debug must redact: {dbg}");
    }

    #[test]
    fn authenticate_no_keys_returns_network_unavailable() {
        let store = FakeStore::new(vec![]);
        let pool = KeyPool::new(vec![]);
        let a = OpenAiApiKeyPoolAdapter::new(pool, store)
            .with_clock(|| 1_000_000u64)
            .with_jitter(|| 0u64);
        assert_eq!(
            a.authenticate(&sref("sref://x")),
            Err(AuthError::NetworkUnavailable)
        );
    }

    #[test]
    fn revoke_blacklists_key() {
        let a = make_adapter(&["sk-key0"]);
        let t = a.authenticate(&sref("sref://unused")).unwrap();
        a.revoke(&t).unwrap();
        assert_eq!(
            *a.pool.lock().unwrap().key_status(0),
            KeyStatus::Blacklisted
        );
        // Next authenticate should fail (pool exhausted)
        assert_eq!(
            a.authenticate(&sref("sref://x")),
            Err(AuthError::NetworkUnavailable)
        );
    }

    #[test]
    fn record_call_result_terminal_blacklists() {
        let a = make_adapter(&["sk-key0"]);
        a.record_call_result(0, 401, None);
        assert_eq!(
            *a.pool.lock().unwrap().key_status(0),
            KeyStatus::Blacklisted
        );
    }

    #[test]
    fn record_call_result_three_transients_enter_cooling() {
        let a = make_adapter(&["sk-key0"]);
        a.record_call_result(0, 429, None);
        a.record_call_result(0, 429, None);
        a.record_call_result(0, 429, None);
        assert!(matches!(
            a.pool.lock().unwrap().key_status(0),
            KeyStatus::Cooling { .. }
        ));
    }

    #[test]
    fn record_call_result_quota_exhausted_blacklists() {
        let a = make_adapter(&["sk-key0"]);
        let body = br#"{"error":{"type":"insufficient_quota","message":"Quota exceeded."}}"#;
        a.record_call_result(0, 429, Some(body));
        assert_eq!(
            *a.pool.lock().unwrap().key_status(0),
            KeyStatus::Blacklisted
        );
    }

    #[test]
    fn auth_headers_contain_bearer_not_x_api_key() {
        let a = make_adapter(&["sk-key0"]);
        let hdrs = a.auth_headers_for(0).unwrap();
        let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
        assert!(map["authorization"].starts_with("Bearer "));
        assert!(!map.contains_key("x-api-key"));
    }
}
