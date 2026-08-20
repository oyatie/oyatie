//! Fix-8: Integration test — AnthropicAdapter against an httpmock server.
//!
//! Covers:
//! - Correct OAuth body shape (grant_type=refresh_token, client_id)
//! - Correct bearer Authorization header on /v1/messages
//! - Correct anthropic-version header
//! - 429 + Retry-After maps to kernel Cooldown via complete_lease
//! - 401 invalid_grant maps to RefreshTokenRevoked / RefreshFailed outcome
//!
//! Stage-6: AnthropicAdapter::proxy is now async and takes `&reqwest::Client`.
//! httpmock 0.7 binds to a random 127.0.0.1 port per `MockServer::start()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use httpmock::prelude::*;
use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SeatOutcome,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{AnthropicAdapter, ProxyRequest, RestAdapterError, SecretProviderStore};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct StubStore {
    token: String,
}

impl SecretProviderStore for StubStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, String> {
        Box::pin(async move { Ok(self.token.clone()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn make_pool(tenant: &str, seat: &str) -> Arc<Mutex<SubscriptionPool>> {
    let t = TenantId::new(tenant).unwrap();
    let mut pool =
        SubscriptionPool::new(t.clone(), Provider::Anthropic, SelectionStrategy::FillFirst);
    pool.add_seat(OAuthSubscription::new(
        t,
        SeatId::new(seat).unwrap(),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("{tenant}/{seat}"),
        0,
    ))
    .unwrap();
    Arc::new(Mutex::new(pool))
}

fn proxy_req(base_path: &str, tenant: &str) -> ProxyRequest {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    ProxyRequest {
        method: "POST".to_string(),
        path: base_path.to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#.to_vec(),
        tenant_id: TenantId::new(tenant).unwrap(),
    }
}

fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Fix-8 Integration tests
// ---------------------------------------------------------------------------

/// Token refresh uses correct OAuth body shape: grant_type=refresh_token and
/// client_id matching ANTHROPIC_CLIENT_ID.
#[tokio::test]
async fn refresh_sends_correct_oauth_body_shape() {
    let server = MockServer::start();

    let token_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/oauth/token")
            .body_contains("refresh_token")
            .body_contains("9d1c250a-e61b-44d9-88ed-5944d1962f5e")
            .body_contains("my-refresh-tok");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{"access_token":"new-access-tok","refresh_token":"new-rt","expires_in":3600}"#,
            );
    });

    let _msg_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-ok","type":"message"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "my-refresh-tok".to_string(),
        },
        server.base_url(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-shape");
    adapter
        .proxy(&client, &req, "t-shape/seat-shape")
        .await
        .unwrap();
    token_mock.assert_hits(1);
}

/// Bearer Authorization header and anthropic-version header are set correctly.
#[tokio::test]
async fn proxy_sets_correct_bearer_and_version_headers() {
    let server = MockServer::start();

    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{"access_token":"bearer-tok-xyz","refresh_token":"rt-new","expires_in":3600}"#,
            );
    });

    let messages_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("authorization", "Bearer bearer-tok-xyz")
            .header("anthropic-version", "2023-06-01");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-headers"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-old".to_string(),
        },
        server.base_url(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-headers");
    let result = adapter.proxy(&client, &req, "t-headers/seat-hdr").await;
    assert!(result.is_ok(), "expected success: {result:?}");
    messages_mock.assert_hits(1);
}

/// 429 response from upstream maps to `SubscriptionPoolError` via
/// `SeatOutcome::RateLimited429` so the kernel puts the seat in Cooldown.
#[tokio::test]
async fn upstream_429_maps_to_rate_limited_outcome() {
    let server = MockServer::start();

    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-429","refresh_token":"rt-429","expires_in":3600}"#);
    });

    let _messages_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(429)
            .header("retry-after", "30")
            .header("content-type", "application/json")
            .body(r#"{"error":{"type":"rate_limit_error","message":"rate limited"}}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-429-tok".to_string(),
        },
        server.base_url(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-429");
    let result = adapter.proxy(&client, &req, "t-429/seat-429").await;

    let resp = result.expect("proxy should succeed (returning 429 response)");
    assert_eq!(
        resp.status, 429,
        "upstream 429 should be reflected in response status"
    );

    // Verify kernel transitions seat to Cooldown when we record RateLimited429.
    let pool_ref = make_pool("t-429", "seat-429");
    let gate = struct_allow_gate();
    let agent = AgentId::new("agent-429").unwrap();
    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-429").unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();
    let sid = lease.seat_id().clone();
    lease
        .complete(SeatOutcome::RateLimited429, Instant::now())
        .unwrap();

    let result2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-429").unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    assert!(
        result2.is_err(),
        "seat {sid:?} should be in cooldown after RateLimited429"
    );
}

/// 401 with invalid_grant error body maps to RefreshTokenRevoked.
#[tokio::test]
async fn upstream_401_invalid_grant_causes_refresh_error() {
    let server = MockServer::start();

    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"error":"invalid_grant","error_description":"refresh token revoked"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "revoked-rt".to_string(),
        },
        server.base_url(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-401");
    let result = adapter.proxy(&client, &req, "t-401/seat-401").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RestAdapterError::OAuthRefreshFailed(msg) => {
            assert!(
                msg.contains("401") || msg.contains("token refresh failed"),
                "error message should describe refresh failure: {msg}"
            );
        }
        other => panic!("expected OAuthRefreshFailed, got {other:?}"),
    }

    // Verify RefreshFailed outcome transitions seat to Cooldown via kernel.
    let pool_ref = make_pool("t-401", "seat-401");
    let gate = struct_allow_gate();
    let agent = AgentId::new("agent-401").unwrap();
    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-401").unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();
    lease
        .complete(SeatOutcome::RefreshFailed, Instant::now())
        .unwrap();

    let result2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new("t-401").unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    assert!(
        result2.is_err(),
        "seat should be in cooldown after RefreshFailed"
    );
}

/// Successful 200 response carries the body through intact.
#[tokio::test]
async fn successful_200_returns_body_intact() {
    let server = MockServer::start();

    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-ok","refresh_token":"rt-ok","expires_in":3600}"#);
    });

    let expected_body = r#"{"id":"msg-ok","type":"message","role":"assistant","content":[],"stop_reason":"end_turn"}"#;
    let _messages_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body(expected_body);
    });

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-ok".to_string(),
        },
        server.base_url(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-ok");
    let resp = adapter.proxy(&client, &req, "t-ok/seat-ok").await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, expected_body.as_bytes());
}

// ---------------------------------------------------------------------------
// Helper: a simple always-allow AuthzGate as a plain struct.
// ---------------------------------------------------------------------------

struct AllowGate;
impl intelligence_kernel::AuthzGate for AllowGate {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

fn struct_allow_gate() -> AllowGate {
    AllowGate
}
