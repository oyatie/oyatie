use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use oya_cloud_intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, McpServerConfig, McpServerPermissionPolicy, McpServerToolPolicy, Message,
    ProcessSpawnOptions, SpawnedClaudeProcess, UserMessage, query, query_stream,
};
use futures::{StreamExt, channel::mpsc as futures_mpsc, stream};
use serde_json::json;
use tempfile::tempdir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::oneshot,
    time::{sleep, timeout},
};

#[tokio::test]
async fn query_streams_messages_from_fake_cli() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type": "assistant",
  "session_id": "session-1",
  "uuid": "assistant-uuid",
  "message": {
    "id": "msg_1",
    "model": "claude-test",
    "content": [{"type": "text", "text": "hello " + user["message"]["content"]}]
  }
}), flush=True)
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 5,
  "duration_api_ms": 4,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-1",
  "result": "done"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut stream = query("world", options).unwrap();

    assert_eq!(stream.initialization_result().await.unwrap()["ok"], true);

    let first = stream.next().await.unwrap().unwrap();
    match first {
        Message::Assistant(message) => {
            assert_eq!(message.model, "claude-test");
            assert_eq!(message.session_id.as_deref(), Some("session-1"));
        }
        other => panic!("expected assistant, got {other:?}"),
    }

    let second = stream.next().await.unwrap().unwrap();
    match second {
        Message::Result(result) => assert_eq!(result.result.as_deref(), Some("done")),
        other => panic!("expected result, got {other:?}"),
    }

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_stream_sends_sdk_user_messages_from_streaming_input() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-stream-input.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
context = json.loads(sys.stdin.readline())
question = json.loads(sys.stdin.readline())
assert context["type"] == "user"
assert context["message"]["role"] == "user"
assert context["message"]["content"] == "context first"
assert context["shouldQuery"] == False
assert question["message"]["content"] == "ask now"
assert question["parent_tool_use_id"] == "toolu_parent"
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-stream-input",
  "result": context["message"]["content"] + "|" + question["message"]["content"]
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let input = stream::iter([
        UserMessage::text("context first").should_query(false),
        UserMessage::text("ask now").parent_tool_use_id("toolu_parent"),
    ]);
    let mut stream = query_stream(input, options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("context first|ask now"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_stream_exposes_package_exported_control_requests() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-query-controls.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
seen_user = False
seen_controls = []
while len(seen_controls) < 11:
    line = sys.stdin.readline()
    assert line, "expected SDK input"
    message = json.loads(line)
    if message["type"] == "user":
        assert message["message"]["content"] == "control context"
        seen_user = True
        continue
    assert message["type"] == "control_request"
    request = message["request"]
    seen_controls.append(request["subtype"])
    if request["subtype"] == "set_model":
        assert request["model"] == "claude-test"
        response = {}
    elif request["subtype"] == "mcp_status":
        response = {"mcpServers": [{"name": "filesystem", "status": "connected"}]}
    elif request["subtype"] == "mcp_set_servers":
        assert request["servers"]["filesystem"]["type"] == "stdio"
        assert request["servers"]["filesystem"]["command"] == "fs-mcp"
        assert request["servers"]["filesystem"]["timeout"] == 2500
        assert request["servers"]["filesystem"]["alwaysLoad"] == True
        assert request["servers"]["docs"]["type"] == "http"
        assert request["servers"]["docs"]["url"] == "https://mcp.example/http"
        assert request["servers"]["docs"]["tools"][0]["name"] == "read_docs"
        assert request["servers"]["docs"]["tools"][0]["permission_policy"] == "always_allow"
        assert request["servers"]["docs"]["timeout"] == 5000
        assert request["servers"]["docs"]["alwaysLoad"] == False
        response = {"added": ["filesystem", "docs"], "removed": [], "errors": {}}
    elif request["subtype"] == "rename_session":
        assert request["title"] == "Control title"
        response = {}
    elif request["subtype"] == "set_color":
        assert request["color"] == "blue"
        response = {}
    elif request["subtype"] == "file_suggestions":
        assert request["query"] == "@src/"
        response = {"suggestions": [{"path": "src/lib.rs"}]}
    elif request["subtype"] == "get_binary_version":
        response = {"version": "1.2.3"}
    elif request["subtype"] == "get_session_cost":
        response = {"summary": "$0.01"}
    elif request["subtype"] == "mcp_call":
        assert request["tool"] == "mcp__filesystem__read"
        assert request["arguments"]["path"] == "README.md"
        response = {"content": [{"type": "text", "text": "ok"}]}
    elif request["subtype"] == "mcp_message":
        assert request["server_name"] == "filesystem"
        assert request["message"]["method"] == "ping"
        response = {"mcp_response": {"jsonrpc": "2.0", "result": {}, "id": request["message"]["id"]}}
    else:
        raise AssertionError(f"unexpected control subtype {request['subtype']}")
    print(json.dumps({
      "type": "control_response",
      "response": {
        "subtype": "success",
        "request_id": message["request_id"],
        "response": response
      }
    }), flush=True)
assert seen_user
assert seen_controls == ["set_model", "mcp_status", "mcp_set_servers", "mcp_set_servers", "rename_session", "set_color", "file_suggestions", "get_binary_version", "get_session_cost", "mcp_call", "mcp_message"]
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-query-controls",
  "result": "controlled"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let (mut input_tx, input_rx) = futures_mpsc::unbounded();
    let mut stream = query_stream(input_rx, options).unwrap();

    input_tx
        .start_send(UserMessage::text("control context"))
        .unwrap();
    stream.set_model(Some("claude-test")).await.unwrap();
    let statuses = stream.mcp_server_status().await.unwrap();
    assert_eq!(statuses[0].name, "filesystem");
    assert!(statuses[0].status.is_connected());
    let mut servers = BTreeMap::new();
    servers.insert(
        "filesystem".to_owned(),
        McpServerConfig::Stdio {
            command: "fs-mcp".to_owned(),
            args: Vec::new(),
            env: BTreeMap::new(),
            timeout: Some(2500),
            always_load: Some(true),
        },
    );
    servers.insert(
        "docs".to_owned(),
        McpServerConfig::Http {
            url: "https://mcp.example/http".to_owned(),
            headers: BTreeMap::new(),
            tools: vec![McpServerToolPolicy {
                name: "read_docs".to_owned(),
                permission_policy: McpServerPermissionPolicy::AlwaysAllow,
            }],
            timeout: Some(5000),
            always_load: Some(false),
        },
    );
    let set_servers = stream.set_mcp_servers(&servers).await.unwrap();
    assert_eq!(set_servers["added"][0], "filesystem");
    let set_servers_typed = stream.set_mcp_servers_typed(&servers).await.unwrap();
    assert_eq!(set_servers_typed.added[0], "filesystem");
    assert_eq!(set_servers_typed.added[1], "docs");
    assert!(set_servers_typed.errors.is_empty());
    stream.rename_session("Control title").await.unwrap();
    stream.set_color("blue").await.unwrap();
    assert_eq!(
        stream.file_suggestions("@src/").await.unwrap()["suggestions"][0]["path"],
        "src/lib.rs"
    );
    assert_eq!(
        stream.get_binary_version().await.unwrap()["version"],
        "1.2.3"
    );
    assert_eq!(stream.get_session_cost().await.unwrap()["summary"], "$0.01");
    assert_eq!(
        stream
            .mcp_call(
                "mcp__filesystem__read",
                Some(serde_json::json!({"path": "README.md"}))
            )
            .await
            .unwrap()["content"][0]["text"],
        "ok"
    );
    assert_eq!(
        stream
            .mcp_message(
                "filesystem",
                serde_json::json!({"jsonrpc": "2.0", "method": "ping", "id": 1})
            )
            .await
            .unwrap()["mcp_response"]["id"],
        1
    );

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("controlled"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_stream_exposes_initialization_metadata_helpers() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-query-metadata.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({
  "type": "control_response",
  "response": {
    "subtype": "success",
    "request_id": init["request_id"],
    "response": {
      "commands": [{"name": "review", "description": "Review work", "argumentHint": "[topic]", "aliases": ["rv"]}],
      "agents": [{"name": "critic", "description": "Challenge assumptions", "model": "claude-test"}],
      "output_style": "default",
      "available_output_styles": ["default"],
      "models": [{"value": "claude-test", "displayName": "Claude Test", "description": "Fake model"}],
      "account": {"email": "agent@example.com", "organization": "Example Org", "subscriptionType": "pro"}
    }
  }
}), flush=True)
seen_rewind = False
while True:
    line = sys.stdin.readline()
    assert line, "expected SDK input"
    message = json.loads(line)
    if message["type"] == "control_request":
        request = message["request"]
        assert request["subtype"] == "rewind_files"
        assert request["user_message_id"] == "user-message-id"
        assert request["dry_run"] is True
        assert "dryRun" not in request
        seen_rewind = True
        print(json.dumps({
          "type": "control_response",
          "response": {
            "subtype": "success",
            "request_id": message["request_id"],
            "response": {"canRewind": True, "filesChanged": ["README.md"], "insertions": 2, "deletions": 1}
          }
        }), flush=True)
        continue
    assert message["type"] == "user"
    assert message["message"]["content"] == "metadata context"
    break
assert seen_rewind
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-query-metadata",
  "result": "metadata"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let (mut input_tx, input_rx) = futures_mpsc::unbounded();
    let mut stream = query_stream(input_rx, options).unwrap();

    let initialization = stream.initialization_result().await.unwrap();
    assert_eq!(initialization["commands"][0]["name"], "review");
    assert_eq!(
        stream.initialization_result_typed().await.unwrap().models[0].value,
        "claude-test"
    );
    assert_eq!(
        stream.supported_commands().await.unwrap()[0]["argumentHint"],
        "[topic]"
    );
    assert_eq!(
        stream.supported_commands_typed().await.unwrap()[0].aliases[0],
        "rv"
    );
    assert_eq!(
        stream.supported_models_typed().await.unwrap()[0].display_name,
        "Claude Test"
    );
    assert_eq!(
        stream.supported_agents_typed().await.unwrap()[0].name,
        "critic"
    );
    assert_eq!(
        stream.account_info().await.unwrap().unwrap()["email"],
        "agent@example.com"
    );
    assert_eq!(
        stream.account_info_typed().await.unwrap().email.as_deref(),
        Some("agent@example.com")
    );
    let rewind = stream.rewind_files("user-message-id", true).await.unwrap();
    assert_eq!(rewind["canRewind"], true);
    assert_eq!(rewind["filesChanged"][0], "README.md");
    let rewind_typed = stream
        .rewind_files_typed("user-message-id", true)
        .await
        .unwrap();
    assert!(rewind_typed.can_rewind);
    assert_eq!(
        rewind_typed.files_changed.as_deref(),
        Some(&["README.md".to_owned()][..])
    );
    assert_eq!(rewind_typed.insertions, Some(2));
    assert_eq!(rewind_typed.deletions, Some(1));

    input_tx
        .start_send(UserMessage::text("metadata context"))
        .unwrap();
    drop(input_tx);

    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("metadata"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_uses_custom_process_spawner_without_local_cli() {
    let dir = tempdir().unwrap();
    let missing_cli = dir
        .path()
        .join("remote-claude-that-is-not-installed-locally");
    let remote_cwd = dir.path().join("remote-cwd-that-is-not-local");
    let captured_spawn = Arc::new(Mutex::new(None::<ProcessSpawnOptions>));
    let killed = Arc::new(AtomicBool::new(false));

    let options = ClaudeAgentOptions::builder()
        .cli_path(&missing_cli)
        .executable_arg("--launcher-flag")
        .cwd(&remote_cwd)
        .env("CUSTOM_ENV", "custom-value")
        .spawn_claude_code_process({
            let captured_spawn = Arc::clone(&captured_spawn);
            let killed = Arc::clone(&killed);
            move |spawn_options: ProcessSpawnOptions| {
                assert!(!spawn_options.signal.is_aborted());
                *captured_spawn.lock().unwrap() = Some(spawn_options);

                let (sdk_stdin, process_stdin) = tokio::io::duplex(16 * 1024);
                let (process_stdout, sdk_stdout) = tokio::io::duplex(16 * 1024);
                let wait_task = tokio::spawn(async move {
                    let mut stdin_lines = BufReader::new(process_stdin).lines();
                    let mut stdout = process_stdout;

                    let init: serde_json::Value =
                        serde_json::from_str(&stdin_lines.next_line().await.unwrap().unwrap())
                            .unwrap();
                    assert_eq!(init["type"], "control_request");
                    assert_eq!(init["request"]["subtype"], "initialize");
                    stdout
                        .write_all(
                            json!({
                                "type": "control_response",
                                "response": {
                                    "subtype": "success",
                                    "request_id": init["request_id"],
                                    "response": {"ok": true}
                                }
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .await
                        .unwrap();
                    stdout.write_all(b"\n").await.unwrap();

                    let user: serde_json::Value =
                        serde_json::from_str(&stdin_lines.next_line().await.unwrap().unwrap())
                            .unwrap();
                    assert_eq!(user["type"], "user");
                    assert_eq!(user["message"]["content"], "from custom process");
                    stdout
                        .write_all(
                            json!({
                                "type": "result",
                                "subtype": "success",
                                "duration_ms": 1,
                                "duration_api_ms": 1,
                                "is_error": false,
                                "num_turns": 1,
                                "session_id": "session-custom-spawn",
                                "result": "custom spawn ok"
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .await
                        .unwrap();
                    stdout.write_all(b"\n").await.unwrap();
                    Ok(())
                });
                let killed = Arc::clone(&killed);
                async move {
                    Ok(SpawnedClaudeProcess::new(
                        sdk_stdin,
                        sdk_stdout,
                        async move { wait_task.await.unwrap() },
                        move || {
                            killed.store(true, Ordering::SeqCst);
                        },
                    ))
                }
            }
        })
        .build();

    let mut stream = query("from custom process", options).unwrap();
    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("custom spawn ok"))
    );
    assert!(stream.next().await.is_none());

    let spawn_options = captured_spawn.lock().unwrap().take().unwrap();
    assert_eq!(spawn_options.command, missing_cli);
    assert_eq!(spawn_options.cwd.as_deref(), Some(remote_cwd.as_path()));
    assert_eq!(
        spawn_options.env.get("CUSTOM_ENV").map(String::as_str),
        Some("custom-value")
    );
    assert_eq!(
        spawn_options
            .env
            .get("CLAUDE_CODE_ENTRYPOINT")
            .map(String::as_str),
        Some("sdk-rs")
    );
    assert!(!spawn_options.env.contains_key("CLAUDECODE"));
    assert!(
        spawn_options
            .args
            .windows(2)
            .any(|window| window == ["--input-format".to_owned(), "stream-json".to_owned()])
    );
    assert_eq!(
        spawn_options.args.first().map(String::as_str),
        Some("--launcher-flag")
    );
    assert!(!killed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn query_wraps_javascript_cli_path_with_executable_for_custom_spawner() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("claude.mjs");
    let captured_spawn = Arc::new(Mutex::new(None::<ProcessSpawnOptions>));

    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .executable("node")
        .executable_arg("--runtime-flag")
        .spawn_claude_code_process({
            let captured_spawn = Arc::clone(&captured_spawn);
            move |spawn_options: ProcessSpawnOptions| {
                *captured_spawn.lock().unwrap() = Some(spawn_options);

                let (sdk_stdin, process_stdin) = tokio::io::duplex(16 * 1024);
                let (process_stdout, sdk_stdout) = tokio::io::duplex(16 * 1024);
                let wait_task = tokio::spawn(async move {
                    let mut stdin_lines = BufReader::new(process_stdin).lines();
                    let mut stdout = process_stdout;

                    let init: serde_json::Value =
                        serde_json::from_str(&stdin_lines.next_line().await.unwrap().unwrap())
                            .unwrap();
                    stdout
                        .write_all(
                            json!({
                                "type": "control_response",
                                "response": {
                                    "subtype": "success",
                                    "request_id": init["request_id"],
                                    "response": {"ok": true}
                                }
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .await
                        .unwrap();
                    stdout.write_all(b"\n").await.unwrap();

                    let _user: serde_json::Value =
                        serde_json::from_str(&stdin_lines.next_line().await.unwrap().unwrap())
                            .unwrap();
                    stdout
                        .write_all(
                            json!({
                                "type": "result",
                                "subtype": "success",
                                "duration_ms": 1,
                                "duration_api_ms": 1,
                                "is_error": false,
                                "num_turns": 1,
                                "session_id": "session-js-wrapper",
                                "result": "wrapped"
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .await
                        .unwrap();
                    stdout.write_all(b"\n").await.unwrap();
                    Ok(())
                });
                async move {
                    Ok(SpawnedClaudeProcess::new(
                        sdk_stdin,
                        sdk_stdout,
                        async move { wait_task.await.unwrap() },
                        || {},
                    ))
                }
            }
        })
        .build();

    let mut stream = query("from js wrapper", options).unwrap();
    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("wrapped"))
    );
    assert!(stream.next().await.is_none());

    let spawn_options = captured_spawn.lock().unwrap().take().unwrap();
    assert_eq!(spawn_options.command, PathBuf::from("node"));
    assert_eq!(
        spawn_options.args.first().map(String::as_str),
        Some("--runtime-flag")
    );
    assert_eq!(
        spawn_options.args.get(1).map(String::as_str),
        Some(script.to_string_lossy().as_ref())
    );
    assert_eq!(
        spawn_options.args.get(2).map(String::as_str),
        Some("--output-format")
    );
}

#[tokio::test]
async fn dropping_query_kills_custom_process() {
    let killed = Arc::new(AtomicBool::new(false));
    let (spawned_tx, spawned_rx) = oneshot::channel();
    let (aborted_tx, aborted_rx) = oneshot::channel();
    let spawned_tx = Arc::new(Mutex::new(Some(spawned_tx)));
    let aborted_tx = Arc::new(Mutex::new(Some(aborted_tx)));

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process({
            let killed = Arc::clone(&killed);
            let spawned_tx = Arc::clone(&spawned_tx);
            let aborted_tx = Arc::clone(&aborted_tx);
            move |spawn_options: ProcessSpawnOptions| {
                if let Some(tx) = spawned_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
                assert!(!spawn_options.signal.is_aborted());
                let mut signal = spawn_options.signal.clone();
                let aborted_tx = Arc::clone(&aborted_tx);
                tokio::spawn(async move {
                    let aborted = signal.aborted().await;
                    if let Some(tx) = aborted_tx.lock().unwrap().take() {
                        let _ = tx.send(aborted);
                    }
                });
                let (sdk_stdin, _process_stdin) = tokio::io::duplex(1024);
                let (_process_stdout, sdk_stdout) = tokio::io::duplex(1024);
                let killed = Arc::clone(&killed);
                async move {
                    Ok(SpawnedClaudeProcess::new(
                        sdk_stdin,
                        sdk_stdout,
                        std::future::pending::<oya_cloud_intelligence_claude_agent_sdk::Result<()>>(),
                        move || {
                            killed.store(true, Ordering::SeqCst);
                        },
                    ))
                }
            }
        })
        .build();

    let stream = query("drop before completion", options).unwrap();
    spawned_rx.await.unwrap();
    drop(stream);

    timeout(Duration::from_secs(1), async {
        while !killed.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        timeout(Duration::from_secs(1), aborted_rx)
            .await
            .unwrap()
            .unwrap()
    );
}

#[tokio::test]
async fn query_maps_tool_config_preview_format_to_cli_environment() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-tool-config.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, os, sys
assert os.environ["CLAUDE_CODE_QUESTION_PREVIEW_FORMAT"] == "html"
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-tool-config",
  "result": "done"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .ask_user_question_preview_format(oya_cloud_intelligence_claude_agent_sdk::QuestionPreviewFormat::Html)
        .build();
    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_forwards_cli_stderr_when_callback_is_configured() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-stderr.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
sys.stderr.write("debug before init\n")
sys.stderr.flush()
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
json.loads(sys.stdin.readline())
sys.stderr.write("debug after prompt\n")
sys.stderr.flush()
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-stderr",
  "result": "done"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let stderr = Arc::new(Mutex::new(String::new()));
    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .stderr({
            let stderr = Arc::clone(&stderr);
            move |chunk| stderr.lock().unwrap().push_str(&chunk)
        })
        .build();
    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));
    assert!(stream.next().await.is_none());

    timeout(Duration::from_secs(1), async {
        loop {
            let captured = stderr.lock().unwrap().clone();
            if captured.contains("debug before init") && captured.contains("debug after prompt") {
                return;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn query_continues_forwarding_cli_stderr_after_callback_panic() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-stderr-panic.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys, time
sys.stderr.write("stderr before init\n")
sys.stderr.flush()
time.sleep(0.2)
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
json.loads(sys.stdin.readline())
sys.stderr.write("stderr after panic\n")
sys.stderr.flush()
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-stderr-panic",
  "result": "done"
}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let stderr = Arc::new(Mutex::new(String::new()));
    let panic_count = Arc::new(AtomicUsize::new(0));
    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .stderr({
            let stderr = Arc::clone(&stderr);
            let panic_count = Arc::clone(&panic_count);
            move |chunk| {
                stderr.lock().unwrap().push_str(&chunk);
                if panic_count.fetch_add(1, Ordering::SeqCst) == 0 {
                    panic!("simulated stderr callback panic");
                }
            }
        })
        .build();
    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));
    assert!(stream.next().await.is_none());

    timeout(Duration::from_secs(1), async {
        loop {
            let captured = stderr.lock().unwrap().clone();
            if captured.contains("stderr before init") && captured.contains("stderr after panic") {
                return;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn query_rejects_stdout_lines_over_max_buffer_size() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-large-line.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
json.loads(sys.stdin.readline())
print('{"type":"assistant","message":{"model":"claude-test","content":[{"type":"text","text":"' + ("x" * 512) + '"}]}}', flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let mut options = ClaudeAgentOptions::builder().cli_path(&script).build();
    options.max_buffer_size = Some(128);
    let mut stream = query("world", options).unwrap();
    let error = stream.next().await.unwrap().unwrap_err().to_string();
    assert!(error.contains("max_buffer_size"));
}

#[tokio::test]
async fn query_reports_nonzero_process_exit_after_stream_end() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-nonzero-exit.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "control_response",
  "response": {"subtype": "success", "request_id": init["request_id"], "response": {"ok": True}}
}), flush=True)
json.loads(sys.stdin.readline())
print(json.dumps({
  "type": "result",
  "subtype": "success",
  "duration_ms": 1,
  "duration_api_ms": 1,
  "is_error": False,
  "num_turns": 1,
  "session_id": "session-nonzero",
  "result": "done"
}), flush=True)
sys.exit(7)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));

    let error = stream.next().await.unwrap().unwrap_err().to_string();
    assert!(error.contains("non-zero status"));
}
