//! Fix-8: Integration test — AnthropicAdapter against a scripted HTTP server.
//!
//! Covers:
//! - Correct OAuth body shape (grant_type=refresh_token, client_id)
//! - Correct bearer Authorization header on /v1/messages
//! - Correct anthropic-version header
//! - 429 + Retry-After maps to kernel Cooldown via complete_lease
//! - 401 invalid_grant maps to RefreshTokenRevoked / RefreshFailed outcome
//!
//! Stage-6: AnthropicAdapter::proxy is now async and takes `&reqwest::Client`.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2), which binds a random 127.0.0.1 port per `ScriptedServer::start()`. The
//! `body_contains` / `header` MATCHERS become direct assertions on the recorded
//! request: a matcher only decides which mock answers, so a matcher that stopped
//! matching would silently fall through to another mock, whereas an assertion on the
//! recorded request fails loudly. Request ORDER (token exchange, then upstream call)
//! is asserted too, which the order-independent matchers never expressed.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SeatOutcome,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{AnthropicAdapter, ProxyRequest, RestAdapterError, SecretProviderStore};
use scripted_http_server::{ScriptedResponse, ScriptedServer};

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
    let server =
        ScriptedServer::start(vec![
        ScriptedResponse::ok().header("content-type", "application/json").body(
            r#"{"access_token":"new-access-tok","refresh_token":"new-rt","expires_in":3600}"#,
        ),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-ok","type":"message"}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "my-refresh-tok".to_string(),
        },
        server.base_url().to_owned(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-shape");
    adapter
        .proxy(&client, &req, "t-shape/seat-shape")
        .await
        .unwrap();

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"],
        "exactly one token exchange, then the upstream call"
    );
    // Was three `body_contains` matchers; now three assertions on the body that was
    // actually sent.
    let token_body = requests[0].body_string();
    assert!(
        token_body.contains("refresh_token"),
        "OAuth body must use the refresh_token grant: {token_body}"
    );
    assert!(
        token_body.contains("9d1c250a-e61b-44d9-88ed-5944d1962f5e"),
        "OAuth body must carry ANTHROPIC_CLIENT_ID: {token_body}"
    );
    assert!(
        token_body.contains("my-refresh-tok"),
        "OAuth body must carry the stored refresh token: {token_body}"
    );
}

/// Bearer Authorization header and anthropic-version header are set correctly.
#[tokio::test]
async fn proxy_sets_correct_bearer_and_version_headers() {
    let server =
        ScriptedServer::start(vec![
        ScriptedResponse::ok().header("content-type", "application/json").body(
            r#"{"access_token":"bearer-tok-xyz","refresh_token":"rt-new","expires_in":3600}"#,
        ),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-headers"}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-old".to_string(),
        },
        server.base_url().to_owned(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-headers");
    let result = adapter.proxy(&client, &req, "t-headers/seat-hdr").await;
    assert!(result.is_ok(), "expected success: {result:?}");

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    // The two header MATCHERS become assertions: a matcher that stopped matching would
    // just select a different mock, so the original could not distinguish "header wrong"
    // from "no such mock".
    assert_eq!(
        requests[1].header("authorization"),
        Some("Bearer bearer-tok-xyz"),
        "upstream must carry the freshly-minted access token as a bearer"
    );
    assert_eq!(requests[1].header("anthropic-version"), Some("2023-06-01"));
}

/// 429 response from upstream maps to `SubscriptionPoolError` via
/// `SeatOutcome::RateLimited429` so the kernel puts the seat in Cooldown.
#[tokio::test]
async fn upstream_429_maps_to_rate_limited_outcome() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-429","refresh_token":"rt-429","expires_in":3600}"#),
        ScriptedResponse::status(429)
            .header("retry-after", "30")
            .header("content-type", "application/json")
            .body(r#"{"error":{"type":"rate_limit_error","message":"rate limited"}}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-429-tok".to_string(),
        },
        server.base_url().to_owned(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-429");
    let result = adapter.proxy(&client, &req, "t-429/seat-429").await;

    let resp = result.expect("proxy should succeed (returning 429 response)");
    assert_eq!(
        resp.status, 429,
        "upstream 429 should be reflected in response status"
    );
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    assert_eq!(
        resp.headers.get("retry-after").map(String::as_str),
        Some("30"),
        "Retry-After must survive the response filter — it is what drives the cooldown"
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
    let server = ScriptedServer::start(vec![
        ScriptedResponse::status(401)
            .header("content-type", "application/json")
            .body(r#"{"error":"invalid_grant","error_description":"refresh token revoked"}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "revoked-rt".to_string(),
        },
        server.base_url().to_owned(),
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

    // A failed refresh must not reach the upstream endpoint at all: only the token
    // exchange should appear. The httpmock original scripted no /v1/messages mock, so a
    // stray upstream call would have 404'd rather than failed an assertion.
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token"],
        "a revoked refresh token must never produce an upstream call"
    );

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
    let expected_body = r#"{"id":"msg-ok","type":"message","role":"assistant","content":[],"stop_reason":"end_turn"}"#;
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-ok","refresh_token":"rt-ok","expires_in":3600}"#),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(expected_body),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-ok".to_string(),
        },
        server.base_url().to_owned(),
    );

    let client = make_client();
    let req = proxy_req("/v1/messages", "t-ok");
    let resp = adapter.proxy(&client, &req, "t-ok/seat-ok").await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, expected_body.as_bytes());

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    // The inbound body must also arrive upstream intact, not just the response.
    assert_eq!(
        requests[1].body,
        br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#.to_vec()
    );
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
