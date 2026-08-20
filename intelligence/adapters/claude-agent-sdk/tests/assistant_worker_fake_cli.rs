//! Hermetic assistant-worker test: initial-prompt output forwarding to the bridge.
//!
//! Replaces the previous on-disk fake-CLI script (an external interpreter the
//! SDK transport spawned) with the same in-process Rust fake the sibling
//! claude-agent-sdk tests use, driven through the SDK's
//! `spawn_claude_code_process` hook (`support_fake_cli.rs`). This removes the
//! external-runtime dependency, the on-disk script + chmod dance, and the
//! wall-clock deadline flakiness, while still exercising the real
//! `run_assistant_worker` initial-prompt -> bridge forwarding path: the worker
//! drives Claude through the fake CLI and emits the assistant/result envelopes to
//! a localhost bridge whose request sequence the test asserts.
//!
//! Network feature gated, matching the production `run_assistant_worker` cfg.
#![cfg(feature = "network")]

#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, write_json_line};

use intelligence_claude_agent_sdk::{
    AssistantWorkerOptions, AttachBridgeSessionOptions, ClaudeAgentOptions, run_assistant_worker,
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

#[tokio::test]
async fn run_assistant_worker_forwards_initial_prompt_output_to_bridge() {
    // In-process fake Claude CLI: mirrors the old fake script's
    // read-init / ack / read-user / emit-assistant / emit-result sequence, but
    // runs as a Rust async closure over the SDK's duplex stdin/stdout instead of
    // a spawned external-interpreter subprocess.
    let spawn = fake_cli(|mut reader, mut writer, _options| async move {
        let init = expect_json_line(&mut reader).await;
        assert_eq!(init["type"], "control_request");
        assert_eq!(init["request"]["subtype"], "initialize");
        write_json_line(
            &mut writer,
            &json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": init["request_id"],
                    "response": {"ok": true}
                }
            }),
        )
        .await;

        let user = expect_json_line(&mut reader).await;
        assert_eq!(user["type"], "user");
        assert_eq!(user["message"]["content"], "start");

        write_json_line(
            &mut writer,
            &json!({
                "type": "assistant",
                "session_id": "claude-session-1",
                "uuid": "assistant-uuid",
                "message": {
                    "id": "msg_1",
                    "model": "claude-test",
                    "content": [{"type": "text", "text": "done"}]
                }
            }),
        )
        .await;
        write_json_line(
            &mut writer,
            &json!({
                "type": "result",
                "subtype": "success",
                "duration_ms": 1,
                "duration_api_ms": 1,
                "is_error": false,
                "num_turns": 1,
                "session_id": "claude-session-1",
                "result": "done"
            }),
        )
        .await;
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        for step in 0..5 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_http_request(&mut stream).await;
            match step {
                0 => {
                    assert!(
                        request.starts_with("PUT /v1/code/sessions/cse_worker/worker HTTP/1.1"),
                        "{request}"
                    );
                }
                1 => {
                    let body = request_json_body(&request);
                    assert_eq!(body["worker_status"], "running");
                }
                2 => {
                    let payload = request_json_body(&request)["events"][0]["payload"].clone();
                    assert_eq!(payload["type"], "assistant");
                    assert_eq!(payload["session_id"], "claude-session-1");
                }
                3 => {
                    let payload = request_json_body(&request)["events"][0]["payload"].clone();
                    assert_eq!(payload["type"], "result");
                    assert_eq!(payload["session_id"], "claude-session-1");
                }
                4 => {
                    let body = request_json_body(&request);
                    assert_eq!(body["worker_status"], "idle");
                }
                _ => unreachable!(),
            }
            write_http_json(&mut stream, "200 OK", "{}").await;
        }
    });

    let options = AssistantWorkerOptions::new(
        AttachBridgeSessionOptions::new("cse_worker", "worker-jwt", base_url)
            .epoch(3)
            .outbound_only(true)
            .heartbeat_interval_ms(60_000),
        ClaudeAgentOptions::builder()
            .spawn_claude_code_process(spawn)
            .build(),
    )
    .initial_prompt("start");
    let mut handle = run_assistant_worker(options).await.unwrap();
    server.await.unwrap();
    assert_eq!(
        handle.claude_session_id().await.as_deref(),
        Some("claude-session-1")
    );
    handle.teardown().await.unwrap();
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
    use tokio::io::AsyncReadExt;

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

async fn write_http_json(stream: &mut tokio::net::TcpStream, status: &str, body: &str) {
    use tokio::io::AsyncWriteExt;

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await.unwrap();
}

fn request_json_body(request: &str) -> Value {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    serde_json::from_str(body).unwrap_or_else(|error| {
        panic!("failed to parse request body as JSON: {error}; request={request}")
    })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
