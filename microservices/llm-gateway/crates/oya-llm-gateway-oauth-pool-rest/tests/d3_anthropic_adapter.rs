//! D3 Anthropic provider adapter contract tests (Stage-5 GREEN).
//!
//! Tests verify the OAuth refresh state machine: secret store round-trips,
//! error propagation, and that adapter methods return proper errors when no
//! real Anthropic server is reachable (no more todo!() panics).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_llm_gateway_oauth_pool_kernel::{SeatId, TenantId};
use oya_llm_gateway_oauth_pool_rest::{AnthropicAdapter, OpenBaoSecretStore, RestAdapterError};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct StubSecretStore {
    stored: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl StubSecretStore {
    fn new() -> Self {
        Self {
            stored: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn with_entry(handle: &str, token: &str) -> Self {
        let s = Self::new();
        s.stored
            .lock()
            .unwrap()
            .insert(handle.to_string(), token.to_string());
        s
    }
}

impl OpenBaoSecretStore for StubSecretStore {
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError> {
        self.stored
            .lock()
            .unwrap()
            .get(handle)
            .cloned()
            .ok_or(RestAdapterError::SecretNotFound)
    }

    fn store_refresh_token(&self, handle: &str, plaintext: &str) -> Result<(), RestAdapterError> {
        if plaintext.is_empty() {
            return Err(RestAdapterError::InvalidSecret);
        }
        self.stored
            .lock()
            .unwrap()
            .insert(handle.to_string(), plaintext.to_string());
        Ok(())
    }
}

struct FailingSecretStore;

impl OpenBaoSecretStore for FailingSecretStore {
    fn fetch_refresh_token(&self, _handle: &str) -> Result<String, RestAdapterError> {
        Err(RestAdapterError::SecretStoreUnavailable(
            "vault sealed".to_string(),
        ))
    }

    fn store_refresh_token(&self, _handle: &str, _plaintext: &str) -> Result<(), RestAdapterError> {
        Err(RestAdapterError::SecretStoreUnavailable(
            "vault sealed".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// D3-1: AnthropicAdapter can be constructed with a valid secret store.
#[test]
fn d3_adapter_constructs_with_secret_store() {
    let store = StubSecretStore::new();
    let _adapter = AnthropicAdapter::new(store);
    // Construction succeeds; no panic.
}

/// D3-2: refresh_token returns OAuthRefreshFailed when no Anthropic server is
/// reachable (Stage-5 GREEN: no longer panics, returns a real error).
#[test]
fn d3_refresh_token_returns_network_error_without_server() {
    let store = StubSecretStore::with_entry("handle-1", "rt-value");
    let adapter = AnthropicAdapter::with_base_url(store, "http://127.0.0.1:1".to_string());
    let result = adapter.refresh_token("handle-1");
    assert!(
        result.is_err(),
        "expected refresh_token to fail without a real server"
    );
    assert!(
        matches!(result.unwrap_err(), RestAdapterError::OAuthRefreshFailed(_)),
        "expected OAuthRefreshFailed"
    );
}

/// D3-3: exchange_authorization_code returns OAuthRefreshFailed when no
/// Anthropic server is reachable (Stage-5 GREEN: no longer panics).
#[test]
fn d3_exchange_authorization_code_returns_network_error_without_server() {
    let store = StubSecretStore::new();
    let adapter = AnthropicAdapter::with_base_url(store, "http://127.0.0.1:1".to_string());
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let result = adapter.exchange_authorization_code(&tenant, &seat, "auth-code");
    assert!(
        result.is_err(),
        "expected exchange_authorization_code to fail without a real server"
    );
    assert!(
        matches!(result.unwrap_err(), RestAdapterError::OAuthRefreshFailed(_)),
        "expected OAuthRefreshFailed"
    );
}

/// D3-4: StubSecretStore returns SecretNotFound for unknown handle.
#[test]
fn d3_secret_store_returns_not_found_for_unknown_handle() {
    let store = StubSecretStore::new();
    let err = store.fetch_refresh_token("unknown").unwrap_err();
    assert_eq!(err, RestAdapterError::SecretNotFound);
}

/// D3-5: StubSecretStore rejects empty plaintext.
#[test]
fn d3_secret_store_rejects_empty_plaintext() {
    let store = StubSecretStore::new();
    let err = store.store_refresh_token("handle-1", "").unwrap_err();
    assert_eq!(err, RestAdapterError::InvalidSecret);
}

/// D3-6: StubSecretStore round-trips a stored token.
#[test]
fn d3_secret_store_roundtrips_token() {
    let store = StubSecretStore::new();
    store
        .store_refresh_token("handle-rt", "my-refresh-token")
        .unwrap();
    let fetched = store.fetch_refresh_token("handle-rt").unwrap();
    assert_eq!(fetched, "my-refresh-token");
}

/// D3-7: FailingSecretStore returns SecretStoreUnavailable (simulates vault
/// sealed / network-partition scenario).
#[test]
fn d3_failing_secret_store_returns_unavailable() {
    let err = FailingSecretStore
        .fetch_refresh_token("any-handle")
        .unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable, got {err:?}"
    );
}

/// D3-8: RestAdapterError::PoolError variant wraps SubscriptionPoolError.
#[test]
fn d3_rest_adapter_error_wraps_pool_error() {
    use oya_llm_gateway_oauth_pool_kernel::SubscriptionPoolError;
    let e: RestAdapterError = SubscriptionPoolError::NoEligibleSeat.into();
    assert_eq!(
        e,
        RestAdapterError::PoolError(SubscriptionPoolError::NoEligibleSeat)
    );
}

/// D3-9: refresh_token propagates SecretNotFound from the secret store without
/// making any network call.
#[test]
fn d3_refresh_token_propagates_secret_not_found() {
    let store = StubSecretStore::new(); // empty — handle-missing will return SecretNotFound
    // Use real base URL — it should never be reached because the store lookup fails first.
    let adapter = AnthropicAdapter::new(store);
    let result = adapter.refresh_token("nonexistent-handle");
    assert_eq!(result.unwrap_err(), RestAdapterError::SecretNotFound);
}

/// D3-10: refresh_token propagates SecretStoreUnavailable from the secret store.
#[test]
fn d3_refresh_token_propagates_store_unavailable() {
    let adapter = AnthropicAdapter::new(FailingSecretStore);
    let result = adapter.refresh_token("any-handle");
    assert!(
        matches!(
            result.unwrap_err(),
            RestAdapterError::SecretStoreUnavailable(_)
        ),
        "expected SecretStoreUnavailable"
    );
}
