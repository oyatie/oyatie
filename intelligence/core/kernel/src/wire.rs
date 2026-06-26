//! OpenAI-shaped wire types — the canonical request/response/usage contract.
//!
//! Per LiteLLM-parity gap #26 (`types_schemas`) and top-gap #2 in the
//! reference-dissection findings: streaming, modality dispatch, and
//! error-mapping all depend on one OpenAI-shaped `wire` contract. This module
//! is that contract for the `/v1/chat/completions` endpoint plus the shared
//! `Usage` (token-count) shape.
//!
//! Scope discipline:
//! - **Pure data + (de)serialization only. ZERO I/O** — no network/clock/rand/fs.
//!   The types only depend on `serde`/`serde_json` (pure, in-tree), so the
//!   kernel stays I/O-free per the clean-arch parity floor.
//! - Mirrors OpenAI's published `/v1/chat/completions` wire schema faithfully
//!   (request, non-streaming response, streaming chunk, usage, error envelope).
//!   Source: <https://platform.openai.com/docs/api-reference/chat> (verify field
//!   set at impl/cutover time — provider wire shapes drift).
//! - Anthropic-Messages <-> OpenAI mapping is **out of scope** (lane T2c, the
//!   `ProviderTransform` seam). Nothing here references Anthropic shapes.
//!
//! Serialization conventions matching the OpenAI clients:
//! - Optional request fields are omitted when `None` (`skip_serializing_if`) so
//!   we emit the same minimal body an OpenAI SDK would.
//! - Unknown/optional fields deserialize tolerantly (`#[serde(default)]`).
//! - Arbitrary-JSON fields (tool `parameters`, `response_format.json_schema`)
//!   are faithfully kept as [`serde_json::Value`] — they are caller-defined
//!   JSON Schema documents, not a fixed shape.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

/// `POST /v1/chat/completions` request body (OpenAI chat-completions wire shape).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<StringOrArray>,
    /// Legacy max-tokens cap; superseded by `max_completion_tokens` for newer
    /// models but still honored on the wire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// `stream_options` — controls streaming extras; `include_usage` asks the
/// provider to emit a final usage-bearing chunk.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// `stop` accepts either a single string or an array of strings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    Single(String),
    Many(Vec<String>),
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// A chat message. Reused for request input and for the assistant message in a
/// response choice (the OpenAI wire shape is the same object).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    /// `null`/absent for an assistant message that only made tool calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Assistant tool-call requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Set on a `role: "tool"` result message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    Developer,
    #[default]
    User,
    Assistant,
    Tool,
}

/// Message content: a plain string, or an array of typed content parts
/// (multimodal). `null` content maps to `ChatMessage.content == None`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A single multimodal content part.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    /// `auto` | `low` | `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

/// A tool definition. `kind` is `"function"` for every tool today.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionDef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Caller-defined JSON Schema for the function arguments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// `tool_choice`: a mode string (`none`/`auto`/`required`) or a specific
/// named-function selection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    Named(NamedToolChoice),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedToolChoice {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionName,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionName {
    pub name: String,
}

/// An assistant tool-call in a (non-streaming) response or request message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    /// Arguments are a JSON-encoded **string** on the wire, not a JSON object.
    pub arguments: String,
}

// ---------------------------------------------------------------------------
// response_format
// ---------------------------------------------------------------------------

/// `response_format` discriminated by `type`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    /// Structured-outputs schema; the inner document is caller-defined.
    JsonSchema { json_schema: Value },
}

// ---------------------------------------------------------------------------
// Non-streaming response
// ---------------------------------------------------------------------------

/// A non-streaming `chat.completion` response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    /// Always `"chat.completion"`.
    pub object: String,
    /// Unix epoch seconds.
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
}

// ---------------------------------------------------------------------------
// Streaming response (chat.completion.chunk)
// ---------------------------------------------------------------------------

/// A streaming `chat.completion.chunk` (one SSE `data:` frame).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    /// Always `"chat.completion.chunk"`.
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
    /// Present only on the final chunk when `stream_options.include_usage` was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChoiceDelta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// The incremental delta carried by a streaming chunk choice.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChoiceDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallChunk>>,
}

/// A streaming tool-call fragment. Every field except `index` is partial and
/// optional — the decoder accumulates `arguments` across chunks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCallChunk {
    pub index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallChunk>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallChunk {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

// ---------------------------------------------------------------------------
// Usage / token-count (shared shape)
// ---------------------------------------------------------------------------

/// Token accounting — the shared usage/token-count shape. Carried by both the
/// non-streaming response and the final streaming chunk; also the canonical
/// shape a token-count probe returns.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u32>,
}

// ---------------------------------------------------------------------------
// Error envelope
// ---------------------------------------------------------------------------

/// OpenAI error envelope (`{"error": {...}}`). This is the wire shape the
/// T2b `map_error()` taxonomy reads; the mapping logic itself is out of scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    /// Struct-level round-trip: serialize -> parse -> assert structural equality.
    /// (Compares structs, not bytes, so `None`-omission vs explicit-`null` is a
    /// non-issue.)
    fn roundtrip<T>(value: &T) -> T
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, &back, "round-trip changed the value");
        back
    }

    #[test]
    fn request_roundtrips_with_rich_fields() {
        let req = ChatCompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![
                ChatMessage {
                    role: Role::System,
                    content: Some(MessageContent::Text("be terse".to_string())),
                    ..Default::default()
                },
                ChatMessage {
                    role: Role::User,
                    content: Some(MessageContent::Parts(vec![
                        ContentPart::Text {
                            text: "what is this".to_string(),
                        },
                        ContentPart::ImageUrl {
                            image_url: ImageUrl {
                                url: "https://example.test/a.png".to_string(),
                                detail: Some("low".to_string()),
                            },
                        },
                    ])),
                    ..Default::default()
                },
            ],
            temperature: Some(0.7),
            stream: Some(true),
            stream_options: Some(StreamOptions {
                include_usage: Some(true),
            }),
            stop: Some(StringOrArray::Many(vec!["\n\n".to_string()])),
            max_completion_tokens: Some(256),
            tools: Some(vec![Tool {
                kind: "function".to_string(),
                function: FunctionDef {
                    name: "get_weather".to_string(),
                    description: Some("look up weather".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "city": { "type": "string" } },
                        "required": ["city"]
                    })),
                },
            }]),
            tool_choice: Some(ToolChoice::Named(NamedToolChoice {
                kind: "function".to_string(),
                function: FunctionName {
                    name: "get_weather".to_string(),
                },
            })),
            response_format: Some(ResponseFormat::JsonObject),
            ..Default::default()
        };
        roundtrip(&req);
    }

    #[test]
    fn parses_canonical_openai_request_json() {
        // Faithful to the documented chat-completions request shape.
        let raw = r#"{
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "user", "content": "hi"}
            ],
            "temperature": 0.2,
            "stop": "STOP",
            "tool_choice": "auto"
        }"#;
        let req: ChatCompletionRequest = serde_json::from_str(raw).expect("parse");
        assert_eq!(req.model, "gpt-4o-mini");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, Role::User);
        assert_eq!(
            req.messages[0].content,
            Some(MessageContent::Text("hi".to_string()))
        );
        assert_eq!(req.temperature, Some(0.2));
        assert_eq!(req.stop, Some(StringOrArray::Single("STOP".to_string())));
        assert_eq!(req.tool_choice, Some(ToolChoice::Mode(ToolChoiceMode::Auto)));
    }

    #[test]
    fn minimal_request_omits_none_fields() {
        let req = ChatCompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![ChatMessage {
                role: Role::User,
                content: Some(MessageContent::Text("hi".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("\"model\":\"gpt-4o\""));
        assert!(json.contains("\"messages\""));
        // Optional fields must NOT appear when unset (matches OpenAI SDK bodies).
        assert!(!json.contains("temperature"), "got: {json}");
        assert!(!json.contains("stream"), "got: {json}");
        assert!(!json.contains("tool_choice"), "got: {json}");
        // role serializes snake_case; content as a bare string (not tagged).
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"hi\""));
    }

    #[test]
    fn parses_canonical_response_with_tool_call_and_usage() {
        let raw = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_1",
                                "type": "function",
                                "function": {"name": "get_weather", "arguments": "{\"city\":\"NYC\"}"}
                            }
                        ]
                    },
                    "finish_reason": "tool_calls"
                }
            ],
            "usage": {"prompt_tokens": 9, "completion_tokens": 12, "total_tokens": 21},
            "system_fingerprint": "fp_44709d6fcb"
        }"#;
        let resp: ChatCompletionResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(resp.object, "chat.completion");
        assert_eq!(resp.choices.len(), 1);
        let choice = &resp.choices[0];
        assert_eq!(choice.finish_reason, Some(FinishReason::ToolCalls));
        assert_eq!(choice.message.role, Role::Assistant);
        assert_eq!(choice.message.content, None);
        let tool_calls = choice.message.tool_calls.as_ref().expect("tool calls");
        assert_eq!(tool_calls[0].id, "call_1");
        assert_eq!(tool_calls[0].function.name, "get_weather");
        let usage = resp.usage.as_ref().expect("usage");
        assert_eq!(usage.total_tokens, 21);
        // Re-serialize and re-parse to confirm structural stability.
        roundtrip(&resp);
    }

    #[test]
    fn usage_roundtrips_with_token_details() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            prompt_tokens_details: Some(PromptTokensDetails {
                cached_tokens: Some(40),
                audio_tokens: None,
            }),
            completion_tokens_details: Some(CompletionTokensDetails {
                reasoning_tokens: Some(8),
                ..Default::default()
            }),
        };
        roundtrip(&usage);
    }

    #[test]
    fn parses_streaming_content_chunk_and_final_usage_chunk() {
        let content_chunk = r#"{
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [
                {"index": 0, "delta": {"role": "assistant", "content": "Hel"}, "finish_reason": null}
            ]
        }"#;
        let chunk: ChatCompletionChunk = serde_json::from_str(content_chunk).expect("parse");
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices[0].delta.role, Some(Role::Assistant));
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hel"));
        assert_eq!(chunk.choices[0].finish_reason, None);
        roundtrip(&chunk);

        // Final usage-bearing chunk: empty delta + usage + finish_reason.
        let final_chunk = r#"{
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 7, "total_tokens": 10}
        }"#;
        let last: ChatCompletionChunk = serde_json::from_str(final_chunk).expect("parse");
        assert_eq!(last.choices[0].finish_reason, Some(FinishReason::Stop));
        assert_eq!(last.usage.expect("usage").total_tokens, 10);
    }

    #[test]
    fn parses_streaming_tool_call_delta() {
        let raw = r#"{
            "id": "chatcmpl-2",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "tool_calls": [
                            {"index": 0, "id": "call_9", "type": "function",
                             "function": {"name": "f", "arguments": "{\"a\":"}}
                        ]
                    },
                    "finish_reason": null
                }
            ]
        }"#;
        let chunk: ChatCompletionChunk = serde_json::from_str(raw).expect("parse");
        let tc = chunk.choices[0]
            .delta
            .tool_calls
            .as_ref()
            .expect("tool_calls");
        assert_eq!(tc[0].index, 0);
        assert_eq!(tc[0].id.as_deref(), Some("call_9"));
        assert_eq!(
            tc[0].function.as_ref().and_then(|f| f.arguments.as_deref()),
            Some("{\"a\":")
        );
    }

    #[test]
    fn parses_error_envelope() {
        let raw = r#"{
            "error": {
                "message": "Rate limit reached",
                "type": "rate_limit_error",
                "param": null,
                "code": "rate_limit_exceeded"
            }
        }"#;
        let err: ApiErrorResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(err.error.message, "Rate limit reached");
        assert_eq!(err.error.kind.as_deref(), Some("rate_limit_error"));
        assert_eq!(err.error.param, None);
        assert_eq!(err.error.code.as_deref(), Some("rate_limit_exceeded"));
        roundtrip(&err);
    }

    #[test]
    fn finish_reason_and_role_use_snake_case_wire_tokens() {
        assert_eq!(
            serde_json::to_string(&FinishReason::ToolCalls).expect("ser"),
            "\"tool_calls\""
        );
        assert_eq!(
            serde_json::to_string(&FinishReason::ContentFilter).expect("ser"),
            "\"content_filter\""
        );
        assert_eq!(serde_json::to_string(&Role::Tool).expect("ser"), "\"tool\"");
        assert_eq!(
            serde_json::to_string(&ToolChoiceMode::Required).expect("ser"),
            "\"required\""
        );
    }

    #[test]
    fn response_format_json_schema_keeps_arbitrary_document() {
        let rf = ResponseFormat::JsonSchema {
            json_schema: json!({"name": "out", "schema": {"type": "object"}}),
        };
        let s = serde_json::to_string(&rf).expect("ser");
        assert!(s.contains("\"type\":\"json_schema\""));
        roundtrip(&rf);
    }
}
