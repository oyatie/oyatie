use std::{fs, os::unix::fs::PermissionsExt};

use intelligence_claude_agent_sdk::{
    CallToolResult, ClaudeAgentOptions, ClaudeSDKClient, JsonSchema, Message, SdkMcpTool,
    create_sdk_mcp_server, query,
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn client_routes_sdk_mcp_list_and_call_messages() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-sdk-mcp.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_init",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}
  }
}), flush=True)
init_response = json.loads(sys.stdin.readline())
assert init_response["response"]["response"]["mcp_response"]["result"]["protocolVersion"] == "2025-06-18"
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_list",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":1,"method":"tools/list"}
  }
}), flush=True)
list_response = json.loads(sys.stdin.readline())
tools = list_response["response"]["response"]["mcp_response"]["result"]["tools"]
assert tools[0]["name"] == "add"
assert tools[0]["inputSchema"]["type"] == "object"
assert tools[0]["inputSchema"]["properties"]["a"]["type"] == "integer"
assert tools[0]["inputSchema"]["required"] == ["a", "b"]
assert tools[0]["inputSchema"]["additionalProperties"] == False
assert tools[0]["_meta"]["anthropic/searchHint"] == "Use for math"
assert tools[0]["_meta"]["anthropic/alwaysLoad"] == True
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_call",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":1,"b":2}}}
  }
}), flush=True)
call_response = json.loads(sys.stdin.readline())
call_result = call_response["response"]["response"]["mcp_response"]["result"]
content = call_result["content"]
assert content[0]["text"] == "3"
assert call_result["structuredContent"]["sum"] == 3
assert call_result["isError"] == False
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_bad",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"arguments":{}}}
  }
}), flush=True)
bad_response = json.loads(sys.stdin.readline())
assert bad_response["response"]["response"]["mcp_response"]["error"]["code"] == -32602
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_bad_schema",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add","arguments":{"a":1}}}
  }
}), flush=True)
schema_response = json.loads(sys.stdin.readline())
schema_error = schema_response["response"]["response"]["mcp_response"]["error"]
assert schema_error["code"] == -32602
assert "arguments.b is required" in schema_error["message"]
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_typed_args",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"integer_only","arguments":{"value":1.5}}}
  }
}), flush=True)
typed_response = json.loads(sys.stdin.readline())
typed_error = typed_response["response"]["response"]["mcp_response"]["error"]
assert typed_error["code"] == -32602
assert "typed tool arguments" in typed_error["message"]
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"mcp-ok"}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    #[derive(Deserialize)]
    struct AddArgs {
        a: i64,
        b: i64,
    }

    #[derive(Deserialize)]
    struct IntegerArgs {
        value: i64,
    }

    let add = SdkMcpTool::new_typed(
        "add",
        "Add two numbers",
        JsonSchema::object()
            .required_property("a", JsonSchema::integer())
            .required_property("b", JsonSchema::integer())
            .additional_properties(false),
        |args: AddArgs, _extra| async move {
            let sum = args.a + args.b;
            Ok(CallToolResult {
                content: vec![json!({"type": "text", "text": sum.to_string()})],
                structured_content: Some(json!({"sum": sum})),
                is_error: Some(false),
            })
        },
    )
    .search_hint("Use for math")
    .always_load(true);
    let integer_only = SdkMcpTool::new_typed(
        "integer_only",
        "Accept a JSON number that must deserialize into a Rust integer",
        JsonSchema::object().required_property("value", JsonSchema::number()),
        |args: IntegerArgs, _extra| async move {
            Ok(CallToolResult {
                content: vec![json!({"type": "text", "text": args.value.to_string()})],
                structured_content: None,
                is_error: Some(false),
            })
        },
    );
    let server = create_sdk_mcp_server("calculator", "1.0.0", vec![add, integer_only]);
    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .sdk_mcp_server("calc", server)
        .build();
    let mut client = ClaudeSDKClient::new(options);
    client.query("use calculator").await.unwrap();
    let response = client.receive_response().await.unwrap();
    assert!(
        matches!(response.last(), Some(Message::Result(result)) if result.result.as_deref() == Some("mcp-ok"))
    );
    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn query_routes_sdk_mcp_messages() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("fake-sdk-mcp-query.py");
    fs::write(
        &script,
        r#"#!/usr/bin/env python3
import json, sys
init = json.loads(sys.stdin.readline())
print(json.dumps({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}}), flush=True)
user = json.loads(sys.stdin.readline())
assert user["type"] == "user"
print(json.dumps({
  "type":"control_request",
  "request_id":"mcp_call_query",
  "request":{
    "subtype":"mcp_message",
    "server_name":"calc",
    "message":{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"add","arguments":{"a":4,"b":5}}}
  }
}), flush=True)
call_response = json.loads(sys.stdin.readline())
call_result = call_response["response"]["response"]["mcp_response"]["result"]
assert call_result["content"][0]["text"] == "9"
assert call_result["structuredContent"]["sum"] == 9
print(json.dumps({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":False,"num_turns":1,"session_id":"s","result":"query-mcp-ok"}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let add = SdkMcpTool::new(
        "add",
        "Add two numbers",
        json!({
            "type": "object",
            "properties": {"a": {"type": "number"}, "b": {"type": "number"}},
            "required": ["a", "b"]
        }),
        |args, _extra| async move {
            let sum = args["a"].as_i64().unwrap() + args["b"].as_i64().unwrap();
            Ok(CallToolResult {
                content: vec![json!({"type": "text", "text": sum.to_string()})],
                structured_content: Some(json!({"sum": sum})),
                is_error: Some(false),
            })
        },
    );
    let server = create_sdk_mcp_server("calculator", "1.0.0", vec![add]);
    let options = ClaudeAgentOptions::builder()
        .cli_path(&script)
        .sdk_mcp_server("calc", server)
        .build();
    let mut stream = query("use calculator", options).unwrap();
    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("query-mcp-ok"))
    );
    assert!(stream.next().await.is_none());
}
