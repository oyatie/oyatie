#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt as _;
use intelligence_gemini_adapter::{GeminiAdapterError, GeminiApiKeyAdapter, GeminiProxyRequest};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn one_shot_http_server(response: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake provider");
    let addr = listener.local_addr().expect("fake provider addr");
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept fake provider");
        let mut buf = vec![0_u8; 16 * 1024];
        let n = socket.read(&mut buf).await.expect("read fake request");
        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write fake response");
        request
    });
    (format!("http://{addr}/v1beta"), handle)
}

fn assert_header(request: &str, header: &str, value: &str) {
    let needle = format!("{header}: {value}");
    assert!(
        request
            .lines()
            .any(|line| line.eq_ignore_ascii_case(&needle)),
        "missing header `{needle}` in request:\n{request}"
    );
}

#[tokio::test]
async fn gemini_generate_content_injects_api_key_header_and_strips_caller_credentials() {
    let (base_url, upstream_request) = one_shot_http_server(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Connection: x-upstream-hop\r\n\
         X-Upstream-Hop: remove-me\r\n\
         Content-Length: 11\r\n\
         \r\n\
         {\"ok\":true}",
    )
    .await;
    let adapter = GeminiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        "Bearer caller-token".to_string(),
    );
    headers.insert(
        "x-goog-api-key".to_string(),
        "caller-controlled-key".to_string(),
    );
    headers.insert("connection".to_string(), "x-drop-me".to_string());
    headers.insert("x-drop-me".to_string(), "must-not-forward".to_string());

    let response = adapter
        .proxy_generate_content(
            "provider-api-key",
            "gemini-2.5-flash",
            GeminiProxyRequest {
                body: br#"{"contents":[{"role":"user","parts":[{"text":"hello"}]}]}"#.to_vec(),
                extra_headers: headers,
            },
        )
        .await
        .expect("Gemini generateContent proxy succeeds");

    assert_eq!(response.status, 200);
    assert_eq!(response.body, br#"{"ok":true}"#);
    assert!(!response.headers.contains_key("connection"));
    assert!(!response.headers.contains_key("x-upstream-hop"));

    let request = upstream_request.await.expect("fake provider request");
    assert!(
        request.starts_with("POST /v1beta/models/gemini-2.5-flash:generateContent "),
        "unexpected request path:\n{request}"
    );
    assert_header(&request, "x-goog-api-key", "provider-api-key");
    assert!(!request.contains("caller-token"));
    assert!(!request.contains("caller-controlled-key"));
    assert!(!request.contains("must-not-forward"));
}

#[tokio::test]
async fn gemini_stream_generate_content_uses_alt_sse_and_keeps_raw_stream_bytes() {
    let (base_url, upstream_request) = one_shot_http_server(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream\r\n\
         Content-Length: 10\r\n\
         \r\n\
         data: hi\n\n",
    )
    .await;
    let adapter = GeminiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

    let (status, _headers, stream): (
        u16,
        BTreeMap<String, String>,
        Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static>>,
    ) = adapter
        .proxy_stream_generate_content(
            "provider-api-key",
            "gemini-2.5-flash",
            GeminiProxyRequest {
                body: br#"{"contents":[{"role":"user","parts":[{"text":"hello"}]}]}"#.to_vec(),
                extra_headers: BTreeMap::new(),
            },
        )
        .await
        .expect("Gemini stream proxy opens");

    assert_eq!(status, 200);
    let body = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|chunk| chunk.expect("stream chunk"))
        .fold(Vec::new(), |mut acc, bytes| {
            acc.extend_from_slice(&bytes);
            acc
        });
    assert_eq!(body, b"data: hi\n\n");

    let request = upstream_request.await.expect("fake provider request");
    assert!(
        request.starts_with("POST /v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse "),
        "unexpected request path:\n{request}"
    );
    assert_header(&request, "x-goog-api-key", "provider-api-key");
    assert_header(&request, "accept", "text/event-stream");
}

#[tokio::test]
async fn gemini_rate_limit_error_preserves_retry_after_seconds() {
    let (base_url, _upstream_request) = one_shot_http_server(
        "HTTP/1.1 429 Too Many Requests\r\n\
         Retry-After: 7\r\n\
         Content-Length: 0\r\n\
         \r\n",
    )
    .await;
    let adapter = GeminiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);

    let error = adapter
        .proxy_generate_content(
            "provider-api-key",
            "gemini-2.5-flash",
            GeminiProxyRequest {
                body: br#"{"contents":[]}"#.to_vec(),
                extra_headers: BTreeMap::new(),
            },
        )
        .await
        .expect_err("429 should map to RateLimited");

    assert_eq!(
        error,
        GeminiAdapterError::RateLimited {
            retry_after_secs: Some(7)
        }
    );
}

#[tokio::test]
async fn xproxy_wire_001_gemini_adapter_translates_openai_chat_request_and_response_at_adapter_boundary()
 {
    let upstream_body = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"ok"}]}}],"usageMetadata":{"promptTokenCount":2,"candidatesTokenCount":3}}"#;
    let (base_url, upstream_request) = one_shot_http_server(Box::leak(
        format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            upstream_body.len(),
            upstream_body
        )
        .into_boxed_str(),
    ))
    .await;
    let adapter = GeminiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);
    let request_body = br#"{"model":"gemini:gemini-2.5-flash","messages":[{"role":"system","content":"Be brief"},{"role":"user","content":"hello"}]}"#;

    let response = adapter
        .proxy_openai_chat("provider-api-key", request_body.to_vec(), BTreeMap::new())
        .await
        .expect("adapter owns OpenAI-to-Gemini translation");

    assert_eq!(response.status, 200);
    let response_json: serde_json::Value =
        serde_json::from_slice(&response.body).expect("OpenAI-compatible response");
    assert_eq!(response_json["object"], "chat.completion");
    assert_eq!(
        response_json["choices"][0]["message"]["content"],
        serde_json::json!("ok")
    );
    assert_eq!(response_json["usage"]["total_tokens"], serde_json::json!(5));

    let request = upstream_request.await.expect("fake provider request");
    assert!(
        request.starts_with("POST /v1beta/models/gemini-2.5-flash:generateContent "),
        "unexpected request path:\n{request}"
    );
    assert!(request.contains("\"contents\""));
    assert!(request.contains("\"systemInstruction\""));
    assert!(!request.contains("\"messages\""));
}

#[tokio::test]
async fn xproxy_wire_001_gemini_adapter_translates_anthropic_messages_request_and_response_at_adapter_boundary()
 {
    let upstream_body = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"pong"}]}}],"usageMetadata":{"promptTokenCount":4,"candidatesTokenCount":6}}"#;
    let (base_url, upstream_request) = one_shot_http_server(Box::leak(
        format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            upstream_body.len(),
            upstream_body
        )
        .into_boxed_str(),
    ))
    .await;
    let adapter = GeminiApiKeyAdapter::with_base_url(Arc::new(reqwest::Client::new()), base_url);
    let request_body = br#"{"model":"gemini-2.5-pro","system":"Be exact","max_tokens":64,"messages":[{"role":"user","content":"ping"}]}"#;

    let response = adapter
        .proxy_anthropic_messages("provider-api-key", request_body.to_vec(), BTreeMap::new())
        .await
        .expect("adapter owns Anthropic-to-Gemini translation");

    assert_eq!(response.status, 200);
    let response_json: serde_json::Value =
        serde_json::from_slice(&response.body).expect("Anthropic-compatible response");
    assert_eq!(response_json["type"], "message");
    assert_eq!(
        response_json["content"][0]["text"],
        serde_json::json!("pong")
    );
    assert_eq!(response_json["usage"]["input_tokens"], serde_json::json!(4));
    assert_eq!(
        response_json["usage"]["output_tokens"],
        serde_json::json!(6)
    );

    let request = upstream_request.await.expect("fake provider request");
    assert!(
        request.starts_with("POST /v1beta/models/gemini-2.5-pro:generateContent "),
        "unexpected request path:\n{request}"
    );
    assert!(request.contains("\"contents\""));
    assert!(request.contains("\"systemInstruction\""));
    assert!(!request.contains("\"messages\""));
}
