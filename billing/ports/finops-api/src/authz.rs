//! Fail-closed authorization seam for the `cloud.finops.report` surface
//! (AUTH-005 class; Wave-2 capability-billing remediation; ADR-0591).
//!
//! ## Why this module exists
//!
//! The Cloud FinOps report surface generates a multi-tenant **cloud-spend
//! report** — `FINANCIAL_REGULATED_CREDIT`-class cost data for a target tenant.
//! Before this seam, the only "authz" was [`crate::validate_cloud_finops_report_request`]
//! → `validate_authorization`, which merely cross-checks the **caller-supplied**
//! [`crate::CloudFinopsApiAuthorization`] DTO (`decision_id` / `tenant_id` /
//! `principal_id` / `allowed_surfaces`) for internal consistency.  A caller who
//! can reach the boundary simply self-asserts
//! `allowed_surfaces = ["cloud.finops.report"]` and the request is accepted —
//! an unauthenticated path that lets ANY caller exfiltrate a tenant's
//! cloud-spend report (the AUTH-005 "forgeable caller-supplied authz" class; the
//! gap-fill CRIT this ADR remediates).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine in
//! `iam/ports/policy-cedar-api/src/authz.rs` (ADR-0572 / #815,
//! `cedar.policy.publish`) and `intelligence/adapters/rest/src/lib.rs`
//! (`constant_time_eq` bearer compare + a PDP `decide` port), plus the
//! true-blast-radius lesson from `secrets/ports/kms-api` (ADR / #817: the
//! resource handed to the PDP is the TARGET resource derived from a trusted
//! source, never flattened to the caller's own tenant). The seam is:
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
//!    alternate adapter).  The caller-supplied principal id is NEVER the source
//!    of truth; it is only ever a cross-check input against the verified
//!    identity.
//! 2. The verified principal is AUTHORIZED for
//!    `action = cloud.finops.report` on the target
//!    [`FinopsReportResource`] (the report's tenant/scope from a TRUSTED source)
//!    via a PDP [`FinopsReportAuthorizer`] port (`ensure_authorized`).  The
//!    tenant axis is asserted by the decision — a verified principal alone never
//!    grants a tenant.  Cross-tenant report generation is deniable AT THE PDP.
//! 3. The boundary REFUSES to serve without both ports configured (no
//!    default-allow fallback): see [`FinopsReportAuthzProvider`] — there is no
//!    `Default` and no constructor that yields a provider without both ports.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`FinopsReportAuthorizer`] are PORTS owned by this
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
/// caller-supplied principal id travels alongside as a CROSS-CHECK only — never
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
/// [`PrincipalVerifier`] implementations inside this crate.  External crates must
/// obtain one by running a real [`PrincipalVerifier`] (e.g.
/// [`ConfiguredBearerPrincipalVerifier`]).
///
/// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
/// cryptographic proof.  It prevents accidental struct-literal forging and proves
/// that *some* `PrincipalVerifier` ran.  It does NOT prevent hostile in-process
/// code from constructing its own `ConfiguredBearerPrincipalVerifier` with a
/// known secret and minting a token that way, nor does it protect against a
/// compromised or stub verifier implementation.  The real security guarantee
/// comes from the *combination* of: (1) the verifier running before the
/// sensitive op, (2) the PDP authorization decision bound to the TARGET resource,
/// and (3) the active cross-check in
/// [`crate::generate_cloud_finops_report_from_api`].  This type is one layer of
/// that defense, not the sole barrier.
///
/// Within the same crate, tests use the `#[cfg(test)]` constructor
/// [`VerifiedPrincipal::new_for_test`] to mint tokens without a real credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY — private: see unforgeability note above
    tenant_id: String,    // data_class: INTERNAL_ONLY — private: see unforgeability note above
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
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    /// The authoritative tenant the principal acts within.
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
pub enum FinopsReportAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The scope of a FinOps report resource: whether it covers one specific tenant
/// or the whole platform (all-tenant aggregate spend). Carried explicitly so the
/// PDP sees the **true blast radius** of the read, not a flattened per-tenant
/// representation.
///
/// A platform aggregate report exposes EVERY tenant's cloud spend. Presenting it
/// to the PDP as tenant-scoped with the caller's own tenant would silently
/// authorize tenant-finops-admins to exfiltrate platform-wide spend — the
/// CRITICAL escalation this enum prevents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinopsReportScope {
    /// The report is scoped to a single tenant identified by `tenant_id` in the
    /// enclosing [`FinopsReportResource`].
    Tenant,
    /// The report is platform-wide (aggregates ALL tenants). The PDP must treat
    /// this as a platform-level resource requiring platform-admin authority, NOT
    /// as a resource belonging to any individual tenant.
    Platform,
}

/// The resource a FinOps report decision is made against: the target report id,
/// the scope (tenant vs. platform-wide), and the tenant when tenant-scoped.
///
/// The scope is **explicit** so the PDP sees the true blast radius. The tenant
/// axis is asserted by the authorizer — a verified principal alone never grants
/// the tenant.  The `tenant_id` MUST be derived from a TRUSTED source (the loaded
/// target / the bound report tenant), NOT echoed back from a caller header, so a
/// cross-tenant read (principal of tenant A asking for tenant B's report) is
/// deniable at the PDP.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinopsReportResource {
    /// The report id being generated (path/body, already bound equal).
    pub report_id: String, // data_class: INTERNAL_ONLY
    /// Whether this report covers a single tenant or the whole platform.
    /// The PDP MUST distinguish these: a platform report requires platform-admin
    /// authority, not mere tenant-finops-admin authority.
    pub scope: FinopsReportScope, // data_class: INTERNAL_ONLY
    /// The tenant whose cloud spend the report exposes. For
    /// [`FinopsReportScope::Tenant`] this is the TARGET tenant; for
    /// [`FinopsReportScope::Platform`] this field is an empty string (the PDP
    /// must key on `scope == Platform`, not this field).
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

/// PORT: decide whether `principal` may read `resource` (the FinOps report).
///
/// The decision is `decide(principal, action = cloud.finops.report, resource)`.
/// Adapter: the cloud-iam PDP client (the owned W5 destination). The default
/// posture is deny; any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(FinopsReportAuthorizationError::Refused)` so the caller can map them
///    to HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
///
/// 2. **Enforce a deadline.** This is a synchronous call on a request path;
///    a hung PDP hangs the caller thread.  Adapters MUST enforce their own
///    deadline and map expiry to `Err(Refused)`.  No deadline is enforced by
///    this port — it is the adapter's responsibility.
///
/// 3. **Do not panic.** The release profile uses `panic = "abort"`, so a panic
///    in production terminates the process rather than being catchable. Do NOT
///    rely on `catch_unwind` (defeated by `panic = "abort"`); adapters MUST NOT
///    panic — use `Err(Refused)` for every recoverable and unrecoverable fault.
pub trait FinopsReportAuthorizer: Send + Sync {
    /// Authorize `principal` to read `resource`, or refuse.
    ///
    /// # Errors
    /// [`FinopsReportAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError>;
}

/// The authz provider the boundary depends on: a principal verifier PORT plus a
/// FinOps report authorizer PORT. The boundary REFUSES to serve without one
/// configured (no `Default`, no default-allow fallback) — both ports are
/// REQUIRED, non-optional construction arguments.
pub struct FinopsReportAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn FinopsReportAuthorizer>, // data_class: INTERNAL_ONLY
}

impl FinopsReportAuthzProvider {
    /// Assemble the provider from a principal verifier and a report authorizer.
    /// Both are REQUIRED: there is no constructor that yields a provider without
    /// a configured principal-verification + PDP authorization seam.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn FinopsReportAuthorizer>,
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

    /// Authorize the verified principal for the FinOps report resource via the
    /// PDP port. Default-deny / fail-closed.
    ///
    /// # Errors
    /// [`FinopsReportAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &FinopsReportResource,
    ) -> Result<(), FinopsReportAuthorizationError> {
        self.authorizer.ensure_authorized(principal, resource)
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `iam/ports/policy-cedar-api/src/authz.rs` and
/// `intelligence/adapters/rest/src/lib.rs` `constant_time_eq` — NEVER use a naive
/// `==` on secret material.
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
/// credential (e.g. a deploy-time operator token for a single known tenant) or
/// for integration tests. In multi-tenant production, every caller presents a
/// distinct credential bound to their own tenant — a single shared secret cannot
/// distinguish them, so all callers would be granted the same identity.
///
/// The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID verifier
/// (ADR-0561), which derives the principal and tenant from the verified peer
/// certificate, not from a configured mapping.
///
/// Construction REFUSES an empty bearer secret or bound identity so a provider
/// that cannot prove a credential root can never authenticate a caller (mirrors
/// the cloud-pdp boot-refusal doctrine).
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
    use std::sync::Arc;

    #[test]
    fn constant_time_eq_matches_only_identical_bytes() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secreT"));
        assert!(!constant_time_eq(b"secret", b"secret-longer"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn bearer_verifier_refuses_empty_secret_or_identity() {
        // NOTE: the verifier itself is intentionally NOT `Debug` (it holds a
        // SECRET), so we match on the error rather than `unwrap_err()`ing the Ok.
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("  ", "sp_x", "ten_x"),
            Err(AuthzProviderConfigError::EmptyBearerSecret)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("secret", "", "ten_x"),
            Err(AuthzProviderConfigError::EmptyBoundIdentity)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("secret", "sp_x", ""),
            Err(AuthzProviderConfigError::EmptyBoundIdentity)
        ));
    }

    #[test]
    fn bearer_verifier_binds_identity_from_config_not_headers() {
        let verifier = ConfiguredBearerPrincipalVerifier::new("secret", "sp_real", "ten_real")
            .expect("verifier");
        // The claimed_* header fields are deliberately a DIFFERENT identity — they
        // must NOT influence the bound result.
        let verified = verifier
            .verify_principal(&CallerCredential {
                authorization: Some("Bearer secret".to_string()),
                claimed_principal_id: "sp_attacker".to_string(),
                claimed_tenant_id: "ten_attacker".to_string(),
            })
            .expect("valid bearer");
        assert_eq!(verified.principal_id(), "sp_real");
        assert_eq!(verified.tenant_id(), "ten_real");
    }

    #[test]
    fn bearer_verifier_rejects_missing_and_wrong_credentials() {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new("secret", "sp_x", "ten_x").expect("verifier");
        assert_eq!(
            verifier.verify_principal(&CallerCredential {
                authorization: None,
                claimed_principal_id: String::new(),
                claimed_tenant_id: String::new(),
            }),
            Err(PrincipalVerificationError::MissingCredential)
        );
        assert_eq!(
            verifier.verify_principal(&CallerCredential {
                authorization: Some("Bearer wrong".to_string()),
                claimed_principal_id: String::new(),
                claimed_tenant_id: String::new(),
            }),
            Err(PrincipalVerificationError::InvalidCredential)
        );
        // A bare token with no "Bearer " scheme prefix is rejected.
        assert_eq!(
            verifier.verify_principal(&CallerCredential {
                authorization: Some("secret".to_string()),
                claimed_principal_id: String::new(),
                claimed_tenant_id: String::new(),
            }),
            Err(PrincipalVerificationError::InvalidCredential)
        );
    }

    struct AllowAll;
    impl FinopsReportAuthorizer for AllowAll {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _r: &FinopsReportResource,
        ) -> Result<(), FinopsReportAuthorizationError> {
            Ok(())
        }
    }

    #[test]
    fn provider_delegates_to_both_ports() {
        let verifier =
            Arc::new(ConfiguredBearerPrincipalVerifier::new("secret", "sp_x", "ten_x").unwrap());
        let provider = FinopsReportAuthzProvider::new(verifier, Arc::new(AllowAll));
        let verified = provider
            .verify_principal(&CallerCredential {
                authorization: Some("Bearer secret".to_string()),
                claimed_principal_id: "sp_x".to_string(),
                claimed_tenant_id: "ten_x".to_string(),
            })
            .expect("verifies");
        let resource = FinopsReportResource {
            report_id: "finr_x".to_string(),
            scope: FinopsReportScope::Tenant,
            tenant_id: "ten_x".to_string(),
        };
        assert_eq!(provider.ensure_authorized(&verified, &resource), Ok(()));
    }

    #[test]
    fn verified_principal_test_constructor_exposes_accessors() {
        let p = VerifiedPrincipal::new_for_test("sp_t", "ten_t");
        assert_eq!(p.principal_id(), "sp_t");
        assert_eq!(p.tenant_id(), "ten_t");
    }
}
