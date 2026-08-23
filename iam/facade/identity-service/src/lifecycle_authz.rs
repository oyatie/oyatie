//! Lifecycle-authorization adapter for the principal-lifecycle control plane.
//!
//! ADR-0581 / AUTH-005: the `:suspend`/`:retire` routes are a mutating control
//! plane and must be gated by a fail-closed PDP decision bound to the TARGET
//! principal's real tenant. [`iam_identity_workload_rest::LifecycleAuthorizer`]
//! is the PORT (owned by the boundary crate); this is the concrete ADAPTER,
//! placed in the composition root (the service crate) per the ports-and-adapters
//! layering — the boundary crate never depends on a concrete PDP.
//!
//! This adapter enforces the tenant-isolation invariant directly: a verified
//! caller may suspend/retire a principal ONLY within its own tenant. A
//! cross-tenant request (caller in tenant A acting on tenant B's principal) is
//! DENIED regardless of action — closing the IDOR / blast-radius axis. It is the
//! W5-shaped default; a richer Cedar-policy-backed iam PDP swaps in behind
//! the same port without touching the REST surface.
//!
//! ADR-0083 Tier 3: panic-free; `#![forbid(unsafe_code)]` inherited at crate root.

use iam_identity_workload_rest::{AuthzFault, LifecycleAuthorizer, LifecycleAuthzRequest};

/// Tenant-scoped lifecycle authorizer: PERMIT iff the verified caller's tenant
/// equals the target principal's tenant. Cross-tenant suspend/retire is denied.
///
/// This is deliberately conservative (same-tenant only). A platform-admin
/// capability or a Cedar-policy-backed decision is a future adapter swap behind
/// the same [`LifecycleAuthorizer`] port; the REST boundary is unaffected.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TenantScopedLifecycleAuthorizer;

impl TenantScopedLifecycleAuthorizer {
    /// Construct the authorizer.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl LifecycleAuthorizer for TenantScopedLifecycleAuthorizer {
    fn decide(&self, request: &LifecycleAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        // Defensive: an empty caller/target tenant is never a valid permit basis
        // (fail-closed) — a misconfigured credential cannot grant cross-tenant or
        // null-tenant access.
        if request.caller_tenant.trim().is_empty() || request.target_tenant.trim().is_empty() {
            return Err(AuthzFault::new("empty tenant on lifecycle authz request"));
        }
        // Tenant isolation: deny-wins for any cross-tenant request.
        Ok(request.caller_tenant == request.target_tenant)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use iam_identity_workload_rest::LifecycleAction;

    fn request<'a>(caller_tenant: &'a str, target_tenant: &'a str) -> LifecycleAuthzRequest<'a> {
        LifecycleAuthzRequest {
            caller_tenant,
            caller_id: "lifecycle-control-plane",
            action: LifecycleAction::Suspend,
            target_tenant,
            target_workload_id: "wl_secrets_sync",
        }
    }

    #[test]
    fn same_tenant_is_permitted() {
        let authz = TenantScopedLifecycleAuthorizer::new();
        assert_eq!(authz.decide(&request("ten_acme", "ten_acme")), Ok(true));
    }

    #[test]
    fn cross_tenant_is_denied() {
        let authz = TenantScopedLifecycleAuthorizer::new();
        // Caller in tenant A attempting to act on tenant B's principal -> deny.
        assert_eq!(authz.decide(&request("ten_a", "ten_b")), Ok(false));
    }

    #[test]
    fn empty_tenant_is_fail_closed_fault() {
        let authz = TenantScopedLifecycleAuthorizer::new();
        assert!(authz.decide(&request("", "ten_acme")).is_err());
        assert!(authz.decide(&request("ten_acme", "")).is_err());
    }
}
