use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::{Map, Value};

/// Wire-level control envelope discriminator used by SDK control frames.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SDKControlEnvelopeType {
    ControlRequest,
    ControlResponse,
    Other(String),
}

impl SDKControlEnvelopeType {
    pub const CONTROL_REQUEST: &'static str = "control_request";
    pub const CONTROL_RESPONSE: &'static str = "control_response";

    pub fn as_str(&self) -> &str {
        match self {
            Self::ControlRequest => Self::CONTROL_REQUEST,
            Self::ControlResponse => Self::CONTROL_RESPONSE,
            Self::Other(value) => value,
        }
    }
}

impl Serialize for SDKControlEnvelopeType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SDKControlEnvelopeType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            Self::CONTROL_REQUEST => Self::ControlRequest,
            Self::CONTROL_RESPONSE => Self::ControlResponse,
            _ => Self::Other(value),
        })
    }
}

/// Wire-level SDK control request envelope.
///
/// The inner `request` is intentionally raw JSON because upstream exposes a
/// large evolving union of request subtypes. Unknown outer fields are preserved
/// in [`Self::extra`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlRequest {
    #[serde(rename = "type")]
    pub envelope_type: SDKControlEnvelopeType,
    pub request_id: String,
    pub request: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl SDKControlRequest {
    pub fn new(request_id: impl Into<String>, request: Value) -> Self {
        Self {
            envelope_type: SDKControlEnvelopeType::ControlRequest,
            request_id: request_id.into(),
            request,
            extra: Map::new(),
        }
    }
}

/// Wire-level SDK control response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlResponse {
    #[serde(rename = "type")]
    pub envelope_type: SDKControlEnvelopeType,
    pub response: SDKControlResponsePayload,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl SDKControlResponse {
    pub fn success(request_id: impl Into<String>, response: Value) -> Self {
        Self {
            envelope_type: SDKControlEnvelopeType::ControlResponse,
            response: SDKControlResponsePayload::Success(SDKControlSuccessResponse {
                response_type: SDKControlSuccessResponseType::Success,
                request_id: request_id.into(),
                response,
                extra: Map::new(),
            }),
            extra: Map::new(),
        }
    }

    pub fn error(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            envelope_type: SDKControlEnvelopeType::ControlResponse,
            response: SDKControlResponsePayload::Error(SDKControlErrorResponse {
                response_type: SDKControlErrorResponseType::Error,
                request_id: request_id.into(),
                error: error.into(),
                extra: Map::new(),
            }),
            extra: Map::new(),
        }
    }
}

/// Success, error, or future SDK control response payload.
#[derive(Debug, Clone, PartialEq)]
pub enum SDKControlResponsePayload {
    Success(SDKControlSuccessResponse),
    Error(SDKControlErrorResponse),
    Other(Value),
}

impl Serialize for SDKControlResponsePayload {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Success(response) => response.serialize(serializer),
            Self::Error(response) => response.serialize(serializer),
            Self::Other(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SDKControlResponsePayload {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value.get("subtype").and_then(Value::as_str) {
            Some("success") => serde_json::from_value(value)
                .map(Self::Success)
                .map_err(de::Error::custom),
            Some("error") => serde_json::from_value(value)
                .map(Self::Error)
                .map_err(de::Error::custom),
            _ => Ok(Self::Other(value)),
        }
    }
}

/// Successful SDK control response payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlSuccessResponse {
    #[serde(rename = "subtype")]
    pub response_type: SDKControlSuccessResponseType,
    pub request_id: String,
    #[serde(default)]
    pub response: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl SDKControlSuccessResponse {
    pub fn new(request_id: impl Into<String>, response: Value) -> Self {
        Self {
            response_type: SDKControlSuccessResponseType::Success,
            request_id: request_id.into(),
            response,
            extra: Map::new(),
        }
    }
}

/// SDK control success literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SDKControlSuccessResponseType {
    #[serde(rename = "success")]
    Success,
}

/// Failed SDK control response payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlErrorResponse {
    #[serde(rename = "subtype")]
    pub response_type: SDKControlErrorResponseType,
    pub request_id: String,
    pub error: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl SDKControlErrorResponse {
    pub fn new(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            response_type: SDKControlErrorResponseType::Error,
            request_id: request_id.into(),
            error: error.into(),
            extra: Map::new(),
        }
    }
}

/// SDK control error literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SDKControlErrorResponseType {
    #[serde(rename = "error")]
    Error,
}

/// Typed protocol-envelope response returned by the SDK `initialize` control request.
///
/// Required fields follow the upstream TypeScript SDK shape; unknown fields are
/// preserved in [`Self::extra`] so callers can inspect newer Claude Code fields
/// before this crate grows first-class Rust wrappers for them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlInitializeResponse {
    pub commands: Vec<SlashCommand>,
    pub agents: Vec<AgentInfo>,
    pub output_style: String,
    pub available_output_styles: Vec<String>,
    pub models: Vec<ModelInfo>,
    pub account: AccountInfo,
    #[serde(default)]
    pub fast_mode_state: Option<FastModeState>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Fast-mode state literal defined by the upstream SDK protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FastModeState {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "cooldown")]
    Cooldown,
    #[serde(rename = "on")]
    On,
}

impl FastModeState {
    pub const OFF: &'static str = "off";
    pub const COOLDOWN: &'static str = "cooldown";
    pub const ON: &'static str = "on";

    pub fn as_str(&self) -> &str {
        match self {
            Self::Off => Self::OFF,
            Self::Cooldown => Self::COOLDOWN,
            Self::On => Self::ON,
        }
    }
}

/// Slash command metadata advertised by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    #[serde(rename = "argumentHint")]
    pub argument_hint: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Model metadata advertised by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub value: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    #[serde(default, rename = "supportsEffort")]
    pub supports_effort: Option<bool>,
    #[serde(default, rename = "supportedEffortLevels")]
    pub supported_effort_levels: Option<Vec<ModelEffortLevel>>,
    #[serde(default, rename = "supportsAdaptiveThinking")]
    pub supports_adaptive_thinking: Option<bool>,
    #[serde(default, rename = "supportsFastMode")]
    pub supports_fast_mode: Option<bool>,
    #[serde(default, rename = "supportsAutoMode")]
    pub supports_auto_mode: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Model effort literal advertised by Claude Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelEffortLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "xhigh")]
    XHigh,
    #[serde(rename = "max")]
    Max,
}

impl ModelEffortLevel {
    pub const LOW: &'static str = "low";
    pub const MEDIUM: &'static str = "medium";
    pub const HIGH: &'static str = "high";
    pub const XHIGH: &'static str = "xhigh";
    pub const MAX: &'static str = "max";

    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => Self::LOW,
            Self::Medium => Self::MEDIUM,
            Self::High => Self::HIGH,
            Self::XHigh => Self::XHIGH,
            Self::Max => Self::MAX,
        }
    }
}

/// Agent metadata advertised by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Source of the API key/auth credential reported by Claude Code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeySource {
    User,
    Project,
    Org,
    Temporary,
    Oauth,
    #[serde(untagged)]
    Other(String),
}

/// Auth/account metadata advertised by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default, rename = "subscriptionType")]
    pub subscription_type: Option<String>,
    #[serde(default, rename = "tokenSource")]
    pub token_source: Option<String>,
    #[serde(default, rename = "apiKeySource")]
    pub api_key_source: Option<ApiKeySource>,
    #[serde(default, rename = "apiProvider")]
    pub api_provider: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Typed protocol-envelope response returned by the SDK `mcp_status` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpStatusResponse {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Vec<McpServerStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Result of an SDK `mcp_set_servers` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpSetServersResult {
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(default)]
    pub errors: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Result of an SDK `rewind_files` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindFilesResult {
    #[serde(rename = "canRewind")]
    pub can_rewind: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default, rename = "filesChanged")]
    pub files_changed: Option<Vec<String>>,
    #[serde(default)]
    pub insertions: Option<u64>,
    #[serde(default)]
    pub deletions: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Response returned by the SDK `side_question` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SideQuestionResponse {
    pub response: String,
    #[serde(default)]
    pub synthetic: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// MCP server connection status and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerStatus {
    pub name: String,
    pub status: McpServerConnectionStatus,
    #[serde(default, rename = "serverInfo")]
    pub server_info: Option<McpServerInfo>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub config: Option<McpServerStatusConfig>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<McpToolInfo>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// MCP server status literal defined by the upstream SDK protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpServerConnectionStatus {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "needs-auth")]
    NeedsAuth,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "disabled")]
    Disabled,
}

impl McpServerConnectionStatus {
    pub const CONNECTED: &'static str = "connected";
    pub const FAILED: &'static str = "failed";
    pub const NEEDS_AUTH: &'static str = "needs-auth";
    pub const PENDING: &'static str = "pending";
    pub const DISABLED: &'static str = "disabled";

    pub fn as_str(&self) -> &str {
        match self {
            Self::Connected => Self::CONNECTED,
            Self::Failed => Self::FAILED,
            Self::NeedsAuth => Self::NEEDS_AUTH,
            Self::Pending => Self::PENDING,
            Self::Disabled => Self::DISABLED,
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

/// MCP server configuration returned in status responses.
#[derive(Debug, Clone, PartialEq)]
pub enum McpServerStatusConfig {
    Stdio(McpStdioServerStatusConfig),
    Sse(McpSseServerStatusConfig),
    Http(McpHttpServerStatusConfig),
    Sdk(McpSdkServerStatusConfig),
    ClaudeAiProxy(McpClaudeAiProxyServerConfig),
    Raw(Value),
}

impl Serialize for McpServerStatusConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Stdio(config) => config.serialize(serializer),
            Self::Sse(config) => config.serialize(serializer),
            Self::Http(config) => config.serialize(serializer),
            Self::Sdk(config) => config.serialize(serializer),
            Self::ClaudeAiProxy(config) => config.serialize(serializer),
            Self::Raw(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for McpServerStatusConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| de::Error::custom("MCP status config must be an object"))?;
        match object.get("type").and_then(Value::as_str) {
            Some("stdio") => serde_json::from_value(value)
                .map(Self::Stdio)
                .map_err(de::Error::custom),
            None if object.contains_key("command") => serde_json::from_value(value)
                .map(Self::Stdio)
                .map_err(de::Error::custom),
            Some("sse") => serde_json::from_value(value)
                .map(Self::Sse)
                .map_err(de::Error::custom),
            Some("http") => serde_json::from_value(value)
                .map(Self::Http)
                .map_err(de::Error::custom),
            Some("sdk") => serde_json::from_value(value)
                .map(Self::Sdk)
                .map_err(de::Error::custom),
            Some("claudeai-proxy") => serde_json::from_value(value)
                .map(Self::ClaudeAiProxy)
                .map_err(de::Error::custom),
            Some(_) => Ok(Self::Raw(value)),
            None => Err(de::Error::custom(
                "MCP status config without type must include command for stdio",
            )),
        }
    }
}

/// Stdio MCP server config returned in status responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpStdioServerStatusConfig {
    #[serde(default, rename = "type")]
    pub transport_type: Option<McpStdioServerConfigType>,
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub tools: Option<Vec<McpServerConfigTool>>,
    #[serde(default, rename = "alwaysLoad")]
    pub always_load: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Optional stdio type literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpStdioServerConfigType {
    #[serde(rename = "stdio")]
    Stdio,
}

/// SSE MCP server config returned in status responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpSseServerStatusConfig {
    #[serde(rename = "type")]
    pub transport_type: McpSseServerConfigType,
    pub url: String,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub tools: Option<Vec<McpServerConfigTool>>,
    #[serde(default, rename = "alwaysLoad")]
    pub always_load: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// SSE type literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpSseServerConfigType {
    #[serde(rename = "sse")]
    Sse,
}

/// HTTP MCP server config returned in status responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpHttpServerStatusConfig {
    #[serde(rename = "type")]
    pub transport_type: McpHttpServerConfigType,
    pub url: String,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub tools: Option<Vec<McpServerConfigTool>>,
    #[serde(default, rename = "alwaysLoad")]
    pub always_load: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// HTTP type literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpHttpServerConfigType {
    #[serde(rename = "http")]
    Http,
}

/// Tool policy entry accepted by remote MCP server configs.
///
/// The current TypeScript package reports HTTP/SSE remote configs with
/// per-tool permission policy objects. The string variant preserves older or
/// CLI-specific status payloads that report a bare list of tool names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerConfigTool {
    Name(String),
    Policy(McpServerToolPolicy),
}

/// Per-tool permission policy carried by remote MCP server configs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct McpServerToolPolicy {
    pub name: String,
    pub permission_policy: McpServerPermissionPolicy,
}

/// Remote MCP server tool permission policy literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpServerPermissionPolicy {
    #[serde(rename = "always_allow")]
    AlwaysAllow,
    #[serde(rename = "always_ask")]
    AlwaysAsk,
    #[serde(rename = "always_deny")]
    AlwaysDeny,
}

impl McpServerPermissionPolicy {
    pub const ALWAYS_ALLOW: &'static str = "always_allow";
    pub const ALWAYS_ASK: &'static str = "always_ask";
    pub const ALWAYS_DENY: &'static str = "always_deny";

    pub fn as_str(&self) -> &str {
        match self {
            Self::AlwaysAllow => Self::ALWAYS_ALLOW,
            Self::AlwaysAsk => Self::ALWAYS_ASK,
            Self::AlwaysDeny => Self::ALWAYS_DENY,
        }
    }
}

/// SDK MCP server config returned in status responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpSdkServerStatusConfig {
    #[serde(rename = "type")]
    pub transport_type: McpSdkServerConfigType,
    pub name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// SDK type literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpSdkServerConfigType {
    #[serde(rename = "sdk")]
    Sdk,
}

/// Claude.ai proxy MCP server config returned in status responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpClaudeAiProxyServerConfig {
    #[serde(rename = "type")]
    pub transport_type: McpClaudeAiProxyServerConfigType,
    pub url: String,
    pub id: String,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Claude.ai proxy type literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpClaudeAiProxyServerConfigType {
    #[serde(rename = "claudeai-proxy")]
    ClaudeAiProxy,
}

/// MCP server implementation metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// MCP tool metadata advertised by an MCP server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub annotations: Option<McpToolAnnotations>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// MCP tool safety annotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolAnnotations {
    #[serde(default, rename = "readOnly")]
    pub read_only: Option<bool>,
    #[serde(default)]
    pub destructive: Option<bool>,
    #[serde(default, rename = "openWorld")]
    pub open_world: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Typed protocol-envelope response returned by the SDK `read_file` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SDKControlReadFileResponse {
    pub contents: String,
    pub abs_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<ReadFileEncoding>,
}

/// File encoding selector for the SDK `read_file` control request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReadFileEncoding {
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "base64")]
    Base64,
}

impl ReadFileEncoding {
    pub fn as_protocol_value(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Base64 => "base64",
        }
    }
}

/// Typed protocol-envelope response returned by the SDK `reload_plugins` control request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKControlReloadPluginsResponse {
    pub commands: Vec<SlashCommand>,
    pub agents: Vec<AgentInfo>,
    pub plugins: Vec<PluginInfo>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Vec<McpServerStatus>,
    pub error_count: u64,
}

/// Plugin metadata returned after a plugin reload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Typed protocol-envelope response returned by the SDK `get_context_usage` control request.
///
/// Required fields follow the upstream Python SDK shape. `categories` is typed
/// because upstream exposes a dedicated category row; the other nested context
/// collections intentionally remain JSON object maps to preserve the
/// `dict[str, Any]` rows accepted by the official SDK. Unknown top-level fields
/// are preserved in [`Self::extra`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextUsageResponse {
    pub categories: Vec<ContextUsageCategory>,
    #[serde(rename = "totalTokens")]
    pub total_tokens: u64,
    #[serde(rename = "maxTokens")]
    pub max_tokens: u64,
    #[serde(rename = "rawMaxTokens")]
    pub raw_max_tokens: u64,
    pub percentage: f64,
    pub model: String,
    #[serde(rename = "isAutoCompactEnabled")]
    pub is_auto_compact_enabled: bool,
    #[serde(rename = "memoryFiles")]
    pub memory_files: Vec<Map<String, Value>>,
    #[serde(rename = "mcpTools")]
    pub mcp_tools: Vec<Map<String, Value>>,
    pub agents: Vec<Map<String, Value>>,
    #[serde(rename = "gridRows")]
    pub grid_rows: Vec<Vec<Map<String, Value>>>,
    #[serde(default, rename = "autoCompactThreshold")]
    pub auto_compact_threshold: Option<u64>,
    #[serde(default, rename = "deferredBuiltinTools")]
    pub deferred_builtin_tools: Option<Vec<Map<String, Value>>>,
    #[serde(default, rename = "systemTools")]
    pub system_tools: Option<Vec<Map<String, Value>>>,
    #[serde(default, rename = "systemPromptSections")]
    pub system_prompt_sections: Option<Vec<Map<String, Value>>>,
    #[serde(default, rename = "slashCommands")]
    pub slash_commands: Option<Map<String, Value>>,
    #[serde(default)]
    pub skills: Option<Map<String, Value>>,
    #[serde(default, rename = "messageBreakdown")]
    pub message_breakdown: Option<Map<String, Value>>,
    #[serde(default, rename = "apiUsage")]
    pub api_usage: Option<Map<String, Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Context usage category row returned by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextUsageCategory {
    pub name: String,
    pub tokens: u64,
    pub color: String,
    #[serde(default, rename = "isDeferred")]
    pub is_deferred: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn initialization_requires_upstream_required_fields() {
        let mut initialization = json!({
            "commands": [],
            "agents": [],
            "output_style": "default",
            "available_output_styles": ["default"],
            "models": [],
            "account": {}
        });
        assert!(
            serde_json::from_value::<SDKControlInitializeResponse>(initialization.clone()).is_ok()
        );

        initialization.as_object_mut().unwrap().remove("account");
        assert!(serde_json::from_value::<SDKControlInitializeResponse>(initialization).is_err());
    }

    #[test]
    fn initialization_rejects_unknown_fast_mode_state() {
        let initialization = json!({
            "commands": [],
            "agents": [],
            "output_style": "default",
            "available_output_styles": ["default"],
            "models": [],
            "account": {},
            "fast_mode_state": "cooldown"
        });
        let parsed = serde_json::from_value::<SDKControlInitializeResponse>(initialization)
            .expect("documented fast mode state should parse");
        assert_eq!(
            parsed.fast_mode_state.as_ref().map(FastModeState::as_str),
            Some("cooldown")
        );

        assert!(
            serde_json::from_value::<SDKControlInitializeResponse>(json!({
                "commands": [],
                "agents": [],
                "output_style": "default",
                "available_output_styles": ["default"],
                "models": [],
                "account": {},
                "fast_mode_state": "paused"
            }))
            .is_err()
        );
    }

    #[test]
    fn nested_initialization_types_require_upstream_required_fields() {
        assert!(
            serde_json::from_value::<SlashCommand>(json!({
                "name": "help",
                "description": "Show help",
                "argumentHint": "[topic]"
            }))
            .is_ok()
        );
        assert!(
            serde_json::from_value::<SlashCommand>(json!({
                "name": "help",
                "description": "Show help"
            }))
            .is_err()
        );

        assert!(
            serde_json::from_value::<ModelInfo>(json!({
                "value": "claude-test",
                "displayName": "Claude Test",
                "description": "Test model",
                "supportedEffortLevels": ["low", "medium", "high", "xhigh", "max"],
                "supportsAutoMode": true
            }))
            .is_ok()
        );
        let model_without_effort: ModelInfo = serde_json::from_value(json!({
            "value": "claude-test",
            "displayName": "Claude Test",
            "description": "Test model"
        }))
        .unwrap();
        assert!(model_without_effort.supported_effort_levels.is_none());
        assert!(
            serde_json::from_value::<ModelInfo>(json!({
                "value": "claude-test",
                "description": "Test model"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<ModelInfo>(json!({
                "value": "claude-test",
                "displayName": "Claude Test",
                "description": "Test model",
                "supportedEffortLevels": ["low", "extreme"]
            }))
            .is_err()
        );
    }

    #[test]
    fn mcp_status_requires_server_list_and_status() {
        assert!(
            serde_json::from_value::<McpStatusResponse>(json!({
                "mcpServers": [{"name": "filesystem", "status": "connected"}]
            }))
            .is_ok()
        );
        assert!(serde_json::from_value::<McpStatusResponse>(json!({})).is_err());
        assert!(serde_json::from_value::<McpServerStatus>(json!({"name": "filesystem"})).is_err());
        assert!(
            serde_json::from_value::<McpServerStatus>(
                json!({"name": "filesystem", "status": "bogus"})
            )
            .is_err()
        );

        let status: McpServerStatus =
            serde_json::from_value(json!({"name": "oauth", "status": "needs-auth"})).unwrap();
        assert_eq!(status.status.as_str(), "needs-auth");
        assert!(status.tools.is_none());
    }

    #[test]
    fn mcp_status_parses_documented_config_variants() {
        let stdio: McpServerStatus = serde_json::from_value(json!({
            "name": "filesystem",
            "status": "connected",
            "config": {
                "type": "stdio",
                "command": "fs-mcp",
                "args": ["--root", "."],
                "env": {"RUST_LOG": "debug"},
                "timeout": 120,
                "tools": ["read_file"],
                "alwaysLoad": true
            }
        }))
        .unwrap();
        match stdio.config.unwrap() {
            McpServerStatusConfig::Stdio(config) => {
                assert_eq!(config.command, "fs-mcp");
                assert_eq!(config.env.unwrap()["RUST_LOG"], "debug");
                assert_eq!(config.timeout, Some(120));
                assert!(matches!(
                    &config.tools.unwrap()[0],
                    McpServerConfigTool::Name(name) if name == "read_file"
                ));
                assert_eq!(config.always_load, Some(true));
            }
            other => panic!("expected stdio config, got {other:?}"),
        }

        let http: McpServerStatus = serde_json::from_value(json!({
            "name": "remote",
            "status": "connected",
            "config": {
                "type": "http",
                "url": "https://example.com/mcp",
                "headers": {"Authorization": "Bearer token"},
                "tools": [{"name": "search", "permission_policy": "always_allow"}],
                "timeout": 90
            }
        }))
        .unwrap();
        match http.config.unwrap() {
            McpServerStatusConfig::Http(config) => {
                assert_eq!(config.timeout, Some(90));
                let tools = config.tools.unwrap();
                match &tools[0] {
                    McpServerConfigTool::Policy(policy) => {
                        assert_eq!(policy.name, "search");
                        assert_eq!(
                            policy.permission_policy.as_str(),
                            McpServerPermissionPolicy::ALWAYS_ALLOW
                        );
                    }
                    other => panic!("expected tool policy, got {other:?}"),
                }
            }
            other => panic!("expected http config, got {other:?}"),
        }

        let sse: McpServerStatus = serde_json::from_value(json!({
            "name": "events",
            "status": "connected",
            "config": {
                "type": "sse",
                "url": "https://example.com/events",
                "tools": [{"name": "read", "permission_policy": "always_ask"}],
                "alwaysLoad": false
            }
        }))
        .unwrap();
        assert!(matches!(
            &sse.config.unwrap(),
            McpServerStatusConfig::Sse(config)
                if config.tools.as_ref().unwrap()[0]
                    == McpServerConfigTool::Policy(McpServerToolPolicy {
                        name: "read".to_string(),
                        permission_policy: McpServerPermissionPolicy::AlwaysAsk,
                    })
        ));

        let proxy: McpServerStatus = serde_json::from_value(json!({
            "name": "connector",
            "status": "connected",
            "config": {
                "type": "claudeai-proxy",
                "url": "https://claude.ai/api/mcp",
                "id": "connector-id"
            }
        }))
        .unwrap();
        assert!(matches!(
            proxy.config.unwrap(),
            McpServerStatusConfig::ClaudeAiProxy(_)
        ));

        assert!(
            serde_json::from_value::<McpServerStatus>(json!({
                "name": "remote",
                "status": "connected",
                "config": {"type": "http", "headers": {"Authorization": "Bearer token"}}
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<McpServerStatus>(json!({
                "name": "remote",
                "status": "connected",
                "config": {
                    "type": "http",
                    "url": "https://example.com/mcp",
                    "headers": {"Retry": 3}
                }
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<McpServerStatus>(json!({
                "name": "remote",
                "status": "connected",
                "config": {
                    "type": "http",
                    "url": "https://example.com/mcp",
                    "tools": [{"name": "search", "permission_policy": "sometimes_allow"}]
                }
            }))
            .is_err()
        );
    }

    #[test]
    fn mcp_status_preserves_unknown_config_variants_for_protocol_drift() {
        let status: McpServerStatus = serde_json::from_value(json!({
            "name": "workspace",
            "status": "connected",
            "config": {
                "type": "ws",
                "url": "wss://mcp.example/ws",
                "headersHelper": "mcp-headers",
                "role": "comms",
                "timeout": 2500,
                "alwaysLoad": true
            }
        }))
        .unwrap();

        match status.config.unwrap() {
            McpServerStatusConfig::Raw(config) => {
                assert_eq!(config["type"], "ws");
                assert_eq!(config["headersHelper"], "mcp-headers");
                assert_eq!(config["role"], "comms");
                assert_eq!(config["timeout"], 2500);
                assert_eq!(config["alwaysLoad"], true);
            }
            other => panic!("expected raw config, got {other:?}"),
        }
    }

    #[test]
    fn context_usage_requires_upstream_required_fields() {
        let mut context = json!({
            "categories": [{"name": "Messages", "tokens": 42, "color": "blue"}],
            "totalTokens": 42,
            "maxTokens": 200000,
            "rawMaxTokens": 200000,
            "percentage": 0.021,
            "model": "claude-test",
            "isAutoCompactEnabled": true,
            "memoryFiles": [{"path": "CLAUDE.md", "type": "project", "tokens": 5}],
            "mcpTools": [{"name": "search", "serverName": "ref", "tokens": 7, "isLoaded": true}],
            "agents": [{"agentType": "coder", "source": "sdk", "tokens": 9}],
            "gridRows": [[{"label": "Messages", "tokens": 42}]],
            "deferredBuiltinTools": [{"name": "Read", "tokens": 3}],
            "systemTools": [{"name": "Bash", "tokens": 4}],
            "systemPromptSections": [{"name": "Core", "tokens": 6}],
            "messageBreakdown": {"toolCalls": 3, "provider": {"cached": true}},
            "apiUsage": {"input_tokens": 10, "provider": {"cached": true}}
        });
        let parsed = serde_json::from_value::<ContextUsageResponse>(context.clone()).unwrap();
        assert_eq!(parsed.memory_files[0]["path"], "CLAUDE.md");
        assert_eq!(parsed.mcp_tools[0]["serverName"], "ref");
        assert_eq!(parsed.agents[0]["agentType"], "coder");
        assert_eq!(parsed.grid_rows[0][0]["tokens"], 42);
        assert_eq!(
            parsed.message_breakdown.unwrap()["provider"]["cached"],
            true
        );
        assert_eq!(parsed.api_usage.unwrap()["input_tokens"], 10);

        context.as_object_mut().unwrap().remove("totalTokens");
        assert!(serde_json::from_value::<ContextUsageResponse>(context).is_err());
    }

    #[test]
    fn context_usage_preserves_upstream_dict_rows() {
        let valid_context = json!({
            "categories": [{"name": "Messages", "tokens": 42, "color": "blue"}],
            "totalTokens": 42,
            "maxTokens": 200000,
            "rawMaxTokens": 200000,
            "percentage": 0.021,
            "model": "claude-test",
            "isAutoCompactEnabled": true,
            "memoryFiles": [{"custom": {"path": "CLAUDE.md"}, "tokens": "provider-specific"}],
            "mcpTools": [{"serverName": "ref"}],
            "agents": [{"source": "sdk"}],
            "gridRows": [[{"label": "Messages", "tokens": "forty-two"}]],
            "deferredBuiltinTools": [{"kind": "read", "tokens": "deferred"}],
            "systemTools": [{"tool": "Bash"}],
            "systemPromptSections": [{"section": "Core"}],
            "messageBreakdown": {"providerSpecific": {"cached": true}, "label": "usage"},
            "apiUsage": null
        });
        let parsed = serde_json::from_value::<ContextUsageResponse>(valid_context.clone())
            .expect("provider-specific context dict rows and null apiUsage should parse");
        assert_eq!(parsed.memory_files[0]["custom"]["path"], "CLAUDE.md");
        assert_eq!(parsed.mcp_tools[0]["serverName"], "ref");
        assert_eq!(parsed.agents[0]["source"], "sdk");
        assert_eq!(parsed.grid_rows[0][0]["tokens"], "forty-two");
        assert_eq!(
            parsed.deferred_builtin_tools.unwrap()[0]["tokens"],
            "deferred"
        );
        assert_eq!(parsed.system_tools.unwrap()[0]["tool"], "Bash");
        assert_eq!(parsed.system_prompt_sections.unwrap()[0]["section"], "Core");
        assert_eq!(
            parsed.message_breakdown.unwrap()["providerSpecific"]["cached"],
            true
        );
        assert!(parsed.api_usage.is_none());
    }
}
