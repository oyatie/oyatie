//! Crate-local error type for the Kubernetes subsystem.
//!
//! Mirrors the pattern used across operating-system: a rich, domain-specific enum that
//! converts cleanly into the workspace-wide [`os_kernel::Error`] at the crate
//! boundary.

use std::fmt;

/// Result alias for the Kubernetes subsystem.
pub type Result<T> = std::result::Result<T, K8sError>;

/// Errors produced while configuring or bootstrapping Kubernetes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum K8sError {
    /// A configuration field failed validation.
    InvalidConfig(String),
    /// A node name was malformed.
    InvalidNodeName(String),
    /// A required secret or certificate was missing.
    MissingSecret(String),
    /// A static pod / manifest could not be rendered.
    Render(String),
    /// A bootstrap manifest was duplicated or out of order.
    Bootstrap(String),
    /// An etcd member operation was attempted from an invalid state.
    EtcdState(String),
    /// A requested component is not a control-plane component.
    UnknownComponent(String),
    /// The control plane has not been bootstrapped yet.
    NotBootstrapped,
}

impl K8sError {
    /// Short, stable kind string for matching and logging.
    pub fn kind(&self) -> &'static str {
        match self {
            K8sError::InvalidConfig(_) => "invalid_config",
            K8sError::InvalidNodeName(_) => "invalid_node_name",
            K8sError::MissingSecret(_) => "missing_secret",
            K8sError::Render(_) => "render",
            K8sError::Bootstrap(_) => "bootstrap",
            K8sError::EtcdState(_) => "etcd_state",
            K8sError::UnknownComponent(_) => "unknown_component",
            K8sError::NotBootstrapped => "not_bootstrapped",
        }
    }
}

impl fmt::Display for K8sError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            K8sError::InvalidConfig(m) => write!(f, "invalid kubernetes config: {m}"),
            K8sError::InvalidNodeName(m) => write!(f, "invalid node name: {m}"),
            K8sError::MissingSecret(m) => write!(f, "missing secret: {m}"),
            K8sError::Render(m) => write!(f, "render error: {m}"),
            K8sError::Bootstrap(m) => write!(f, "bootstrap error: {m}"),
            K8sError::EtcdState(m) => write!(f, "etcd state error: {m}"),
            K8sError::UnknownComponent(m) => write!(f, "unknown component: {m}"),
            K8sError::NotBootstrapped => write!(f, "control plane not bootstrapped"),
        }
    }
}

impl std::error::Error for K8sError {}

impl From<K8sError> for os_kernel::Error {
    fn from(e: K8sError) -> Self {
        match e {
            K8sError::InvalidConfig(m)
            | K8sError::InvalidNodeName(m)
            | K8sError::MissingSecret(m) => os_kernel::Error::Invalid(m),
            K8sError::Render(m) | K8sError::Bootstrap(m) => os_kernel::Error::Other(m),
            K8sError::EtcdState(m) => os_kernel::Error::InvalidState(m),
            K8sError::UnknownComponent(m) => os_kernel::Error::NotFound(m),
            K8sError::NotBootstrapped => {
                os_kernel::Error::InvalidState("control plane not bootstrapped".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_stable() {
        assert_eq!(K8sError::InvalidConfig("x".into()).kind(), "invalid_config");
        assert_eq!(K8sError::NotBootstrapped.kind(), "not_bootstrapped");
    }

    #[test]
    fn converts_into_core_error() {
        let core: os_kernel::Error = K8sError::EtcdState("not joined".into()).into();
        assert_eq!(core.kind(), "invalid_state");
        let core: os_kernel::Error = K8sError::UnknownComponent("foo".into()).into();
        assert_eq!(core.kind(), "not_found");
    }

    #[test]
    fn display_renders_message() {
        assert_eq!(
            K8sError::MissingSecret("ca".into()).to_string(),
            "missing secret: ca"
        );
    }
}
