use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use tokio::sync::watch;

use crate::{
    error::{ClaudeAgentError, Result},
    messages::AssistantMessageError,
    options::PermissionUpdate,
    tools::{BuiltinToolInput, SdkMcpServer},
};

pub type PermissionFuture = Pin<Box<dyn Future<Output = Result<PermissionResult>> + Send>>;
pub type PermissionCallback = Arc<dyn Fn(ToolPermissionRequest) -> PermissionFuture + Send + Sync>;
pub type HookFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type HookCallback = Arc<dyn Fn(HookCallbackRequest) -> HookFuture + Send + Sync>;
pub type ElicitationFuture = Pin<Box<dyn Future<Output = Result<ElicitationResult>> + Send>>;
pub type ElicitationCallback =
    Arc<dyn Fn(ElicitationRequest, ElicitationCallbackOptions) -> ElicitationFuture + Send + Sync>;
pub type TokenRefreshFuture = Pin<Box<dyn Future<Output = Result<Option<String>>> + Send>>;
pub type TokenRefreshCallback =
    Arc<dyn Fn(TokenRefreshCallbackOptions) -> TokenRefreshFuture + Send + Sync>;
pub type UserDialogFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type UserDialogCallback =
    Arc<dyn Fn(UserDialogRequest, UserDialogCallbackOptions) -> UserDialogFuture + Send + Sync>;
pub type StderrCallback = Arc<dyn Fn(String) + Send + Sync>;

#[derive(Clone, Default)]
pub struct CallbackRegistry {
    pub can_use_tool: Option<PermissionCallback>,
    pub on_elicitation: Option<ElicitationCallback>,
    pub get_oauth_token: Option<TokenRefreshCallback>,
    pub get_host_auth_token: Option<TokenRefreshCallback>,
    pub on_user_dialog: Option<UserDialogCallback>,
    pub hooks: BTreeMap<String, Vec<HookMatcher>>,
    pub sdk_mcp_servers: BTreeMap<String, Arc<SdkMcpServer>>,
    pub stderr: Option<StderrCallback>,
}

impl std::fmt::Debug for CallbackRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field(
                "can_use_tool",
                &self.can_use_tool.as_ref().map(|_| "<callback>"),
            )
            .field(
                "on_elicitation",
                &self.on_elicitation.as_ref().map(|_| "<callback>"),
            )
            .field(
                "get_oauth_token",
                &self.get_oauth_token.as_ref().map(|_| "<callback>"),
            )
            .field(
                "get_host_auth_token",
                &self.get_host_auth_token.as_ref().map(|_| "<callback>"),
            )
            .field(
                "on_user_dialog",
                &self.on_user_dialog.as_ref().map(|_| "<callback>"),
            )
            .field("hooks", &self.hooks)
            .field(
                "sdk_mcp_servers",
                &self.sdk_mcp_servers.keys().collect::<Vec<_>>(),
            )
            .field("stderr", &self.stderr.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct HookMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCallback>,
    pub timeout: Option<f64>,
}

/// Hook event names exported by the upstream SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PostToolBatch,
    Notification,
    UserPromptSubmit,
    UserPromptExpansion,
    SessionStart,
    SessionEnd,
    Stop,
    StopFailure,
    SubagentStart,
    SubagentStop,
    PreCompact,
    PostCompact,
    PermissionRequest,
    PermissionDenied,
    Setup,
    TeammateIdle,
    TaskCreated,
    TaskCompleted,
    Elicitation,
    ElicitationResult,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    InstructionsLoaded,
    CwdChanged,
    FileChanged,
    MessageDisplay,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Debug for HookMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookMatcher")
            .field("matcher", &self.matcher)
            .field("hooks", &self.hooks.len())
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPermissionRequest {
    #[serde(alias = "tool_name")]
    pub tool_name: String,
    #[serde(default)]
    pub input: Value,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "permission_suggestions"
    )]
    pub permission_suggestions: Option<Vec<PermissionUpdate>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "blocked_path"
    )]
    pub blocked_path: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "decision_reason"
    )]
    pub decision_reason: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "decision_reason_type"
    )]
    pub decision_reason_type: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "classifier_approvable"
    )]
    pub classifier_approvable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "display_name"
    )]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(alias = "tool_use_id")]
    pub tool_use_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "agent_id")]
    pub agent_id: Option<String>,
}

impl ToolPermissionRequest {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "behavior",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum PermissionResult {
    Allow {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        updated_input: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        updated_permissions: Option<Vec<PermissionUpdate>>,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "toolUseID")]
        tool_use_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decision_classification: Option<PermissionDecisionClassification>,
    },
    Deny {
        message: String,
        #[serde(default)]
        interrupt: bool,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "toolUseID")]
        tool_use_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decision_classification: Option<PermissionDecisionClassification>,
    },
}

/// Telemetry classification for host permission decisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecisionClassification {
    UserTemporary,
    UserPermanent,
    UserReject,
    #[serde(untagged)]
    Other(String),
}

impl From<&str> for PermissionDecisionClassification {
    fn from(value: &str) -> Self {
        match value {
            "user_temporary" => Self::UserTemporary,
            "user_permanent" => Self::UserPermanent,
            "user_reject" => Self::UserReject,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl From<String> for PermissionDecisionClassification {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl PermissionResult {
    pub fn allow() -> Self {
        Self::Allow {
            updated_input: None,
            updated_permissions: None,
            tool_use_id: None,
            decision_classification: None,
        }
    }

    pub fn allow_with_updated_input(updated_input: Value) -> Self {
        Self::Allow {
            updated_input: Some(updated_input),
            updated_permissions: None,
            tool_use_id: None,
            decision_classification: None,
        }
    }

    pub fn deny(message: impl Into<String>) -> Self {
        Self::Deny {
            message: message.into(),
            interrupt: false,
            tool_use_id: None,
            decision_classification: None,
        }
    }

    pub fn deny_and_interrupt(message: impl Into<String>) -> Self {
        Self::Deny {
            message: message.into(),
            interrupt: true,
            tool_use_id: None,
            decision_classification: None,
        }
    }

    pub fn with_tool_use_id(mut self, tool_use_id: impl Into<String>) -> Self {
        match &mut self {
            Self::Allow {
                tool_use_id: id, ..
            }
            | Self::Deny {
                tool_use_id: id, ..
            } => {
                *id = Some(tool_use_id.into());
            }
        }
        self
    }

    pub fn with_decision_classification(
        mut self,
        decision_classification: impl Into<PermissionDecisionClassification>,
    ) -> Self {
        let decision_classification = decision_classification.into();
        match &mut self {
            Self::Allow {
                decision_classification: classification,
                ..
            }
            | Self::Deny {
                decision_classification: classification,
                ..
            } => {
                *classification = Some(decision_classification);
            }
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationRequest {
    #[serde(alias = "mcp_server_name", alias = "server_name")]
    pub server_name: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "elicitation_id"
    )]
    pub elicitation_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "requested_schema"
    )]
    pub requested_schema: Option<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "display_name"
    )]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    Form,
    Url,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationResult {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<Map<String, Value>>,
    pub action: ElicitationHookAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ElicitationResult {
    pub fn accept() -> Self {
        Self {
            meta: None,
            action: ElicitationHookAction::Accept,
            content: None,
            extra: Map::new(),
        }
    }

    pub fn accept_with_content(content: Map<String, Value>) -> Self {
        Self {
            meta: None,
            action: ElicitationHookAction::Accept,
            content: Some(content),
            extra: Map::new(),
        }
    }

    pub fn decline() -> Self {
        Self {
            meta: None,
            action: ElicitationHookAction::Decline,
            content: None,
            extra: Map::new(),
        }
    }

    pub fn cancel() -> Self {
        Self {
            meta: None,
            action: ElicitationHookAction::Cancel,
            content: None,
            extra: Map::new(),
        }
    }

    pub fn with_meta(mut self, meta: Map<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ElicitationCallbackOptions {
    pub signal: ElicitationAbortSignal,
}

pub type TokenRefreshCallbackOptions = ElicitationCallbackOptions;
pub type UserDialogCallbackOptions = ElicitationCallbackOptions;

impl ElicitationCallbackOptions {
    pub(crate) fn new() -> (Self, ElicitationAbortGuard) {
        let (options, guard, _) = Self::new_with_abort_handle();
        (options, guard)
    }

    pub(crate) fn new_with_abort_handle() -> (Self, ElicitationAbortGuard, ElicitationAbortHandle) {
        let (sender, receiver) = watch::channel(false);
        let abort_handle = ElicitationAbortHandle {
            sender: sender.clone(),
        };
        (
            Self {
                signal: ElicitationAbortSignal::new(receiver),
            },
            ElicitationAbortGuard {
                sender,
                completed: false,
            },
            abort_handle,
        )
    }
}

#[derive(Clone)]
pub struct ElicitationAbortSignal {
    receiver: watch::Receiver<bool>,
}

impl ElicitationAbortSignal {
    fn new(receiver: watch::Receiver<bool>) -> Self {
        Self { receiver }
    }

    pub fn is_aborted(&self) -> bool {
        *self.receiver.borrow()
    }

    pub async fn aborted(&mut self) -> bool {
        if self.is_aborted() {
            return true;
        }
        while self.receiver.changed().await.is_ok() {
            if self.is_aborted() {
                return true;
            }
        }
        self.is_aborted()
    }
}

impl std::fmt::Debug for ElicitationAbortSignal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ElicitationAbortSignal")
            .field("aborted", &self.is_aborted())
            .finish()
    }
}

pub(crate) struct ElicitationAbortGuard {
    sender: watch::Sender<bool>,
    completed: bool,
}

#[derive(Clone)]
pub(crate) struct ElicitationAbortHandle {
    sender: watch::Sender<bool>,
}

impl ElicitationAbortHandle {
    pub(crate) fn abort(&self) {
        let _ = self.sender.send(true);
    }
}

impl ElicitationAbortGuard {
    pub(crate) fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for ElicitationAbortGuard {
    fn drop(&mut self) {
        if !self.completed {
            let _ = self.sender.send(true);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDialogRequest {
    #[serde(alias = "dialog_kind")]
    pub dialog_kind: String,
    #[serde(default)]
    pub payload: Map<String, Value>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "tool_use_id"
    )]
    pub tool_use_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookCallbackRequest {
    pub callback_id: String,
    #[serde(default)]
    pub input: Value,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "tool_use_id"
    )]
    pub tool_use_id: Option<String>,
}

impl HookCallbackRequest {
    /// Parse the raw hook callback input into the documented hook-input union.
    pub fn hook_input(&self) -> Result<HookInput> {
        parse_hook_input(self.input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookBaseInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreToolUseHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_use_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

impl PreToolUseHookInput {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolUseHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_response: Value,
    pub tool_use_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

impl PostToolUseHookInput {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolUseFailureHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_use_id: String,
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_interrupt: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

impl PostToolUseFailureHookInput {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolBatchHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_calls: Vec<PostToolBatchToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolBatchToolCall {
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_use_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<Value>,
}

impl PostToolBatchToolCall {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub notification_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPromptSubmitHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPromptExpansionHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub expansion_type: String,
    pub command_name: String,
    pub command_args: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_source: Option<String>,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStartHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Session end reason literals exported by the upstream SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    Clear,
    Resume,
    Logout,
    PromptInputExit,
    Other,
    BypassPermissionsDisabled,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEndHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub reason: ExitReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundTaskSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCronSummary {
    pub id: String,
    pub schedule: String,
    pub recurring: bool,
    pub prompt: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub stop_hook_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_assistant_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_tasks: Option<Vec<BackgroundTaskSummary>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_crons: Option<Vec<SessionCronSummary>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopFailureHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub error: AssistantMessageError,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_assistant_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentStartHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub agent_id: String,
    pub agent_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentStopHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub stop_hook_active: bool,
    pub agent_id: String,
    pub agent_transcript_path: String,
    pub agent_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_assistant_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_tasks: Option<Vec<BackgroundTaskSummary>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_crons: Option<Vec<SessionCronSummary>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreCompactHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostCompactHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub trigger: String,
    pub compact_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequestHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_suggestions: Option<Vec<PermissionUpdate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

impl PermissionRequestHookInput {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionDeniedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_use_id: String,
    pub reason: String,
}

impl PermissionDeniedHookInput {
    pub fn builtin_input(&self) -> Result<Option<BuiltinToolInput>> {
        BuiltinToolInput::parse(&self.tool_name, self.tool_input.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub trigger: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeammateIdleHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub teammate_name: String,
    pub team_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCompletedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub task_id: String,
    pub task_subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teammate_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCreatedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub task_id: String,
    pub task_subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teammate_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigChangeHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CwdChangedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub old_cwd: String,
    pub new_cwd: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileChangedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub file_path: String,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstructionsLoadedHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub file_path: String,
    pub memory_type: String,
    pub load_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub globs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorktreeCreateHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorktreeRemoveHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub worktree_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageDisplayHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub turn_id: String,
    pub message_id: String,
    pub index: u64,
    #[serde(rename = "final")]
    pub final_: bool,
    pub delta: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElicitationHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub mcp_server_name: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElicitationResultHookInput {
    #[serde(flatten)]
    pub base: HookBaseInput,
    pub hook_event_name: String,
    pub mcp_server_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    pub action: ElicitationHookAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HookInput {
    PreToolUse(PreToolUseHookInput),
    PostToolUse(PostToolUseHookInput),
    PostToolUseFailure(PostToolUseFailureHookInput),
    PostToolBatch(PostToolBatchHookInput),
    Notification(NotificationHookInput),
    UserPromptSubmit(UserPromptSubmitHookInput),
    UserPromptExpansion(UserPromptExpansionHookInput),
    SessionStart(SessionStartHookInput),
    SessionEnd(SessionEndHookInput),
    Stop(StopHookInput),
    StopFailure(StopFailureHookInput),
    SubagentStart(SubagentStartHookInput),
    SubagentStop(SubagentStopHookInput),
    PreCompact(PreCompactHookInput),
    PostCompact(PostCompactHookInput),
    PermissionRequest(PermissionRequestHookInput),
    PermissionDenied(PermissionDeniedHookInput),
    Setup(SetupHookInput),
    TeammateIdle(TeammateIdleHookInput),
    TaskCompleted(TaskCompletedHookInput),
    TaskCreated(TaskCreatedHookInput),
    ConfigChange(ConfigChangeHookInput),
    CwdChanged(CwdChangedHookInput),
    FileChanged(FileChangedHookInput),
    InstructionsLoaded(InstructionsLoadedHookInput),
    WorktreeCreate(WorktreeCreateHookInput),
    WorktreeRemove(WorktreeRemoveHookInput),
    MessageDisplay(MessageDisplayHookInput),
    Elicitation(ElicitationHookInput),
    ElicitationResult(ElicitationResultHookInput),
    Unknown {
        hook_event_name: Option<String>,
        raw: Value,
    },
}

impl Serialize for HookInput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::PreToolUse(input) => input.serialize(serializer),
            Self::PostToolUse(input) => input.serialize(serializer),
            Self::PostToolUseFailure(input) => input.serialize(serializer),
            Self::PostToolBatch(input) => input.serialize(serializer),
            Self::Notification(input) => input.serialize(serializer),
            Self::UserPromptSubmit(input) => input.serialize(serializer),
            Self::UserPromptExpansion(input) => input.serialize(serializer),
            Self::SessionStart(input) => input.serialize(serializer),
            Self::SessionEnd(input) => input.serialize(serializer),
            Self::Stop(input) => input.serialize(serializer),
            Self::StopFailure(input) => input.serialize(serializer),
            Self::SubagentStart(input) => input.serialize(serializer),
            Self::SubagentStop(input) => input.serialize(serializer),
            Self::PreCompact(input) => input.serialize(serializer),
            Self::PostCompact(input) => input.serialize(serializer),
            Self::PermissionRequest(input) => input.serialize(serializer),
            Self::PermissionDenied(input) => input.serialize(serializer),
            Self::Setup(input) => input.serialize(serializer),
            Self::TeammateIdle(input) => input.serialize(serializer),
            Self::TaskCompleted(input) => input.serialize(serializer),
            Self::TaskCreated(input) => input.serialize(serializer),
            Self::ConfigChange(input) => input.serialize(serializer),
            Self::CwdChanged(input) => input.serialize(serializer),
            Self::FileChanged(input) => input.serialize(serializer),
            Self::InstructionsLoaded(input) => input.serialize(serializer),
            Self::WorktreeCreate(input) => input.serialize(serializer),
            Self::WorktreeRemove(input) => input.serialize(serializer),
            Self::MessageDisplay(input) => input.serialize(serializer),
            Self::Elicitation(input) => input.serialize(serializer),
            Self::ElicitationResult(input) => input.serialize(serializer),
            Self::Unknown { raw, .. } => raw.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for HookInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        parse_hook_input(Value::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookDecision {
    #[serde(rename = "block")]
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPermissionDecision {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ask")]
    Ask,
    #[serde(rename = "defer")]
    Defer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsyncHookJsonOutput {
    pub async_timeout: Option<u64>,
}

impl AsyncHookJsonOutput {
    pub fn new(async_timeout: Option<u64>) -> Self {
        Self { async_timeout }
    }
}

impl Serialize for AsyncHookJsonOutput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = Map::new();
        object.insert("async".into(), Value::Bool(true));
        if let Some(async_timeout) = self.async_timeout {
            object.insert("asyncTimeout".into(), async_timeout.into());
        }
        Value::Object(object).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AsyncHookJsonOutput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(rename = "async")]
            async_: bool,
            #[serde(default, rename = "asyncTimeout")]
            async_timeout: Option<u64>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if !wire.async_ {
            return Err(serde::de::Error::custom(
                "async hook output must set async to true",
            ));
        }
        Ok(Self {
            async_timeout: wire.async_timeout,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SyncHookJsonOutput {
    pub continue_: Option<bool>,
    pub suppress_output: Option<bool>,
    pub stop_reason: Option<String>,
    pub decision: Option<HookDecision>,
    pub system_message: Option<String>,
    pub reason: Option<String>,
    pub terminal_sequence: Option<String>,
    pub hook_specific_output: Option<HookSpecificOutput>,
    pub extra: Map<String, Value>,
}

impl SyncHookJsonOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn continue_(mut self, value: bool) -> Self {
        self.continue_ = Some(value);
        self
    }

    pub fn suppress_output(mut self, value: bool) -> Self {
        self.suppress_output = Some(value);
        self
    }

    pub fn stop_reason(mut self, value: impl Into<String>) -> Self {
        self.stop_reason = Some(value.into());
        self
    }

    pub fn decision(mut self, value: HookDecision) -> Self {
        self.decision = Some(value);
        self
    }

    pub fn system_message(mut self, value: impl Into<String>) -> Self {
        self.system_message = Some(value.into());
        self
    }

    pub fn reason(mut self, value: impl Into<String>) -> Self {
        self.reason = Some(value.into());
        self
    }

    pub fn terminal_sequence(mut self, value: impl Into<String>) -> Self {
        self.terminal_sequence = Some(value.into());
        self
    }

    pub fn hook_specific_output(mut self, value: HookSpecificOutput) -> Self {
        self.hook_specific_output = Some(value);
        self
    }

    pub fn extra(mut self, key: impl Into<String>, value: Value) -> Self {
        let key = key.into();
        if !is_sync_hook_output_reserved_extra_key(&key) {
            self.extra.insert(key, value);
        }
        self
    }
}

impl Serialize for SyncHookJsonOutput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = Map::new();
        for (key, value) in &self.extra {
            if !is_sync_hook_output_reserved_extra_key(key) {
                object.insert(key.clone(), value.clone());
            }
        }
        if let Some(value) = self.continue_ {
            object.insert("continue".into(), Value::Bool(value));
        }
        if let Some(value) = self.suppress_output {
            object.insert("suppressOutput".into(), Value::Bool(value));
        }
        if let Some(value) = &self.stop_reason {
            object.insert("stopReason".into(), Value::String(value.clone()));
        }
        if let Some(value) = self.decision {
            object.insert(
                "decision".into(),
                serde_json::to_value(value).map_err(serde::ser::Error::custom)?,
            );
        }
        if let Some(value) = &self.system_message {
            object.insert("systemMessage".into(), Value::String(value.clone()));
        }
        if let Some(value) = &self.reason {
            object.insert("reason".into(), Value::String(value.clone()));
        }
        if let Some(value) = &self.terminal_sequence {
            object.insert("terminalSequence".into(), Value::String(value.clone()));
        }
        if let Some(value) = &self.hook_specific_output {
            object.insert(
                "hookSpecificOutput".into(),
                serde_json::to_value(value).map_err(serde::ser::Error::custom)?,
            );
        }
        Value::Object(object).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SyncHookJsonOutput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            #[serde(default, rename = "continue")]
            continue_: Option<bool>,
            #[serde(default)]
            suppress_output: Option<bool>,
            #[serde(default)]
            stop_reason: Option<String>,
            #[serde(default)]
            decision: Option<HookDecision>,
            #[serde(default)]
            system_message: Option<String>,
            #[serde(default)]
            reason: Option<String>,
            #[serde(default)]
            terminal_sequence: Option<String>,
            #[serde(default)]
            hook_specific_output: Option<HookSpecificOutput>,
            #[serde(flatten)]
            extra: Map<String, Value>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if let Some(key) = wire
            .extra
            .keys()
            .find(|key| is_sync_hook_output_reserved_extra_key(key))
        {
            return Err(serde::de::Error::custom(format!(
                "sync hook output extra field cannot use reserved key {key}"
            )));
        }
        Ok(Self {
            continue_: wire.continue_,
            suppress_output: wire.suppress_output,
            stop_reason: wire.stop_reason,
            decision: wire.decision,
            system_message: wire.system_message,
            reason: wire.reason,
            terminal_sequence: wire.terminal_sequence,
            hook_specific_output: wire.hook_specific_output,
            extra: wire.extra,
        })
    }
}

fn is_sync_hook_output_reserved_extra_key(key: &str) -> bool {
    matches!(
        key,
        "async"
            | "asyncTimeout"
            | "continue"
            | "suppressOutput"
            | "stopReason"
            | "decision"
            | "systemMessage"
            | "reason"
            | "terminalSequence"
            | "hookSpecificOutput"
    )
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum HookJsonOutput {
    Async(AsyncHookJsonOutput),
    Sync(Box<SyncHookJsonOutput>),
}

impl<'de> Deserialize<'de> for HookJsonOutput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if value.get("async").is_some() {
            serde_json::from_value(value)
                .map(Self::Async)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(|output| Self::Sync(Box::new(output)))
                .map_err(serde::de::Error::custom)
        }
    }
}

impl From<AsyncHookJsonOutput> for HookJsonOutput {
    fn from(output: AsyncHookJsonOutput) -> Self {
        Self::Async(output)
    }
}

impl From<SyncHookJsonOutput> for HookJsonOutput {
    fn from(output: SyncHookJsonOutput) -> Self {
        Self::Sync(Box::new(output))
    }
}

impl From<AsyncHookJsonOutput> for Value {
    fn from(output: AsyncHookJsonOutput) -> Self {
        serde_json::to_value(output).expect("hook JSON output serialization should be infallible")
    }
}

impl From<SyncHookJsonOutput> for Value {
    fn from(output: SyncHookJsonOutput) -> Self {
        serde_json::to_value(output).expect("hook JSON output serialization should be infallible")
    }
}

impl From<HookJsonOutput> for Value {
    fn from(output: HookJsonOutput) -> Self {
        serde_json::to_value(output).expect("hook JSON output serialization should be infallible")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "hookEventName",
    rename_all = "PascalCase",
    rename_all_fields = "camelCase"
)]
pub enum HookSpecificOutput {
    PreToolUse {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        permission_decision: Option<HookPermissionDecision>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        permission_decision_reason: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        updated_input: Option<Map<String, Value>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    UserPromptSubmit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        suppress_original_prompt: Option<bool>,
    },
    UserPromptExpansion {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    SessionStart {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        initial_user_message: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reload_skills: Option<bool>,
    },
    Setup {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    SubagentStart {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    PostToolUse {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        updated_tool_output: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        updated_mcp_tool_output: Option<Value>,
    },
    PostToolUseFailure {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    PostToolBatch {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    MessageDisplay {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_content: Option<String>,
    },
    PermissionRequest {
        decision: PermissionResult,
    },
    PermissionDenied {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retry: Option<bool>,
    },
    WorktreeCreate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        worktree_path: Option<String>,
    },
    CwdChanged {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
    },
    FileChanged {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
    },
    Elicitation {
        action: ElicitationHookAction,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<Map<String, Value>>,
    },
    ElicitationResult {
        action: ElicitationHookAction,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<Map<String, Value>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElicitationHookAction {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "decline")]
    Decline,
    #[serde(rename = "cancel")]
    Cancel,
}

fn parse_hook_input(input: Value) -> Result<HookInput> {
    let hook_event_name = input
        .get("hook_event_name")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let Some(event_name) = hook_event_name.as_deref() else {
        return Ok(HookInput::Unknown {
            hook_event_name,
            raw: input,
        });
    };
    macro_rules! parse_variant {
        ($variant:ident, $ty:ty) => {
            serde_json::from_value::<$ty>(input)
                .map(HookInput::$variant)
                .map_err(ClaudeAgentError::from)
        };
    }
    match event_name {
        "PreToolUse" => parse_variant!(PreToolUse, PreToolUseHookInput),
        "PostToolUse" => parse_variant!(PostToolUse, PostToolUseHookInput),
        "PostToolUseFailure" => parse_variant!(PostToolUseFailure, PostToolUseFailureHookInput),
        "PostToolBatch" => parse_variant!(PostToolBatch, PostToolBatchHookInput),
        "Notification" => parse_variant!(Notification, NotificationHookInput),
        "UserPromptSubmit" => parse_variant!(UserPromptSubmit, UserPromptSubmitHookInput),
        "UserPromptExpansion" => {
            parse_variant!(UserPromptExpansion, UserPromptExpansionHookInput)
        }
        "SessionStart" => parse_variant!(SessionStart, SessionStartHookInput),
        "SessionEnd" => parse_variant!(SessionEnd, SessionEndHookInput),
        "Stop" => parse_variant!(Stop, StopHookInput),
        "StopFailure" => parse_variant!(StopFailure, StopFailureHookInput),
        "SubagentStart" => parse_variant!(SubagentStart, SubagentStartHookInput),
        "SubagentStop" => parse_variant!(SubagentStop, SubagentStopHookInput),
        "PreCompact" => parse_variant!(PreCompact, PreCompactHookInput),
        "PostCompact" => parse_variant!(PostCompact, PostCompactHookInput),
        "PermissionRequest" => parse_variant!(PermissionRequest, PermissionRequestHookInput),
        "PermissionDenied" => parse_variant!(PermissionDenied, PermissionDeniedHookInput),
        "Setup" => parse_variant!(Setup, SetupHookInput),
        "TeammateIdle" => parse_variant!(TeammateIdle, TeammateIdleHookInput),
        "TaskCompleted" => parse_variant!(TaskCompleted, TaskCompletedHookInput),
        "TaskCreated" => parse_variant!(TaskCreated, TaskCreatedHookInput),
        "ConfigChange" => parse_variant!(ConfigChange, ConfigChangeHookInput),
        "CwdChanged" => parse_variant!(CwdChanged, CwdChangedHookInput),
        "FileChanged" => parse_variant!(FileChanged, FileChangedHookInput),
        "InstructionsLoaded" => {
            parse_variant!(InstructionsLoaded, InstructionsLoadedHookInput)
        }
        "WorktreeCreate" => parse_variant!(WorktreeCreate, WorktreeCreateHookInput),
        "WorktreeRemove" => parse_variant!(WorktreeRemove, WorktreeRemoveHookInput),
        "MessageDisplay" => parse_variant!(MessageDisplay, MessageDisplayHookInput),
        "Elicitation" => parse_variant!(Elicitation, ElicitationHookInput),
        "ElicitationResult" => {
            parse_variant!(ElicitationResult, ElicitationResultHookInput)
        }
        _ => Ok(HookInput::Unknown {
            hook_event_name,
            raw: input,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base_fields() -> Value {
        json!({
            "session_id": "session-1",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/workspace",
            "permission_mode": "default"
        })
    }

    #[test]
    fn elicitation_types_parse_wire_shape_and_preserve_result_extensions() {
        let request: ElicitationRequest = serde_json::from_value(json!({
            "subtype": "elicitation",
            "mcp_server_name": "github",
            "message": "Authorize GitHub",
            "mode": "form",
            "requested_schema": {"type": "object"},
            "display_name": "GitHub",
            "elicitation_id": "elicit-1"
        }))
        .unwrap();
        assert_eq!(request.server_name, "github");
        assert_eq!(request.mode, Some(ElicitationMode::Form));
        assert_eq!(request.display_name.as_deref(), Some("GitHub"));
        assert_eq!(request.elicitation_id.as_deref(), Some("elicit-1"));

        let result = ElicitationResult::accept_with_content(
            json!({"account": "octo"}).as_object().unwrap().clone(),
        )
        .with_meta(json!({"progressToken": "p1"}).as_object().unwrap().clone())
        .with_extra("extension", json!(true));
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({
                "_meta": {"progressToken": "p1"},
                "action": "accept",
                "content": {"account": "octo"},
                "extension": true
            })
        );
    }

    #[test]
    fn hook_callback_request_parses_documented_pre_tool_use_input() {
        let mut input = base_fields();
        input.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": {"command": "pwd", "timeout": 1000},
                "tool_use_id": "toolu_1",
                "agent_id": "agent-1",
                "agent_type": "general-purpose",
                "new_field": "preserved"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let request = HookCallbackRequest {
            callback_id: "callback-1".into(),
            input,
            tool_use_id: Some("toolu_1".into()),
        };

        let parsed = request.hook_input().unwrap();
        let HookInput::PreToolUse(pre_tool_use) = &parsed else {
            panic!("expected PreToolUse hook input");
        };
        assert_eq!(pre_tool_use.base.session_id, "session-1");
        assert_eq!(pre_tool_use.base.cwd, "/workspace");
        assert_eq!(pre_tool_use.base.extra["new_field"], "preserved");
        assert_eq!(pre_tool_use.tool_use_id, "toolu_1");
        assert_eq!(pre_tool_use.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(pre_tool_use.agent_type.as_deref(), Some("general-purpose"));
        let Some(BuiltinToolInput::Bash(bash)) = pre_tool_use.builtin_input().unwrap() else {
            panic!("expected Bash input");
        };
        assert_eq!(bash.command, "pwd");

        let serialized = serde_json::to_value(&parsed).unwrap();
        assert_eq!(serialized["hook_event_name"], "PreToolUse");
        assert_eq!(serialized["final"], Value::Null);
        let reparsed: HookInput = serde_json::from_value(serialized).unwrap();
        assert!(matches!(reparsed, HookInput::PreToolUse(_)));
    }

    #[test]
    fn tool_permission_request_preserves_current_reason_metadata() {
        let request = serde_json::from_value::<ToolPermissionRequest>(json!({
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "input": {"command": "rm -rf /tmp/example"},
            "decision_reason": "safety check requires approval",
            "decision_reason_type": "safetyCheck",
            "classifier_approvable": false,
            "tool_use_id": "toolu_1"
        }))
        .unwrap();

        assert_eq!(request.decision_reason_type.as_deref(), Some("safetyCheck"));
        assert_eq!(request.classifier_approvable, Some(false));
    }

    #[test]
    fn hook_input_parses_documented_batch_and_message_display_shapes() {
        let mut batch = base_fields();
        batch.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "PostToolBatch",
                "tool_calls": [{
                    "tool_name": "Read",
                    "tool_input": {"file_path": "README.md"},
                    "tool_use_id": "toolu_read",
                    "tool_response": [{"type": "text", "text": "ok"}]
                }]
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::PostToolBatch(batch) = parse_hook_input(batch).unwrap() else {
            panic!("expected PostToolBatch hook input");
        };
        assert_eq!(batch.tool_calls[0].tool_name, "Read");
        assert!(batch.tool_calls[0].tool_response.is_some());

        let mut display = base_fields();
        display.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "MessageDisplay",
                "turn_id": "turn-1",
                "message_id": "msg-1",
                "index": 2,
                "final": true,
                "delta": "hello"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::MessageDisplay(display) = parse_hook_input(display).unwrap() else {
            panic!("expected MessageDisplay hook input");
        };
        assert!(display.final_);
        assert_eq!(
            serde_json::to_value(HookInput::MessageDisplay(display)).unwrap()["final"],
            true
        );
    }

    #[test]
    fn hook_input_parses_current_package_exported_lifecycle_and_elicitation_shapes() {
        let mut expansion = base_fields();
        expansion.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "UserPromptExpansion",
                "expansion_type": "slash_command",
                "command_name": "/review",
                "command_args": "--staged",
                "command_source": "project",
                "prompt": "review staged changes"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::UserPromptExpansion(expansion) = parse_hook_input(expansion).unwrap() else {
            panic!("expected UserPromptExpansion hook input");
        };
        assert_eq!(expansion.command_name, "/review");
        assert_eq!(expansion.command_source.as_deref(), Some("project"));

        let mut cwd_changed = base_fields();
        cwd_changed.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "CwdChanged",
                "old_cwd": "/workspace/old",
                "new_cwd": "/workspace/new"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::CwdChanged(cwd_changed) = parse_hook_input(cwd_changed).unwrap() else {
            panic!("expected CwdChanged hook input");
        };
        assert_eq!(cwd_changed.old_cwd, "/workspace/old");
        assert_eq!(cwd_changed.new_cwd, "/workspace/new");

        let mut file_changed = base_fields();
        file_changed.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "FileChanged",
                "file_path": "/workspace/src/lib.rs",
                "event": "change"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::FileChanged(file_changed) = parse_hook_input(file_changed).unwrap() else {
            panic!("expected FileChanged hook input");
        };
        assert_eq!(file_changed.file_path, "/workspace/src/lib.rs");
        assert_eq!(file_changed.event, "change");

        let mut instructions = base_fields();
        instructions.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "InstructionsLoaded",
                "file_path": "/workspace/CLAUDE.md",
                "memory_type": "Project",
                "load_reason": "session_start",
                "globs": ["**/*.md"],
                "trigger_file_path": "/workspace/src/lib.rs",
                "parent_file_path": "/workspace/README.md"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::InstructionsLoaded(instructions) = parse_hook_input(instructions).unwrap()
        else {
            panic!("expected InstructionsLoaded hook input");
        };
        assert_eq!(instructions.memory_type, "Project");
        assert_eq!(
            instructions.globs.as_deref(),
            Some(&["**/*.md".to_owned()][..])
        );

        let mut denied = base_fields();
        denied.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "PermissionDenied",
                "tool_name": "Bash",
                "tool_input": {"command": "rm -rf /tmp/nope"},
                "tool_use_id": "toolu_denied",
                "reason": "blocked by policy"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::PermissionDenied(denied) = parse_hook_input(denied).unwrap() else {
            panic!("expected PermissionDenied hook input");
        };
        assert_eq!(denied.reason, "blocked by policy");
        assert!(matches!(
            denied.builtin_input().unwrap(),
            Some(BuiltinToolInput::Bash(_))
        ));

        let mut post_compact = base_fields();
        post_compact.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "PostCompact",
                "trigger": "auto",
                "compact_summary": "summary"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::PostCompact(post_compact) = parse_hook_input(post_compact).unwrap() else {
            panic!("expected PostCompact hook input");
        };
        assert_eq!(post_compact.compact_summary, "summary");

        let mut stop_failure = base_fields();
        stop_failure.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "StopFailure",
                "error": "rate_limit",
                "error_details": "retry later",
                "last_assistant_message": "partial"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::StopFailure(stop_failure) = parse_hook_input(stop_failure).unwrap() else {
            panic!("expected StopFailure hook input");
        };
        assert_eq!(stop_failure.error, AssistantMessageError::RateLimit);
        assert_eq!(stop_failure.error_details.as_deref(), Some("retry later"));

        let mut task_created = base_fields();
        task_created.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "TaskCreated",
                "task_id": "task-1",
                "task_subject": "audit",
                "task_description": "check docs",
                "teammate_name": "Noether",
                "team_name": "review"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::TaskCreated(task_created) = parse_hook_input(task_created).unwrap() else {
            panic!("expected TaskCreated hook input");
        };
        assert_eq!(task_created.task_subject, "audit");
        assert_eq!(task_created.teammate_name.as_deref(), Some("Noether"));

        let mut elicitation = base_fields();
        elicitation.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "Elicitation",
                "mcp_server_name": "github",
                "message": "Authorize GitHub",
                "mode": "form",
                "url": "https://example.com/auth",
                "elicitation_id": "elicit-1",
                "requested_schema": {"type": "object"}
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::Elicitation(elicitation) = parse_hook_input(elicitation).unwrap() else {
            panic!("expected Elicitation hook input");
        };
        assert_eq!(elicitation.mcp_server_name, "github");
        assert_eq!(elicitation.mode, Some(ElicitationMode::Form));

        let mut elicitation_result = base_fields();
        elicitation_result.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "ElicitationResult",
                "mcp_server_name": "github",
                "elicitation_id": "elicit-1",
                "mode": "url",
                "action": "accept",
                "content": {"account": "octo"}
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::ElicitationResult(elicitation_result) =
            parse_hook_input(elicitation_result).unwrap()
        else {
            panic!("expected ElicitationResult hook input");
        };
        assert_eq!(elicitation_result.action, ElicitationHookAction::Accept);
        assert_eq!(elicitation_result.mode, Some(ElicitationMode::Url));
        assert_eq!(elicitation_result.content.unwrap()["account"], "octo");
    }

    #[test]
    fn hook_input_serializes_subagent_identity_once() {
        let mut input = base_fields();
        input.as_object_mut().unwrap().extend(
            json!({
                "hook_event_name": "SubagentStart",
                "agent_id": "agent-1",
                "agent_type": "code-reviewer"
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        let HookInput::SubagentStart(start) = parse_hook_input(input).unwrap() else {
            panic!("expected SubagentStart hook input");
        };
        assert_eq!(start.agent_id, "agent-1");
        assert_eq!(start.agent_type, "code-reviewer");

        let serialized = serde_json::to_string(&HookInput::SubagentStart(start)).unwrap();
        assert_eq!(serialized.matches("\"agent_id\"").count(), 1);
        assert_eq!(serialized.matches("\"agent_type\"").count(), 1);
    }

    #[test]
    fn hook_input_preserves_unknown_hook_events() {
        let input = json!({
            "hook_event_name": "FutureHook",
            "future": true
        });
        let HookInput::Unknown {
            hook_event_name,
            raw,
        } = parse_hook_input(input).unwrap()
        else {
            panic!("expected unknown hook input");
        };
        assert_eq!(hook_event_name.as_deref(), Some("FutureHook"));
        assert_eq!(raw["future"], true);
    }

    #[test]
    fn hook_json_output_serializes_documented_sync_and_async_shapes() {
        let updated_input = serde_json::from_value::<Map<String, Value>>(json!({
            "command": "echo safe"
        }))
        .unwrap();
        let sync = SyncHookJsonOutput::new()
            .continue_(false)
            .suppress_output(true)
            .stop_reason("blocked")
            .decision(HookDecision::Block)
            .system_message("Command blocked")
            .reason("Unsafe command")
            .terminal_sequence("\u{7}")
            .hook_specific_output(HookSpecificOutput::PreToolUse {
                permission_decision: Some(HookPermissionDecision::Deny),
                permission_decision_reason: Some("unsafe".into()),
                updated_input: Some(updated_input),
                additional_context: Some("Use a safer command".into()),
            });
        let value = serde_json::to_value(sync).unwrap();

        assert_eq!(value["continue"], false);
        assert_eq!(value["suppressOutput"], true);
        assert_eq!(value["decision"], "block");
        assert_eq!(value["terminalSequence"], "\u{7}");
        assert_eq!(value["hookSpecificOutput"]["hookEventName"], "PreToolUse");
        assert_eq!(value["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(
            value["hookSpecificOutput"]["updatedInput"]["command"],
            "echo safe"
        );
        assert_eq!(value.get("continue_"), None);

        let async_output = AsyncHookJsonOutput::new(Some(5000));
        let value = serde_json::to_value(HookJsonOutput::from(async_output)).unwrap();
        assert_eq!(value["async"], true);
        assert_eq!(value["asyncTimeout"], 5000);

        assert!(serde_json::from_value::<AsyncHookJsonOutput>(json!({"async": false})).is_err());
        assert!(serde_json::from_value::<HookJsonOutput>(json!({"async": false})).is_err());
    }

    #[test]
    fn hook_json_output_extra_cannot_override_reserved_fields() {
        let sync = SyncHookJsonOutput::new()
            .continue_(true)
            .extra("continue", json!(false))
            .extra("async", json!(false))
            .extra("customFutureField", json!("kept"));
        let value = serde_json::to_value(HookJsonOutput::from(sync)).unwrap();

        assert_eq!(value["continue"], true);
        assert_eq!(value.get("async"), None);
        assert_eq!(value["customFutureField"], "kept");
        assert!(serde_json::from_value::<SyncHookJsonOutput>(json!({"async": false})).is_err());
    }

    #[test]
    fn hook_json_output_serializes_permission_request_decision_shape() {
        let output =
            SyncHookJsonOutput::new().hook_specific_output(HookSpecificOutput::PermissionRequest {
                decision: PermissionResult::allow_with_updated_input(json!({
                    "file_path": "./safe.txt"
                })),
            });
        let value: Value = output.into();

        assert_eq!(
            value["hookSpecificOutput"]["hookEventName"],
            "PermissionRequest"
        );
        assert_eq!(value["hookSpecificOutput"]["decision"]["behavior"], "allow");
        assert_eq!(
            value["hookSpecificOutput"]["decision"]["updatedInput"]["file_path"],
            "./safe.txt"
        );
    }

    #[test]
    fn permission_result_serializes_current_metadata_fields() {
        let allow = serde_json::to_value(
            PermissionResult::allow()
                .with_tool_use_id("toolu_123")
                .with_decision_classification("user_permanent"),
        )
        .unwrap();
        assert_eq!(allow["behavior"], "allow");
        assert_eq!(allow["toolUseID"], "toolu_123");
        assert_eq!(allow["decisionClassification"], "user_permanent");
        assert!(allow.get("toolUseId").is_none());

        let deny = serde_json::to_value(
            PermissionResult::deny("blocked").with_decision_classification("user_reject"),
        )
        .unwrap();
        assert_eq!(deny["behavior"], "deny");
        assert_eq!(deny["decisionClassification"], "user_reject");
    }

    #[test]
    fn hook_json_output_serializes_current_reference_event_specific_shapes() {
        let content = serde_json::from_value::<Map<String, Value>>(json!({
            "username": "alice"
        }))
        .unwrap();

        let cases = [
            (
                SyncHookJsonOutput::new().hook_specific_output(
                    HookSpecificOutput::UserPromptSubmit {
                        additional_context: Some("deploy target: staging".into()),
                        session_title: Some("deploy check".into()),
                        suppress_original_prompt: Some(true),
                    },
                ),
                json!({
                    "hookEventName": "UserPromptSubmit",
                    "additionalContext": "deploy target: staging",
                    "sessionTitle": "deploy check",
                    "suppressOriginalPrompt": true
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(
                    HookSpecificOutput::MessageDisplay {
                        display_content: Some("redacted".into()),
                    },
                ),
                json!({
                    "hookEventName": "MessageDisplay",
                    "displayContent": "redacted"
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(HookSpecificOutput::SessionStart {
                    additional_context: Some("branch: main".into()),
                    initial_user_message: Some("hello".into()),
                    session_title: Some("main".into()),
                    watch_paths: Some(vec!["/workspace/Cargo.toml".into()]),
                    reload_skills: Some(true),
                }),
                json!({
                    "hookEventName": "SessionStart",
                    "additionalContext": "branch: main",
                    "initialUserMessage": "hello",
                    "sessionTitle": "main",
                    "watchPaths": ["/workspace/Cargo.toml"],
                    "reloadSkills": true
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(
                    HookSpecificOutput::WorktreeCreate {
                        worktree_path: Some("/workspace/.worktrees/feature".into()),
                    },
                ),
                json!({
                    "hookEventName": "WorktreeCreate",
                    "worktreePath": "/workspace/.worktrees/feature"
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(HookSpecificOutput::CwdChanged {
                    watch_paths: Some(vec!["/workspace/.env".into()]),
                }),
                json!({
                    "hookEventName": "CwdChanged",
                    "watchPaths": ["/workspace/.env"]
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(HookSpecificOutput::Elicitation {
                    action: ElicitationHookAction::Accept,
                    content: Some(content),
                }),
                json!({
                    "hookEventName": "Elicitation",
                    "action": "accept",
                    "content": {"username": "alice"}
                }),
            ),
            (
                SyncHookJsonOutput::new().hook_specific_output(
                    HookSpecificOutput::PermissionDenied { retry: Some(true) },
                ),
                json!({
                    "hookEventName": "PermissionDenied",
                    "retry": true
                }),
            ),
        ];

        for (output, expected) in cases {
            let value: Value = output.into();
            assert_eq!(value["hookSpecificOutput"], expected);
        }
    }
}
