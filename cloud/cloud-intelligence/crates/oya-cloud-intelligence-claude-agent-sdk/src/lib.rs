//! Rust implementation of the Claude Agent SDK interface.
//!
//! This crate follows the official Agent SDK docs and upstream Python/TypeScript
//! subprocess protocol: `claude --output-format stream-json --input-format stream-json`,
//! an SDK `initialize` control request, JSONL user input, and streamed typed messages.

/// SDK version reported to the CLI. A literal (not `env!("CARGO_PKG_VERSION")`)
/// because hermetic buck2 builds do not inject cargo env vars into first-party
/// targets; tracks the workspace version.
pub(crate) const SDK_VERSION: &str = "0.1.0";

mod assistant;
mod bridge;
mod callbacks;
mod client;
mod direct_connect;
mod error;
mod messages;
mod options;
mod query;
mod runtime;
mod session_store;
mod sessions;
mod settings;
mod status;
mod tools;
mod transport;

pub use assistant::*;
pub use bridge::*;
pub use callbacks::*;
pub use client::ClaudeSDKClient;
pub use direct_connect::*;
pub use error::{AbortError, ClaudeAgentError, Result};
pub use messages::*;
pub use options::*;
pub use query::{Query, WarmQuery, query, query_stream, startup, startup_with_timeout};
pub use session_store::*;
pub use sessions::*;
pub use settings::*;
pub use status::*;
pub use tools::*;
pub use transport::{
    ClaudeProcessSpawner, ProcessAbortSignal, ProcessSpawnOptions, ProcessSpawnerFuture,
    ProcessWaitFuture, SharedClaudeProcessSpawner, SpawnedClaudeProcess,
};

/// Marker used by system-prompt arrays to separate cacheable static content
/// from dynamic per-session content.
pub const SYSTEM_PROMPT_DYNAMIC_BOUNDARY: &str = "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__";

/// Exit reasons currently exported by the TypeScript SDK for stop/session hooks.
pub const EXIT_REASONS: &[&str] = &[
    "clear",
    "resume",
    "logout",
    "prompt_input_exit",
    "other",
    "bypass_permissions_disabled",
];

/// Hook event names currently exported by the TypeScript SDK.
pub const HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PostToolBatch",
    "Notification",
    "UserPromptSubmit",
    "UserPromptExpansion",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PostCompact",
    "PermissionRequest",
    "PermissionDenied",
    "Setup",
    "TeammateIdle",
    "TaskCreated",
    "TaskCompleted",
    "Elicitation",
    "ElicitationResult",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
    "InstructionsLoaded",
    "CwdChanged",
    "FileChanged",
    "MessageDisplay",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_package_exported_constants() {
        assert_eq!(
            SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
            "__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__"
        );
        assert_eq!(
            EXIT_REASONS,
            &[
                "clear",
                "resume",
                "logout",
                "prompt_input_exit",
                "other",
                "bypass_permissions_disabled"
            ]
        );
        assert!(HOOK_EVENTS.contains(&"PreToolUse"));
        assert!(HOOK_EVENTS.contains(&"MessageDisplay"));
        assert_eq!(HOOK_EVENTS.len(), 30);
    }
}
