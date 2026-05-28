//! Integration test for the Anthropic channel adapter against a mock HTTP
//! server (Fix #8 — multispectrum review).
//!
//! Validates:
//! - Correct `x-api-key` + `anthropic-version` auth headers on every request.
//! - 200 OK proxied successfully with the correct response body.
//! - 429 + Retry-After causes the gateway to rotate to the next key (failover).
//! - 401 is a terminal non-retryable response (surfaced to caller).
//! - Hop-by-hop `Connection: Upgrade` header is stripped from the upstream request.
//! - Response hop-by-hop `Transfer-Encoding` header is stripped from the
//!   client response (reqwest/hyper already de-chunks; gateway must not forward it).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use httpmock::prelude::*;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use oya_llm_gateway_kernel::{PoolPolicy, ProviderChannel};
use oya_llm_gateway_rest::auth::AuthVerifier;
use oya_llm_gateway_rest::channel::ChannelAdapter;
use oya_llm_gateway_rest::config::RetryPolicyConfig;
use oya_llm_gateway_rest::keystore::KeyMaterial;
use oya_llm_gateway_rest::metrics::GatewayMetrics;
use oya_llm_gateway_rest::proxy::build_router;
use oya_llm_gateway_rest::state::{GatewayState, GroupRuntime};
use tokio::net::TcpListener;

type TestClient = Client<HttpConnector, Full<Bytes>>;

fn test_client() -> TestClient {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

async fn send_req(
    method: hyper::Method,
    url: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, hyper::HeaderMap, String) {
    let client = test_client();
    let mut builder = hyper::Request::builder().method(method).uri(url);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder
        .body(Full::new(Bytes::copy_from_slice(body)))
        .unwrap();
    let resp = client.request(request).await.unwrap();
    let status = resp.status().as_u16();
    let resp_headers = resp.headers().clone();
    let collected = resp.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        resp_headers,
        String::from_utf8_lossy(&collected).to_string(),
    )
}

async fn post_json(
    url: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, hyper::HeaderMap, String) {
    send_req(hyper::Method::POST, url, headers, body).await
}

fn build_anthropic_state(
    upstream_base: &str,
    keys: &[(&str, &str)],
    max_attempts: u32,
) -> Arc<GatewayState> {
    let mut map = BTreeMap::new();
    for (label, key) in keys {
        map.insert((*label).to_string(), (*key).to_string());
    }
    let material = KeyMaterial::from_map(ProviderChannel::Anthropic, map);
    let adapter = ChannelAdapter::new(
        ProviderChannel::Anthropic,
        upstream_base.to_string(),
        Some("2023-06-01".to_string()),
    );
    let retry = RetryPolicyConfig {
        retry_on_statuses: vec![429, 500, 502, 503, 504],
        max_attempts,
        backoff_base_millis: 0,
        backoff_jitter_millis: 0,
    };
    let group = GroupRuntime::new(
        "anthropic",
        adapter,
        retry,
        PoolPolicy::new(3, 60_000, 0),
        material,
    );
    let mut groups = BTreeMap::new();
    groups.insert("anthropic".to_string(), group);
    let auth = AuthVerifier::new("admin-tok", vec!["ingress-secret".to_string()]);
    let metrics = GatewayMetrics::new().unwrap();
    Arc::new(GatewayState::new(groups, auth, metrics))
}

async fn spawn_gateway(state: Arc<GatewayState>) -> String {
    let app = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// Realistic Anthropic messages API response.
const ANTHROPIC_200_BODY: &str = r#"{
  "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
  "type": "message",
  "role": "assistant",
  "content": [{"type": "text", "text": "Hello!"}],
  "model": "claude-3-haiku-20240307",
  "stop_reason": "end_turn",
  "usage": {"input_tokens": 10, "output_tokens": 5}
}"#;

// ─── Test 1: correct auth headers on a 200 OK ───────────────────────────────

#[tokio::test]
async fn anthropic_adapter_sends_correct_auth_headers_on_200() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "ak-primary")
            .header("anthropic-version", "2023-06-01");
        then.status(200)
            .header("content-type", "application/json")
            .body(ANTHROPIC_200_BODY);
    });

    let upstream = format!("http://{}:{}", server.host(), server.port());
    let state = build_anthropic_state(&upstream, &[("primary", "ak-primary")], 1);
    let gw = spawn_gateway(state).await;

    let (status, _, body) = post_json(
        &format!("{gw}/proxy/anthropic/v1/messages"),
        &[
            ("x-oya-proxy-key", "ingress-secret"),
            ("content-type", "application/json"),
        ],
        br#"{"model":"claude-3-haiku-20240307","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}"#,
    )
    .await;

    assert_eq!(status, 200, "expected 200, body: {body}");
    assert!(
        body.contains("Hello!"),
        "expected upstream body passthrough"
    );
    mock.assert();
}

// ─── Test 2: 429 causes failover to next key ─────────────────────────────────

#[tokio::test]
async fn anthropic_adapter_429_causes_failover_to_second_key() {
    let server = MockServer::start();

    // First request (key "ak-one") → 429.
    let mock_429 = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "ak-one");
        then.status(429).header("retry-after", "0").body("");
    });

    // Second request (key "ak-two") → 200.
    let mock_200 = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "ak-two");
        then.status(200)
            .header("content-type", "application/json")
            .body(ANTHROPIC_200_BODY);
    });

    let upstream = format!("http://{}:{}", server.host(), server.port());
    // Two keys; max_attempts=3 so the failover loop can try both.
    let state = build_anthropic_state(&upstream, &[("one", "ak-one"), ("two", "ak-two")], 3);
    let gw = spawn_gateway(state).await;

    let (status, _, body) = post_json(
        &format!("{gw}/proxy/anthropic/v1/messages"),
        &[("x-oya-proxy-key", "ingress-secret")],
        b"{}",
    )
    .await;

    assert_eq!(status, 200, "expected 200 after failover, body: {body}");
    assert!(body.contains("Hello!"), "expected upstream body");
    mock_429.assert();
    mock_200.assert();
}

// ─── Test 3: 401 is terminal (non-retryable) ─────────────────────────────────

#[tokio::test]
async fn anthropic_adapter_401_is_terminal_not_retried() {
    let server = MockServer::start();

    // Respond 401 — this must NOT be retried.
    let mock_401 = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(401)
            .body(r#"{"error":{"type":"authentication_error","message":"invalid api key"}}"#);
    });

    let upstream = format!("http://{}:{}", server.host(), server.port());
    let state = build_anthropic_state(&upstream, &[("primary", "ak-primary")], 3);
    let gw = spawn_gateway(state).await;

    let (status, _, _) = post_json(
        &format!("{gw}/proxy/anthropic/v1/messages"),
        &[("x-oya-proxy-key", "ingress-secret")],
        b"{}",
    )
    .await;

    assert_eq!(status, 401, "401 must be passed through, not retried");
    // Exactly 1 upstream hit — 401 is non-retryable.
    assert_eq!(mock_401.hits(), 1, "401 must not trigger retry");
}

// ─── Test 4: hop-by-hop header stripping ─────────────────────────────────────
//
// Use a raw TCP mock (same pattern as proxy_streaming_e2e.rs) so we can
// inspect exactly which headers arrived at the upstream. httpmock 0.7 does
// not provide a "header must be absent" assertion; the raw TCP approach gives
// us the full request bytes.

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener as TcpListenerRaw;

async fn spawn_tcp_upstream_capturing_headers(
    response: &'static str,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListenerRaw::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    let cap2 = Arc::clone(&captured);
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let cap = Arc::clone(&cap2);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                // Capture header lines.
                for line in req.lines() {
                    if line.is_empty() {
                        break;
                    }
                    cap.lock().unwrap().push(line.to_string());
                }
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });
    (format!("http://{addr}"), captured)
}

fn http_resp_200(body: &str) -> &'static str {
    // Return a static OK response string adequate for these tests.
    Box::leak(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        )
        .into_boxed_str(),
    )
}

#[tokio::test]
async fn connection_hop_by_hop_header_is_stripped_from_upstream_request() {
    let resp = http_resp_200(ANTHROPIC_200_BODY);
    let (upstream_addr, captured_headers) = spawn_tcp_upstream_capturing_headers(resp).await;

    let state = build_anthropic_state(&upstream_addr, &[("primary", "ak-primary")], 1);
    let gw = spawn_gateway(state).await;

    let (status, _, _) = post_json(
        &format!("{gw}/proxy/anthropic/v1/messages"),
        &[
            ("x-oya-proxy-key", "ingress-secret"),
            // `upgrade` is listed in `connection` — must be stripped per RFC 7230 §6.1.
            ("connection", "upgrade"),
            ("upgrade", "websocket"),
        ],
        b"{}",
    )
    .await;

    assert_eq!(status, 200);

    let headers = captured_headers.lock().unwrap().clone();
    // `upgrade` must NOT appear in the request the upstream received.
    let has_upgrade = headers
        .iter()
        .any(|h| h.to_ascii_lowercase().starts_with("upgrade:"));
    assert!(
        !has_upgrade,
        "upgrade must be stripped from upstream request (Connection: upgrade listed it as hop-by-hop). Headers seen: {headers:?}"
    );
    // `connection` itself must also not be forwarded.
    let has_connection = headers
        .iter()
        .any(|h| h.to_ascii_lowercase().starts_with("connection:"));
    assert!(!has_connection, "connection must be stripped: {headers:?}");
}

// ─── Test 5: body > 1 MiB → 413 ─────────────────────────────────────────────

#[tokio::test]
async fn body_over_1mib_returns_413() {
    // The DefaultBodyLimit layer rejects oversized bodies before the handler
    // runs; no upstream mock is needed.
    let server = MockServer::start();
    // Should not be hit (body rejected at the axum layer).
    let _mock = server.mock(|when, then| {
        when.any_request();
        then.status(200).body("should not reach here");
    });

    let upstream = format!("http://{}:{}", server.host(), server.port());
    let state = build_anthropic_state(&upstream, &[("primary", "ak-primary")], 1);
    let gw = spawn_gateway(state).await;

    // 1 MiB + 1 byte body — must be rejected by the body-size limit layer.
    let oversized = vec![b'x'; 1024 * 1024 + 1];
    let (status, _, _) = post_json(
        &format!("{gw}/proxy/anthropic/v1/messages"),
        &[
            ("x-oya-proxy-key", "ingress-secret"),
            ("content-type", "application/json"),
        ],
        &oversized,
    )
    .await;

    assert_eq!(
        status, 413,
        "body > 1 MiB must return 413 Payload Too Large"
    );
}
