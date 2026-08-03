//! M02-P02-IP-002 — Anthropic-compat passthrough adapter (`/v1/messages`).
//!
//! Rust counterpart to ccproxy-api `claude_api` plugin. Defines the Anthropic
//! Messages-API shape, the translator between that shape and the internal
//! `CapabilityInvokeRequest`, and the SSE event sequence emitter for streaming
//! responses. Built on the hyper backbone (router + SSE kernel) per workspace
//! directive 2026-05-14: NO axum, NO tower-http — we own the stack.
//!
//! Provider-specific code is bounded to this crate. The shared rotation kernel
//! lives in `intelligence-provider-pool-kernel`.
//!
//! Linus good-taste: streaming and non-streaming share one code path. Non-
//! streaming responses are a single chunk on the same stream surface — the
//! handler does not branch.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_sse_kernel::SseEvent;
use intelligence_account_kernel::ProviderFamily;
use intelligence_provider_pool_kernel::{
    AccountHealthMap, PoolError, PoolRoutingDecision, ProviderAccountId, ProviderAccountPool,
    RequestMetadata, UnixMillis, UsageSnapshotMap, pick_account,
};
use std::fmt;

pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";
pub const ANTHROPIC_BETA_HEADER: &str = "anthropic-beta";

/// data_class: INTERNAL_ONLY — Anthropic-shape Messages request.
#[derive(Clone, Debug, PartialEq)]
pub struct AnthropicMessagesRequest {
    pub model: String,                   // data_class: INTERNAL_ONLY
    pub system: Option<String>,          // data_class: TENANT_SCOPED
    pub messages: Vec<AnthropicMessage>, // data_class: TENANT_SCOPED
    pub max_tokens: u32,                 // data_class: INTERNAL_ONLY
    pub stream: bool,                    // data_class: INTERNAL_ONLY
    pub temperature: Option<f32>,        // data_class: INTERNAL_ONLY
    pub stop_sequences: Vec<String>,     // data_class: INTERNAL_ONLY
}

/// data_class: TENANT_SCOPED — individual message in Anthropic shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,                 // data_class: INTERNAL_ONLY
    pub content: Vec<AnthropicContentBlock>, // data_class: TENANT_SCOPED
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnthropicRole {
    User,
    Assistant,
}

impl AnthropicRole {
    pub fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnthropicContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input_json: String,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    Image {
        media_type: String,
        base64_data: String,
    },
}

/// Response shape Anthropic emits at the end of a successful Messages call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnthropicMessagesResponse {
    pub id: String,
    pub model: String,
    pub role: AnthropicRole,
    pub content: Vec<AnthropicContentBlock>,
    pub stop_reason: AnthropicStopReason,
    pub usage: AnthropicUsage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnthropicStopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

impl AnthropicStopReason {
    pub fn name(self) -> &'static str {
        match self {
            Self::EndTurn => "end_turn",
            Self::MaxTokens => "max_tokens",
            Self::StopSequence => "stop_sequence",
            Self::ToolUse => "tool_use",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Internal capability-invoke request — shared with IP-003 OpenAI adapter at
/// the translator boundary so the rotation kernel stays provider-agnostic.
#[derive(Clone, Debug, PartialEq)]
pub struct CapabilityInvokeRequest {
    pub model_hint: String,
    pub system_prompt: Option<String>,
    pub turns: Vec<InternalTurn>,
    pub max_output_tokens: u32,
    pub temperature: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternalTurn {
    pub role: InternalRole,
    pub parts: Vec<InternalPart>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternalRole {
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InternalPart {
    Text(String),
    ToolInvocation {
        id: String,
        name: String,
        args_json: String,
    },
    ToolResult {
        call_id: String,
        payload: String,
        is_error: bool,
    },
    Image {
        mime: String,
        base64: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeResponse {
    pub completion: String,
    pub tool_calls: Vec<ToolInvocation>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub finish: FinishReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolInvocation {
    pub id: String,
    pub name: String,
    pub args_json: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolUse,
    StopSequence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompatError {
    EmptyMessages,
    EmptyModel,
    InvalidMaxTokens,
    Pool(PoolError),
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessages => write!(f, "messages array is empty"),
            Self::EmptyModel => write!(f, "model is empty"),
            Self::InvalidMaxTokens => write!(f, "max_tokens must be > 0"),
            Self::Pool(e) => write!(f, "pool error: {e}"),
        }
    }
}

impl From<PoolError> for CompatError {
    fn from(e: PoolError) -> Self {
        Self::Pool(e)
    }
}

/// Pure translator: Anthropic shape → internal CapabilityInvokeRequest.
/// No I/O. Idempotent. Validates structural invariants up-front.
pub fn to_internal_invoke(
    req: &AnthropicMessagesRequest,
) -> Result<CapabilityInvokeRequest, CompatError> {
    if req.model.is_empty() {
        return Err(CompatError::EmptyModel);
    }
    if req.messages.is_empty() {
        return Err(CompatError::EmptyMessages);
    }
    if req.max_tokens == 0 {
        return Err(CompatError::InvalidMaxTokens);
    }
    let turns = req
        .messages
        .iter()
        .map(|m| InternalTurn {
            role: match m.role {
                AnthropicRole::User => InternalRole::User,
                AnthropicRole::Assistant => InternalRole::Assistant,
            },
            parts: m.content.iter().map(content_block_to_part).collect(),
        })
        .collect();
    Ok(CapabilityInvokeRequest {
        model_hint: req.model.clone(),
        system_prompt: req.system.clone(),
        turns,
        max_output_tokens: req.max_tokens,
        temperature: req.temperature,
        stop_sequences: req.stop_sequences.clone(),
        stream: req.stream,
    })
}

fn content_block_to_part(b: &AnthropicContentBlock) -> InternalPart {
    match b {
        AnthropicContentBlock::Text { text } => InternalPart::Text(text.clone()),
        AnthropicContentBlock::ToolUse {
            id,
            name,
            input_json,
        } => InternalPart::ToolInvocation {
            id: id.clone(),
            name: name.clone(),
            args_json: input_json.clone(),
        },
        AnthropicContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => InternalPart::ToolResult {
            call_id: tool_use_id.clone(),
            payload: content.clone(),
            is_error: *is_error,
        },
        AnthropicContentBlock::Image {
            media_type,
            base64_data,
        } => InternalPart::Image {
            mime: media_type.clone(),
            base64: base64_data.clone(),
        },
    }
}

/// Pure translator: internal response → Anthropic shape.
pub fn from_internal_invoke(
    resp: &CapabilityInvokeResponse,
    model: &str,
    response_id: &str,
) -> AnthropicMessagesResponse {
    let mut content = Vec::new();
    if !resp.completion.is_empty() {
        content.push(AnthropicContentBlock::Text {
            text: resp.completion.clone(),
        });
    }
    for tc in &resp.tool_calls {
        content.push(AnthropicContentBlock::ToolUse {
            id: tc.id.clone(),
            name: tc.name.clone(),
            input_json: tc.args_json.clone(),
        });
    }
    AnthropicMessagesResponse {
        id: response_id.to_owned(),
        model: model.to_owned(),
        role: AnthropicRole::Assistant,
        content,
        stop_reason: match resp.finish {
            FinishReason::Stop => AnthropicStopReason::EndTurn,
            FinishReason::Length => AnthropicStopReason::MaxTokens,
            FinishReason::ToolUse => AnthropicStopReason::ToolUse,
            FinishReason::StopSequence => AnthropicStopReason::StopSequence,
        },
        usage: AnthropicUsage {
            input_tokens: resp.input_tokens,
            output_tokens: resp.output_tokens,
        },
    }
}

/// Pure pool-routing entry: takes the parsed request + pool state and returns
/// the routing decision. Provider-agnostic — the kernel does the work; this
/// crate only wires it.
pub fn route_request(
    req: &AnthropicMessagesRequest,
    pool: &ProviderAccountPool,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    now_unix_ms: u64,
    previous_account: Option<ProviderAccountId>,
) -> Result<PoolRoutingDecision, CompatError> {
    if pool.provider != ProviderFamily::Claude {
        // Anthropic-compat may only route to Claude-family pools.
        return Err(CompatError::Pool(PoolError::EmptyMembers));
    }
    let mut meta = RequestMetadata::new(req.model.clone());
    meta.previous_account = previous_account;
    Ok(pick_account(
        pool,
        &meta,
        usage,
        health,
        UnixMillis(now_unix_ms),
    )?)
}

/// Build the SSE event sequence for a streaming Anthropic Messages response.
/// Mirrors the upstream Anthropic event order:
///   message_start, content_block_start, content_block_delta…,
///   content_block_stop, message_delta, message_stop.
///
/// Linus good-taste: non-streaming responses use the same sequence, collapsed
/// to a single terminal event by the egress layer.
pub fn sse_relay(resp: &CapabilityInvokeResponse, response_id: &str, model: &str) -> Vec<SseEvent> {
    let mut out = Vec::new();
    out.push(SseEvent::data(format!(
        r#"{{"type":"message_start","message":{{"id":"{response_id}","model":"{model}","role":"assistant"}}}}"#,
    ))
    .with_event("message_start"));
    out.push(
        SseEvent::data(
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        )
        .with_event("content_block_start"),
    );
    for chunk in chunkify(&resp.completion, 64) {
        let escaped = json_escape(&chunk);
        out.push(
            SseEvent::data(format!(
                r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{escaped}"}}}}"#,
            ))
            .with_event("content_block_delta"),
        );
    }
    out.push(
        SseEvent::data(r#"{"type":"content_block_stop","index":0}"#)
            .with_event("content_block_stop"),
    );
    let stop = match resp.finish {
        FinishReason::Stop => "end_turn",
        FinishReason::Length => "max_tokens",
        FinishReason::ToolUse => "tool_use",
        FinishReason::StopSequence => "stop_sequence",
    };
    out.push(
        SseEvent::data(format!(
            r#"{{"type":"message_delta","delta":{{"stop_reason":"{stop}"}},"usage":{{"output_tokens":{}}}}}"#,
            resp.output_tokens
        ))
        .with_event("message_delta"),
    );
    out.push(SseEvent::data(r#"{"type":"message_stop"}"#).with_event("message_stop"));
    out
}

fn chunkify(s: &str, n: usize) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.as_bytes()
        .chunks(n.max(1))
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect()
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Registered route surface (handler placeholders bound at the runtime layer).
pub fn build_routes<H: Clone>(
    messages_handler: H,
    count_tokens_handler: H,
) -> Result<Router<H>, RouterError> {
    let mut r: Router<H> = Router::new();
    r.route(HttpMethod::Post, "/v1/messages", messages_handler)?;
    r.route(
        HttpMethod::Get,
        "/v1/messages/count_tokens",
        count_tokens_handler,
    )?;
    Ok(r)
}

/// Pure helper: estimate token count from raw character length using a
/// 4-char/token rule of thumb (matches the upstream `count_tokens` rough
/// approximation; the runtime adapter can substitute the upstream call when
/// network is available).
pub fn count_tokens_handler(text: &str) -> u32 {
    let chars = text.chars().count() as u32;
    chars.div_ceil(4)
}

/// Pure helper: validate request invariants used by `messages_handler`
/// before any pool routing happens. Useful for hand-off into the runtime
/// adapter and for the IP-005 drift lane to introspect.
pub fn messages_handler(
    req: &AnthropicMessagesRequest,
) -> Result<CapabilityInvokeRequest, CompatError> {
    to_internal_invoke(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_provider_pool_kernel::{
        DurationMs, PoolId, PoolRoutingStrategy, ProviderTier, TenantId,
    };
    use std::collections::BTreeSet;

    fn sample_req() -> AnthropicMessagesRequest {
        AnthropicMessagesRequest {
            model: "claude-sonnet-4-6".into(),
            system: Some("you are helpful".into()),
            messages: vec![AnthropicMessage {
                role: AnthropicRole::User,
                content: vec![AnthropicContentBlock::Text {
                    text: "hello".into(),
                }],
            }],
            max_tokens: 1024,
            stream: false,
            temperature: Some(0.7),
            stop_sequences: vec!["</done>".into()],
        }
    }

    #[test]
    fn translate_basic_request() {
        let r = sample_req();
        let internal = to_internal_invoke(&r).unwrap();
        assert_eq!(internal.model_hint, "claude-sonnet-4-6");
        assert_eq!(internal.system_prompt.as_deref(), Some("you are helpful"));
        assert_eq!(internal.max_output_tokens, 1024);
        assert_eq!(internal.turns.len(), 1);
        assert_eq!(internal.turns[0].role, InternalRole::User);
    }

    #[test]
    fn translate_rejects_empty_messages() {
        let mut r = sample_req();
        r.messages.clear();
        assert_eq!(to_internal_invoke(&r), Err(CompatError::EmptyMessages));
    }

    #[test]
    fn translate_rejects_empty_model() {
        let mut r = sample_req();
        r.model.clear();
        assert_eq!(to_internal_invoke(&r), Err(CompatError::EmptyModel));
    }

    #[test]
    fn translate_rejects_zero_max_tokens() {
        let mut r = sample_req();
        r.max_tokens = 0;
        assert_eq!(to_internal_invoke(&r), Err(CompatError::InvalidMaxTokens));
    }

    #[test]
    fn translate_tool_use_block() {
        let r = AnthropicMessagesRequest {
            messages: vec![AnthropicMessage {
                role: AnthropicRole::Assistant,
                content: vec![AnthropicContentBlock::ToolUse {
                    id: "tu-1".into(),
                    name: "search".into(),
                    input_json: "{\"q\":\"x\"}".into(),
                }],
            }],
            ..sample_req()
        };
        let internal = to_internal_invoke(&r).unwrap();
        assert!(matches!(
            internal.turns[0].parts[0],
            InternalPart::ToolInvocation { .. }
        ));
    }

    #[test]
    fn translate_image_block() {
        let r = AnthropicMessagesRequest {
            messages: vec![AnthropicMessage {
                role: AnthropicRole::User,
                content: vec![AnthropicContentBlock::Image {
                    media_type: "image/png".into(),
                    base64_data: "iVBORw==".into(),
                }],
            }],
            ..sample_req()
        };
        let internal = to_internal_invoke(&r).unwrap();
        assert!(matches!(
            internal.turns[0].parts[0],
            InternalPart::Image { .. }
        ));
    }

    #[test]
    fn translate_tool_result_block_preserves_error_flag() {
        let r = AnthropicMessagesRequest {
            messages: vec![AnthropicMessage {
                role: AnthropicRole::User,
                content: vec![AnthropicContentBlock::ToolResult {
                    tool_use_id: "tu-1".into(),
                    content: "boom".into(),
                    is_error: true,
                }],
            }],
            ..sample_req()
        };
        let internal = to_internal_invoke(&r).unwrap();
        let part = &internal.turns[0].parts[0];
        match part {
            InternalPart::ToolResult { is_error, .. } => assert!(*is_error),
            _ => panic!("expected ToolResult"),
        }
    }

    #[test]
    fn response_translation_includes_tool_calls() {
        let resp = CapabilityInvokeResponse {
            completion: "hi".into(),
            tool_calls: vec![ToolInvocation {
                id: "tu-1".into(),
                name: "fn".into(),
                args_json: "{}".into(),
            }],
            input_tokens: 5,
            output_tokens: 2,
            finish: FinishReason::ToolUse,
        };
        let out = from_internal_invoke(&resp, "claude-sonnet-4-6", "msg_1");
        assert_eq!(out.stop_reason, AnthropicStopReason::ToolUse);
        assert_eq!(out.content.len(), 2); // text + tool_use
        assert_eq!(out.usage.output_tokens, 2);
    }

    #[test]
    fn response_translation_empty_completion_omits_text_block() {
        let resp = CapabilityInvokeResponse {
            completion: String::new(),
            tool_calls: Vec::new(),
            input_tokens: 1,
            output_tokens: 0,
            finish: FinishReason::Stop,
        };
        let out = from_internal_invoke(&resp, "m", "id-x");
        assert!(out.content.is_empty());
        assert_eq!(out.stop_reason, AnthropicStopReason::EndTurn);
    }

    #[test]
    fn sse_relay_emits_expected_event_order() {
        let resp = CapabilityInvokeResponse {
            completion: "hi".into(),
            tool_calls: Vec::new(),
            input_tokens: 1,
            output_tokens: 1,
            finish: FinishReason::Stop,
        };
        let events = sse_relay(&resp, "msg_1", "claude-sonnet-4-6");
        let names: Vec<&str> = events
            .iter()
            .map(|e| e.event.as_deref().unwrap_or(""))
            .collect();
        assert!(names.starts_with(&["message_start", "content_block_start"]));
        assert_eq!(names.last(), Some(&"message_stop"));
    }

    #[test]
    fn sse_relay_includes_tool_use_finish_reason() {
        let resp = CapabilityInvokeResponse {
            completion: "".into(),
            tool_calls: Vec::new(),
            input_tokens: 0,
            output_tokens: 0,
            finish: FinishReason::ToolUse,
        };
        let events = sse_relay(&resp, "x", "m");
        let combined: String = events.iter().map(|e| e.data.as_str()).collect();
        assert!(combined.contains("tool_use"));
    }

    #[test]
    fn build_routes_registers_both_endpoints() {
        let r: Router<&'static str> = build_routes("messages", "count_tokens").unwrap();
        assert_eq!(r.count(), 2);
        let (h, _, _) = r.match_route(HttpMethod::Post, "/v1/messages").unwrap();
        assert_eq!(*h, "messages");
        let (h, _, _) = r
            .match_route(HttpMethod::Get, "/v1/messages/count_tokens")
            .unwrap();
        assert_eq!(*h, "count_tokens");
    }

    #[test]
    fn count_tokens_handler_rough_estimate() {
        // 4 char/token rule
        assert_eq!(count_tokens_handler("aaaa"), 1);
        assert_eq!(count_tokens_handler("aaaaaaaa"), 2);
        assert_eq!(count_tokens_handler(""), 0);
        assert_eq!(count_tokens_handler("a"), 1);
    }

    #[test]
    fn messages_handler_alias_validates() {
        assert!(messages_handler(&sample_req()).is_ok());
        let mut bad = sample_req();
        bad.messages.clear();
        assert!(messages_handler(&bad).is_err());
    }

    #[test]
    fn route_request_routes_to_claude_pool() {
        let mut members = BTreeSet::new();
        members.insert(ProviderAccountId("a1".into()));
        let pool = ProviderAccountPool::new(
            PoolId("p".into()),
            ProviderFamily::Claude,
            ProviderTier::Pro,
            TenantId("t".into()),
            members,
            PoolRoutingStrategy::RoundRobin,
            DurationMs(60_000),
        );
        let d = route_request(
            &sample_req(),
            &pool,
            &Default::default(),
            &Default::default(),
            123,
            None,
        )
        .unwrap();
        assert_eq!(d.account_id, ProviderAccountId("a1".into()));
    }

    #[test]
    fn route_request_refuses_non_claude_pool() {
        let mut members = BTreeSet::new();
        members.insert(ProviderAccountId("a1".into()));
        let pool = ProviderAccountPool::new(
            PoolId("p".into()),
            ProviderFamily::OpenAiOrCodex,
            ProviderTier::Pro,
            TenantId("t".into()),
            members,
            PoolRoutingStrategy::RoundRobin,
            DurationMs(60_000),
        );
        let r = route_request(
            &sample_req(),
            &pool,
            &Default::default(),
            &Default::default(),
            123,
            None,
        );
        assert!(matches!(r, Err(CompatError::Pool(_))));
    }

    #[test]
    fn json_escape_quotes_and_newlines() {
        let out = json_escape("a\"b\nc\\d");
        assert!(out.contains("\\\""));
        assert!(out.contains("\\n"));
        assert!(out.contains("\\\\"));
    }

    #[test]
    fn anthropic_role_names_distinct() {
        assert_ne!(AnthropicRole::User.name(), AnthropicRole::Assistant.name());
    }

    #[test]
    fn stop_reason_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            AnthropicStopReason::EndTurn,
            AnthropicStopReason::MaxTokens,
            AnthropicStopReason::StopSequence,
            AnthropicStopReason::ToolUse,
        ]
        .iter()
        .map(|r| r.name())
        .collect();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn compat_error_display_distinct() {
        let m: Vec<String> = vec![
            format!("{}", CompatError::EmptyMessages),
            format!("{}", CompatError::EmptyModel),
            format!("{}", CompatError::InvalidMaxTokens),
            format!("{}", CompatError::Pool(PoolError::EmptyMembers)),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn api_version_constant_is_pinned() {
        assert_eq!(ANTHROPIC_API_VERSION, "2023-06-01");
    }

    #[test]
    fn chunkify_handles_empty_string() {
        assert!(chunkify("", 4).is_empty());
    }

    #[test]
    fn chunkify_respects_chunk_size() {
        let chunks = chunkify("aaaabbbb", 4);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "aaaa");
        assert_eq!(chunks[1], "bbbb");
    }
}
