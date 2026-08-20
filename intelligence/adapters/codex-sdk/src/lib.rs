//! Rust SDK for embedding Codex in Rust applications.
//!
//! This crate mirrors the existing TypeScript SDK's `codex exec --experimental-json`
//! transport: it spawns the Codex CLI, writes the prompt to stdin, and reads JSONL
//! events from stdout.
//!
//! Source contract:
//! - TypeScript SDK README: <https://github.com/openai/codex/tree/main/sdk/typescript>
//! - TypeScript CLI transport: <https://github.com/openai/codex/blob/main/sdk/typescript/src/exec.ts>
//! - TypeScript event/item shapes: <https://github.com/openai/codex/tree/main/sdk/typescript/src>
//! - Python SDK public API: <https://github.com/openai/codex/blob/main/sdk/python/docs/api-reference.md>
//! - Python generated app-server contract: <https://github.com/openai/codex/blob/main/sdk/python/src/openai_codex/generated/v2_all.py>
//! - Python SDK runtime packaging note: <https://github.com/openai/codex/tree/main/sdk/python-runtime>

/// SDK version reported to the app-server. A literal (not `env!("CARGO_PKG_VERSION")`)
/// because hermetic buck2 builds do not inject cargo env vars into first-party
/// targets; tracks the workspace version.
pub(crate) const SDK_VERSION: &str = "0.1.0";

mod app_server;
#[cfg(feature = "async")]
mod async_app_server;
mod codex;
mod error;
mod events;
mod exec;
mod input;
mod items;
mod options;
mod protocol_schema;
mod schema;
mod thread;

pub use app_server::{
    AppCodex, AppInput, AppLoginHandle, AppRunInput, AppServerClient, AppServerConfig, AppThread,
    AppTurnHandle, AppTurnResult, AppTurnStream, CURRENT_APP_SERVER_REQUEST_METHODS,
    CURRENT_UPSTREAM_MAIN_SHA, InitializeResponse, JsonObject, Notification, ServerInfo,
};
#[cfg(feature = "async")]
pub use async_app_server::{
    AsyncAppCodex, AsyncAppLoginHandle, AsyncAppThread, AsyncAppTurnHandle, AsyncAppTurnStream,
};
pub use codex::Codex;
pub use error::{CodexError, Result};
pub use events::{
    ItemCompletedEvent, ItemStartedEvent, ItemUpdatedEvent, ThreadError, ThreadEvent,
    ThreadStartedEvent, TurnCompletedEvent, TurnFailedEvent, TurnStartedEvent, Usage,
};
pub use input::{Input, UserInput};
pub use items::{
    AgentMessageItem, CommandExecutionItem, CommandExecutionStatus, ErrorItem, FileChangeItem,
    FileUpdateChange, McpToolCallItem, McpToolCallResult, McpToolCallStatus, PatchApplyStatus,
    PatchChangeKind, ReasoningItem, ThreadItem, TodoItem, TodoListItem, WebSearchItem,
};
pub use options::{
    ApprovalMode, CodexOptions, ModelReasoningEffort, SandboxMode, ThreadOptions, TurnOptions,
    WebSearchMode,
};
pub use protocol_schema::{
    APP_SERVER_PROTOCOL_SCHEMA_JSON, AppServerProtocolSchemaSummary,
    app_server_protocol_definition, app_server_protocol_definition_names,
    app_server_protocol_schema_json, app_server_protocol_schema_summary,
};
pub use thread::{EventStream, RunResult, RunStreamedResult, StreamedTurn, Thread, Turn};
