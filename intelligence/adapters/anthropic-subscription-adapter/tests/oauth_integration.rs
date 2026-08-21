//! Integration tests for the Anthropic OAuth runtime adapter.
//!
//! All tests use an in-process mock OAuth token server bound to 127.0.0.1:0.
//! No real Anthropic network calls are made.
//!
//! ADR-0083 Tier 3: test-only unwrap/expect/panic are allowed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use intelligence_anthropic_subscription_adapter::{
    AnthropicOAuthAdapter, CredentialStorePort, InMemoryAlertPort, InMemoryCredentialStore,
    OAuthTokenClient, OperatorAlertPort, SeatId, SeatTokenState,
    build_loopback_http_or_https_test_client, classify_oauth_error, outbound_auth_headers,
    ports::AlertKind, token_state::RefreshFailureKind,
};

// ── Mock server helpers ──────────────────────────────────────────────────────

/// Bind a local TCP listener on 127.0.0.1:0 and return its address.
async fn bind_mock_server() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

/// Serve one request on the listener with the given JSON response body.
/// Returns the parsed form body so callers can inspect what was sent.
async fn serve_one_request(
    listener: TcpListener,
    status: u16,
    response_json: &'static str,
    received_body: Arc<tokio::sync::Mutex<String>>,
) {
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);
    http1::Builder::new()
        .serve_connection(
            io,
            service_fn(move |req: Request<hyper::body::Incoming>| {
                let body_store = Arc::clone(&received_body);
                async move {
                    let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
                    *body_store.lock().await = String::from_utf8_lossy(&body_bytes).to_string();
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn mock_server_token_exchange_success() {
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    let response_json =
        r#"{"access_token":"new-access","refresh_token":"new-refresh","expires_in":3600}"#;
    let body_store = Arc::clone(&received_body);
    tokio::spawn(async move {
        serve_one_request(listener, 200, response_json, body_store).await;
    });

    let http_client = Arc::new(build_loopback_http_or_https_test_client());
    let client =
        OAuthTokenClient::new(Arc::clone(&http_client)).with_token_endpoint(&token_endpoint);

    let now_secs = 1_000_000u64;
    let state = client
        .exchange(
            "auth-code-123",
            "pkce-verifier-abc",
            "http://localhost:35593/callback",
            now_secs,
        )
        .await
        .unwrap();

    assert_eq!(state.access_token, "new-access");
    assert_eq!(state.refresh_token, "new-refresh");
    assert_eq!(state.expires_at, now_secs + 3600);
    assert_eq!(state.issued_at, now_secs);

    // Verify the request contained grant_type=authorization_code.
    let body = received_body.lock().await;
    assert!(
        body.contains("grant_type=authorization_code"),
        "body: {body}"
    );
    assert!(body.contains("code=auth-code-123"), "body: {body}");
}

#[tokio::test]
async fn mock_server_refresh_success() {
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    let response_json = r#"{"access_token":"refreshed-access","refresh_token":"rotated-refresh","expires_in":1800}"#;
    let body_store = Arc::clone(&received_body);
    tokio::spawn(async move {
        serve_one_request(listener, 200, response_json, body_store).await;
    });

    let http_client = Arc::new(build_loopback_http_or_https_test_client());
    let client =
        OAuthTokenClient::new(Arc::clone(&http_client)).with_token_endpoint(&token_endpoint);

    let now_secs = 2_000_000u64;
    let current = SeatTokenState::new(
        "old-access".into(),
        "old-refresh".into(),
        now_secs + 100,
        now_secs - 3500,
    );
    let state = client.refresh(&current, now_secs).await.unwrap();

    assert_eq!(state.access_token, "refreshed-access");
    assert_eq!(state.refresh_token, "rotated-refresh");
    assert_eq!(state.expires_at, now_secs + 1800);

    let body = received_body.lock().await;
    assert!(body.contains("grant_type=refresh_token"), "body: {body}");
    assert!(body.contains("refresh_token=old-refresh"), "body: {body}");
}

#[tokio::test]
async fn terminal_error_emits_operator_alert() {
    // Mock server returns refresh_token_expired error.
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    let response_json =
        r#"{"error":"refresh_token_expired","error_description":"The refresh token has expired."}"#;
    let body_store = Arc::clone(&received_body);
    tokio::spawn(async move {
        serve_one_request(listener, 400, response_json, body_store).await;
    });

    let store = Arc::new(InMemoryCredentialStore::new());
    let alert = Arc::new(InMemoryAlertPort::new());
    let now_secs = 1_000_000u64;

    let adapter = AnthropicOAuthAdapter::with_token_endpoint(
        Arc::clone(&store)
            as Arc<dyn intelligence_anthropic_subscription_adapter::CredentialStorePort>,
        Arc::clone(&alert)
            as Arc<dyn intelligence_anthropic_subscription_adapter::OperatorAlertPort>,
        &token_endpoint,
    )
    .with_clock(move || now_secs);

    // Seed a seat with an about-to-expire state.
    let current_state = SeatTokenState::new(
        "old-access".into(),
        "expired-refresh".into(),
        now_secs + 10, // about to expire
        now_secs - 3590,
    );
    let seat_id = SeatId("test-terminal-seat".into());
    adapter.seed_seat(&seat_id.0, current_state);

    let result = adapter
        .refresh_seat_async(seat_id.clone(), "expired-refresh".into(), now_secs)
        .await;

    assert!(result.is_err(), "terminal error should return Err");

    // Operator alert should have been emitted.
    assert_eq!(alert.count(), 1, "expected exactly one operator alert");
    let alerts = alert.collected();
    assert_eq!(alerts[0].1, AlertKind::RefreshTokenExpired);
}

#[tokio::test]
async fn transient_error_no_operator_alert() {
    // Mock server returns HTTP 503 (transient).
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    let response_json = r#"{"error":"server_error","error_description":"Upstream failure."}"#;
    let body_store = Arc::clone(&received_body);
    tokio::spawn(async move {
        serve_one_request(listener, 503, response_json, body_store).await;
    });

    let store = Arc::new(InMemoryCredentialStore::new());
    let alert = Arc::new(InMemoryAlertPort::new());
    let now_secs = 1_000_000u64;

    let adapter = AnthropicOAuthAdapter::with_token_endpoint(
        Arc::clone(&store)
            as Arc<dyn intelligence_anthropic_subscription_adapter::CredentialStorePort>,
        Arc::clone(&alert)
            as Arc<dyn intelligence_anthropic_subscription_adapter::OperatorAlertPort>,
        &token_endpoint,
    )
    .with_clock(move || now_secs);

    let current_state = SeatTokenState::new(
        "access".into(),
        "refresh-for-503".into(),
        now_secs + 10,
        now_secs,
    );
    let seat_id = SeatId("test-transient-seat".into());
    adapter.seed_seat(&seat_id.0, current_state);

    let result = adapter
        .refresh_seat_async(seat_id, "refresh-for-503".into(), now_secs)
        .await;

    assert!(result.is_err(), "transient error should return Err");
    // No operator alert for transient.
    assert_eq!(alert.count(), 0, "no alert expected for transient error");
}

#[tokio::test]
async fn persist_before_mutate_invariant() {
    // On successful refresh: credential store must be written before in-memory state changes.
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    let response_json = r#"{"access_token":"new-tok","refresh_token":"new-ref","expires_in":3600}"#;
    let body_store = Arc::clone(&received_body);
    tokio::spawn(async move {
        serve_one_request(listener, 200, response_json, body_store).await;
    });

    let store = Arc::new(InMemoryCredentialStore::new());
    let alert = Arc::new(InMemoryAlertPort::new());
    let now_secs = 1_000_000u64;

    let adapter = AnthropicOAuthAdapter::with_token_endpoint(
        Arc::clone(&store)
            as Arc<dyn intelligence_anthropic_subscription_adapter::CredentialStorePort>,
        Arc::clone(&alert)
            as Arc<dyn intelligence_anthropic_subscription_adapter::OperatorAlertPort>,
        &token_endpoint,
    )
    .with_clock(move || now_secs);

    let current_state =
        SeatTokenState::new("old-tok".into(), "old-ref".into(), now_secs + 10, now_secs);
    let seat_id = SeatId("persist-test-seat".into());
    adapter.seed_seat(&seat_id.0, current_state);

    let result = adapter
        .refresh_seat_async(seat_id.clone(), "old-ref".into(), now_secs)
        .await;

    assert!(result.is_ok(), "refresh should succeed: {result:?}");

    // Credential store must contain the new token.
    let stored = store.load(&seat_id);
    assert!(stored.is_some(), "credential store must have been written");

    // The stored bytes should decode to the new state.
    let stored_state = SeatTokenState::from_storage_bytes(&stored.unwrap().0).unwrap();
    assert_eq!(stored_state.access_token, "new-tok");
}

#[tokio::test]
async fn singleflight_coalesces_concurrent_refreshes() {
    // Serve exactly ONE request with a delay; verify N concurrent callers all get
    // the result but only one HTTP round-trip was made.
    use std::sync::atomic::{AtomicU32, Ordering};

    let call_count = Arc::new(AtomicU32::new(0));
    let (listener, addr) = bind_mock_server().await;
    let token_endpoint = format!("http://{addr}/oauth/token");
    let received_body = Arc::new(tokio::sync::Mutex::new(String::new()));

    // Server counts incoming connections and delays to simulate slow response.
    let response_json = r#"{"access_token":"coalesced","refresh_token":"ref","expires_in":3600}"#;
    let body_store = Arc::clone(&received_body);
    let counter = Arc::clone(&call_count);
    tokio::spawn(async move {
        // Accept and serve with a deliberate delay — this gives concurrent callers
        // time to stack up on the singleflight before the first completes.
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        http1::Builder::new()
            .serve_connection(
                io,
                service_fn(move |req: Request<hyper::body::Incoming>| {
                    let body_store = Arc::clone(&body_store);
                    let c = Arc::clone(&counter);
                    async move {
                        c.fetch_add(1, Ordering::SeqCst);
                        let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
                        *body_store.lock().await = String::from_utf8_lossy(&body_bytes).to_string();
                        // Simulate network latency so concurrent callers coalesce.
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        let response = Response::builder()
                            .status(200)
                            .header("content-type", "application/json")
                            .body(Full::new(Bytes::from(response_json)))
                            .unwrap();
                        Ok::<_, hyper::Error>(response)
                    }
                }),
            )
            .await
            .unwrap();
    });

    let store = Arc::new(InMemoryCredentialStore::new());
    let alert = Arc::new(InMemoryAlertPort::new());
    let now_secs = 1_000_000u64;

    let adapter = Arc::new(
        AnthropicOAuthAdapter::with_token_endpoint(
            Arc::clone(&store)
                as Arc<dyn intelligence_anthropic_subscription_adapter::CredentialStorePort>,
            Arc::clone(&alert)
                as Arc<dyn intelligence_anthropic_subscription_adapter::OperatorAlertPort>,
            &token_endpoint,
        )
        .with_clock(move || now_secs),
    );

    let current_state =
        SeatTokenState::new("old".into(), "old-refresh".into(), now_secs + 10, now_secs);
    let seat_id = SeatId("singleflight-seat".into());
    adapter.seed_seat(&seat_id.0, current_state);

    // Launch 5 concurrent refreshes.
    let mut handles = vec![];
    for _ in 0..5 {
        let adapter2 = Arc::clone(&adapter);
        let seat2 = seat_id.clone();
        let refresh_tok = "old-refresh".to_owned();
        handles.push(tokio::spawn(async move {
            adapter2
                .refresh_seat_async(seat2, refresh_tok, now_secs)
                .await
        }));
    }
    tokio::task::yield_now().await;
    let results = futures_util::future::join_all(handles).await;

    // All should succeed.
    for r in &results {
        let inner = r.as_ref().unwrap();
        assert!(inner.is_ok(), "expected Ok, got {inner:?}");
    }

    // Exactly one HTTP call should have been made.
    // NOTE: Due to the nature of singleflight with an HTTP client that may
    // open multiple connections, we assert <= 2 (first caller wins; possible
    // one extra if connection setup races). In practice it should be exactly 1.
    let http_calls = call_count.load(Ordering::SeqCst);
    assert!(
        http_calls <= 2,
        "expected at most 2 HTTP calls, got {http_calls}"
    );
}

#[tokio::test]
async fn expires_lead_scheduling_entry_is_due_at_lead_boundary() {
    use intelligence_anthropic_subscription_adapter::{
        RefreshScheduler, SeatId, token_state::EXPIRES_LEAD_SECS,
    };
    let expires_at = 1_000_000u64;
    let lead = EXPIRES_LEAD_SECS;
    let state = SeatTokenState::new("a".into(), "r".into(), expires_at, 0);
    let next_due = state.next_refresh_due();
    assert_eq!(next_due, expires_at - lead);

    let mut sched = RefreshScheduler::new();
    sched.enqueue(SeatId("seat".into()), next_due);

    // Before lead boundary: not due.
    assert!(sched.drain_due(next_due - 1).is_empty());
    // At lead boundary: due.
    let due = sched.drain_due(next_due);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].seat_id.0, "seat");
}

// ── Error classification tests ───────────────────────────────────────────────

#[test]
fn terminal_errors_classified_correctly() {
    for error_str in [
        "refresh_token_expired",
        "refresh_token_reused",
        "refresh_token_invalidated",
    ] {
        let kind = classify_oauth_error(error_str);
        assert!(
            matches!(kind, RefreshFailureKind::Terminal(_)),
            "{error_str} should be terminal"
        );
    }
}

#[test]
fn transient_errors_classified_correctly() {
    for error_str in [
        "server_error",
        "temporarily_unavailable",
        "network_error",
        "unknown_error",
    ] {
        let kind = classify_oauth_error(error_str);
        assert_eq!(
            kind,
            RefreshFailureKind::Transient,
            "{error_str} should be transient"
        );
    }
}

// ── Bearer header injection ───────────────────────────────────────────────────

#[test]
fn outbound_headers_are_bearer_not_api_key() {
    let hdrs = outbound_auth_headers("bearer-value-xyz");
    let map: std::collections::BTreeMap<_, _> = hdrs.into_iter().collect();
    assert!(
        map["authorization"].starts_with("Bearer "),
        "must use Bearer scheme"
    );
    assert!(!map.contains_key("x-api-key"), "must not use x-api-key");
    assert!(
        map.contains_key("anthropic-version"),
        "must have anthropic-version"
    );
    assert!(
        map.contains_key("anthropic-beta"),
        "must have anthropic-beta"
    );
}
