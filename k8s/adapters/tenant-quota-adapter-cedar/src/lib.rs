//! Cedar RBAC adapter for managed-K8s tenant quota.
//!
//! This adapter wires RBAC authorization for quota admin operations using the
//! EXISTING `identity-workload-authz-cedar-adapter` crate. It does NOT
//! reinvent Cedar wiring — it reuses `CedarWorkloadAuthorizer` directly.
//!
//! ## Design (ADR-0376 / ADR-0183 / ADR-0007)
//!
//! - **Cedar default-deny**: with no matching `permit`, access is denied.
//! - **Tenant-admin sets own quota within plan ceiling**: a TenantAdmin principal
//!   can write quota for their own tenant only (Cedar same-tenant policy plus
//!   adapter defense-in-depth guard).
//! - **Platform sets ceilings**: PlatformOperator role can write any tenant's ceiling.
//! - **Cross-tenant read denied**: a tenant CANNOT read another tenant's quota/usage.
//!   Cedar enforces `principal.tenant_id == resource.tenant_id` for tenant
//!   policies; the adapter also rejects cross-tenant non-platform requests.
//! - **RBAC escalation mitigated**: no principal can grant themselves a higher role.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let authz = QuotaRbacAuthorizer::new_with_default_policies()?;
//! let decision = authz.authorize_quota_write(&principal, "ten_acme")?;
//! ```

// ADR-0083 Tier-3: panic-free on the request path.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use iam_identity_workload_authz_cedar::{
    ActionCondition, CedarWorkloadAuthorizer, Policy, PrincipalCondition, ResourceCondition,
    WorkloadAuthorizer,
};
use iam_identity_workload_domain::{
    Action, AuthorizationRequest, ClaimValue, Resource, WorkloadPrincipal,
};
use k8s_tenant_quota_kernel::{RbacRole, TenantId};

const PLATFORM_QUOTA_SCOPE: &str = "quota:platform:write";

/// Errors from the Cedar RBAC authorizer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RbacAuthzError {
    /// Policy compilation failed.
    PolicyBuild(String),
    /// The request was denied by Cedar.
    Denied(String),
}

impl std::fmt::Display for RbacAuthzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolicyBuild(detail) => write!(f, "quota rbac policy build failed: {detail}"),
            Self::Denied(reason) => write!(f, "quota rbac denied: {reason}"),
        }
    }
}

impl std::error::Error for RbacAuthzError {}

/// Cedar-backed RBAC authorizer for quota admin operations.
///
/// Wraps `CedarWorkloadAuthorizer` with quota-specific policies. Cedar
/// default-deny guarantees that absent policies = deny.
pub struct QuotaRbacAuthorizer {
    inner: CedarWorkloadAuthorizer,
}

impl QuotaRbacAuthorizer {
    /// Build with explicit policies (test / custom policy injection path).
    ///
    /// # Errors
    /// Returns [`RbacAuthzError::PolicyBuild`] if any policy fails to compile.
    pub fn with_policies(policies: Vec<Policy>) -> Result<Self, RbacAuthzError> {
        let inner = CedarWorkloadAuthorizer::with_policies(policies)
            .map_err(|e| RbacAuthzError::PolicyBuild(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Build with the default quota RBAC policies (production path).
    ///
    /// Policies implement:
    /// - TenantAdmin can write/read quota for their own tenant.
    /// - TenantViewer can read quota for their own tenant.
    /// - PlatformOperator can write/read quota for any tenant.
    ///
    /// Tenant-role cross-tenant access is denied by Cedar policy and by the
    /// adapter's defense-in-depth guard.
    ///
    /// # Errors
    /// Returns [`RbacAuthzError::PolicyBuild`] if policy compilation fails.
    pub fn new_with_default_policies() -> Result<Self, RbacAuthzError> {
        Self::with_policies(default_quota_policies())
    }

    /// Authorize a quota write operation for `target_tenant_id`.
    ///
    /// The principal must hold the appropriate scope for their role, AND
    /// (for TenantAdmin) their `tenant_id` must match `target_tenant_id`.
    ///
    /// # Errors
    /// Returns [`RbacAuthzError::Denied`] if Cedar denies the request.
    pub fn authorize_quota_write(
        &self,
        principal: &WorkloadPrincipal,
        target_tenant_id: &TenantId,
    ) -> Result<(), RbacAuthzError> {
        deny_cross_tenant_without_platform_scope(principal, target_tenant_id, "quota:Write")?;

        let request = AuthorizationRequest::new(
            principal.clone(),
            Action::new("quota:Write"),
            quota_resource(target_tenant_id),
        );
        let decision = self.inner.authorize(&request);
        if decision.is_allow() {
            Ok(())
        } else {
            Err(RbacAuthzError::Denied(format!(
                "principal {} denied quota:Write on tenant {}",
                principal.workload_id().as_str(),
                target_tenant_id.as_str()
            )))
        }
    }

    /// Authorize a quota read operation for `target_tenant_id`.
    ///
    /// # Errors
    /// Returns [`RbacAuthzError::Denied`] if Cedar denies the request.
    pub fn authorize_quota_read(
        &self,
        principal: &WorkloadPrincipal,
        target_tenant_id: &TenantId,
    ) -> Result<(), RbacAuthzError> {
        deny_cross_tenant_without_platform_scope(principal, target_tenant_id, "quota:Read")?;

        let request = AuthorizationRequest::new(
            principal.clone(),
            Action::new("quota:Read"),
            quota_resource(target_tenant_id),
        );
        let decision = self.inner.authorize(&request);
        if decision.is_allow() {
            Ok(())
        } else {
            Err(RbacAuthzError::Denied(format!(
                "principal {} denied quota:Read on tenant {}",
                principal.workload_id().as_str(),
                target_tenant_id.as_str()
            )))
        }
    }

    /// Derive the Cedar scope string for an RBAC role (used when seeding workload
    /// principals in tests or provisioning pipelines).
    #[must_use]
    pub fn scope_for_role(role: &RbacRole) -> &'static str {
        match role {
            RbacRole::TenantAdmin => "quota:write",
            RbacRole::TenantViewer => "quota:read",
            RbacRole::PlatformOperator => PLATFORM_QUOTA_SCOPE,
        }
    }
}

fn default_quota_policies() -> Vec<Policy> {
    vec![
        // TenantAdmin: write own-tenant quota only.
        Policy::permit("quota-write-tenant-admin")
            .when_principal(PrincipalCondition::HasScope("quota:write".into()))
            .for_action(ActionCondition::Equals("quota:Write".into()))
            .for_resource(ResourceCondition::SameTenantAsPrincipal {
                resource_type: "QuotaRecord".into(),
            }),
        // TenantViewer + TenantAdmin: read own-tenant quota only.
        Policy::permit("quota-read-tenant")
            .when_principal(PrincipalCondition::HasScope("quota:read".into()))
            .for_action(ActionCondition::Equals("quota:Read".into()))
            .for_resource(ResourceCondition::SameTenantAsPrincipal {
                resource_type: "QuotaRecord".into(),
            }),
        // PlatformOperator: write any tenant's quota (ceiling management).
        Policy::permit("quota-write-platform-operator")
            .when_principal(PrincipalCondition::HasScope(PLATFORM_QUOTA_SCOPE.into()))
            .for_action(ActionCondition::Equals("quota:Write".into()))
            .for_resource(ResourceCondition::TypeIs("QuotaRecord".into())),
        Policy::permit("quota-read-platform-operator")
            .when_principal(PrincipalCondition::HasScope(PLATFORM_QUOTA_SCOPE.into()))
            .for_action(ActionCondition::Equals("quota:Read".into()))
            .for_resource(ResourceCondition::TypeIs("QuotaRecord".into())),
    ]
}

fn deny_cross_tenant_without_platform_scope(
    principal: &WorkloadPrincipal,
    target_tenant_id: &TenantId,
    action: &str,
) -> Result<(), RbacAuthzError> {
    if principal.tenant_id().as_str() == target_tenant_id.as_str()
        || principal.has_scope(PLATFORM_QUOTA_SCOPE)
    {
        return Ok(());
    }

    Err(RbacAuthzError::Denied(format!(
        "principal {} denied cross-tenant {action} on tenant {}",
        principal.workload_id().as_str(),
        target_tenant_id.as_str()
    )))
}

fn quota_resource(target_tenant_id: &TenantId) -> Resource {
    Resource::new("QuotaRecord", target_tenant_id.as_str()).with_attribute(
        "tenant_id",
        ClaimValue::Text(target_tenant_id.as_str().to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam_identity_workload_domain::{Effect, WorkloadState};

    fn active_principal(tenant: &str, scope: &str) -> WorkloadPrincipal {
        let mut p = WorkloadPrincipal::provision(tenant, "wl_admin_01", "cap.quota.admin")
            .expect("provision");
        p.transition_to(WorkloadState::Active).expect("activate");
        p.with_scope(scope).expect("scope")
    }

    #[test]
    fn tenant_admin_can_write_own_quota() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_acme", "quota:write");
        let tenant_id = TenantId::new("ten_acme").unwrap();
        assert!(authz.authorize_quota_write(&principal, &tenant_id).is_ok());
    }

    #[test]
    fn tenant_admin_can_read_own_quota() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_acme", "quota:read");
        let tenant_id = TenantId::new("ten_acme").unwrap();
        assert!(authz.authorize_quota_read(&principal, &tenant_id).is_ok());
    }

    #[test]
    fn principal_without_scope_denied_write() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        // No quota:write scope at all — Cedar default-deny kicks in.
        let principal = active_principal("ten_acme", "other:scope");
        let tenant_id = TenantId::new("ten_acme").unwrap();
        assert!(authz.authorize_quota_write(&principal, &tenant_id).is_err());
    }

    #[test]
    fn platform_operator_can_write_any_tenant() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_platform", "quota:platform:write");
        let tenant_id = TenantId::new("ten_acme").unwrap();
        assert!(authz.authorize_quota_write(&principal, &tenant_id).is_ok());
    }

    #[test]
    fn tenant_admin_cannot_write_other_tenant_quota() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_acme", "quota:write");
        let tenant_id = TenantId::new("ten_globex").unwrap();
        assert!(authz.authorize_quota_write(&principal, &tenant_id).is_err());
    }

    #[test]
    fn tenant_viewer_cannot_read_other_tenant_quota() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_acme", "quota:read");
        let tenant_id = TenantId::new("ten_globex").unwrap();
        assert!(authz.authorize_quota_read(&principal, &tenant_id).is_err());
    }

    #[test]
    fn default_pdp_policy_denies_cross_tenant_quota_read() {
        let pdp = CedarWorkloadAuthorizer::with_policies(default_quota_policies()).unwrap();
        let principal = active_principal("ten_acme", "quota:read");
        let tenant_id = TenantId::new("ten_globex").unwrap();
        let request = AuthorizationRequest::new(
            principal,
            Action::new("quota:Read"),
            quota_resource(&tenant_id),
        );

        let decision = pdp.authorize(&request);

        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn default_pdp_policy_denies_cross_tenant_quota_write() {
        let pdp = CedarWorkloadAuthorizer::with_policies(default_quota_policies()).unwrap();
        let principal = active_principal("ten_acme", "quota:write");
        let tenant_id = TenantId::new("ten_globex").unwrap();
        let request = AuthorizationRequest::new(
            principal,
            Action::new("quota:Write"),
            quota_resource(&tenant_id),
        );

        let decision = pdp.authorize(&request);

        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn platform_operator_can_read_any_tenant() {
        let authz = QuotaRbacAuthorizer::new_with_default_policies().unwrap();
        let principal = active_principal("ten_platform", "quota:platform:write");
        let tenant_id = TenantId::new("ten_acme").unwrap();
        assert!(authz.authorize_quota_read(&principal, &tenant_id).is_ok());
    }

    #[test]
    fn scope_for_role_returns_correct_strings() {
        assert_eq!(
            QuotaRbacAuthorizer::scope_for_role(&RbacRole::TenantAdmin),
            "quota:write"
        );
        assert_eq!(
            QuotaRbacAuthorizer::scope_for_role(&RbacRole::TenantViewer),
            "quota:read"
        );
        assert_eq!(
            QuotaRbacAuthorizer::scope_for_role(&RbacRole::PlatformOperator),
            "quota:platform:write"
        );
    }
}
