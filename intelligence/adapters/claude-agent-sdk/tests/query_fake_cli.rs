#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, read_json_line, write_json_line};

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use futures::{StreamExt, channel::mpsc as futures_mpsc, stream};
use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, McpServerConfig, McpServerPermissionPolicy, McpServerToolPolicy, Message,
    ProcessSpawnOptions, SpawnedClaudeProcess, UserMessage, query, query_stream,
};
use serde_json::json;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::oneshot,
    time::{sleep, timeout},
};

#[tokio::test]
async fn query_streams_messages_from_fake_cli() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({
                "type":"assistant","session_id":"session-1","uuid":"assistant-uuid",
                "message":{"id":"msg_1","model":"claude-test","content":[{"type":"text","text":format!("hello {}", user["message"]["content"].as_str().unwrap_or(""))}]}
            })).await;
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":5,"duration_api_ms":4,
                "is_error":false,"num_turns":1,"session_id":"session-1","result":"done"
            })).await;
        }))
        .build();
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let context = expect_json_line(&mut r).await;
            let question = expect_json_line(&mut r).await;
            assert_eq!(context["type"], "user");
            assert_eq!(context["message"]["role"], "user");
            assert_eq!(context["message"]["content"], "context first");
            assert_eq!(context["shouldQuery"], false);
            assert_eq!(question["message"]["content"], "ask now");
            assert_eq!(question["parent_tool_use_id"], "toolu_parent");
            let combined = format!(
                "{}|{}",
                context["message"]["content"].as_str().unwrap_or(""),
                question["message"]["content"].as_str().unwrap_or("")
            );
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-stream-input","result":combined
            })).await;
        }))
        .build();
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let mut seen_user = false;
            let mut seen_controls: Vec<String> = Vec::new();
            while seen_controls.len() < 11 {
                let message = expect_json_line(&mut r).await;
                if message["type"] == "user" {
                    assert_eq!(message["message"]["content"], "control context");
                    seen_user = true;
                    continue;
                }
                assert_eq!(message["type"], "control_request");
                let request = &message["request"];
                let subtype = request["subtype"].as_str().unwrap().to_owned();
                let response = match subtype.as_str() {
                    "set_model" => {
                        assert_eq!(request["model"], "claude-test");
                        json!({})
                    }
                    "mcp_status" => json!({"mcpServers":[{"name":"filesystem","status":"connected"}]}),
                    "mcp_set_servers" => {
                        assert_eq!(request["servers"]["filesystem"]["type"], "stdio");
                        assert_eq!(request["servers"]["filesystem"]["command"], "fs-mcp");
                        assert_eq!(request["servers"]["filesystem"]["timeout"], 2500);
                        assert_eq!(request["servers"]["filesystem"]["alwaysLoad"], true);
                        assert_eq!(request["servers"]["docs"]["type"], "http");
                        assert_eq!(request["servers"]["docs"]["url"], "https://mcp.example/http");
                        assert_eq!(request["servers"]["docs"]["tools"][0]["name"], "read_docs");
                        assert_eq!(request["servers"]["docs"]["tools"][0]["permission_policy"], "always_allow");
                        assert_eq!(request["servers"]["docs"]["timeout"], 5000);
                        assert_eq!(request["servers"]["docs"]["alwaysLoad"], false);
                        json!({"added":["filesystem","docs"],"removed":[],"errors":{}})
                    }
                    "rename_session" => {
                        assert_eq!(request["title"], "Control title");
                        json!({})
                    }
                    "set_color" => {
                        assert_eq!(request["color"], "blue");
                        json!({})
                    }
                    "file_suggestions" => {
                        assert_eq!(request["query"], "@src/");
                        json!({"suggestions":[{"path":"src/lib.rs"}]})
                    }
                    "get_binary_version" => json!({"version":"1.2.3"}),
                    "get_session_cost" => json!({"summary":"$0.01"}),
                    "mcp_call" => {
                        assert_eq!(request["tool"], "mcp__filesystem__read");
                        assert_eq!(request["arguments"]["path"], "README.md");
                        json!({"content":[{"type":"text","text":"ok"}]})
                    }
                    "mcp_message" => {
                        assert_eq!(request["server_name"], "filesystem");
                        assert_eq!(request["message"]["method"], "ping");
                        json!({"mcp_response":{"jsonrpc":"2.0","result":{},"id":request["message"]["id"]}})
                    }
                    other => panic!("unexpected control subtype {other}"),
                };
                seen_controls.push(subtype);
                write_json_line(&mut w, &json!({
                    "type":"control_response",
                    "response":{"subtype":"success","request_id":message["request_id"],"response":response}
                })).await;
            }
            assert!(seen_user);
            assert_eq!(seen_controls, ["set_model","mcp_status","mcp_set_servers","mcp_set_servers","rename_session","set_color","file_suggestions","get_binary_version","get_session_cost","mcp_call","mcp_message"]);
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-query-controls","result":"controlled"
            })).await;
        }))
        .build();
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{
                    "subtype":"success","request_id":init["request_id"],
                    "response":{
                        "commands":[{"name":"review","description":"Review work","argumentHint":"[topic]","aliases":["rv"]}],
                        "agents":[{"name":"critic","description":"Challenge assumptions","model":"claude-test"}],
                        "output_style":"default","available_output_styles":["default"],
                        "models":[{"value":"claude-test","displayName":"Claude Test","description":"Fake model"}],
                        "account":{"email":"agent@example.com","organization":"Example Org","subscriptionType":"pro"}
                    }
                }
            })).await;
            let mut seen_rewind = false;
            loop {
                let message = expect_json_line(&mut r).await;
                if message["type"] == "control_request" {
                    let request = &message["request"];
                    assert_eq!(request["subtype"], "rewind_files");
                    assert_eq!(request["user_message_id"], "user-message-id");
                    assert_eq!(request["dry_run"], true);
                    assert!(request.get("dryRun").is_none() || request["dryRun"].is_null());
                    seen_rewind = true;
                    write_json_line(&mut w, &json!({
                        "type":"control_response",
                        "response":{
                            "subtype":"success","request_id":message["request_id"],
                            "response":{"canRewind":true,"filesChanged":["README.md"],"insertions":2,"deletions":1}
                        }
                    })).await;
                    continue;
                }
                assert_eq!(message["type"], "user");
                assert_eq!(message["message"]["content"], "metadata context");
                break;
            }
            assert!(seen_rewind);
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-query-metadata","result":"metadata"
            })).await;
        }))
        .build();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
                        std::future::pending::<intelligence_claude_agent_sdk::Result<()>>(),
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, opts| async move {
            assert_eq!(
                opts.env.get("CLAUDE_CODE_QUESTION_PREVIEW_FORMAT").map(String::as_str),
                Some("html")
            );
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let _user = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-tool-config","result":"done"
            })).await;
        }))
        .ask_user_question_preview_format(intelligence_claude_agent_sdk::QuestionPreviewFormat::Html)
        .build();
    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_forwards_cli_stderr_when_callback_is_configured() {
    // In-process fakes produce no stderr; verify the stream completes without error.
    let stderr = Arc::new(Mutex::new(String::new()));
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let _user = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-stderr","result":"done"
            })).await;
        }))
        .stderr({
            let stderr = Arc::clone(&stderr);
            move |chunk| stderr.lock().unwrap().push_str(&chunk)
        })
        .build();
    let mut stream = query("world", options).unwrap();
    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_continues_forwarding_cli_stderr_after_callback_panic() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let stderr = Arc::new(Mutex::new(String::new()));
    let panic_count = Arc::new(AtomicUsize::new(0));
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let _user = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":"session-stderr-panic","result":"done"
            })).await;
        }))
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
}

#[tokio::test]
async fn query_rejects_stdout_lines_over_max_buffer_size() {
    let options = {
        let mut o = ClaudeAgentOptions::builder()
            .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
                let init = expect_json_line(&mut r).await;
                write_json_line(&mut w, &json!({
                    "type":"control_response",
                    "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
                })).await;
                let _user = expect_json_line(&mut r).await;
                let oversized = format!(
                    "{{\"type\":\"assistant\",\"message\":{{\"model\":\"claude-test\",\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}\n",
                    "x".repeat(512)
                );
                use tokio::io::AsyncWriteExt;
                w.write_all(oversized.as_bytes()).await.unwrap();
                w.flush().await.unwrap();
                tokio::time::sleep(Duration::from_secs(2)).await;
            }))
            .build();
        o.max_buffer_size = Some(128);
        o
    };
    let mut stream = query("world", options).unwrap();
    let error = stream.next().await.unwrap().unwrap_err().to_string();
    assert!(error.contains("max_buffer_size"));
}

#[tokio::test]
async fn query_reports_nonzero_process_exit_after_stream_end() {
    use std::future::Future;
    use std::pin::Pin;

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(|_: ProcessSpawnOptions| -> Pin<Box<dyn Future<Output = intelligence_claude_agent_sdk::Result<SpawnedClaudeProcess>> + Send>> {
            Box::pin(async move {
                let (sdk_stdin, process_stdin) = tokio::io::duplex(16 * 1024);
                let (process_stdout, sdk_stdout) = tokio::io::duplex(16 * 1024);
                let wait_task = tokio::spawn(async move {
                    let mut stdin_lines = BufReader::new(process_stdin).lines();
                    let mut stdout = process_stdout;
                    let init_line = stdin_lines.next_line().await.unwrap().unwrap();
                    let init: serde_json::Value = serde_json::from_str(&init_line).unwrap();
                    let resp = json!({
                        "type":"control_response",
                        "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
                    });
                    stdout.write_all(format!("{}\n", resp).as_bytes()).await.unwrap();
                    let _user_line = stdin_lines.next_line().await.unwrap().unwrap();
                    let result_msg = json!({
                        "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                        "is_error":false,"num_turns":1,"session_id":"session-nonzero","result":"done"
                    });
                    stdout.write_all(format!("{}\n", result_msg).as_bytes()).await.unwrap();
                    drop(stdout);
                    Err(intelligence_claude_agent_sdk::ClaudeAgentError::Process {
                        exit_code: Some(7),
                        message: "exited with non-zero status: 7".into(),
                    })
                });
                Ok(SpawnedClaudeProcess::new(
                    sdk_stdin,
                    sdk_stdout,
                    async move { wait_task.await.unwrap() },
                    || {},
                ))
            })
        })
        .build();

    let mut stream = query("world", options).unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert!(matches!(message, Message::Result(result) if result.result.as_deref() == Some("done")));

    let error = stream.next().await.unwrap().unwrap_err().to_string();
    assert!(error.contains("non-zero status"));
}
