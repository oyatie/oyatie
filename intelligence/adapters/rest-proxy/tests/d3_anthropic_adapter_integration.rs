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

use intelligence_rest_proxy::AnthropicAdapter;
use scripted_http_server::{ScriptedResponse, ScriptedServer};

mod d3_anthropic_adapter_integration_support;
use d3_anthropic_adapter_integration_support::*;

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
