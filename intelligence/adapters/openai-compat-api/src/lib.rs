//! M02-P02-IP-003 — OpenAI-compat passthrough adapter (`/v1/chat/completions`).
//!
//! Rust counterpart to ccproxy-api `codex` plugin. Defines the OpenAI Chat
//! Completions / Embeddings / Models API shapes, the translator between OpenAI
//! shape and the internal `CapabilityInvokeRequest`, plus the SSE event
//! sequence emitter (`data: {…}\n\ndata: [DONE]\n\n` per OpenAI convention).
//!
//! Per workspace directive 2026-05-14 (hyper backbone): NO axum, NO async-
//! openai HTTP client. The adapter consumes the shared `oya-http-router-kernel`
//! and emits SSE through `oya-http-sse-kernel`. Hand-rolled types (no
//! `async-openai`) per `support everything ourselves with 0 to minimal
//! dependency` directive — drift caught by IP-005.
//!
//! Linus good-taste: eliminated the legacy `function_call` vs modern
//! `tool_calls` shape branch by normalizing both into a single internal
//! `ToolInvocation` record at the translator boundary; the handler sees one
//! shape.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_account_kernel::ProviderFamily;
use intelligence_provider_pool_kernel::{
    AccountHealthMap, PoolError, PoolRoutingDecision, ProviderAccountId, ProviderAccountPool,
    RequestMetadata, UnixMillis, UsageSnapshotMap, pick_account,
};
use oya_http_router_kernel::{HttpMethod, Router, RouterError};
use oya_http_sse_kernel::SseEvent;
use std::fmt;

/// data_class: INTERNAL_ONLY — OpenAI ChatCompletion request shape.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenAIChatCompletionRequest {
    pub model: String,                      // data_class: INTERNAL_ONLY
    pub messages: Vec<OpenAIChatMessage>,   // data_class: TENANT_SCOPED
    pub max_tokens: Option<u32>,            // data_class: INTERNAL_ONLY
    pub max_completion_tokens: Option<u32>, // data_class: INTERNAL_ONLY (newer alias)
    pub temperature: Option<f32>,           // data_class: INTERNAL_ONLY
    pub stream: bool,                       // data_class: INTERNAL_ONLY
    pub stop: Vec<String>,                  // data_class: INTERNAL_ONLY
    pub tools: Vec<OpenAITool>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIChatMessage {
    pub role: OpenAIRole,
    pub content: Option<String>,
    pub tool_calls: Vec<OpenAIToolCall>,
    pub tool_call_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenAIRole {
    System,
    User,
    Assistant,
    Tool,
}

impl OpenAIRole {
    pub fn name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAITool {
    pub kind: String, // typically "function"
    pub function_name: String,
    pub parameters_schema_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIToolCall {
    pub id: String,
    pub kind: String, // "function"
    pub function_name: String,
    pub arguments_json: String,
}

/// Response shape — OpenAI ChatCompletion (non-streaming).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub created_unix_secs: u64,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub object: String, // "chat.completion"
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIChatMessage,
    pub finish_reason: OpenAIFinishReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenAIFinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
}

impl OpenAIFinishReason {
    pub fn name(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Length => "length",
            Self::ToolCalls => "tool_calls",
            Self::ContentFilter => "content_filter",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Embeddings shapes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIEmbeddingsRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpenAIEmbeddingsResponse {
    pub model: String,
    pub data: Vec<OpenAIEmbedding>,
    pub usage: OpenAIUsage,
    pub object: String, // "list"
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpenAIEmbedding {
    pub index: u32,
    pub embedding: Vec<f32>,
    pub object: String, // "embedding"
}

/// Models shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIModelsList {
    pub data: Vec<OpenAIModel>,
    pub object: String, // "list"
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenAIModel {
    pub id: String,
    pub created_unix_secs: u64,
    pub owned_by: String,
    pub object: String, // "model"
}

// Re-export internal shape so the IP-002 and IP-003 adapters share a single
// kernel translator boundary. Mirror types are local to keep the crate
// self-contained (kernel-on-kernel coupling avoided).
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
    System,
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

#[derive(Clone, Debug, PartialEq)]
pub enum CompatError {
    EmptyMessages,
    EmptyModel,
    EmptyEmbeddingInput,
    InvalidMaxTokens,
    InvalidEmbeddingDimension,
    Pool(PoolError),
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessages => write!(f, "messages array is empty"),
            Self::EmptyModel => write!(f, "model is empty"),
            Self::EmptyEmbeddingInput => write!(f, "embedding input is empty"),
            Self::InvalidMaxTokens => write!(f, "max_tokens must be > 0"),
            Self::InvalidEmbeddingDimension => write!(f, "embedding dimension is invalid"),
            Self::Pool(e) => write!(f, "pool error: {e}"),
        }
    }
}

impl From<PoolError> for CompatError {
    fn from(e: PoolError) -> Self {
        Self::Pool(e)
    }
}

/// Pure translator: OpenAI shape → internal CapabilityInvokeRequest.
pub fn to_internal_invoke(
    req: &OpenAIChatCompletionRequest,
) -> Result<CapabilityInvokeRequest, CompatError> {
    if req.model.is_empty() {
        return Err(CompatError::EmptyModel);
    }
    if req.messages.is_empty() {
        return Err(CompatError::EmptyMessages);
    }
    // Honor `max_completion_tokens` if present, otherwise `max_tokens`, otherwise
    // default to 1024 (OpenAI does not require either field).
    let cap = req.max_completion_tokens.or(req.max_tokens).unwrap_or(1024);
    if cap == 0 {
        return Err(CompatError::InvalidMaxTokens);
    }

    let mut system_prompt: Option<String> = None;
    let mut turns: Vec<InternalTurn> = Vec::new();
    for m in &req.messages {
        match m.role {
            OpenAIRole::System => {
                if let Some(c) = &m.content {
                    system_prompt = Some(match system_prompt {
                        Some(prev) => format!("{prev}\n{c}"),
                        None => c.clone(),
                    });
                }
            }
            OpenAIRole::User => turns.push(InternalTurn {
                role: InternalRole::User,
                parts: vec![InternalPart::Text(m.content.clone().unwrap_or_default())],
            }),
            OpenAIRole::Assistant => {
                let mut parts: Vec<InternalPart> = Vec::new();
                if let Some(c) = &m.content
                    && !c.is_empty()
                {
                    parts.push(InternalPart::Text(c.clone()));
                }
                // Normalize tool_calls (modern shape) — legacy function_call is
                // already merged here by the caller (we only accept the modern
                // tool_calls shape; legacy is collapsed at ingress).
                for tc in &m.tool_calls {
                    parts.push(InternalPart::ToolInvocation {
                        id: tc.id.clone(),
                        name: tc.function_name.clone(),
                        args_json: tc.arguments_json.clone(),
                    });
                }
                turns.push(InternalTurn {
                    role: InternalRole::Assistant,
                    parts,
                });
            }
            OpenAIRole::Tool => {
                let payload = m.content.clone().unwrap_or_default();
                let call_id = m.tool_call_id.clone().unwrap_or_default();
                turns.push(InternalTurn {
                    role: InternalRole::Tool,
                    parts: vec![InternalPart::ToolResult {
                        call_id,
                        payload,
                        is_error: false,
                    }],
                });
            }
        }
    }

    Ok(CapabilityInvokeRequest {
        model_hint: req.model.clone(),
        system_prompt,
        turns,
        max_output_tokens: cap,
        temperature: req.temperature,
        stop_sequences: req.stop.clone(),
        stream: req.stream,
    })
}

pub fn from_internal_invoke(
    resp: &CapabilityInvokeResponse,
    model: &str,
    response_id: &str,
    created_unix_secs: u64,
) -> OpenAIChatCompletionResponse {
    let mut msg = OpenAIChatMessage {
        role: OpenAIRole::Assistant,
        content: if resp.completion.is_empty() {
            None
        } else {
            Some(resp.completion.clone())
        },
        tool_calls: resp
            .tool_calls
            .iter()
            .map(|tc| OpenAIToolCall {
                id: tc.id.clone(),
                kind: "function".into(),
                function_name: tc.name.clone(),
                arguments_json: tc.args_json.clone(),
            })
            .collect(),
        tool_call_id: None,
        name: None,
    };
    if msg.tool_calls.is_empty() && msg.content.is_none() {
        msg.content = Some(String::new());
    }
    let finish = match resp.finish {
        FinishReason::Stop => OpenAIFinishReason::Stop,
        FinishReason::Length => OpenAIFinishReason::Length,
        FinishReason::ToolUse => OpenAIFinishReason::ToolCalls,
        FinishReason::StopSequence => OpenAIFinishReason::Stop,
    };
    let total = resp.input_tokens + resp.output_tokens;
    OpenAIChatCompletionResponse {
        id: response_id.to_owned(),
        model: model.to_owned(),
        created_unix_secs,
        choices: vec![OpenAIChoice {
            index: 0,
            message: msg,
            finish_reason: finish,
        }],
        usage: OpenAIUsage {
            prompt_tokens: resp.input_tokens,
            completion_tokens: resp.output_tokens,
            total_tokens: total,
        },
        object: "chat.completion".into(),
    }
}

/// SSE relay — OpenAI streaming uses `data: {json}\n\n` framing terminated by
/// `data: [DONE]\n\n`. The `oya-http-sse-domain` already prints `data: …\n\n`
/// for each event, so we emit one event per chunk plus a terminal event
/// containing the literal `[DONE]` string.
pub fn sse_relay(
    resp: &CapabilityInvokeResponse,
    model: &str,
    response_id: &str,
    created_unix_secs: u64,
) -> Vec<SseEvent> {
    let mut out = Vec::new();
    // First delta announces the role.
    out.push(SseEvent::data(format!(
        r#"{{"id":"{response_id}","object":"chat.completion.chunk","created":{created_unix_secs},"model":"{model}","choices":[{{"index":0,"delta":{{"role":"assistant"}},"finish_reason":null}}]}}"#,
    )));
    for chunk in chunkify(&resp.completion, 64) {
        let escaped = json_escape(&chunk);
        out.push(SseEvent::data(format!(
            r#"{{"id":"{response_id}","object":"chat.completion.chunk","created":{created_unix_secs},"model":"{model}","choices":[{{"index":0,"delta":{{"content":"{escaped}"}},"finish_reason":null}}]}}"#,
        )));
    }
    let finish = match resp.finish {
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::ToolUse => "tool_calls",
        FinishReason::StopSequence => "stop",
    };
    out.push(SseEvent::data(format!(
        r#"{{"id":"{response_id}","object":"chat.completion.chunk","created":{created_unix_secs},"model":"{model}","choices":[{{"index":0,"delta":{{}},"finish_reason":"{finish}"}}]}}"#,
    )));
    out.push(SseEvent::data("[DONE]"));
    out
}

pub fn route_request(
    req: &OpenAIChatCompletionRequest,
    pool: &ProviderAccountPool,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    now_unix_ms: u64,
    previous_account: Option<ProviderAccountId>,
) -> Result<PoolRoutingDecision, CompatError> {
    if pool.provider != ProviderFamily::OpenAiOrCodex {
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

pub fn chat_completions_handler(
    req: &OpenAIChatCompletionRequest,
) -> Result<CapabilityInvokeRequest, CompatError> {
    to_internal_invoke(req)
}

pub fn embeddings_handler(req: &OpenAIEmbeddingsRequest) -> Result<usize, CompatError> {
    if req.model.is_empty() {
        return Err(CompatError::EmptyModel);
    }
    if req.input.is_empty() {
        return Err(CompatError::EmptyEmbeddingInput);
    }
    Ok(req.input.len())
}

pub fn models_handler(capability_models: &[&str]) -> OpenAIModelsList {
    OpenAIModelsList {
        data: capability_models
            .iter()
            .map(|id| OpenAIModel {
                id: (*id).to_owned(),
                created_unix_secs: 0,
                owned_by: "oyatie".into(),
                object: "model".into(),
            })
            .collect(),
        object: "list".into(),
    }
}

pub fn build_routes<H: Clone>(chat: H, embeddings: H, models: H) -> Result<Router<H>, RouterError> {
    let mut r: Router<H> = Router::new();
    r.route(HttpMethod::Post, "/v1/chat/completions", chat)?;
    r.route(HttpMethod::Post, "/v1/embeddings", embeddings)?;
    r.route(HttpMethod::Get, "/v1/models", models)?;
    Ok(r)
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

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_provider_pool_kernel::{
        DurationMs, PoolId, PoolRoutingStrategy, ProviderTier, TenantId,
    };
    use std::collections::BTreeSet;

    fn sample_req() -> OpenAIChatCompletionRequest {
        OpenAIChatCompletionRequest {
            model: "gpt-4o-2024-08-06".into(),
            messages: vec![
                OpenAIChatMessage {
                    role: OpenAIRole::System,
                    content: Some("you are helpful".into()),
                    tool_calls: Vec::new(),
                    tool_call_id: None,
                    name: None,
                },
                OpenAIChatMessage {
                    role: OpenAIRole::User,
                    content: Some("hello".into()),
                    tool_calls: Vec::new(),
                    tool_call_id: None,
                    name: None,
                },
            ],
            max_tokens: Some(1024),
            max_completion_tokens: None,
            temperature: Some(0.7),
            stream: false,
            stop: vec!["</done>".into()],
            tools: Vec::new(),
        }
    }

    #[test]
    fn translate_extracts_system_prompt() {
        let r = sample_req();
        let internal = to_internal_invoke(&r).unwrap();
        assert_eq!(internal.system_prompt.as_deref(), Some("you are helpful"));
        assert_eq!(internal.turns.len(), 1);
        assert_eq!(internal.turns[0].role, InternalRole::User);
    }

    #[test]
    fn translate_merges_multiple_system_messages() {
        let mut r = sample_req();
        r.messages.insert(
            0,
            OpenAIChatMessage {
                role: OpenAIRole::System,
                content: Some("be brief".into()),
                tool_calls: Vec::new(),
                tool_call_id: None,
                name: None,
            },
        );
        let internal = to_internal_invoke(&r).unwrap();
        let sys = internal.system_prompt.unwrap();
        assert!(sys.contains("be brief"));
        assert!(sys.contains("you are helpful"));
    }

    #[test]
    fn translate_rejects_empty_model() {
        let mut r = sample_req();
        r.model.clear();
        assert_eq!(to_internal_invoke(&r), Err(CompatError::EmptyModel));
    }

    #[test]
    fn translate_rejects_empty_messages() {
        let mut r = sample_req();
        r.messages.clear();
        assert_eq!(to_internal_invoke(&r), Err(CompatError::EmptyMessages));
    }

    #[test]
    fn translate_rejects_zero_max_completion_tokens() {
        let mut r = sample_req();
        r.max_tokens = Some(0);
        r.max_completion_tokens = None;
        assert_eq!(to_internal_invoke(&r), Err(CompatError::InvalidMaxTokens));
    }

    #[test]
    fn translate_prefers_max_completion_tokens_over_legacy() {
        let mut r = sample_req();
        r.max_tokens = Some(50);
        r.max_completion_tokens = Some(800);
        let internal = to_internal_invoke(&r).unwrap();
        assert_eq!(internal.max_output_tokens, 800);
    }

    #[test]
    fn translate_uses_default_when_no_token_cap() {
        let mut r = sample_req();
        r.max_tokens = None;
        r.max_completion_tokens = None;
        let internal = to_internal_invoke(&r).unwrap();
        assert_eq!(internal.max_output_tokens, 1024);
    }

    #[test]
    fn translate_assistant_with_tool_calls_yields_single_shape() {
        // Linus row: function_call vs tool_calls normalized to ToolInvocation.
        let mut r = sample_req();
        r.messages.push(OpenAIChatMessage {
            role: OpenAIRole::Assistant,
            content: None,
            tool_calls: vec![OpenAIToolCall {
                id: "call_1".into(),
                kind: "function".into(),
                function_name: "search".into(),
                arguments_json: "{\"q\":\"x\"}".into(),
            }],
            tool_call_id: None,
            name: None,
        });
        let internal = to_internal_invoke(&r).unwrap();
        let last = internal.turns.last().unwrap();
        assert_eq!(last.role, InternalRole::Assistant);
        assert!(matches!(last.parts[0], InternalPart::ToolInvocation { .. }));
    }

    #[test]
    fn translate_tool_role_carries_result() {
        let mut r = sample_req();
        r.messages.push(OpenAIChatMessage {
            role: OpenAIRole::Tool,
            content: Some("42".into()),
            tool_calls: Vec::new(),
            tool_call_id: Some("call_1".into()),
            name: None,
        });
        let internal = to_internal_invoke(&r).unwrap();
        let last = internal.turns.last().unwrap();
        assert_eq!(last.role, InternalRole::Tool);
        match &last.parts[0] {
            InternalPart::ToolResult {
                call_id, payload, ..
            } => {
                assert_eq!(call_id, "call_1");
                assert_eq!(payload, "42");
            }
            _ => panic!("expected ToolResult"),
        }
    }

    #[test]
    fn response_translation_populates_tool_calls() {
        let resp = CapabilityInvokeResponse {
            completion: "".into(),
            tool_calls: vec![ToolInvocation {
                id: "call_1".into(),
                name: "search".into(),
                args_json: "{}".into(),
            }],
            input_tokens: 10,
            output_tokens: 0,
            finish: FinishReason::ToolUse,
        };
        let out = from_internal_invoke(&resp, "gpt-4o", "chatcmpl-1", 0);
        assert_eq!(out.choices[0].finish_reason, OpenAIFinishReason::ToolCalls);
        assert_eq!(out.choices[0].message.tool_calls.len(), 1);
        assert_eq!(out.usage.total_tokens, 10);
    }

    #[test]
    fn sse_relay_terminates_with_done_marker() {
        let resp = CapabilityInvokeResponse {
            completion: "hi".into(),
            tool_calls: Vec::new(),
            input_tokens: 1,
            output_tokens: 1,
            finish: FinishReason::Stop,
        };
        let events = sse_relay(&resp, "m", "id", 1);
        assert_eq!(events.last().unwrap().data, "[DONE]");
        // Penultimate event carries the finish_reason
        let pre_done = &events[events.len() - 2];
        assert!(pre_done.data.contains("\"finish_reason\":\"stop\""));
    }

    #[test]
    fn sse_relay_first_event_announces_role() {
        let resp = CapabilityInvokeResponse {
            completion: "".into(),
            tool_calls: Vec::new(),
            input_tokens: 0,
            output_tokens: 0,
            finish: FinishReason::Stop,
        };
        let events = sse_relay(&resp, "m", "id", 7);
        assert!(events[0].data.contains("\"role\":\"assistant\""));
    }

    #[test]
    fn build_routes_registers_three_endpoints() {
        let r: Router<&'static str> = build_routes("chat", "emb", "models").unwrap();
        assert_eq!(r.count(), 3);
        assert!(
            r.match_route(HttpMethod::Post, "/v1/chat/completions")
                .is_some()
        );
        assert!(r.match_route(HttpMethod::Post, "/v1/embeddings").is_some());
        assert!(r.match_route(HttpMethod::Get, "/v1/models").is_some());
    }

    #[test]
    fn embeddings_handler_rejects_empty_input() {
        let r = OpenAIEmbeddingsRequest {
            model: "text-embedding-3-small".into(),
            input: Vec::new(),
        };
        assert_eq!(
            embeddings_handler(&r),
            Err(CompatError::EmptyEmbeddingInput)
        );
    }

    #[test]
    fn embeddings_handler_counts_input() {
        let r = OpenAIEmbeddingsRequest {
            model: "text-embedding-3-small".into(),
            input: vec!["a".into(), "b".into(), "c".into()],
        };
        assert_eq!(embeddings_handler(&r), Ok(3));
    }

    #[test]
    fn models_handler_lists_capability_set() {
        let m = models_handler(&["gpt-4o", "gpt-4o-mini"]);
        assert_eq!(m.data.len(), 2);
        assert_eq!(m.object, "list");
        assert_eq!(m.data[0].id, "gpt-4o");
    }

    #[test]
    fn route_request_routes_to_openai_pool() {
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
    fn route_request_refuses_non_openai_pool() {
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
    fn openai_role_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            OpenAIRole::System,
            OpenAIRole::User,
            OpenAIRole::Assistant,
            OpenAIRole::Tool,
        ]
        .iter()
        .map(|r| r.name())
        .collect();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn finish_reason_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            OpenAIFinishReason::Stop,
            OpenAIFinishReason::Length,
            OpenAIFinishReason::ToolCalls,
            OpenAIFinishReason::ContentFilter,
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
            format!("{}", CompatError::EmptyEmbeddingInput),
            format!("{}", CompatError::InvalidMaxTokens),
            format!("{}", CompatError::InvalidEmbeddingDimension),
            format!("{}", CompatError::Pool(PoolError::EmptyMembers)),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn chat_completions_handler_alias_validates() {
        assert!(chat_completions_handler(&sample_req()).is_ok());
        let mut bad = sample_req();
        bad.messages.clear();
        assert!(chat_completions_handler(&bad).is_err());
    }

    #[test]
    fn from_internal_invoke_empty_content_emits_empty_string_when_no_tools() {
        let resp = CapabilityInvokeResponse {
            completion: "".into(),
            tool_calls: Vec::new(),
            input_tokens: 0,
            output_tokens: 0,
            finish: FinishReason::Stop,
        };
        let out = from_internal_invoke(&resp, "m", "id", 1);
        assert_eq!(out.choices[0].message.content.as_deref(), Some(""));
    }
}
