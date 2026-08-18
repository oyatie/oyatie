use intelligence_claude_agent_sdk::parse_direct_connect_url;

#[cfg(feature = "network")]
use tokio::io::AsyncReadExt;

#[test]
fn parses_current_typescript_direct_connect_url_shapes() {
    let parsed = parse_direct_connect_url("cc://127.0.0.1:1455/sdk-token").unwrap();
    assert_eq!(parsed.server_url, "http://127.0.0.1:1455");
    assert_eq!(parsed.auth_token.as_deref(), Some("sdk-token"));

    let parsed = parse_direct_connect_url("cc://agent.local:8080").unwrap();
    assert_eq!(parsed.server_url, "http://agent.local:8080");
    assert_eq!(parsed.auth_token, None);

    let parsed = parse_direct_connect_url("agent.local:8080/sessions/ignored").unwrap();
    assert_eq!(parsed.server_url, "http://agent.local:8080");
    assert_eq!(parsed.auth_token, None);

    let parsed = parse_direct_connect_url("https://Example.COM:8443/path?ignored=true").unwrap();
    assert_eq!(parsed.server_url, "https://example.com:8443");
    assert_eq!(parsed.auth_token, None);
}

#[test]
fn rejects_unix_socket_direct_connect_urls_like_typescript() {
    let error = parse_direct_connect_url("cc+unix:///tmp/claude.sock").unwrap_err();
    assert_eq!(
        error.to_string(),
        "Unix socket connect (cc+unix://) is not supported by the SDK transport"
    );
    assert_eq!(error.code(), None);
}

#[cfg(feature = "network")]
#[allow(clippy::result_large_err)]
#[tokio::test]
async fn direct_connect_query_uses_current_http_and_websocket_protocol() {
    use futures::{SinkExt, StreamExt};
    use intelligence_claude_agent_sdk::{
        ClaudeAgentOptions, DirectConnectTransportOptions, Message, PermissionMode,
        query_direct_connect,
    };
    use serde_json::{Value, json};
    use tokio::{io::AsyncWriteExt, net::TcpListener};
    use tokio_tungstenite::{accept_hdr_async, tungstenite::Message as WsMessage};

    let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws_listener.local_addr().unwrap();
    let ws_url = format!("ws://{ws_addr}/sdk");

    let http_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http_addr = http_listener.local_addr().unwrap();
    let http_url = format!("http://{http_addr}");

    let http_task = tokio::spawn(async move {
        let (mut stream, _) = http_listener.accept().await.unwrap();
        let request = read_http_request(&mut stream).await;
        assert!(request.starts_with("POST /sessions HTTP/1.1"), "{request}");
        assert!(
            request.contains("Authorization: Bearer token-1")
                || request.contains("authorization: Bearer token-1"),
            "{request}"
        );
        assert!(request.contains("\"cwd\":\"/workspace\""), "{request}");
        assert!(request.contains("\"session_key\":\"key-1\""), "{request}");
        assert!(
            request.contains("\"permission_mode\":\"plan\""),
            "{request}"
        );
        let body = json!({
            "session_id": "session-1",
            "ws_url": ws_url,
            "work_dir": "/workspace"
        })
        .to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });

    let ws_task = tokio::spawn(async move {
        let (stream, _) = ws_listener.accept().await.unwrap();
        let mut ws = accept_hdr_async(
            stream,
            |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
                assert_eq!(
                    request
                        .headers()
                        .get("authorization")
                        .and_then(|value| value.to_str().ok()),
                    Some("Bearer token-1")
                );
                Ok(response)
            },
        )
        .await
        .unwrap();

        let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
        let initialize: Value = serde_json::from_str(first.trim()).unwrap();
        assert_eq!(initialize["type"], "control_request");
        assert_eq!(initialize["request"]["subtype"], "initialize");
        let request_id = initialize["request_id"].as_str().unwrap();
        ws.send(WsMessage::Text(
            format!(
                "{}\n",
                json!({
                    "type": "control_response",
                    "response": {
                        "subtype": "success",
                        "request_id": request_id,
                        "response": {}
                    }
                })
            )
            .into(),
        ))
        .await
        .unwrap();

        let second = ws.next().await.unwrap().unwrap().into_text().unwrap();
        let user: Value = serde_json::from_str(second.trim()).unwrap();
        assert_eq!(user["type"], "user");
        assert_eq!(user["message"]["content"], "hello direct");
        ws.send(WsMessage::Text(
            format!(
                "{}\n",
                json!({
                    "type": "result",
                    "subtype": "success",
                    "duration_ms": 1,
                    "duration_api_ms": 1,
                    "is_error": false,
                    "num_turns": 1,
                    "session_id": "session-1",
                    "total_cost_usd": 0.0,
                    "result": "ok"
                })
            )
            .into(),
        ))
        .await
        .unwrap();
        ws.close(None).await.unwrap();
    });

    let mut query = query_direct_connect(
        "hello direct",
        DirectConnectTransportOptions::new(http_url)
            .auth_token("token-1")
            .cwd("/workspace")
            .session_key("key-1")
            .permission_mode(PermissionMode::Plan),
        ClaudeAgentOptions::builder().build(),
    )
    .await
    .unwrap();

    let mut result_seen = false;
    while let Some(message) = query.next().await.transpose().unwrap() {
        if let Message::Result(result) = message {
            assert_eq!(result.session_id, "session-1");
            assert_eq!(result.result.as_deref(), Some("ok"));
            result_seen = true;
        }
    }
    assert!(result_seen);
    http_task.await.unwrap();
    ws_task.await.unwrap();
}

#[cfg(feature = "network")]
#[allow(clippy::result_large_err)]
#[tokio::test]
async fn websocket_query_sends_browser_auth_and_headers() {
    use futures::{SinkExt, StreamExt};
    use intelligence_claude_agent_sdk::{
        AuthMessage, ClaudeAgentOptions, Message, WebSocketOptions, query_websocket,
    };
    use serde_json::{Value, json};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_hdr_async, tungstenite::Message as WsMessage};

    let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws_listener.local_addr().unwrap();
    let ws_url = format!("ws://{ws_addr}/browser");

    let ws_task = tokio::spawn(async move {
        let (stream, _) = ws_listener.accept().await.unwrap();
        let mut ws = accept_hdr_async(
            stream,
            |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
                assert_eq!(
                    request
                        .headers()
                        .get("x-sdk-test")
                        .and_then(|value| value.to_str().ok()),
                    Some("yes")
                );
                Ok(response)
            },
        )
        .await
        .unwrap();

        let auth = ws.next().await.unwrap().unwrap().into_text().unwrap();
        let auth: Value = serde_json::from_str(auth.trim()).unwrap();
        assert_eq!(auth["type"], "auth");
        assert_eq!(auth["credential"]["token"], "oauth-1");

        let request_id = loop {
            let frame = ws.next().await.unwrap().unwrap().into_text().unwrap();
            let value: Value = serde_json::from_str(frame.trim()).unwrap();
            match value["type"].as_str() {
                Some("keep_alive") => continue,
                Some("control_request") => {
                    assert_eq!(value["request"]["subtype"], "initialize");
                    break value["request_id"].as_str().unwrap().to_owned();
                }
                other => panic!("unexpected frame before initialize: {other:?} {value}"),
            }
        };
        ws.send(WsMessage::Text(
            format!(
                "{}\n",
                json!({
                    "type": "control_response",
                    "response": {
                        "subtype": "success",
                        "request_id": request_id,
                        "response": {}
                    }
                })
            )
            .into(),
        ))
        .await
        .unwrap();

        loop {
            let frame = ws.next().await.unwrap().unwrap().into_text().unwrap();
            let value: Value = serde_json::from_str(frame.trim()).unwrap();
            if value["type"] == "keep_alive" {
                continue;
            }
            assert_eq!(value["type"], "user");
            assert_eq!(value["message"]["content"], "hello browser");
            break;
        }
        ws.send(WsMessage::Text(
            format!(
                "{}\n",
                json!({
                    "type": "result",
                    "subtype": "success",
                    "duration_ms": 1,
                    "duration_api_ms": 1,
                    "is_error": false,
                    "num_turns": 1,
                    "session_id": "browser-session-1",
                    "total_cost_usd": 0.0,
                    "result": "browser-ok"
                })
            )
            .into(),
        ))
        .await
        .unwrap();
        ws.close(None).await.unwrap();
    });

    let mut query = query_websocket(
        "hello browser",
        WebSocketOptions::new(ws_url)
            .header("x-sdk-test", "yes")
            .auth_message(AuthMessage::oauth("oauth-1"))
            .keep_alive_interval_ms(60_000),
        ClaudeAgentOptions::builder().build(),
    )
    .await
    .unwrap();

    let mut result_seen = false;
    while let Some(message) = query.next().await.transpose().unwrap() {
        if let Message::Result(result) = message {
            assert_eq!(result.session_id, "browser-session-1");
            assert_eq!(result.result.as_deref(), Some("browser-ok"));
            result_seen = true;
        }
    }
    assert!(result_seen);
    ws_task.await.unwrap();
}

#[cfg(feature = "network")]
async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut temp = [0; 1024];
    loop {
        let read = stream.read(&mut temp).await.unwrap();
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(header_end) = find_subslice(&buffer, b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&buffer[..header_end + 4]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())?
                })
                .unwrap_or(0);
            while buffer.len().saturating_sub(header_end + 4) < content_length {
                let read = stream.read(&mut temp).await.unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp[..read]);
            }
            break;
        }
    }
    String::from_utf8_lossy(&buffer).into_owned()
}

#[cfg(feature = "network")]
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
