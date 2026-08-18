//! Fail-closed authorization seam for the managed-K8s control-plane-host admin
//! control plane (AUTH-005 class; GitHub #979; mirrors
//! `iam/ports/policy-cedar-api/src/authz.rs` + the tenancy `VerifiedCaller`
//! extractor doctrine, and the sibling k8s facade authz seams).
//!
//! ## Why this module exists
//!
//! `POST /admin/control-planes{,/status,/teardown}` previously had ZERO
//! authentication or authorization: any caller who reached the socket could
//! provision, inspect, or tear down a tenant control plane (the AUTH-005 class).
//!
//! These are PLATFORM-LEVEL operations (they manage the management cluster, not
//! a single tenant), so this seam closes the gap with:
//!
//! 1. A real principal VERIFIED from an unforgeable bearer credential
//!    (constant-time compared; [`PrincipalVerifier`] port).
//! 2. A PDP authorization decision via the [`PlatformAdminAuthorizer`] port: the
//!    verified principal MUST hold the platform-operator scope. A
//!    `ConfiguredPlatformAdminAuthorizer` is the reference adapter behind the
//!    same `ensure_authorized` port; a cloud-iam Cedar adapter is a drop-in.
//! 3. A REQUIRED provider in `AppState` (no authz-less constructor).

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

/// The platform-operator scope a verified principal must hold to operate the
/// control-plane-host admin surface.
pub const PLATFORM_OPERATOR_SCOPE: &str = "control-plane:platform:admin";

/// The control-plane-host operation a verified principal is authorized for. All
/// three are platform-level admin operations requiring the platform scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlPlaneAction {
    /// Provision a new tenant control plane.
    Provision,
    /// Read a control plane's status.
    Status,
    /// Drain + delete a control plane.
    Teardown,
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
/// [`PrincipalVerifier`]. Type-level defense-in-depth (not cryptographic); the
/// real barrier is the bearer compare + the PDP decision.
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

    /// The scopes carried by the verified credential.
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Whether the principal holds `scope`.
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|granted| granted == scope)
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
pub enum ControlPlaneAuthorizationError {
    /// The PDP returned a deny decision (the principal lacks the platform scope).
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

/// PORT: decide whether `principal` may perform a platform-level `action`.
pub trait PlatformAdminAuthorizer: Send + Sync {
    /// Authorize `principal` for `action`, or refuse.
    ///
    /// # Errors
    /// [`ControlPlaneAuthorizationError`] on deny or any PDP fault (all faults
    /// MUST be `Refused`; the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: ControlPlaneAction,
    ) -> Result<(), ControlPlaneAuthorizationError>;
}

/// The reference [`PlatformAdminAuthorizer`] adapter: a fail-closed scope check.
/// The verified principal is authorized iff it holds the configured
/// platform-operator scope. A cloud-iam Cedar adapter is a drop-in behind the
/// same port (the W5 destination); the scope check is the break-glass posture.
pub struct ConfiguredPlatformAdminAuthorizer {
    required_scope: String, // data_class: INTERNAL_ONLY
}

impl ConfiguredPlatformAdminAuthorizer {
    /// Build with the default platform-operator scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required_scope: PLATFORM_OPERATOR_SCOPE.to_owned(),
        }
    }
}

impl Default for ConfiguredPlatformAdminAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformAdminAuthorizer for ConfiguredPlatformAdminAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        _action: ControlPlaneAction,
    ) -> Result<(), ControlPlaneAuthorizationError> {
        if principal.has_scope(&self.required_scope) {
            Ok(())
        } else {
            Err(ControlPlaneAuthorizationError::Denied)
        }
    }
}

/// The authz provider the router depends on: a verifier PORT + an authorizer
/// PORT. The router REFUSES to serve without one configured (no default-allow).
#[derive(Clone)]
pub struct ControlPlaneAuthzProvider {
    verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: Arc<dyn PlatformAdminAuthorizer>, // data_class: INTERNAL_ONLY
}

impl ControlPlaneAuthzProvider {
    /// Assemble the provider from a verifier and an authorizer.
    #[must_use]
    pub fn new(
        verifier: Arc<dyn PrincipalVerifier>,
        authorizer: Arc<dyn PlatformAdminAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Build the production provider: a configured-bearer verifier (break-glass
    /// platform operator) + the scope-check authorizer. REFUSES an empty bearer
    /// secret (boot-fatal).
    ///
    /// # Errors
    /// [`AuthzProviderConfigError`] when the bearer secret or bound identity is
    /// empty.
    pub fn from_bearer_secret(
        bearer_secret: impl Into<String>,
        bound_principal_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
    ) -> Result<Self, AuthzProviderConfigError> {
        let verifier = ConfiguredBearerPrincipalVerifier::new(
            bearer_secret,
            bound_principal_id,
            bound_tenant_id,
            vec![PLATFORM_OPERATOR_SCOPE.to_owned()],
        )?;
        Ok(Self::new(
            Arc::new(verifier),
            Arc::new(ConfiguredPlatformAdminAuthorizer::new()),
        ))
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

    /// Authorize the verified principal for the action via the PDP port.
    /// Default-deny / fail-closed; a panicking authorizer is caught (test/debug
    /// best-effort) and mapped to `Refused`.
    ///
    /// # Errors
    /// [`ControlPlaneAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        action: ControlPlaneAction,
    ) -> Result<(), ControlPlaneAuthorizationError> {
        let authorizer = Arc::clone(&self.authorizer);
        let principal = principal.clone();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, action)
        }))
        .unwrap_or(Err(ControlPlaneAuthorizationError::Refused))
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
}

impl std::fmt::Display for AuthzProviderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBearerSecret => write!(f, "authz provider bearer secret must be non-empty"),
            Self::EmptyBoundIdentity => {
                write!(f, "authz provider bound principal/tenant must be non-empty")
            }
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
    fn platform_admin_allowed_other_scope_denied() {
        let authz = ConfiguredPlatformAdminAuthorizer::new();
        let operator = VerifiedPrincipal::new_for_test(
            "op",
            "ten_platform",
            vec![PLATFORM_OPERATOR_SCOPE.to_owned()],
        );
        let tenant =
            VerifiedPrincipal::new_for_test("t", "ten_acme", vec!["other:scope".to_owned()]);
        assert!(
            authz
                .ensure_authorized(&operator, ControlPlaneAction::Provision)
                .is_ok()
        );
        assert_eq!(
            authz.ensure_authorized(&tenant, ControlPlaneAction::Provision),
            Err(ControlPlaneAuthorizationError::Denied)
        );
    }

    #[test]
    fn provider_verifies_then_authorizes() {
        let provider =
            ControlPlaneAuthzProvider::from_bearer_secret("s3cr3t", "op", "ten_platform").unwrap();
        let principal = provider.verify_principal(&cred("s3cr3t")).unwrap();
        assert!(
            provider
                .ensure_authorized(&principal, ControlPlaneAction::Teardown)
                .is_ok()
        );
        assert_eq!(
            provider.verify_principal(&cred("nope")).err(),
            Some(PrincipalVerificationError::InvalidCredential)
        );
    }

    struct FaultAuthorizer;
    impl PlatformAdminAuthorizer for FaultAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _a: ControlPlaneAction,
        ) -> Result<(), ControlPlaneAuthorizationError> {
            Err(ControlPlaneAuthorizationError::Refused)
        }
    }

    #[test]
    fn provider_maps_pdp_fault_to_refused() {
        let verifier =
            Arc::new(ConfiguredBearerPrincipalVerifier::new("s", "op", "t", vec![]).unwrap());
        let provider = ControlPlaneAuthzProvider::new(verifier, Arc::new(FaultAuthorizer));
        let p = VerifiedPrincipal::new_for_test("op", "t", vec![]);
        assert_eq!(
            provider.ensure_authorized(&p, ControlPlaneAction::Status),
            Err(ControlPlaneAuthorizationError::Refused)
        );
    }
}
