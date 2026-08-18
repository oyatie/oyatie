//! Fail-closed authorization seam for the managed-K8s cluster-lifecycle
//! admission control plane (AUTH-005 class; GitHub #979; mirrors
//! `iam/ports/policy-cedar-api/src/authz.rs` + the tenancy `VerifiedCaller`
//! extractor doctrine, and the sibling `tenant-quota-app` authz seam).
//!
//! ## Why this module exists
//!
//! `POST /clusters` previously "authorized" by comparing the caller-supplied
//! `x-oya-tenant-id` header to the caller-supplied `body.tenant_id` — both are
//! forgeable request inputs, so any caller could provision a cluster for ANY
//! tenant (the AUTH-005 class). This seam replaces that header==body trust with:
//!
//! 1. A real principal VERIFIED from an unforgeable bearer credential
//!    (constant-time compared; [`PrincipalVerifier`] port).
//! 2. A PDP authorization decision via the [`ClusterAuthorizer`] port (the Cedar
//!    `ClusterLifecycleRbacAuthorizer`): the VERIFIED principal's tenant is
//!    authoritative, and `body.tenant_id` becomes a PDP-checked resource
//!    selector — a cross-tenant create is denied unless the principal holds the
//!    platform scope.
//! 3. A REQUIRED provider in `AppState` (no authz-less constructor).

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use iam_identity_workload_domain::{WorkloadPrincipal, WorkloadState};
use k8s_cluster_lifecycle_adapter_cedar::ClusterLifecycleRbacAuthorizer;

/// The owning-capability id bound to a verified cluster operator's workload
/// principal when it is presented to the Cedar PDP.
const CLUSTER_OPERATOR_CAPABILITY: &str = "cap.k8s.cluster-lifecycle";

/// The cluster lifecycle operation a verified principal is authorized for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClusterAction {
    /// Provision (create) a tenant cluster (`cluster:Create`).
    Create,
}

/// The credential a caller presents to prove a real principal identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
}

/// A principal whose identity has been verified from a caller credential.
///
/// Fields are PRIVATE; [`VerifiedPrincipal::new`] is `pub(crate)` so external
/// crates cannot forge one by struct literal — they must run a real
/// [`PrincipalVerifier`]. Type-level defense-in-depth (not a cryptographic
/// guarantee); the real barrier is the bearer compare + the PDP decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY — private
    tenant_id: String,    // data_class: INTERNAL_ONLY — private
    scopes: Vec<String>,  // data_class: INTERNAL_ONLY — private
}

impl VerifiedPrincipal {
    /// Mint a verified principal. **`pub(crate)` only**.
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

/// Why principal verification refused (fail-closed → HTTP 401).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalVerificationError {
    /// No credential was presented.
    MissingCredential,
    /// A credential was presented but did not verify (opaque on purpose).
    InvalidCredential,
}

/// Why authorization refused (fail-closed → HTTP 403).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClusterAuthorizationError {
    /// The PDP returned a deny decision.
    Denied,
    /// The PDP refused to decide (treated as deny).
    Refused,
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
pub trait PrincipalVerifier: Send + Sync {
    /// Verify `credential` and return the authoritative principal, or refuse.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] (fail-closed: caller maps to HTTP 401).
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may perform `action` on `target_tenant`.
pub trait ClusterAuthorizer: Send + Sync {
    /// Authorize `principal` for `action` on `target_tenant`, or refuse.
    ///
    /// # Errors
    /// [`ClusterAuthorizationError`] on deny or any PDP fault (all faults MUST
    /// be `Refused`; the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: ClusterAction,
        target_tenant: &str,
    ) -> Result<(), ClusterAuthorizationError>;
}

/// The Cedar-backed [`ClusterAuthorizer`] adapter: builds the verified
/// principal's workload identity and delegates the decision to the
/// [`ClusterLifecycleRbacAuthorizer`] (Cedar default-deny).
pub struct CedarClusterAuthorizer {
    rbac: ClusterLifecycleRbacAuthorizer,
}

impl CedarClusterAuthorizer {
    /// Build the adapter over the default cluster RBAC policies.
    ///
    /// # Errors
    /// [`AuthzProviderConfigError::Policy`] if the Cedar policy set fails to
    /// compile (the composition root MUST refuse to serve).
    pub fn new_with_default_policies() -> Result<Self, AuthzProviderConfigError> {
        let rbac = ClusterLifecycleRbacAuthorizer::new_with_default_policies()
            .map_err(|e| AuthzProviderConfigError::Policy(e.to_string()))?;
        Ok(Self { rbac })
    }
}

impl ClusterAuthorizer for CedarClusterAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: ClusterAction,
        target_tenant: &str,
    ) -> Result<(), ClusterAuthorizationError> {
        let workload = build_workload_principal(principal)?;
        match action {
            ClusterAction::Create => self
                .rbac
                .authorize_cluster_create(&workload, target_tenant)
                .map_err(|_| ClusterAuthorizationError::Denied),
        }
    }
}

/// Build the Cedar workload principal from the verified credential. A
/// construction failure is fail-closed → `Refused`.
fn build_workload_principal(
    principal: &VerifiedPrincipal,
) -> Result<WorkloadPrincipal, ClusterAuthorizationError> {
    let mut workload = WorkloadPrincipal::provision(
        principal.tenant_id(),
        principal.principal_id(),
        CLUSTER_OPERATOR_CAPABILITY,
    )
    .map_err(|_| ClusterAuthorizationError::Refused)?;
    workload
        .transition_to(WorkloadState::Active)
        .map_err(|_| ClusterAuthorizationError::Refused)?;
    for scope in principal.scopes() {
        workload
            .grant_scope(scope.clone())
            .map_err(|_| ClusterAuthorizationError::Refused)?;
    }
    Ok(workload)
}

/// The authz provider the router depends on: a verifier PORT + an authorizer
/// PORT. The router REFUSES to serve without one configured (no default-allow).
#[derive(Clone)]
pub struct ClusterAuthzProvider {
    verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: Arc<dyn ClusterAuthorizer>, // data_class: INTERNAL_ONLY
}

impl ClusterAuthzProvider {
    /// Assemble the provider from a verifier and an authorizer.
    #[must_use]
    pub fn new(
        verifier: Arc<dyn PrincipalVerifier>,
        authorizer: Arc<dyn ClusterAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Build the production provider: a configured-bearer verifier (break-glass
    /// platform operator) + the Cedar-backed authorizer. REFUSES an empty bearer
    /// secret (boot-fatal).
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
        let authorizer = CedarClusterAuthorizer::new_with_default_policies()?;
        Ok(Self::new(Arc::new(verifier), Arc::new(authorizer)))
    }

    /// Verify the caller principal via the [`PrincipalVerifier`] port.
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
    /// Default-deny / fail-closed; a panicking authorizer is caught (test/debug
    /// best-effort) and mapped to `Refused`.
    ///
    /// # Errors
    /// [`ClusterAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: ClusterAction,
        target_tenant: &str,
    ) -> Result<(), ClusterAuthorizationError> {
        let authorizer = Arc::clone(&self.authorizer);
        let principal = principal.clone();
        let target_tenant = target_tenant.to_owned();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, action, &target_tenant)
        }))
        .unwrap_or(Err(ClusterAuthorizationError::Refused))
    }
}

/// Constant-time byte comparison (no early-exit). NEVER `==` on secret material.
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

/// A reference [`PrincipalVerifier`] adapter (constant-time bearer compare;
/// binds a single break-glass `(principal_id, tenant_id, scopes)`). ⚠ BREAK-GLASS
/// ONLY — production swaps the cloud-iam mTLS/SPIFFE peer-SVID verifier. REFUSES
/// an empty bearer secret or bound identity.
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

/// Why the authz provider refused construction (boot-fatal).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthzProviderConfigError {
    /// The bearer secret was empty/whitespace.
    EmptyBearerSecret,
    /// The bound principal/tenant identity was empty.
    EmptyBoundIdentity,
    /// The Cedar policy set failed to compile.
    Policy(String),
}

impl std::fmt::Display for AuthzProviderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBearerSecret => write!(f, "authz provider bearer secret must be non-empty"),
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
            ConfiguredBearerPrincipalVerifier::new("", "wl_p", "ten_p", vec![]).err(),
            Some(AuthzProviderConfigError::EmptyBearerSecret)
        );
    }

    #[test]
    fn cedar_authorizer_allows_same_tenant_and_denies_cross_tenant() {
        let authz = CedarClusterAuthorizer::new_with_default_policies().unwrap();
        let admin = VerifiedPrincipal::new_for_test(
            "wl_admin",
            "ten_acme",
            vec!["cluster:write".to_owned()],
        );
        assert!(
            authz
                .ensure_authorized(&admin, ClusterAction::Create, "ten_acme")
                .is_ok()
        );
        assert_eq!(
            authz.ensure_authorized(&admin, ClusterAction::Create, "ten_globex"),
            Err(ClusterAuthorizationError::Denied)
        );
    }

    struct FaultAuthorizer;
    impl ClusterAuthorizer for FaultAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _a: ClusterAction,
            _t: &str,
        ) -> Result<(), ClusterAuthorizationError> {
            Err(ClusterAuthorizationError::Refused)
        }
    }

    #[test]
    fn provider_maps_pdp_fault_to_refused() {
        let verifier = Arc::new(
            ConfiguredBearerPrincipalVerifier::new("s", "wl_op", "ten_p", vec![]).unwrap(),
        );
        let provider = ClusterAuthzProvider::new(verifier, Arc::new(FaultAuthorizer));
        let p = VerifiedPrincipal::new_for_test("wl_op", "ten_p", vec![]);
        assert_eq!(
            provider.ensure_authorized(&p, ClusterAction::Create, "ten_acme"),
            Err(ClusterAuthorizationError::Refused)
        );
    }
}
