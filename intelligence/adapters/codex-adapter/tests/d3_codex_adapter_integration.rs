//! D3 integration tests — CodexAdapter against an httpmock server.
//!
//! Covers:
//! 1. Token refresh — POST shape matches, response parsed correctly.
//! 2. Proxy request — bearer header + cli-version User-Agent correct.
//! 3. 429 with Retry-After → error includes the duration.
//! 4. 401 invalid_grant → terminal RefreshFailed error.
//! 5. 200 streaming SSE → bytes_stream returns the upstream bytes.
//! 6. Hop-by-hop response headers stripped.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::sync::Arc;

use httpmock::prelude::*;
use intelligence_codex_adapter::{CodexAdapter, CodexAdapterError, CodexProxyRequest, HOP_BY_HOP};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_client() -> Arc<reqwest::Client> {
    Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap(),
    )
}

fn empty_request() -> CodexProxyRequest {
    CodexProxyRequest {
        body: br#"{"model":"codex","messages":[]}"#.to_vec(),
        extra_headers: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Test 1: Token refresh — POST shape matches, response parsed correctly.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn token_refresh_post_shape_and_response_parsed() {
    let server = MockServer::start();

    let session_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/auth/session")
            .header("Cookie", "__Secure-next-auth.session-token=my-refresh-tok")
            .header_exists("User-Agent");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"accessToken":"new-access-tok","user":{"email":"test@example.com"}}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.refresh_token("my-refresh-tok").await;

    assert!(result.is_ok(), "expected Ok, got: {result:?}");
    let tokens = result.unwrap();
    assert_eq!(tokens.access_token, "new-access-tok");
    session_mock.assert_hits(1);
}

// ---------------------------------------------------------------------------
// Test 2: Proxy request — bearer header + cli-version User-Agent correct.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_sets_bearer_and_cli_version_user_agent() {
    let server = MockServer::start();

    let codex_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/backend-api/codex/responses")
            .header("authorization", "Bearer test-access-token")
            .header("user-agent", "cli/0.27.0")
            .header("x-openai-beta", "codex-runs");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"resp-ok","object":"codex.response"}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.proxy("test-access-token", empty_request()).await;

    assert!(result.is_ok(), "expected Ok, got: {result:?}");
    let resp = result.unwrap();
    assert_eq!(resp.status, 200);
    codex_mock.assert_hits(1);
}

// ---------------------------------------------------------------------------
// Test 3: 429 with Retry-After → error includes the duration.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_429_with_retry_after_returns_rate_limited_error() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/backend-api/codex/responses");
        then.status(429)
            .header("retry-after", "60")
            .header("content-type", "application/json")
            .body(r#"{"error":"rate_limit_exceeded"}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.proxy("some-token", empty_request()).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CodexAdapterError::RateLimited { retry_after_secs } => {
            assert_eq!(
                retry_after_secs,
                Some(60),
                "expected Retry-After 60 seconds"
            );
        }
        other => panic!("expected RateLimited, got: {other:?}"),
    }
}

#[tokio::test]
async fn refresh_429_with_retry_after_returns_rate_limited_error() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/api/auth/session");
        then.status(429)
            .header("retry-after", "30")
            .body("rate limited");
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.refresh_token("some-refresh-tok").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CodexAdapterError::RateLimited { retry_after_secs } => {
            assert_eq!(retry_after_secs, Some(30));
        }
        other => panic!("expected RateLimited, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 4: 401 invalid_grant → terminal RefreshFailed error.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_401_invalid_grant_returns_refresh_failed() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/api/auth/session");
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"error":"invalid_grant","error_description":"refresh token revoked"}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.refresh_token("revoked-rt").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CodexAdapterError::RefreshFailed(msg) => {
            assert!(
                msg.contains("401") || msg.contains("session refresh failed"),
                "error message should describe refresh failure: {msg}"
            );
        }
        other => panic!("expected RefreshFailed, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 5: 200 streaming SSE → bytes_stream returns the upstream bytes.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_stream_200_returns_upstream_bytes() {
    use futures_util::StreamExt;

    let server = MockServer::start();

    let sse_body = "data: {\"delta\":\"hello\"}\n\ndata: [DONE]\n\n";
    let _mock = server.mock(|when, then| {
        when.method(POST).path("/backend-api/codex/responses");
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(sse_body);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let stream_result = adapter.proxy_stream("stream-token", empty_request()).await;

    assert!(stream_result.is_ok(), "expected Ok from proxy_stream");
    let (status, _headers, mut stream) = stream_result.unwrap();
    assert_eq!(status, 200);

    let mut collected = Vec::new();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.expect("chunk read error");
        collected.extend_from_slice(&bytes);
    }
    assert_eq!(
        String::from_utf8_lossy(&collected),
        sse_body,
        "streamed bytes must match upstream body"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Hop-by-hop response headers stripped.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn response_hop_by_hop_headers_stripped() {
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/backend-api/codex/responses");
        then.status(200)
            .header("content-type", "application/json")
            .header("transfer-encoding", "chunked")
            .header("connection", "keep-alive")
            .header("keep-alive", "timeout=60")
            .header("x-custom-response", "present")
            .body(r#"{"id":"resp-hop"}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());
    let result = adapter.proxy("hop-token", empty_request()).await;

    assert!(result.is_ok());
    let resp = result.unwrap();

    for h in HOP_BY_HOP {
        assert!(
            !resp.headers.contains_key(*h),
            "hop-by-hop header '{h}' should have been stripped from response"
        );
    }

    assert!(
        resp.headers.contains_key("content-type"),
        "content-type should pass through the response filter"
    );
    assert!(
        resp.headers.contains_key("x-custom-response"),
        "custom non-hop header should pass through"
    );
}

#[tokio::test]
async fn request_hop_by_hop_headers_not_forwarded_upstream() {
    let server = MockServer::start();

    // If any hop-by-hop header reaches the server, it returns 500 (canary).
    let canaries: Vec<_> = HOP_BY_HOP
        .iter()
        .map(|h| {
            server.mock(|when, then| {
                when.method(POST)
                    .path("/backend-api/codex/responses")
                    .header_exists(*h);
                then.status(500).body("canary hit");
            })
        })
        .collect();

    let success_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/backend-api/codex/responses")
            .header_exists("x-safe-header");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"resp-hop-req"}"#);
    });

    let adapter = CodexAdapter::with_base_url(make_client(), server.base_url());

    let mut headers = BTreeMap::new();
    headers.insert("x-safe-header".to_string(), "keep-me".to_string());
    for h in HOP_BY_HOP {
        headers.insert(h.to_string(), "drop-me".to_string());
    }

    let req = CodexProxyRequest {
        body: b"{}".to_vec(),
        extra_headers: headers,
    };

    let result = adapter.proxy("hop-req-token", req).await;
    assert!(result.is_ok(), "proxy should succeed: {result:?}");

    success_mock.assert_hits(1);
    for (canary, h) in canaries.iter().zip(HOP_BY_HOP.iter()) {
        assert_eq!(
            canary.hits(),
            0,
            "hop-by-hop header '{h}' was forwarded upstream — must be stripped"
        );
    }
}
