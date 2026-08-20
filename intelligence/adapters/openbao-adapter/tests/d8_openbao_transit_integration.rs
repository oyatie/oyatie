//! D8 OpenBao Transit integration tests against a local httpmock server.
//!
//! These tests exercise the real HTTP paths of [`OpenBaoTransitStore`]
//! (encrypt, decrypt, kv_write, kv_read) without requiring a live OpenBao
//! instance. Each test binds `httpmock::MockServer` on a random local port and
//! exercises exactly the scenario described in the test name.
//!
//! Covered scenarios:
//! 1. Encrypt/decrypt roundtrip (Transit mock → correct base64 flow)
//! 2. Token rotation (store new value, fetch returns new value)
//! 3. Vault sealed (HTTP 503 from any endpoint) → SecretStoreUnavailable
//! 4. Vault forbidden (HTTP 403) → SecretStoreUnavailable
//! 5. Vault token expired (HTTP 401) → SecretStoreUnavailable (terminal)
//! 6. Debug redaction (vault_token never appears in {:?} output)
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures::executor::block_on;
use httpmock::Mock;
use httpmock::prelude::*;
use intelligence_openbao_adapter::{OpenBaoTransitStore, RestAdapterError, SecretProviderStore};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const KEY: &str = "cloud-intelligence-rt";

fn make_store(server: &MockServer) -> OpenBaoTransitStore {
    let http = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap(),
    );
    OpenBaoTransitStore::new(http, server.base_url(), KEY, "s.test-vault-token")
}

/// Build the JSON body Transit returns for an encrypt call.
fn transit_encrypt_response(ciphertext: &str) -> String {
    format!(r#"{{"data":{{"ciphertext":"{ciphertext}"}}}}"#)
}

/// Build the JSON body Transit returns for a decrypt call (base64-encoded plaintext).
fn transit_decrypt_response(plaintext_b64: &str) -> String {
    format!(r#"{{"data":{{"plaintext":"{plaintext_b64}"}}}}"#)
}

/// Build the JSON body KV-v2 returns for a secret read.
fn kv_read_response(ciphertext: &str) -> String {
    format!(r#"{{"data":{{"data":{{"ciphertext":"{ciphertext}"}}}}}}"#)
}

// ---------------------------------------------------------------------------
// Test 1: Encrypt/decrypt roundtrip via Transit mock
// ---------------------------------------------------------------------------

/// Plaintext → encrypt mock → ciphertext → decrypt mock → original plaintext.
#[tokio::test(flavor = "multi_thread")]
async fn d8_transit_encrypt_decrypt_roundtrip() {
    let server = MockServer::start();

    let plaintext = b"my-refresh-token-secret-value";
    let plaintext_b64 = BASE64.encode(plaintext);
    let fake_ciphertext = "vault:v1:AHNKdGVzdA==";

    let _enc_mock = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/v1/transit/encrypt/{KEY}"))
            .header("X-Vault-Token", "s.test-vault-token")
            .body_contains(&plaintext_b64);
        then.status(200)
            .header("content-type", "application/json")
            .body(transit_encrypt_response(fake_ciphertext));
    });

    let _dec_mock = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/v1/transit/decrypt/{KEY}"))
            .header("X-Vault-Token", "s.test-vault-token")
            .body_contains(fake_ciphertext);
        then.status(200)
            .header("content-type", "application/json")
            .body(transit_decrypt_response(&plaintext_b64));
    });

    let store = make_store(&server);

    // Encrypt
    let ciphertext = store.encrypt_envelope(plaintext).await.unwrap();
    assert_eq!(ciphertext, fake_ciphertext);

    // Decrypt
    let recovered = store.decrypt_envelope(&ciphertext).await.unwrap();
    assert_eq!(recovered, plaintext);
}

// ---------------------------------------------------------------------------
// Test 2: Token rotation — store, fetch, store new, fetch returns new
// ---------------------------------------------------------------------------

/// Full store+fetch cycle with token rotation via the trait methods.
/// Each phase uses a fresh `MockServer` so mocks do not overlap across phases.
#[tokio::test(flavor = "multi_thread")]
async fn d8_token_rotation_store_fetch_store_fetch() {
    let handle = "tenant-a/seat-1";
    let rt_v1 = "refresh-token-v1";
    let rt_v2 = "refresh-token-v2";
    let cipher_v1 = "vault:v1:enc-v1";
    let cipher_v2 = "vault:v1:enc-v2";

    // --- Phase 1: Store v1 ---
    {
        let server = MockServer::start();
        let enc_mock = server.mock(|when, then| {
            when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
            then.status(200)
                .header("content-type", "application/json")
                .body(transit_encrypt_response(cipher_v1));
        });
        let kv_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v1/secret/data/{KEY}/{handle}"))
                .body_contains(cipher_v1);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"request_id":"r1"}"#);
        });
        let store = make_store(&server);
        store.store_refresh_token(handle, rt_v1).await.unwrap();
        enc_mock.assert_hits(1);
        kv_mock.assert_hits(1);
    }

    // --- Phase 2: Fetch v1 ---
    {
        let server = MockServer::start();
        let kv_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v1/secret/data/{KEY}/{handle}"));
            then.status(200)
                .header("content-type", "application/json")
                .body(kv_read_response(cipher_v1));
        });
        let dec_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v1/transit/decrypt/{KEY}"))
                .body_contains(cipher_v1);
            then.status(200)
                .header("content-type", "application/json")
                .body(transit_decrypt_response(&BASE64.encode(rt_v1.as_bytes())));
        });
        let store = make_store(&server);
        let fetched = store.fetch_refresh_token(handle).await.unwrap();
        assert_eq!(fetched, rt_v1);
        kv_mock.assert_hits(1);
        dec_mock.assert_hits(1);
    }

    // --- Phase 3: Store v2 (rotation) ---
    {
        let server = MockServer::start();
        let enc_mock = server.mock(|when, then| {
            when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
            then.status(200)
                .header("content-type", "application/json")
                .body(transit_encrypt_response(cipher_v2));
        });
        let kv_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v1/secret/data/{KEY}/{handle}"))
                .body_contains(cipher_v2);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"request_id":"r2"}"#);
        });
        let store = make_store(&server);
        store.store_refresh_token(handle, rt_v2).await.unwrap();
        enc_mock.assert_hits(1);
        kv_mock.assert_hits(1);
    }

    // --- Phase 4: Fetch v2 (after rotation) ---
    {
        let server = MockServer::start();
        let kv_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v1/secret/data/{KEY}/{handle}"));
            then.status(200)
                .header("content-type", "application/json")
                .body(kv_read_response(cipher_v2));
        });
        let dec_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v1/transit/decrypt/{KEY}"))
                .body_contains(cipher_v2);
            then.status(200)
                .header("content-type", "application/json")
                .body(transit_decrypt_response(&BASE64.encode(rt_v2.as_bytes())));
        });
        let store = make_store(&server);
        let fetched = store.fetch_refresh_token(handle).await.unwrap();
        assert_eq!(fetched, rt_v2, "fetch after rotation must return v2");
        kv_mock.assert_hits(1);
        dec_mock.assert_hits(1);
    }
}

// ---------------------------------------------------------------------------
// Test 3: Vault sealed (503) → SecretStoreUnavailable
// ---------------------------------------------------------------------------

/// Any 503 from the encrypt endpoint maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_sealed_503_store_returns_unavailable() {
    let server = MockServer::start();

    let _sealed = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
        then.status(503)
            .header("content-type", "application/json")
            .body(r#"{"errors":["ErrVaultSealed"]}"#);
    });

    let store = make_store(&server);
    let err = store
        .store_refresh_token("t/s", "some-token")
        .await
        .unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable for 503, got {err:?}"
    );
}

/// 503 on the KV read path also maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_sealed_503_fetch_returns_unavailable() {
    let server = MockServer::start();

    let _sealed = server.mock(|when, then| {
        when.method(GET).path(format!("/v1/secret/data/{KEY}/t/s"));
        then.status(503)
            .header("content-type", "application/json")
            .body(r#"{"errors":["ErrVaultSealed"]}"#);
    });

    let store = make_store(&server);
    let err = store.fetch_refresh_token("t/s").await.unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable for 503, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Vault forbidden (403) → SecretStoreUnavailable
// ---------------------------------------------------------------------------

/// 403 on the encrypt endpoint maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_forbidden_403_store_returns_unavailable() {
    let server = MockServer::start();

    let _forbidden = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
        then.status(403)
            .header("content-type", "application/json")
            .body(r#"{"errors":["1 error occurred: permission denied"]}"#);
    });

    let store = make_store(&server);
    let err = store
        .store_refresh_token("t/s", "some-token")
        .await
        .unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable for 403, got {err:?}"
    );
}

/// 403 on KV read maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_forbidden_403_fetch_returns_unavailable() {
    let server = MockServer::start();

    let _forbidden = server.mock(|when, then| {
        when.method(GET).path(format!("/v1/secret/data/{KEY}/t/s"));
        then.status(403)
            .header("content-type", "application/json")
            .body(r#"{"errors":["permission denied"]}"#);
    });

    let store = make_store(&server);
    let err = store.fetch_refresh_token("t/s").await.unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(_)),
        "expected SecretStoreUnavailable for 403, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Vault token expired (401) → SecretStoreUnavailable (terminal)
// ---------------------------------------------------------------------------

/// 401 on the encrypt endpoint is a terminal error — vault token must be
/// renewed out of band. Maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_token_expired_401_store_returns_unavailable() {
    let server = MockServer::start();

    let _expired = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"errors":["permission denied"]}"#);
    });

    let store = make_store(&server);
    let err = store.store_refresh_token("t/s", "tok").await.unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(ref msg) if msg.contains("401")),
        "expected SecretStoreUnavailable(contains '401') for 401, got {err:?}"
    );
}

/// 401 on KV read is also terminal.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_token_expired_401_fetch_returns_unavailable() {
    let server = MockServer::start();

    let _expired = server.mock(|when, then| {
        when.method(GET).path(format!("/v1/secret/data/{KEY}/t/s"));
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"errors":["permission denied"]}"#);
    });

    let store = make_store(&server);
    let err = store.fetch_refresh_token("t/s").await.unwrap_err();
    assert!(
        matches!(err, RestAdapterError::SecretStoreUnavailable(ref msg) if msg.contains("401")),
        "expected SecretStoreUnavailable(contains '401') for 401, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 6 (Bonus): vault_token never appears in Debug output
// ---------------------------------------------------------------------------

/// The vault token MUST NOT appear in any `{:?}` formatting of the store.
#[test]
fn d8_vault_token_redacted_in_debug_output() {
    let secret_token = "s.super-secret-vault-token-DO-NOT-LOG";
    let http = Arc::new(reqwest::Client::new());
    let store = OpenBaoTransitStore::new(http, "https://vault.example.com", KEY, secret_token);

    let debug_str = format!("{store:?}");
    assert!(
        !debug_str.contains(secret_token),
        "vault_token must not appear in Debug output; got: {debug_str}"
    );
    assert!(
        debug_str.contains("<REDACTED>"),
        "Debug output should contain '<REDACTED>' placeholder; got: {debug_str}"
    );
}

// ---------------------------------------------------------------------------
// Empty plaintext is rejected before any network I/O
// ---------------------------------------------------------------------------

/// Storing empty plaintext returns `InvalidSecret` without hitting the network.
#[tokio::test]
async fn d8_store_empty_plaintext_returns_invalid_secret() {
    let http = Arc::new(reqwest::Client::new());
    // Use a non-existent URL — the test must never reach the network.
    let store = OpenBaoTransitStore::new(http, "http://127.0.0.1:0", KEY, "s.tok");
    let err = store.store_refresh_token("h", "").await.unwrap_err();
    assert_eq!(err, RestAdapterError::InvalidSecret);
}

// ---------------------------------------------------------------------------
// Async runtime boundary
// ---------------------------------------------------------------------------

const RUNTIME_ERROR: &str = "OpenBao secret-provider operation requires a Tokio runtime";

fn runtime_boundary_store(server: &MockServer) -> OpenBaoTransitStore {
    OpenBaoTransitStore::new(
        Arc::new(reqwest::Client::new()),
        server.base_url(),
        KEY,
        "s.tok",
    )
}

fn assert_runtime_boundary_error<T>(result: Result<T, RestAdapterError>) {
    match result {
        Err(error) => assert_eq!(
            error,
            RestAdapterError::SecretStoreUnavailable(RUNTIME_ERROR.to_string())
        ),
        Ok(_) => panic!("async facade must return a runtime-boundary error"),
    }
}

fn runtime_boundary_mocks(server: &MockServer) -> (Mock<'_>, Mock<'_>, Mock<'_>) {
    let fetch = server.mock(|when, then| {
        when.method(GET).path(format!("/v1/secret/data/{KEY}/h"));
        then.status(200).body(kv_read_response("vault:v1:unused"));
    });
    let store = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
        then.status(200)
            .body(transit_encrypt_response("vault:v1:unused"));
    });
    let readiness = server.mock(|when, then| {
        when.method(GET).path("/v1/sys/health");
        then.status(200);
    });
    (fetch, store, readiness)
}

fn assert_no_runtime_boundary_io(mocks: &(Mock<'_>, Mock<'_>, Mock<'_>)) {
    mocks.0.assert_hits(0);
    mocks.1.assert_hits(0);
    mocks.2.assert_hits(0);
}

/// Constructing and dropping an async trait future outside Tokio is inert: it
/// neither starts an HTTP request nor touches the runtime.
#[test]
fn d8_async_facade_construction_and_drop_outside_tokio_are_inert() {
    let server = MockServer::start();
    let mocks = runtime_boundary_mocks(&server);
    let store = runtime_boundary_store(&server);

    drop(store.fetch_refresh_token("h"));
    drop(store.store_refresh_token("h", "token"));
    drop(store.readiness_probe());
    assert_no_runtime_boundary_io(&mocks);
}

/// Polling trait futures outside Tokio returns a typed error before I/O.
#[test]
fn d8_async_facade_outside_tokio_returns_typed_error_before_io() {
    let server = MockServer::start();
    let mocks = runtime_boundary_mocks(&server);
    let store = runtime_boundary_store(&server);

    assert_runtime_boundary_error(block_on(store.fetch_refresh_token("h")));
    assert_runtime_boundary_error(block_on(store.store_refresh_token("h", "token")));
    assert_runtime_boundary_error(block_on(store.readiness_probe()));
    assert_no_runtime_boundary_io(&mocks);
}

/// A current-thread runtime directly awaits the complete OpenBao HTTP flow.
#[tokio::test(flavor = "current_thread")]
async fn d8_async_facade_current_thread_localset_preserves_http_behavior() {
    let server = MockServer::start();
    let handle = "tenant-a/seat-1";
    let plaintext = "refresh-token";
    let ciphertext = "vault:v1:localset";
    let plaintext_b64 = BASE64.encode(plaintext);

    let store_encrypt = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/encrypt/{KEY}"));
        then.status(200)
            .header("content-type", "application/json")
            .body(transit_encrypt_response(ciphertext));
    });
    let store_write = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/v1/secret/data/{KEY}/{handle}"));
        then.status(200);
    });
    let fetch_read = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v1/secret/data/{KEY}/{handle}"));
        then.status(200)
            .header("content-type", "application/json")
            .body(kv_read_response(ciphertext));
    });
    let fetch_decrypt = server.mock(|when, then| {
        when.method(POST).path(format!("/v1/transit/decrypt/{KEY}"));
        then.status(200)
            .header("content-type", "application/json")
            .body(transit_decrypt_response(&plaintext_b64));
    });
    let readiness = server.mock(|when, then| {
        when.method(GET).path("/v1/sys/health");
        then.status(200);
    });
    let store = make_store(&server);
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            store.store_refresh_token(handle, plaintext).await.unwrap();
            assert_eq!(store.fetch_refresh_token(handle).await.unwrap(), plaintext);
            store.readiness_probe().await.unwrap();
        })
        .await;

    store_encrypt.assert_hits(1);
    store_write.assert_hits(1);
    fetch_read.assert_hits(1);
    fetch_decrypt.assert_hits(1);
    readiness.assert_hits(1);
}

/// A spawned request on a one-worker runtime completes its real readiness
/// request; no worker is synchronously held while OpenBao HTTP progresses.
#[test]
fn d8_async_facade_one_worker_spawned_handler_completes() {
    let server = MockServer::start();
    let readiness = server.mock(|when, then| {
        when.method(GET).path("/v1/sys/health");
        then.status(200);
    });
    let store = Arc::new(make_store(&server));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let task = tokio::spawn(async move { store.readiness_probe().await });
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(2), task)
            .await
            .expect("one-worker handler must not deadlock")
            .expect("spawned handler must not panic");
        outcome.expect("OpenBao readiness request must succeed");
    });

    readiness.assert_hits(1);
}
