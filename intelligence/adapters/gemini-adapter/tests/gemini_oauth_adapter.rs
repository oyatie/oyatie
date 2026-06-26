//! Hermetic integration tests for the Gemini subscription-OAuth (Antigravity)
//! adapter. Every upstream (token endpoint, Code Assist data plane,
//! `onboardUser` daily host) is a loopback fake — zero real network.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt as _;
use intelligence_gemini_adapter::{GeminiOAuthAdapter, GeminiOAuthError};
use intelligence_gemini_adapter::GeminiProxyRequest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Spawn a loopback HTTP server that records every request line/headers/body it
/// receives and replies with `response` to each connection. Returns the base
/// URL and the shared request log. The accept loop runs until the process exits
/// (the test drops it).
async fn recording_server(response: String) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake google endpoint");
    let addr = listener.local_addr().expect("fake endpoint addr");
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_writer = Arc::clone(&log);
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };
            let mut buf = vec![0_u8; 64 * 1024];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            log_writer.lock().unwrap().push(request);
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });
    (format!("http://{addr}"), log)
}

fn http(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn assert_header(request: &str, header: &str, value: &str) {
    let needle = format!("{header}: {value}");
    assert!(
        request.lines().any(|line| line.eq_ignore_ascii_case(&needle)),
        "missing header `{needle}` in request:\n{request}"
    );
}

fn client() -> Arc<reqwest::Client> {
    Arc::new(reqwest::Client::new())
}

#[tokio::test]
async fn refresh_uses_form_encoded_confidential_client_grant() {
    let (token_url, log) = recording_server(http(
        "200 OK",
        "application/json",
        r#"{"access_token":"at-abc","expires_in":3600}"#,
    ))
    .await;
    let adapter = GeminiOAuthAdapter::with_endpoints(
        client(),
        token_url,
        "http://unused.data",
        "http://unused.onboard",
    );

    let token = adapter
        .refresh_token("tenant-1/seat-1", "rt-secret", "client-secret-from-bao")
        .await
        .expect("refresh succeeds");
    assert_eq!(token, "at-abc");

    let reqs = log.lock().unwrap();
    assert_eq!(reqs.len(), 1, "exactly one upstream token exchange");
    let req = &reqs[0];
    assert_header(req, "content-type", "application/x-www-form-urlencoded");
    assert!(req.contains("grant_type=refresh_token"), "grant_type:\n{req}");
    assert!(req.contains("refresh_token=rt-secret"), "refresh token in body");
    assert!(
        req.contains("client_secret=client-secret-from-bao"),
        "client_secret in body (form), not header"
    );
    assert!(
        req.contains("client_id=1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com"),
        "public client_id present"
    );
}

#[tokio::test]
async fn refresh_caches_token_within_expiry_lead() {
    let (token_url, log) = recording_server(http(
        "200 OK",
        "application/json",
        r#"{"access_token":"at-cached","expires_in":3600}"#,
    ))
    .await;
    let adapter = GeminiOAuthAdapter::with_endpoints(
        client(),
        token_url,
        "http://unused.data",
        "http://unused.onboard",
    );

    let t1 = adapter
        .refresh_token("h", "rt", "cs")
        .await
        .expect("first refresh");
    let t2 = adapter
        .refresh_token("h", "rt", "cs")
        .await
        .expect("second refresh served from cache");
    assert_eq!(t1, "at-cached");
    assert_eq!(t2, "at-cached");
    assert_eq!(
        log.lock().unwrap().len(),
        1,
        "second call must be served from cache (no upstream hit)"
    );
}

#[tokio::test]
async fn refresh_fails_closed_on_rejected_grant() {
    let (token_url, _log) = recording_server(http(
        "400 Bad Request",
        "application/json",
        r#"{"error":"invalid_grant"}"#,
    ))
    .await;
    let adapter = GeminiOAuthAdapter::with_endpoints(
        client(),
        token_url,
        "http://unused.data",
        "http://unused.onboard",
    );

    let err = adapter
        .refresh_token("h", "rt", "cs")
        .await
        .expect_err("rejected grant must fail closed");
    assert!(matches!(err, GeminiOAuthError::RefreshFailed(_)), "{err:?}");
}

#[tokio::test]
async fn resolve_project_via_load_code_assist_sends_antigravity_idetype_in_body() {
    let (data_base, data_log) = recording_server(http(
        "200 OK",
        "application/json",
        r#"{"cloudaicompanionProject":"projects/acme-42"}"#,
    ))
    .await;
    let (onboard_base, onboard_log) =
        recording_server(http("200 OK", "application/json", r#"{}"#)).await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, onboard_base);

    let project = adapter
        .resolve_project("h", "at-xyz")
        .await
        .expect("project resolves via loadCodeAssist");
    assert_eq!(project, "projects/acme-42");

    let reqs = data_log.lock().unwrap();
    assert_eq!(reqs.len(), 1);
    let req = &reqs[0];
    assert!(
        req.starts_with("POST /v1internal:loadCodeAssist "),
        "wrong method path:\n{req}"
    );
    assert_header(req, "authorization", "Bearer at-xyz");
    assert!(req.contains("ANTIGRAVITY"), "ideType in body:\n{req}");
    assert!(
        onboard_log.lock().unwrap().is_empty(),
        "onboardUser must NOT be hit when loadCodeAssist resolves"
    );
}

#[tokio::test]
async fn resolve_project_falls_back_to_onboard_user() {
    // loadCodeAssist returns 200 but no project -> fallback to onboardUser.
    let (data_base, _data_log) =
        recording_server(http("200 OK", "application/json", r#"{"other":"x"}"#)).await;
    let (onboard_base, onboard_log) = recording_server(http(
        "200 OK",
        "application/json",
        r#"{"done":true,"response":{"cloudaicompanionProject":{"id":"proj-onboarded"}}}"#,
    ))
    .await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, onboard_base);

    let project = adapter
        .resolve_project("h", "at-xyz")
        .await
        .expect("project resolves via onboardUser fallback");
    assert_eq!(project, "proj-onboarded");

    let reqs = onboard_log.lock().unwrap();
    assert_eq!(reqs.len(), 1);
    assert!(
        reqs[0].starts_with("POST /v1internal:onboardUser "),
        "wrong fallback method path:\n{}",
        reqs[0]
    );
}

#[tokio::test]
async fn resolve_project_fails_closed_when_neither_yields_project() {
    let (data_base, _d) = recording_server(http("404 Not Found", "application/json", "{}")).await;
    let (onboard_base, _o) =
        recording_server(http("200 OK", "application/json", r#"{"nope":true}"#)).await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, onboard_base);

    let err = adapter
        .resolve_project("h", "at-xyz")
        .await
        .expect_err("must fail closed");
    assert!(
        matches!(err, GeminiOAuthError::ProjectResolutionFailed(_)),
        "{err:?}"
    );
}

#[tokio::test]
async fn proxy_injects_bearer_and_strips_credential_and_hop_headers() {
    let response = "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Connection: x-upstream-hop\r\n\
         X-Upstream-Hop: remove-me\r\n\
         Content-Length: 11\r\n\
         \r\n\
         {\"ok\":true}"
        .to_string();
    let (data_base, log) = recording_server(response).await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, "http://unused.onboard");

    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), "Bearer caller-token".to_string());
    headers.insert("connection".to_string(), "x-drop-me".to_string());
    headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());
    headers.insert("host".to_string(), "attacker.example".to_string());
    headers.insert("x-goog-api-key".to_string(), "leak-key".to_string());

    let resp = adapter
        .proxy(
            "oauth-access-token",
            GeminiProxyRequest {
                body: br#"{"model":"gemini-test"}"#.to_vec(),
                extra_headers: headers,
            },
        )
        .await
        .expect("proxy succeeds");

    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, br#"{"ok":true}"#);
    assert!(!resp.headers.contains_key("connection"), "hop-by-hop leaked");
    assert!(!resp.headers.contains_key("x-upstream-hop"));

    let reqs = log.lock().unwrap();
    let req = &reqs[0];
    assert!(
        req.starts_with("POST /v1internal:generateContent "),
        "wrong data path:\n{req}"
    );
    assert_header(req, "authorization", "Bearer oauth-access-token");
    assert!(!req.contains("caller-token"), "caller bearer leaked");
    assert!(!req.contains("must-not-forward"), "connection-nominated header leaked");
    assert!(!req.contains("attacker.example"), "host leaked");
    assert!(!req.to_ascii_lowercase().contains("x-goog-api-key:"), "api-key header leaked");
}

#[tokio::test]
async fn proxy_maps_retry_after_rate_limit() {
    let (data_base, _log) = recording_server(
        "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 23\r\nContent-Length: 0\r\n\r\n".to_string(),
    )
    .await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, "http://unused.onboard");

    let err = adapter
        .proxy(
            "at",
            GeminiProxyRequest {
                body: b"{}".to_vec(),
                extra_headers: BTreeMap::new(),
            },
        )
        .await
        .expect_err("429 maps to rate limit");
    assert_eq!(
        err,
        GeminiOAuthError::RateLimited {
            retry_after_secs: Some(23)
        }
    );
}

#[tokio::test]
async fn proxy_stream_forces_sse_accept_and_streams_bytes() {
    let response = "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream\r\n\
         Content-Length: 10\r\n\
         \r\n\
         data: hi\n\n"
        .to_string();
    let (data_base, log) = recording_server(response).await;
    let adapter =
        GeminiOAuthAdapter::with_endpoints(client(), "http://unused.token", data_base, "http://unused.onboard");

    let mut headers = BTreeMap::new();
    headers.insert("accept".to_string(), "application/json".to_string());

    let (status, headers_out, stream) = adapter
        .proxy_stream(
            "at",
            GeminiProxyRequest {
                body: br#"{"model":"gemini-test","stream":true}"#.to_vec(),
                extra_headers: headers,
            },
        )
        .await
        .expect("stream opens");

    assert_eq!(status, 200);
    assert_eq!(
        headers_out.get("content-type").map(String::as_str),
        Some("text/event-stream")
    );
    let body = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|c| c.expect("chunk"))
        .fold(Vec::new(), |mut acc, b| {
            acc.extend_from_slice(&b);
            acc
        });
    assert_eq!(body, b"data: hi\n\n");

    let reqs = log.lock().unwrap();
    let req = &reqs[0];
    assert!(
        req.starts_with("POST /v1internal:streamGenerateContent?alt=sse "),
        "wrong stream path:\n{req}"
    );
    assert_header(req, "accept", "text/event-stream");
    assert!(!req.contains("application/json"), "caller accept not overridden");
}
