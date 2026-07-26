//! D3 Anthropic provider adapter contract tests (Stage-6 GREEN).
//!
//! Tests verify the OAuth refresh state machine: secret store round-trips,
//! error propagation, and that adapter methods return proper errors when no
//! real Anthropic server is reachable (no more todo!() panics).
//!
//! Stage-6 changes: AnthropicAdapter methods are now async and take
//! `&reqwest::Client`. Tests use `#[tokio::test]` and a shared client.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_kernel::{SeatId, TenantId};
use intelligence_rest::{
    AnthropicAdapter, RestAdapterError, SecretProviderFuture, SecretProviderStore,
};

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

impl SecretProviderStore for StubSecretStore {
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async move {
            self.stored
                .lock()
                .unwrap()
                .get(handle)
                .cloned()
                .ok_or(RestAdapterError::SecretNotFound)
        })
    }

    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async move {
            if plaintext.is_empty() {
                return Err(RestAdapterError::InvalidSecret);
            }
            self.stored
                .lock()
                .unwrap()
                .insert(handle.to_string(), plaintext.to_string());
            Ok(())
        })
    }
}

struct FailingSecretStore;

impl SecretProviderStore for FailingSecretStore {
    fn fetch_refresh_token<'a>(&'a self, _handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async {
            Err(RestAdapterError::SecretStoreUnavailable(
                "secret provider unavailable".to_string(),
            ))
        })
    }

    fn store_refresh_token<'a>(
        &'a self,
        _handle: &'a str,
        _plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async {
            Err(RestAdapterError::SecretStoreUnavailable(
                "secret provider unavailable".to_string(),
            ))
        })
    }
}

fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
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
/// reachable (Stage-6 GREEN: async, returns a real error).
#[tokio::test]
async fn d3_refresh_token_returns_network_error_without_server() {
    let store = StubSecretStore::with_entry("handle-1", "rt-value");
    let adapter = AnthropicAdapter::with_base_url(store, "http://127.0.0.1:1".to_string());
    let client = make_client();
    let result = adapter.refresh_token(&client, "handle-1").await;
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
/// Anthropic server is reachable (Stage-6 GREEN: async).
#[tokio::test]
async fn d3_exchange_authorization_code_returns_network_error_without_server() {
    let store = StubSecretStore::new();
    let adapter = AnthropicAdapter::with_base_url(store, "http://127.0.0.1:1".to_string());
    let client = make_client();
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let result = adapter
        .exchange_authorization_code(&client, &tenant, &seat, "auth-code")
        .await;
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
#[tokio::test]
async fn d3_secret_store_returns_not_found_for_unknown_handle() {
    let store = StubSecretStore::new();
    let err = store.fetch_refresh_token("unknown").await.unwrap_err();
    assert_eq!(err, RestAdapterError::SecretNotFound);
}

/// D3-5: StubSecretStore rejects empty plaintext.
#[tokio::test]
async fn d3_secret_store_rejects_empty_plaintext() {
    let store = StubSecretStore::new();
    let err = store.store_refresh_token("handle-1", "").await.unwrap_err();
    assert_eq!(err, RestAdapterError::InvalidSecret);
}

/// D3-6: StubSecretStore round-trips a stored token.
#[tokio::test]
async fn d3_secret_store_roundtrips_token() {
    let store = StubSecretStore::new();
    store
        .store_refresh_token("handle-rt", "my-refresh-token")
        .await
        .unwrap();
    let fetched = store.fetch_refresh_token("handle-rt").await.unwrap();
    assert_eq!(fetched, "my-refresh-token");
}

/// D3-7: FailingSecretStore returns SecretStoreUnavailable (simulates vault
/// sealed / network-partition scenario).
#[tokio::test]
async fn d3_failing_secret_store_returns_unavailable() {
    let err = FailingSecretStore
        .fetch_refresh_token("any-handle")
        .await
        .unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable, got {err:?}"
    );
}

/// D3-8: RestAdapterError::PoolError variant wraps SubscriptionPoolError.
#[test]
fn d3_rest_adapter_error_wraps_pool_error() {
    use intelligence_kernel::SubscriptionPoolError;
    let e: RestAdapterError = SubscriptionPoolError::NoEligibleSeat.into();
    assert_eq!(
        e,
        RestAdapterError::PoolError(SubscriptionPoolError::NoEligibleSeat)
    );
}

/// D3-9: refresh_token propagates SecretNotFound from the secret store without
/// making any network call.
#[tokio::test]
async fn d3_refresh_token_propagates_secret_not_found() {
    let store = StubSecretStore::new(); // empty — handle-missing will return SecretNotFound
    // Use real base URL — it should never be reached because the store lookup fails first.
    let adapter = AnthropicAdapter::new(store);
    let client = make_client();
    let result = adapter.refresh_token(&client, "nonexistent-handle").await;
    assert_eq!(result.unwrap_err(), RestAdapterError::SecretNotFound);
}

/// D3-10: refresh_token propagates SecretStoreUnavailable from the secret store.
#[tokio::test]
async fn d3_refresh_token_propagates_store_unavailable() {
    let adapter = AnthropicAdapter::new(FailingSecretStore);
    let client = make_client();
    let result = adapter.refresh_token(&client, "any-handle").await;
    assert!(
        matches!(
            result.unwrap_err(),
            RestAdapterError::SecretStoreUnavailable(_)
        ),
        "expected SecretStoreUnavailable"
    );
}
