//! identity — identity service for oyatie.
//!
//! Single-crate-per-service pattern per ADR-0509.
//! Subsystems: auth, oidc, oauth2, realms, users, storage, rest, grpc, observability.
//! (Passkeys/WebAuthn returns behind a port in its own sub-slice — the
//! webauthn-rs -> openssl chain is buck2-unbuildable on current runners,
//! see the friction ledger.)

#![forbid(unsafe_code)]

pub mod auth;
pub mod config;
pub mod decision_authz {
    //! Decision-authorization adapter for the READ decision surfaces.
    //!
    //! AUTH-005: `/authorize`, `/authorize-with-token`, `/authorize:batch`, and
    //! `/tokens/validate` are decision surfaces that — before ADR-0581's seam was
    //! extended to them — built the authorized principal (or introspected a token)
    //! from caller-supplied input over an unauthenticated socket. A forged body or a
    //! cross-tenant token therefore obtained an arbitrary cross-tenant decision.
    //! [`iam_identity_workload_rest::DecisionAuthorizer`] is the PORT (owned by the
    //! boundary crate); this is the concrete ADAPTER, placed in the composition root
    //! (the service crate) per the ports-and-adapters layering — the boundary crate
    //! never depends on a concrete PDP.
    //!
    //! This adapter enforces the tenant-isolation invariant directly: a verified
    //! caller may obtain a decision ONLY within its own tenant. A cross-tenant
    //! request (caller in tenant A asking for a decision over tenant B's subject) is
    //! DENIED regardless of action — closing the cross-tenant-entitlement / IDOR
    //! axis. It is the W5-shaped default; a richer Cedar-policy-backed iam PDP
    //! swaps in behind the same port without touching the delivery surfaces.
    //!
    //! ADR-0083 Tier 3: panic-free; `#![forbid(unsafe_code)]` inherited at crate root.

    use iam_identity_workload_rest::{AuthzFault, DecisionAuthorizer, DecisionAuthzRequest};

    /// Tenant-scoped decision authorizer: PERMIT iff the verified caller's tenant
    /// equals the subject's tenant. Cross-tenant decisions are denied.
    ///
    /// This is deliberately conservative (same-tenant only) and mirrors
    /// [`crate::lifecycle_authz::TenantScopedLifecycleAuthorizer`]. A platform-admin
    /// capability or a Cedar-policy-backed decision is a future adapter swap behind
    /// the same [`DecisionAuthorizer`] port; the delivery surfaces are unaffected.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct TenantScopedDecisionAuthorizer;

    impl TenantScopedDecisionAuthorizer {
        /// Construct the authorizer.
        #[must_use]
        pub fn new() -> Self {
            Self
        }
    }

    impl DecisionAuthorizer for TenantScopedDecisionAuthorizer {
        fn decide(&self, request: &DecisionAuthzRequest<'_>) -> Result<bool, AuthzFault> {
            // Defensive: an empty caller/subject tenant is never a valid permit basis
            // (fail-closed) — a misconfigured credential or an unidentifiable subject
            // cannot grant cross-tenant or null-tenant access.
            if request.caller_tenant.trim().is_empty() || request.subject_tenant.trim().is_empty() {
                return Err(AuthzFault::new("empty tenant on decision authz request"));
            }
            // Tenant isolation: deny-wins for any cross-tenant request.
            Ok(request.caller_tenant == request.subject_tenant)
        }
    }

    #[cfg(test)]
    mod tests {
        #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

        use super::*;

        fn request<'a>(
            caller_tenant: &'a str,
            subject_tenant: &'a str,
        ) -> DecisionAuthzRequest<'a> {
            DecisionAuthzRequest {
                caller_tenant,
                caller_id: "control-plane",
                subject_tenant,
                subject_workload_id: "wl_secrets_sync",
                action: "cloud.kms.Decrypt",
                resource_type: "Secret",
                resource_id: "db-password",
            }
        }

        #[test]
        fn same_tenant_is_permitted() {
            let authz = TenantScopedDecisionAuthorizer::new();
            assert_eq!(authz.decide(&request("ten_acme", "ten_acme")), Ok(true));
        }

        #[test]
        fn cross_tenant_is_denied() {
            let authz = TenantScopedDecisionAuthorizer::new();
            // Caller in tenant A asking for a decision over tenant B's subject -> deny.
            assert_eq!(authz.decide(&request("ten_a", "ten_evil")), Ok(false));
        }

        #[test]
        fn empty_tenant_is_fail_closed_fault() {
            let authz = TenantScopedDecisionAuthorizer::new();
            assert!(authz.decide(&request("", "ten_acme")).is_err());
            assert!(authz.decide(&request("ten_acme", "")).is_err());
        }
    }
}
pub mod grpc;
pub mod lifecycle_authz;
pub mod oauth2;
pub mod observability;
pub mod oidc;
pub mod realms;
pub mod rest;
pub mod server;
pub mod storage;
pub mod users;

use iam_identity_workload_app::{InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_rest::WorkloadAuthzState;

use crate::observability::TracingAuditSink;

/// The composed application state: in-memory bring-up stores behind the
/// repository/denylist ports (G03 swaps the durable store in behind the same
/// ports), the embedded Cedar PDP, the static JWKS, and the tracing audit sink.
pub type AppState = WorkloadAuthzState<
    InMemoryWorkloadPrincipalRepository,
    InMemoryRevocationDenylist,
    CedarWorkloadAuthorizer,
    TracingAuditSink,
>;
