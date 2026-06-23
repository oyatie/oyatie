//! Fail-closed authorization seam for the `cloud.network.lb.create` control
//! plane (AUTH-005 / C9 class; ADR-0587).
//!
//! ## Why this module exists
//!
//! Load-balancer creation
//! ([`crate::create_cloud_network_load_balancer_from_api`]) is a MUTATING
//! multi-tenant control plane. Before this seam the only "authz" was the
//! request-supplied `CloudNetworkLbApiAuthorization` blob, whose
//! `allowed_surfaces` list the boundary merely cross-checked for internal
//! consistency. An attacker who can reach the call sets
//! `allowed_surfaces = ["cloud.network.lb.create"]` (with a matching
//! self-attested `tenant_id` / `principal_id`) and the request is accepted — an
//! unauthenticated control plane (the AUTH-005 class the founder mandate
//! requires to be impossible to ship).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine in
//! `iam/ports/policy-cedar-api/src/authz.rs` (#815) and the workload-principal
//! lifecycle fix (#816):
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge — a
//!    bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
//!    alternate adapter). The request-supplied principal id is NEVER the source
//!    of truth; it is only ever a cross-check input against the verified
//!    identity.
//! 2. The verified principal is AUTHORIZED for
//!    `action = cloud.network.lb.create` on the TARGET `{tenant, load_balancer}`
//!    via a PDP [`LbCreateAuthorizer`] port (`ensure_authorized`). The target
//!    tenant is derived from the trusted request body (already cross-checked
//!    equal to the verified principal's tenant), so a cross-tenant action is
//!    deniable AT THE PDP.
//! 3. The boundary REFUSES TO SERVE without both ports configured (no
//!    default-allow fallback): [`create_cloud_network_load_balancer_from_api`]
//!    takes a required `&CloudNetworkLbAuthzProvider`.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`LbCreateAuthorizer`] are PORTS owned by this
//! boundary crate. The concrete cloud-iam PDP client and the bearer/SVID
//! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
//! destination). The port shapes model that destination so they do not change at
//! cutover; transient infra is absorbed by the adapter.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead. The
/// request-supplied principal id travels alongside as a CROSS-CHECK only — never
/// as proof of identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
    /// The caller-asserted principal id (cross-check input).
    pub claimed_principal_id: String, // data_class: INTERNAL_ONLY
    /// The caller-asserted principal tenant (cross-check input).
    pub claimed_tenant_id: String, // data_class: INTERNAL_ONLY
}

/// A principal whose identity has been verified from a caller credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; there is no public constructor — external crates
/// cannot build a `VerifiedPrincipal` by struct literal or any public API.
/// [`VerifiedPrincipal::new`] is `pub(crate)`, callable only by
/// [`PrincipalVerifier`] implementations inside this crate. External crates must
/// obtain one by running a real [`PrincipalVerifier`] (e.g.
/// [`ConfiguredBearerPrincipalVerifier`]).
///
/// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
/// cryptographic proof. It prevents accidental struct-literal forging and proves
/// that *some* `PrincipalVerifier` ran. The real security guarantee comes from
/// the combination of: (1) verifying the credential before any mutation, (2) the
/// PDP authorization decision against the target resource, and (3) the active
/// cross-check in [`crate::create_cloud_network_load_balancer_from_api`].
///
/// Within the same crate, tests use the `#[cfg(test)]` constructor
/// [`VerifiedPrincipal::new_for_test`] to mint tokens without a real credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY — private: see unforgeability note
    tenant_id: String,    // data_class: INTERNAL_ONLY — private: see unforgeability note
}

impl VerifiedPrincipal {
    /// Mint a verified principal. **`pub(crate)` only** — callers outside this
    /// crate cannot call this; they must go through a [`PrincipalVerifier`].
    pub(crate) fn new(principal_id: impl Into<String>, tenant_id: impl Into<String>) -> Self {
        Self {
            principal_id: principal_id.into(),
            tenant_id: tenant_id.into(),
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

    /// Test-only constructor that mints a token without a real credential.
    /// Only available inside this crate under `#[cfg(test)]`.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        principal_id: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        Self::new(principal_id, tenant_id)
    }
}

/// Why principal verification refused. Every variant is fail-closed: the caller
/// maps it to HTTP 401 and the request never reaches the authorizer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalVerificationError {
    /// No credential was presented (no `Authorization` header).
    MissingCredential,
    /// A credential was presented but did not verify (bad bearer, untrusted
    /// SVID, expired, …). Deliberately opaque so probing cannot distinguish
    /// "wrong token" from "no such principal".
    InvalidCredential,
}

/// Why authorization refused. Each variant maps to HTTP 403 (the principal is
/// authenticated but not permitted for this action/resource/tenant).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LbCreateAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The resource a load-balancer-create decision is made against: the TARGET
/// tenant and load-balancer id, derived from the trusted request body (already
/// cross-checked equal to the verified principal's tenant). The tenant axis is
/// asserted by the authorizer — a verified principal alone never grants the
/// tenant. Presenting the TARGET tenant (not a flattened caller tenant) is what
/// makes a cross-tenant create deniable at the PDP (no IDOR).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LbCreateResource {
    /// The tenant whose catalog the load balancer lands in (trusted source).
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// The load balancer resource id being created (from the path/body, already
    /// bound equal).
    pub load_balancer_id: String, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination). The verifier — not the headers — is the source
/// of truth for caller identity.
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

/// PORT: decide whether `principal` may create the load balancer `resource`.
///
/// The decision is
/// `decide(principal, action = cloud.network.lb.create, resource)`. Adapter: the
/// cloud-iam PDP client (the owned W5 destination). The default posture is deny;
/// any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(LbCreateAuthorizationError::Refused)` so the caller can map them to
///    HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
/// 2. **Enforce a deadline.** A hung PDP hangs the caller. Adapters MUST enforce
///    their own deadline and map expiry to `Err(Refused)`.
/// 3. **Do not panic.** The release profile uses `panic = "abort"`, so a panic
///    in production terminates the process rather than being catchable. Adapters
///    MUST NOT panic — use `Err(Refused)` for every recoverable and
///    unrecoverable fault. (Do not rely on `catch_unwind` for production fault
///    isolation; it is defeated by `panic = "abort"`.)
pub trait LbCreateAuthorizer: Send + Sync {
    /// Authorize `principal` to create the load balancer `resource`, or refuse.
    ///
    /// # Errors
    /// [`LbCreateAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError>;
}

/// The authz provider the boundary depends on: a principal verifier PORT plus an
/// LB-create authorizer PORT. The boundary REFUSES to serve without one
/// configured (no default-allow fallback) — there is no `Default` impl.
pub struct CloudNetworkLbAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn LbCreateAuthorizer>, // data_class: INTERNAL_ONLY
}

impl CloudNetworkLbAuthzProvider {
    /// Assemble the provider from a principal verifier and an LB-create
    /// authorizer.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn LbCreateAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal via the [`PrincipalVerifier`] port. The
    /// headers are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the LB-create resource via the PDP
    /// port. Default-deny / fail-closed.
    ///
    /// # Errors
    /// [`LbCreateAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError> {
        self.authorizer.ensure_authorized(principal, resource)
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `iam/ports/policy-cedar-api/src/authz.rs` — NEVER use
/// a naive `==` on secret material.
///
/// **Residual:** the length of both inputs is visible from the XOR seed
/// (`a.len() ^ b.len()`). This is the accepted repo-wide residual; bearer tokens
/// are fixed-length secrets. Use a MAC (HMAC-SHA256) if length-hiding is
/// required.
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
/// identity from the configured mapping (NOT from the caller headers).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// This adapter binds ONE static `(principal_id, tenant_id)` pair to a single
/// shared secret. It is suitable only as a single-principal break-glass
/// credential or for integration tests. The production W5 adapter is the
/// cloud-iam mTLS/SPIFFE peer-SVID verifier, which derives the principal and
/// tenant from the verified peer certificate, not from a configured mapping.
///
/// Construction REFUSES an empty bearer secret or bound identity so a provider
/// that cannot prove a credential root can never authenticate a caller.
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,      // data_class: SECRET
    bound_principal_id: String, // data_class: INTERNAL_ONLY
    bound_tenant_id: String,    // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerPrincipalVerifier {
    /// Construct, REFUSING an empty bearer secret or empty bound identity. A
    /// process that cannot prove a credential root must never authenticate.
    ///
    /// # Errors
    /// [`AuthzProviderConfigError`] when the secret or bound identity is empty.
    pub fn new(
        bearer_secret: impl Into<String>,
        bound_principal_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
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
        ))
    }
}

/// Why the authz provider refused construction. Boot-fatal: the composition root
/// MUST refuse to serve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthzProviderConfigError {
    /// The bearer secret was empty/whitespace (no provable credential root).
    EmptyBearerSecret,
    /// The bound principal/tenant identity was empty.
    EmptyBoundIdentity,
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
        }
    }
}

impl std::error::Error for AuthzProviderConfigError {}
