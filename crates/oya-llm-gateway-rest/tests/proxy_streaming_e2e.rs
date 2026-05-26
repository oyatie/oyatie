//! End-to-end behavioral proof of the reverse proxy: real axum app + real
//! HTTP upstream (a tiny in-process mock), exercising ingress auth, pooled-key
//! injection, SSE streaming passthrough, and 429 → next-key failover.
//!
//! No network egress: the "upstream" is a `tokio` TCP listener on localhost
//! that the gateway forwards to. Pooled keys are supplied directly (the
//! OpenBao path is unit-tested separately); this test focuses on the proxy
//! data path.
//!
//! The test client is a minimal hyper-util client (reqwest is intentionally
//! NOT a dependency of this crate — the gateway proxies on hyper directly).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A minimal localhost HTTP test client over hyper-util (plain HTTP only — the
/// gateway and mock upstream both listen on `127.0.0.1`). Returns the response
/// status and the fully-collected body as a string.
type TestClient = Client<HttpConnector, Full<Bytes>>;

fn test_client() -> TestClient {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

/// Send a request with the given method/url/headers/body and collect the
/// response. `(status, body_string)`.
async fn send(
    method: hyper::Method,
    url: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> (u16, String) {
    let client = test_client();
    let mut builder = hyper::Request::builder().method(method).uri(url);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder
        .body(Full::new(Bytes::copy_from_slice(body.as_bytes())))
        .unwrap();
    let resp = client.request(request).await.unwrap();
    let status = resp.status().as_u16();
    let collected = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&collected).to_string())
}

async fn post(url: &str, headers: &[(&str, &str)], body: &str) -> (u16, String) {
    send(hyper::Method::POST, url, headers, body).await
}

async fn get(url: &str) -> (u16, String) {
    send(hyper::Method::GET, url, &[], "").await
}

/// Build a framing-correct HTTP/1.1 response: a status line, an accurate
/// `Content-Length`, `Connection: close` (so EOF cleanly delimits the body),
/// any extra headers, then the body.
fn http_response(status_line: &str, extra_headers: &[(&str, &str)], body: &str) -> String {
    let mut head = format!("HTTP/1.1 {status_line}\r\n");
    for (k, v) in extra_headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    head.push_str("Connection: close\r\n\r\n");
    head.push_str(body);
    head
}

/// A tiny mock upstream. For each accepted connection it reads the request
/// (capturing the `authorization` header), then replies with `script` — a
/// closure mapping the request count to a raw HTTP/1.1 response string.
async fn spawn_mock_upstream<F>(script: F) -> (String, Arc<AtomicUsize>, Arc<std::sync::Mutex<Vec<String>>>)
where
    F: Fn(usize) -> String + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let seen_auth = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let count2 = Arc::clone(&count);
    let seen2 = Arc::clone(&seen_auth);
    let script = Arc::new(script);

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let n = count2.fetch_add(1, Ordering::SeqCst);
            let script = Arc::clone(&script);
            let seen = Arc::clone(&seen2);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let read = socket.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..read]).to_string();
                // Capture the Authorization header value the upstream saw.
                // Match the header NAME case-insensitively but preserve the
                // original-case VALUE (so we can assert the exact bytes the
                // gateway injected).
                for line in req.lines() {
                    if let Some(colon) = line.find(':') {
                        let (name, value) = line.split_at(colon);
                        if name.eq_ignore_ascii_case("authorization") {
                            seen.lock().unwrap().push(value[1..].trim().to_string());
                        }
                    }
                }
                let response = script(n);
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    (format!("http://{addr}"), count, seen_auth)
}

fn build_state(upstream: &str, keys: &[(&str, &str)], policy: PoolPolicy, retry: RetryPolicyConfig) -> Arc<GatewayState> {
    let mut map = BTreeMap::new();
    for (label, key) in keys {
        map.insert((*label).to_string(), (*key).to_string());
    }
    let material = KeyMaterial::from_map(ProviderChannel::OpenAi, map);
    let adapter = ChannelAdapter::new(ProviderChannel::OpenAi, upstream.to_string(), None);
    let group = GroupRuntime::new("codex", adapter, retry, policy, material);
    let mut groups = BTreeMap::new();
    groups.insert("codex".to_string(), group);
    let auth = AuthVerifier::new("admin-tok", vec!["ingress-secret".to_string()]);
    let metrics = GatewayMetrics::new().unwrap();
    Arc::new(GatewayState::new(groups, auth, metrics))
}

/// Boot the gateway router on a localhost port; return its base URL.
async fn spawn_gateway(state: Arc<GatewayState>) -> String {
    let app = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn unauthorized_without_ingress_key() {
    let (upstream, _c, _a) = spawn_mock_upstream(|_| http_response("200 OK", &[], "hi")).await;
    let state = build_state(&upstream, &[("a", "sk-aaa")], PoolPolicy::new(3, 1000, 0), RetryPolicyConfig::default());
    let base = spawn_gateway(state).await;

    // No x-oya-proxy-key header → 401, and the upstream must NOT be hit.
    let (status, _body) = post(&format!("{base}/proxy/codex/v1/chat"), &[], "{}").await;
    assert_eq!(status, 401);
}

#[tokio::test]
async fn streams_upstream_body_and_injects_pooled_bearer() {
    // SSE-style chunked body. The gateway must pass it through verbatim and
    // the upstream must observe the POOLED key, never a client-supplied one.
    let sse_body = "data: {\"delta\":\"hello\"}\n\ndata: [DONE]\n\n";
    let (upstream, count, seen_auth) = spawn_mock_upstream(move |_| {
        http_response("200 OK", &[("Content-Type", "text/event-stream")], sse_body)
    })
    .await;
    let state = build_state(&upstream, &[("a", "sk-POOLED-aaa")], PoolPolicy::new(3, 1000, 0), RetryPolicyConfig::default());
    let base = spawn_gateway(state).await;

    let (status, body) = post(
        &format!("{base}/proxy/codex/v1/chat/completions"),
        &[
            ("x-oya-proxy-key", "ingress-secret"),
            // A client-supplied Authorization must be stripped + replaced.
            ("authorization", "Bearer CLIENT-SHOULD-BE-DROPPED"),
        ],
        "{\"stream\":true}",
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.contains("data: {\"delta\":\"hello\"}"));
    assert!(body.contains("[DONE]"));
    assert_eq!(count.load(Ordering::SeqCst), 1, "upstream hit exactly once");

    // The upstream saw the POOLED bearer, not the client's.
    let auths = seen_auth.lock().unwrap().clone();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0], "Bearer sk-POOLED-aaa");
}

#[tokio::test]
async fn retries_on_429_then_succeeds_with_next_key() {
    // First attempt → 429 (retryable); second attempt → 200. With 2 keys the
    // proxy rotates to the next key and succeeds.
    let (upstream, count, seen_auth) = spawn_mock_upstream(|n| {
        if n == 0 {
            http_response("429 Too Many Requests", &[], "")
        } else {
            http_response("200 OK", &[], "ok")
        }
    })
    .await;
    let retry = RetryPolicyConfig {
        retry_on_statuses: vec![429, 500, 502, 503, 504],
        max_attempts: 3,
        backoff_base_millis: 0,
        backoff_jitter_millis: 0,
    };
    let state = build_state(
        &upstream,
        &[("a", "sk-key-1"), ("b", "sk-key-2")],
        PoolPolicy::new(3, 60_000, 0),
        retry,
    );
    let base = spawn_gateway(state).await;

    let (status, body) = post(
        &format!("{base}/proxy/codex/v1/chat"),
        &[("x-oya-proxy-key", "ingress-secret")],
        "{}",
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body, "ok");
    assert_eq!(count.load(Ordering::SeqCst), 2, "one retry → two upstream hits");

    // Two distinct pooled keys were used across the two attempts (round-robin).
    let auths = seen_auth.lock().unwrap().clone();
    assert_eq!(auths.len(), 2);
    assert_ne!(auths[0], auths[1]);
    assert!(auths.iter().all(|a| a.starts_with("Bearer sk-key-")));
}

#[tokio::test]
async fn all_429_exhausts_retries_and_returns_503() {
    let (upstream, count, _a) =
        spawn_mock_upstream(|_| http_response("429 Too Many Requests", &[], "")).await;
    let retry = RetryPolicyConfig {
        retry_on_statuses: vec![429],
        max_attempts: 2,
        backoff_base_millis: 0,
        backoff_jitter_millis: 0,
    };
    let state = build_state(&upstream, &[("a", "sk-1"), ("b", "sk-2")], PoolPolicy::new(5, 60_000, 0), retry);
    let base = spawn_gateway(state).await;

    let (status, _body) = post(
        &format!("{base}/proxy/codex/v1/chat"),
        &[("x-oya-proxy-key", "ingress-secret")],
        "{}",
    )
    .await;
    // Retries exhausted → 503.
    assert_eq!(status, 503);
    assert_eq!(count.load(Ordering::SeqCst), 2, "exactly max_attempts upstream hits");
}

#[tokio::test]
async fn metrics_endpoint_exposes_families() {
    let (upstream, _c, _a) = spawn_mock_upstream(|_| http_response("200 OK", &[], "hi"))
    .await;
    let state = build_state(&upstream, &[("a", "sk-aaa")], PoolPolicy::new(3, 1000, 0), RetryPolicyConfig::default());
    let base = spawn_gateway(state).await;

    let (status, body) = get(&format!("{base}/metrics")).await;
    assert_eq!(status, 200);
    assert!(body.contains("oya_llm_gateway_active_keys"));
    // The active-key gauge for the group is exposed; no secret appears.
    assert!(body.contains("group=\"codex\""));
    assert!(!body.contains("sk-aaa"));
}

#[tokio::test]
async fn unknown_group_returns_404() {
    let (upstream, _c, _a) = spawn_mock_upstream(|_| http_response("200 OK", &[], "hi"))
    .await;
    let state = build_state(&upstream, &[("a", "sk-aaa")], PoolPolicy::new(3, 1000, 0), RetryPolicyConfig::default());
    let base = spawn_gateway(state).await;

    let (status, _body) = post(
        &format!("{base}/proxy/nonexistent/v1/chat"),
        &[("x-oya-proxy-key", "ingress-secret")],
        "{}",
    )
    .await;
    assert_eq!(status, 404);
}
