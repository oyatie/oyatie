use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::error::{ClaudeAgentError, Result};
use crate::options::PermissionMode;
use crate::status::{FastModeState, SlashCommand};
use crate::tools::BuiltinToolInput;

/// Text, thinking, tool, and server-tool blocks found in assistant/user messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: Option<Value>,
        is_error: Option<bool>,
    },
    ServerToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ServerToolResult {
        tool_use_id: String,
        content: Value,
    },
    Unknown {
        data: Value,
    },
}

impl ContentBlock {
    pub fn builtin_tool_input(&self) -> Result<Option<BuiltinToolInput>> {
        match self {
            Self::ToolUse { name, input, .. } | Self::ServerToolUse { name, input, .. } => {
                BuiltinToolInput::parse(name, input.clone())
            }
            _ => Ok(None),
        }
    }
}

impl Serialize for ContentBlock {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Text { text } => serde_json::json!({"type": "text", "text": text}),
            Self::Thinking {
                thinking,
                signature,
            } => serde_json::json!({
                "type": "thinking",
                "thinking": thinking,
                "signature": signature,
            }),
            Self::ToolUse { id, name, input } => serde_json::json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": input,
            }),
            Self::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                let mut object = Map::new();
                object.insert("type".into(), Value::String("tool_result".into()));
                object.insert("tool_use_id".into(), Value::String(tool_use_id.clone()));
                if let Some(content) = content {
                    object.insert("content".into(), content.clone());
                }
                if let Some(is_error) = is_error {
                    object.insert("is_error".into(), Value::Bool(*is_error));
                }
                Value::Object(object)
            }
            Self::ServerToolUse { id, name, input } => serde_json::json!({
                "type": "server_tool_use",
                "id": id,
                "name": name,
                "input": input,
            }),
            Self::ServerToolResult {
                tool_use_id,
                content,
            } => serde_json::json!({
                "type": "advisor_tool_result",
                "tool_use_id": tool_use_id,
                "content": content,
            }),
            Self::Unknown { data } => data.clone(),
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContentBlock {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = Value::deserialize(deserializer)?;
        let Some(block_type) = data.get("type").and_then(Value::as_str) else {
            return Ok(Self::Unknown { data });
        };
        match block_type {
            "text" => Ok(Self::Text {
                text: required_string_for_content(&data, "text")?,
            }),
            "thinking" => Ok(Self::Thinking {
                thinking: required_string_for_content(&data, "thinking")?,
                signature: required_string_for_content(&data, "signature")?,
            }),
            "tool_use" => Ok(Self::ToolUse {
                id: required_string_for_content(&data, "id")?,
                name: required_string_for_content(&data, "name")?,
                input: data.get("input").cloned().unwrap_or(Value::Null),
            }),
            "tool_result" => Ok(Self::ToolResult {
                tool_use_id: required_string_for_content(&data, "tool_use_id")?,
                content: data.get("content").cloned(),
                is_error: data.get("is_error").and_then(Value::as_bool),
            }),
            "server_tool_use" => Ok(Self::ServerToolUse {
                id: required_string_for_content(&data, "id")?,
                name: required_string_for_content(&data, "name")?,
                input: data.get("input").cloned().unwrap_or(Value::Null),
            }),
            "advisor_tool_result" | "server_tool_result" => Ok(Self::ServerToolResult {
                tool_use_id: required_string_for_content(&data, "tool_use_id")?,
                content: data.get("content").cloned().unwrap_or(Value::Null),
            }),
            _ => Ok(Self::Unknown { data }),
        }
    }
}

fn required_string_for_content<E>(data: &Value, field: &str) -> std::result::Result<String, E>
where
    E: serde::de::Error,
{
    data.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| E::custom(format!("content block missing string field {field}")))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// Top-level SDK messages yielded by [`crate::query`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Status(StatusMessage),
    Result(Box<ResultMessage>),
    StreamEvent(StreamEvent),
    ToolProgress(ToolProgressMessage),
    AuthStatus(AuthStatusMessage),
    ToolUseSummary(ToolUseSummaryMessage),
    RateLimitEvent(RateLimitEvent),
    PromptSuggestion(PromptSuggestionMessage),
    TaskStarted(TaskStartedMessage),
    TaskProgress(TaskProgressMessage),
    TaskNotification(TaskNotificationMessage),
    TaskUpdated(TaskUpdatedMessage),
    HookEvent(HookEventMessage),
    LocalCommandOutput(LocalCommandOutputMessage),
    FilesPersisted(FilesPersistedMessage),
    ApiRetry(ApiRetryMessage),
    ElicitationComplete(ElicitationCompleteMessage),
    MemoryRecall(MemoryRecallMessage),
    Notification(NotificationMessage),
    PluginInstall(PluginInstallMessage),
    SessionStateChanged(SessionStateChangedMessage),
    PermissionDeniedEvent(PermissionDeniedEventMessage),
    ThinkingTokens(ThinkingTokensMessage),
    CommandsChanged(CommandsChangedMessage),
    CompactBoundary(CompactBoundaryMessage),
    MirrorError(MirrorErrorMessage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: UserContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<UserMessagePriority>,
    #[serde(
        default,
        rename = "isSynthetic",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_synthetic: Option<bool>,
    #[serde(
        default,
        rename = "shouldQuery",
        skip_serializing_if = "Option::is_none"
    )]
    pub should_query: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
    #[serde(default, rename = "isReplay", skip_serializing_if = "Option::is_none")]
    pub is_replay: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_attachments: Option<Vec<Value>>,
}

impl UserMessage {
    /// Create a user message with plain text content.
    pub fn text(content: impl Into<String>) -> Self {
        Self::new(UserContent::Text(content.into()))
    }

    /// Create a user message with structured content blocks.
    pub fn blocks(blocks: Vec<ContentBlock>) -> Self {
        Self::new(UserContent::Blocks(blocks))
    }

    fn new(content: UserContent) -> Self {
        Self {
            content,
            uuid: None,
            session_id: None,
            parent_tool_use_id: None,
            tool_use_result: None,
            priority: None,
            is_synthetic: None,
            should_query: None,
            timestamp: None,
            origin: None,
            subagent_type: None,
            task_description: None,
            is_replay: None,
            file_attachments: None,
        }
    }

    /// Set the parent tool-use ID for this user message.
    pub fn parent_tool_use_id(mut self, parent_tool_use_id: impl Into<String>) -> Self {
        self.parent_tool_use_id = Some(parent_tool_use_id.into());
        self
    }

    /// Set the session identifier associated with this user message.
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set whether this message should trigger an assistant turn.
    pub fn should_query(mut self, should_query: bool) -> Self {
        self.should_query = Some(should_query);
        self
    }

    /// Set the message scheduling priority.
    pub fn priority(mut self, priority: UserMessagePriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Mark the message as synthetic or human-authored.
    pub fn synthetic(mut self, is_synthetic: bool) -> Self {
        self.is_synthetic = Some(is_synthetic);
        self
    }

    /// Attach a provenance marker forwarded by Claude Code.
    pub fn origin(mut self, origin: MessageOrigin) -> Self {
        self.origin = Some(origin);
        self
    }
}

/// User message scheduling priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserMessagePriority {
    #[serde(rename = "now")]
    Now,
    #[serde(rename = "next")]
    Next,
    #[serde(rename = "later")]
    Later,
}

/// Provenance for user messages and their corresponding result messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MessageOrigin {
    Human,
    Channel {
        server: String,
    },
    Peer {
        #[serde(rename = "from")]
        from_: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    TaskNotification,
    Coordinator,
    AutoContinuation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<AssistantMessageError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
}

/// Assistant error literal emitted by Claude Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssistantMessageError {
    #[serde(rename = "authentication_failed")]
    AuthenticationFailed,
    #[serde(rename = "oauth_org_not_allowed")]
    OauthOrgNotAllowed,
    #[serde(rename = "billing_error")]
    BillingError,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "overloaded")]
    Overloaded,
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    #[serde(rename = "model_not_found")]
    ModelNotFound,
    #[serde(rename = "server_error")]
    ServerError,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "max_output_tokens")]
    MaxOutputTokens,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub status: Option<SdkStatus>,
    #[serde(
        default,
        rename = "permissionMode",
        skip_serializing_if = "Option::is_none"
    )]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_result: Option<CompactResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_error: Option<String>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SdkStatus {
    #[serde(rename = "compacting")]
    Compacting,
    #[serde(rename = "requesting")]
    Requesting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompactResult {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskUsage {
    pub total_tokens: u64,
    pub tool_uses: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStartedMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub task_id: String,
    pub description: String,
    pub uuid: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_transcript: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskProgressMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub task_id: String,
    pub description: String,
    pub usage: TaskUsage,
    pub uuid: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskNotificationMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub task_id: String,
    pub status: TaskNotificationStatus,
    pub output_file: String,
    pub summary: String,
    pub uuid: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<TaskUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_transcript: Option<bool>,
}

/// Status literal for task completion notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskNotificationStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "stopped")]
    Stopped,
}

/// System message emitted when a background task state changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskUpdatedMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub task_id: String,
    pub patch: TaskUpdatePatch,
    pub uuid: String,
    pub session_id: String,
}

/// Wire-safe subset of task state fields in a task update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskUpdatePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskUpdateStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_paused_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_backgrounded: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Status literal for task update patches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskUpdateStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "killed")]
    Killed,
    #[serde(rename = "paused")]
    Paused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorErrorMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<Value>,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalCommandOutputMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub content: String,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesPersistedMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub files: Vec<PersistedFile>,
    pub failed: Vec<FailedPersistedFile>,
    pub processed_at: String,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedFile {
    pub filename: String,
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedPersistedFile {
    pub filename: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiRetryMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub attempt: u64,
    pub max_retries: u64,
    pub retry_delay_ms: u64,
    pub error_status: Option<u16>,
    pub error: AssistantMessageError,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElicitationCompleteMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub mcp_server_name: String,
    pub elicitation_id: String,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecallMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub mode: MemoryRecallMode,
    pub memories: Vec<RecalledMemory>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryRecallMode {
    #[serde(rename = "select")]
    Select,
    #[serde(rename = "synthesize")]
    Synthesize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecalledMemory {
    pub path: String,
    pub scope: MemoryScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryScope {
    #[serde(rename = "personal")]
    Personal,
    #[serde(rename = "team")]
    Team,
    #[serde(rename = "organization")]
    Organization,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub key: String,
    pub text: String,
    pub priority: NotificationPriority,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationPriority {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "immediate")]
    Immediate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginInstallMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub status: PluginInstallStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginInstallStatus {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "installed")]
    Installed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStateChangedMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub state: SessionState,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "requires_action")]
    RequiresAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionDeniedEventMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub tool_name: String,
    pub tool_use_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_reason_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_reason: Option<String>,
    pub message: String,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThinkingTokensMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub estimated_tokens: u64,
    pub estimated_tokens_delta: u64,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandsChangedMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub commands: Vec<SlashCommand>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactBoundaryMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: Value,
    pub compact_metadata: CompactMetadata,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactMetadata {
    pub trigger: CompactTrigger,
    pub pre_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserved_segment: Option<PreservedSegment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserved_messages: Option<PreservedMessages>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompactTrigger {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservedSegment {
    pub head_uuid: String,
    pub anchor_uuid: String,
    pub tail_uuid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservedMessages {
    pub anchor_uuid: String,
    pub uuids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolProgressMessage {
    pub tool_use_id: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    pub elapsed_time_seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub uuid: String,
    pub session_id: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthStatusMessage {
    #[serde(rename = "isAuthenticating")]
    pub is_authenticating: bool,
    pub output: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub uuid: String,
    pub session_id: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolUseSummaryMessage {
    pub summary: String,
    pub preceding_tool_use_ids: Vec<String>,
    pub uuid: String,
    pub session_id: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptSuggestionMessage {
    pub suggestion: String,
    pub uuid: String,
    pub session_id: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferredToolUse {
    pub id: String,
    pub name: String,
    pub input: Map<String, Value>,
}

/// Per-model usage statistics returned in result messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelUsage {
    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,
    #[serde(rename = "cacheReadInputTokens")]
    pub cache_read_input_tokens: u64,
    #[serde(rename = "cacheCreationInputTokens")]
    pub cache_creation_input_tokens: u64,
    #[serde(rename = "webSearchRequests")]
    pub web_search_requests: u64,
    #[serde(rename = "costUSD")]
    pub cost_usd: f64,
    #[serde(rename = "contextWindow")]
    pub context_window: u64,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Information about a denied tool use in a result message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionDenial {
    pub tool_name: String,
    pub tool_use_id: String,
    pub tool_input: Map<String, Value>,
}

/// Why the query loop terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalReason {
    #[serde(rename = "blocking_limit")]
    BlockingLimit,
    #[serde(rename = "rapid_refill_breaker")]
    RapidRefillBreaker,
    #[serde(rename = "prompt_too_long")]
    PromptTooLong,
    #[serde(rename = "image_error")]
    ImageError,
    #[serde(rename = "model_error")]
    ModelError,
    #[serde(rename = "aborted_streaming")]
    AbortedStreaming,
    #[serde(rename = "aborted_tools")]
    AbortedTools,
    #[serde(rename = "stop_hook_prevented")]
    StopHookPrevented,
    #[serde(rename = "hook_stopped")]
    HookStopped,
    #[serde(rename = "tool_deferred")]
    ToolDeferred,
    #[serde(rename = "max_turns")]
    MaxTurns,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultMessage {
    pub subtype: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u64,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttft_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_request_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_request_from_spawn_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_usage: Option<BTreeMap<String, ModelUsage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_denials: Option<Vec<PermissionDenial>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deferred_tool_use: Option<DeferredToolUse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_error_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<TerminalReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast_mode_state: Option<FastModeState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamEvent {
    pub uuid: String,
    pub session_id: String,
    #[serde(default)]
    pub event: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttft_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub status: RateLimitStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_type: Option<RateLimitType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utilization: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overage_status: Option<RateLimitStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overage_resets_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overage_disabled_reason: Option<OverageDisabledReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_using_overage: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surpassed_threshold: Option<f64>,
    #[serde(default)]
    pub raw: Value,
}

/// Rate limit status literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RateLimitStatus {
    #[serde(rename = "allowed")]
    Allowed,
    #[serde(rename = "allowed_warning")]
    AllowedWarning,
    #[serde(rename = "rejected")]
    Rejected,
}

/// Claude.ai rate limit window type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RateLimitType {
    #[serde(rename = "five_hour")]
    FiveHour,
    #[serde(rename = "seven_day")]
    SevenDay,
    #[serde(rename = "seven_day_opus")]
    SevenDayOpus,
    #[serde(rename = "seven_day_sonnet")]
    SevenDaySonnet,
    #[serde(rename = "overage")]
    Overage,
}

/// Reason overage/pay-as-you-go usage is unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OverageDisabledReason {
    #[serde(rename = "overage_not_provisioned")]
    OverageNotProvisioned,
    #[serde(rename = "org_level_disabled")]
    OrgLevelDisabled,
    #[serde(rename = "org_level_disabled_until")]
    OrgLevelDisabledUntil,
    #[serde(rename = "out_of_credits")]
    OutOfCredits,
    #[serde(rename = "seat_tier_level_disabled")]
    SeatTierLevelDisabled,
    #[serde(rename = "member_level_disabled")]
    MemberLevelDisabled,
    #[serde(rename = "seat_tier_zero_credit_limit")]
    SeatTierZeroCreditLimit,
    #[serde(rename = "group_zero_credit_limit")]
    GroupZeroCreditLimit,
    #[serde(rename = "member_zero_credit_limit")]
    MemberZeroCreditLimit,
    #[serde(rename = "org_service_level_disabled")]
    OrgServiceLevelDisabled,
    #[serde(rename = "no_limits_configured")]
    NoLimitsConfigured,
    #[serde(rename = "fetch_error")]
    FetchError,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookEventMessage {
    pub subtype: String,
    pub hook_event_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<HookOutcome>,
    #[serde(default)]
    pub data: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Hook execution outcome literal for hook response messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookOutcome {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "cancelled")]
    Cancelled,
}

macro_rules! parse_message_with_raw_data {
    ($data:expr, $ty:ty) => {{
        let raw = $data;
        let mut message: $ty = serde_json::from_value(raw.clone())?;
        message.data = raw;
        Ok(message)
    }};
}

/// Parse one raw CLI JSON object into a typed SDK message.
pub fn parse_message(data: Value) -> Result<Option<Message>> {
    let object = data.as_object().ok_or_else(|| {
        ClaudeAgentError::message_parse("message must be a JSON object", data.clone())
    })?;
    let Some(message_type) = object.get("type").and_then(Value::as_str) else {
        return Err(ClaudeAgentError::message_parse(
            "message missing type",
            data,
        ));
    };

    match message_type {
        "user" => parse_user(data).map(|msg| Some(Message::User(msg))),
        "assistant" => parse_assistant(data).map(|msg| Some(Message::Assistant(msg))),
        "system" => parse_system(data).map(Some),
        "result" => parse_result(data).map(|msg| Some(Message::Result(Box::new(msg)))),
        "stream_event" => parse_stream_event(data).map(|msg| Some(Message::StreamEvent(msg))),
        "tool_progress" => parse_message_with_raw_data!(data, ToolProgressMessage)
            .map(|msg| Some(Message::ToolProgress(msg))),
        "auth_status" => parse_message_with_raw_data!(data, AuthStatusMessage)
            .map(|msg| Some(Message::AuthStatus(msg))),
        "tool_use_summary" => parse_message_with_raw_data!(data, ToolUseSummaryMessage)
            .map(|msg| Some(Message::ToolUseSummary(msg))),
        "rate_limit_event" => parse_rate_limit(data).map(|msg| Some(Message::RateLimitEvent(msg))),
        "prompt_suggestion" => parse_message_with_raw_data!(data, PromptSuggestionMessage)
            .map(|msg| Some(Message::PromptSuggestion(msg))),
        // Forward compatibility: unknown top-level messages are ignored like the Python SDK.
        _ => Ok(None),
    }
}

fn parse_user(data: Value) -> Result<UserMessage> {
    let message = data
        .get("message")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            ClaudeAgentError::message_parse("user message missing message object", data.clone())
        })?;
    let content_value = message.get("content").cloned().ok_or_else(|| {
        ClaudeAgentError::message_parse("user message missing content", data.clone())
    })?;
    let content = if let Some(text) = content_value.as_str() {
        UserContent::Text(text.to_owned())
    } else {
        UserContent::Blocks(serde_json::from_value(content_value)?)
    };
    Ok(UserMessage {
        content,
        uuid: optional_string(&data, "uuid"),
        session_id: optional_string(&data, "session_id"),
        parent_tool_use_id: optional_string(&data, "parent_tool_use_id"),
        tool_use_result: data.get("tool_use_result").cloned(),
        priority: optional_typed(&data, "priority")?,
        is_synthetic: optional_bool(&data, "isSynthetic"),
        should_query: optional_bool(&data, "shouldQuery"),
        timestamp: optional_string(&data, "timestamp"),
        origin: optional_typed(&data, "origin")?,
        subagent_type: optional_string(&data, "subagent_type"),
        task_description: optional_string(&data, "task_description"),
        is_replay: optional_bool(&data, "isReplay"),
        file_attachments: optional_typed(&data, "file_attachments")?,
    })
}

fn parse_assistant(data: Value) -> Result<AssistantMessage> {
    let message = data
        .get("message")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            ClaudeAgentError::message_parse(
                "assistant message missing message object",
                data.clone(),
            )
        })?;
    let content = message.get("content").cloned().ok_or_else(|| {
        ClaudeAgentError::message_parse("assistant message missing content", data.clone())
    })?;
    Ok(AssistantMessage {
        content: serde_json::from_value(content)?,
        model: required_string_obj(message, "model", &data)?,
        parent_tool_use_id: optional_string(&data, "parent_tool_use_id"),
        error: optional_typed(&data, "error")?,
        usage: message.get("usage").cloned(),
        message_id: message
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        stop_reason: message
            .get("stop_reason")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        stop_details: message.get("stop_details").cloned(),
        session_id: optional_string(&data, "session_id"),
        uuid: optional_string(&data, "uuid"),
        request_id: optional_string(&data, "request_id"),
        subagent_type: optional_string(&data, "subagent_type"),
        task_description: optional_string(&data, "task_description"),
    })
}

fn parse_system(data: Value) -> Result<Message> {
    let subtype = required_string(&data, "subtype")?;
    match subtype.as_str() {
        "status" => parse_message_with_raw_data!(data, StatusMessage).map(Message::Status),
        "hook_started" | "hook_progress" | "hook_response" => {
            parse_hook_event(subtype, data).map(Message::HookEvent)
        }
        "local_command_output" => parse_message_with_raw_data!(data, LocalCommandOutputMessage)
            .map(Message::LocalCommandOutput),
        "files_persisted" => {
            parse_message_with_raw_data!(data, FilesPersistedMessage).map(Message::FilesPersisted)
        }
        "api_retry" => parse_message_with_raw_data!(data, ApiRetryMessage).map(Message::ApiRetry),
        "elicitation_complete" => parse_message_with_raw_data!(data, ElicitationCompleteMessage)
            .map(Message::ElicitationComplete),
        "memory_recall" => {
            parse_message_with_raw_data!(data, MemoryRecallMessage).map(Message::MemoryRecall)
        }
        "notification" => {
            parse_message_with_raw_data!(data, NotificationMessage).map(Message::Notification)
        }
        "plugin_install" => {
            parse_message_with_raw_data!(data, PluginInstallMessage).map(Message::PluginInstall)
        }
        "session_state_changed" => parse_message_with_raw_data!(data, SessionStateChangedMessage)
            .map(Message::SessionStateChanged),
        "permission_denied" => parse_message_with_raw_data!(data, PermissionDeniedEventMessage)
            .map(Message::PermissionDeniedEvent),
        "thinking_tokens" => {
            parse_message_with_raw_data!(data, ThinkingTokensMessage).map(Message::ThinkingTokens)
        }
        "commands_changed" => {
            parse_message_with_raw_data!(data, CommandsChangedMessage).map(Message::CommandsChanged)
        }
        "compact_boundary" => {
            parse_message_with_raw_data!(data, CompactBoundaryMessage).map(Message::CompactBoundary)
        }
        "task_started" => Ok(Message::TaskStarted(TaskStartedMessage {
            subtype,
            task_id: required_string(&data, "task_id")?,
            description: required_string(&data, "description")?,
            uuid: required_string(&data, "uuid")?,
            session_id: required_string(&data, "session_id")?,
            tool_use_id: optional_string(&data, "tool_use_id"),
            task_type: optional_string(&data, "task_type"),
            subagent_type: optional_string(&data, "subagent_type"),
            workflow_name: optional_string(&data, "workflow_name"),
            prompt: optional_string(&data, "prompt"),
            skip_transcript: optional_bool(&data, "skip_transcript"),
            data,
        })),
        "task_progress" => {
            let usage = required_typed(&data, "usage")?;
            Ok(Message::TaskProgress(TaskProgressMessage {
                subtype,
                task_id: required_string(&data, "task_id")?,
                description: required_string(&data, "description")?,
                usage,
                uuid: required_string(&data, "uuid")?,
                session_id: required_string(&data, "session_id")?,
                tool_use_id: optional_string(&data, "tool_use_id"),
                subagent_type: optional_string(&data, "subagent_type"),
                last_tool_name: optional_string(&data, "last_tool_name"),
                summary: optional_string(&data, "summary"),
                data,
            }))
        }
        "task_notification" => {
            let status = required_typed(&data, "status")?;
            let usage = optional_typed(&data, "usage")?;
            Ok(Message::TaskNotification(TaskNotificationMessage {
                subtype,
                task_id: required_string(&data, "task_id")?,
                status,
                output_file: required_string(&data, "output_file")?,
                summary: required_string(&data, "summary")?,
                uuid: required_string(&data, "uuid")?,
                session_id: required_string(&data, "session_id")?,
                tool_use_id: optional_string(&data, "tool_use_id"),
                usage,
                skip_transcript: optional_bool(&data, "skip_transcript"),
                data,
            }))
        }
        "task_updated" => {
            let patch = required_typed(&data, "patch")?;
            Ok(Message::TaskUpdated(TaskUpdatedMessage {
                subtype,
                task_id: required_string(&data, "task_id")?,
                patch,
                uuid: required_string(&data, "uuid")?,
                session_id: required_string(&data, "session_id")?,
                data,
            }))
        }
        "mirror_error" => Ok(Message::MirrorError(MirrorErrorMessage {
            subtype,
            key: data.get("key").cloned(),
            error: optional_string(&data, "error").unwrap_or_default(),
            data,
        })),
        _ => Ok(Message::System(SystemMessage { subtype, data })),
    }
}

fn parse_hook_event(subtype: String, data: Value) -> Result<HookEventMessage> {
    Ok(HookEventMessage {
        subtype,
        hook_event_name: data
            .get("hook_event")
            .or_else(|| data.get("hook_event_name"))
            .or_else(|| data.get("hook_name"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        hook_id: optional_string(&data, "hook_id"),
        hook_name: optional_string(&data, "hook_name"),
        hook_event: optional_string(&data, "hook_event"),
        stdout: optional_string(&data, "stdout"),
        stderr: optional_string(&data, "stderr"),
        output: optional_string(&data, "output"),
        exit_code: data.get("exit_code").and_then(Value::as_i64),
        outcome: optional_typed(&data, "outcome")?,
        session_id: optional_string(&data, "session_id"),
        uuid: optional_string(&data, "uuid"),
        data,
    })
}

fn parse_result(data: Value) -> Result<ResultMessage> {
    let deferred_tool_use = data
        .get("deferred_tool_use")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?;
    let model_usage = data
        .get("modelUsage")
        .cloned()
        .or_else(|| data.get("model_usage").cloned())
        .map(serde_json::from_value)
        .transpose()?;
    let permission_denials = data
        .get("permission_denials")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?;
    let errors = data
        .get("errors")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?;
    Ok(ResultMessage {
        subtype: required_string(&data, "subtype")?,
        duration_ms: required_u64(&data, "duration_ms")?,
        duration_api_ms: required_u64(&data, "duration_api_ms")?,
        is_error: required_bool(&data, "is_error")?,
        num_turns: required_u64(&data, "num_turns")?,
        session_id: required_string(&data, "session_id")?,
        stop_reason: optional_string(&data, "stop_reason"),
        total_cost_usd: data.get("total_cost_usd").and_then(Value::as_f64),
        ttft_ms: data.get("ttft_ms").and_then(Value::as_u64),
        time_to_request_ms: data.get("time_to_request_ms").and_then(Value::as_u64),
        time_to_request_from_spawn_ms: data
            .get("time_to_request_from_spawn_ms")
            .and_then(Value::as_u64),
        usage: data.get("usage").cloned(),
        result: optional_string(&data, "result"),
        structured_output: data.get("structured_output").cloned(),
        model_usage,
        permission_denials,
        deferred_tool_use,
        errors,
        api_error_status: data
            .get("api_error_status")
            .and_then(Value::as_u64)
            .map(|v| v as u16),
        uuid: optional_string(&data, "uuid"),
        terminal_reason: optional_typed(&data, "terminal_reason")?,
        fast_mode_state: optional_typed(&data, "fast_mode_state")?,
        origin: optional_typed(&data, "origin")?,
    })
}

fn parse_stream_event(data: Value) -> Result<StreamEvent> {
    Ok(StreamEvent {
        uuid: required_string(&data, "uuid")?,
        session_id: required_string(&data, "session_id")?,
        event: data.get("event").cloned().unwrap_or(Value::Null),
        parent_tool_use_id: optional_string(&data, "parent_tool_use_id"),
        ttft_ms: data.get("ttft_ms").and_then(Value::as_u64),
    })
}

fn parse_rate_limit(data: Value) -> Result<RateLimitEvent> {
    let info = data.get("rate_limit_info").cloned().ok_or_else(|| {
        ClaudeAgentError::message_parse("rate_limit_event missing rate_limit_info", data.clone())
    })?;
    let raw = info.clone();
    let info_obj = info.as_object().ok_or_else(|| {
        ClaudeAgentError::message_parse("rate_limit_info must be object", data.clone())
    })?;
    Ok(RateLimitEvent {
        rate_limit_info: RateLimitInfo {
            status: required_typed_obj(info_obj, "status", &data)?,
            resets_at: info_obj.get("resetsAt").and_then(Value::as_i64),
            rate_limit_type: optional_typed_obj(info_obj, "rateLimitType", &data)?,
            utilization: info_obj.get("utilization").and_then(Value::as_f64),
            overage_status: optional_typed_obj(info_obj, "overageStatus", &data)?,
            overage_resets_at: info_obj.get("overageResetsAt").and_then(Value::as_i64),
            overage_disabled_reason: optional_typed_obj(info_obj, "overageDisabledReason", &data)?,
            is_using_overage: info_obj.get("isUsingOverage").and_then(Value::as_bool),
            surpassed_threshold: info_obj.get("surpassedThreshold").and_then(Value::as_f64),
            raw,
        },
        uuid: required_string(&data, "uuid")?,
        session_id: required_string(&data, "session_id")?,
    })
}

fn required_string(data: &Value, field: &str) -> Result<String> {
    data.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ClaudeAgentError::message_parse(format!("missing string field {field}"), data.clone())
        })
}

fn required_string_obj(object: &Map<String, Value>, field: &str, full: &Value) -> Result<String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ClaudeAgentError::message_parse(format!("missing string field {field}"), full.clone())
        })
}

fn required_u64(data: &Value, field: &str) -> Result<u64> {
    data.get(field).and_then(Value::as_u64).ok_or_else(|| {
        ClaudeAgentError::message_parse(format!("missing integer field {field}"), data.clone())
    })
}

fn required_bool(data: &Value, field: &str) -> Result<bool> {
    data.get(field).and_then(Value::as_bool).ok_or_else(|| {
        ClaudeAgentError::message_parse(format!("missing bool field {field}"), data.clone())
    })
}

fn required_typed<T>(data: &Value, field: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let value = data.get(field).cloned().ok_or_else(|| {
        ClaudeAgentError::message_parse(format!("missing field {field}"), data.clone())
    })?;
    serde_json::from_value(value).map_err(ClaudeAgentError::from)
}

fn required_typed_obj<T>(object: &Map<String, Value>, field: &str, full: &Value) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let value = object.get(field).cloned().ok_or_else(|| {
        ClaudeAgentError::message_parse(format!("missing field {field}"), full.clone())
    })?;
    serde_json::from_value(value).map_err(ClaudeAgentError::from)
}

fn optional_typed<T>(data: &Value, field: &str) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    data.get(field)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(ClaudeAgentError::from)
}

fn optional_typed_obj<T>(
    object: &Map<String, Value>,
    field: &str,
    _full: &Value,
) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    object
        .get(field)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(ClaudeAgentError::from)
}

fn optional_string(data: &Value, field: &str) -> Option<String> {
    data.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn optional_bool(data: &Value, field: &str) -> Option<bool> {
    data.get(field).and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_assistant_message_blocks() {
        let raw = json!({
            "type": "assistant",
            "session_id": "s1",
            "uuid": "u1",
            "message": {
                "id": "msg_1",
                "model": "claude-test",
                "stop_reason": "end_turn",
                "content": [
                    {"type": "text", "text": "hello"},
                    {"type": "tool_use", "id": "toolu_1", "name": "Read", "input": {"file_path": "a"}}
                ]
            }
        });
        let parsed = parse_message(raw).unwrap().unwrap();
        let Message::Assistant(message) = parsed else {
            panic!("wrong message")
        };
        assert_eq!(message.model, "claude-test");
        assert_eq!(message.content.len(), 2);
    }

    #[test]
    fn parses_result_message() {
        let raw = json!({
            "type": "result",
            "subtype": "success",
            "duration_ms": 10,
            "duration_api_ms": 8,
            "is_error": false,
            "num_turns": 1,
            "session_id": "s1",
            "result": "done",
            "modelUsage": {
                "claude-test": {
                    "inputTokens": 1,
                    "outputTokens": 2,
                    "cacheReadInputTokens": 0,
                    "cacheCreationInputTokens": 0,
                    "webSearchRequests": 0,
                    "costUSD": 0.01,
                    "contextWindow": 200000,
                    "maxOutputTokens": 8192
                }
            }
        });
        let parsed = parse_message(raw).unwrap().unwrap();
        let Message::Result(message) = parsed else {
            panic!("wrong message")
        };
        assert_eq!(message.result.as_deref(), Some("done"));
        assert_eq!(message.model_usage.unwrap()["claude-test"].output_tokens, 2);
    }

    #[test]
    fn parses_documented_message_metadata_shapes() {
        let user = parse_message(json!({
            "type": "user",
            "uuid": "u-user",
            "session_id": "s1",
            "parent_tool_use_id": null,
            "isSynthetic": true,
            "isReplay": true,
            "shouldQuery": false,
            "priority": "next",
            "timestamp": "2026-06-03T12:00:00Z",
            "subagent_type": "reviewer",
            "task_description": "Review the patch",
            "file_attachments": [{"file_id": "file_1"}],
            "origin": {"kind": "peer", "from": "agent-1", "name": "Reviewer"},
            "message": {"role": "user", "content": "context only"}
        }))
        .unwrap()
        .unwrap();
        let Message::User(user) = user else {
            panic!("wrong user message")
        };
        assert_eq!(user.session_id.as_deref(), Some("s1"));
        assert_eq!(user.is_synthetic, Some(true));
        assert_eq!(user.is_replay, Some(true));
        assert_eq!(user.should_query, Some(false));
        assert_eq!(user.priority, Some(UserMessagePriority::Next));
        assert_eq!(user.timestamp.as_deref(), Some("2026-06-03T12:00:00Z"));
        assert_eq!(user.subagent_type.as_deref(), Some("reviewer"));
        assert_eq!(user.task_description.as_deref(), Some("Review the patch"));
        assert_eq!(
            user.file_attachments.as_ref().unwrap()[0]["file_id"],
            "file_1"
        );
        assert!(matches!(
            user.origin,
            Some(MessageOrigin::Peer { ref from_, ref name })
                if from_ == "agent-1" && name.as_deref() == Some("Reviewer")
        ));

        let assistant = parse_message(json!({
            "type": "assistant",
            "uuid": "u-assistant",
            "session_id": "s1",
            "request_id": "req_1",
            "parent_tool_use_id": null,
            "error": "model_not_found",
            "subagent_type": "reviewer",
            "task_description": "Review the patch",
            "message": {
                "model": "claude-test",
                "content": [],
                "stop_reason": "refusal",
                "stop_details": {"reason": "safety"}
            }
        }))
        .unwrap()
        .unwrap();
        let Message::Assistant(assistant) = assistant else {
            panic!("wrong assistant message")
        };
        assert_eq!(assistant.error, Some(AssistantMessageError::ModelNotFound));
        assert_eq!(assistant.request_id.as_deref(), Some("req_1"));
        assert_eq!(assistant.subagent_type.as_deref(), Some("reviewer"));
        assert_eq!(
            assistant.task_description.as_deref(),
            Some("Review the patch")
        );
        assert_eq!(assistant.stop_reason.as_deref(), Some("refusal"));
        assert_eq!(assistant.stop_details.as_ref().unwrap()["reason"], "safety");

        let result = parse_message(json!({
            "type": "result",
            "subtype": "success",
            "duration_ms": 10,
            "duration_api_ms": 8,
            "ttft_ms": 2,
            "time_to_request_ms": 3,
            "time_to_request_from_spawn_ms": 4,
            "is_error": false,
            "api_error_status": null,
            "num_turns": 1,
            "session_id": "s1",
            "uuid": "u-result",
            "result": "done",
            "stop_reason": "end_turn",
            "total_cost_usd": 0.25,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5,
                "cache_creation_input_tokens": 0,
                "cache_read_input_tokens": 0,
                "cache_creation": {"ephemeral_5m_input_tokens": 0, "ephemeral_1h_input_tokens": 0},
                "server_tool_use": {},
                "service_tier": "standard",
                "speed": "fast",
                "inference_geo": "us",
                "iterations": {}
            },
            "modelUsage": {
                "claude-test": {
                    "inputTokens": 10,
                    "outputTokens": 5,
                    "cacheReadInputTokens": 1,
                    "cacheCreationInputTokens": 2,
                    "webSearchRequests": 0,
                    "costUSD": 0.25,
                    "contextWindow": 200000,
                    "maxOutputTokens": 8192
                }
            },
            "permission_denials": [{
                "tool_name": "Bash",
                "tool_use_id": "toolu_1",
                "tool_input": {"command": "rm -rf /tmp/nope"}
            }],
            "deferred_tool_use": {"id": "toolu_2", "name": "Edit", "input": {"file_path": "README.md"}},
            "terminal_reason": "completed",
            "fast_mode_state": "cooldown",
            "origin": {"kind": "task-notification"}
        }))
        .unwrap()
        .unwrap();
        let Message::Result(result) = result else {
            panic!("wrong result message")
        };
        assert_eq!(result.ttft_ms, Some(2));
        assert_eq!(result.time_to_request_ms, Some(3));
        assert_eq!(result.time_to_request_from_spawn_ms, Some(4));
        assert_eq!(result.terminal_reason, Some(TerminalReason::Completed));
        assert_eq!(
            result.fast_mode_state.as_ref().map(FastModeState::as_str),
            Some("cooldown")
        );
        assert!(matches!(
            result.origin,
            Some(MessageOrigin::TaskNotification)
        ));
        assert_eq!(
            result
                .model_usage
                .as_ref()
                .unwrap()
                .get("claude-test")
                .unwrap()
                .input_tokens,
            10
        );
        assert_eq!(
            result.permission_denials.as_ref().unwrap()[0].tool_name,
            "Bash"
        );
        assert_eq!(
            result.deferred_tool_use.as_ref().unwrap().input["file_path"],
            "README.md"
        );

        let stream_event = parse_message(json!({
            "type": "stream_event",
            "event": {"type": "content_block_delta", "delta": {"type": "text_delta", "text": "hi"}},
            "parent_tool_use_id": "toolu_parent",
            "uuid": "u-stream",
            "session_id": "s1",
            "ttft_ms": 7
        }))
        .unwrap()
        .unwrap();
        let Message::StreamEvent(stream_event) = stream_event else {
            panic!("wrong stream event")
        };
        assert_eq!(stream_event.ttft_ms, Some(7));
        assert_eq!(
            stream_event.parent_tool_use_id.as_deref(),
            Some("toolu_parent")
        );

        let task_progress = parse_message(json!({
            "type": "system",
            "subtype": "task_progress",
            "task_id": "task-1",
            "tool_use_id": "toolu_3",
            "description": "running",
            "subagent_type": "general-purpose",
            "usage": {"total_tokens": 20, "tool_uses": 2, "duration_ms": 50},
            "last_tool_name": "Read",
            "summary": "read files",
            "uuid": "u-task",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::TaskProgress(progress) = task_progress else {
            panic!("wrong task progress message")
        };
        assert_eq!(progress.usage.total_tokens, 20);
        assert_eq!(progress.subagent_type.as_deref(), Some("general-purpose"));
        assert_eq!(progress.summary.as_deref(), Some("read files"));

        let task_notification = parse_message(json!({
            "type": "system",
            "subtype": "task_notification",
            "task_id": "task-1",
            "status": "stopped",
            "output_file": "/tmp/out",
            "summary": "stopped",
            "usage": {"total_tokens": 20, "tool_uses": 2, "duration_ms": 50},
            "skip_transcript": true,
            "uuid": "u-notify",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::TaskNotification(notification) = task_notification else {
            panic!("wrong task notification message")
        };
        assert_eq!(notification.status, TaskNotificationStatus::Stopped);
        assert_eq!(notification.usage.as_ref().unwrap().tool_uses, 2);
        assert_eq!(notification.skip_transcript, Some(true));

        let task_updated = parse_message(json!({
            "type": "system",
            "subtype": "task_updated",
            "task_id": "task-1",
            "patch": {
                "status": "paused",
                "description": "waiting",
                "end_time": 1234,
                "total_paused_ms": 100,
                "error": "none",
                "is_backgrounded": true
            },
            "uuid": "u-update",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::TaskUpdated(updated) = task_updated else {
            panic!("wrong task updated message")
        };
        assert_eq!(updated.patch.status, Some(TaskUpdateStatus::Paused));
        assert_eq!(updated.patch.total_paused_ms, Some(100));

        let rate_limit = parse_message(json!({
            "type": "rate_limit_event",
            "rate_limit_info": {
                "status": "allowed_warning",
                "resetsAt": 123,
                "rateLimitType": "overage",
                "utilization": 0.8,
                "overageStatus": "allowed",
                "overageResetsAt": 456,
                "overageDisabledReason": "fetch_error",
                "isUsingOverage": true,
                "surpassedThreshold": 0.75
            },
            "uuid": "u-rate",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::RateLimitEvent(rate_limit) = rate_limit else {
            panic!("wrong rate limit message")
        };
        assert_eq!(
            rate_limit.rate_limit_info.status,
            RateLimitStatus::AllowedWarning
        );
        assert_eq!(
            rate_limit.rate_limit_info.rate_limit_type,
            Some(RateLimitType::Overage)
        );
        assert_eq!(
            rate_limit.rate_limit_info.overage_disabled_reason,
            Some(OverageDisabledReason::FetchError)
        );
        assert_eq!(rate_limit.rate_limit_info.is_using_overage, Some(true));
        assert_eq!(rate_limit.rate_limit_info.surpassed_threshold, Some(0.75));
    }

    #[test]
    fn user_message_builder_sets_session_id() {
        let message = UserMessage::text("hello").session_id("thread-42");
        let value = serde_json::to_value(message).unwrap();
        assert_eq!(value["session_id"], "thread-42");
    }

    #[test]
    fn parses_top_level_sdk_stream_messages() {
        let tool_progress = parse_message(json!({
            "type": "tool_progress",
            "tool_use_id": "toolu_1",
            "tool_name": "Bash",
            "parent_tool_use_id": null,
            "elapsed_time_seconds": 1.25,
            "task_id": "task-1",
            "uuid": "u-tool-progress",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::ToolProgress(tool_progress) = tool_progress else {
            panic!("wrong tool progress message")
        };
        assert_eq!(tool_progress.tool_use_id, "toolu_1");
        assert_eq!(tool_progress.tool_name, "Bash");
        assert_eq!(tool_progress.parent_tool_use_id, None);
        assert_eq!(tool_progress.elapsed_time_seconds, 1.25);
        assert_eq!(tool_progress.task_id.as_deref(), Some("task-1"));
        assert_eq!(tool_progress.data["type"], "tool_progress");

        let auth_status = parse_message(json!({
            "type": "auth_status",
            "isAuthenticating": true,
            "output": ["Open browser"],
            "error": "not yet",
            "uuid": "u-auth",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::AuthStatus(auth_status) = auth_status else {
            panic!("wrong auth status message")
        };
        assert!(auth_status.is_authenticating);
        assert_eq!(auth_status.output, vec!["Open browser"]);
        assert_eq!(auth_status.error.as_deref(), Some("not yet"));
        assert_eq!(auth_status.data["type"], "auth_status");

        let tool_summary = parse_message(json!({
            "type": "tool_use_summary",
            "summary": "Read and edited files",
            "preceding_tool_use_ids": ["toolu_1", "toolu_2"],
            "uuid": "u-summary",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::ToolUseSummary(tool_summary) = tool_summary else {
            panic!("wrong tool summary message")
        };
        assert_eq!(tool_summary.summary, "Read and edited files");
        assert_eq!(
            tool_summary.preceding_tool_use_ids,
            vec!["toolu_1", "toolu_2"]
        );
        assert_eq!(tool_summary.data["type"], "tool_use_summary");

        let prompt_suggestion = parse_message(json!({
            "type": "prompt_suggestion",
            "suggestion": "Run the tests",
            "uuid": "u-suggestion",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::PromptSuggestion(prompt_suggestion) = prompt_suggestion else {
            panic!("wrong prompt suggestion message")
        };
        assert_eq!(prompt_suggestion.suggestion, "Run the tests");
        assert_eq!(prompt_suggestion.data["type"], "prompt_suggestion");
    }

    #[test]
    fn parses_documented_system_stream_events() {
        let status = parse_message(json!({
            "type": "system",
            "subtype": "status",
            "status": "requesting",
            "permissionMode": "acceptEdits",
            "compact_result": "success",
            "uuid": "u-status",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::Status(status) = status else {
            panic!("wrong status message")
        };
        assert_eq!(status.status, Some(SdkStatus::Requesting));
        assert_eq!(status.permission_mode, Some(PermissionMode::AcceptEdits));
        assert_eq!(status.compact_result, Some(CompactResult::Success));
        assert_eq!(status.data["subtype"], "status");

        let local_output = parse_message(json!({
            "type": "system",
            "subtype": "local_command_output",
            "content": "usage summary",
            "uuid": "u-local",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::LocalCommandOutput(local_output) = local_output else {
            panic!("wrong local output message")
        };
        assert_eq!(local_output.content, "usage summary");
        assert_eq!(local_output.data["subtype"], "local_command_output");

        let hook_progress = parse_message(json!({
            "type": "system",
            "subtype": "hook_progress",
            "hook_id": "hook-1",
            "hook_name": "pre-tool",
            "hook_event": "PreToolUse",
            "stdout": "out",
            "stderr": "err",
            "output": "out\nerr",
            "uuid": "u-hook",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::HookEvent(hook_progress) = hook_progress else {
            panic!("wrong hook progress message")
        };
        assert_eq!(hook_progress.subtype, "hook_progress");
        assert_eq!(hook_progress.hook_event_name, "PreToolUse");
        assert_eq!(hook_progress.hook_id.as_deref(), Some("hook-1"));
        assert_eq!(hook_progress.hook_name.as_deref(), Some("pre-tool"));
        assert_eq!(hook_progress.stdout.as_deref(), Some("out"));
        assert_eq!(hook_progress.stderr.as_deref(), Some("err"));
        assert_eq!(hook_progress.output.as_deref(), Some("out\nerr"));

        let hook_response = parse_message(json!({
            "type": "system",
            "subtype": "hook_response",
            "hook_id": "hook-1",
            "hook_name": "pre-tool",
            "hook_event": "PreToolUse",
            "output": "done",
            "stdout": "done",
            "stderr": "",
            "exit_code": 0,
            "outcome": "success",
            "uuid": "u-hook-response",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::HookEvent(hook_response) = hook_response else {
            panic!("wrong hook response message")
        };
        assert_eq!(hook_response.exit_code, Some(0));
        assert_eq!(hook_response.outcome, Some(HookOutcome::Success));

        let files_persisted = parse_message(json!({
            "type": "system",
            "subtype": "files_persisted",
            "files": [{"filename": "src/lib.rs", "file_id": "file_1"}],
            "failed": [{"filename": "README.md", "error": "denied"}],
            "processed_at": "2026-06-03T12:00:00Z",
            "uuid": "u-files",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::FilesPersisted(files_persisted) = files_persisted else {
            panic!("wrong files persisted message")
        };
        assert_eq!(files_persisted.files[0].file_id, "file_1");
        assert_eq!(files_persisted.failed[0].error, "denied");

        let api_retry = parse_message(json!({
            "type": "system",
            "subtype": "api_retry",
            "attempt": 2,
            "max_retries": 5,
            "retry_delay_ms": 250,
            "error_status": null,
            "error": "rate_limit",
            "uuid": "u-retry",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::ApiRetry(api_retry) = api_retry else {
            panic!("wrong api retry message")
        };
        assert_eq!(api_retry.attempt, 2);
        assert_eq!(api_retry.error_status, None);
        assert_eq!(api_retry.error, AssistantMessageError::RateLimit);

        let permission_denied = parse_message(json!({
            "type": "system",
            "subtype": "permission_denied",
            "tool_name": "Bash",
            "tool_use_id": "toolu_denied",
            "agent_id": "agent-1",
            "decision_reason_type": "mode",
            "decision_reason": "dontAsk",
            "message": "Denied",
            "uuid": "u-denied",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::PermissionDeniedEvent(permission_denied) = permission_denied else {
            panic!("wrong permission denied message")
        };
        assert_eq!(permission_denied.tool_name, "Bash");
        assert_eq!(permission_denied.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(permission_denied.message, "Denied");

        let memory_recall = parse_message(json!({
            "type": "system",
            "subtype": "memory_recall",
            "mode": "synthesize",
            "memories": [{
                "path": "<synthesis:/memories>",
                "scope": "team",
                "content": "Prefer Rust."
            }],
            "uuid": "u-memory",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::MemoryRecall(memory_recall) = memory_recall else {
            panic!("wrong memory recall message")
        };
        assert_eq!(memory_recall.mode, MemoryRecallMode::Synthesize);
        assert_eq!(memory_recall.memories[0].scope, MemoryScope::Team);
        assert_eq!(
            memory_recall.memories[0].content.as_deref(),
            Some("Prefer Rust.")
        );

        let notification = parse_message(json!({
            "type": "system",
            "subtype": "notification",
            "key": "build",
            "text": "Build done",
            "priority": "high",
            "color": "green",
            "timeout_ms": 1000,
            "uuid": "u-notification",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::Notification(notification) = notification else {
            panic!("wrong notification message")
        };
        assert_eq!(notification.priority, NotificationPriority::High);
        assert_eq!(notification.timeout_ms, Some(1000));

        let plugin_install = parse_message(json!({
            "type": "system",
            "subtype": "plugin_install",
            "status": "installed",
            "name": "example",
            "uuid": "u-plugin",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::PluginInstall(plugin_install) = plugin_install else {
            panic!("wrong plugin install message")
        };
        assert_eq!(plugin_install.status, PluginInstallStatus::Installed);
        assert_eq!(plugin_install.name.as_deref(), Some("example"));

        let session_state = parse_message(json!({
            "type": "system",
            "subtype": "session_state_changed",
            "state": "requires_action",
            "uuid": "u-state",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::SessionStateChanged(session_state) = session_state else {
            panic!("wrong session state message")
        };
        assert_eq!(session_state.state, SessionState::RequiresAction);

        let thinking_tokens = parse_message(json!({
            "type": "system",
            "subtype": "thinking_tokens",
            "estimated_tokens": 100,
            "estimated_tokens_delta": 10,
            "uuid": "u-thinking",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::ThinkingTokens(thinking_tokens) = thinking_tokens else {
            panic!("wrong thinking tokens message")
        };
        assert_eq!(thinking_tokens.estimated_tokens, 100);
        assert_eq!(thinking_tokens.estimated_tokens_delta, 10);

        let commands_changed = parse_message(json!({
            "type": "system",
            "subtype": "commands_changed",
            "commands": [{
                "name": "test",
                "description": "Run tests",
                "argumentHint": "[filter]",
                "aliases": ["t"],
                "source": "project"
            }],
            "uuid": "u-commands",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::CommandsChanged(commands_changed) = commands_changed else {
            panic!("wrong commands changed message")
        };
        assert_eq!(commands_changed.commands[0].argument_hint, "[filter]");
        assert_eq!(commands_changed.commands[0].extra["source"], "project");

        let compact_boundary = parse_message(json!({
            "type": "system",
            "subtype": "compact_boundary",
            "compact_metadata": {
                "trigger": "auto",
                "pre_tokens": 1000,
                "post_tokens": 400,
                "duration_ms": 12,
                "preserved_segment": {
                    "head_uuid": "u-head",
                    "anchor_uuid": "u-anchor",
                    "tail_uuid": "u-tail"
                },
                "preserved_messages": {
                    "anchor_uuid": "u-anchor",
                    "uuids": ["u-1", "u-2"]
                },
                "strategy": "summary"
            },
            "uuid": "u-compact",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::CompactBoundary(compact_boundary) = compact_boundary else {
            panic!("wrong compact boundary message")
        };
        assert_eq!(
            compact_boundary.compact_metadata.trigger,
            CompactTrigger::Auto
        );
        assert_eq!(compact_boundary.compact_metadata.post_tokens, Some(400));
        assert_eq!(
            compact_boundary
                .compact_metadata
                .preserved_segment
                .as_ref()
                .unwrap()
                .anchor_uuid,
            "u-anchor"
        );
        assert_eq!(
            compact_boundary.compact_metadata.extra["strategy"],
            "summary"
        );

        let elicitation_complete = parse_message(json!({
            "type": "system",
            "subtype": "elicitation_complete",
            "mcp_server_name": "github",
            "elicitation_id": "elicit-1",
            "uuid": "u-elicit",
            "session_id": "s1"
        }))
        .unwrap()
        .unwrap();
        let Message::ElicitationComplete(elicitation_complete) = elicitation_complete else {
            panic!("wrong elicitation complete message")
        };
        assert_eq!(elicitation_complete.mcp_server_name, "github");
        assert_eq!(elicitation_complete.elicitation_id, "elicit-1");
    }

    #[test]
    fn skips_unknown_top_level_messages() {
        let raw = json!({"type": "future_message", "x": 1});
        assert!(parse_message(raw).unwrap().is_none());
    }

    #[test]
    fn preserves_unknown_content_block_payload() {
        let raw = json!({
            "type": "assistant",
            "message": {
                "model": "claude-test",
                "content": [
                    {"type": "future_block", "custom": {"nested": true}}
                ]
            }
        });
        let parsed = parse_message(raw).unwrap().unwrap();
        let Message::Assistant(message) = parsed else {
            panic!("wrong message")
        };
        let ContentBlock::Unknown { data } = &message.content[0] else {
            panic!("expected unknown content block")
        };
        assert_eq!(data["type"], "future_block");
        assert_eq!(data["custom"]["nested"], true);
        assert_eq!(
            serde_json::to_value(&message.content[0]).unwrap()["custom"]["nested"],
            true
        );
    }
}
