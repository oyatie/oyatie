#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, write_json_line};

use futures::StreamExt;
use intelligence_claude_agent_sdk::{
    CallToolResult, ClaudeAgentOptions, ClaudeSDKClient, JsonSchema, Message, SdkMcpTool,
    create_sdk_mcp_server, query,
};
use serde::Deserialize;
use serde_json::json;

#[tokio::test]
async fn client_routes_sdk_mcp_list_and_call_messages() {
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
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");

            // initialize MCP
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_init","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}}})).await;
            let init_response = expect_json_line(&mut r).await;
            assert_eq!(init_response["response"]["response"]["mcp_response"]["result"]["protocolVersion"], "2025-06-18");

            // list tools
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_list","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":1,"method":"tools/list"}}})).await;
            let list_response = expect_json_line(&mut r).await;
            let tools = &list_response["response"]["response"]["mcp_response"]["result"]["tools"];
            assert_eq!(tools[0]["name"], "add");
            assert_eq!(tools[0]["inputSchema"]["type"], "object");
            assert_eq!(tools[0]["inputSchema"]["properties"]["a"]["type"], "integer");
            assert_eq!(tools[0]["inputSchema"]["required"], json!(["a","b"]));
            assert_eq!(tools[0]["inputSchema"]["additionalProperties"], false);
            assert_eq!(tools[0]["_meta"]["anthropic/searchHint"], "Use for math");
            assert_eq!(tools[0]["_meta"]["anthropic/alwaysLoad"], true);

            // call add(1, 2)
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_call","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":1,"b":2}}}}})).await;
            let call_response = expect_json_line(&mut r).await;
            let call_result = &call_response["response"]["response"]["mcp_response"]["result"];
            let content = &call_result["content"];
            assert_eq!(content[0]["text"], "3");
            assert_eq!(call_result["structuredContent"]["sum"], 3);
            assert_eq!(call_result["isError"], false);

            // call with missing tool name → error -32602
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_bad","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"arguments":{}}}}})).await;
            let bad_response = expect_json_line(&mut r).await;
            assert_eq!(bad_response["response"]["response"]["mcp_response"]["error"]["code"], -32602);

            // call add with missing required arg b → schema error
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_bad_schema","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add","arguments":{"a":1}}}}})).await;
            let schema_response = expect_json_line(&mut r).await;
            let schema_error = &schema_response["response"]["response"]["mcp_response"]["error"];
            assert_eq!(schema_error["code"], -32602);
            assert!(schema_error["message"].as_str().unwrap_or("").contains("arguments.b is required"));

            // call integer_only with float 1.5 → typed deserialization error
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_typed_args","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"integer_only","arguments":{"value":1.5}}}}})).await;
            let typed_response = expect_json_line(&mut r).await;
            let typed_error = &typed_response["response"]["response"]["mcp_response"]["error"];
            assert_eq!(typed_error["code"], -32602);
            assert!(typed_error["message"].as_str().unwrap_or("").contains("typed tool arguments"));

            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"mcp-ok"})).await;
        }))
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
        .spawn_claude_code_process(fake_cli(|mut r, mut w, _| async move {
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({"type":"control_response","response":{"subtype":"success","request_id":init["request_id"],"response":{}}})).await;
            let user = expect_json_line(&mut r).await;
            assert_eq!(user["type"], "user");
            write_json_line(&mut w, &json!({"type":"control_request","request_id":"mcp_call_query","request":{"subtype":"mcp_message","server_name":"calc","message":{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"add","arguments":{"a":4,"b":5}}}}})).await;
            let call_response = expect_json_line(&mut r).await;
            let call_result = &call_response["response"]["response"]["mcp_response"]["result"];
            assert_eq!(call_result["content"][0]["text"], "9");
            assert_eq!(call_result["structuredContent"]["sum"], 9);
            write_json_line(&mut w, &json!({"type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"session_id":"s","result":"query-mcp-ok"})).await;
        }))
        .sdk_mcp_server("calc", server)
        .build();
    let mut stream = query("use calculator", options).unwrap();
    let message = stream.next().await.unwrap().unwrap();
    assert!(
        matches!(message, Message::Result(result) if result.result.as_deref() == Some("query-mcp-ok"))
    );
    assert!(stream.next().await.is_none());
}
