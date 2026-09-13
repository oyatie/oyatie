#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::Instant;

use intelligence_kernel::{AgentId, SeatOutcome, SubscriptionPool, TenantId};
use intelligence_rest_proxy::{AnthropicAdapter, RestAdapterError};
use scripted_http_server::{ScriptedResponse, ScriptedServer};

mod d3_anthropic_adapter_integration_support;
use d3_anthropic_adapter_integration_support::*;

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
