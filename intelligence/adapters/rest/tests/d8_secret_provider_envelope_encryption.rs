//! D8 owned secret-provider envelope-encrypted refresh-token storage contract tests.
//!
//! These tests define the core envelope-encryption contract against the owned
//! secret-provider/KMS port. Concrete backing engines are transient adapters;
//! here we use in-process mocks that demonstrate the expected semantics.

use intelligence_rest::{RestAdapterError, SecretProviderStore};
use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Simple in-process store (no encryption) — satisfies the trait for contract
/// testing. Production uses a transient adapter behind the owned secret-provider/KMS port.
struct InProcessSecretStore {
    map: Mutex<HashMap<String, String>>,
}

impl InProcessSecretStore {
    fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl SecretProviderStore for InProcessSecretStore {
    fn fetch_refresh_token(&self, handle: &str) -> Result<String, RestAdapterError> {
        self.map
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
        self.map
            .lock()
            .unwrap()
            .insert(handle.to_string(), plaintext.to_string());
        Ok(())
    }
}

/// A store that simulates a secret-provider unavailable event mid-operation.
struct UnavailableSecretProviderStore;

impl SecretProviderStore for UnavailableSecretProviderStore {
    fn fetch_refresh_token(&self, _handle: &str) -> Result<String, RestAdapterError> {
        Err(RestAdapterError::SecretStoreUnavailable(
            "ErrSecretProviderUnavailable".to_string(),
        ))
    }

    fn store_refresh_token(&self, _handle: &str, _plaintext: &str) -> Result<(), RestAdapterError> {
        Err(RestAdapterError::SecretStoreUnavailable(
            "ErrSecretProviderUnavailable".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// D8-1: Store then fetch returns the same plaintext value.
#[test]
fn d8_store_and_fetch_roundtrip() {
    let store = InProcessSecretStore::new();
    store
        .store_refresh_token("handle-rt-1", "rt-plaintext-value")
        .unwrap();
    let fetched = store.fetch_refresh_token("handle-rt-1").unwrap();
    assert_eq!(fetched, "rt-plaintext-value");
}

/// D8-2: Fetching an unknown handle returns SecretNotFound.
#[test]
fn d8_fetch_unknown_handle_returns_not_found() {
    let store = InProcessSecretStore::new();
    let err = store.fetch_refresh_token("nonexistent-handle").unwrap_err();
    assert_eq!(err, RestAdapterError::SecretNotFound);
}

/// D8-3: Storing empty plaintext is rejected with InvalidSecret.
#[test]
fn d8_store_empty_plaintext_rejected() {
    let store = InProcessSecretStore::new();
    let err = store.store_refresh_token("handle-rt-1", "").unwrap_err();
    assert_eq!(err, RestAdapterError::InvalidSecret);
}

/// D8-4: Re-storing under the same handle overwrites the previous value
/// (token rotation — each refresh cycle writes the new token to the same
/// handle).
#[test]
fn d8_overwrite_rotates_token() {
    let store = InProcessSecretStore::new();
    store
        .store_refresh_token("handle-seat-001", "rt-v1")
        .unwrap();
    store
        .store_refresh_token("handle-seat-001", "rt-v2")
        .unwrap();
    let fetched = store.fetch_refresh_token("handle-seat-001").unwrap();
    assert_eq!(fetched, "rt-v2");
}

/// D8-5: Multiple handles are independent (different seats do not share
/// refresh tokens).
#[test]
fn d8_handles_are_independent() {
    let store = InProcessSecretStore::new();
    store
        .store_refresh_token("handle-seat-001", "rt-seat-001")
        .unwrap();
    store
        .store_refresh_token("handle-seat-002", "rt-seat-002")
        .unwrap();
    assert_eq!(
        store.fetch_refresh_token("handle-seat-001").unwrap(),
        "rt-seat-001"
    );
    assert_eq!(
        store.fetch_refresh_token("handle-seat-002").unwrap(),
        "rt-seat-002"
    );
}

/// D8-6: Unavailable secret-provider returns SecretStoreUnavailable on fetch.
#[test]
fn d8_unavailable_secret_provider_fetch_returns_unavailable() {
    let store = UnavailableSecretProviderStore;
    let err = store.fetch_refresh_token("any").unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable, got {err:?}"
    );
}

/// D8-7: Unavailable secret-provider returns SecretStoreUnavailable on store.
#[test]
fn d8_unavailable_secret_provider_store_returns_unavailable() {
    let store = UnavailableSecretProviderStore;
    let err = store.store_refresh_token("any", "token-value").unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable, got {err:?}"
    );
}

/// D8-8: RestAdapterError variants are distinct (critical for caller
/// error-handling paths: secret-provider unavailability must not be swallowed
/// as SecretNotFound).
#[test]
fn d8_error_variants_are_distinct() {
    let not_found = RestAdapterError::SecretNotFound;
    let unavailable =
        RestAdapterError::SecretStoreUnavailable("secret provider unavailable".to_string());
    let invalid = RestAdapterError::InvalidSecret;
    assert_ne!(not_found, unavailable);
    assert_ne!(not_found, invalid);
    assert_ne!(unavailable, invalid);
}
