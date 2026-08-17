//! Fail-closed authorization seam for the managed-K8s tenant-quota control
//! plane (AUTH-005 class; GitHub #979; mirrors `iam/ports/policy-cedar-api/
//! src/authz.rs` + the tenancy `VerifiedCaller` extractor doctrine).
//!
//! ## Why this module exists
//!
//! The quota admin surface (`PUT/GET /tenants/{id}/quota`, `GET
//! /tenants/{id}/usage`, `POST /tenants/{id}/quota/check`) is a multi-tenant
//! control plane. Before this seam the router shipped with NO caller
//! authentication and NO authorization decision: the
//! `k8s/adapters/tenant-quota-adapter-cedar` `QuotaRbacAuthorizer` existed and
//! was tested but was NEVER wired — any caller who reached the socket could
//! read or overwrite any tenant's quota (the AUTH-005 class the #770
//! authz-coverage gate baselined as debt).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine:
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE verifier is a drop-in alternate
//!    adapter). The URL/header-supplied tenant is NEVER the source of truth for
//!    identity.
//! 2. The verified principal is AUTHORIZED for the action on the target tenant
//!    via the [`QuotaAuthorizer`] PDP port (`ensure_authorized`), backed by the
//!    existing Cedar [`QuotaRbacAuthorizer`]. The tenant axis is asserted by the
//!    decision — a verified principal alone never grants a tenant.
//! 3. The router REFUSES TO SERVE without the provider configured (no
//!    default-allow fallback): [`crate::AppState`] holds a REQUIRED
//!    [`QuotaAuthzProvider`] with no authz-less constructor.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`QuotaAuthorizer`] are PORTS owned by this
//! boundary crate. The concrete Cedar PDP client and the bearer/SVID credential
//! store are ADAPTERS — the Cedar one ([`CedarQuotaAuthorizer`]) wraps the
//! existing `QuotaRbacAuthorizer`, the bearer one is
//! [`ConfiguredBearerPrincipalVerifier`]. The port shapes model the owned W5
//! destination so they do not change at cutover.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use iam_identity_workload_domain::{WorkloadPrincipal, WorkloadState};
use k8s_tenant_quota_adapter_cedar::QuotaRbacAuthorizer;
use k8s_tenant_quota_kernel::TenantId;

/// The owning-capability id bound to a verified quota operator's workload
/// principal when it is presented to the Cedar PDP. Quota admin is a single
/// owned capability; the principal id (workload id) and tenant come from the
/// verified credential.
const QUOTA_OPERATOR_CAPABILITY: &str = "cap.k8s.tenant-quota";

/// The quota operation a verified principal is authorized for. The PDP decision
/// is keyed on this action plus the target tenant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaAction {
    /// Set or replace a tenant's quota (`quota:Write`).
    Write,
    /// Read a tenant's quota / usage or run a quota check (`quota:Read`).
    Read,
}

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
}

/// A principal whose identity has been verified from a caller credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; the only public constructor is absent — external
/// crates cannot build a `VerifiedPrincipal` by struct literal or any public API.
/// [`VerifiedPrincipal::new`] is `pub(crate)`, callable only by
/// [`PrincipalVerifier`] implementations inside this crate. The real security
/// guarantee comes from the COMBINATION of: (1) the `VerifiedCaller`
/// `FromRequestParts` extractor running before the body is deserialized,
/// (2) the constant-time bearer compare, and (3) the PDP authorization decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY — private: see note above
    tenant_id: String,    // data_class: INTERNAL_ONLY — private: see note above
    scopes: Vec<String>,  // data_class: INTERNAL_ONLY — private: see note above
}

impl VerifiedPrincipal {
    /// Mint a verified principal. **`pub(crate)` only** — callers outside this
    /// crate cannot call this; they must go through a [`PrincipalVerifier`].
    pub(crate) fn new(
        principal_id: impl Into<String>,
        tenant_id: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            principal_id: principal_id.into(),
            tenant_id: tenant_id.into(),
            scopes,
        }
    }

    /// The authoritative principal id bound from the verified credential.
    #[must_use]
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    /// The authoritative tenant the principal acts within.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// The scopes carried by the verified credential (drive the Cedar decision).
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Test-only constructor that mints a principal without a real credential.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        principal_id: impl Into<String>,
        tenant_id: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self::new(principal_id, tenant_id, scopes)
    }
}

/// Why principal verification refused. Every variant is fail-closed: the caller
/// maps it to HTTP 401 and the request never reaches the authorizer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalVerificationError {
    /// No credential was presented (no `Authorization` header).
    MissingCredential,
    /// A credential was presented but did not verify. Deliberately opaque so
    /// probing cannot distinguish "wrong token" from "no such principal".
    InvalidCredential,
}

/// Why authorization refused. Each variant maps to HTTP 403 (the principal is
/// authenticated but not permitted for this action/tenant).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/tenant.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
///
/// Adapter: the configured-bearer verifier ([`ConfiguredBearerPrincipalVerifier`])
/// or a cloud-iam mTLS/SPIFFE peer-SVID verifier (the W5 destination).
pub trait PrincipalVerifier: Send + Sync {
    /// Verify `credential` and return the authoritative principal, or refuse.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] when no credential is presented or it does
    /// not verify (fail-closed: the caller MUST treat this as 401).
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may perform `action` on `target_tenant`.
///
/// Adapter: [`CedarQuotaAuthorizer`] wrapping the owned `QuotaRbacAuthorizer`
/// (the cloud-iam Cedar PDP is the W5 destination). The default posture is
/// deny; any internal fault MUST be mapped to `Err(Refused)` (fail-closed).
pub trait QuotaAuthorizer: Send + Sync {
    /// Authorize `principal` for `action` on `target_tenant`, or refuse.
    ///
    /// # Errors
    /// [`QuotaAuthorizationError`] on an explicit deny or any PDP fault (all
    /// faults MUST be `Refused`; the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: QuotaAction,
        target_tenant: &TenantId,
    ) -> Result<(), QuotaAuthorizationError>;
}

/// The Cedar-backed [`QuotaAuthorizer`] adapter: builds the verified principal's
/// workload identity and delegates the decision to the existing
/// [`QuotaRbacAuthorizer`] (Cedar default-deny). This is the wiring SLICE A adds.
pub struct CedarQuotaAuthorizer {
    rbac: QuotaRbacAuthorizer,
}

impl CedarQuotaAuthorizer {
    /// Build the adapter over the default quota RBAC policies (production path).
    ///
    /// # Errors
    /// [`AuthzProviderConfigError::Policy`] if the Cedar policy set fails to
    /// compile (the composition root MUST refuse to serve).
    pub fn new_with_default_policies() -> Result<Self, AuthzProviderConfigError> {
        let rbac = QuotaRbacAuthorizer::new_with_default_policies()
            .map_err(|e| AuthzProviderConfigError::Policy(e.to_string()))?;
        Ok(Self { rbac })
    }
}

impl QuotaAuthorizer for CedarQuotaAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: QuotaAction,
        target_tenant: &TenantId,
    ) -> Result<(), QuotaAuthorizationError> {
        let workload = build_workload_principal(principal)?;
        let decision = match action {
            QuotaAction::Write => self.rbac.authorize_quota_write(&workload, target_tenant),
            QuotaAction::Read => self.rbac.authorize_quota_read(&workload, target_tenant),
        };
        decision.map_err(|_| QuotaAuthorizationError::Denied)
    }
}

/// Build the Cedar workload principal from the verified credential. A
/// construction failure (invalid bound id/scope) is fail-closed → `Refused`.
fn build_workload_principal(
    principal: &VerifiedPrincipal,
) -> Result<WorkloadPrincipal, QuotaAuthorizationError> {
    let mut workload = WorkloadPrincipal::provision(
        principal.tenant_id(),
        principal.principal_id(),
        QUOTA_OPERATOR_CAPABILITY,
    )
    .map_err(|_| QuotaAuthorizationError::Refused)?;
    workload
        .transition_to(WorkloadState::Active)
        .map_err(|_| QuotaAuthorizationError::Refused)?;
    for scope in principal.scopes() {
        workload
            .grant_scope(scope.clone())
            .map_err(|_| QuotaAuthorizationError::Refused)?;
    }
    Ok(workload)
}

/// The authz provider the router depends on: a principal verifier PORT plus a
/// quota authorizer PORT. The router REFUSES to serve without one configured
/// (no default-allow fallback) — see [`crate::AppState`].
pub struct QuotaAuthzProvider {
    verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: Arc<dyn QuotaAuthorizer>, // data_class: INTERNAL_ONLY
}

impl QuotaAuthzProvider {
    /// Assemble the provider from a principal verifier and a quota authorizer.
    #[must_use]
    pub fn new(verifier: Arc<dyn PrincipalVerifier>, authorizer: Arc<dyn QuotaAuthorizer>) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Build the production provider: a configured-bearer verifier (break-glass
    /// platform operator) + the Cedar-backed authorizer. REFUSES an empty bearer
    /// secret (boot-fatal — no provable credential root, no service).
    ///
    /// # Errors
    /// [`AuthzProviderConfigError`] when the bearer secret is empty or the Cedar
    /// policy set fails to compile.
    pub fn from_bearer_secret(
        bearer_secret: impl Into<String>,
        bound_principal_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
        bound_scopes: Vec<String>,
    ) -> Result<Self, AuthzProviderConfigError> {
        let verifier = ConfiguredBearerPrincipalVerifier::new(
            bearer_secret,
            bound_principal_id,
            bound_tenant_id,
            bound_scopes,
        )?;
        let authorizer = CedarQuotaAuthorizer::new_with_default_policies()?;
        Ok(Self::new(Arc::new(verifier), Arc::new(authorizer)))
    }

    /// Verify the caller principal. Delegates to the [`PrincipalVerifier`] port —
    /// the headers are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the action/tenant via the PDP port.
    /// Default-deny / fail-closed. A panicking authorizer is caught (test/debug
    /// best-effort; production uses `panic = "abort"`) and mapped to `Refused`.
    ///
    /// # Errors
    /// [`QuotaAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: QuotaAction,
        target_tenant: &TenantId,
    ) -> Result<(), QuotaAuthorizationError> {
        let authorizer = Arc::clone(&self.authorizer);
        let principal = principal.clone();
        let target_tenant = target_tenant.clone();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, action, &target_tenant)
        }))
        .unwrap_or(Err(QuotaAuthorizationError::Refused))
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. NEVER use a naive `==` on secret material.
#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

/// A reference [`PrincipalVerifier`] adapter that verifies a bearer token by a
/// constant-time compare against a configured secret, then binds the principal
/// identity + scopes from the configured mapping (NOT from caller headers).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// Binds ONE static `(principal_id, tenant_id, scopes)` tuple to a single shared
/// secret — a single-principal break-glass credential (e.g. a deploy-time
/// platform-operator token) or for tests. The production W5 adapter is the
/// cloud-iam mTLS/SPIFFE peer-SVID verifier, which derives the principal, tenant,
/// and scopes from the verified peer certificate. Construction REFUSES an empty
/// bearer secret or bound identity so a provider that cannot prove a credential
/// root can never authenticate a caller.
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,      // data_class: SECRET
    bound_principal_id: String, // data_class: INTERNAL_ONLY
    bound_tenant_id: String,    // data_class: INTERNAL_ONLY
    bound_scopes: Vec<String>,  // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerPrincipalVerifier {
    /// Construct, REFUSING an empty bearer secret or empty bound identity.
    ///
    /// # Errors
    /// [`AuthzProviderConfigError`] when the secret or bound identity is empty.
    pub fn new(
        bearer_secret: impl Into<String>,
        bound_principal_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
        bound_scopes: Vec<String>,
    ) -> Result<Self, AuthzProviderConfigError> {
        let bearer_secret = bearer_secret.into();
        let bound_principal_id = bound_principal_id.into();
        let bound_tenant_id = bound_tenant_id.into();
        if bearer_secret.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBearerSecret);
        }
        if bound_principal_id.trim().is_empty() || bound_tenant_id.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBoundIdentity);
        }
        Ok(Self {
            bearer_secret,
            bound_principal_id,
            bound_tenant_id,
            bound_scopes,
        })
    }
}

impl PrincipalVerifier for ConfiguredBearerPrincipalVerifier {
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        let Some(authorization) = credential.authorization.as_deref() else {
            return Err(PrincipalVerificationError::MissingCredential);
        };
        let Some(presented) = authorization.strip_prefix("Bearer ") else {
            return Err(PrincipalVerificationError::InvalidCredential);
        };
        if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
            return Err(PrincipalVerificationError::InvalidCredential);
        }
        Ok(VerifiedPrincipal::new(
            self.bound_principal_id.clone(),
            self.bound_tenant_id.clone(),
            self.bound_scopes.clone(),
        ))
    }
}

/// Why the authz provider refused construction. Boot-fatal: the composition root
/// MUST refuse to serve (no default-allow fallback when authz is unavailable).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthzProviderConfigError {
    /// The bearer secret was empty/whitespace (no provable credential root).
    EmptyBearerSecret,
    /// The bound principal/tenant identity was empty.
    EmptyBoundIdentity,
    /// The Cedar policy set failed to compile.
    Policy(String),
}

impl std::fmt::Display for AuthzProviderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBearerSecret => {
                write!(f, "authz provider bearer secret must be non-empty")
            }
            Self::EmptyBoundIdentity => {
                write!(f, "authz provider bound principal/tenant must be non-empty")
            }
            Self::Policy(detail) => write!(f, "authz provider cedar policy build failed: {detail}"),
        }
    }
}

impl std::error::Error for AuthzProviderConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn cred(token: &str) -> CallerCredential {
        CallerCredential {
            authorization: Some(format!("Bearer {token}")),
        }
    }

    #[test]
    fn bearer_verifier_refuses_empty_secret() {
        assert_eq!(
            ConfiguredBearerPrincipalVerifier::new("", "p", "t", vec![]).err(),
            Some(AuthzProviderConfigError::EmptyBearerSecret)
        );
    }

    #[test]
    fn bearer_verifier_accepts_matching_token() {
        let v = ConfiguredBearerPrincipalVerifier::new(
            "s3cr3t",
            "op",
            "ten_platform",
            vec!["quota:platform:write".to_owned()],
        )
        .unwrap();
        let p = v.verify_principal(&cred("s3cr3t")).unwrap();
        assert_eq!(p.principal_id(), "op");
        assert_eq!(p.tenant_id(), "ten_platform");
    }

    #[test]
    fn bearer_verifier_rejects_wrong_and_missing() {
        let v = ConfiguredBearerPrincipalVerifier::new("s3cr3t", "op", "t", vec![]).unwrap();
        assert_eq!(
            v.verify_principal(&cred("nope")).err(),
            Some(PrincipalVerificationError::InvalidCredential)
        );
        assert_eq!(
            v.verify_principal(&CallerCredential {
                authorization: None
            })
            .err(),
            Some(PrincipalVerificationError::MissingCredential)
        );
    }

    #[test]
    fn cedar_authorizer_allows_same_tenant_admin_and_denies_cross_tenant() {
        let authz = CedarQuotaAuthorizer::new_with_default_policies().unwrap();
        let admin =
            VerifiedPrincipal::new_for_test("wl_admin", "ten_acme", vec!["quota:write".to_owned()]);
        let own = TenantId::new("ten_acme").unwrap();
        let other = TenantId::new("ten_globex").unwrap();
        assert!(
            authz
                .ensure_authorized(&admin, QuotaAction::Write, &own)
                .is_ok()
        );
        assert_eq!(
            authz.ensure_authorized(&admin, QuotaAction::Write, &other),
            Err(QuotaAuthorizationError::Denied)
        );
    }

    /// A faulting authorizer (PDP fault) must surface as `Refused` (=> 403), not
    /// a 5xx — fail-closed.
    struct FaultAuthorizer;
    impl QuotaAuthorizer for FaultAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _a: QuotaAction,
            _t: &TenantId,
        ) -> Result<(), QuotaAuthorizationError> {
            Err(QuotaAuthorizationError::Refused)
        }
    }

    #[test]
    fn provider_maps_pdp_fault_to_refused() {
        let verifier =
            Arc::new(ConfiguredBearerPrincipalVerifier::new("s", "op", "t", vec![]).unwrap());
        let provider = QuotaAuthzProvider::new(verifier, Arc::new(FaultAuthorizer));
        let p = VerifiedPrincipal::new_for_test("op", "t", vec![]);
        let tid = TenantId::new("ten_acme").unwrap();
        assert_eq!(
            provider.ensure_authorized(&p, QuotaAction::Read, &tid),
            Err(QuotaAuthorizationError::Refused)
        );
    }
}
