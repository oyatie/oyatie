//! Fail-closed authorization seam for the `cedar.policy.publish` control plane
//! (AUTH-005 class; task #124; ADR-0572).
//!
//! ## Why this module exists
//!
//! The publish surface (`POST /policies/{policy_id}/versions/{version}`) is a
//! MUTATING multi-tenant control plane.  Before this seam, the only "authz" was
//! [`crate::validate_authorization`], which merely cross-checks self-attested
//! `x-principal-*` / `x-authorization-*` headers for internal consistency.  An
//! attacker who can reach the socket sets those headers consistently and the
//! request is accepted — an unauthenticated control plane (the AUTH-005 class
//! that PR #768 shipped and the #780 authz-coverage gate baselined as debt).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine in
//! `intelligence/adapters/rest/src/lib.rs` (`constant_time_eq` bearer compare +
//! a PDP `decide` port) and the cloud-iam PDP caller-authn precedent
//! (`iam/facade/cloud-pdp-app/src/mtls.rs`, ADR-0561 / #38):
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE verifier is a drop-in alternate
//!    adapter).  The URL/header-supplied principal id is NEVER the source of
//!    truth; it is only ever a cross-check input against the verified identity.
//! 2. The verified principal is AUTHORIZED for
//!    `action = cedar.policy.publish` on the target `{policy_id, tenant}` via a
//!    PDP [`PublishAuthorizer`] port (`decide`).  The tenant axis is asserted by
//!    the decision — a verified principal alone never grants a tenant.
//! 3. The router REFUSES TO SERVE without both ports configured (no
//!    default-allow fallback): see [`crate::rest::build_router`].
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`PublishAuthorizer`] are PORTS owned by this
//! boundary crate.  The concrete cloud-iam PDP client and the bearer/SVID
//! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
//! destination).  The port shapes model that destination so they do not change
//! at cutover; transient infra is absorbed by the adapter.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead.  The
/// header-supplied principal id travels alongside as a CROSS-CHECK only — never
/// as proof of identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
    /// The caller-asserted principal id from `x-principal-id` (cross-check input).
    pub claimed_principal_id: String, // data_class: INTERNAL_ONLY
    /// The caller-asserted principal tenant from `x-principal-tenant-id`
    /// (cross-check input).
    pub claimed_tenant_id: String, // data_class: INTERNAL_ONLY
}

/// A principal whose identity has been VERIFIED from an unforgeable credential.
///
/// Only the [`PrincipalVerifier`] port may construct this; the fields are the
/// authoritative identity bound from the verified credential, NOT the
/// self-attested headers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    /// Authoritative principal id derived from the verified credential.
    pub principal_id: String, // data_class: INTERNAL_ONLY
    /// Authoritative tenant the principal acts within.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
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
pub enum PublishAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The resource a publish decision is made against: the target policy and the
/// tenant whose policy store it lands in. The tenant axis is asserted by the
/// authorizer — a verified principal alone never grants the tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishResource {
    /// The policy id being published (from the path/body, already bound equal).
    pub policy_id: String, // data_class: INTERNAL_ONLY
    /// The tenant whose policy store the version lands in (the operator tenant
    /// for global scope, the scope tenant for tenant scope).
    pub tenant_id: String, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination, ADR-0561). The verifier — not the headers — is
/// the source of truth for caller identity.
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

/// PORT: decide whether `principal` may publish `resource`.
///
/// The decision is `decide(principal, action = cedar.policy.publish, resource)`.
/// Adapter: the cloud-iam PDP client (the owned W5 destination). The default
/// posture is deny; any refusal is treated as deny (fail-closed).
pub trait PublishAuthorizer: Send + Sync {
    /// Authorize `principal` to publish `resource`, or refuse.
    ///
    /// # Errors
    /// [`PublishAuthorizationError`] on an explicit deny or a PDP refusal
    /// (fail-closed: the caller MUST treat this as 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError>;
}

/// The authz provider the router depends on: a principal verifier PORT plus a
/// publish authorizer PORT. The router REFUSES to serve without one configured
/// (no default-allow fallback) — see [`crate::rest::build_router`].
pub struct CedarPolicyAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn PublishAuthorizer>, // data_class: INTERNAL_ONLY
}

impl CedarPolicyAuthzProvider {
    /// Assemble the provider from a principal verifier and a publish authorizer.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn PublishAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal. Returns the authoritative identity or a
    /// fail-closed 401-class refusal. Delegates to the [`PrincipalVerifier`]
    /// port — the headers are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the publish resource via the PDP
    /// port. Default-deny / fail-closed.
    ///
    /// # Errors
    /// [`PublishAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError> {
        self.authorizer.ensure_authorized(principal, resource)
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `intelligence/adapters/rest/src/lib.rs`
/// `constant_time_eq` — NEVER use a naive `==` on secret material.
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
/// This is the in-process, TLS-terminator-agnostic verify→bind→deny logic; a
/// cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561) is the drop-in W5
/// alternate. Construction REFUSES an empty bearer secret so a provider that
/// cannot prove a credential root can never authenticate a caller (mirrors the
/// cloud-pdp boot-refusal doctrine).
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,            // data_class: SECRET
    bound_principal_id: String,       // data_class: INTERNAL_ONLY
    bound_tenant_id: String,          // data_class: INTERNAL_ONLY
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
        Ok(VerifiedPrincipal {
            principal_id: self.bound_principal_id.clone(),
            tenant_id: self.bound_tenant_id.clone(),
        })
    }
}

/// Why the authz provider refused construction. Boot-fatal: the composition root
/// MUST refuse to serve, mirroring the cloud-pdp `build_state` boot-refusal.
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
