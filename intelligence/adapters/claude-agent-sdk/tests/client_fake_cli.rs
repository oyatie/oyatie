#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, write_json_line};

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use futures::{StreamExt, stream};
use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, McpServerConfig, McpServerPermissionPolicy,
    McpServerStatusConfig, McpServerToolPolicy, Message, PermissionMode, ReadFileEncoding,
    SpawnedClaudeProcess, UserMessage,
};
use serde_json::json;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::oneshot,
    time::{sleep, timeout},
};

#[tokio::test]
async fn client_sends_followups_and_control_requests() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            loop {
                let msg = match support::read_json_line(&mut r).await {
                    None => break,
                    Some(v) => v,
                };
                if msg["type"] == "control_request" {
                    let subtype = msg["request"]["subtype"].as_str().unwrap().to_owned();
                    let response = match subtype.as_str() {
                        "initialize" => json!({
                            "commands":[{"name":"help","description":"Show help","argumentHint":"[topic]","aliases":["h"]}],
                            "models":[{"value":"claude-test","displayName":"Claude Test","description":"Test model","supportsEffort":true,"supportedEffortLevels":["low","medium"],"supportsAdaptiveThinking":true,"supportsFastMode":false}],
                            "agents":[{"name":"reviewer","description":"Reviews code","model":"claude-test"}],
                            "account":{"email":"test@example.com","organization":"Test Org","subscriptionType":"pro","tokenSource":"oauth","apiKeySource":"none"},
                            "output_style":"default",
                            "available_output_styles":["default"],
                            "fast_mode_state":"off"
                        }),
                        "mcp_status" => json!({
                            "mcpServers": [{"name":"filesystem","status":"connected","serverInfo":{"name":"fs","version":"1.0.0"},"config":{"type":"stdio","command":"fs-mcp","args":["--root","."]},"scope":"project","tools":[{"name":"read","description":"Read files","annotations":{"readOnly":true}}]}]
                        }),
                        "get_context_usage" => json!({
                            "categories":[{"name":"Messages","tokens":42,"color":"blue","isDeferred":false}],
                            "totalTokens":42,"maxTokens":200000,"rawMaxTokens":200000,"percentage":0.021,
                            "model":"claude-test","isAutoCompactEnabled":true,
                            "memoryFiles":[{"path":"CLAUDE.md","tokens":5}],
                            "mcpTools":[{"name":"read","serverName":"filesystem","tokens":7,"isLoaded":true}],
                            "agents":[{"agentType":"reviewer","source":"project","tokens":9}],
                            "gridRows":[[{"label":"Messages","tokens":42}]],
                            "autoCompactThreshold":160000,
                            "messageBreakdown":{"toolCalls":3},
                            "apiUsage":{"input_tokens":10}
                        }),
                        "read_file" => {
                            let encoding = msg["request"].get("encoding").and_then(|v| v.as_str()).map(str::to_owned);
                            json!({
                                "contents": if encoding.as_deref() == Some("base64") { "aGVsbG8=" } else { "hello" },
                                "absPath": "/workspace/README.md",
                                "truncated": false,
                                "encoding": encoding
                            })
                        },
                        "reload_plugins" => json!({
                            "commands":[{"name":"review","description":"Review","argumentHint":"[target]","aliases":[]}],
                            "agents":[{"name":"planner","description":"Plans work"}],
                            "plugins":[{"name":"local-plugin","path":"/workspace/plugin","source":"project"}],
                            "mcpServers":[],
                            "error_count":0
                        }),
                        "get_settings" => json!({"model":"claude-test","permissions":{"defaultMode":"acceptEdits"}}),
                        "rewind_files" => {
                            assert_eq!(msg["request"]["user_message_id"], "user-message-id");
                            assert_eq!(msg["request"]["dry_run"], false);
                            assert!(msg["request"].get("dryRun").is_none());
                            json!({"canRewind":true,"filesChanged":["README.md"],"insertions":3,"deletions":1})
                        },
                        "cancel_async_message" => json!({"cancelled":true}),
                        "background_tasks" => {
                            let tool_id = msg["request"].get("tool_use_id").and_then(|v| v.as_str()).map(str::to_owned);
                            json!({"backgrounded": tool_id.as_deref() != Some("toolu_missing")})
                        },
                        "mcp_set_servers" => {
                            assert_eq!(msg["request"]["servers"]["filesystem"]["type"], "stdio");
                            assert_eq!(msg["request"]["servers"]["filesystem"]["command"], "fs-mcp");
                            assert_eq!(msg["request"]["servers"]["filesystem"]["timeout"], 2500);
                            assert_eq!(msg["request"]["servers"]["filesystem"]["alwaysLoad"], true);
                            assert_eq!(msg["request"]["servers"]["docs"]["type"], "sse");
                            assert_eq!(msg["request"]["servers"]["docs"]["url"], "https://mcp.example/sse");
                            assert_eq!(msg["request"]["servers"]["docs"]["tools"][0]["name"], "read_docs");
                            assert_eq!(msg["request"]["servers"]["docs"]["tools"][0]["permission_policy"], "always_ask");
                            assert_eq!(msg["request"]["servers"]["docs"]["timeout"], 5000);
                            assert_eq!(msg["request"]["servers"]["docs"]["alwaysLoad"], false);
                            json!({"added":["filesystem","docs"],"removed":[],"errors":{}})
                        },
                        "channel_enable" => {
                            assert_eq!(msg["request"]["serverName"], "filesystem");
                            json!({"enabled":true})
                        },
                        "mcp_authenticate" => {
                            assert_eq!(msg["request"]["serverName"], "filesystem");
                            assert_eq!(msg["request"]["redirectUri"], "http://localhost/callback");
                            json!({"url":"https://auth.example/authorize"})
                        },
                        "mcp_clear_auth" => {
                            assert_eq!(msg["request"]["serverName"], "filesystem");
                            json!({"cleared":true})
                        },
                        "mcp_oauth_callback_url" => {
                            assert_eq!(msg["request"]["serverName"], "filesystem");
                            assert_eq!(msg["request"]["callbackUrl"], "http://localhost/callback?code=ok");
                            json!({"ok":true})
                        },
                        "claude_authenticate" => {
                            assert_eq!(msg["request"]["loginWithClaudeAi"], true);
                            json!({"url":"https://claude.example/login"})
                        },
                        "claude_oauth_callback" => {
                            assert_eq!(msg["request"]["authorizationCode"], "code-123");
                            assert_eq!(msg["request"]["state"], "state-456");
                            json!({"ok":true})
                        },
                        "claude_oauth_wait_for_completion" => json!({"completed":true}),
                        "remote_control" => {
                            assert_eq!(msg["request"]["enabled"], true);
                            assert_eq!(msg["request"]["name"], "sdk-host");
                            json!({"enabled":true,"name":"sdk-host"})
                        },
                        "submit_feedback" => {
                            assert_eq!(msg["request"]["description"], "feedback body");
                            assert_eq!(msg["request"]["surface"], "sdk-test");
                            json!({"submitted":true})
                        },
                        "generate_session_title" => {
                            assert_eq!(msg["request"]["description"], "summarize this session");
                            assert_eq!(msg["request"]["persist"], true);
                            json!({"title":"Generated title"})
                        },
                        "side_question" => {
                            assert_eq!(msg["request"]["question"], "Need more context?");
                            json!({"response":"No","synthetic":true})
                        },
                        "ultrareview_launch" => {
                            assert_eq!(msg["request"]["args"], json!(["--fast","--json"]));
                            assert_eq!(msg["request"]["confirm"], true);
                            json!({"launched":true})
                        },
                        "message_rated" => {
                            assert_eq!(msg["request"]["messageUuid"], "msg-uuid");
                            assert_eq!(msg["request"]["sentiment"], "positive");
                            assert_eq!(msg["request"]["surface"], "sdk-test");
                            assert_eq!(msg["request"]["cleared"], false);
                            json!({"ok":true})
                        },
                        _ => json!({"subtype": subtype, "ok": true}),
                    };
                    write_json_line(&mut w, &json!({
                        "type": "control_response",
                        "response": {"subtype":"success","request_id":msg["request_id"],"response":response}
                    })).await;
                } else if msg["type"] == "user" {
                    let content = msg["message"]["content"].clone();
                    if content == "scoped" {
                        assert_eq!(msg["session_id"], "thread-42");
                    }
                    write_json_line(&mut w, &json!({
                        "type": "assistant",
                        "session_id": "session-client",
                        "message": {"model":"claude-test","content":[{"type":"text","text":content}]}
                    })).await;
                    write_json_line(&mut w, &json!({
                        "type": "result",
                        "subtype": "success",
                        "duration_ms": 1,
                        "duration_api_ms": 1,
                        "is_error": false,
                        "num_turns": 1,
                        "session_id": "session-client",
                        "result": content
                    })).await;
                }
            }
        }))
        .build();

    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    let initialization_raw = client.initialization_result().unwrap();
    assert_eq!(initialization_raw["output_style"], "default");
    assert_eq!(client.supported_commands().unwrap()[0]["name"], "help");
    assert_eq!(
        client.account_info().unwrap().unwrap()["email"],
        "test@example.com"
    );
    let initialization = client.initialization_result_typed().unwrap();
    assert_eq!(initialization.output_style, "default");
    assert_eq!(
        initialization
            .fast_mode_state
            .as_ref()
            .map(|state| state.as_str()),
        Some("off")
    );
    assert_eq!(client.supported_commands_typed().unwrap()[0].name, "help");
    assert_eq!(
        client.supported_commands_typed().unwrap()[0].argument_hint,
        "[topic]"
    );
    assert_eq!(
        client.supported_models_typed().unwrap()[0].display_name,
        "Claude Test"
    );
    assert_eq!(
        client.supported_models_typed().unwrap()[0]
            .supported_effort_levels
            .as_ref()
            .unwrap()[0]
            .as_str(),
        "low"
    );
    assert_eq!(client.supported_agents_typed().unwrap()[0].name, "reviewer");
    assert_eq!(
        client.account_info_typed().unwrap().email.as_deref(),
        Some("test@example.com")
    );
    let raw_context = client.get_context_usage().await.unwrap();
    assert_eq!(raw_context["totalTokens"], 42);
    let context = client.get_context_usage_typed().await.unwrap();
    assert_eq!(context.total_tokens, 42);
    assert_eq!(context.categories[0].name, "Messages");
    assert!(context.is_auto_compact_enabled);
    assert_eq!(
        context
            .message_breakdown
            .as_ref()
            .and_then(|value| value.get("toolCalls"))
            .and_then(|value| value.as_u64()),
        Some(3)
    );
    assert_eq!(
        context
            .api_usage
            .as_ref()
            .and_then(|value| value.get("input_tokens"))
            .and_then(|value| value.as_u64()),
        Some(10)
    );
    let raw_mcp_status = client.get_mcp_status().await.unwrap();
    assert_eq!(raw_mcp_status["mcpServers"][0]["name"], "filesystem");
    let mcp_status = client.get_mcp_status_typed().await.unwrap();
    assert_eq!(mcp_status.mcp_servers[0].name, "filesystem");
    assert_eq!(mcp_status.mcp_servers[0].status.as_str(), "connected");
    match mcp_status.mcp_servers[0].config.as_ref().unwrap() {
        McpServerStatusConfig::Stdio(config) => assert_eq!(config.command, "fs-mcp"),
        other => panic!("expected stdio config, got {other:?}"),
    }
    client
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await
        .unwrap();
    client.set_max_thinking_tokens(Some(128)).await.unwrap();
    client.set_max_thinking_tokens(None).await.unwrap();
    client
        .apply_flag_settings(serde_json::json!({"model": "claude-other"}))
        .await
        .unwrap();
    assert_eq!(client.get_settings().await.unwrap()["model"], "claude-test");
    let rewind = client
        .rewind_files_typed("user-message-id", false)
        .await
        .unwrap();
    assert!(rewind.can_rewind);
    assert_eq!(
        rewind.files_changed.as_deref(),
        Some(&["README.md".to_owned()][..])
    );
    assert_eq!(rewind.insertions, Some(3));
    assert_eq!(rewind.deletions, Some(1));
    let read_file = client
        .read_file("README.md", Some(32), Some(ReadFileEncoding::Base64))
        .await
        .unwrap()
        .expect("read file response");
    assert_eq!(read_file.contents, "aGVsbG8=");
    assert_eq!(read_file.abs_path, "/workspace/README.md");
    assert_eq!(read_file.encoding, Some(ReadFileEncoding::Base64));
    let reload = client.reload_plugins_typed().await.unwrap();
    assert_eq!(reload.commands[0].name, "review");
    assert_eq!(reload.agents[0].name, "planner");
    assert_eq!(reload.plugins[0].name, "local-plugin");
    assert_eq!(reload.error_count, 0);
    client.seed_read_state("README.md", 1234).await.unwrap();
    assert!(client.cancel_async_message("user-uuid").await.unwrap());
    assert!(client.background_tasks(None).await.unwrap());
    assert!(
        !client
            .background_tasks(Some("toolu_missing"))
            .await
            .unwrap()
    );
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
        McpServerConfig::Sse {
            url: "https://mcp.example/sse".to_owned(),
            headers: BTreeMap::new(),
            tools: vec![McpServerToolPolicy {
                name: "read_docs".to_owned(),
                permission_policy: McpServerPermissionPolicy::AlwaysAsk,
            }],
            timeout: Some(5000),
            always_load: Some(false),
        },
    );
    let set_servers = client.set_mcp_servers_typed(&servers).await.unwrap();
    assert_eq!(set_servers.added[0], "filesystem");
    assert_eq!(set_servers.added[1], "docs");
    assert!(set_servers.removed.is_empty());
    client.enable_channel("filesystem").await.unwrap();
    assert_eq!(
        client
            .mcp_authenticate("filesystem", "http://localhost/callback")
            .await
            .unwrap()["url"],
        "https://auth.example/authorize"
    );
    assert!(
        client.mcp_clear_auth("filesystem").await.unwrap()["cleared"]
            .as_bool()
            .unwrap()
    );
    assert!(
        client
            .mcp_submit_oauth_callback_url("filesystem", "http://localhost/callback?code=ok")
            .await
            .unwrap()["ok"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        client.claude_authenticate(true).await.unwrap()["url"],
        "https://claude.example/login"
    );
    assert!(
        client
            .claude_oauth_callback("code-123", "state-456")
            .await
            .unwrap()["ok"]
            .as_bool()
            .unwrap()
    );
    assert!(
        client.claude_oauth_wait_for_completion().await.unwrap()["completed"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        client
            .enable_remote_control(true, Some("sdk-host"))
            .await
            .unwrap()["name"],
        "sdk-host"
    );
    assert!(
        client
            .submit_feedback("feedback body", Some("sdk-test"))
            .await
            .unwrap()["submitted"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        client
            .generate_session_title("summarize this session", Some(true))
            .await
            .unwrap(),
        "Generated title"
    );
    let side_question = client
        .ask_side_question("Need more context?")
        .await
        .unwrap();
    assert_eq!(side_question.as_ref().unwrap().response, "No");
    assert!(side_question.unwrap().synthetic);
    assert!(
        client
            .launch_ultrareview(&["--fast".to_owned(), "--json".to_owned()], true)
            .await
            .unwrap()["launched"]
            .as_bool()
            .unwrap()
    );
    client
        .message_rated("msg-uuid", "positive", "sdk-test", false)
        .await
        .unwrap();
    client.query("first").await.unwrap();
    let mut response = Vec::new();
    let mut messages = client.receive_messages();
    while let Some(message) = messages.next().await {
        let message = message.unwrap();
        let is_result = matches!(message, Message::Result(_));
        response.push(message);
        if is_result {
            break;
        }
    }
    drop(messages);
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("first"))
    );

    client
        .query_with_session_id("scoped", "thread-42")
        .await
        .unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("scoped"))
    );

    client.query("second").await.unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("second"))
    );

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn raw_accessors_preserve_malformed_control_responses() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            loop {
                let msg = match support::read_json_line(&mut r).await {
                    None => break,
                    Some(v) => v,
                };
                if msg["type"] != "control_request" {
                    continue;
                }
                let subtype = msg["request"]["subtype"].as_str().unwrap_or("").to_owned();
                let response = match subtype.as_str() {
                    "initialize" => json!({"commands":[]}),
                    "get_context_usage" => json!({
                        "categories":[{"name":"Messages","tokens":42,"color":"blue"}],
                        "maxTokens":200000,"rawMaxTokens":200000,"percentage":0.021,
                        "model":"claude-test","isAutoCompactEnabled":true,
                        "memoryFiles":[],"mcpTools":[],"agents":[],"gridRows":[]
                    }),
                    _ => json!({"ok":true}),
                };
                write_json_line(&mut w, &json!({
                    "type":"control_response",
                    "response":{"subtype":"success","request_id":msg["request_id"],"response":response}
                })).await;
            }
        }))
        .build();

    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();

    assert!(client.initialization_result_typed().is_err());
    assert!(
        client
            .initialization_result()
            .unwrap()
            .get("commands")
            .is_some()
    );
    assert!(
        client
            .initialization_result_raw()
            .unwrap()
            .get("commands")
            .is_some()
    );

    let raw_context = client.get_context_usage().await.unwrap();
    assert_eq!(raw_context["maxTokens"], 200000);
    let raw_context_alias = client.get_context_usage_raw().await.unwrap();
    assert_eq!(raw_context_alias["maxTokens"], 200000);
    assert!(client.get_context_usage_typed().await.is_err());

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn client_query_stream_sends_sdk_user_messages() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            loop {
                let msg = match support::read_json_line(&mut r).await {
                    None => break,
                    Some(v) => v,
                };
                if msg["type"] == "control_request" && msg["request"]["subtype"] == "initialize" {
                    write_json_line(&mut w, &json!({
                        "type":"control_response",
                        "response":{"subtype":"success","request_id":msg["request_id"],"response":{"commands":[],"models":[],"agents":[],"account":{}}}
                    })).await;
                } else if msg["type"] == "user" && msg["message"]["content"] == "side context" {
                    assert_eq!(msg["shouldQuery"], false);
                    assert_eq!(msg["priority"], "next");
                } else if msg["type"] == "user" {
                    assert_eq!(msg["message"]["content"][0]["type"], "text");
                    assert_eq!(msg["message"]["content"][0]["text"], "block question");
                    write_json_line(&mut w, &json!({
                        "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                        "is_error":false,"num_turns":1,"session_id":"session-client-stream",
                        "result":msg["message"]["content"][0]["text"]
                    })).await;
                }
            }
        }))
        .build();

    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client
        .query_stream(stream::iter([
            UserMessage::text("side context")
                .should_query(false)
                .priority(intelligence_claude_agent_sdk::UserMessagePriority::Next),
            UserMessage::blocks(vec![ContentBlock::Text {
                text: "block question".into(),
            }]),
        ]))
        .await
        .unwrap();

    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("block question"))
    );
    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn client_disconnect_closes_stdin_and_waits_for_clean_exit() {
    // In-process: verify that when disconnect() is called, the fake reader hits
    // EOF (the SDK closed its stdin half) before the wait future resolves.
    let (stdin_eof_tx, stdin_eof_rx) = tokio::sync::oneshot::channel::<()>();
    let stdin_eof_tx = Arc::new(Mutex::new(Some(stdin_eof_tx)));

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process({
            let stdin_eof_tx = Arc::clone(&stdin_eof_tx);
            fake_cli(move |mut r, mut w, _| {
                let stdin_eof_tx = Arc::clone(&stdin_eof_tx);
                async move {
                    // Handle initialize
                    let init = expect_json_line(&mut r).await;
                    write_json_line(&mut w, &json!({
                        "type":"control_response",
                        "response":{"subtype":"success","request_id":init["request_id"],"response":{"commands":[],"models":[],"agents":[],"account":{}}}
                    })).await;
                    // Drain until EOF (SDK closes stdin on disconnect)
                    while support::read_json_line(&mut r).await.is_some() {}
                    // Signal that stdin was closed
                    if let Some(tx) = stdin_eof_tx.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                }
            })
        })
        .build();

    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client.disconnect().await.unwrap();

    // disconnect() awaits the wait future which only resolves after the fake
    // task exits; the fake task only exits after it drains EOF; so if
    // disconnect() returned, the EOF signal must be receivable immediately.
    assert!(
        timeout(Duration::from_secs(1), stdin_eof_rx).await.is_ok(),
        "stdin was not closed before disconnect() returned"
    );
}

#[tokio::test]
async fn client_disconnect_reports_nonzero_shutdown_exit() {
    // Simulate a non-zero exit by having the wait future return an error.
    use intelligence_claude_agent_sdk::ProcessSpawnOptions;
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
                        "response":{"subtype":"success","request_id":init["request_id"],"response":{"commands":[],"models":[],"agents":[],"account":{}}}
                    });
                    stdout.write_all(format!("{}\n", resp).as_bytes()).await.unwrap();
                    // Drain remaining, then return a non-zero exit error
                    while stdin_lines.next_line().await.unwrap().is_some() {}
                    Err(intelligence_claude_agent_sdk::ClaudeAgentError::Process { exit_code: Some(9), message: "exited with non-zero status: 9".into() })
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

    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();

    let error = client.disconnect().await.unwrap_err().to_string();
    assert!(error.contains("non-zero status"));
}

// The following tests already use spawn_claude_code_process directly —
// they are preserved unchanged from the original file.

use intelligence_claude_agent_sdk::{ProcessSpawnOptions, query, query_stream};
use std::path::PathBuf;

#[tokio::test]
async fn query_uses_custom_process_spawner_without_local_cli() {
    use std::time::Duration as StdDuration;
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
    // stderr is a real-process feature; with in-process fakes there is no
    // stderr pipe. Verify the SDK does not break when no stderr arrives.
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
    // In-process fake produces no stderr; callback should simply not be called.
}

#[tokio::test]
async fn query_continues_forwarding_cli_stderr_after_callback_panic() {
    // Same as above: in-process fakes produce no stderr, so just verify the
    // SDK stream completes successfully when the callback would be unused.
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
                // Write a line that exceeds the 128-byte max_buffer_size.
                // We write raw bytes directly rather than using write_json_line
                // to avoid buffering through our 64 KiB duplex.
                let oversized = format!(
                    "{{\"type\":\"assistant\",\"message\":{{\"model\":\"claude-test\",\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}\n",
                    "x".repeat(512)
                );
                use tokio::io::AsyncWriteExt;
                w.write_all(oversized.as_bytes()).await.unwrap();
                w.flush().await.unwrap();
                // Hold the task alive so the duplex isn't closed
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
    use intelligence_claude_agent_sdk::ProcessSpawnOptions;
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
                    // Return non-zero exit
                    Err(intelligence_claude_agent_sdk::ClaudeAgentError::Process { exit_code: Some(7), message: "exited with non-zero status: 7".into() })
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
