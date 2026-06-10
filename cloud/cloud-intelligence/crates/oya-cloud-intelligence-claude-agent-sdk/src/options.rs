use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer},
    ser::SerializeMap,
};
use serde_json::{Map, Value};

use crate::{
    callbacks::{
        CallbackRegistry, ElicitationCallback, ElicitationCallbackOptions, ElicitationRequest,
        ElicitationResult, HookCallback, HookMatcher, PermissionCallback, StderrCallback,
        TokenRefreshCallback, TokenRefreshCallbackOptions, ToolPermissionRequest,
        UserDialogCallback, UserDialogCallbackOptions, UserDialogRequest,
    },
    error::{ClaudeAgentError, Result},
    session_store::{SessionStore, SessionStoreFlushMode, SharedSessionStore},
    status::{McpServerPermissionPolicy, McpServerToolPolicy},
    tools::SdkMcpServer,
    transport::{ClaudeProcessSpawner, SharedClaudeProcessSpawner},
};

/// Permission modes supported by Claude Code / Agent SDK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    AcceptEdits,
    Plan,
    BypassPermissions,
    DontAsk,
    Auto,
}

impl PermissionMode {
    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AcceptEdits => "acceptEdits",
            Self::Plan => "plan",
            Self::BypassPermissions => "bypassPermissions",
            Self::DontAsk => "dontAsk",
            Self::Auto => "auto",
        }
    }
}

/// Filesystem settings sources to load for SDK sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    User,
    Project,
    Local,
}

impl SettingSource {
    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
            Self::Local => "local",
        }
    }
}

/// Model effort / adaptive thinking guidance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
    #[serde(untagged)]
    Other(String),
}

impl EffortLevel {
    pub fn as_cli_value(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
            Self::Other(value) => value,
        }
    }
}

/// System prompt configuration.
#[derive(Debug, Clone, PartialEq)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<String>),
    Preset {
        preset: String,
        append: Option<String>,
        exclude_dynamic_sections: Option<bool>,
    },
    File {
        path: PathBuf,
    },
}

impl Serialize for SystemPrompt {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Text(text) => serializer.serialize_str(text),
            Self::Blocks(blocks) => blocks.serialize(serializer),
            Self::Preset {
                preset,
                append,
                exclude_dynamic_sections,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "preset")?;
                map.serialize_entry("preset", preset)?;
                if let Some(append) = append {
                    map.serialize_entry("append", append)?;
                }
                if let Some(exclude_dynamic_sections) = exclude_dynamic_sections {
                    map.serialize_entry("excludeDynamicSections", exclude_dynamic_sections)?;
                }
                map.end()
            }
            Self::File { path } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "file")?;
                map.serialize_entry("path", path)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for SystemPrompt {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(text) => Ok(Self::Text(text)),
            Value::Array(values) => values
                .into_iter()
                .map(|value| match value {
                    Value::String(block) => Ok(block),
                    other => Err(de::Error::custom(format!(
                        "system prompt blocks must be strings, got {other}"
                    ))),
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map(Self::Blocks),
            Value::Object(mut object) => {
                let prompt_type = object
                    .remove("type")
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .ok_or_else(|| de::Error::custom("system prompt object requires type"))?;
                match prompt_type.as_str() {
                    "preset" => {
                        let preset = object
                            .remove("preset")
                            .and_then(|value| value.as_str().map(str::to_owned))
                            .ok_or_else(|| {
                                de::Error::custom("preset system prompt requires preset")
                            })?;
                        let append = object
                            .remove("append")
                            .and_then(|value| value.as_str().map(str::to_owned));
                        let exclude_dynamic_sections = object
                            .remove("excludeDynamicSections")
                            .or_else(|| object.remove("exclude_dynamic_sections"))
                            .and_then(|value| value.as_bool());
                        Ok(Self::Preset {
                            preset,
                            append,
                            exclude_dynamic_sections,
                        })
                    }
                    "file" => {
                        let path = object
                            .remove("path")
                            .and_then(|value| value.as_str().map(PathBuf::from))
                            .ok_or_else(|| de::Error::custom("file system prompt requires path"))?;
                        Ok(Self::File { path })
                    }
                    other => Err(de::Error::custom(format!(
                        "unsupported system prompt type: {other}"
                    ))),
                }
            }
            other => Err(de::Error::custom(format!(
                "system prompt must be a string, string array, or object, got {other}"
            ))),
        }
    }
}

impl From<String> for SystemPrompt {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for SystemPrompt {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<Vec<String>> for SystemPrompt {
    fn from(value: Vec<String>) -> Self {
        Self::Blocks(value)
    }
}

impl From<Vec<&str>> for SystemPrompt {
    fn from(value: Vec<&str>) -> Self {
        Self::Blocks(value.into_iter().map(str::to_owned).collect())
    }
}

/// Base tool availability selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tools {
    List(Vec<String>),
    Preset { r#type: String, preset: String },
}

impl Tools {
    pub fn claude_code_preset() -> Self {
        Self::Preset {
            r#type: "preset".to_owned(),
            preset: "claude_code".to_owned(),
        }
    }
}

/// SDK beta feature names. Kept open so the Rust SDK does not lag upstream beta headers.
pub type SdkBeta = String;

/// API-side task budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBudget {
    pub total: u64,
}

/// Subagent definition sent in the SDK initialize control request.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefinition {
    pub description: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disallowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "criticalSystemReminder_EXPERIMENTAL")]
    pub critical_system_reminder_experimental: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
}

/// Permission rule value used by permission update suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRuleValue {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_content: Option<String>,
}

/// Permission behavior literals used by permission update suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehavior {
    Allow,
    Deny,
    Ask,
    #[serde(untagged)]
    Other(String),
}

/// Settings destination literals used by permission update suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionUpdateDestination {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    Session,
    CliArg,
    #[serde(untagged)]
    Other(String),
}

/// Permission update protocol shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionUpdate {
    #[serde(rename = "type")]
    pub update_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<PermissionRuleValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<PermissionBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<PermissionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// MCP stdio/SSE/HTTP/SDK server config. Kept structurally flexible for parity.
#[derive(Debug, Clone, PartialEq)]
pub enum McpServerConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
        timeout: Option<u64>,
        always_load: Option<bool>,
    },
    Sse {
        url: String,
        headers: BTreeMap<String, String>,
        tools: Vec<McpServerToolPolicy>,
        timeout: Option<u64>,
        always_load: Option<bool>,
    },
    Http {
        url: String,
        headers: BTreeMap<String, String>,
        tools: Vec<McpServerToolPolicy>,
        timeout: Option<u64>,
        always_load: Option<bool>,
    },
    /// Serializable SDK server descriptor. The in-process instance is owned by `tools`.
    Sdk { name: String },
    /// Forward-compatible raw config for current/future upstream MCP transports.
    Raw(Value),
    /// Legacy placeholder for intentionally opaque configs constructed by callers.
    Unknown,
}

impl Serialize for McpServerConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Stdio {
                command,
                args,
                env,
                timeout,
                always_load,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "stdio")?;
                map.serialize_entry("command", command)?;
                if !args.is_empty() {
                    map.serialize_entry("args", args)?;
                }
                if !env.is_empty() {
                    map.serialize_entry("env", env)?;
                }
                if let Some(timeout) = effective_mcp_timeout(timeout) {
                    map.serialize_entry("timeout", &timeout)?;
                }
                if let Some(always_load) = always_load {
                    map.serialize_entry("alwaysLoad", always_load)?;
                }
                map.end()
            }
            Self::Sse {
                url,
                headers,
                tools,
                timeout,
                always_load,
            } => serialize_remote_mcp_server(
                serializer,
                "sse",
                url,
                headers,
                tools,
                timeout,
                always_load,
            ),
            Self::Http {
                url,
                headers,
                tools,
                timeout,
                always_load,
            } => serialize_remote_mcp_server(
                serializer,
                "http",
                url,
                headers,
                tools,
                timeout,
                always_load,
            ),
            Self::Sdk { name } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "sdk")?;
                map.serialize_entry("name", name)?;
                map.end()
            }
            Self::Raw(value) => value.serialize(serializer),
            Self::Unknown => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "unknown")?;
                map.end()
            }
        }
    }
}

fn serialize_remote_mcp_server<S>(
    serializer: S,
    transport_type: &str,
    url: &str,
    headers: &BTreeMap<String, String>,
    tools: &[McpServerToolPolicy],
    timeout: &Option<u64>,
    always_load: &Option<bool>,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut map = serializer.serialize_map(None)?;
    map.serialize_entry("type", transport_type)?;
    map.serialize_entry("url", url)?;
    if !headers.is_empty() {
        map.serialize_entry("headers", headers)?;
    }
    if !tools.is_empty() {
        map.serialize_entry("tools", tools)?;
    }
    if let Some(timeout) = effective_mcp_timeout(timeout) {
        map.serialize_entry("timeout", &timeout)?;
    }
    if let Some(always_load) = always_load {
        map.serialize_entry("alwaysLoad", always_load)?;
    }
    map.end()
}

fn effective_mcp_timeout(timeout: &Option<u64>) -> Option<u64> {
    timeout.filter(|timeout| *timeout >= 1000)
}

impl<'de> Deserialize<'de> for McpServerConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct StdioWire {
            command: String,
            #[serde(default)]
            args: Vec<String>,
            #[serde(default)]
            env: BTreeMap<String, String>,
            timeout: Option<u64>,
            #[serde(alias = "always_load")]
            always_load: Option<bool>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RemoteWire {
            url: String,
            #[serde(default)]
            headers: BTreeMap<String, String>,
            #[serde(default)]
            tools: Vec<McpServerToolPolicy>,
            timeout: Option<u64>,
            #[serde(alias = "always_load")]
            always_load: Option<bool>,
        }

        #[derive(Deserialize)]
        struct SdkWire {
            name: String,
        }

        let value = Value::deserialize(deserializer)?;
        let object = value.as_object().ok_or_else(|| {
            de::Error::custom("MCP server config must be an object with type or command")
        })?;
        let transport_type = object.get("type").and_then(Value::as_str);
        match transport_type {
            Some("stdio") => {
                let wire: StdioWire = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Self::Stdio {
                    command: wire.command,
                    args: wire.args,
                    env: wire.env,
                    timeout: wire.timeout,
                    always_load: wire.always_load,
                })
            }
            None if object.contains_key("command") => {
                let wire: StdioWire = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Self::Stdio {
                    command: wire.command,
                    args: wire.args,
                    env: wire.env,
                    timeout: wire.timeout,
                    always_load: wire.always_load,
                })
            }
            Some("sse") => {
                let wire: RemoteWire = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Self::Sse {
                    url: wire.url,
                    headers: wire.headers,
                    tools: wire.tools,
                    timeout: wire.timeout,
                    always_load: wire.always_load,
                })
            }
            Some("http") => {
                let wire: RemoteWire = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Self::Http {
                    url: wire.url,
                    headers: wire.headers,
                    tools: wire.tools,
                    timeout: wire.timeout,
                    always_load: wire.always_load,
                })
            }
            Some("sdk") => {
                let wire: SdkWire = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Self::Sdk { name: wire.name })
            }
            Some(_) => Ok(Self::Raw(value)),
            None => Err(de::Error::custom(
                "MCP server config without type must include command for stdio",
            )),
        }
    }
}

impl McpServerConfig {
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            timeout: None,
            always_load: None,
        }
    }

    pub fn sse(url: impl Into<String>) -> Self {
        Self::Sse {
            url: url.into(),
            headers: BTreeMap::new(),
            tools: Vec::new(),
            timeout: None,
            always_load: None,
        }
    }

    pub fn http(url: impl Into<String>) -> Self {
        Self::Http {
            url: url.into(),
            headers: BTreeMap::new(),
            tools: Vec::new(),
            timeout: None,
            always_load: None,
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        if let Self::Stdio { args, .. } = &mut self {
            args.push(arg.into());
        }
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Self::Stdio { env, .. } = &mut self {
            env.insert(key.into(), value.into());
        }
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        match &mut self {
            Self::Sse { headers, .. } | Self::Http { headers, .. } => {
                headers.insert(key.into(), value.into());
            }
            Self::Stdio { .. } | Self::Sdk { .. } | Self::Raw(_) | Self::Unknown => {}
        }
        self
    }

    pub fn with_tool_policy(
        mut self,
        name: impl Into<String>,
        permission_policy: McpServerPermissionPolicy,
    ) -> Self {
        match &mut self {
            Self::Sse { tools, .. } | Self::Http { tools, .. } => {
                tools.push(McpServerToolPolicy {
                    name: name.into(),
                    permission_policy,
                });
            }
            Self::Stdio { .. } | Self::Sdk { .. } | Self::Raw(_) | Self::Unknown => {}
        }
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        match &mut self {
            Self::Stdio { timeout, .. }
            | Self::Sse { timeout, .. }
            | Self::Http { timeout, .. } => *timeout = Some(timeout_ms),
            Self::Sdk { .. } | Self::Raw(_) | Self::Unknown => {}
        }
        self
    }

    pub fn with_always_load(mut self, always_load_enabled: bool) -> Self {
        match &mut self {
            Self::Stdio { always_load, .. }
            | Self::Sse { always_load, .. }
            | Self::Http { always_load, .. } => *always_load = Some(always_load_enabled),
            Self::Sdk { .. } | Self::Raw(_) | Self::Unknown => {}
        }
        self
    }
}

/// MCP config can be supplied as a map, file path, or raw JSON string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServers {
    Map(BTreeMap<String, McpServerConfig>),
    PathOrJson(String),
}

impl Default for McpServers {
    fn default() -> Self {
        Self::Map(BTreeMap::new())
    }
}

/// Plugin configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SdkPluginConfig {
    Local { path: String },
}

/// Skill selection for the main session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Skills {
    All(String),
    List(Vec<String>),
}

impl Skills {
    pub fn all() -> Self {
        Self::All("all".to_owned())
    }
}

/// Per-tool built-in tool behavior configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_user_question: Option<AskUserQuestionToolConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskUserQuestionToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_format: Option<QuestionPreviewFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionPreviewFormat {
    Markdown,
    Html,
}

impl QuestionPreviewFormat {
    pub fn as_env_value(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Html => "html",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxNetworkConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_domains_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_unix_sockets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_all_unix_sockets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_local_binding: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_mach_lookup: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_proxy_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socks_proxy_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_terminate: Option<SandboxTlsTerminateConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxTlsTerminateConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_key_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFilesystemConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_write: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny_write: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny_read: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_read: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_read_paths_only: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxIgnoreViolations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network: Vec<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub other: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_if_unavailable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_allow_bash_if_sandboxed: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unsandboxed_commands: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<SandboxNetworkConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<SandboxFilesystemConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_violations: Option<SandboxIgnoreViolations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_weaker_nested_sandbox: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_weaker_network_isolation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ripgrep: Option<SandboxRipgrepConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bwrap_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socat_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRipgrepConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Output format discriminator exported by the upstream SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormatType {
    JsonSchema,
    #[serde(untagged)]
    Other(String),
}

/// Base output-format shape with a type discriminator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseOutputFormat {
    #[serde(rename = "type")]
    pub output_type: OutputFormatType,
}

/// Structured-response JSON Schema output format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonSchemaOutputFormat {
    #[serde(rename = "type")]
    pub output_type: OutputFormatType,
    pub schema: Value,
}

impl JsonSchemaOutputFormat {
    pub fn new(schema: impl Into<Value>) -> Self {
        Self {
            output_type: OutputFormatType::JsonSchema,
            schema: schema.into(),
        }
    }
}

/// Output format configuration accepted by Claude Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputFormat {
    JsonSchema(JsonSchemaOutputFormat),
    Raw(Value),
}

impl OutputFormat {
    pub fn json_schema(schema: impl Into<Value>) -> Self {
        Self::JsonSchema(JsonSchemaOutputFormat::new(schema))
    }
}

impl From<JsonSchemaOutputFormat> for OutputFormat {
    fn from(format: JsonSchemaOutputFormat) -> Self {
        Self::JsonSchema(format)
    }
}

impl From<JsonSchemaOutputFormat> for Value {
    fn from(format: JsonSchemaOutputFormat) -> Self {
        serde_json::to_value(format)
            .expect("json schema output format serialization should be infallible")
    }
}

impl From<OutputFormat> for Value {
    fn from(format: OutputFormat) -> Self {
        serde_json::to_value(format).expect("output format serialization should be infallible")
    }
}

/// Display mode for thinking/reasoning output.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingDisplay {
    Summarized,
    Omitted,
    #[serde(untagged)]
    Other(String),
}

impl ThinkingDisplay {
    pub fn as_cli_value(&self) -> &str {
        match self {
            Self::Summarized => "summarized",
            Self::Omitted => "omitted",
            Self::Other(value) => value,
        }
    }
}

impl From<&str> for ThinkingDisplay {
    fn from(value: &str) -> Self {
        match value {
            "summarized" => Self::Summarized,
            "omitted" => Self::Omitted,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl From<String> for ThinkingDisplay {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// Adaptive thinking configuration helper.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThinkingAdaptive {
    pub display: Option<ThinkingDisplay>,
}

impl Serialize for ThinkingAdaptive {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "adaptive")?;
        if let Some(display) = &self.display {
            map.serialize_entry("display", display)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ThinkingAdaptive {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(rename = "type")]
            kind: String,
            #[serde(default)]
            display: Option<ThinkingDisplay>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if wire.kind != "adaptive" {
            return Err(de::Error::custom(format!(
                "expected adaptive thinking config, got {}",
                wire.kind
            )));
        }
        Ok(Self {
            display: wire.display,
        })
    }
}

/// Fixed-budget thinking configuration helper.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThinkingEnabled {
    pub budget_tokens: Option<u32>,
    pub display: Option<ThinkingDisplay>,
}

impl Serialize for ThinkingEnabled {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "enabled")?;
        if let Some(budget_tokens) = self.budget_tokens {
            map.serialize_entry("budgetTokens", &budget_tokens)?;
        }
        if let Some(display) = &self.display {
            map.serialize_entry("display", display)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ThinkingEnabled {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(rename = "type")]
            kind: String,
            #[serde(default, rename = "budgetTokens")]
            budget_tokens: Option<u32>,
            #[serde(default)]
            display: Option<ThinkingDisplay>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if wire.kind != "enabled" {
            return Err(de::Error::custom(format!(
                "expected enabled thinking config, got {}",
                wire.kind
            )));
        }
        Ok(Self {
            budget_tokens: wire.budget_tokens,
            display: wire.display,
        })
    }
}

/// Disabled thinking configuration helper.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ThinkingDisabled;

impl Serialize for ThinkingDisabled {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("type", "disabled")?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for ThinkingDisabled {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(rename = "type")]
            kind: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        if wire.kind != "disabled" {
            return Err(de::Error::custom(format!(
                "expected disabled thinking config, got {}",
                wire.kind
            )));
        }
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThinkingConfig {
    Adaptive {
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
    Enabled {
        #[serde(
            rename = "budgetTokens",
            alias = "budgetTokens",
            skip_serializing_if = "Option::is_none"
        )]
        budget_tokens: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
    Disabled,
}

impl From<ThinkingAdaptive> for ThinkingConfig {
    fn from(config: ThinkingAdaptive) -> Self {
        Self::Adaptive {
            display: config.display,
        }
    }
}

impl From<ThinkingEnabled> for ThinkingConfig {
    fn from(config: ThinkingEnabled) -> Self {
        Self::Enabled {
            budget_tokens: config.budget_tokens,
            display: config.display,
        }
    }
}

impl From<ThinkingDisabled> for ThinkingConfig {
    fn from(_: ThinkingDisabled) -> Self {
        Self::Disabled
    }
}

/// Main query/client options for Rust SDK sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeAgentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Tools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, alias = "allowedTools", skip_serializing_if = "Vec::is_empty")]
    pub allowed_tools: Vec<String>,
    #[serde(alias = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<SystemPrompt>,
    #[serde(default, alias = "mcpServers")]
    pub mcp_servers: McpServers,
    #[serde(default, alias = "strictMcpConfig")]
    pub strict_mcp_config: bool,
    #[serde(alias = "permissionMode", skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default, alias = "continue", alias = "continueConversation")]
    pub continue_conversation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
    #[serde(alias = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(alias = "resumeSessionAt", skip_serializing_if = "Option::is_none")]
    pub resume_session_at: Option<String>,
    #[serde(alias = "maxTurns", skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(alias = "maxBudgetUsd", skip_serializing_if = "Option::is_none")]
    pub max_budget_usd: Option<f64>,
    #[serde(
        default,
        alias = "disallowedTools",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub disallowed_tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(alias = "fallbackModel", skip_serializing_if = "Option::is_none")]
    pub fallback_model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub betas: Vec<SdkBeta>,
    #[serde(default, alias = "allowDangerouslySkipPermissions")]
    pub allow_dangerously_skip_permissions: bool,
    #[serde(
        alias = "permissionPromptToolName",
        skip_serializing_if = "Option::is_none"
    )]
    pub permission_prompt_tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(
        alias = "pathToClaudeCodeExecutable",
        skip_serializing_if = "Option::is_none"
    )]
    pub cli_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<PathBuf>,
    #[serde(
        default,
        alias = "executableArgs",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub executable_args: Vec<String>,
    #[serde(alias = "debugFile", skip_serializing_if = "Option::is_none")]
    pub debug_file: Option<PathBuf>,
    #[serde(default)]
    pub debug: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_settings_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub settings: Option<String>,
    #[serde(alias = "managedSettings", skip_serializing_if = "Option::is_none")]
    pub managed_settings: Option<Value>,
    #[serde(
        default,
        alias = "additionalDirectories",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub add_dirs: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(
        default,
        alias = "extraArgs",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub extra_args: BTreeMap<String, Option<String>>,
    #[serde(alias = "maxBufferSize", skip_serializing_if = "Option::is_none")]
    pub max_buffer_size: Option<usize>,
    #[serde(default, alias = "includePartialMessages")]
    pub include_partial_messages: bool,
    #[serde(default, alias = "includeHookEvents")]
    pub include_hook_events: bool,
    #[serde(default, alias = "forkSession")]
    pub fork_session: bool,
    #[serde(default, alias = "forwardSubagentText")]
    pub forward_subagent_text: bool,
    #[serde(default, alias = "promptSuggestions")]
    pub prompt_suggestions: bool,
    #[serde(default, alias = "agentProgressSummaries")]
    pub agent_progress_summaries: bool,
    #[serde(
        default,
        alias = "toolAliases",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub tool_aliases: BTreeMap<String, String>,
    #[serde(
        alias = "planModeInstructions",
        skip_serializing_if = "Option::is_none"
    )]
    pub plan_mode_instructions: Option<String>,
    #[serde(
        alias = "appendSubagentSystemPrompt",
        skip_serializing_if = "Option::is_none"
    )]
    pub append_subagent_system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(
        default,
        alias = "webSearchIsolationExemptMcpServers",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub web_search_isolation_exempt_mcp_servers: Vec<String>,
    #[serde(alias = "persistSession", skip_serializing_if = "Option::is_none")]
    pub persist_session: Option<bool>,
    #[serde(alias = "toolConfig", skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub agents: BTreeMap<String, AgentDefinition>,
    #[serde(alias = "settingSources", skip_serializing_if = "Option::is_none")]
    pub setting_sources: Option<Vec<SettingSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Skills>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxSettings>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<SdkPluginConfig>,
    #[serde(alias = "maxThinkingTokens", skip_serializing_if = "Option::is_none")]
    pub max_thinking_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<EffortLevel>,
    #[serde(alias = "outputFormat", skip_serializing_if = "Option::is_none")]
    pub output_format: Option<Value>,
    #[serde(default, alias = "enableFileCheckpointing")]
    pub enable_file_checkpointing: bool,
    #[serde(skip)]
    pub session_store: Option<SharedSessionStore>,
    #[serde(default, alias = "sessionStoreFlush")]
    pub session_store_flush: SessionStoreFlushMode,
    #[serde(
        default,
        alias = "loadTimeoutMs",
        skip_serializing_if = "Option::is_none"
    )]
    pub load_timeout_ms: Option<u64>,
    #[serde(alias = "taskBudget", skip_serializing_if = "Option::is_none")]
    pub task_budget: Option<TaskBudget>,
    #[serde(skip)]
    pub callbacks: CallbackRegistry,
    #[serde(skip)]
    pub spawn_claude_code_process: Option<SharedClaudeProcessSpawner>,
}

impl ClaudeAgentOptions {
    pub fn builder() -> ClaudeAgentOptionsBuilder {
        ClaudeAgentOptionsBuilder::default()
    }

    /// Build the Claude CLI arguments after the executable path.
    ///
    /// Mirrors the upstream Python subprocess transport and TypeScript reference:
    /// stream-json output, verbose logs, and stream-json stdin are always enabled.
    pub fn to_cli_args(&self) -> Result<Vec<String>> {
        self.validate_sdk_mcp_servers()?;
        if self.session_store.is_some() && self.enable_file_checkpointing {
            return Err(ClaudeAgentError::InvalidOption(
                "session_store cannot be combined with enable_file_checkpointing".into(),
            ));
        }
        if self.session_store.is_some() && self.persist_session == Some(false) {
            return Err(ClaudeAgentError::InvalidOption(
                "session_store cannot be combined with persist_session(false)".into(),
            ));
        }
        if self
            .fallback_model
            .as_ref()
            .zip(self.model.as_ref())
            .is_some_and(|(fallback, model)| fallback == model)
        {
            return Err(ClaudeAgentError::InvalidOption(
                "fallback_model cannot be the same as model".into(),
            ));
        }

        let mut args = vec![
            "--output-format".into(),
            "stream-json".into(),
            "--verbose".into(),
        ];

        match &self.system_prompt {
            None | Some(SystemPrompt::Text(_)) | Some(SystemPrompt::Blocks(_)) => {}
            Some(SystemPrompt::File { path }) => args.extend([
                "--system-prompt-file".into(),
                path.to_string_lossy().into_owned(),
            ]),
            Some(SystemPrompt::Preset { .. }) => {}
        }

        if let Some(tools) = &self.tools {
            match tools {
                Tools::List(list) if list.is_empty() => args.extend(["--tools".into(), "".into()]),
                Tools::List(list) => args.extend(["--tools".into(), list.join(",")]),
                Tools::Preset { .. } => args.extend(["--tools".into(), "default".into()]),
            }
        }

        let (allowed_tools, setting_sources) = self.effective_allowed_tools_and_setting_sources();
        if !allowed_tools.is_empty() {
            args.extend(["--allowedTools".into(), allowed_tools.join(",")]);
        }
        if let Some(max_turns) = self.max_turns {
            args.extend(["--max-turns".into(), max_turns.to_string()]);
        }
        if let Some(max_budget_usd) = self.max_budget_usd {
            args.extend(["--max-budget-usd".into(), max_budget_usd.to_string()]);
        }
        if !self.disallowed_tools.is_empty() {
            args.extend(["--disallowedTools".into(), self.disallowed_tools.join(",")]);
        }
        if let Some(task_budget) = &self.task_budget {
            args.extend(["--task-budget".into(), task_budget.total.to_string()]);
        }
        if let Some(model) = &self.model {
            args.extend(["--model".into(), model.clone()]);
        }
        if let Some(agent) = &self.agent {
            args.extend(["--agent".into(), agent.clone()]);
        }
        if let Some(fallback_model) = &self.fallback_model {
            args.extend(["--fallback-model".into(), fallback_model.clone()]);
        }
        if !self.betas.is_empty() {
            args.extend(["--betas".into(), self.betas.join(",")]);
        }
        if let Some(debug_file) = &self.debug_file {
            args.extend([
                "--debug-file".into(),
                debug_file.to_string_lossy().into_owned(),
            ]);
        } else if self.debug {
            args.push("--debug".into());
        }
        if self.callbacks.can_use_tool.is_some() && self.permission_prompt_tool_name.is_some() {
            return Err(ClaudeAgentError::InvalidOption(
                "can_use_tool callback cannot be used with permission_prompt_tool_name".into(),
            ));
        }
        if let Some(tool_name) = self
            .permission_prompt_tool_name
            .as_deref()
            .or_else(|| self.callbacks.can_use_tool.as_ref().map(|_| "stdio"))
        {
            args.extend(["--permission-prompt-tool".into(), tool_name.to_owned()]);
        }
        if let Some(mode) = self.permission_mode {
            args.extend(["--permission-mode".into(), mode.as_cli_value().into()]);
        }
        if self.allow_dangerously_skip_permissions {
            args.push("--allow-dangerously-skip-permissions".into());
        }
        if self.continue_conversation {
            args.push("--continue".into());
        }
        if let Some(resume) = &self.resume {
            args.extend(["--resume".into(), resume.clone()]);
        }
        if let Some(resume_session_at) = &self.resume_session_at {
            args.extend(["--resume-session-at".into(), resume_session_at.clone()]);
        }
        if let Some(session_id) = &self.session_id {
            args.extend(["--session-id".into(), session_id.clone()]);
        }
        if self.persist_session == Some(false) {
            args.push("--no-session-persistence".into());
        }
        if let Some(managed_settings) = &self.managed_settings {
            args.extend([
                "--managed-settings".into(),
                serde_json::to_string(managed_settings)?,
            ]);
        }
        if let Some(settings) = self.settings_argument()? {
            args.extend(["--settings".into(), settings]);
        }
        for directory in &self.add_dirs {
            args.extend(["--add-dir".into(), directory.to_string_lossy().into_owned()]);
        }
        if let Some(mcp_config) = self.mcp_config_argument()? {
            args.extend(["--mcp-config".into(), mcp_config]);
        }
        if self.include_partial_messages {
            args.push("--include-partial-messages".into());
        }
        if self.include_hook_events {
            args.push("--include-hook-events".into());
        }
        if self.strict_mcp_config {
            args.push("--strict-mcp-config".into());
        }
        if self.fork_session {
            args.push("--fork-session".into());
        }
        if self.session_store.is_some() {
            args.push("--session-mirror".into());
        }
        if let Some(sources) = setting_sources {
            args.push(format!(
                "--setting-sources={}",
                sources
                    .iter()
                    .map(|s| s.as_cli_value())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        for plugin in &self.plugins {
            match plugin {
                SdkPluginConfig::Local { path } => {
                    args.extend(["--plugin-dir".into(), path.clone()])
                }
            }
        }
        for (flag, value) in &self.extra_args {
            args.push(format!("--{flag}"));
            if let Some(value) = value {
                args.push(value.clone());
            }
        }
        if let Some(thinking) = &self.thinking {
            match thinking {
                ThinkingConfig::Adaptive { display } => {
                    args.extend(["--thinking".into(), "adaptive".into()]);
                    if let Some(display) = display {
                        args.extend(["--thinking-display".into(), display.as_cli_value().into()]);
                    }
                }
                ThinkingConfig::Enabled {
                    budget_tokens,
                    display,
                } => {
                    if let Some(budget_tokens) = budget_tokens {
                        args.extend(["--max-thinking-tokens".into(), budget_tokens.to_string()]);
                    } else {
                        args.extend(["--thinking".into(), "adaptive".into()]);
                    }
                    if let Some(display) = display {
                        args.extend(["--thinking-display".into(), display.as_cli_value().into()]);
                    }
                }
                ThinkingConfig::Disabled => args.extend(["--thinking".into(), "disabled".into()]),
            }
        } else if let Some(tokens) = self.max_thinking_tokens {
            if tokens == 0 {
                args.extend(["--thinking".into(), "disabled".into()]);
            } else {
                args.extend(["--max-thinking-tokens".into(), tokens.to_string()]);
            }
        }
        if let Some(effort) = &self.effort {
            args.extend(["--effort".into(), effort.as_cli_value().into()]);
        }
        if let Some(schema) = self.output_format_json_schema()? {
            args.extend(["--json-schema".into(), serde_json::to_string(&schema)?]);
        }

        args.extend(["--input-format".into(), "stream-json".into()]);
        Ok(args)
    }

    pub(crate) fn initialize_payload(&self) -> Value {
        let mut request = Map::new();
        request.insert("subtype".into(), Value::String("initialize".into()));
        request.insert("hooks".into(), Value::Null);
        if let Some(plan_mode_instructions) = &self.plan_mode_instructions {
            request.insert(
                "planModeInstructions".into(),
                Value::String(plan_mode_instructions.clone()),
            );
        }
        if let Some(append_subagent_system_prompt) = &self.append_subagent_system_prompt {
            request.insert(
                "appendSubagentSystemPrompt".into(),
                Value::String(append_subagent_system_prompt.clone()),
            );
        }
        match &self.system_prompt {
            None => {
                request.insert(
                    "systemPrompt".into(),
                    Value::Array(vec![Value::String(String::new())]),
                );
            }
            Some(SystemPrompt::Text(text)) => {
                request.insert(
                    "systemPrompt".into(),
                    Value::Array(vec![Value::String(text.clone())]),
                );
            }
            Some(SystemPrompt::Blocks(blocks)) => {
                request.insert(
                    "systemPrompt".into(),
                    Value::Array(blocks.iter().cloned().map(Value::String).collect()),
                );
            }
            Some(SystemPrompt::Preset {
                append: Some(append),
                ..
            }) => {
                request.insert("appendSystemPrompt".into(), Value::String(append.clone()));
            }
            Some(SystemPrompt::Preset { append: None, .. }) | Some(SystemPrompt::File { .. }) => {}
        }
        if !self.tool_aliases.is_empty() {
            request.insert(
                "toolAliases".into(),
                serde_json::to_value(&self.tool_aliases).unwrap_or(Value::Null),
            );
        }
        if let Some(title) = &self.title {
            request.insert("title".into(), Value::String(title.clone()));
        }
        if !self.web_search_isolation_exempt_mcp_servers.is_empty() {
            request.insert(
                "webSearchIsolationExemptMcpServers".into(),
                Value::Array(
                    self.web_search_isolation_exempt_mcp_servers
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if self.prompt_suggestions {
            request.insert("promptSuggestions".into(), Value::Bool(true));
        }
        if self.agent_progress_summaries {
            request.insert("agentProgressSummaries".into(), Value::Bool(true));
        }
        if self.forward_subagent_text {
            request.insert("forwardSubagentText".into(), Value::Bool(true));
        }
        if !self.agents.is_empty() {
            request.insert(
                "agents".into(),
                serde_json::to_value(&self.agents).unwrap_or(Value::Null),
            );
        }
        if let Some(SystemPrompt::Preset {
            exclude_dynamic_sections: Some(value),
            ..
        }) = &self.system_prompt
        {
            request.insert("excludeDynamicSections".into(), Value::Bool(*value));
        }
        if let Some(Skills::List(skills)) = &self.skills {
            request.insert(
                "skills".into(),
                Value::Array(skills.iter().cloned().map(Value::String).collect()),
            );
        }
        Value::Object(request)
    }

    fn effective_allowed_tools_and_setting_sources(
        &self,
    ) -> (Vec<String>, Option<Vec<SettingSource>>) {
        let mut allowed_tools = self.allowed_tools.clone();
        let mut setting_sources = self.setting_sources.clone();

        match &self.skills {
            None => {}
            Some(Skills::All(_)) => {
                if !allowed_tools.iter().any(|tool| tool == "Skill") {
                    allowed_tools.push("Skill".into());
                }
                if setting_sources.is_none() {
                    setting_sources = Some(vec![SettingSource::User, SettingSource::Project]);
                }
            }
            Some(Skills::List(skills)) => {
                for skill in skills {
                    let pattern = format!("Skill({skill})");
                    if !allowed_tools.iter().any(|tool| tool == &pattern) {
                        allowed_tools.push(pattern);
                    }
                }
                if setting_sources.is_none() {
                    setting_sources = Some(vec![SettingSource::User, SettingSource::Project]);
                }
            }
        }

        (allowed_tools, setting_sources)
    }

    fn settings_argument(&self) -> Result<Option<String>> {
        match (&self.settings, &self.sandbox) {
            (None, None) => Ok(None),
            (Some(settings), None) => Ok(Some(settings.clone())),
            (settings, Some(sandbox)) => {
                let mut object = if let Some(settings) = settings {
                    parse_settings_object(settings)?
                } else {
                    Map::new()
                };
                object.insert("sandbox".into(), serde_json::to_value(sandbox)?);
                Ok(Some(Value::Object(object).to_string()))
            }
        }
    }

    fn mcp_config_argument(&self) -> Result<Option<String>> {
        match &self.mcp_servers {
            McpServers::PathOrJson(path_or_json) if !path_or_json.is_empty() => {
                Ok(Some(path_or_json.clone()))
            }
            McpServers::PathOrJson(_) => Ok(None),
            McpServers::Map(map) if map.is_empty() => Ok(None),
            McpServers::Map(map) => {
                let mut outer = Map::new();
                outer.insert("mcpServers".into(), serde_json::to_value(map)?);
                Ok(Some(Value::Object(outer).to_string()))
            }
        }
    }

    fn validate_sdk_mcp_servers(&self) -> Result<()> {
        if self.callbacks.sdk_mcp_servers.is_empty() {
            return Ok(());
        }

        let McpServers::Map(map) = &self.mcp_servers else {
            return Err(ClaudeAgentError::InvalidOption(
                "sdk_mcp_server requires map-based mcp_servers so the CLI receives matching SDK server descriptors".into(),
            ));
        };

        for alias in self.callbacks.sdk_mcp_servers.keys() {
            match map.get(alias) {
                Some(McpServerConfig::Sdk { .. }) => {}
                Some(_) => {
                    return Err(ClaudeAgentError::InvalidOption(format!(
                        "sdk_mcp_server alias '{alias}' conflicts with a non-SDK MCP server config",
                    )));
                }
                None => {
                    return Err(ClaudeAgentError::InvalidOption(format!(
                        "sdk_mcp_server alias '{alias}' is missing from mcp_servers",
                    )));
                }
            }
        }

        Ok(())
    }

    fn output_format_json_schema(&self) -> Result<Option<Value>> {
        let Some(output_format) = &self.output_format else {
            return Ok(None);
        };
        if output_format.get("type").and_then(Value::as_str) == Some("json_schema") {
            Ok(output_format.get("schema").cloned())
        } else {
            Ok(None)
        }
    }
}

fn parse_settings_object(settings: &str) -> Result<Map<String, Value>> {
    let trimmed = settings.trim();
    if trimmed.is_empty() {
        return Ok(Map::new());
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let value: Value = serde_json::from_str(trimmed)?;
        return value.as_object().cloned().ok_or_else(|| {
            ClaudeAgentError::InvalidOption("settings JSON must be an object".into())
        });
    }

    let path = PathBuf::from(settings);
    if path.exists() {
        let value: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
        return value.as_object().cloned().ok_or_else(|| {
            ClaudeAgentError::InvalidOption("settings file JSON must be an object".into())
        });
    }

    Err(ClaudeAgentError::InvalidOption(format!(
        "settings file does not exist: {settings}"
    )))
}

fn deserialize_settings_option<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(settings)) => Ok(Some(settings)),
        Some(Value::Object(object)) => Ok(Some(Value::Object(object).to_string())),
        Some(other) => Err(de::Error::custom(format!(
            "settings must be a string path/JSON object or an inline object, got {other}"
        ))),
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClaudeAgentOptionsBuilder {
    options: ClaudeAgentOptions,
}

impl ClaudeAgentOptionsBuilder {
    pub fn build(self) -> ClaudeAgentOptions {
        self.options
    }

    pub fn cli_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.cli_path = Some(path.into());
        self
    }

    pub fn executable(mut self, executable: impl Into<PathBuf>) -> Self {
        self.options.executable = Some(executable.into());
        self
    }

    pub fn executable_arg(mut self, arg: impl Into<String>) -> Self {
        self.options.executable_args.push(arg.into());
        self
    }

    pub fn executable_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options.executable_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.cwd = Some(path.into());
        self
    }

    pub fn agent(mut self, agent: impl Into<String>) -> Self {
        self.options.agent = Some(agent.into());
        self
    }

    pub fn allowed_tool(mut self, tool: impl Into<String>) -> Self {
        self.options.allowed_tools.push(tool.into());
        self
    }

    pub fn allowed_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options.allowed_tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn disallowed_tool(mut self, tool: impl Into<String>) -> Self {
        self.options.disallowed_tools.push(tool.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<SystemPrompt>) -> Self {
        self.options.system_prompt = Some(prompt.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.options.model = Some(model.into());
        self
    }

    pub fn fallback_model(mut self, model: impl Into<String>) -> Self {
        self.options.fallback_model = Some(model.into());
        self
    }

    pub fn max_turns(mut self, max_turns: u32) -> Self {
        self.options.max_turns = Some(max_turns);
        self
    }

    pub fn max_budget_usd(mut self, max_budget_usd: f64) -> Self {
        self.options.max_budget_usd = Some(max_budget_usd);
        self
    }

    pub fn task_budget(mut self, total: u64) -> Self {
        self.options.task_budget = Some(TaskBudget { total });
        self
    }

    pub fn resume(mut self, session_id: impl Into<String>) -> Self {
        self.options.resume = Some(session_id.into());
        self
    }

    pub fn resume_session_at(mut self, message_uuid: impl Into<String>) -> Self {
        self.options.resume_session_at = Some(message_uuid.into());
        self
    }

    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.options.session_id = Some(session_id.into());
        self
    }

    pub fn continue_conversation(mut self, enabled: bool) -> Self {
        self.options.continue_conversation = enabled;
        self
    }

    pub fn permission_mode(mut self, mode: PermissionMode) -> Self {
        self.options.permission_mode = Some(mode);
        self
    }

    pub fn allow_dangerously_skip_permissions(mut self, enabled: bool) -> Self {
        self.options.allow_dangerously_skip_permissions = enabled;
        self
    }

    pub fn mcp_servers(mut self, servers: BTreeMap<String, McpServerConfig>) -> Self {
        self.options.mcp_servers = McpServers::Map(servers);
        self
    }

    pub fn setting_sources(mut self, sources: Vec<SettingSource>) -> Self {
        self.options.setting_sources = Some(sources);
        self
    }

    pub fn skills(mut self, skills: Skills) -> Self {
        self.options.skills = Some(skills);
        self
    }

    pub fn debug(mut self, enabled: bool) -> Self {
        self.options.debug = enabled;
        self
    }

    pub fn debug_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.debug_file = Some(path.into());
        self
    }

    pub fn settings(mut self, settings: impl Into<String>) -> Self {
        self.options.settings = Some(settings.into());
        self
    }

    pub fn managed_settings(mut self, settings: impl Into<Value>) -> Self {
        self.options.managed_settings = Some(settings.into());
        self
    }

    pub fn add_dir(mut self, directory: impl Into<PathBuf>) -> Self {
        self.options.add_dirs.push(directory.into());
        self
    }

    pub fn add_dirs(mut self, directories: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.options.add_dirs = directories.into_iter().map(Into::into).collect();
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.env.insert(key.into(), value.into());
        self
    }

    pub fn extra_arg(mut self, flag: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        self.options
            .extra_args
            .insert(flag.into(), value.map(Into::into));
        self
    }

    pub fn output_format(mut self, output_format: impl Into<Value>) -> Self {
        self.options.output_format = Some(output_format.into());
        self
    }

    pub fn output_format_json_schema(mut self, schema: impl Into<Value>) -> Self {
        self.options.output_format = Some(serde_json::json!({
            "type": "json_schema",
            "schema": schema.into(),
        }));
        self
    }

    pub fn include_partial_messages(mut self, enabled: bool) -> Self {
        self.options.include_partial_messages = enabled;
        self
    }

    pub fn include_hook_events(mut self, enabled: bool) -> Self {
        self.options.include_hook_events = enabled;
        self
    }

    pub fn fork_session(mut self, enabled: bool) -> Self {
        self.options.fork_session = enabled;
        self
    }

    pub fn forward_subagent_text(mut self, enabled: bool) -> Self {
        self.options.forward_subagent_text = enabled;
        self
    }

    pub fn prompt_suggestions(mut self, enabled: bool) -> Self {
        self.options.prompt_suggestions = enabled;
        self
    }

    pub fn agent_progress_summaries(mut self, enabled: bool) -> Self {
        self.options.agent_progress_summaries = enabled;
        self
    }

    pub fn plan_mode_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.options.plan_mode_instructions = Some(instructions.into());
        self
    }

    pub fn append_subagent_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.options.append_subagent_system_prompt = Some(prompt.into());
        self
    }

    pub fn tool_alias(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.options.tool_aliases.insert(from.into(), to.into());
        self
    }

    pub fn tool_aliases(mut self, aliases: BTreeMap<String, String>) -> Self {
        self.options.tool_aliases = aliases;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    pub fn web_search_isolation_exempt_mcp_server(mut self, server: impl Into<String>) -> Self {
        self.options
            .web_search_isolation_exempt_mcp_servers
            .push(server.into());
        self
    }

    pub fn persist_session(mut self, persist: bool) -> Self {
        self.options.persist_session = Some(persist);
        self
    }

    pub fn ask_user_question_preview_format(mut self, format: QuestionPreviewFormat) -> Self {
        let config = self
            .options
            .tool_config
            .get_or_insert_with(ToolConfig::default);
        let ask_user_question = config
            .ask_user_question
            .get_or_insert_with(AskUserQuestionToolConfig::default);
        ask_user_question.preview_format = Some(format);
        self
    }

    pub fn tool_config(mut self, config: ToolConfig) -> Self {
        self.options.tool_config = Some(config);
        self
    }

    pub fn can_use_tool<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(ToolPermissionRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<crate::callbacks::PermissionResult>>
            + Send
            + 'static,
    {
        let callback: PermissionCallback =
            std::sync::Arc::new(move |request| Box::pin(callback(request)));
        self.options.callbacks.can_use_tool = Some(callback);
        self
    }

    pub fn on_elicitation<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(ElicitationRequest, ElicitationCallbackOptions) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<ElicitationResult>> + Send + 'static,
    {
        let callback: ElicitationCallback =
            std::sync::Arc::new(move |request, options| Box::pin(callback(request, options)));
        self.options.callbacks.on_elicitation = Some(callback);
        self
    }

    pub fn get_oauth_token<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(TokenRefreshCallbackOptions) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Option<String>>> + Send + 'static,
    {
        let callback: TokenRefreshCallback =
            std::sync::Arc::new(move |options| Box::pin(callback(options)));
        self.options.callbacks.get_oauth_token = Some(callback);
        self
    }

    pub fn get_host_auth_token<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(TokenRefreshCallbackOptions) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Option<String>>> + Send + 'static,
    {
        let callback: TokenRefreshCallback =
            std::sync::Arc::new(move |options| Box::pin(callback(options)));
        self.options.callbacks.get_host_auth_token = Some(callback);
        self
    }

    pub fn on_user_dialog<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(UserDialogRequest, UserDialogCallbackOptions) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let callback: UserDialogCallback =
            std::sync::Arc::new(move |request, options| Box::pin(callback(request, options)));
        self.options.callbacks.on_user_dialog = Some(callback);
        self
    }

    pub fn stderr<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let callback: StderrCallback = std::sync::Arc::new(callback);
        self.options.callbacks.stderr = Some(callback);
        self
    }

    pub fn hook<F, Fut>(
        mut self,
        event: impl Into<String>,
        matcher: Option<impl Into<String>>,
        timeout: Option<f64>,
        callback: F,
    ) -> Self
    where
        F: Fn(crate::callbacks::HookCallbackRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let callback: HookCallback =
            std::sync::Arc::new(move |request| Box::pin(callback(request)));
        self.options
            .callbacks
            .hooks
            .entry(event.into())
            .or_default()
            .push(HookMatcher {
                matcher: matcher.map(Into::into),
                hooks: vec![callback],
                timeout,
            });
        self
    }

    pub fn sdk_mcp_server(mut self, alias: impl Into<String>, server: SdkMcpServer) -> Self {
        let alias = alias.into();
        let server_name = server.name.clone();
        self.options
            .callbacks
            .sdk_mcp_servers
            .insert(alias.clone(), Arc::new(server));

        let mut servers = match std::mem::take(&mut self.options.mcp_servers) {
            McpServers::Map(map) => map,
            McpServers::PathOrJson(value) if value.is_empty() => BTreeMap::new(),
            McpServers::PathOrJson(value) => {
                self.options.mcp_servers = McpServers::PathOrJson(value);
                return self;
            }
        };
        servers.insert(alias, McpServerConfig::Sdk { name: server_name });
        self.options.mcp_servers = McpServers::Map(servers);
        self
    }

    pub fn session_store<S>(mut self, store: S) -> Self
    where
        S: SessionStore + 'static,
    {
        self.options.session_store = Some(SharedSessionStore::new(store));
        self
    }

    pub fn shared_session_store(mut self, store: SharedSessionStore) -> Self {
        self.options.session_store = Some(store);
        self
    }

    pub fn session_store_flush(mut self, flush: SessionStoreFlushMode) -> Self {
        self.options.session_store_flush = flush;
        self
    }

    pub fn load_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.options.load_timeout_ms = Some(timeout_ms);
        self
    }

    pub fn spawn_claude_code_process<S>(mut self, spawner: S) -> Self
    where
        S: ClaudeProcessSpawner + 'static,
    {
        self.options.spawn_claude_code_process = Some(SharedClaudeProcessSpawner::new(spawner));
        self
    }
}

pub(crate) fn find_cli(explicit: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(path.clone());
        }
        if is_bare_command(path)
            && let Some(resolved) = find_on_path(path)
        {
            return Ok(resolved);
        }
        return Err(ClaudeAgentError::CliNotFoundAt { path: path.clone() });
    }

    if let Some(path) = find_on_path(Path::new("claude")) {
        return Ok(path);
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    let candidates = home
        .into_iter()
        .flat_map(|home| {
            vec![
                home.join(".npm-global/bin/claude"),
                home.join(".local/bin/claude"),
                home.join("node_modules/.bin/claude"),
                home.join(".yarn/bin/claude"),
                home.join(".claude/local/claude"),
            ]
        })
        .chain([PathBuf::from("/usr/local/bin/claude")]);

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ClaudeAgentError::CliNotFound)
}

fn is_bare_command(path: &Path) -> bool {
    let mut components = path.components();
    matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none()
}

fn find_on_path(binary: &Path) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, sync::Mutex};
    use tempfile::tempdir;

    static PATH_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct PathEnvGuard {
        previous: Option<OsString>,
    }

    impl PathEnvGuard {
        fn prepend(path: &std::path::Path) -> Self {
            let previous = std::env::var_os("PATH");
            let paths = std::iter::once(path.to_path_buf())
                .chain(
                    previous
                        .as_ref()
                        .into_iter()
                        .flat_map(|value| std::env::split_paths(value)),
                )
                .collect::<Vec<_>>();
            let joined = std::env::join_paths(paths).unwrap();
            // SAFETY: This test serializes PATH mutations with PATH_ENV_LOCK and
            // restores the previous value in Drop before releasing the lock.
            unsafe {
                std::env::set_var("PATH", joined);
            }
            Self { previous }
        }
    }

    impl Drop for PathEnvGuard {
        fn drop(&mut self) {
            // SAFETY: Protected by PATH_ENV_LOCK for the full guard lifetime.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var("PATH", value),
                    None => std::env::remove_var("PATH"),
                }
            }
        }
    }

    #[test]
    fn explicit_bare_cli_path_resolves_through_path() {
        let _lock = PATH_ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cli = dir.path().join("claude");
        fs::write(&cli, "#!/bin/sh\n").unwrap();
        let _path_guard = PathEnvGuard::prepend(dir.path());

        let requested = PathBuf::from("claude");
        assert_eq!(find_cli(Some(&requested)).unwrap(), cli);
    }

    #[test]
    fn skills_inject_allowed_tools_and_setting_sources() {
        let options = ClaudeAgentOptions::builder()
            .skills(Skills::List(vec!["review".into()]))
            .build();
        let args = options.to_cli_args().unwrap();
        assert!(
            args.windows(2)
                .any(|w| w == ["--allowedTools", "Skill(review)"])
        );
        assert!(
            args.iter()
                .any(|arg| arg == "--setting-sources=user,project")
        );
    }

    #[test]
    fn emits_major_cli_flags() {
        let options = ClaudeAgentOptions::builder()
            .system_prompt("You are useful")
            .allowed_tools(["Read", "Bash"])
            .permission_mode(PermissionMode::AcceptEdits)
            .max_turns(3)
            .model("claude-test")
            .build();
        let args = options.to_cli_args().unwrap();
        assert_eq!(args[0..3], ["--output-format", "stream-json", "--verbose"]);
        assert!(
            args.windows(2)
                .any(|w| w == ["--allowedTools", "Read,Bash"])
        );
        assert!(
            args.windows(2)
                .any(|w| w == ["--permission-mode", "acceptEdits"])
        );
        assert!(args.windows(2).any(|w| w == ["--max-turns", "3"]));
        assert!(args.windows(2).any(|w| w == ["--model", "claude-test"]));
        assert_eq!(args[args.len() - 2..], ["--input-format", "stream-json"]);
    }

    #[test]
    fn system_prompt_uses_current_initialize_payload_shape() {
        assert_eq!(
            ClaudeAgentOptions::default().initialize_payload()["systemPrompt"],
            serde_json::json!([""])
        );

        let text = ClaudeAgentOptions::builder()
            .system_prompt("static prompt")
            .build();
        let args = text.to_cli_args().unwrap();
        assert!(!args.iter().any(|arg| arg == "--system-prompt"));
        assert_eq!(
            text.initialize_payload()["systemPrompt"],
            serde_json::json!(["static prompt"])
        );

        let blocks = ClaudeAgentOptions::builder()
            .system_prompt(SystemPrompt::Blocks(vec![
                "static instructions".into(),
                crate::SYSTEM_PROMPT_DYNAMIC_BOUNDARY.into(),
                "session context".into(),
            ]))
            .build();
        let args = blocks.to_cli_args().unwrap();
        assert!(!args.iter().any(|arg| arg == "--system-prompt"));
        assert_eq!(
            blocks.initialize_payload()["systemPrompt"],
            serde_json::json!([
                "static instructions",
                crate::SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
                "session context"
            ])
        );

        let preset = ClaudeAgentOptions::builder()
            .system_prompt(SystemPrompt::Preset {
                preset: "claude_code".into(),
                append: Some("extra instructions".into()),
                exclude_dynamic_sections: Some(true),
            })
            .build();
        let args = preset.to_cli_args().unwrap();
        assert!(!args.iter().any(|arg| arg == "--append-system-prompt"));
        let payload = preset.initialize_payload();
        assert_eq!(payload["appendSystemPrompt"], "extra instructions");
        assert_eq!(payload["excludeDynamicSections"], true);
    }

    #[test]
    fn emits_current_upstream_option_flags_and_initialize_payload() {
        let options = ClaudeAgentOptions::builder()
            .agent("reviewer")
            .allow_dangerously_skip_permissions(true)
            .debug(true)
            .debug_file("/tmp/claude-debug.log")
            .managed_settings(serde_json::json!({"permissions": {"deny": ["Bash(rm *)"]}}))
            .persist_session(false)
            .resume_session_at("assistant-uuid")
            .plan_mode_instructions("write a plan")
            .append_subagent_system_prompt("subagent note")
            .tool_alias("Bash", "mcp__workspace__bash")
            .title("SDK parity")
            .web_search_isolation_exempt_mcp_server("docs")
            .prompt_suggestions(true)
            .agent_progress_summaries(true)
            .forward_subagent_text(true)
            .build();
        let args = options.to_cli_args().unwrap();
        assert!(args.windows(2).any(|w| w == ["--agent", "reviewer"]));
        assert!(
            args.iter()
                .any(|arg| arg == "--allow-dangerously-skip-permissions")
        );
        assert!(
            args.windows(2)
                .any(|w| w == ["--debug-file", "/tmp/claude-debug.log"])
        );
        assert!(!args.iter().any(|arg| arg == "--debug"));
        assert!(args.iter().any(|arg| arg == "--no-session-persistence"));
        assert!(
            args.windows(2)
                .any(|w| w == ["--resume-session-at", "assistant-uuid"])
        );
        let managed_settings = args
            .windows(2)
            .find_map(|window| (window[0] == "--managed-settings").then(|| &window[1]))
            .expect("managed settings arg");
        let managed_settings: Value = serde_json::from_str(managed_settings).unwrap();
        assert_eq!(
            managed_settings["permissions"]["deny"],
            serde_json::json!(["Bash(rm *)"])
        );

        let init = options.initialize_payload();
        assert_eq!(init["planModeInstructions"], "write a plan");
        assert_eq!(init["appendSubagentSystemPrompt"], "subagent note");
        assert_eq!(init["toolAliases"]["Bash"], "mcp__workspace__bash");
        assert_eq!(init["title"], "SDK parity");
        assert_eq!(
            init["webSearchIsolationExemptMcpServers"],
            serde_json::json!(["docs"])
        );
        assert_eq!(init["promptSuggestions"], true);
        assert_eq!(init["agentProgressSummaries"], true);
        assert_eq!(init["forwardSubagentText"], true);
    }

    #[test]
    fn sdk_mcp_server_emits_serializable_config() {
        let server = crate::tools::create_sdk_mcp_server("calculator", "1.0.0", vec![]);
        let options = ClaudeAgentOptions::builder()
            .sdk_mcp_server("calc", server)
            .build();
        let args = options.to_cli_args().unwrap();
        let config = args
            .windows(2)
            .find_map(|window| (window[0] == "--mcp-config").then(|| &window[1]))
            .expect("mcp config");
        let config: Value = serde_json::from_str(config).unwrap();
        assert_eq!(config["mcpServers"]["calc"]["type"], "sdk");
        assert_eq!(config["mcpServers"]["calc"]["name"], "calculator");
    }

    #[test]
    fn mcp_server_config_emits_current_remote_fields() {
        let mut servers = BTreeMap::new();
        servers.insert(
            "filesystem".to_owned(),
            McpServerConfig::Stdio {
                command: "fs-mcp".to_owned(),
                args: vec!["--root".to_owned(), ".".to_owned()],
                env: BTreeMap::new(),
                timeout: Some(2500),
                always_load: Some(true),
            },
        );
        servers.insert(
            "docs".to_owned(),
            McpServerConfig::Http {
                url: "https://mcp.example/http".to_owned(),
                headers: BTreeMap::new(),
                tools: vec![crate::status::McpServerToolPolicy {
                    name: "read_docs".to_owned(),
                    permission_policy: crate::status::McpServerPermissionPolicy::AlwaysDeny,
                }],
                timeout: Some(5000),
                always_load: Some(false),
            },
        );

        let options = ClaudeAgentOptions::builder().mcp_servers(servers).build();
        let args = options.to_cli_args().unwrap();
        let config = args
            .windows(2)
            .find_map(|window| (window[0] == "--mcp-config").then(|| &window[1]))
            .expect("mcp config");
        let config: Value = serde_json::from_str(config).unwrap();
        assert_eq!(config["mcpServers"]["filesystem"]["timeout"], 2500);
        assert_eq!(config["mcpServers"]["filesystem"]["alwaysLoad"], true);
        assert_eq!(config["mcpServers"]["docs"]["timeout"], 5000);
        assert_eq!(config["mcpServers"]["docs"]["alwaysLoad"], false);
        assert_eq!(
            config["mcpServers"]["docs"]["tools"][0]["permission_policy"],
            "always_deny"
        );
    }

    #[test]
    fn mcp_server_config_ignores_subsecond_timeouts_like_upstream() {
        let mut servers = BTreeMap::new();
        servers.insert(
            "filesystem".to_owned(),
            McpServerConfig::Stdio {
                command: "fs-mcp".to_owned(),
                args: Vec::new(),
                env: BTreeMap::new(),
                timeout: Some(999),
                always_load: None,
            },
        );
        servers.insert(
            "docs".to_owned(),
            McpServerConfig::Http {
                url: "https://mcp.example/http".to_owned(),
                headers: BTreeMap::new(),
                tools: Vec::new(),
                timeout: Some(500),
                always_load: None,
            },
        );
        servers.insert(
            "events".to_owned(),
            McpServerConfig::Sse {
                url: "https://mcp.example/sse".to_owned(),
                headers: BTreeMap::new(),
                tools: Vec::new(),
                timeout: Some(1000),
                always_load: None,
            },
        );

        let options = ClaudeAgentOptions::builder().mcp_servers(servers).build();
        let args = options.to_cli_args().unwrap();
        let config = args
            .windows(2)
            .find_map(|window| (window[0] == "--mcp-config").then(|| &window[1]))
            .expect("mcp config");
        let config: Value = serde_json::from_str(config).unwrap();
        assert!(config["mcpServers"]["filesystem"].get("timeout").is_none());
        assert!(config["mcpServers"]["docs"].get("timeout").is_none());
        assert_eq!(config["mcpServers"]["events"]["timeout"], 1000);
    }

    #[test]
    fn deserializes_stdio_mcp_server_without_type_tag() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "mcpServers": {
                "filesystem": {
                    "command": "fs-mcp",
                    "args": ["--root", "."],
                    "env": {"ROOT": "."},
                    "timeout": 2500,
                    "alwaysLoad": true
                }
            }
        }))
        .unwrap();

        let McpServers::Map(servers) = &options.mcp_servers else {
            panic!("expected map config");
        };
        assert!(matches!(
            servers.get("filesystem"),
            Some(McpServerConfig::Stdio {
                command,
                timeout: Some(2500),
                always_load: Some(true),
                ..
            }) if command == "fs-mcp"
        ));

        let args = options.to_cli_args().unwrap();
        let config = args
            .windows(2)
            .find_map(|window| (window[0] == "--mcp-config").then(|| &window[1]))
            .expect("mcp config");
        let config: Value = serde_json::from_str(config).unwrap();
        assert_eq!(config["mcpServers"]["filesystem"]["type"], "stdio");
        assert_eq!(config["mcpServers"]["filesystem"]["command"], "fs-mcp");
    }

    #[test]
    fn preserves_unknown_mcp_server_configs_for_protocol_drift() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "mcpServers": {
                "workspace": {
                    "type": "ws",
                    "url": "wss://mcp.example/ws",
                    "headersHelper": "mcp-headers",
                    "role": "comms",
                    "timeout": 2500,
                    "alwaysLoad": true
                }
            }
        }))
        .unwrap();

        let args = options.to_cli_args().unwrap();
        let config = args
            .windows(2)
            .find_map(|window| (window[0] == "--mcp-config").then(|| &window[1]))
            .expect("mcp config");
        let config: Value = serde_json::from_str(config).unwrap();
        assert_eq!(config["mcpServers"]["workspace"]["type"], "ws");
        assert_eq!(
            config["mcpServers"]["workspace"]["headersHelper"],
            "mcp-headers"
        );
        assert_eq!(config["mcpServers"]["workspace"]["role"], "comms");
        assert_eq!(config["mcpServers"]["workspace"]["timeout"], 2500);
        assert_eq!(config["mcpServers"]["workspace"]["alwaysLoad"], true);
    }

    #[test]
    fn deserializes_typescript_option_aliases() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "allowedTools": ["Read"],
            "disallowedTools": ["Bash"],
            "pathToClaudeCodeExecutable": "/tmp/claude",
            "executableArgs": ["--launcher-flag"],
            "additionalDirectories": ["/workspace/extra"],
            "continue": true,
            "maxTurns": 7,
            "maxBudgetUsd": 1.5,
            "fallbackModel": "claude-fallback",
            "enableFileCheckpointing": true,
            "forkSession": true,
            "includePartialMessages": true,
            "includeHookEvents": true,
            "forwardSubagentText": true,
            "promptSuggestions": true,
            "agentProgressSummaries": true,
            "toolAliases": {"Bash": "mcp__sandbox__bash"},
            "planModeInstructions": "plan first",
            "permissionMode": "plan",
            "allowDangerouslySkipPermissions": true,
            "permissionPromptToolName": "stdio",
            "sessionId": "00000000-0000-0000-0000-000000000001",
            "resumeSessionAt": "assistant-uuid",
            "strictMcpConfig": true,
            "debugFile": "/tmp/claude-debug.log",
            "managedSettings": {"permissions": {"deny": ["Bash(rm *)"]}},
            "settingSources": ["user"],
            "systemPrompt": {
                "type": "preset",
                "preset": "claude_code",
                "append": "extra instructions",
                "excludeDynamicSections": true
            },
            "thinking": {"type": "enabled", "budgetTokens": 1234},
            "outputFormat": {"type": "json_schema", "schema": {"type": "object"}},
            "taskBudget": {"total": 1000},
            "loadTimeoutMs": 42,
            "sessionStoreFlush": "eager",
            "persistSession": false,
            "mcpServers": {
                "docs": {
                    "type": "http",
                    "url": "https://mcp.example/http",
                    "timeout": 5000,
                    "alwaysLoad": true
                }
            }
        }))
        .unwrap();

        assert_eq!(options.allowed_tools, ["Read"]);
        assert_eq!(options.disallowed_tools, ["Bash"]);
        assert_eq!(options.cli_path, Some(PathBuf::from("/tmp/claude")));
        assert_eq!(options.executable_args, ["--launcher-flag"]);
        assert_eq!(options.add_dirs, [PathBuf::from("/workspace/extra")]);
        assert!(options.continue_conversation);
        assert_eq!(options.max_turns, Some(7));
        assert_eq!(options.max_budget_usd, Some(1.5));
        assert_eq!(options.fallback_model.as_deref(), Some("claude-fallback"));
        assert!(options.enable_file_checkpointing);
        assert!(options.fork_session);
        assert!(options.include_partial_messages);
        assert!(options.include_hook_events);
        assert!(options.forward_subagent_text);
        assert!(options.prompt_suggestions);
        assert!(options.agent_progress_summaries);
        assert_eq!(
            options.tool_aliases.get("Bash").map(String::as_str),
            Some("mcp__sandbox__bash")
        );
        assert_eq!(
            options.plan_mode_instructions.as_deref(),
            Some("plan first")
        );
        assert_eq!(options.permission_mode, Some(PermissionMode::Plan));
        assert!(options.allow_dangerously_skip_permissions);
        assert_eq!(
            options.permission_prompt_tool_name.as_deref(),
            Some("stdio")
        );
        assert_eq!(
            options.session_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000001")
        );
        assert_eq!(options.resume_session_at.as_deref(), Some("assistant-uuid"));
        assert!(options.strict_mcp_config);
        assert_eq!(
            options.debug_file,
            Some(PathBuf::from("/tmp/claude-debug.log"))
        );
        assert_eq!(
            options.managed_settings.as_ref().unwrap()["permissions"]["deny"],
            serde_json::json!(["Bash(rm *)"])
        );
        assert_eq!(options.setting_sources, Some(vec![SettingSource::User]));
        assert!(matches!(
            options.system_prompt,
            Some(SystemPrompt::Preset {
                exclude_dynamic_sections: Some(true),
                ..
            })
        ));
        assert!(matches!(
            options.thinking,
            Some(ThinkingConfig::Enabled {
                budget_tokens: Some(1234),
                ..
            })
        ));
        assert_eq!(
            options.output_format.as_ref().unwrap()["type"],
            "json_schema"
        );
        assert_eq!(
            options.task_budget.as_ref().map(|budget| budget.total),
            Some(1000)
        );
        assert_eq!(options.load_timeout_ms, Some(42));
        assert_eq!(options.session_store_flush, SessionStoreFlushMode::Eager);
        assert_eq!(options.persist_session, Some(false));
        assert!(matches!(
            &options.mcp_servers,
            McpServers::Map(servers)
                if matches!(servers.get("docs"), Some(McpServerConfig::Http { timeout: Some(5000), always_load: Some(true), .. }))
        ));
    }

    #[test]
    fn deserializes_typescript_executable_option() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "executable": "node",
            "executableArgs": ["--runtime-flag"],
            "pathToClaudeCodeExecutable": "/tmp/claude.mjs"
        }))
        .unwrap();

        assert_eq!(options.executable, Some(PathBuf::from("node")));
        assert_eq!(options.executable_args, ["--runtime-flag"]);
        assert_eq!(options.cli_path, Some(PathBuf::from("/tmp/claude.mjs")));
    }

    #[test]
    fn deserializes_typescript_inline_settings_object() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "settings": {
                "model": "claude-test",
                "permissions": {"allow": ["Read"]}
            }
        }))
        .unwrap();

        let args = options.to_cli_args().unwrap();
        let settings = args
            .windows(2)
            .find_map(|window| (window[0] == "--settings").then(|| &window[1]))
            .expect("settings arg");
        let settings: Value = serde_json::from_str(settings).unwrap();
        assert_eq!(settings["model"], "claude-test");
        assert_eq!(
            settings["permissions"]["allow"],
            serde_json::json!(["Read"])
        );
    }

    #[test]
    fn deserializes_typescript_system_prompt_string_and_blocks() {
        let text: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "systemPrompt": "static prompt"
        }))
        .unwrap();
        assert_eq!(
            text.initialize_payload()["systemPrompt"],
            serde_json::json!(["static prompt"])
        );

        let blocks: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "systemPrompt": [
                "static instructions",
                crate::SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
                "session context"
            ]
        }))
        .unwrap();
        assert_eq!(
            blocks.initialize_payload()["systemPrompt"],
            serde_json::json!([
                "static instructions",
                crate::SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
                "session context"
            ])
        );
    }

    #[test]
    fn deserializes_enabled_thinking_without_budget_as_adaptive() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "thinking": {
                "type": "enabled",
                "display": "summarized"
            }
        }))
        .unwrap();

        assert!(matches!(
            options.thinking,
            Some(ThinkingConfig::Enabled {
                budget_tokens: None,
                display: Some(ref display),
            }) if display == &ThinkingDisplay::Summarized
        ));

        let args = options.to_cli_args().unwrap();
        assert!(
            args.windows(2)
                .any(|window| window == ["--thinking", "adaptive"])
        );
        assert!(
            args.windows(2)
                .any(|window| window == ["--thinking-display", "summarized"])
        );
        assert!(!args.iter().any(|arg| arg == "--max-thinking-tokens"));
    }

    #[test]
    fn deprecated_zero_max_thinking_tokens_disables_thinking() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "maxThinkingTokens": 0
        }))
        .unwrap();

        let args = options.to_cli_args().unwrap();
        assert!(
            args.windows(2)
                .any(|window| window == ["--thinking", "disabled"])
        );
        assert!(!args.iter().any(|arg| arg == "--max-thinking-tokens"));
    }

    #[test]
    fn output_format_json_schema_builder_emits_cli_schema() {
        let options = ClaudeAgentOptions::builder()
            .output_format_json_schema(
                crate::tools::JsonSchema::object()
                    .required_property("company_name", crate::tools::JsonSchema::string())
                    .optional_property("founded_year", crate::tools::JsonSchema::integer())
                    .build(),
            )
            .build();
        let args = options.to_cli_args().unwrap();
        let schema = args
            .windows(2)
            .find_map(|window| (window[0] == "--json-schema").then(|| &window[1]))
            .expect("json schema arg");
        let schema: Value = serde_json::from_str(schema).unwrap();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["company_name"]["type"], "string");
        assert_eq!(schema["properties"]["founded_year"]["type"], "integer");
        assert_eq!(schema["required"], serde_json::json!(["company_name"]));
    }

    #[test]
    fn sdk_mcp_callback_requires_matching_map_config() {
        let mut options = ClaudeAgentOptions {
            mcp_servers: McpServers::PathOrJson("mcp.json".into()),
            ..Default::default()
        };
        options.callbacks.sdk_mcp_servers.insert(
            "calc".into(),
            Arc::new(crate::tools::create_sdk_mcp_server(
                "calculator",
                "1.0.0",
                vec![],
            )),
        );
        assert!(matches!(
            options.to_cli_args(),
            Err(ClaudeAgentError::InvalidOption(message))
                if message.contains("map-based mcp_servers")
        ));
    }

    #[test]
    fn rejects_upstream_option_conflicts() {
        let same_model = ClaudeAgentOptions::builder()
            .model("claude-test")
            .fallback_model("claude-test")
            .build();
        assert!(matches!(
            same_model.to_cli_args(),
            Err(ClaudeAgentError::InvalidOption(message))
                if message.contains("fallback_model cannot be the same as model")
        ));

        let session_store_with_no_persistence = ClaudeAgentOptions::builder()
            .session_store(crate::session_store::InMemorySessionStore::default())
            .persist_session(false)
            .build();
        assert!(matches!(
            session_store_with_no_persistence.to_cli_args(),
            Err(ClaudeAgentError::InvalidOption(message))
                if message.contains("session_store cannot be combined with persist_session(false)")
        ));
    }

    #[test]
    fn sandbox_settings_merge_rejects_missing_settings_file() {
        let options = ClaudeAgentOptions {
            settings: Some("/definitely/missing/settings.json".into()),
            sandbox: Some(SandboxSettings {
                enabled: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(matches!(
            options.to_cli_args(),
            Err(ClaudeAgentError::InvalidOption(message))
                if message.contains("settings file does not exist")
        ));
    }

    #[test]
    fn sandbox_settings_preserve_current_upstream_fields() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "network": {
                    "tlsTerminate": {
                        "caCertPath": "/tmp/ca.crt",
                        "caKeyPath": "/tmp/ca.key"
                    }
                },
                "filesystem": {
                    "allowWrite": ["/workspace/out"],
                    "denyWrite": ["/workspace/secret"],
                    "denyRead": ["/workspace/private"],
                    "allowRead": ["/workspace/private/public"],
                    "allowManagedReadPathsOnly": true
                },
                "ignoreViolations": {
                    "file": ["/tmp/noisy-file"],
                    "network": ["example.com"],
                    "custom": ["opaque-upstream-category"]
                },
                "enableWeakerNetworkIsolation": true,
                "ripgrep": {
                    "command": "rg-custom",
                    "args": ["--json"]
                },
                "bwrapPath": "/usr/bin/bwrap",
                "socatPath": "/usr/bin/socat"
            }
        }))
        .unwrap();

        let args = options.to_cli_args().unwrap();
        let settings = args
            .windows(2)
            .find_map(|window| (window[0] == "--settings").then(|| &window[1]))
            .expect("settings arg");
        let settings: Value = serde_json::from_str(settings).unwrap();
        let sandbox = &settings["sandbox"];
        assert_eq!(sandbox["failIfUnavailable"], true);
        assert_eq!(
            sandbox["network"]["tlsTerminate"]["caCertPath"],
            "/tmp/ca.crt"
        );
        assert_eq!(
            sandbox["filesystem"]["allowWrite"],
            serde_json::json!(["/workspace/out"])
        );
        assert_eq!(
            sandbox["filesystem"]["denyRead"],
            serde_json::json!(["/workspace/private"])
        );
        assert_eq!(sandbox["filesystem"]["allowManagedReadPathsOnly"], true);
        assert_eq!(
            sandbox["ignoreViolations"]["file"],
            serde_json::json!(["/tmp/noisy-file"])
        );
        assert_eq!(
            sandbox["ignoreViolations"]["custom"],
            serde_json::json!(["opaque-upstream-category"])
        );
        assert_eq!(sandbox["enableWeakerNetworkIsolation"], true);
        assert_eq!(sandbox["ripgrep"]["command"], "rg-custom");
        assert_eq!(sandbox["ripgrep"]["args"], serde_json::json!(["--json"]));
        assert_eq!(sandbox["bwrapPath"], "/usr/bin/bwrap");
        assert_eq!(sandbox["socatPath"], "/usr/bin/socat");
    }

    #[test]
    fn agent_definition_preserves_experimental_critical_reminder() {
        let options: ClaudeAgentOptions = serde_json::from_value(serde_json::json!({
            "agents": {
                "reviewer": {
                    "description": "reviews changes",
                    "prompt": "Review the diff.",
                    "criticalSystemReminder_EXPERIMENTAL": "Check tests first."
                }
            }
        }))
        .unwrap();

        let init = options.initialize_payload();
        assert_eq!(
            init["agents"]["reviewer"]["criticalSystemReminder_EXPERIMENTAL"],
            "Check tests first."
        );
    }
}
