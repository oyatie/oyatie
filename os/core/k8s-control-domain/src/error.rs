//! Crate-local error type for the Kubernetes control-plane controllers.
//!
//! Mirrors the operating-system pattern: a rich, domain-specific enum that converts
//! cleanly into the workspace-wide [`os_kernel::Error`] at the crate boundary.

use std::fmt;

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, ControlError>;

/// Errors produced while configuring the Kubernetes control plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlError {
    /// A configuration field failed validation.
    InvalidConfig(String),
    /// A required secret / certificate / key was missing.
    MissingSecret(String),
    /// A static pod or manifest could not be rendered.
    Render(String),
    /// An admission / audit / encryption policy was malformed.
    Policy(String),
    /// A manifest was duplicated, or referenced an unknown manifest.
    Manifest(String),
    /// A controller reconcile was attempted from an invalid state.
    Reconcile(String),
    /// A kubeaccess request was denied or malformed.
    KubeAccess(String),
}

impl ControlError {
    /// Short, stable kind string for matching and logging.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            ControlError::InvalidConfig(_) => "invalid_config",
            ControlError::MissingSecret(_) => "missing_secret",
            ControlError::Render(_) => "render",
            ControlError::Policy(_) => "policy",
            ControlError::Manifest(_) => "manifest",
            ControlError::Reconcile(_) => "reconcile",
            ControlError::KubeAccess(_) => "kubeaccess",
        }
    }
}

impl fmt::Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControlError::InvalidConfig(m) => write!(f, "invalid control-plane config: {m}"),
            ControlError::MissingSecret(m) => write!(f, "missing secret: {m}"),
            ControlError::Render(m) => write!(f, "render error: {m}"),
            ControlError::Policy(m) => write!(f, "policy error: {m}"),
            ControlError::Manifest(m) => write!(f, "manifest error: {m}"),
            ControlError::Reconcile(m) => write!(f, "reconcile error: {m}"),
            ControlError::KubeAccess(m) => write!(f, "kubeaccess error: {m}"),
        }
    }
}

impl std::error::Error for ControlError {}

impl From<ControlError> for os_kernel::Error {
    fn from(e: ControlError) -> Self {
        match e {
            ControlError::InvalidConfig(m) | ControlError::Policy(m) => {
                os_kernel::Error::Invalid(m)
            }
            ControlError::MissingSecret(m) => os_kernel::Error::NotFound(m),
            ControlError::Render(m) | ControlError::Manifest(m) => os_kernel::Error::Other(m),
            ControlError::Reconcile(m) => os_kernel::Error::InvalidState(m),
            ControlError::KubeAccess(m) => os_kernel::Error::PermissionDenied(m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_stable() {
        assert_eq!(
            ControlError::InvalidConfig("x".into()).kind(),
            "invalid_config"
        );
        assert_eq!(ControlError::KubeAccess("x".into()).kind(), "kubeaccess");
        assert_eq!(ControlError::Policy("x".into()).kind(), "policy");
    }

    #[test]
    fn converts_into_core_error() {
        let core: os_kernel::Error = ControlError::Reconcile("bad".into()).into();
        assert_eq!(core.kind(), "invalid_state");
        let core: os_kernel::Error = ControlError::MissingSecret("ca".into()).into();
        assert_eq!(core.kind(), "not_found");
        let core: os_kernel::Error = ControlError::KubeAccess("denied".into()).into();
        assert_eq!(core.kind(), "permission_denied");
    }

    #[test]
    fn display_renders_message() {
        assert_eq!(
            ControlError::MissingSecret("apiserver-key".into()).to_string(),
            "missing secret: apiserver-key"
        );
    }
}
