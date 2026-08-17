//! Fail-closed authorization seam for the CRM revenue control plane
//! (AUTH-005 class; ADR-0603).
//!
//! ## Why this module exists
//!
//! The CRM REST/gRPC/AsyncAPI adapters (`adapter::http`, `adapter::grpc`,
//! `adapter::asyncapi`) are MUTATING multi-tenant control planes. Before this
//! seam the request DTOs (`HttpRequest`, `GrpcRequest`, `AsyncApiMessage`)
//! carried a caller-supplied `tenant_id` / `principal_id` straight into the
//! [`crate::usecase::UsecaseContext`] actor, and `PolicyPort::authorize` then
//! authorized against that caller-supplied identity. An attacker who can reach
//! the socket sets `tenant_id` to any victim tenant and mutates their CRM
//! records — forge identity → cross-tenant CRM mutation (the AUTH-005 class
//! that PR #768 shipped and the #780 authz-coverage gate baselined as debt).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine in
//! `iam/ports/policy-cedar-api/src/authz.rs` (ADR-0572 / #815) and
//! `intelligence/adapters/rest/src/lib.rs`:
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a
//!    drop-in alternate adapter). The body-supplied `principal_id` / `tenant_id`
//!    are NEVER the source of truth and are NEVER read by the gate:
//!    [`CallerCredential`] carries only the unforgeable credential, and
//!    [`authorize_crm_command`] takes no request body, so a forged body claim is
//!    structurally ignored — it can be neither an authz input nor the resource
//!    tenant.
//! 2. The verified principal is AUTHORIZED for the CRM action
//!    (`crm.<capability>.mutate`) on the TARGET tenant via a Cedar PDP
//!    [`CrmAuthorizer`] port (`decide`). The tenant axis is asserted by the
//!    decision against the target tenant taken from a TRUSTED source (the
//!    verified principal or a routing binding) — a verified principal alone
//!    never grants a tenant, and the caller body never selects the resource
//!    tenant (true blast-radius).
//! 3. The adapter REFUSES TO SERVE a mutation without both ports configured
//!    (no default-allow fallback): see [`authorize_crm_command`].
//!
//! ## Live-serving status (edge obligation)
//!
//! Today `HttpHandler::handle` / `GrpcHandler::handle` / `AsyncApiHandler::handle`
//! return `contract_stub` — there is no bound socket yet (dead-until-edge). This
//! module installs the unforgeable seam so that WHEN the edge is wired the body
//! `tenant_id` / `principal_id` grant nothing. The edge that binds a real
//! listener MUST, before this seam runs:
//!   * extract the bearer/SVID credential in transport middleware
//!     (`route_layer` / `FromRequestParts`) BEFORE body deserialization, and
//!   * install `DefaultBodyLimit`, and
//!   * refuse to boot without a [`PrincipalVerifier`] + [`CrmAuthorizer`]
//!     configured (mirror the cloud-pdp boot-refusal doctrine), and
//!   * enter `usecase::ServiceInteractor` ONLY with an actor derived from the
//!     verified principal in the [`AuthorizedCrmContext`] this gate returns
//!     (via `AuthorizedCrmContext::tenant_id` / `::principal_id`).
//!     `usecase::UsecaseContext` currently builds its actor from `Deserialize`
//!     (caller-supplied) — the un-gated AUTH-005 residual (ADR-0603), not
//!     currently reachable (no live caller, no bound socket). The edge MUST NOT route an adapter
//!     into `submit_command` with a caller-built actor; bind tenant/principal
//!     from the verified principal instead.
//!
//! See `adapter::http` / `adapter::grpc` module docs for the seam call sites.
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`CrmAuthorizer`] are PORTS owned by this crate.
//! The concrete cloud-iam PDP client and the bearer/SVID credential store are
//! ADAPTERS that live OUTSIDE this crate (the owned W5 destination). The port
//! shapes model that destination so they do not change at cutover; transient
//! infra is absorbed by the adapter.

use crate::domain::Capability;

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead. Built from the
/// transport `Authorization` header by the edge middleware — NEVER from the body.
///
/// It deliberately carries ONLY the unforgeable credential. There is no
/// caller-asserted `principal_id` / `tenant_id` field: a free-floating body
/// claim must never look like an authz input, and the verified identity is the
/// sole source of truth (see [`authorize_crm_command`] / [`AuthorizedCrmContext`]).
#[derive(Clone, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
}

/// Custom `Debug` that REDACTS the `authorization` field (data_class: SECRET) so
/// the live bearer token never appears in logs, tracing spans, or panic output.
/// Mirrors the redacting pattern on `ConfiguredBearerPrincipalVerifier` and the
/// payroll adapter's `CallerCredential` (#133 secret-Debug class).
impl std::fmt::Debug for CallerCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallerCredential")
            .field("authorization", &"<redacted>")
            .finish()
    }
}

/// A principal whose identity has been verified from a caller credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; there is no public constructor — external crates
/// cannot build a `VerifiedPrincipal` by struct literal or any public API.
/// [`VerifiedPrincipal::new`] is `pub(crate)`, callable only by
/// [`PrincipalVerifier`] implementations inside this crate. External crates
/// must obtain one by running a real [`PrincipalVerifier`].
///
/// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
/// cryptographic proof. It prevents accidental struct-literal forging and
/// proves that *some* `PrincipalVerifier` ran. It does NOT stop hostile
/// in-process code from constructing its own
/// [`ConfiguredBearerPrincipalVerifier`] with a known secret. The real security
/// guarantee comes from the *combination* of: (1) credential verification
/// before body deserialization at the edge, (2) the PDP authorization decision
/// against the trusted target tenant, and (3) refusing to serve without both
/// ports. This type is one layer of that defense, not the sole barrier.
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
pub enum CrmAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The CRM action a decision is made against, derived from the route capability
/// (server-side route metadata, never the caller body). Maps to a Cedar action
/// id of the form `crm.<capability>.mutate`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrmAction(pub Capability);

impl CrmAction {
    /// The Cedar action id (e.g. `crm.account-master.mutate`).
    #[must_use]
    pub fn action_id(&self) -> String {
        let cap = match self.0 {
            Capability::AccountMaster => "account-master",
            Capability::Opportunity => "opportunity",
            Capability::Quote => "quote",
            Capability::Campaign => "campaign",
            Capability::ServiceCase => "service-case",
        };
        format!("crm.{cap}.mutate")
    }
}

/// The resource a CRM decision is made against: the capability and the TARGET
/// tenant. The target tenant is bound from a TRUSTED source (the verified
/// principal, or a routing binding) — NEVER from the caller body — so the PDP
/// sees the true blast radius. The tenant axis is asserted by the authorizer; a
/// verified principal alone never grants the tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrmResource {
    /// The CRM capability being mutated (from server-side route metadata).
    pub action: CrmAction, // data_class: INTERNAL_ONLY
    /// The tenant whose CRM records the mutation lands in, from a TRUSTED
    /// source. The caller body never selects this.
    pub target_tenant_id: String, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination, ADR-0561). The verifier — not the body — is
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

/// PORT: decide whether `principal` may perform the CRM action on `resource`.
///
/// The decision is `decide(principal, action, resource)`. Adapter: the
/// cloud-iam Cedar PDP client (the owned W5 destination). The default posture
/// is deny; any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(CrmAuthorizationError::Refused)` so the caller maps them to HTTP 403
///    (fail-closed). Never propagate an internal error as `Ok(())`.
/// 2. **Enforce a deadline.** A hung PDP must not hang the request thread; map
///    expiry to `Err(Refused)`.
/// 3. **Do not panic.** Use `Err(Refused)` for every recoverable and
///    unrecoverable fault.
pub trait CrmAuthorizer: Send + Sync {
    /// Authorize `principal` for `resource`, or refuse.
    ///
    /// # Errors
    /// [`CrmAuthorizationError`] on an explicit deny or any PDP fault (timeout,
    /// network, unavailability — all MUST be `Refused`; fail-closed: the caller
    /// maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &CrmResource,
    ) -> Result<(), CrmAuthorizationError>;
}

/// The authz provider the adapters depend on: a principal verifier PORT plus a
/// CRM authorizer PORT. A mutation REFUSES to proceed without one configured
/// (no default-allow fallback): see [`authorize_crm_command`].
pub struct CrmAuthzProvider {
    verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: std::sync::Arc<dyn CrmAuthorizer>,   // data_class: INTERNAL_ONLY
}

impl CrmAuthzProvider {
    /// Assemble the provider from a principal verifier and a CRM authorizer.
    #[must_use]
    pub fn new(
        verifier: std::sync::Arc<dyn PrincipalVerifier>,
        authorizer: std::sync::Arc<dyn CrmAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal. Returns the authoritative identity or a
    /// fail-closed 401-class refusal. The body is never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the CRM resource via the PDP port.
    /// Default-deny / fail-closed.
    ///
    /// # Errors
    /// [`CrmAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &CrmResource,
    ) -> Result<(), CrmAuthorizationError> {
        self.authorizer.ensure_authorized(principal, resource)
    }
}

/// The proof a handler holds AFTER the fail-closed gate has run: the verified
/// principal and the authorized action, nothing else. It is the ONLY value an
/// adapter may use to derive the resource/scope tenant downstream.
///
/// ## Why this type exists (the cross-tenant invariant)
///
/// The request DTOs (`HttpRequest`, `GrpcRequest`, `AsyncApiMessage`) still carry
/// a caller-supplied body `tenant_id` / `principal_id`. If a handler ever read
/// the body tenant as the resource tenant, a valid caller could forge a victim
/// tenant in the body and mutate the victim's records while authorized as their
/// own tenant. To make that STRUCTURALLY impossible, the gate does not return a
/// bare principal alongside the still-readable body; it returns this context,
/// and the adapters bind the resource scope from [`Self::tenant_id`] only. The
/// body tenant can therefore never be the authz basis NOR the resource tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedCrmContext {
    principal: VerifiedPrincipal,
    action: CrmAction,
}

impl AuthorizedCrmContext {
    /// The verified principal that passed the gate.
    #[must_use]
    pub fn principal(&self) -> &VerifiedPrincipal {
        &self.principal
    }

    /// The authorized CRM action (from server-side route metadata).
    #[must_use]
    pub fn action(&self) -> CrmAction {
        self.action
    }

    /// The VERIFIED tenant — the ONLY legitimate resource/scope tenant. Adapters
    /// MUST bind the resource tenant from this, never from the request body.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        self.principal.tenant_id()
    }

    /// The VERIFIED principal id — the ONLY legitimate actor identity.
    #[must_use]
    pub fn principal_id(&self) -> &str {
        self.principal.principal_id()
    }
}

/// Drive the full fail-closed gate for one CRM mutation: verify the caller
/// credential, bind the target tenant from the TRUSTED verified principal, then
/// authorize the action against that tenant via the PDP. Returns an
/// [`AuthorizedCrmContext`] on success; any failure is fail-closed (401 on
/// verification, 403 on authorization).
///
/// The request body is NOT a parameter and is NEVER consulted: the resource
/// tenant is bound solely from the verified principal, so a forged body tenant
/// is structurally ignored — it cannot be an authz input nor the resource scope.
///
/// # Errors
/// [`CrmGateError`] — `Unauthenticated` maps to 401, `Unauthorized` to 403.
pub fn authorize_crm_command(
    provider: &CrmAuthzProvider,
    credential: &CallerCredential,
    action: CrmAction,
) -> Result<AuthorizedCrmContext, CrmGateError> {
    let principal = provider
        .verify_principal(credential)
        .map_err(CrmGateError::Unauthenticated)?;

    let resource = CrmResource {
        action,
        target_tenant_id: principal.tenant_id().to_string(),
    };
    provider
        .ensure_authorized(&principal, &resource)
        .map_err(CrmGateError::Unauthorized)?;
    Ok(AuthorizedCrmContext { principal, action })
}

/// The outcome of the fail-closed CRM gate. `Unauthenticated` → 401,
/// `Unauthorized` → 403.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrmGateError {
    /// Credential verification failed (401).
    Unauthenticated(PrincipalVerificationError),
    /// The verified principal is not permitted (403).
    Unauthorized(CrmAuthorizationError),
}

impl CrmGateError {
    /// The HTTP status this gate failure maps to.
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Unauthenticated(_) => 401,
            Self::Unauthorized(_) => 403,
        }
    }
}

impl From<CrmGateError> for crate::error::ServiceError {
    /// Map the fail-closed gate failure to a service error whose KIND carries the
    /// 401-vs-403 distinction STRUCTURALLY (`Unauthenticated` / `Forbidden`), so
    /// the edge maps the HTTP status from the kind, not by matching the message
    /// string. The message stays deliberately coarse (no "wrong token" vs "no
    /// principal" distinction) so probing cannot fingerprint the failure mode.
    fn from(error: CrmGateError) -> Self {
        match error {
            CrmGateError::Unauthenticated(_) => {
                crate::error::ServiceError::unauthenticated("authorization", "unauthenticated")
            }
            CrmGateError::Unauthorized(_) => {
                crate::error::ServiceError::forbidden("authorization", "forbidden")
            }
        }
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `iam/ports/policy-cedar-api/src/authz.rs`
/// `constant_time_eq` — NEVER use a naive `==` on secret material.
///
/// **Residual:** the length of both inputs is visible from the XOR seed
/// (`a.len() ^ b.len()`). Same residual as the repo reference and accepted; in
/// practice bearer tokens are fixed-length secrets.
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
/// identity from the configured mapping (NOT from the caller body).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// This adapter binds ONE static `(principal_id, tenant_id)` pair to a single
/// shared secret. It is suitable only as a single-principal break-glass
/// credential or for integration tests. The production W5 adapter is the
/// cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561), which derives the
/// principal and tenant from the verified peer certificate.
///
/// Construction REFUSES an empty bearer secret or empty bound identity so a
/// process that cannot prove a credential root can never authenticate a caller.
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,      // data_class: SECRET
    bound_principal_id: String, // data_class: INTERNAL_ONLY
    bound_tenant_id: String,    // data_class: INTERNAL_ONLY
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
/// MUST refuse to serve, mirroring the cloud-pdp boot-refusal.
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

    /// A PDP adapter that always grants — for the GREEN path only.
    struct AllowAuthorizer;
    impl CrmAuthorizer for AllowAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _r: &CrmResource,
        ) -> Result<(), CrmAuthorizationError> {
            Ok(())
        }
    }

    /// A PDP adapter that always denies.
    struct DenyAuthorizer;
    impl CrmAuthorizer for DenyAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _r: &CrmResource,
        ) -> Result<(), CrmAuthorizationError> {
            Err(CrmAuthorizationError::Denied)
        }
    }

    /// A PDP adapter that faults (must be treated as deny / 403).
    struct FaultAuthorizer;
    impl CrmAuthorizer for FaultAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _r: &CrmResource,
        ) -> Result<(), CrmAuthorizationError> {
            Err(CrmAuthorizationError::Refused)
        }
    }

    fn bearer_provider(authorizer: std::sync::Arc<dyn CrmAuthorizer>) -> CrmAuthzProvider {
        let verifier = ConfiguredBearerPrincipalVerifier::new(
            "s3cr3t-bearer-token",
            "svc-crm-op",
            "tenant-alpha",
        )
        .expect("verifier builds");
        CrmAuthzProvider::new(std::sync::Arc::new(verifier), authorizer)
    }

    fn credential(auth: Option<&str>) -> CallerCredential {
        CallerCredential {
            authorization: auth.map(str::to_string),
        }
    }

    // RED: forged identity (no/invalid credential) is rejected 401, never reaches PDP.
    #[test]
    fn forged_request_without_credential_is_401() {
        let provider = bearer_provider(std::sync::Arc::new(AllowAuthorizer));
        let err = authorize_crm_command(
            &provider,
            &credential(None),
            CrmAction(Capability::AccountMaster),
        )
        .unwrap_err();
        assert_eq!(err.http_status(), 401);
    }

    #[test]
    fn forged_request_with_bad_bearer_is_401() {
        let provider = bearer_provider(std::sync::Arc::new(AllowAuthorizer));
        let err = authorize_crm_command(
            &provider,
            &credential(Some("Bearer wrong-token")),
            CrmAction(Capability::Opportunity),
        )
        .unwrap_err();
        assert_eq!(err.http_status(), 401);
    }

    // Invariant: the PDP resource tenant is ALWAYS the verified tenant. The gate
    // takes no body, so there is no body tenant that could ever bind the resource.
    #[test]
    fn resource_tenant_is_always_the_verified_tenant() {
        struct CaptureAuthorizer(std::sync::Mutex<Option<String>>);
        impl CrmAuthorizer for CaptureAuthorizer {
            fn ensure_authorized(
                &self,
                _p: &VerifiedPrincipal,
                r: &CrmResource,
            ) -> Result<(), CrmAuthorizationError> {
                *self.0.lock().unwrap() = Some(r.target_tenant_id.clone());
                Ok(())
            }
        }
        let capture = std::sync::Arc::new(CaptureAuthorizer(std::sync::Mutex::new(None)));
        let provider = bearer_provider(capture.clone());
        let ctx = authorize_crm_command(
            &provider,
            &credential(Some("Bearer s3cr3t-bearer-token")),
            CrmAction(Capability::Quote),
        )
        .expect("authorized");
        assert_eq!(capture.0.lock().unwrap().as_deref(), Some("tenant-alpha"));
        // The returned context's tenant is the verified tenant — the only legitimate scope.
        assert_eq!(ctx.tenant_id(), "tenant-alpha");
    }

    // RED: PDP deny → 403.
    #[test]
    fn pdp_deny_is_403() {
        let provider = bearer_provider(std::sync::Arc::new(DenyAuthorizer));
        let err = authorize_crm_command(
            &provider,
            &credential(Some("Bearer s3cr3t-bearer-token")),
            CrmAction(Capability::Campaign),
        )
        .unwrap_err();
        assert_eq!(err.http_status(), 403);
    }

    // RED: PDP fault must be treated as deny → 403 (fail-closed).
    #[test]
    fn pdp_fault_is_403_fail_closed() {
        let provider = bearer_provider(std::sync::Arc::new(FaultAuthorizer));
        let err = authorize_crm_command(
            &provider,
            &credential(Some("Bearer s3cr3t-bearer-token")),
            CrmAction(Capability::ServiceCase),
        )
        .unwrap_err();
        assert_eq!(err.http_status(), 403);
    }

    // GREEN: valid credential + PDP grant → authorized; context bound to the VERIFIED identity.
    #[test]
    fn properly_authorized_request_succeeds_on_verified_tenant() {
        let provider = bearer_provider(std::sync::Arc::new(AllowAuthorizer));
        let ctx = authorize_crm_command(
            &provider,
            &credential(Some("Bearer s3cr3t-bearer-token")),
            CrmAction(Capability::AccountMaster),
        )
        .expect("authorized");
        assert_eq!(ctx.tenant_id(), "tenant-alpha");
        // The bound identity comes from the verifier, NOT any caller-supplied value.
        assert_eq!(ctx.principal_id(), "svc-crm-op");
    }

    // The redacting Debug must never print the live bearer token.
    #[test]
    fn caller_credential_debug_redacts_bearer() {
        let cred = credential(Some("Bearer s3cr3t-bearer-token"));
        let rendered = format!("{cred:?}");
        assert!(rendered.contains("<redacted>"), "{rendered}");
        assert!(!rendered.contains("s3cr3t-bearer-token"), "{rendered}");
    }

    // 401 and 403 map to DISTINCT ServiceError kinds, not one Authorization kind.
    #[test]
    fn gate_error_maps_to_distinct_service_error_kinds() {
        use crate::error::ServiceErrorKind;
        let unauth: crate::error::ServiceError =
            CrmGateError::Unauthenticated(PrincipalVerificationError::MissingCredential).into();
        let forbid: crate::error::ServiceError =
            CrmGateError::Unauthorized(CrmAuthorizationError::Denied).into();
        assert_eq!(unauth.kind(), ServiceErrorKind::Unauthenticated);
        assert_eq!(unauth.http_status(), 401);
        assert_eq!(forbid.kind(), ServiceErrorKind::Forbidden);
        assert_eq!(forbid.http_status(), 403);
    }

    #[test]
    fn verifier_refuses_empty_secret() {
        // ConfiguredBearerPrincipalVerifier holds a SECRET and deliberately has
        // no Debug impl, so match on the Err rather than `unwrap_err`.
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("", "p", "t"),
            Err(AuthzProviderConfigError::EmptyBearerSecret)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("s", "", "t"),
            Err(AuthzProviderConfigError::EmptyBoundIdentity)
        ));
    }

    #[test]
    fn action_id_maps_capability() {
        assert_eq!(
            CrmAction(Capability::AccountMaster).action_id(),
            "crm.account-master.mutate"
        );
        assert_eq!(
            CrmAction(Capability::ServiceCase).action_id(),
            "crm.service-case.mutate"
        );
    }

    #[test]
    fn verified_principal_has_no_public_constructor() {
        // Compile-time guarantee: external crates cannot build VerifiedPrincipal.
        // Within this crate we can only mint via the verifier or new_for_test.
        let p = VerifiedPrincipal::new_for_test("p", "t");
        assert_eq!(p.tenant_id(), "t");
    }
}
