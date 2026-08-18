//! Integration tests for the OpenAI API-key pool adapter.
//!
//! All tests use an in-process mock HTTP server bound to 127.0.0.1:0.
//! No real OpenAI network calls are made.
//!
//! ADR-0083 Tier 3: test-only unwrap/expect/panic are allowed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use intelligence_account_domain::{
    SecretMaterial, SecretReference, SecretStoreError, SecretStorePort,
};
use intelligence_account_kernel::{AuthError, ProviderAuthPort};
use oya_intelligence_adapter_openai_subscription_adapter::{
    KeyPool, KeyStatus, OpenAiApiKeyPoolAdapter,
};

// ── Shared secret store ──────────────────────────────────────────────────────

#[derive(Clone)]
struct TestStore(Arc<Mutex<HashMap<SecretReference, Vec<u8>>>>);

impl TestStore {
    fn new(entries: Vec<(String, String)>) -> Self {
        let map = entries
            .into_iter()
            .map(|(k, v)| (SecretReference::new(k).unwrap(), v.into_bytes()))
            .collect();
        Self(Arc::new(Mutex::new(map)))
    }
}

impl SecretStorePort for TestStore {
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

// ── Mock server helpers ──────────────────────────────────────────────────────

async fn bind_mock_server() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

/// Serve one request; return the `Authorization` header value received.
async fn serve_one_request(
    listener: TcpListener,
    status: u16,
    response_json: &'static str,
    received_auth: Arc<Mutex<String>>,
) {
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);
    http1::Builder::new()
        .serve_connection(
            io,
            service_fn(move |req: Request<hyper::body::Incoming>| {
                let auth_store = Arc::clone(&received_auth);
                async move {
                    let auth = req
                        .headers()
                        .get("authorization")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_owned();
                    *auth_store.lock().unwrap() = auth;
                    // consume body
                    let _ = req.into_body().collect().await.unwrap();
                    let response = Response::builder()
                        .status(status)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(response_json)))
                        .unwrap();
                    Ok::<_, hyper::Error>(response)
                }
            }),
        )
        .await
        .unwrap();
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_pool_adapter(keys: &[&str]) -> OpenAiApiKeyPoolAdapter<TestStore> {
    let sref_paths: Vec<String> = keys.iter().map(|k| format!("sref://{k}")).collect();
    let store_entries: Vec<(String, String)> = sref_paths
        .iter()
        .zip(keys.iter())
        .map(|(p, k)| (p.clone(), (*k).to_owned()))
        .collect();
    let store = TestStore::new(store_entries);
    let pool = KeyPool::new(sref_paths).with_jitter_max(0);
    OpenAiApiKeyPoolAdapter::new(pool, store)
        .with_clock(|| 1_000_000u64)
        .with_jitter(|| 0u64)
}

fn sref(s: &str) -> SecretReference {
    SecretReference::new(s.to_owned()).unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn mock_server_receives_bearer_header_not_x_api_key() {
    let (listener, addr) = bind_mock_server().await;
    let received_auth = Arc::new(Mutex::new(String::new()));

    let response_json = r#"{"id":"chatcmpl-test","object":"chat.completion"}"#;
    let auth_store = Arc::clone(&received_auth);
    tokio::spawn(async move {
        serve_one_request(listener, 200, response_json, auth_store).await;
    });

    let adapter = make_pool_adapter(&["sk-test-key-abc"]);

    // Get auth headers for key 0
    let hdrs = adapter.auth_headers_for(0).unwrap();
    let auth_val = hdrs.iter().find(|(k, _)| k == "authorization").unwrap();

    // Should be Bearer, not x-api-key
    assert!(
        auth_val.1.starts_with("Bearer "),
        "header must use Bearer scheme: {}",
        auth_val.1
    );
    assert!(
        !hdrs.iter().any(|(k, _)| k == "x-api-key"),
        "must not use x-api-key"
    );

    // Simulate the HTTP call using hyper with those headers
    let http_client = Arc::new(
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build_http(),
    );
    let url = format!("http://{addr}/v1/chat/completions");
    let mut req_builder = hyper::Request::builder()
        .method("POST")
        .uri(&url)
        .header("content-type", "application/json");
    for (k, v) in &hdrs {
        req_builder = req_builder.header(k.as_str(), v.as_str());
    }
    let req = req_builder
        .body(Full::new(Bytes::from(
            r#"{"model":"gpt-4o","messages":[]}"#,
        )))
        .unwrap();

    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Verify the server saw Bearer header
    let auth_received = received_auth.lock().unwrap().clone();
    assert!(
        auth_received.starts_with("Bearer "),
        "server received: {auth_received}"
    );
    assert!(
        auth_received.contains("sk-test-key-abc"),
        "server must receive actual key material: {auth_received}"
    );
}

#[tokio::test]
async fn terminal_401_blacklists_key_and_exhausts_pool() {
    let adapter = make_pool_adapter(&["sk-key0"]);

    // Record a 401 result
    adapter.record_call_result(0, 401, None);

    // Pool should be exhausted
    assert_eq!(
        adapter.authenticate(&sref("sref://unused")),
        Err(AuthError::NetworkUnavailable),
        "all keys blacklisted, must return NetworkUnavailable"
    );
}

#[tokio::test]
async fn terminal_401_on_first_key_falls_back_to_second() {
    let adapter = make_pool_adapter(&["sk-key0", "sk-key1"]);

    // Blacklist key 0
    adapter.record_call_result(0, 401, None);

    // Next authenticate should succeed with key 1
    let t = adapter.authenticate(&sref("sref://unused")).unwrap();
    assert!(
        t.token_id_redacted().ends_with("-1"),
        "expected pool index 1, got: {}",
        t.token_id_redacted()
    );
}

#[tokio::test]
async fn three_transient_429s_enter_cooling_all_keys_unavailable() {
    let adapter = make_pool_adapter(&["sk-key0"]);

    // 3 consecutive transient 429s
    adapter.record_call_result(0, 429, None);
    adapter.record_call_result(0, 429, None);
    adapter.record_call_result(0, 429, None);

    // Key is cooling; at epoch 1_000_000 it should be cooling until 1_000_060
    assert_eq!(
        adapter.authenticate(&sref("sref://unused")),
        Err(AuthError::NetworkUnavailable),
        "cooling key must not be selected"
    );
}

#[tokio::test]
async fn insufficient_quota_429_blacklists_via_error_type() {
    let adapter = make_pool_adapter(&["sk-key0"]);

    let body = br#"{"error":{"type":"insufficient_quota","message":"You exceeded your current quota.","code":"insufficient_quota"}}"#;
    adapter.record_call_result(0, 429, Some(body));

    // Key must be blacklisted (terminal), not cooling
    assert_eq!(
        adapter.authenticate(&sref("sref://unused")),
        Err(AuthError::NetworkUnavailable),
    );
}

#[tokio::test]
async fn all_keys_blacklisted_returns_network_unavailable() {
    let adapter = make_pool_adapter(&["sk-k0", "sk-k1", "sk-k2"]);
    adapter.record_call_result(0, 401, None);
    adapter.record_call_result(1, 403, None);
    adapter.record_call_result(2, 401, None);

    assert_eq!(
        adapter.authenticate(&sref("sref://unused")),
        Err(AuthError::NetworkUnavailable),
    );
}

#[test]
fn bearer_header_contains_key_material() {
    let adapter = make_pool_adapter(&["sk-actual-key-value"]);
    let hdrs = adapter.auth_headers_for(0).unwrap();
    let auth = hdrs.iter().find(|(k, _)| k == "authorization").unwrap();
    assert_eq!(auth.1, "Bearer sk-actual-key-value");
}

#[test]
fn no_x_api_key_in_headers() {
    let adapter = make_pool_adapter(&["sk-key"]);
    let hdrs = adapter.auth_headers_for(0).unwrap();
    assert!(!hdrs.iter().any(|(k, _)| k == "x-api-key"));
}
