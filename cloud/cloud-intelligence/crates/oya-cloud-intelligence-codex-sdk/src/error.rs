use std::fmt;

/// SDK-wide result type.
pub type Result<T> = std::result::Result<T, CodexError>;

/// Errors surfaced by the Rust Codex SDK.
#[derive(Debug)]
pub enum CodexError {
    /// A local I/O operation failed.
    Io(std::io::Error),
    /// A JSONL event or JSON schema could not be serialized/deserialized.
    Json(serde_json::Error),
    /// The Codex process exited unsuccessfully.
    CliExit { code: Option<i32>, stderr: String },
    /// A required pipe was unavailable on the spawned process.
    MissingPipe(&'static str),
    /// The CLI emitted a `turn.failed` event.
    TurnFailed { message: String },
    /// The SDK was given an unsupported config override.
    InvalidConfig(String),
    /// The app-server transport closed before a response was received.
    TransportClosed,
    /// The app-server returned a JSON-RPC error response.
    Rpc {
        code: i64,
        message: String,
        data: Option<serde_json::Value>,
    },
    /// The app-server emitted an invalid protocol message.
    Protocol(String),
    /// `TurnOptions::output_schema` must be a JSON object.
    InvalidOutputSchema,
}

impl fmt::Display for CodexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::CliExit { code, stderr } => {
                let detail =
                    code.map_or_else(|| "signal".to_string(), |code| format!("code {code}"));
                if stderr.trim().is_empty() {
                    write!(f, "Codex exec exited with {detail}")
                } else {
                    write!(f, "Codex exec exited with {detail}: {stderr}")
                }
            }
            Self::MissingPipe(pipe) => write!(f, "child process has no {pipe}"),
            Self::TurnFailed { message } => write!(f, "Codex turn failed: {message}"),
            Self::InvalidConfig(message) => write!(f, "invalid Codex config override: {message}"),
            Self::TransportClosed => write!(f, "app-server transport closed"),
            Self::Rpc {
                code,
                message,
                data,
            } => {
                if let Some(data) = data {
                    write!(f, "app-server JSON-RPC error {code}: {message}: {data}")
                } else {
                    write!(f, "app-server JSON-RPC error {code}: {message}")
                }
            }
            Self::Protocol(message) => write!(f, "app-server protocol error: {message}"),
            Self::InvalidOutputSchema => write!(f, "output_schema must be a JSON object"),
        }
    }
}

impl std::error::Error for CodexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CodexError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for CodexError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
