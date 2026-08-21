//! Fix-5: Hop-by-hop header filter — RFC 7230 §6.1 headers must not be
//! forwarded upstream and must not be returned to the caller.
//!
//! Stage-6: AnthropicAdapter::proxy is now async and takes `&reqwest::Client`.
//! Tests updated to use `#[tokio::test]` and `.await`.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). The canary mocks — a `header_exists(h)` mock returning 500, asserted at
//! `hits() == 0` — become DIRECT assertions on the recorded upstream request. That is
//! strictly stronger: a canary at zero hits only proves httpmock did not SELECT that
//! mock, which a matcher-precedence change could make vacuous, whereas the port reads
//! the header list that actually crossed the wire and asserts each name is absent.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use intelligence_kernel::TenantId;
use intelligence_rest::{AnthropicAdapter, ProxyRequest, RestAdapterError, SecretProviderStore};
use scripted_http_server::{Chunk, RecordedRequest, ScriptedResponse, ScriptedServer};

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

fn token_response() -> ScriptedResponse {
    ScriptedResponse::ok()
        .header("content-type", "application/json")
        .body(r#"{"access_token":"tok-abc","refresh_token":"rt-new","expires_in":3600}"#)
}

fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

/// The assertion the httpmock canaries stood in for: NONE of the hop-by-hop names,
/// and no `Connection`-nominated name, reached upstream.
fn assert_no_hop_by_hop_forwarded(request: &RecordedRequest) {
    for header in HOP_BY_HOP {
        // `connection` is special-cased below: reqwest's own HTTP/1.1 client sets it,
        // so its presence is the transport's, not a forwarded inbound value.
        if *header == "connection" {
            continue;
        }
        assert!(
            !request.has_header(header),
            "hop-by-hop header '{header}' was forwarded to upstream — must be stripped \
             (upstream saw: {:?})",
            request.headers
        );
    }
    // Whatever reqwest itself put on the wire, the inbound sentinel value must never
    // appear in it.
    assert!(
        !request
            .header_values("connection")
            .iter()
            .any(|value| value.contains("x-nominated") || value.contains("should-be-dropped")),
        "the inbound Connection value leaked upstream: {:?}",
        request.header_values("connection")
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Upstream must NOT receive any RFC 7230 hop-by-hop headers that were
/// present in the inbound client request.
#[tokio::test]
async fn hop_by_hop_headers_not_forwarded_to_upstream() {
    // The upstream response deliberately carries hop-by-hop headers AND a genuinely
    // chunked body, so the response-side filter has something real to strip.
    let server = ScriptedServer::start(vec![
        token_response(),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .header("connection", "keep-alive")
            .header("keep-alive", "timeout=60")
            .chunks(vec![Chunk::new(r#"{"id":"msg-1","type":"message"}"#)]),
    ]);

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url().to_owned());
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

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"],
        "expected exactly one token exchange then one upstream message call"
    );
    let upstream = &requests[1];

    assert_no_hop_by_hop_forwarded(upstream);
    assert!(
        !upstream.has_header("x-nominated"),
        "the Connection-nominated header 'x-nominated' was forwarded upstream"
    );
    // The non-hop-by-hop header must survive, or the filter is just dropping everything.
    assert_eq!(upstream.header("x-custom-app"), Some("test-set"));

    for h in HOP_BY_HOP {
        assert!(
            !resp.headers.contains_key(*h),
            "response header '{h}' should have been stripped but was present: {:?}",
            resp.headers
        );
    }

    assert!(
        resp.headers.contains_key("content-type"),
        "content-type should pass through the response filter"
    );
    assert_eq!(resp.body, br#"{"id":"msg-1","type":"message"}"#.to_vec());
}

/// Connection-header-nominated tokens must also be stripped.
#[tokio::test]
async fn connection_nominated_header_stripped() {
    let server = ScriptedServer::start(vec![
        token_response(),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-2"}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url().to_owned());
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
    assert!(result.is_ok(), "proxy failed: {result:?}");

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    let upstream = &requests[1];
    assert!(
        !upstream.has_header("x-nominated"),
        "connection-nominated header must be stripped (upstream saw: {:?})",
        upstream.headers
    );
    assert_no_hop_by_hop_forwarded(upstream);
}

/// Safe headers (non-hop-by-hop) must pass through to upstream.
#[tokio::test]
async fn safe_headers_are_forwarded() {
    let server = ScriptedServer::start(vec![
        token_response(),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-safe"}"#),
    ]);

    let adapter = AnthropicAdapter::with_base_url(StubStore, server.base_url().to_owned());
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
    assert!(result.is_ok(), "proxy failed: {result:?}");

    let requests = server.requests();
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"]
    );
    // `header_exists("x-safe-header")` selected the mock; here the VALUE is asserted too.
    assert_eq!(requests[1].header("x-safe-header"), Some("present"));
    assert_eq!(requests[1].header("content-type"), Some("application/json"));
}

// Keeps the `RestAdapterError` import meaningful under `unused_imports = "allow"`:
// the proxy signature these tests exercise returns it, and naming the type here means
// a change to that error surface breaks this file rather than passing silently.
#[allow(dead_code)]
fn _error_type_is_named(error: RestAdapterError) -> RestAdapterError {
    error
}
