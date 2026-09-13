//! Stage-7 SSE streaming passthrough tests.
//!
//! Covers:
//! 1. Non-streaming request (Accept omitted) → returns one-shot JSON body.
//! 2. Streaming request (Accept: text/event-stream) → returns chunked body matching upstream SSE bytes.
//! 3. Streaming request preserves hop-by-hop filtering on the response path.
//! 4. Lease NOT released until stream completes (3 streams vs 2-seat pool → 3rd is 503).
//! 5. Stream error mid-flight → seat outcome is ServerError5xx / RefreshFailed, NOT Ok.
//! 6. Client drops stream before completion → lease still released cleanly via Drop.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use axum::body::Body;
use futures::StreamExt as _;
use intelligence_rest_proxy::AnthropicAdapter;
use scripted_http_server::{Chunk, ScriptedResponse, ScriptedServer};

mod d3_sse_streaming_support;
use d3_sse_streaming_support::*;

// ---------------------------------------------------------------------------
// Test 1: Non-streaming request returns one-shot JSON body
// ---------------------------------------------------------------------------

/// SSE-1: Non-streaming request (Accept omitted) → one-shot JSON body returned.
#[tokio::test]
async fn sse1_non_streaming_returns_json_body() {
    let expected = r#"{"id":"msg-1","type":"message"}"#;
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok","refresh_token":"rt2","expires_in":3600}"#),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(expected),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt".to_string(),
        },
        server.base_url().to_owned(),
    );
    let client = make_client();
    let req = proxy_req_json("/v1/messages", "t1");
    let resp = adapter.proxy(&client, &req, "t1/seat-1").await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, expected.as_bytes());
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
}

// ---------------------------------------------------------------------------
// Test 2: Streaming request returns chunked SSE bytes matching upstream
// ---------------------------------------------------------------------------

/// SSE-2: Streaming request → raw SSE bytes from upstream passed through intact.
#[tokio::test]
async fn sse2_streaming_returns_sse_bytes() {
    let delta_event = "data: {\"type\":\"content_block_delta\"}\n\n";
    let done_event = "data: [DONE]\n\n";
    let sse_body = format!("{delta_event}{done_event}");
    // Unlike httpmock, this server sends a GENUINELY chunked `text/event-stream`
    // response — one frame per event, each flushed on its own — so the pass-through
    // path is exercised against real incremental framing rather than one buffered write.
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-sse","refresh_token":"rt-sse","expires_in":3600}"#),
        ScriptedResponse::ok().sse(vec![
            Chunk::new(delta_event),
            Chunk::after(std::time::Duration::from_millis(30), done_event),
        ]),
    ]);

    let adapter = AnthropicAdapter::with_base_url(
        StubStore {
            token: "rt-sse".to_string(),
        },
        server.base_url().to_owned(),
    );
    let client = make_client();
    let req = proxy_req_sse("/v1/messages", "t2");

    // proxy_stream requires the access token to be pre-fetched.
    let access_token = adapter.refresh_token(&client, "t2/seat-2").await.unwrap();
    let (status, mut stream) = adapter
        .proxy_stream(&client, &access_token, req)
        .await
        .unwrap();

    assert_eq!(status, 200);

    let mut collected = Vec::<u8>::new();
    let mut frames = 0usize;
    while let Some(chunk) = stream.next().await {
        collected.extend_from_slice(&chunk.unwrap());
        frames += 1;
    }
    assert_eq!(collected, sse_body.as_bytes());
    // Pass-through must stay INCREMENTAL: the two upstream frames must not be
    // coalesced into one before reaching the caller. httpmock could not express this.
    assert!(
        frames >= 2,
        "SSE frames were buffered into {frames} chunk(s); streaming pass-through must \
         forward each upstream frame as it arrives"
    );

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    // Was a `header("accept", "text/event-stream")` matcher on the mock.
    assert_eq!(requests[1].header("accept"), Some("text/event-stream"));
}

// ---------------------------------------------------------------------------
// Test 3: Streaming preserves hop-by-hop filtering (no transfer-encoding leaked)
// ---------------------------------------------------------------------------

/// SSE-3: Streaming response — hop-by-hop headers are not present on the axum
/// response. (The router sets only the gateway's own headers.)
#[tokio::test]
async fn sse3_streaming_response_hop_by_hop_not_leaked() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-hbh","refresh_token":"rt-hbh","expires_in":3600}"#),
        // The httpmock original carried the note "httpmock does not actually send
        // chunked encoding", so the leak this test is named for could not actually
        // occur and the assertion below was vacuous. `.sse(..)` sends a REAL
        // `Transfer-Encoding: chunked` response, and reqwest surfaces
        // transfer-encoding/connection/keep-alive on it (verified), so the filter now
        // has something genuine to strip.
        // Every upstream hop-by-hop value here is DISTINGUISHABLE from anything the
        // gateway sets for its own hop, so "was it stripped?" is answerable. The
        // gateway legitimately sets `connection: keep-alive` itself (lib.rs, SSE branch)
        // — that is its own hop, not a leak — so the upstream value carries an extra
        // nominated token that must never appear downstream.
        ScriptedResponse::ok()
            .header("connection", "keep-alive, x-upstream-nominated")
            .header("keep-alive", "timeout=97")
            .header("x-upstream-nominated", "leak-me")
            .sse(vec![Chunk::new("data: ping\n\n")]),
    ]);

    // Use the full axum router path for this test.
    let pool = make_pool_2_seats("t3");
    let state = make_app_state(server.base_url().to_owned(), pool, "t3");
    let app = intelligence_rest_proxy::build_router(state);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .header("authorization", "Bearer ingress-token")
        .header("accept", "text/event-stream")
        .header("x-agent-id", "agent-sse3")
        .body(Body::from(
            r#"{"model":"claude-opus-4-5","max_tokens":5,"messages":[],"stream":true}"#,
        ))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
    // transfer-encoding and connection must NOT be forwarded by the gateway.
    assert!(
        response.headers().get("transfer-encoding").is_none()
            || response
                .headers()
                .get("transfer-encoding")
                .map(|v| v.to_str().unwrap_or(""))
                != Some("chunked"),
        "transfer-encoding: chunked must not be leaked from upstream"
    );
    // Upstream's `Keep-Alive: timeout=97` and its Connection-nominated header must not
    // survive the hop. The httpmock original could assert none of this: its own comment
    // records that httpmock never actually sent these headers, so the single assertion
    // above was vacuous. This server does send them (verified against reqwest).
    assert!(
        response.headers().get("keep-alive").is_none(),
        "upstream Keep-Alive must not be forwarded: {:?}",
        response.headers()
    );
    assert!(
        response.headers().get("x-upstream-nominated").is_none(),
        "the upstream Connection-nominated header must not be forwarded: {:?}",
        response.headers()
    );
    // `connection` is the gateway's OWN per-hop header on the SSE branch, so its
    // presence is correct — but it must be the gateway's value, never upstream's.
    if let Some(connection) = response.headers().get("connection") {
        let connection = connection.to_str().unwrap_or("");
        assert!(
            !connection.contains("x-upstream-nominated"),
            "upstream's Connection value leaked downstream: {connection}"
        );
    }
    for hop in ["upgrade", "proxy-authenticate", "proxy-authorization", "te"] {
        assert!(
            response.headers().get(hop).is_none(),
            "hop-by-hop header '{hop}' must not be present: {:?}",
            response.headers()
        );
    }
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
}
