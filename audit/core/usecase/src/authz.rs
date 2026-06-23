//! Fail-closed authorization seam for the `audit.event.emit` boundary
//! (AUTH-005 class; C15 audit-tamper-evidence finding; ADR-0588).
//!
//! ## Why this module exists
//!
//! [`crate::emit_audit_event_authorized`] appends an immutable record to the
//! platform audit chain — the substrate the whole platform relies on for
//! tamper-evidence. Before this seam the only "authz" was the in-crate
//! `validate_authorization`, which merely cross-checked a CALLER-SUPPLIED
//! [`crate::AuditEventEmitAuthorization`] DTO (`{tenant_id, producer_id,
//! decision_id, allowed_surfaces}`) for internal consistency against the
//! envelope. Any caller who can reach the boundary fabricates
//! `allowed_surfaces = ["audit.event.emit"]` with a matching tenant/producer and
//! the request is accepted — forged authz fields emit audit records, defeating
//! the tamper-evidence the audit chain exists to provide (the C15 finding). The
//! self-attested authorization is NEVER a grant.
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine in
//! `iam/ports/policy-cedar-api/src/authz.rs` (#815 / ADR-0572) and
//! `iam/facade/identity-workload-rest` (#816 / ADR-0581):
//!
//! 1. A real producer principal is VERIFIED from a credential the caller cannot
//!    forge — a bearer token compared in constant time against a configured
//!    secret (the [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier
//!    is a drop-in alternate adapter). The envelope/payload-supplied
//!    tenant/producer ids are NEVER the source of truth; they are only ever a
//!    cross-check input against the verified identity.
//! 2. The verified principal is AUTHORIZED for `action = audit.event.emit` on
//!    the target `{tenant, surface}` via an [`AuditEmitAuthorizer`] PDP port
//!    (`decide`). The tenant axis is asserted by the decision — a verified
//!    principal alone never grants a tenant. The platform audit chain is a
//!    PLATFORM resource: a tenant-scoped emit and a platform-scoped emit are
//!    distinct [`AuditEmitScope`] values so the PDP sees the true blast radius.
//! 3. The boundary REFUSES to operate without both ports configured (no
//!    default-allow fallback): see [`AuditEmitAuthzProvider`] — it is a REQUIRED,
//!    non-optional argument of [`crate::emit_audit_event_authorized`], the only
//!    public emit path.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`AuditEmitAuthorizer`] are PORTS owned by this
//! boundary crate. The concrete cloud-iam Cedar PDP client and the bearer/SVID
//! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
//! destination). The port shapes model that destination so they do not change at
//! cutover; transient infra is absorbed by the adapter.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The credential a caller presents to prove a real producer principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead. The
/// envelope-supplied producer/tenant ids travel alongside as a CROSS-CHECK only
/// — never as proof of identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
    /// The caller-asserted producer id from the envelope (cross-check input).
    pub claimed_producer_id: String, // data_class: INTERNAL_ONLY
    /// The caller-asserted tenant id from the envelope (cross-check input).
    pub claimed_tenant_id: String, // data_class: INTERNAL_ONLY
}

/// A producer principal whose identity has been verified from a caller
/// credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; there is no public constructor — external crates
/// cannot build a `VerifiedProducerPrincipal` by struct literal or any public
/// API. [`VerifiedProducerPrincipal::new`] is `pub(crate)`, callable only by
/// [`PrincipalVerifier`] implementations inside this crate. External crates must
/// obtain one by running a real [`PrincipalVerifier`] (e.g.
/// [`ConfiguredBearerPrincipalVerifier`]).
///
/// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
/// cryptographic proof. It prevents accidental struct-literal forging and proves
/// that *some* `PrincipalVerifier` ran. It does NOT prevent hostile in-process
/// code from constructing its own `ConfiguredBearerPrincipalVerifier` with a
/// known secret and minting a token that way, nor does it protect against a
/// compromised or stub verifier implementation. The real security guarantee
/// comes from the *combination* of: (1) credential verification before the
/// mutation, (2) the PDP authorization decision, and (3) the active cross-check
/// in [`crate::emit_audit_event_authorized`]. This type is one layer of that
/// defense, not the sole barrier.
///
/// Within the same crate, tests use the `#[cfg(test)]` constructor
/// [`VerifiedProducerPrincipal::new_for_test`] to mint tokens without a real
/// credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedProducerPrincipal {
    producer_id: String, // data_class: INTERNAL_ONLY — private: see unforgeability note above
    tenant_id: String,   // data_class: INTERNAL_ONLY — private: see unforgeability note above
}

impl VerifiedProducerPrincipal {
    /// Mint a verified producer principal. **`pub(crate)` only** — callers
    /// outside this crate cannot call this; they must go through a
    /// [`PrincipalVerifier`].
    pub(crate) fn new(producer_id: impl Into<String>, tenant_id: impl Into<String>) -> Self {
        Self {
            producer_id: producer_id.into(),
            tenant_id: tenant_id.into(),
        }
    }

    /// The authoritative producer id bound from the verified credential.
    #[must_use]
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }

    /// The authoritative tenant the producer acts within.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Test-only constructor that mints a token without a real credential.
    /// Only available inside this crate under `#[cfg(test)]`.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        producer_id: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        Self::new(producer_id, tenant_id)
    }
}

/// Why principal verification refused. Every variant is fail-closed: the caller
/// maps it to HTTP 401 and the request never reaches the authorizer or the
/// audit chain.
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
pub enum AuditEmitAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The scope of an audit-emit resource: whether it records an event for one
/// specific tenant or for the platform itself. This is carried explicitly so
/// the PDP sees the **true blast radius** of the action, not a flattened
/// per-tenant representation.
///
/// A platform-scoped emit records against the platform's own audit lineage (no
/// owning tenant). Presenting it to the PDP as tenant-scoped with the caller's
/// own tenant would silently authorize tenant producers to forge platform-level
/// audit records — the CRITICAL escalation this enum prevents (the #815
/// global-scope lesson).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEmitScope {
    /// The event is recorded for a single tenant identified by `tenant_id` in
    /// the enclosing [`AuditEmitResource`].
    Tenant,
    /// The event is recorded against the platform's own audit lineage (applies
    /// to no individual tenant). The PDP must treat this as a platform-level
    /// resource requiring platform-audit authority, NOT as a resource belonging
    /// to any individual tenant.
    Platform,
}

/// The resource an audit-emit decision is made against: the target tenant, the
/// surface the event attests, and the scope (tenant vs. platform-level). The
/// `tenant_id` and `surface` are derived from a TRUSTED source — the validated
/// request payload's TARGET fields — never flattened to the caller's own
/// verified tenant, so a cross-tenant emit is deniable AT THE PDP (no IDOR; the
/// #817 lesson). The tenant axis is asserted by the authorizer — a verified
/// principal alone never grants the tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEmitResource {
    /// The tenant the audit event is recorded for (from the validated payload's
    /// TARGET tenant, not the caller's verified tenant). For
    /// [`AuditEmitScope::Platform`] this field is an empty string (the PDP must
    /// key on `scope == Platform`, not this field).
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// The operational surface the event attests (from the validated payload).
    pub surface: String, // data_class: INTERNAL_ONLY
    /// Whether this emit records for a single tenant or the platform itself.
    /// The PDP MUST distinguish these: a platform emit requires platform-audit
    /// authority, not mere tenant-producer authority.
    pub scope: AuditEmitScope, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedProducerPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination, ADR-0561). The verifier — not the envelope
/// fields — is the source of truth for caller identity.
pub trait PrincipalVerifier: Send + Sync {
    /// Verify `credential` and return the authoritative principal, or refuse.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] when no credential is presented or it does
    /// not verify (fail-closed: the caller MUST treat this as 401).
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedProducerPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may emit an audit event for `resource`.
///
/// The decision is `decide(principal, action = audit.event.emit, resource)`.
/// Adapter: the cloud-iam Cedar PDP client (the owned W5 destination). The
/// default posture is deny; any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(AuditEmitAuthorizationError::Refused)` so the caller can map them to
///    HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
///
/// 2. **Enforce a deadline.** This is a synchronous call on a mutation path; a
///    hung PDP hangs the caller thread. Adapters MUST enforce their own deadline
///    and map expiry to `Err(Refused)`. No deadline is enforced by this port —
///    it is the adapter's responsibility.
///
/// 3. **Do not panic.** The release profile uses `panic = "abort"`, so a panic
///    in production terminates the process rather than being catchable.
///    `catch_unwind` is DEFEATED by `panic = "abort"`; this crate deliberately
///    does NOT wrap the adapter in `catch_unwind` and does NOT claim a
///    panic-becomes-403 guarantee. Adapters MUST NOT panic — use `Err(Refused)`
///    for every recoverable and unrecoverable fault.
pub trait AuditEmitAuthorizer: Send + Sync {
    /// Authorize `principal` to emit an audit event for `resource`, or refuse.
    ///
    /// # Errors
    /// [`AuditEmitAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedProducerPrincipal,
        resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError>;
}

/// The authz provider the audit-emit boundary depends on: a principal verifier
/// PORT plus an audit-emit authorizer PORT. The boundary REFUSES to emit without
/// one configured (no default-allow fallback) — it is a REQUIRED, non-optional
/// argument of [`crate::emit_audit_event_authorized`].
pub struct AuditEmitAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn AuditEmitAuthorizer>, // data_class: INTERNAL_ONLY
}

impl AuditEmitAuthzProvider {
    /// Assemble the provider from a principal verifier and an audit-emit
    /// authorizer. There is no `Default` and no allow-all constructor: a process
    /// that cannot prove a credential root and reach a PDP can never emit.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn AuditEmitAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal. Returns the authoritative identity or a
    /// fail-closed 401-class refusal. Delegates to the [`PrincipalVerifier`]
    /// port — the envelope fields are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedProducerPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the audit-emit resource via the PDP
    /// port. Default-deny / fail-closed.
    ///
    /// # Errors
    /// [`AuditEmitAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedProducerPrincipal,
        resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError> {
        self.authorizer.ensure_authorized(principal, resource)
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `iam/ports/policy-cedar-api/src/authz.rs`
/// `constant_time_eq` — NEVER use a naive `==` on secret material.
///
/// **Residual:** the length of both inputs is visible from the XOR seed
/// (`a.len() ^ b.len()`), so an attacker who can probe many lengths still learns
/// whether lengths match. This is the same residual as the repo reference and is
/// accepted; in practice bearer tokens are fixed-length secrets. Use a MAC
/// (HMAC-SHA256) if length-hiding is required.
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
/// constant-time compare against a configured secret, then binds the producer
/// identity from the configured mapping (NOT from the envelope fields).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// This adapter binds ONE static `(producer_id, tenant_id)` pair to a single
/// shared secret. It is suitable only as a **single-principal break-glass**
/// credential (e.g. a deploy-time operator token for a single known tenant) or
/// for integration tests. In multi-tenant production, every producer presents a
/// distinct credential bound to its own tenant — a single shared secret cannot
/// distinguish them, so all callers would be granted the same identity.
///
/// The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID verifier
/// (ADR-0561), which derives the producer and tenant from the verified peer
/// certificate, not from a configured mapping.
///
/// Construction REFUSES an empty bearer secret or empty bound identity so a
/// provider that cannot prove a credential root can never authenticate a caller
/// (mirrors the cloud-pdp boot-refusal doctrine).
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,     // data_class: SECRET
    bound_producer_id: String, // data_class: INTERNAL_ONLY
    bound_tenant_id: String,   // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerPrincipalVerifier {
    /// Construct, REFUSING an empty bearer secret or empty bound identity. A
    /// process that cannot prove a credential root must never authenticate.
    ///
    /// # Errors
    /// [`AuthzProviderConfigError`] when the secret or bound identity is empty.
    pub fn new(
        bearer_secret: impl Into<String>,
        bound_producer_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
    ) -> Result<Self, AuthzProviderConfigError> {
        let bearer_secret = bearer_secret.into();
        let bound_producer_id = bound_producer_id.into();
        let bound_tenant_id = bound_tenant_id.into();
        if bearer_secret.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBearerSecret);
        }
        if bound_producer_id.trim().is_empty() || bound_tenant_id.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBoundIdentity);
        }
        Ok(Self {
            bearer_secret,
            bound_producer_id,
            bound_tenant_id,
        })
    }
}

impl PrincipalVerifier for ConfiguredBearerPrincipalVerifier {
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedProducerPrincipal, PrincipalVerificationError> {
        let Some(authorization) = credential.authorization.as_deref() else {
            return Err(PrincipalVerificationError::MissingCredential);
        };
        let Some(presented) = authorization.strip_prefix("Bearer ") else {
            return Err(PrincipalVerificationError::InvalidCredential);
        };
        if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
            return Err(PrincipalVerificationError::InvalidCredential);
        }
        Ok(VerifiedProducerPrincipal::new(
            self.bound_producer_id.clone(),
            self.bound_tenant_id.clone(),
        ))
    }
}

/// Why the authz provider refused construction. Boot-fatal: the composition root
/// MUST refuse to serve, mirroring the cloud-pdp `build_state` boot-refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthzProviderConfigError {
    /// The bearer secret was empty/whitespace (no provable credential root).
    EmptyBearerSecret,
    /// The bound producer/tenant identity was empty.
    EmptyBoundIdentity,
}

impl std::fmt::Display for AuthzProviderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBearerSecret => {
                write!(
                    f,
                    "audit-emit authz provider bearer secret must be non-empty"
                )
            }
            Self::EmptyBoundIdentity => {
                write!(
                    f,
                    "audit-emit authz provider bound producer/tenant must be non-empty"
                )
            }
        }
    }
}

impl std::error::Error for AuthzProviderConfigError {}
