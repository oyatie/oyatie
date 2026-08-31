use std::fmt;
use std::io;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityError {
    field: &'static str,
    reason: String,
}

impl IdentityError {
    pub(crate) fn new(field: &'static str, reason: impl Into<String>) -> Self {
        Self {
            field,
            reason: reason.into(),
        }
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid {}: {}", self.field, self.reason)
    }
}

impl std::error::Error for IdentityError {}

#[derive(Debug)]
pub enum SnapshotFailure {
    InvalidInput(IdentityError),
    InvalidProfile(String),
    AmbiguousMergeBase {
        count: usize,
    },
    MissingMergeBase,
    ObjectMismatch(String),
    ObjectCollision(String),
    MalformedOutput(String),
    DuplicatePath(Vec<u8>),
    MissingContent(String),
    UnexpectedContent(String),
    LimitExceeded {
        limit: &'static str,
        maximum: u64,
        observed: u64,
    },
    Cancelled,
    DeadlineExceeded,
    ToolUnavailable(String),
    ToolFailed {
        operation: &'static str,
        status: Option<i32>,
        stderr: String,
    },
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl SnapshotFailure {
    pub fn io(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }
}

impl From<IdentityError> for SnapshotFailure {
    fn from(value: IdentityError) -> Self {
        Self::InvalidInput(value)
    }
}

impl fmt::Display for SnapshotFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(error) => error.fmt(formatter),
            Self::InvalidProfile(reason) => write!(formatter, "invalid snapshot profile: {reason}"),
            Self::AmbiguousMergeBase { count } => {
                write!(
                    formatter,
                    "repository has {count} merge bases; exactly one is required"
                )
            }
            Self::MissingMergeBase => formatter.write_str("repository has no merge base"),
            Self::ObjectMismatch(reason) => {
                write!(formatter, "repository object mismatch: {reason}")
            }
            Self::ObjectCollision(identity) => {
                write!(
                    formatter,
                    "repository object collision detected for {identity}"
                )
            }
            Self::MalformedOutput(reason) => {
                write!(formatter, "malformed repository output: {reason}")
            }
            Self::DuplicatePath(path) => {
                write!(formatter, "repository manifest repeats path {:?}", path)
            }
            Self::MissingContent(identity) => {
                write!(
                    formatter,
                    "selected repository content {identity} is missing"
                )
            }
            Self::UnexpectedContent(identity) => {
                write!(
                    formatter,
                    "repository returned unselected content {identity}"
                )
            }
            Self::LimitExceeded {
                limit,
                maximum,
                observed,
            } => write!(
                formatter,
                "repository snapshot exceeded {limit}: maximum {maximum}, observed {observed}"
            ),
            Self::Cancelled => formatter.write_str("repository snapshot was cancelled"),
            Self::DeadlineExceeded => {
                formatter.write_str("repository snapshot deadline was exceeded")
            }
            Self::ToolUnavailable(reason) => {
                write!(formatter, "repository tool unavailable: {reason}")
            }
            Self::ToolFailed {
                operation,
                status,
                stderr,
            } => write!(
                formatter,
                "repository tool failed during {operation} with status {status:?}: {stderr}"
            ),
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
        }
    }
}

impl std::error::Error for SnapshotFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub(crate) fn enforce_limit(
    limit: &'static str,
    maximum: u64,
    observed: u64,
) -> Result<(), SnapshotFailure> {
    if observed > maximum {
        Err(SnapshotFailure::LimitExceeded {
            limit,
            maximum,
            observed,
        })
    } else {
        Ok(())
    }
}
