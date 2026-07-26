//! Fix-5: Hop-by-hop header filter — RFC 7230 §6.1 headers must not be
//! forwarded upstream and must not be returned to the caller.
//!
//! Stage-6: AnthropicAdapter::proxy is now async and takes `&reqwest::Client`.
//! Tests updated to use `#[tokio::test]` and `.await`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use httpmock::prelude::*;
use intelligence_kernel::TenantId;
use intelligence_rest::{AnthropicAdapter, ProxyRequest, RestAdapterError, SecretProviderStore};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct StubStore;
impl SecretProviderStore for StubStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, String> {
        Box::pin(async { Ok("stub-rt".to_string()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

/// The full RFC 7230 §6.1 hop-by-hop set.
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

fn token_mock(server: &MockServer) -> httpmock::Mock<'_> {
    server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok-abc","refresh_token":"rt-new","expires_in":3600}"#);
    })
}

fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Upstream must NOT receive any RFC 7230 hop-by-hop headers that were
/// present in the inbound client request.
#[tokio::test]
async fn hop_by_hop_headers_not_forwarded_to_upstream() {
    let server = MockServer::start();
    let _tk = token_mock(&server);

    let canaries: Vec<_> = HOP_BY_HOP
        .iter()
        .map(|h| {
            server.mock(|when, then| {
                when.method(POST).path("/v1/messages").header_exists(*h);
                then.status(500).body("canary hit");
            })
        })
        .collect();

    let messages_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header_exists("x-custom-app");
        then.status(200)
            .header("content-type", "application/json")
            .header("transfer-encoding", "chunked")
            .header("connection", "keep-alive")
            .header("keep-alive", "timeout=60")
            .body(r#"{"id":"msg-1","type":"message"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url());
    let client = make_client();

    let mut headers = BTreeMap::new();
    headers.insert("x-custom-app".to_string(), "test-set".to_string());
    for h in HOP_BY_HOP {
        headers.insert(h.to_string(), "should-be-dropped".to_string());
    }
    headers.insert(
        "connection".to_string(),
        "keep-alive, x-nominated".to_string(),
    );
    headers.insert("x-nominated".to_string(), "strip-me".to_string());

    let req = ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#.to_vec(),
        tenant_id: TenantId::new("t-hop").unwrap(),
    };

    let resp = adapter
        .proxy(&client, &req, "t-hop/seat-hop")
        .await
        .unwrap();

    messages_mock.assert_hits(1);

    for (canary, header_name) in canaries.iter().zip(HOP_BY_HOP.iter()) {
        assert_eq!(
            canary.hits(),
            0,
            "hop-by-hop header '{header_name}' was forwarded to upstream — must be stripped"
        );
    }

    for h in HOP_BY_HOP {
        assert!(
            !resp.headers.contains_key(*h),
            "response header '{h}' should have been stripped but was present"
        );
    }

    assert!(
        resp.headers.contains_key("content-type"),
        "content-type should pass through the response filter"
    );
}

/// Connection-header-nominated tokens must also be stripped.
#[tokio::test]
async fn connection_nominated_header_stripped() {
    let server = MockServer::start();
    let _tk = token_mock(&server);

    let canary = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header_exists("x-nominated");
        then.status(500).body("canary nominated");
    });

    let _success = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-2"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url());
    let client = make_client();
    let mut headers = BTreeMap::new();
    headers.insert("connection".to_string(), "x-nominated".to_string());
    headers.insert("x-nominated".to_string(), "strip-me-too".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());

    let req = ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers,
        body: b"{}".to_vec(),
        tenant_id: TenantId::new("t-nominated").unwrap(),
    };

    let result = adapter.proxy(&client, &req, "t-nominated/seat-1").await;
    assert!(result.is_ok());
    assert_eq!(
        canary.hits(),
        0,
        "connection-nominated header must be stripped"
    );
}

/// Safe headers (non-hop-by-hop) must pass through to upstream.
#[tokio::test]
async fn safe_headers_are_forwarded() {
    let server = MockServer::start();
    let _tk = token_mock(&server);

    let messages_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header_exists("x-safe-header");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-safe"}"#);
    });

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url());
    let client = make_client();
    let mut headers = BTreeMap::new();
    headers.insert("x-safe-header".to_string(), "present".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());

    let req = ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers,
        body: b"{}".to_vec(),
        tenant_id: TenantId::new("t-safe").unwrap(),
    };

    let result = adapter.proxy(&client, &req, "t-safe/seat-safe").await;
    messages_mock.assert();
    assert!(result.is_ok());
}
