use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

use intelligence_claude_agent_sdk::{ClaudeAgentOptions, Message, UserMessage, startup, startup_with_timeout};
use futures::{StreamExt, stream};
use tempfile::tempdir;

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[tokio::test]
async fn startup_prewarms_before_prompt_and_streams_once_ready() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-startup.py");
    let initialized = dir.path().join("initialized.txt");
    write_executable(
        &script,
        r#"#!/usr/bin/env python3
import json, os, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
with open(os.environ["INITIALIZED_MARKER"], "w") as marker:
    marker.write("ready\n")
    marker.flush()
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type": "assistant",
  "session_id": "session-warm",
  "message": {"model": "claude-test", "content": [{"type": "text", "text": user["message"]["content"]}]}
}), flush=True)
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-warm",
  "result": user["message"]["content"]
}), flush=True)
"#,
    );

    let mut options = ClaudeAgentOptions::builder().cli_path(&script).build();
    options.env.insert(
        "INITIALIZED_MARKER".into(),
        initialized.to_string_lossy().into_owned(),
    );
    let mut warm = startup(options).await.unwrap();

    assert_eq!(fs::read_to_string(initialized).unwrap(), "ready\n");

    let mut stream = warm.query("hello").unwrap();
    let first = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(first, Message::Assistant(message) if message.session_id.as_deref() == Some("session-warm"))
    );
    let second = stream.next().await.unwrap().unwrap();
    assert!(matches!(second, Message::Result(result) if result.result.as_deref() == Some("hello")));
}

#[tokio::test]
async fn warm_query_can_only_be_used_once() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-startup-once.py");
    write_executable(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
user = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-once",
  "result": user["message"]["content"]
}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut warm = startup(options).await.unwrap();
    let mut stream = warm.query("first").unwrap();

    let error = match warm.query("second") {
        Ok(_) => panic!("expected second warm query to fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("can only be called once"));

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("first"))
    );
}

#[tokio::test]
async fn warm_query_accepts_streaming_user_messages() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-startup-stream.py");
    write_executable(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
first = json.loads(sys.stdin.readline())
second = json.loads(sys.stdin.readline())
assert first["message"]["content"] == "warm context"
assert first["shouldQuery"] == False
assert second["message"]["content"] == "warm prompt"
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-warm-stream",
  "result": second["message"]["content"]
}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut warm = startup(options).await.unwrap();
    let mut stream = warm
        .query_stream(stream::iter([
            UserMessage::text("warm context").should_query(false),
            UserMessage::text("warm prompt"),
        ]))
        .unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("warm prompt"))
    );
}

#[tokio::test]
async fn warm_query_close_closes_stdin_without_prompt() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-startup-close.py");
    let closed = dir.path().join("closed.txt");
    write_executable(
        &script,
        r#"#!/usr/bin/env python3
import json, os, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
for _ in sys.stdin:
    pass
open(os.environ["CLOSED_MARKER"], "w").write("closed\n")
"#,
    );

    let mut options = ClaudeAgentOptions::builder().cli_path(&script).build();
    options.env.insert(
        "CLOSED_MARKER".into(),
        closed.to_string_lossy().into_owned(),
    );
    let mut warm = startup(options).await.unwrap();

    warm.close().await.unwrap();

    assert_eq!(fs::read_to_string(closed).unwrap(), "closed\n");
}

#[tokio::test]
async fn startup_honors_initialize_timeout() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-startup-timeout.py");
    write_executable(
        &script,
        r#"#!/usr/bin/env python3
import json, sys, time
json.loads(sys.stdin.readline())
time.sleep(1)
"#,
    );

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let started = Instant::now();
    let error = match startup_with_timeout(options, Duration::from_millis(20)).await {
        Ok(_) => panic!("expected startup to time out"),
        Err(error) => error.to_string(),
    };

    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(error.contains("initialize"));
}
