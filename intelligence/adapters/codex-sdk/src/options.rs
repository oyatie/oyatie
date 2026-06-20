use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{Map, Value};

/// Global options for spawning the Codex CLI.
#[derive(Clone, Debug, Default)]
pub struct CodexOptions {
    pub(crate) codex_path_override: Option<PathBuf>,
    pub(crate) base_url: Option<String>,
    pub(crate) api_key: Option<String>,
    pub(crate) config: Option<Map<String, Value>>,
    pub(crate) env: Option<HashMap<String, String>>,
}

impl CodexOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a specific Codex executable instead of resolving `codex` from `PATH`.
    pub fn with_codex_path_override(mut self, path: impl Into<PathBuf>) -> Self {
        self.codex_path_override = Some(path.into());
        self
    }

    /// Pass `openai_base_url` as a Codex `--config` override.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Inject `CODEX_API_KEY` into the spawned CLI process.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Provide global Codex `--config` overrides as a JSON object.
    ///
    /// The object is flattened into dotted keys and rendered as TOML literals,
    /// matching the TypeScript SDK behavior.
    pub fn with_config(mut self, config: Map<String, Value>) -> Self {
        self.config = Some(config);
        self
    }

    /// Provide the exact environment for the Codex CLI.
    ///
    /// When set, the parent process environment is not inherited. The SDK still
    /// injects required values such as `CODEX_API_KEY` and the originator marker.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }
}

/// Filesystem sandbox mode forwarded to `codex exec --sandbox`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    pub(crate) fn as_cli(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

/// Model reasoning effort forwarded as `--config model_reasoning_effort=...`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

impl ModelReasoningEffort {
    pub(crate) fn as_cli(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
        }
    }
}

/// Web search mode forwarded as `--config web_search=...`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSearchMode {
    Disabled,
    Cached,
    Live,
}

impl WebSearchMode {
    pub(crate) fn as_cli(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Cached => "cached",
            Self::Live => "live",
        }
    }
}

/// Approval policy forwarded as `--config approval_policy=...`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalMode {
    Never,
    OnRequest,
    OnFailure,
    Untrusted,
}

impl ApprovalMode {
    pub(crate) fn as_cli(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::OnRequest => "on-request",
            Self::OnFailure => "on-failure",
            Self::Untrusted => "untrusted",
        }
    }
}

/// Options applied to a new or resumed Codex thread.
#[derive(Clone, Debug, Default)]
pub struct ThreadOptions {
    pub(crate) model: Option<String>,
    pub(crate) sandbox_mode: Option<SandboxMode>,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) additional_directories: Vec<PathBuf>,
    pub(crate) skip_git_repo_check: bool,
    pub(crate) model_reasoning_effort: Option<ModelReasoningEffort>,
    pub(crate) network_access_enabled: Option<bool>,
    pub(crate) web_search_mode: Option<WebSearchMode>,
    pub(crate) web_search_enabled: Option<bool>,
    pub(crate) approval_policy: Option<ApprovalMode>,
}

impl ThreadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_sandbox_mode(mut self, sandbox_mode: SandboxMode) -> Self {
        self.sandbox_mode = Some(sandbox_mode);
        self
    }

    pub fn with_working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    pub fn with_additional_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.additional_directories.push(directory.into());
        self
    }

    pub fn with_additional_directories<I, P>(mut self, directories: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.additional_directories = directories.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_skip_git_repo_check(mut self, skip_git_repo_check: bool) -> Self {
        self.skip_git_repo_check = skip_git_repo_check;
        self
    }

    pub fn with_model_reasoning_effort(mut self, effort: ModelReasoningEffort) -> Self {
        self.model_reasoning_effort = Some(effort);
        self
    }

    pub fn with_network_access_enabled(mut self, enabled: bool) -> Self {
        self.network_access_enabled = Some(enabled);
        self
    }

    pub fn with_web_search_mode(mut self, mode: WebSearchMode) -> Self {
        self.web_search_mode = Some(mode);
        self
    }

    pub fn with_web_search_enabled(mut self, enabled: bool) -> Self {
        self.web_search_enabled = Some(enabled);
        self
    }

    pub fn with_approval_policy(mut self, approval_policy: ApprovalMode) -> Self {
        self.approval_policy = Some(approval_policy);
        self
    }
}

/// Options applied to one turn.
#[derive(Clone, Debug, Default)]
pub struct TurnOptions {
    pub(crate) output_schema: Option<Value>,
}

impl TurnOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// JSON Schema object forwarded through `--output-schema`.
    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = Some(schema);
        self
    }
}
