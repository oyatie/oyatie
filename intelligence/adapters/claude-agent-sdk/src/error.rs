use std::{io, path::PathBuf};

use thiserror::Error;

/// Crate-local result type.
pub type Result<T, E = ClaudeAgentError> = std::result::Result<T, E>;

/// Package-exported abort error type.
///
/// The current TypeScript SDK declares `AbortError extends Error` and uses it
/// for aborted direct-connect operations. Rust APIs can use this lightweight
/// public type for the same cancellation/error boundary without pulling it into
/// the crate-wide [`ClaudeAgentError`] enum.
#[derive(Debug, Clone, Default, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct AbortError {
    message: String,
}

impl AbortError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Errors surfaced by the Rust Claude Agent SDK.
#[derive(Debug, Error)]
pub enum ClaudeAgentError {
    #[error(
        "Claude Code executable not found. Install Claude Code or set ClaudeAgentOptions::cli_path"
    )]
    CliNotFound,

    #[error("Claude Code executable not found at {path}")]
    CliNotFoundAt { path: PathBuf },

    #[error("working directory does not exist: {path}")]
    WorkingDirectoryNotFound { path: PathBuf },

    #[error("failed to start Claude Code: {0}")]
    Connection(String),

    #[error("Claude Code process failed{exit_code:?}: {message}")]
    Process {
        exit_code: Option<i32>,
        message: String,
    },

    #[error("failed to decode CLI JSON line: {source}; line={line}")]
    JsonDecode {
        #[source]
        source: serde_json::Error,
        line: String,
    },

    #[error("failed to parse SDK message: {message}; data={data}")]
    MessageParse {
        message: String,
        data: serde_json::Value,
    },

    #[error("control request timed out: {0}")]
    ControlTimeout(String),

    #[error("control request failed: {0}")]
    Control(String),

    #[error("invalid option: {0}")]
    InvalidOption(String),

    #[error("invalid tool arguments: {0}")]
    ToolArguments(String),

    #[error("session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl ClaudeAgentError {
    pub(crate) fn message_parse(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self::MessageParse {
            message: message.into(),
            data,
        }
    }
}
