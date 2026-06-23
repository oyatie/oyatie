//! Fail-closed authorization seam for the Cloud KMS crypto control plane
//! (AUTH-005 / C5-crypto class; ADR-0573).
//!
//! ## Why this module exists
//!
//! `secrets/ports/kms-api` is the boundary for the **crown-jewel** Cloud KMS
//! encrypt/decrypt surfaces (`cloud.kms.encrypt` / `cloud.kms.decrypt`).  Before
//! this seam, the only "authz" was [`crate::validate_authorization`], which
//! merely cross-checks a **caller-supplied** [`crate::CloudKmsApiAuthorization`]
//! blob (`{decision_id, tenant_id, principal_id, allowed_surfaces}`) for
//! internal consistency against the (also caller-supplied)
//! [`crate::CloudKmsApiPrincipal`].  An attacker who can reach the socket forges
//! that blob — sets `allowed_surfaces` to include the requested surface and the
//! tenant/principal to match — and the request authorizes a KMS decrypt/encrypt
//! with NO verified PDP decision.  This is the AUTH-005 unauthenticated /
//! caller-supplied-authz class on the most security-critical surface in the
//! repo (the C5 CRITICAL finding).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine that
//! landed for the Cedar policy publish control plane (`iam/ports/policy-cedar-api`
//! / ADR-0572) and `intelligence/adapters/rest/src/lib.rs` (`constant_time_eq`
//! bearer compare + a PDP `decide` port) and the cloud-iam PDP caller-authn
//! precedent (`iam/facade/cloud-pdp-app/src/mtls.rs`, ADR-0561 / #38):
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
//!    alternate adapter).  The caller-supplied principal / authorization blob is
//!    NEVER the source of truth for identity.
//! 2. The verified principal is AUTHORIZED for
//!    `action = cloud.kms.encrypt|decrypt` on the target
//!    `{tenant, key_id, data_class, purpose, request_id}` via a PDP
//!    [`KmsCryptoAuthorizer`] port (`decide`).  The resource the PDP sees is
//!    bound to the VERIFIED principal's tenant and the TARGET key id — never
//!    flattened to caller input — so cross-tenant decrypt is deniable (no
//!    key-id IDOR).
//! 3. The boundary REFUSES to authorize a crypto op without both ports running:
//!    the public crate functions require a [`VerifiedKmsPrincipal`] (unforgeable
//!    type) and the composition root binds a [`KmsCryptoAuthzProvider`] — there
//!    is no default-allow fallback and no Default impl.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`KmsCryptoAuthorizer`] are PORTS owned by this
//! boundary crate.  The concrete cloud-iam Cedar PDP client and the bearer/SVID
//! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
//! destination).  The port shapes model that destination so they do not change
//! at cutover; transient infra is absorbed by the adapter.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead.  Any
/// caller-asserted principal id travels alongside as a CROSS-CHECK only — never
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
/// The fields are **private**; there is no public constructor.  External crates
/// cannot build a `VerifiedKmsPrincipal` by struct literal or any public API.
/// [`VerifiedKmsPrincipal::new`] is `pub(crate)`, callable only by
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
/// comes from the *combination* of: (1) bearer/mTLS verification at the edge
/// before the body is processed, (2) the PDP authorization decision bound to the
/// verified tenant + target key, and (3) the active cross-check in
/// [`crate::authorize_cloud_kms_encrypt_from_api`] /
/// [`crate::authorize_cloud_kms_decrypt_from_api`].  This type is one layer of
/// that defense, not the sole barrier.
///
/// Within the same crate, tests use the `#[cfg(test)]` constructor
/// [`VerifiedKmsPrincipal::new_for_test`] to mint tokens without a real credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedKmsPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY — private: see unforgeability note above
    tenant_id: String,    // data_class: INTERNAL_ONLY — private: see unforgeability note above
}

impl VerifiedKmsPrincipal {
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
    /// Only available inside this crate under `#[cfg(test)]`. Integration tests
    /// (a separate crate) mint a principal by running the real
    /// [`ConfiguredBearerPrincipalVerifier`] path instead — proving external
    /// crates CANNOT forge a `VerifiedKmsPrincipal`.
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
pub enum KmsCryptoAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The crypto action a KMS authorization decision is made against. Carried
/// explicitly so the PDP sees whether the caller is requesting plaintext-revealing
/// `Decrypt` or `Encrypt`, never inferring it from a forged surface string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KmsCryptoAction {
    /// `cloud.kms.encrypt`.
    Encrypt,
    /// `cloud.kms.decrypt` (plaintext-revealing — highest sensitivity).
    Decrypt,
}

impl KmsCryptoAction {
    /// The canonical surface string the PDP keys the action on.
    #[must_use]
    pub const fn surface(self) -> &'static str {
        match self {
            Self::Encrypt => crate::CLOUD_KMS_ENCRYPT_SURFACE,
            Self::Decrypt => crate::CLOUD_KMS_DECRYPT_SURFACE,
        }
    }
}

/// The resource a KMS crypto decision is made against.
///
/// ## True blast radius / no IDOR
///
/// Every field that establishes authority — `tenant_id` and `principal_id` — is
/// bound from the VERIFIED principal, NOT from caller input.  `key_id` is the
/// TARGET key from the trusted path binding (already cross-checked equal to the
/// body key id by [`crate::validate_path_key_id`]).  Presenting the resource with
/// the caller's claimed tenant instead of the verified tenant would let tenant A
/// authorize a decrypt of tenant B's key — the cross-tenant IDOR this binding
/// prevents.  The PDP must deny when the verified tenant does not own the target
/// key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsCryptoResource {
    /// The verified tenant the principal acts within (authority source — NOT
    /// caller input). The PDP decides cross-tenant access on this axis.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// The target key id from the trusted path binding.
    pub key_id: String, // data_class: INTERNAL_ONLY
    /// The crypto action (encrypt vs. plaintext-revealing decrypt).
    pub action: KmsCryptoAction, // data_class: INTERNAL_ONLY
    /// The request data class label (purpose-binding / data-class gating input).
    pub data_class: String, // data_class: INTERNAL_ONLY
    /// The request purpose label (purpose-binding input).
    pub purpose: String, // data_class: INTERNAL_ONLY
    /// The request id, for the PDP decision/audit correlation.
    pub request_id: String, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedKmsPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination, ADR-0561). The verifier — not the headers /
/// authorization blob — is the source of truth for caller identity.
pub trait PrincipalVerifier: Send + Sync {
    /// Verify `credential` and return the authoritative principal, or refuse.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] when no credential is presented or it does
    /// not verify (fail-closed: the caller MUST treat this as 401).
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedKmsPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may perform the KMS crypto op on `resource`.
///
/// The decision is `decide(principal, action = cloud.kms.{encrypt,decrypt},
/// resource)`. Adapter: the cloud-iam Cedar PDP client (the owned W5
/// destination). The default posture is deny; any refusal is treated as deny
/// (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(KmsCryptoAuthorizationError::Refused)` so the caller can map them to
///    HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
///
/// 2. **Enforce a deadline.** This is a synchronous call on a request path;
///    a hung PDP hangs the caller thread.  Adapters MUST enforce their own
///    deadline and map expiry to `Err(Refused)`.  No deadline is enforced by
///    this port — it is the adapter's responsibility.
///
/// 3. **Do not panic.** The release profile uses `panic = "abort"` (workspace
///    `Cargo.toml [profile.release]`), so a panic in production terminates the
///    process rather than being catchable.  The [`KmsCryptoAuthzProvider`]
///    wrapper calls `catch_unwind` as a **debug/test-only best-effort** backstop
///    that works only when the panic strategy is `unwind` (i.e. in tests); it
///    MUST NOT be relied upon in production.  Adapters MUST NOT panic — use
///    `Err(Refused)` for every recoverable and unrecoverable fault.
pub trait KmsCryptoAuthorizer: Send + Sync {
    /// Authorize `principal` to perform the crypto op on `resource`, or refuse.
    ///
    /// # Errors
    /// [`KmsCryptoAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedKmsPrincipal,
        resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError>;
}

/// The authz provider the boundary depends on: a principal verifier PORT plus a
/// KMS crypto authorizer PORT. The composition root REFUSES to serve without one
/// configured (no default-allow fallback, no `Default` impl).
pub struct KmsCryptoAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn KmsCryptoAuthorizer>, // data_class: INTERNAL_ONLY
}

impl KmsCryptoAuthzProvider {
    /// Assemble the provider from a principal verifier and a crypto authorizer.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn KmsCryptoAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal. Returns the authoritative identity or a
    /// fail-closed 401-class refusal. Delegates to the [`PrincipalVerifier`]
    /// port — the headers / authorization blob are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedKmsPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the crypto resource via the PDP
    /// port. Default-deny / fail-closed.
    ///
    /// ## Panic / fault handling
    ///
    /// This wrapper calls `catch_unwind` as a **test/debug-only best-effort**
    /// backstop for panicking authorizer implementations.  In production the
    /// release profile sets `panic = "abort"`, which terminates the process
    /// immediately on panic without any unwinding — `catch_unwind` has NO effect
    /// and the process aborts.  The real fail-closed guarantee comes from the
    /// [`KmsCryptoAuthorizer`] adapter contract: adapters MUST map every fault
    /// (timeout, network error, unavailability) to `Err(Refused)` and MUST NOT
    /// panic.  The catch_unwind here catches only test panics so the integration
    /// tests can verify the `panic → 403` property without process termination.
    ///
    /// # Errors
    /// [`KmsCryptoAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedKmsPrincipal,
        resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError> {
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
        .unwrap_or(Err(KmsCryptoAuthorizationError::Refused))
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `intelligence/adapters/rest/src/lib.rs`
/// `constant_time_eq` — NEVER use a naive `==` on secret material.
///
/// **Residual:** the length of both inputs is visible from the XOR seed
/// (`a.len() ^ b.len()`), so an attacker who can probe many lengths still
/// learns whether lengths match. This is the same residual as the repo
/// reference and is accepted; in practice bearer tokens are fixed-length
/// secrets. Use a MAC (HMAC-SHA256) if length-hiding is required.
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
/// credential (e.g. a deploy-time operator token for a single known tenant)
/// or for integration tests. In multi-tenant production, every caller presents
/// a distinct credential bound to their own tenant — a single shared secret
/// cannot distinguish them, so all callers would be granted the same identity.
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
    ) -> Result<VerifiedKmsPrincipal, PrincipalVerificationError> {
        let Some(authorization) = credential.authorization.as_deref() else {
            return Err(PrincipalVerificationError::MissingCredential);
        };
        let Some(presented) = authorization.strip_prefix("Bearer ") else {
            return Err(PrincipalVerificationError::InvalidCredential);
        };
        if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
            return Err(PrincipalVerificationError::InvalidCredential);
        }
        Ok(VerifiedKmsPrincipal::new(
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
    use super::{
        AuthzProviderConfigError, CallerCredential, ConfiguredBearerPrincipalVerifier,
        KmsCryptoAction, KmsCryptoAuthorizationError, KmsCryptoAuthorizer, KmsCryptoAuthzProvider,
        KmsCryptoResource, PrincipalVerificationError, PrincipalVerifier, VerifiedKmsPrincipal,
        constant_time_eq,
    };
    use std::sync::Arc;

    const SECRET: &str = "kms-break-glass";

    /// Build the reference verifier, panicking on a config error without
    /// requiring the (secret-bearing) verifier type to implement `Debug`.
    fn verifier() -> ConfiguredBearerPrincipalVerifier {
        match ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_storage", "ten_alpha") {
            Ok(verifier) => verifier,
            Err(error) => panic!("verifier construction failed: {error}"),
        }
    }

    fn credential(authorization: Option<&str>) -> CallerCredential {
        CallerCredential {
            authorization: authorization.map(str::to_string),
            claimed_principal_id: "sp_storage".to_string(),
            claimed_tenant_id: "ten_alpha".to_string(),
        }
    }

    struct AllowAll;
    impl KmsCryptoAuthorizer for AllowAll {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedKmsPrincipal,
            _resource: &KmsCryptoResource,
        ) -> Result<(), KmsCryptoAuthorizationError> {
            Ok(())
        }
    }

    struct Panicker;
    impl KmsCryptoAuthorizer for Panicker {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedKmsPrincipal,
            _resource: &KmsCryptoResource,
        ) -> Result<(), KmsCryptoAuthorizationError> {
            panic!("boom");
        }
    }

    fn resource() -> KmsCryptoResource {
        KmsCryptoResource {
            tenant_id: "ten_alpha".to_string(),
            key_id: "kms/region-home/ten_alpha/object-key".to_string(),
            action: KmsCryptoAction::Decrypt,
            data_class: "PII_IDENTIFYING".to_string(),
            purpose: "cloud_object_storage".to_string(),
            request_id: "req".to_string(),
        }
    }

    #[test]
    fn verifier_refuses_empty_secret_at_construction() {
        // `.unwrap_err()` would require the Ok type (secret-bearing verifier) to
        // impl Debug; match on the error directly instead so the secret type
        // never gains a `Debug` impl.
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("  ", "sp_storage", "ten_alpha"),
            Err(AuthzProviderConfigError::EmptyBearerSecret)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new(SECRET, "", "ten_alpha"),
            Err(AuthzProviderConfigError::EmptyBoundIdentity)
        ));
    }

    #[test]
    fn verifier_binds_identity_from_config_not_headers() {
        let verifier = verifier();
        // The credential CLAIMS a different identity; the verifier binds from
        // its own config, never the claimed fields.
        let mut cred = credential(Some(&format!("Bearer {SECRET}")));
        cred.claimed_principal_id = "sp_attacker".to_string();
        cred.claimed_tenant_id = "ten_evil".to_string();
        let verified = verifier.verify_principal(&cred).unwrap();
        assert_eq!(verified.principal_id(), "sp_storage");
        assert_eq!(verified.tenant_id(), "ten_alpha");
    }

    #[test]
    fn verifier_rejects_missing_and_wrong_credential() {
        let verifier = verifier();
        assert_eq!(
            verifier.verify_principal(&credential(None)).unwrap_err(),
            PrincipalVerificationError::MissingCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&credential(Some("Bearer wrong")))
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&credential(Some("Basic xyz")))
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
    }

    #[test]
    fn provider_maps_panicking_authorizer_to_refused() {
        let provider = KmsCryptoAuthzProvider::new(Arc::new(verifier()), Arc::new(Panicker));
        let principal = VerifiedKmsPrincipal::new_for_test("sp_storage", "ten_alpha");
        assert_eq!(
            provider
                .ensure_authorized(&principal, &resource())
                .unwrap_err(),
            KmsCryptoAuthorizationError::Refused
        );
    }

    #[test]
    fn provider_allows_when_authorizer_allows() {
        let provider = KmsCryptoAuthzProvider::new(Arc::new(verifier()), Arc::new(AllowAll));
        let principal = VerifiedKmsPrincipal::new_for_test("sp_storage", "ten_alpha");
        assert!(provider.ensure_authorized(&principal, &resource()).is_ok());
    }

    #[test]
    fn constant_time_eq_matches_only_identical_inputs() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn action_surfaces_are_canonical() {
        assert_eq!(KmsCryptoAction::Encrypt.surface(), "cloud.kms.encrypt");
        assert_eq!(KmsCryptoAction::Decrypt.surface(), "cloud.kms.decrypt");
    }
}
