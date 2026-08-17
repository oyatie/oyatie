#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, write_json_line};

use futures::StreamExt;
use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, ClaudeSDKClient, ElicitationMode, ElicitationResult, HookInput,
    HookSpecificOutput, Message, PermissionResult, SyncHookJsonOutput, query,
};
use serde_json::json;
use tokio::time::{Duration, timeout};

#[tokio::test]
async fn query_aborts_elicitation_callback_on_control_cancel_request() {
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"elicit_cancel_1","request":{"subtype":"elicitation","mcp_server_name":"github","message":"Authorize GitHub","mode":"form","elicitation_id":"elicit-cancel-1"}})).await;
            write_json_line(&mut w, &json!({"type":"control_cancel_request","request_id":"elicit_cancel_1"})).await;
            let response = expect_json_line(&mut r).await;
            assert_eq!(response["type"], "control_response");
            assert_eq!(response["response"]["subtype"], "success");
            assert_eq!(response["response"]["request_id"], "elicit_cancel_1");
            assert_eq!(response["response"]["response"]["action"], "decline");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"elicitation-cancel-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"client_elicit_cancel_1","request":{"subtype":"elicitation","mcp_server_name":"github","message":"Authorize GitHub","mode":"form","elicitation_id":"client-elicit-cancel-1"}})).await;
            write_json_line(&mut w, &json!({"type":"control_cancel_request","request_id":"client_elicit_cancel_1"})).await;
            let response = expect_json_line(&mut r).await;
            assert_eq!(response["type"], "control_response");
            assert_eq!(response["response"]["subtype"], "success");
            assert_eq!(response["response"]["request_id"], "client_elicit_cancel_1");
            assert_eq!(response["response"]["response"]["action"], "decline");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"client-elicitation-cancel-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"elicit_req_1","request":{"subtype":"elicitation","mcp_server_name":"github","message":"Authorize GitHub","mode":"form","requested_schema":{"type":"object","properties":{"account":{"type":"string"}}},"title":"GitHub authorization","display_name":"GitHub","description":"Choose the account to authorize","elicitation_id":"elicit-1"}})).await;
            let response = expect_json_line(&mut r).await;
            assert_eq!(response["type"], "control_response");
            assert_eq!(response["response"]["subtype"], "success");
            assert_eq!(response["response"]["request_id"], "elicit_req_1");
            assert_eq!(response["response"]["response"]["action"], "accept");
            assert_eq!(response["response"]["response"]["content"]["account"], "octo");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"elicitation-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}})).await;
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"elicit_req_default","request":{"subtype":"elicitation","mcp_server_name":"github","message":"Authorize GitHub","mode":"url","url":"https://example.invalid/auth","elicitation_id":"elicit-default"}})).await;
            let response = expect_json_line(&mut r).await;
            assert_eq!(response["type"], "control_response");
            assert_eq!(response["response"]["subtype"], "success");
            assert_eq!(response["response"]["request_id"], "elicit_req_default");
            assert_eq!(response["response"]["response"]["action"], "decline");
            assert!(response["response"]["response"].get("content").is_none() || response["response"]["response"]["content"].is_null());
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"default-decline-ok"})).await;
        }))
        .build();
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"perm_1","request":{"subtype":"can_use_tool","tool_name":"Write","input":{"file_path":"/system/config"},"permission_suggestions":null,"blocked_path":null,"tool_use_id":"toolu_1"}})).await;
            let permission = expect_json_line(&mut r).await;
            assert_eq!(permission["type"], "control_response");
            assert_eq!(permission["response"]["subtype"], "success");
            assert_eq!(permission["response"]["response"]["behavior"], "allow");
            assert_eq!(permission["response"]["response"]["updatedInput"]["file_path"], "./sandbox/config");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"permission-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, opts| async move {
            assert_eq!(opts.env.get("CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH").map(String::as_str), Some("1"));
            assert_eq!(opts.env.get("CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH").map(String::as_str), Some("1"));
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"oauth_refresh","request":{"subtype":"oauth_token_refresh"}})).await;
            let oauth = expect_json_line(&mut r).await;
            assert_eq!(oauth["type"], "control_response");
            assert_eq!(oauth["response"]["subtype"], "success");
            assert_eq!(oauth["response"]["request_id"], "oauth_refresh");
            assert_eq!(oauth["response"]["response"]["accessToken"], "oauth-token");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"host_auth_refresh","request":{"subtype":"host_auth_token_refresh"}})).await;
            let host_auth = expect_json_line(&mut r).await;
            assert_eq!(host_auth["type"], "control_response");
            assert_eq!(host_auth["response"]["subtype"], "success");
            assert_eq!(host_auth["response"]["request_id"], "host_auth_refresh");
            assert!(host_auth["response"]["response"]["authToken"].is_null());
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"auth-refresh-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            assert_eq!(init["type"], "control_request");
            assert_eq!(init["request"]["subtype"], "initialize");
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"dialog_req","request":{"subtype":"request_user_dialog","dialog_kind":"computer_use_approval","payload":{"operation":"click","target":"Approve"},"tool_use_id":"toolu_dialog"}})).await;
            let dialog = expect_json_line(&mut r).await;
            assert_eq!(dialog["type"], "control_response");
            assert_eq!(dialog["response"]["subtype"], "success");
            assert_eq!(dialog["response"]["request_id"], "dialog_req");
            assert_eq!(dialog["response"]["response"]["decision"], "approve");
            assert_eq!(dialog["response"]["response"]["toolUseId"], "toolu_dialog");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"dialog-ok"})).await;
        }))
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
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            let hooks = &init["request"]["hooks"];
            let callback_id = hooks["PreToolUse"][0]["hookCallbackIds"][0].as_str().unwrap().to_owned();
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"hook_req","request":{"subtype":"hook_callback","callback_id":callback_id,"input":{"hook_event_name":"PreToolUse","session_id":"s","transcript_path":"/tmp/transcript.jsonl","cwd":"/workspace","tool_name":"Bash","tool_input":{"command":"pwd"},"tool_use_id":"toolu_hook"},"tool_use_id":"toolu_hook"}})).await;
            // Drain until we get the hook control_response (user messages may arrive first)
            let mut hook_response = None;
            loop {
                let inbound = expect_json_line(&mut r).await;
                if inbound["type"] == "user" {
                    continue;
                }
                hook_response = Some(inbound);
                break;
            }
            let hook_response = hook_response.unwrap();
            assert_eq!(hook_response["response"]["response"]["hookSpecificOutput"]["additionalContext"], "checked");
            // consume any remaining user message
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"hook-ok"})).await;
        }))
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
