//! Fail-closed authorization seam for the `dsr.cascade.execute` control plane
//! (AUTH-005 / whole-repo-review C16 class; ADR-0589).
//!
//! ## Why this module exists
//!
//! `POST /privacy/dsr/{dsr_id}:cascade-execute` is a MUTATING, multi-tenant,
//! GDPR-erasure control plane: it records the irreversible cascade that proves a
//! data-subject's records were erased across every store. Before this seam, the
//! only "authz" was [`crate::validate_platform_dsr_cascade_execute_request`],
//! which merely cross-checked a CALLER-SUPPLIED
//! [`crate::PlatformDsrApiAuthorization`] blob
//! (`{decision_id, tenant_id, principal_id, allowed_surfaces}`) for internal
//! consistency. Any caller who reaches the socket fabricates a matching
//! `{tenant_id, principal_id}` and an `allowed_surfaces` containing
//! `dsr.cascade.execute`, and the request is accepted — a forged principal could
//! trigger an erasure cascade with NO verified PDP decision (the C16 CRITICAL).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine
//! landed for `cedar.policy.publish` (`iam/ports/policy-cedar-api/src/authz.rs`,
//! ADR-0572 / #815), the Cloud KMS crypto control plane (ADR-0573 / #817), and
//! the cloud-iam PDP caller-authn precedent (`iam/facade/cloud-pdp-app`,
//! ADR-0561 / #38):
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge — a
//!    bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
//!    alternate adapter). The self-attested principal/tenant fields are NEVER the
//!    source of truth; they are only ever cross-check inputs against the verified
//!    identity.
//! 2. The verified principal is AUTHORIZED for `action = dsr.cascade.execute` on
//!    the TARGET `{dsr_id, scope, tenant}` via a [`DsrCascadeAuthorizer`] PDP port
//!    (`decide`). The tenant/scope axis is asserted by the decision — a verified
//!    principal alone never grants a tenant or platform scope.
//! 3. The boundary REFUSES TO ACT without both ports configured (no
//!    default-allow fallback): [`execute_dsr_cascade_from_api`] REQUIRES a
//!    `&VerifiedDsrPrincipal` and a `&DsrCascadeAuthzProvider` by type, so no
//!    in-process caller or future route can reach the mutation without completing
//!    verification + PDP authorization.
//!
//! [`execute_dsr_cascade_from_api`]: crate::execute_dsr_cascade_from_api
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`DsrCascadeAuthorizer`] are PORTS owned by this
//! boundary crate. The concrete cloud-iam Cedar PDP client and the bearer/SVID
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
/// self-attested principal id travels alongside as a CROSS-CHECK only — never as
/// proof of identity.
///
/// The derived `Debug` is deliberately NOT used: a custom `Debug` redacts the
/// `authorization` field so bearer tokens never appear in logs, tracing spans,
/// or panic output (the #817 review lesson).
#[derive(Clone, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
    /// The caller-asserted principal id from `x-principal-id` (cross-check input).
    pub claimed_principal_id: String, // data_class: INTERNAL_ONLY
    /// The caller-asserted principal tenant from `x-principal-tenant-id`
    /// (cross-check input).
    pub claimed_tenant_id: String, // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for CallerCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallerCredential")
            .field(
                "authorization",
                &self.authorization.as_ref().map(|_| "[REDACTED]"),
            )
            .field("claimed_principal_id", &self.claimed_principal_id)
            .field("claimed_tenant_id", &self.claimed_tenant_id)
            .finish()
    }
}

/// A principal whose identity has been verified from a caller credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; there is no public constructor — external crates
/// cannot build a `VerifiedDsrPrincipal` by struct literal or any public API.
/// [`VerifiedDsrPrincipal::new`] is `pub(crate)`, callable only by
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
/// comes from the *combination* of: (1) bearer verification before body
/// deserialization at the HTTP edge, (2) the PDP authorization decision, and
/// (3) the active cross-check in [`crate::execute_dsr_cascade_from_api`]. This
/// type is one layer of that defense, not the sole barrier.
///
/// Within the same crate, tests use the `#[cfg(test)]` constructor
/// [`VerifiedDsrPrincipal::new_for_test`] to mint tokens without a real
/// credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedDsrPrincipal {
    principal_id: String, // data_class: PII_IDENTIFYING — private: see unforgeability note above
    tenant_id: String,    // data_class: INTERNAL_ONLY — private: see unforgeability note above
}

impl VerifiedDsrPrincipal {
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
pub enum DsrCascadeAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The scope of a DSR cascade resource: whether it affects one specific tenant's
/// data subject or a platform-wide (cross-tenant) erasure. This is carried
/// explicitly so the PDP sees the **true blast radius** of the action, not a
/// flattened per-tenant representation.
///
/// A platform-scoped cascade can erase records that belong to no single tenant
/// (e.g. a shared/global identity store). Presenting it to the PDP as
/// tenant-scoped with the caller's own tenant would silently authorize a
/// tenant-admin for a platform-wide erasure — the CRITICAL escalation this enum
/// prevents (mirrors `PublishScope::Global` in ADR-0572).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DsrCascadeScope {
    /// The cascade is scoped to a single tenant identified by `tenant_id` in the
    /// enclosing [`DsrCascadeResource`].
    Tenant,
    /// The cascade is platform-scoped (affects shared/cross-tenant stores). The
    /// PDP must treat this as a platform-level resource requiring platform-admin
    /// authority, NOT as a resource belonging to any individual tenant.
    Platform,
}

/// The platform/global tenant sentinel. A cascade request whose trusted body
/// tenant equals this sentinel is treated as a platform-scoped erasure that
/// requires platform-admin authority at the PDP, never as a tenant-scoped action
/// flattened to the caller's own tenant.
pub const DSR_PLATFORM_TENANT_SENTINEL: &str = "platform";

/// The resource a DSR cascade decision is made against: the target DSR id, the
/// scope (tenant vs. platform-level), and the tenant when tenant-scoped. The
/// scope is **explicit** so the PDP sees the true blast radius. The tenant axis
/// is asserted by the authorizer — a verified principal alone never grants the
/// tenant.
///
/// The resource is derived from TRUSTED inputs (the path `dsr_id` and the body
/// tenant, after [`crate::validate_platform_dsr_cascade_execute_request`] has
/// bound header/body/principal tenants equal), NOT from a caller-supplied
/// authorization blob. This is what prevents the IDOR / cross-tenant-flattening
/// class (the #817 lesson).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrCascadeResource {
    /// The DSR id whose cascade is being executed (from the path, bound equal to
    /// the body).
    pub dsr_id: String, // data_class: INTERNAL_ONLY
    /// Whether this cascade affects a single tenant or the platform (global).
    /// The PDP MUST distinguish these: platform cascade requires platform-admin
    /// authority, not mere tenant-admin authority.
    pub scope: DsrCascadeScope, // data_class: INTERNAL_ONLY
    /// The tenant whose data subject the cascade erases. For
    /// [`DsrCascadeScope::Tenant`] this is the (trusted) target tenant; for
    /// [`DsrCascadeScope::Platform`] this field is an empty string (the PDP must
    /// key on `scope == Platform`, not this field).
    pub tenant_id: String, // data_class: INTERNAL_ONLY
}

impl DsrCascadeResource {
    /// Build the PDP resource from TRUSTED inputs: the target DSR id and the
    /// trusted target tenant. When the target tenant is the platform sentinel the
    /// resource is platform-scoped (blast-radius = all tenants) and the
    /// `tenant_id` field is blanked so the PDP keys on the scope, not the field.
    #[must_use]
    pub fn for_target(dsr_id: impl Into<String>, target_tenant_id: &str) -> Self {
        if target_tenant_id == DSR_PLATFORM_TENANT_SENTINEL {
            Self {
                dsr_id: dsr_id.into(),
                scope: DsrCascadeScope::Platform,
                tenant_id: String::new(),
            }
        } else {
            Self {
                dsr_id: dsr_id.into(),
                scope: DsrCascadeScope::Tenant,
                tenant_id: target_tenant_id.to_string(),
            }
        }
    }
}

/// PORT: verify a caller credential into a [`VerifiedDsrPrincipal`].
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
    ) -> Result<VerifiedDsrPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may execute the DSR cascade on `resource`.
///
/// The decision is `decide(principal, action = dsr.cascade.execute, resource)`.
/// Adapter: the cloud-iam Cedar PDP client (the owned W5 destination). The
/// default posture is deny; any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(DsrCascadeAuthorizationError::Refused)` so the caller can map them to
///    HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
///
/// 2. **Enforce a deadline.** This is a synchronous call on a request path; a
///    hung PDP hangs the caller thread. Adapters MUST enforce their own deadline
///    and map expiry to `Err(Refused)`. No deadline is enforced by this port — it
///    is the adapter's responsibility.
///
/// 3. **Do not panic.** The release profile uses `panic = "abort"`, so a panic in
///    production terminates the process rather than being catchable. The
///    [`DsrCascadeAuthzProvider`] wrapper calls `catch_unwind` as a
///    **debug/test-only best-effort** backstop that works only when the panic
///    strategy is `unwind` (i.e. in tests); it MUST NOT be relied upon in
///    production. Adapters MUST NOT panic — use `Err(Refused)` for every
///    recoverable and unrecoverable fault.
pub trait DsrCascadeAuthorizer: Send + Sync {
    /// Authorize `principal` to execute the cascade on `resource`, or refuse.
    ///
    /// # Errors
    /// [`DsrCascadeAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedDsrPrincipal,
        resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError>;
}

/// The authz provider the boundary depends on: a principal verifier PORT plus a
/// cascade authorizer PORT. [`crate::execute_dsr_cascade_from_api`] REFUSES to
/// act without one configured (no default-allow fallback) because it takes
/// `&DsrCascadeAuthzProvider` by type and there is no `Default` impl.
pub struct DsrCascadeAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn DsrCascadeAuthorizer>, // data_class: INTERNAL_ONLY
}

impl DsrCascadeAuthzProvider {
    /// Assemble the provider from a principal verifier and a cascade authorizer.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn DsrCascadeAuthorizer>,
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
    ) -> Result<VerifiedDsrPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the cascade resource via the PDP
    /// port. Default-deny / fail-closed.
    ///
    /// ## Panic / fault handling
    ///
    /// This wrapper calls `catch_unwind` as a **test/debug-only best-effort**
    /// backstop for panicking authorizer implementations. In production the
    /// release profile sets `panic = "abort"`, which terminates the process
    /// immediately on panic without any unwinding — `catch_unwind` has NO effect
    /// and the process aborts. The real fail-closed guarantee comes from the
    /// [`DsrCascadeAuthorizer`] adapter contract: adapters MUST map every fault
    /// (timeout, network error, unavailability) to `Err(Refused)` and MUST NOT
    /// panic. The catch_unwind here catches only test panics so integration tests
    /// can verify the `Panic → 403` property without process termination.
    ///
    /// # Errors
    /// [`DsrCascadeAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedDsrPrincipal,
        resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        // Best-effort catch for test-environment panics (panic strategy = unwind).
        // In production (panic = "abort") this catch_unwind is a no-op and a
        // panicking adapter terminates the process — do not rely on this for
        // production fault isolation.
        let authorizer = std::sync::Arc::clone(&self.authorizer);
        let principal = principal.clone();
        let resource = resource.clone();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, &resource)
        }))
        .unwrap_or(Err(DsrCascadeAuthorizationError::Refused))
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `intelligence/adapters/rest/src/lib.rs`
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
/// constant-time compare against a configured secret, then binds the principal
/// identity from the configured mapping (NOT from the caller headers).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// This adapter binds ONE static `(principal_id, tenant_id)` pair to a single
/// shared secret. It is suitable only as a **single-principal break-glass**
/// credential (e.g. a deploy-time privacy-officer token for a single known
/// tenant) or for integration tests. In multi-tenant production, every caller
/// presents a distinct credential bound to their own tenant — a single shared
/// secret cannot distinguish them, so all callers would be granted the same
/// identity.
///
/// The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID verifier
/// (ADR-0561), which derives the principal and tenant from the verified peer
/// certificate, not from a configured mapping.
///
/// Construction REFUSES an empty bearer secret or empty bound identity so a
/// provider that cannot prove a credential root can never authenticate a caller
/// (mirrors the cloud-pdp boot-refusal doctrine).
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,      // data_class: SECRET
    bound_principal_id: String, // data_class: PII_IDENTIFYING
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
    ) -> Result<VerifiedDsrPrincipal, PrincipalVerificationError> {
        let Some(authorization) = credential.authorization.as_deref() else {
            return Err(PrincipalVerificationError::MissingCredential);
        };
        let Some(presented) = authorization.strip_prefix("Bearer ") else {
            return Err(PrincipalVerificationError::InvalidCredential);
        };
        if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
            return Err(PrincipalVerificationError::InvalidCredential);
        }
        Ok(VerifiedDsrPrincipal::new(
            self.bound_principal_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn provider_secret() -> &'static str {
        "dsr-break-glass-secret"
    }

    /// `ConfiguredBearerPrincipalVerifier` deliberately does NOT derive `Debug`
    /// (it holds a SECRET), so we cannot use `unwrap_err()` on its
    /// constructor result — match the error directly instead.
    fn config_err(
        result: Result<ConfiguredBearerPrincipalVerifier, AuthzProviderConfigError>,
    ) -> AuthzProviderConfigError {
        match result {
            Ok(_) => panic!("expected a config error"),
            Err(err) => err,
        }
    }

    #[test]
    fn bearer_verifier_refuses_empty_secret_or_identity() {
        assert_eq!(
            config_err(ConfiguredBearerPrincipalVerifier::new("", "p", "t")),
            AuthzProviderConfigError::EmptyBearerSecret
        );
        assert_eq!(
            config_err(ConfiguredBearerPrincipalVerifier::new("   ", "p", "t")),
            AuthzProviderConfigError::EmptyBearerSecret
        );
        assert_eq!(
            config_err(ConfiguredBearerPrincipalVerifier::new("s", "", "t")),
            AuthzProviderConfigError::EmptyBoundIdentity
        );
        assert_eq!(
            config_err(ConfiguredBearerPrincipalVerifier::new("s", "p", "")),
            AuthzProviderConfigError::EmptyBoundIdentity
        );
    }

    #[test]
    fn bearer_verifier_binds_configured_identity_not_caller_headers() {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new(provider_secret(), "officer:kr", "ten_kr")
                .unwrap();
        let verified = verifier
            .verify_principal(&CallerCredential {
                authorization: Some(format!("Bearer {}", provider_secret())),
                // Caller LIES about identity in the headers — must be ignored.
                claimed_principal_id: "attacker".to_string(),
                claimed_tenant_id: "ten_attacker".to_string(),
            })
            .expect("valid bearer verifies");
        assert_eq!(verified.principal_id(), "officer:kr");
        assert_eq!(verified.tenant_id(), "ten_kr");
    }

    #[test]
    fn bearer_verifier_missing_and_wrong_credential_fail_closed() {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new(provider_secret(), "officer:kr", "ten_kr")
                .unwrap();
        assert_eq!(
            verifier
                .verify_principal(&CallerCredential {
                    authorization: None,
                    claimed_principal_id: String::new(),
                    claimed_tenant_id: String::new(),
                })
                .unwrap_err(),
            PrincipalVerificationError::MissingCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&CallerCredential {
                    authorization: Some("Bearer wrong".to_string()),
                    claimed_principal_id: String::new(),
                    claimed_tenant_id: String::new(),
                })
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&CallerCredential {
                    authorization: Some(provider_secret().to_string()), // no "Bearer " prefix
                    claimed_principal_id: String::new(),
                    claimed_tenant_id: String::new(),
                })
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
    }

    #[test]
    fn constant_time_eq_matches_only_identical_inputs() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn caller_credential_debug_redacts_bearer_secret() {
        let credential = CallerCredential {
            authorization: Some("Bearer super-secret-token".to_string()),
            claimed_principal_id: "officer:kr".to_string(),
            claimed_tenant_id: "ten_kr".to_string(),
        };
        let rendered = format!("{credential:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("super-secret-token"));
    }

    #[test]
    fn resource_for_target_distinguishes_tenant_and_platform_blast_radius() {
        let tenant = DsrCascadeResource::for_target("dsr_001", "ten_kr");
        assert_eq!(tenant.scope, DsrCascadeScope::Tenant);
        assert_eq!(tenant.tenant_id, "ten_kr");

        let platform = DsrCascadeResource::for_target("dsr_001", DSR_PLATFORM_TENANT_SENTINEL);
        assert_eq!(platform.scope, DsrCascadeScope::Platform);
        assert_eq!(platform.tenant_id, "");
    }

    struct AllowAllAuthorizer;
    impl DsrCascadeAuthorizer for AllowAllAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedDsrPrincipal,
            _r: &DsrCascadeResource,
        ) -> Result<(), DsrCascadeAuthorizationError> {
            Ok(())
        }
    }

    struct PanicAuthorizer;
    impl DsrCascadeAuthorizer for PanicAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedDsrPrincipal,
            _r: &DsrCascadeResource,
        ) -> Result<(), DsrCascadeAuthorizationError> {
            panic!("adapter blew up");
        }
    }

    fn provider_with(
        authorizer: std::sync::Arc<dyn DsrCascadeAuthorizer>,
    ) -> DsrCascadeAuthzProvider {
        let verifier = std::sync::Arc::new(
            ConfiguredBearerPrincipalVerifier::new(provider_secret(), "officer:kr", "ten_kr")
                .unwrap(),
        );
        DsrCascadeAuthzProvider::new(verifier, authorizer)
    }

    #[test]
    fn provider_wraps_panicking_authorizer_into_refused() {
        let provider = provider_with(std::sync::Arc::new(PanicAuthorizer));
        let principal = VerifiedDsrPrincipal::new_for_test("officer:kr", "ten_kr");
        let resource = DsrCascadeResource::for_target("dsr_001", "ten_kr");
        // Suppress the default panic hook output during the deliberate panic.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = provider.ensure_authorized(&principal, &resource);
        std::panic::set_hook(prev);
        assert_eq!(result, Err(DsrCascadeAuthorizationError::Refused));
    }

    #[test]
    fn provider_allow_path_authorizes() {
        let provider = provider_with(std::sync::Arc::new(AllowAllAuthorizer));
        let principal = VerifiedDsrPrincipal::new_for_test("officer:kr", "ten_kr");
        let resource = DsrCascadeResource::for_target("dsr_001", "ten_kr");
        assert_eq!(provider.ensure_authorized(&principal, &resource), Ok(()));
    }
}
