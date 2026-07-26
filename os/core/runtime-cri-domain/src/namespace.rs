//! containerd namespaces.
//!
//! containerd isolates metadata and content by namespace. Talos uses two
//! well-known namespaces: `system` for system-critical containers managed by
//! machined (etcd, the kubelet's static-pod bootstrap helpers, ...) and `k8s.io`
//! for CRI-managed Kubernetes workloads.

use os_kernel::error::{Error, Result};

/// A containerd namespace name.
///
/// Names must be non-empty, <= 63 chars and consist of lowercase
/// alphanumerics, `.`, `-` and `_`, mirroring containerd's
/// `namespaces.Validate`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace(String);

impl Namespace {
    /// The Talos system namespace used by machined for system services.
    pub const SYSTEM: &'static str = "system";
    /// The CRI namespace used by the Kubernetes integration.
    pub const K8S: &'static str = "k8s.io";

    /// Validate and construct a namespace.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::invalid("namespace must not be empty"));
        }
        if name.len() > 63 {
            return Err(Error::invalid("namespace too long (max 63)"));
        }
        let ok = name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'));
        if !ok {
            return Err(Error::invalid("namespace has invalid characters"));
        }
        Ok(Namespace(name))
    }

    /// The Talos system namespace.
    pub fn system() -> Self {
        Namespace(Self::SYSTEM.to_string())
    }

    /// The Kubernetes CRI namespace.
    pub fn k8s() -> Self {
        Namespace(Self::K8S.to_string())
    }

    /// The raw namespace string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this is the system namespace.
    pub fn is_system(&self) -> bool {
        self.0 == Self::SYSTEM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_namespaces() {
        assert!(Namespace::system().is_system());
        assert_eq!(Namespace::k8s().as_str(), "k8s.io");
        assert!(!Namespace::k8s().is_system());
    }

    #[test]
    fn validation_rejects_bad_names() {
        assert!(Namespace::new("").is_err());
        assert!(Namespace::new("Bad").is_err());
        assert!(Namespace::new("has space").is_err());
        assert!(Namespace::new("a".repeat(64)).is_err());
        assert!(Namespace::new("k8s.io").is_ok());
        assert!(Namespace::new("my_ns-1").is_ok());
    }
}
