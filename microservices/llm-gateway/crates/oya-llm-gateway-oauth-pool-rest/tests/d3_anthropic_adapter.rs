//! D3 Anthropic provider adapter contract tests (Stage-4 RED).
//!
//! These tests define the OAuth refresh state machine contract that Stage-5
//! GREEN must satisfy. All tests that exercise `todo!()` code FAIL at runtime
//! but MUST compile.

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

/// D3-2: refresh_token panics on stub (expected RED).
#[test]
#[should_panic(expected = "Stage-5 GREEN")]
fn d3_refresh_token_panics_on_stub() {
    let store = StubSecretStore::with_entry("handle-1", "rt-value");
    let adapter = AnthropicAdapter::new(store);
    let _ = adapter.refresh_token("handle-1");
}

/// D3-3: exchange_authorization_code panics on stub (expected RED).
#[test]
#[should_panic(expected = "Stage-5 GREEN")]
fn d3_exchange_authorization_code_panics_on_stub() {
    let store = StubSecretStore::new();
    let adapter = AnthropicAdapter::new(store);
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let _ = adapter.exchange_authorization_code(&tenant, &seat, "auth-code");
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
/// sealed / network-partition scenario that Stage-5 GREEN must handle).
#[test]
fn d3_failing_secret_store_returns_unavailable() {
    let err = FailingSecretStore
        .fetch_refresh_token("any-handle")
        .unwrap_err();
    matches!(err, RestAdapterError::SecretStoreUnavailable(_));
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
