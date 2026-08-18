//! `KubeAccess` (Talos API -> Kubernetes API access) configuration.
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers/kubeaccess`: the
//! `ConfigController` derives the kube-apiserver access policy from machine
//! config (`machine.features.kubernetesTalosAPIAccess`), and the endpoint config
//! drives which Talos API roles are allowed to be granted inside the cluster and
//! which cluster roles may be requested.
//!
//! When enabled, Talos runs an in-cluster service that lets pods request a
//! Talos API client certificate scoped to an allowed role; this module models
//! the policy decision (`authorize`) plus the endpoint config validation.

use crate::error::{ControlError, Result};
use os_kernel::Role;
use std::collections::BTreeSet;

/// The kube-apiserver kubeaccess endpoint configuration.
///
/// Mirrors `KubeAccessConfig`: whether the feature is enabled, the set of Talos
/// API roles that may be granted in-cluster, and the list of allowed Kubernetes
/// namespaces from which requests are honored.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KubeAccessConfig {
    enabled: bool,
    allowed_roles: BTreeSet<Role>,
    allowed_namespaces: BTreeSet<String>,
}

impl KubeAccessConfig {
    /// A disabled config.
    #[must_use]
    pub fn disabled() -> Self {
        KubeAccessConfig::default()
    }

    /// Build an enabled config, validating that at least one role and one
    /// namespace are allowed (otherwise the feature would grant nothing and is
    /// almost certainly a misconfiguration).
    pub fn enabled(
        allowed_roles: impl IntoIterator<Item = Role>,
        allowed_namespaces: impl IntoIterator<Item = String>,
    ) -> Result<Self> {
        let allowed_roles: BTreeSet<Role> = allowed_roles.into_iter().collect();
        let allowed_namespaces: BTreeSet<String> = allowed_namespaces
            .into_iter()
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect();
        if allowed_roles.is_empty() {
            return Err(ControlError::KubeAccess(
                "kubeaccess enabled with no allowed roles".into(),
            ));
        }
        if allowed_namespaces.is_empty() {
            return Err(ControlError::KubeAccess(
                "kubeaccess enabled with no allowed namespaces".into(),
            ));
        }
        Ok(KubeAccessConfig {
            enabled: true,
            allowed_roles,
            allowed_namespaces,
        })
    }

    /// Whether the feature is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The allowed Talos API roles.
    #[must_use]
    pub fn allowed_roles(&self) -> &BTreeSet<Role> {
        &self.allowed_roles
    }

    /// The allowed namespaces.
    #[must_use]
    pub fn allowed_namespaces(&self) -> &BTreeSet<String> {
        &self.allowed_namespaces
    }
}

/// A request from an in-cluster pod for a Talos API client certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRequest {
    /// Namespace the requesting pod runs in.
    pub namespace: String,
    /// The Talos API role being requested.
    pub role: Role,
}

/// The outcome of authorizing an [`AccessRequest`] against a [`KubeAccessConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDecision {
    /// Request granted; the embedded role's OU is what the cert will carry.
    Granted(Role),
    /// Request denied with a reason.
    Denied(String),
}

impl AccessDecision {
    /// Whether the request was granted.
    #[must_use]
    pub fn is_granted(&self) -> bool {
        matches!(self, AccessDecision::Granted(_))
    }
}

/// Authorize an access request against the policy. Mirrors the server-side check
/// the kubeaccess endpoint performs before issuing a scoped Talos API cert.
#[must_use]
pub fn authorize(config: &KubeAccessConfig, request: &AccessRequest) -> AccessDecision {
    if !config.is_enabled() {
        return AccessDecision::Denied("kubeaccess is disabled".into());
    }
    if !config.allowed_namespaces.contains(&request.namespace) {
        return AccessDecision::Denied(format!("namespace '{}' not permitted", request.namespace));
    }
    if !config.allowed_roles.contains(&request.role) {
        return AccessDecision::Denied(format!("role '{}' not permitted", request.role.as_str()));
    }
    AccessDecision::Granted(request.role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disabled() {
        let c = KubeAccessConfig::default();
        assert!(!c.is_enabled());
        assert!(c.allowed_roles().is_empty());
    }

    #[test]
    fn enabled_requires_roles_and_namespaces() {
        assert!(KubeAccessConfig::enabled(vec![], vec!["default".into()]).is_err());
        assert!(KubeAccessConfig::enabled(vec![Role::Reader], vec![]).is_err());
        let c = KubeAccessConfig::enabled(vec![Role::Reader], vec!["kube-system".into()]).unwrap();
        assert!(c.is_enabled());
    }

    #[test]
    fn enabled_filters_blank_namespaces() {
        let c = KubeAccessConfig::enabled(
            vec![Role::Reader],
            vec!["  ".into(), "default".into(), String::new()],
        )
        .unwrap();
        assert_eq!(c.allowed_namespaces().len(), 1);
        assert!(c.allowed_namespaces().contains("default"));
    }

    #[test]
    fn authorize_denied_when_disabled() {
        let c = KubeAccessConfig::disabled();
        let req = AccessRequest {
            namespace: "default".into(),
            role: Role::Reader,
        };
        let d = authorize(&c, &req);
        assert!(!d.is_granted());
        assert_eq!(d, AccessDecision::Denied("kubeaccess is disabled".into()));
    }

    #[test]
    fn authorize_checks_namespace_and_role() {
        let c = KubeAccessConfig::enabled(vec![Role::Reader], vec!["allowed-ns".into()]).unwrap();

        let wrong_ns = authorize(
            &c,
            &AccessRequest {
                namespace: "other".into(),
                role: Role::Reader,
            },
        );
        assert!(matches!(wrong_ns, AccessDecision::Denied(m) if m.contains("namespace")));

        let wrong_role = authorize(
            &c,
            &AccessRequest {
                namespace: "allowed-ns".into(),
                role: Role::Admin,
            },
        );
        assert!(matches!(wrong_role, AccessDecision::Denied(m) if m.contains("role")));
    }

    #[test]
    fn authorize_grants_allowed_request() {
        let c =
            KubeAccessConfig::enabled(vec![Role::Reader, Role::Admin], vec!["kube-system".into()])
                .unwrap();
        let d = authorize(
            &c,
            &AccessRequest {
                namespace: "kube-system".into(),
                role: Role::Reader,
            },
        );
        assert_eq!(d, AccessDecision::Granted(Role::Reader));
        assert!(d.is_granted());
    }
}
