use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt};

use oya_cloud_intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, McpServerConfig, McpServerPermissionPolicy,
    McpServerStatusConfig, McpServerToolPolicy, Message, PermissionMode, ReadFileEncoding,
    UserMessage,
};
use futures::{StreamExt, stream};
use tempfile::tempdir;

#[tokio::test]
async fn client_sends_followups_and_control_requests() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-client.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg["type"] == "control_request":
        subtype = msg["request"]["subtype"]
        if subtype == "initialize":
            response = {
              "commands":[{"name":"help","description":"Show help","argumentHint":"[topic]","aliases":["h"]}],
              "models":[{"value":"claude-test","displayName":"Claude Test","description":"Test model","supportsEffort":True,"supportedEffortLevels":["low","medium"],"supportsAdaptiveThinking":True,"supportsFastMode":False}],
              "agents":[{"name":"reviewer","description":"Reviews code","model":"claude-test"}],
              "account":{"email":"test@example.com","organization":"Test Org","subscriptionType":"pro","tokenSource":"oauth","apiKeySource":"none"},
              "output_style":"default",
              "available_output_styles":["default"],
              "fast_mode_state":"off"
            }
        elif subtype == "mcp_status":
            response = {
              "mcpServers": [{
                "name": "filesystem",
                "status": "connected",
                "serverInfo": {"name": "fs", "version": "1.0.0"},
                "config": {"type": "stdio", "command": "fs-mcp", "args": ["--root", "."]},
                "scope": "project",
                "tools": [{"name": "read", "description": "Read files", "annotations": {"readOnly": True}}]
              }]
            }
        elif subtype == "get_context_usage":
            response = {
              "categories": [{"name": "Messages", "tokens": 42, "color": "blue", "isDeferred": False}],
              "totalTokens": 42,
              "maxTokens": 200000,
              "rawMaxTokens": 200000,
              "percentage": 0.021,
              "model": "claude-test",
              "isAutoCompactEnabled": True,
              "memoryFiles": [{"path": "CLAUDE.md", "tokens": 5}],
              "mcpTools": [{"name": "read", "serverName": "filesystem", "tokens": 7, "isLoaded": True}],
              "agents": [{"agentType": "reviewer", "source": "project", "tokens": 9}],
              "gridRows": [[{"label": "Messages", "tokens": 42}]],
              "autoCompactThreshold": 160000,
              "messageBreakdown": {"toolCalls": 3},
              "apiUsage": {"input_tokens": 10}
            }
        elif subtype == "read_file":
            response = {
              "contents": "aGVsbG8=" if msg["request"].get("encoding") == "base64" else "hello",
              "absPath": "/workspace/README.md",
              "truncated": False,
              "encoding": msg["request"].get("encoding")
            }
        elif subtype == "reload_plugins":
            response = {
              "commands":[{"name":"review","description":"Review","argumentHint":"[target]","aliases":[]}],
              "agents":[{"name":"planner","description":"Plans work"}],
              "plugins":[{"name":"local-plugin","path":"/workspace/plugin","source":"project"}],
              "mcpServers": [],
              "error_count": 0
            }
        elif subtype == "get_settings":
            response = {"model": "claude-test", "permissions": {"defaultMode": "acceptEdits"}}
        elif subtype == "rewind_files":
            assert msg["request"]["user_message_id"] == "user-message-id"
            assert msg["request"]["dry_run"] == False
            assert "dryRun" not in msg["request"]
            response = {"canRewind": True, "filesChanged": ["README.md"], "insertions": 3, "deletions": 1}
        elif subtype == "cancel_async_message":
            response = {"cancelled": True}
        elif subtype == "background_tasks":
            response = {"backgrounded": msg["request"].get("tool_use_id") != "toolu_missing"}
        elif subtype == "mcp_set_servers":
            assert msg["request"]["servers"]["filesystem"]["type"] == "stdio"
            assert msg["request"]["servers"]["filesystem"]["command"] == "fs-mcp"
            assert msg["request"]["servers"]["filesystem"]["timeout"] == 2500
            assert msg["request"]["servers"]["filesystem"]["alwaysLoad"] == True
            assert msg["request"]["servers"]["docs"]["type"] == "sse"
            assert msg["request"]["servers"]["docs"]["url"] == "https://mcp.example/sse"
            assert msg["request"]["servers"]["docs"]["tools"][0]["name"] == "read_docs"
            assert msg["request"]["servers"]["docs"]["tools"][0]["permission_policy"] == "always_ask"
            assert msg["request"]["servers"]["docs"]["timeout"] == 5000
            assert msg["request"]["servers"]["docs"]["alwaysLoad"] == False
            response = {"added": ["filesystem", "docs"], "removed": [], "errors": {}}
        elif subtype == "channel_enable":
            assert msg["request"]["serverName"] == "filesystem"
            response = {"enabled": True}
        elif subtype == "mcp_authenticate":
            assert msg["request"]["serverName"] == "filesystem"
            assert msg["request"]["redirectUri"] == "http://localhost/callback"
            response = {"url": "https://auth.example/authorize"}
        elif subtype == "mcp_clear_auth":
            assert msg["request"]["serverName"] == "filesystem"
            response = {"cleared": True}
        elif subtype == "mcp_oauth_callback_url":
            assert msg["request"]["serverName"] == "filesystem"
            assert msg["request"]["callbackUrl"] == "http://localhost/callback?code=ok"
            response = {"ok": True}
        elif subtype == "claude_authenticate":
            assert msg["request"]["loginWithClaudeAi"] == True
            response = {"url": "https://claude.example/login"}
        elif subtype == "claude_oauth_callback":
            assert msg["request"]["authorizationCode"] == "code-123"
            assert msg["request"]["state"] == "state-456"
            response = {"ok": True}
        elif subtype == "claude_oauth_wait_for_completion":
            response = {"completed": True}
        elif subtype == "remote_control":
            assert msg["request"]["enabled"] == True
            assert msg["request"]["name"] == "sdk-host"
            response = {"enabled": True, "name": "sdk-host"}
        elif subtype == "submit_feedback":
            assert msg["request"]["description"] == "feedback body"
            assert msg["request"]["surface"] == "sdk-test"
            response = {"submitted": True}
        elif subtype == "generate_session_title":
            assert msg["request"]["description"] == "summarize this session"
            assert msg["request"]["persist"] == True
            response = {"title": "Generated title"}
        elif subtype == "side_question":
            assert msg["request"]["question"] == "Need more context?"
            response = {"response": "No", "synthetic": True}
        elif subtype == "ultrareview_launch":
            assert msg["request"]["args"] == ["--fast", "--json"]
            assert msg["request"]["confirm"] == True
            response = {"launched": True}
        elif subtype == "message_rated":
            assert msg["request"]["messageUuid"] == "msg-uuid"
            assert msg["request"]["sentiment"] == "positive"
            assert msg["request"]["surface"] == "sdk-test"
            assert msg["request"]["cleared"] == False
            response = {"ok": True}
        else:
            response = {"subtype": subtype, "ok": True}
        print(json.dumps({
          "type": "control_response",
          "response": {"subtype": "success", "request_id": msg["request_id"], "response": response}
        }), flush=True)
    elif msg["type"] == "user":
        content = msg["message"]["content"]
        if content == "scoped":
            assert msg["session_id"] == "thread-42"
        print(json.dumps({
          "type": "assistant",
          "session_id": "session-client",
          "message": {"model": "claude-test", "content": [{"type": "text", "text": content}]}
        }), flush=True)
        print(json.dumps({
          "type": "result",
          "subtype": "success",
          "duration_ms": 1,
          "duration_api_ms": 1,
          "is_error": False,
          "num_turns": 1,
          "session_id": "session-client",
          "result": content
        }), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
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
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-malformed.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg["type"] != "control_request":
        continue
    subtype = msg["request"]["subtype"]
    if subtype == "initialize":
        response = {"commands": []}
    elif subtype == "get_context_usage":
        response = {
          "categories": [{"name": "Messages", "tokens": 42, "color": "blue"}],
          "maxTokens": 200000,
          "rawMaxTokens": 200000,
          "percentage": 0.021,
          "model": "claude-test",
          "isAutoCompactEnabled": True,
          "memoryFiles": [],
          "mcpTools": [],
          "agents": [],
          "gridRows": []
        }
    else:
        response = {"ok": True}
    print(json.dumps({
      "type": "control_response",
      "response": {"subtype": "success", "request_id": msg["request_id"], "response": response}
    }), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
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
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-client-stream-input.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg["type"] == "control_request" and msg["request"]["subtype"] == "initialize":
        print(json.dumps({
          "type": "control_response",
          "response": {"subtype": "success", "request_id": msg["request_id"], "response": {"commands": [], "models": [], "agents": [], "account": {}}}
        }), flush=True)
    elif msg["type"] == "user" and msg["message"]["content"] == "side context":
        assert msg["shouldQuery"] == False
        assert msg["priority"] == "next"
    elif msg["type"] == "user":
        assert msg["message"]["content"][0]["type"] == "text"
        assert msg["message"]["content"][0]["text"] == "block question"
        print(json.dumps({
          "type": "result",
          "subtype": "success",
          "duration_ms": 1,
          "duration_api_ms": 1,
          "is_error": False,
          "num_turns": 1,
          "session_id": "session-client-stream",
          "result": msg["message"]["content"][0]["text"]
        }), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client
        .query_stream(stream::iter([
            UserMessage::text("side context")
                .should_query(false)
                .priority(oya_cloud_intelligence_claude_agent_sdk::UserMessagePriority::Next),
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
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-disconnect.py");
    let marker = dir.path().join("stdin-closed.txt");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, os, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg["type"] == "control_request" and msg["request"]["subtype"] == "initialize":
        print(json.dumps({
          "type": "control_response",
          "response": {"subtype": "success", "request_id": msg["request_id"], "response": {"commands": [], "models": [], "agents": [], "account": {}}}
        }), flush=True)
open(os.environ["STDIN_CLOSED_MARKER"], "w").write("closed\n")
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let mut options = ClaudeAgentOptions::builder().cli_path(&script).build();
    options.env.insert(
        "STDIN_CLOSED_MARKER".into(),
        marker.to_string_lossy().into_owned(),
    );
    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client.disconnect().await.unwrap();

    assert_eq!(fs::read_to_string(marker).unwrap(), "closed\n");
}

#[tokio::test]
async fn client_disconnect_reports_nonzero_shutdown_exit() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-claude-disconnect-nonzero.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg["type"] == "control_request" and msg["request"]["subtype"] == "initialize":
        print(json.dumps({
          "type": "control_response",
          "response": {"subtype": "success", "request_id": msg["request_id"], "response": {"commands": [], "models": [], "agents": [], "account": {}}}
        }), flush=True)
sys.exit(9)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let options = ClaudeAgentOptions::builder().cli_path(&script).build();
    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();

    let error = client.disconnect().await.unwrap_err().to_string();
    assert!(error.contains("non-zero status"));
}
