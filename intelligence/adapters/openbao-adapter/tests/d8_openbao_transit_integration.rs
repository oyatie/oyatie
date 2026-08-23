//! D8 OpenBao Transit integration tests against a local scripted HTTP server.
//!
//! These tests exercise the real HTTP paths of [`OpenBaoTransitStore`]
//! (encrypt, decrypt, kv_write, kv_read) without requiring a live OpenBao
//! instance. Each test binds a `ScriptedServer` on a random local port and
//! exercises exactly the scenario described in the test name.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). OpenBao's surface is a set of distinct ENDPOINTS rather than a fixed call
//! sequence, so this file ports onto content routing (`vault_server` below) rather than
//! onto a positional script — the routing table is exactly the set of mocks it replaces.
//! Two assertions get stronger: `assert_hits(N)` becomes `hits(&server, method, path)`,
//! and the runtime-boundary tests' triple `assert_hits(0)` becomes an assertion that the
//! server received NO request at all, which also catches I/O to an endpoint nobody
//! thought to mock.
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
use intelligence_openbao_adapter::{OpenBaoTransitStore, RestAdapterError, SecretProviderStore};
use scripted_http_server::{ScriptedResponse, ScriptedServer};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const KEY: &str = "intelligence-app-rt";

/// One OpenBao endpoint: the method and path that reach it, and what it answers.
type VaultRoute = (&'static str, String, ScriptedResponse);

/// A server that answers by ENDPOINT, which is how OpenBao is actually shaped.
/// An unrouted request is answered 404 and still recorded, so a call to an endpoint
/// nobody scripted shows up as a failed assertion rather than as silence.
fn vault_server(routes: Vec<VaultRoute>) -> ScriptedServer {
    ScriptedServer::start_with(move |request| {
        routes
            .iter()
            .find(|(method, path, _)| *method == request.method && *path == request.path())
            .map(|(_, _, response)| response.clone())
            .unwrap_or_else(|| {
                ScriptedResponse::status(404).text(format!(
                    "no OpenBao route for {} {}",
                    request.method,
                    request.path()
                ))
            })
    })
}

/// The port of `mock.assert_hits(n)`: how many requests actually reached an endpoint.
fn hits(server: &ScriptedServer, method: &str, path: &str) -> usize {
    server
        .requests()
        .iter()
        .filter(|request| request.method == method && request.path() == path)
        .count()
}

fn transit_encrypt_path() -> String {
    format!("/v1/transit/encrypt/{KEY}")
}

fn transit_decrypt_path() -> String {
    format!("/v1/transit/decrypt/{KEY}")
}

fn kv_path(handle: &str) -> String {
    format!("/v1/secret/data/{KEY}/{handle}")
}

fn json_response(status: u16, body: String) -> ScriptedResponse {
    ScriptedResponse::status(status)
        .header("content-type", "application/json")
        .body(body)
}

fn make_store(server: &ScriptedServer) -> OpenBaoTransitStore {
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
    let plaintext = b"my-refresh-token-secret-value";
    let plaintext_b64 = BASE64.encode(plaintext);
    let fake_ciphertext = "vault:v1:AHNKdGVzdA==";

    let server = vault_server(vec![
        (
            "POST",
            transit_encrypt_path(),
            json_response(200, transit_encrypt_response(fake_ciphertext)),
        ),
        (
            "POST",
            transit_decrypt_path(),
            json_response(200, transit_decrypt_response(&plaintext_b64)),
        ),
    ]);

    let store = make_store(&server);

    // Encrypt
    let ciphertext = store.encrypt_envelope(plaintext).await.unwrap();
    assert_eq!(ciphertext, fake_ciphertext);

    // Decrypt
    let recovered = store.decrypt_envelope(&ciphertext).await.unwrap();
    assert_eq!(recovered, plaintext);

    // The `header(..)` / `body_contains(..)` MATCHERS become assertions on what was
    // actually sent. A matcher only selects a mock; an assertion fails when it is wrong.
    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec![
            format!("POST {}", transit_encrypt_path()),
            format!("POST {}", transit_decrypt_path()),
        ]
    );
    for request in &requests {
        assert_eq!(
            request.header("x-vault-token"),
            Some("s.test-vault-token"),
            "every OpenBao call must carry the vault token"
        );
    }
    assert!(
        requests[0].body_string().contains(&plaintext_b64),
        "encrypt must send the base64 plaintext: {}",
        requests[0].body_string()
    );
    assert!(
        requests[1].body_string().contains(fake_ciphertext),
        "decrypt must send back the ciphertext encrypt returned: {}",
        requests[1].body_string()
    );
}

// ---------------------------------------------------------------------------
// Test 2: Token rotation — store, fetch, store new, fetch returns new
// ---------------------------------------------------------------------------

/// Full store+fetch cycle with token rotation via the trait methods.
/// Each phase uses a fresh `ScriptedServer` so routes do not overlap across phases.
#[tokio::test(flavor = "multi_thread")]
async fn d8_token_rotation_store_fetch_store_fetch() {
    let handle = "tenant-a/seat-1";
    let rt_v1 = "refresh-token-v1";
    let rt_v2 = "refresh-token-v2";
    let cipher_v1 = "vault:v1:enc-v1";
    let cipher_v2 = "vault:v1:enc-v2";

    // --- Phase 1: Store v1 ---
    {
        let server = vault_server(vec![
            (
                "POST",
                transit_encrypt_path(),
                json_response(200, transit_encrypt_response(cipher_v1)),
            ),
            (
                "POST",
                kv_path(handle),
                json_response(200, r#"{"request_id":"r1"}"#.to_string()),
            ),
        ]);
        let store = make_store(&server);
        store.store_refresh_token(handle, rt_v1).await.unwrap();
        assert_eq!(hits(&server, "POST", &transit_encrypt_path()), 1);
        assert_eq!(hits(&server, "POST", &kv_path(handle)), 1);
        // Was a `body_contains(cipher_v1)` matcher on the KV write.
        let kv_write = &server.requests()[1];
        assert!(
            kv_write.body_string().contains(cipher_v1),
            "KV write must carry the ciphertext encrypt produced: {}",
            kv_write.body_string()
        );
        assert!(
            !kv_write.body_string().contains(rt_v1),
            "the PLAINTEXT refresh token must never reach the KV store: {}",
            kv_write.body_string()
        );
    }

    // --- Phase 2: Fetch v1 ---
    {
        let server = vault_server(vec![
            (
                "GET",
                kv_path(handle),
                json_response(200, kv_read_response(cipher_v1)),
            ),
            (
                "POST",
                transit_decrypt_path(),
                json_response(
                    200,
                    transit_decrypt_response(&BASE64.encode(rt_v1.as_bytes())),
                ),
            ),
        ]);
        let store = make_store(&server);
        let fetched = store.fetch_refresh_token(handle).await.unwrap();
        assert_eq!(fetched, rt_v1);
        assert_eq!(hits(&server, "GET", &kv_path(handle)), 1);
        assert_eq!(hits(&server, "POST", &transit_decrypt_path()), 1);
        assert!(
            server.requests()[1].body_string().contains(cipher_v1),
            "decrypt must be handed the ciphertext the KV read returned"
        );
    }

    // --- Phase 3: Store v2 (rotation) ---
    {
        let server = vault_server(vec![
            (
                "POST",
                transit_encrypt_path(),
                json_response(200, transit_encrypt_response(cipher_v2)),
            ),
            (
                "POST",
                kv_path(handle),
                json_response(200, r#"{"request_id":"r2"}"#.to_string()),
            ),
        ]);
        let store = make_store(&server);
        store.store_refresh_token(handle, rt_v2).await.unwrap();
        assert_eq!(hits(&server, "POST", &transit_encrypt_path()), 1);
        assert_eq!(hits(&server, "POST", &kv_path(handle)), 1);
        let kv_write = &server.requests()[1];
        assert!(
            kv_write.body_string().contains(cipher_v2),
            "the rotation must write the NEW ciphertext: {}",
            kv_write.body_string()
        );
        assert!(
            !kv_write.body_string().contains(rt_v2),
            "the PLAINTEXT refresh token must never reach the KV store: {}",
            kv_write.body_string()
        );
    }

    // --- Phase 4: Fetch v2 (after rotation) ---
    {
        let server = vault_server(vec![
            (
                "GET",
                kv_path(handle),
                json_response(200, kv_read_response(cipher_v2)),
            ),
            (
                "POST",
                transit_decrypt_path(),
                json_response(
                    200,
                    transit_decrypt_response(&BASE64.encode(rt_v2.as_bytes())),
                ),
            ),
        ]);
        let store = make_store(&server);
        let fetched = store.fetch_refresh_token(handle).await.unwrap();
        assert_eq!(fetched, rt_v2, "fetch after rotation must return v2");
        assert_eq!(hits(&server, "GET", &kv_path(handle)), 1);
        assert_eq!(hits(&server, "POST", &transit_decrypt_path()), 1);
        assert!(
            server.requests()[1].body_string().contains(cipher_v2),
            "decrypt must be handed the ROTATED ciphertext, not the old one"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 3: Vault sealed (503) → SecretStoreUnavailable
// ---------------------------------------------------------------------------

/// Any 503 from the encrypt endpoint maps to `SecretStoreUnavailable`.
#[tokio::test(flavor = "multi_thread")]
async fn d8_vault_sealed_503_store_returns_unavailable() {
    let server = vault_server(vec![(
        "POST",
        transit_encrypt_path(),
        json_response(503, r#"{"errors":["ErrVaultSealed"]}"#.to_string()),
    )]);

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
    let server = vault_server(vec![(
        "GET",
        kv_path("t/s"),
        json_response(503, r#"{"errors":["ErrVaultSealed"]}"#.to_string()),
    )]);

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
    let server = vault_server(vec![(
        "POST",
        transit_encrypt_path(),
        json_response(
            403,
            r#"{"errors":["1 error occurred: permission denied"]}"#.to_string(),
        ),
    )]);

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
    let server = vault_server(vec![(
        "GET",
        kv_path("t/s"),
        json_response(403, r#"{"errors":["permission denied"]}"#.to_string()),
    )]);

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
    let server = vault_server(vec![(
        "POST",
        transit_encrypt_path(),
        json_response(401, r#"{"errors":["permission denied"]}"#.to_string()),
    )]);

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
    let server = vault_server(vec![(
        "GET",
        kv_path("t/s"),
        json_response(401, r#"{"errors":["permission denied"]}"#.to_string()),
    )]);

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

fn runtime_boundary_store(server: &ScriptedServer) -> OpenBaoTransitStore {
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

/// Every endpoint the runtime-boundary futures COULD reach, wired to succeed — so that
/// if one of them is reached, the test fails on the traffic rather than on an error.
fn runtime_boundary_server() -> ScriptedServer {
    vault_server(vec![
        (
            "GET",
            kv_path("h"),
            json_response(200, kv_read_response("vault:v1:unused")),
        ),
        (
            "POST",
            transit_encrypt_path(),
            json_response(200, transit_encrypt_response("vault:v1:unused")),
        ),
        ("GET", "/v1/sys/health".to_string(), ScriptedResponse::ok()),
    ])
}

/// Stronger than the three `assert_hits(0)` calls it replaces: those only proved the
/// three KNOWN mocks went unhit, while this proves the server saw no request at all —
/// so I/O to an endpoint nobody thought to mock is caught too.
fn assert_no_runtime_boundary_io(server: &ScriptedServer) {
    assert_eq!(
        server.request_count(),
        0,
        "the runtime boundary must be crossed before any I/O; server saw: {:?}",
        server.request_lines()
    );
}

/// Constructing and dropping an async trait future outside Tokio is inert: it
/// neither starts an HTTP request nor touches the runtime.
#[test]
fn d8_async_facade_construction_and_drop_outside_tokio_are_inert() {
    let server = runtime_boundary_server();
    let store = runtime_boundary_store(&server);

    drop(store.fetch_refresh_token("h"));
    drop(store.store_refresh_token("h", "token"));
    drop(store.readiness_probe());
    assert_no_runtime_boundary_io(&server);
}

/// Polling trait futures outside Tokio returns a typed error before I/O.
#[test]
fn d8_async_facade_outside_tokio_returns_typed_error_before_io() {
    let server = runtime_boundary_server();
    let store = runtime_boundary_store(&server);

    assert_runtime_boundary_error(block_on(store.fetch_refresh_token("h")));
    assert_runtime_boundary_error(block_on(store.store_refresh_token("h", "token")));
    assert_runtime_boundary_error(block_on(store.readiness_probe()));
    assert_no_runtime_boundary_io(&server);
}

/// A current-thread runtime directly awaits the complete OpenBao HTTP flow.
#[tokio::test(flavor = "current_thread")]
async fn d8_async_facade_current_thread_localset_preserves_http_behavior() {
    let handle = "tenant-a/seat-1";
    let plaintext = "refresh-token";
    let ciphertext = "vault:v1:localset";
    let plaintext_b64 = BASE64.encode(plaintext);

    let server = vault_server(vec![
        (
            "POST",
            transit_encrypt_path(),
            json_response(200, transit_encrypt_response(ciphertext)),
        ),
        ("POST", kv_path(handle), ScriptedResponse::ok()),
        (
            "GET",
            kv_path(handle),
            json_response(200, kv_read_response(ciphertext)),
        ),
        (
            "POST",
            transit_decrypt_path(),
            json_response(200, transit_decrypt_response(&plaintext_b64)),
        ),
        ("GET", "/v1/sys/health".to_string(), ScriptedResponse::ok()),
    ]);
    let store = make_store(&server);
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            store.store_refresh_token(handle, plaintext).await.unwrap();
            assert_eq!(store.fetch_refresh_token(handle).await.unwrap(), plaintext);
            store.readiness_probe().await.unwrap();
        })
        .await;

    assert_eq!(hits(&server, "POST", &transit_encrypt_path()), 1);
    assert_eq!(hits(&server, "POST", &kv_path(handle)), 1);
    assert_eq!(hits(&server, "GET", &kv_path(handle)), 1);
    assert_eq!(hits(&server, "POST", &transit_decrypt_path()), 1);
    assert_eq!(hits(&server, "GET", "/v1/sys/health"), 1);
    // The full flow is five calls and no more — no retry, no stray probe.
    assert_eq!(server.request_count(), 5, "{:?}", server.request_lines());
}

/// A spawned request on a one-worker runtime completes its real readiness
/// request; no worker is synchronously held while OpenBao HTTP progresses.
#[test]
fn d8_async_facade_one_worker_spawned_handler_completes() {
    let server = vault_server(vec![(
        "GET",
        "/v1/sys/health".to_string(),
        ScriptedResponse::ok(),
    )]);
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

    assert_eq!(hits(&server, "GET", "/v1/sys/health"), 1);
}
