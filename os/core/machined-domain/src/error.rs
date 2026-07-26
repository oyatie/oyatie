//! Crate-local error type for `talos-machined`.

use std::fmt;
use os_kernel::Error as CoreError;

/// Result alias used throughout this crate.
pub type Result<T> = std::result::Result<T, MachinedError>;

/// Errors raised while sequencing the machine or supervising services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachinedError {
    /// A sequence was requested that is not valid in the current runtime mode
    /// or machine state (e.g. `Upgrade` while still booting).
    SequenceNotAllowed(String),
    /// A task inside a phase failed; the sequence is aborted.
    TaskFailed { task: String, reason: String },
    /// A service could not be started, or transitioned illegally.
    ServiceError { service: String, reason: String },
    /// A referenced service or controller was not registered.
    NotFound(String),
    /// An illegal machine/runtime/service state transition was attempted.
    IllegalTransition { from: String, to: String },
    /// A controller dependency could not be satisfied.
    DependencyUnmet(String),
    /// An error bubbled up from `talos-core`.
    Core(CoreError),
}

impl MachinedError {
    /// Construct a [`MachinedError::SequenceNotAllowed`].
    pub fn sequence_not_allowed(msg: impl Into<String>) -> Self {
        MachinedError::SequenceNotAllowed(msg.into())
    }

    /// Construct a [`MachinedError::TaskFailed`].
    pub fn task_failed(task: impl Into<String>, reason: impl Into<String>) -> Self {
        MachinedError::TaskFailed {
            task: task.into(),
            reason: reason.into(),
        }
    }

    /// Construct a [`MachinedError::ServiceError`].
    pub fn service_error(service: impl Into<String>, reason: impl Into<String>) -> Self {
        MachinedError::ServiceError {
            service: service.into(),
            reason: reason.into(),
        }
    }

    /// Construct a [`MachinedError::NotFound`].
    pub fn not_found(msg: impl Into<String>) -> Self {
        MachinedError::NotFound(msg.into())
    }

    /// Construct a [`MachinedError::IllegalTransition`].
    pub fn illegal_transition(from: impl Into<String>, to: impl Into<String>) -> Self {
        MachinedError::IllegalTransition {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Short stable kind string useful for matching/logging.
    pub fn kind(&self) -> &'static str {
        match self {
            MachinedError::SequenceNotAllowed(_) => "sequence_not_allowed",
            MachinedError::TaskFailed { .. } => "task_failed",
            MachinedError::ServiceError { .. } => "service_error",
            MachinedError::NotFound(_) => "not_found",
            MachinedError::IllegalTransition { .. } => "illegal_transition",
            MachinedError::DependencyUnmet(_) => "dependency_unmet",
            MachinedError::Core(_) => "core",
        }
    }
}

impl From<CoreError> for MachinedError {
    fn from(e: CoreError) -> Self {
        MachinedError::Core(e)
    }
}

impl fmt::Display for MachinedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MachinedError::SequenceNotAllowed(m) => write!(f, "sequence not allowed: {m}"),
            MachinedError::TaskFailed { task, reason } => {
                write!(f, "task '{task}' failed: {reason}")
            }
            MachinedError::ServiceError { service, reason } => {
                write!(f, "service '{service}' error: {reason}")
            }
            MachinedError::NotFound(m) => write!(f, "not found: {m}"),
            MachinedError::IllegalTransition { from, to } => {
                write!(f, "illegal transition from '{from}' to '{to}'")
            }
            MachinedError::DependencyUnmet(m) => write!(f, "dependency unmet: {m}"),
            MachinedError::Core(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for MachinedError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_stable() {
        assert_eq!(MachinedError::task_failed("a", "b").kind(), "task_failed");
        assert_eq!(MachinedError::not_found("x").kind(), "not_found");
        assert_eq!(
            MachinedError::illegal_transition("a", "b").kind(),
            "illegal_transition"
        );
    }

    #[test]
    fn core_error_converts() {
        let e: MachinedError = CoreError::invalid("bad").into();
        assert_eq!(e.kind(), "core");
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn display_formats_task() {
        let e = MachinedError::task_failed("mountRoot", "EBUSY");
        assert_eq!(e.to_string(), "task 'mountRoot' failed: EBUSY");
    }
}
