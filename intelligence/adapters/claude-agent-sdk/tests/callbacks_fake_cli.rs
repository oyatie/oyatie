use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, ClaudeSDKClient, ElicitationMode, ElicitationResult, HookInput,
    HookSpecificOutput, Message, PermissionResult, SyncHookJsonOutput, query,
};
use futures::StreamExt;
use serde_json::json;
use tempfile::{TempDir, tempdir};
use tokio::time::{Duration, timeout};

fn executable_script(name: &str, body: &str) -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let script = dir.path().join(name);
    fs::write(&script, body).unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();
    (dir, script)
}

#[tokio::test]
async fn query_aborts_elicitation_callback_on_control_cancel_request() {
    let (_dir, script) = executable_script(
        "fake-elicitation-cancel.py",
        r#"#!/usr/bin/env python3
import json, sys, time
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":True}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"elicit_cancel_1",
  "request":{
    "subtype":"elicitation",
    "mcp_server_name":"github",
    "message":"Authorize GitHub",
    "mode":"form",
    "elicitation_id":"elicit-cancel-1"
  }
}), flush=True)
time.sleep(0.05)
print(json.dumps({
  "type":"control_cancel_request",
  "request_id":"elicit_cancel_1"
}), flush=True)
response = json.loads(sys.stdin.readline())
assert response["type"] == "control_response"
assert response["response"]["subtype"] == "success"
assert response["response"]["request_id"] == "elicit_cancel_1"
assert response["response"]["response"]["action"] == "decline"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"elicitation-cancel-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .on_elicitation(|request, options| async move {
            assert_eq!(request.elicitation_id.as_deref(), Some("elicit-cancel-1"));
            let mut signal = options.signal;
            assert!(signal.aborted().await);
            Ok(ElicitationResult::decline())
        })
        .build();
    let mut stream = query("authorize", options).unwrap();
    let response = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("control_cancel_request should abort the pending callback")
        .unwrap()
        .unwrap();
    assert!(
        matches!(response, Message::Result(result) if result.result.as_deref() == Some("elicitation-cancel-ok"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn client_aborts_elicitation_callback_on_control_cancel_request() {
    let (_dir, script) = executable_script(
        "fake-client-elicitation-cancel.py",
        r#"#!/usr/bin/env python3
import json, sys, time
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":True}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"client_elicit_cancel_1",
  "request":{
    "subtype":"elicitation",
    "mcp_server_name":"github",
    "message":"Authorize GitHub",
    "mode":"form",
    "elicitation_id":"client-elicit-cancel-1"
  }
}), flush=True)
time.sleep(0.05)
print(json.dumps({
  "type":"control_cancel_request",
  "request_id":"client_elicit_cancel_1"
}), flush=True)
response = json.loads(sys.stdin.readline())
assert response["type"] == "control_response"
assert response["response"]["subtype"] == "success"
assert response["response"]["request_id"] == "client_elicit_cancel_1"
assert response["response"]["response"]["action"] == "decline"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"client-elicitation-cancel-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .on_elicitation(|request, options| async move {
            assert_eq!(
                request.elicitation_id.as_deref(),
                Some("client-elicit-cancel-1")
            );
            let mut signal = options.signal;
            assert!(signal.aborted().await);
            Ok(ElicitationResult::decline())
        })
        .build();
    let mut client = ClaudeSDKClient::new(options);
    client.query("authorize").await.unwrap();
    let response = timeout(Duration::from_secs(2), client.receive_response())
        .await
        .expect("control_cancel_request should abort the pending client callback")
        .unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("client-elicitation-cancel-ok"))
    );
    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn query_handles_on_elicitation_control_request() {
    let (_dir, script) = executable_script(
        "fake-elicitation-query.py",
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":True}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"elicit_req_1",
  "request":{
    "subtype":"elicitation",
    "mcp_server_name":"github",
    "message":"Authorize GitHub",
    "mode":"form",
    "requested_schema":{"type":"object","properties":{"account":{"type":"string"}}},
    "title":"GitHub authorization",
    "display_name":"GitHub",
    "description":"Choose the account to authorize",
    "elicitation_id":"elicit-1"
  }
}), flush=True)
response = json.loads(sys.stdin.readline())
assert response["type"] == "control_response"
assert response["response"]["subtype"] == "success"
assert response["response"]["request_id"] == "elicit_req_1"
assert response["response"]["response"]["action"] == "accept"
assert response["response"]["response"]["content"]["account"] == "octo"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"elicitation-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .on_elicitation(|request, options| async move {
            assert_eq!(request.server_name, "github");
            assert_eq!(request.message, "Authorize GitHub");
            assert_eq!(request.mode, Some(ElicitationMode::Form));
            assert_eq!(request.elicitation_id.as_deref(), Some("elicit-1"));
            assert_eq!(request.title.as_deref(), Some("GitHub authorization"));
            assert_eq!(request.display_name.as_deref(), Some("GitHub"));
            assert_eq!(
                request.description.as_deref(),
                Some("Choose the account to authorize")
            );
            assert!(!options.signal.is_aborted());
            Ok(ElicitationResult::accept_with_content(
                json!({"account": "octo"}).as_object().unwrap().clone(),
            ))
        })
        .build();
    let mut stream = query("authorize", options).unwrap();
    let response = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(response, Message::Result(result) if result.result.as_deref() == Some("elicitation-ok"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn client_declines_unhandled_elicitation_control_request_by_default() {
    let (_dir, script) = executable_script(
        "fake-elicitation-default.py",
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":True}}}), flush=True)
print(json.dumps({
  "type":"control_request",
  "request_id":"elicit_req_default",
  "request":{
    "subtype":"elicitation",
    "mcp_server_name":"github",
    "message":"Authorize GitHub",
    "mode":"url",
    "url":"https://example.invalid/auth",
    "elicitation_id":"elicit-default"
  }
}), flush=True)
response = json.loads(sys.stdin.readline())
assert response["type"] == "control_response"
assert response["response"]["subtype"] == "success"
assert response["response"]["request_id"] == "elicit_req_default"
assert response["response"]["response"]["action"] == "decline"
assert "content" not in response["response"]["response"]
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"default-decline-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder().cli_path(script).build();
    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client.query("after default decline").await.unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("default-decline-ok"))
    );
    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn client_handles_can_use_tool_control_request() {
    let (_dir, script) = executable_script(
        "fake-permission.py",
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"perm_1",
  "request":{
    "subtype":"can_use_tool",
    "tool_name":"Write",
    "input":{"file_path":"/system/config"},
    "permission_suggestions": None,
    "blocked_path": None,
    "tool_use_id":"toolu_1"
  }
}), flush=True)
permission = json.loads(sys.stdin.readline())
assert permission["type"] == "control_response"
assert permission["response"]["subtype"] == "success"
assert permission["response"]["response"]["behavior"] == "allow"
assert permission["response"]["response"]["updatedInput"]["file_path"] == "./sandbox/config"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"permission-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .can_use_tool(|request| async move {
            assert_eq!(request.tool_name, "Write");
            Ok(PermissionResult::allow_with_updated_input(json!({
                "file_path": "./sandbox/config"
            })))
        })
        .build();
    let mut client = ClaudeSDKClient::new(options);
    client.query("write config").await.unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("permission-ok"))
    );
    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn query_handles_oauth_and_host_auth_token_refresh_control_requests() {
    let (_dir, script) = executable_script(
        "fake-auth-refresh.py",
        r#"#!/usr/bin/env python3
import json, os, sys
assert os.environ["CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH"] == "1"
assert os.environ["CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH"] == "1"
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"oauth_refresh",
  "request":{"subtype":"oauth_token_refresh"}
}), flush=True)
oauth = json.loads(sys.stdin.readline())
assert oauth["type"] == "control_response"
assert oauth["response"]["subtype"] == "success"
assert oauth["response"]["request_id"] == "oauth_refresh"
assert oauth["response"]["response"]["accessToken"] == "oauth-token"
print(json.dumps({
  "type":"control_request",
  "request_id":"host_auth_refresh",
  "request":{"subtype":"host_auth_token_refresh"}
}), flush=True)
host_auth = json.loads(sys.stdin.readline())
assert host_auth["type"] == "control_response"
assert host_auth["response"]["subtype"] == "success"
assert host_auth["response"]["request_id"] == "host_auth_refresh"
assert host_auth["response"]["response"]["authToken"] is None
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"auth-refresh-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .get_oauth_token(|options| async move {
            assert!(!options.signal.is_aborted());
            Ok(Some("oauth-token".into()))
        })
        .get_host_auth_token(|options| async move {
            assert!(!options.signal.is_aborted());
            Ok(None)
        })
        .build();
    let mut stream = query("refresh auth", options).unwrap();
    let response = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(response, Message::Result(result) if result.result.as_deref() == Some("auth-refresh-ok"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn query_handles_request_user_dialog_control_request() {
    let (_dir, script) = executable_script(
        "fake-user-dialog.py",
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
assert init["type"] == "control_request"
assert init["request"]["subtype"] == "initialize"
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"dialog_req",
  "request":{
    "subtype":"request_user_dialog",
    "dialog_kind":"computer_use_approval",
    "payload":{"operation":"click","target":"Approve"},
    "tool_use_id":"toolu_dialog"
  }
}), flush=True)
dialog = json.loads(sys.stdin.readline())
assert dialog["type"] == "control_response"
assert dialog["response"]["subtype"] == "success"
assert dialog["response"]["request_id"] == "dialog_req"
assert dialog["response"]["response"]["decision"] == "approve"
assert dialog["response"]["response"]["toolUseId"] == "toolu_dialog"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"dialog-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .on_user_dialog(|request, options| async move {
            assert_eq!(request.dialog_kind, "computer_use_approval");
            assert_eq!(
                request
                    .payload
                    .get("operation")
                    .and_then(|value| value.as_str()),
                Some("click")
            );
            assert_eq!(request.tool_use_id.as_deref(), Some("toolu_dialog"));
            assert!(!options.signal.is_aborted());
            Ok(json!({
                "decision": "approve",
                "toolUseId": request.tool_use_id.unwrap()
            }))
        })
        .build();
    let mut stream = query("open dialog", options).unwrap();
    let response = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(response, Message::Result(result) if result.result.as_deref() == Some("dialog-ok"))
    );
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn client_registers_and_handles_hook_callbacks() {
    let (_dir, script) = executable_script(
        "fake-hook.py",
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
hooks = init["request"]["hooks"]
callback_id = hooks["PreToolUse"][0]["hookCallbackIds"][0]
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
print(json.dumps({
  "type":"control_request",
  "request_id":"hook_req",
  "request":{
    "subtype":"hook_callback",
    "callback_id": callback_id,
    "input":{
      "hook_event_name":"PreToolUse",
      "session_id":"s",
      "transcript_path":"/tmp/transcript.jsonl",
      "cwd":"/workspace",
      "tool_name":"Bash",
      "tool_input":{"command":"pwd"},
      "tool_use_id":"toolu_hook"
    },
    "tool_use_id":"toolu_hook"
  }
}), flush=True)
stored_user = None
while True:
    inbound = json.loads(sys.stdin.readline())
    if inbound["type"] == "user":
        stored_user = inbound
        continue
    hook_response = inbound
    break
assert hook_response["response"]["response"]["hookSpecificOutput"]["additionalContext"] == "checked"
user = stored_user or json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"hook-ok"}), flush=True)
"#,
    );

    let options = ClaudeAgentOptions::builder()
        .cli_path(script)
        .hook(
            "PreToolUse",
            Some("Bash"),
            Some(5.0),
            |request| async move {
                assert_eq!(request.tool_use_id.as_deref(), Some("toolu_hook"));
                let HookInput::PreToolUse(input) = request.hook_input().unwrap() else {
                    panic!("expected PreToolUse hook input");
                };
                assert_eq!(input.base.cwd, "/workspace");
                assert_eq!(input.tool_name, "Bash");
                assert_eq!(input.tool_input["command"], "pwd");
                Ok(SyncHookJsonOutput::new()
                    .hook_specific_output(HookSpecificOutput::PreToolUse {
                        permission_decision: None,
                        permission_decision_reason: None,
                        updated_input: None,
                        additional_context: Some("checked".into()),
                    })
                    .into())
            },
        )
        .build();
    let mut client = ClaudeSDKClient::new(options);
    client.connect(None).await.unwrap();
    client.query("after hook").await.unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("hook-ok"))
    );
    client.disconnect().await.unwrap();
}

#[test]
fn permission_callback_auto_configures_stdio_prompt_tool_and_rejects_conflict() {
    let options = ClaudeAgentOptions::builder()
        .cli_path("/tmp/does-not-matter")
        .can_use_tool(|_| async { Ok(PermissionResult::allow()) })
        .build();
    let args = options.to_cli_args().unwrap();
    assert!(
        args.windows(2)
            .any(|w| w == ["--permission-prompt-tool", "stdio"])
    );

    let mut conflict = ClaudeAgentOptions::builder()
        .can_use_tool(|_| async { Ok(PermissionResult::allow()) })
        .build();
    conflict.permission_prompt_tool_name = Some("other".into());
    assert!(conflict.to_cli_args().is_err());
}
