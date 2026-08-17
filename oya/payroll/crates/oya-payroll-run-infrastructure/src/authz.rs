//! Fail-closed authorization seam for the Payroll run money-mutation control
//! plane (AUTH-005 / money-CRIT class; ADR-0593).
//!
//! ## Why this module exists
//!
//! `oya-payroll-run-infrastructure` is the HTTP runtime adapter for the payroll
//! money-mutation surfaces (`POST /payroll/v1/trial-closes`,
//! `/accounting-journal-drafts`, `/hr-leave-impact-intakes`). Before this seam,
//! [`crate::payroll_runtime_chain`] returned an EMPTY `MiddlewareChain::new()`,
//! so any caller that reaches the dispatch path runs the money-mutating handlers
//! with NO verified identity and NO policy decision. The handlers then trust the
//! caller-supplied `tenant_id` carried in the request BODY — so a caller posts a
//! trial-close or journal draft for ANY tenant they name. This is the AUTH-005
//! unauthenticated / caller-supplied-authz class on a money surface (the money
//! CRITICAL finding for `oya-payroll-run`).
//!
//! This module closes that gap by mirroring the proven fail-closed doctrine that
//! landed for the Cloud KMS crypto control plane (`secrets/ports/kms-api` /
//! ADR-0573), the Cedar policy publish control plane (`iam/ports/policy-cedar-api`
//! / ADR-0572), `intelligence/adapters/rest/src/lib.rs` (`constant_time_eq`
//! bearer compare + a PDP `decide` port), and the cloud-iam PDP caller-authn
//! precedent (ADR-0561 / #38):
//!
//! 1. A real principal is VERIFIED from a credential the caller cannot forge —
//!    a bearer token compared in constant time against a configured secret (the
//!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
//!    alternate adapter). The caller-supplied `tenant_id` body field is NEVER the
//!    source of truth for identity.
//! 2. The verified principal is AUTHORIZED for
//!    `action = payroll.run.{trial_close,journal_draft,hr_leave_intake}` on the
//!    target `{tenant, operation, request_id}` via a PDP
//!    [`MoneyMutationAuthorizer`] port (`ensure_authorized`). The resource the
//!    PDP sees is bound to the VERIFIED principal's tenant — never flattened to
//!    caller input — so cross-tenant mutation is deniable.
//! 3. The boundary REFUSES to serve a money mutation without both ports running:
//!    [`crate::payroll_runtime_chain`] requires a [`PayrollAuthzProvider`] and
//!    the [`PayrollAuthzMiddleware`] short-circuits 401/403 BEFORE the handler
//!    deserializes the body. There is no default-allow fallback and no `Default`
//!    impl. The handlers additionally REJECT (403) any body whose `tenant_id`
//!    does not equal the verified tenant the middleware injected (no
//!    cross-tenant body substitution — true blast radius).
//!
//! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
//!
//! [`PrincipalVerifier`] and [`MoneyMutationAuthorizer`] are PORTS owned by this
//! adapter crate. The concrete cloud-iam Cedar PDP client and the bearer/SVID
//! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
//! destination). The port shapes model that destination so they do not change at
//! cutover; transient infra is absorbed by the adapter.

// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use oya_http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};

/// Request header the bearer credential is presented in.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// Path-captures key the middleware injects the VERIFIED tenant under so the
/// handler can cross-check it against the body-claimed tenant.
pub const VERIFIED_TENANT_CAPTURE_KEY: &str = "verified_tenant_id";

/// The credential a caller presents to prove a real principal identity.
///
/// Today this is a bearer token (constant-time compared by
/// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
/// drop-in alternate that consumes a verified peer leaf instead. Built from the
/// request `Authorization` header by the middleware — never from the body.
#[derive(Clone, Eq, PartialEq)]
pub struct CallerCredential {
    /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
    pub authorization: Option<String>, // data_class: SECRET
}

/// Custom `Debug` that redacts the `authorization` field (data_class: SECRET) so
/// bearer tokens never appear in logs, tracing spans, or panic output.
impl std::fmt::Debug for CallerCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallerCredential")
            .field("authorization", &"[REDACTED]")
            .finish()
    }
}

/// A principal whose identity has been verified from a caller credential.
///
/// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
///
/// The fields are **private**; there is no public constructor. External crates
/// cannot build a `VerifiedPrincipal` by struct literal or any public API.
/// [`VerifiedPrincipal::new`] is `pub(crate)`, callable only by
/// [`PrincipalVerifier`] implementations inside this crate. External crates must
/// obtain one by running a real [`PrincipalVerifier`] (e.g.
/// [`ConfiguredBearerPrincipalVerifier`]).
///
/// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
/// cryptographic proof. It prevents accidental struct-literal forging and proves
/// that *some* `PrincipalVerifier` ran. The real security guarantee comes from
/// the *combination* of: (1) bearer/mTLS verification at the edge before the body
/// is processed, (2) the PDP authorization decision bound to the verified tenant,
/// and (3) the handler's body-tenant cross-check.
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

    /// Test-only constructor that mints a principal without a real credential.
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
/// maps it to HTTP 401 and the request never reaches the authorizer or handler.
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
pub enum MoneyMutationAuthorizationError {
    /// The PDP returned a deny decision for this principal/action/resource.
    Denied,
    /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
    Refused,
}

/// The payroll money-mutation action a decision is made against. Carried
/// explicitly so the PDP keys on the precise operation, never inferring it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoneyMutationAction {
    /// `payroll.run.trial_close`.
    TrialClose,
    /// `payroll.run.journal_draft`.
    JournalDraft,
    /// `payroll.run.hr_leave_intake`.
    HrLeaveImpactIntake,
}

impl MoneyMutationAction {
    /// The canonical surface string the PDP keys the action on.
    #[must_use]
    pub const fn surface(self) -> &'static str {
        match self {
            Self::TrialClose => "payroll.run.trial_close",
            Self::JournalDraft => "payroll.run.journal_draft",
            Self::HrLeaveImpactIntake => "payroll.run.hr_leave_intake",
        }
    }
}

/// The resource a payroll money-mutation decision is made against.
///
/// ## True blast radius / no cross-tenant flattening
///
/// `tenant_id` is bound from the VERIFIED principal, NOT from the request body.
/// Presenting the caller's body-claimed tenant instead would let tenant A
/// authorize a payroll mutation against tenant B's books. The PDP must deny when
/// the verified tenant is not permitted for the action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoneyMutationResource {
    /// The verified tenant the principal acts within (authority source — NOT
    /// caller body input). The PDP decides cross-tenant access on this axis.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// The money-mutation action.
    pub action: MoneyMutationAction, // data_class: INTERNAL_ONLY
    /// The matched route template (for the PDP decision / audit correlation).
    pub route_template: String, // data_class: INTERNAL_ONLY
}

/// PORT: verify a caller credential into a [`VerifiedPrincipal`].
///
/// Adapters: a configured-bearer verifier (this crate's
/// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
/// verifier (the W5 destination, ADR-0561). The verifier — not the headers /
/// body — is the source of truth for caller identity.
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

/// PORT: decide whether `principal` may perform the money mutation on `resource`.
///
/// Adapter: the cloud-iam Cedar PDP client (the owned W5 destination). The
/// default posture is deny; any refusal is treated as deny (fail-closed).
///
/// ## Adapter implementation contract (MUST follow; enforcement is by convention)
///
/// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
///    parse failures, and unavailability MUST all return
///    `Err(MoneyMutationAuthorizationError::Refused)` so the caller maps them to
///    HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
/// 2. **Enforce a deadline.** This is a synchronous call on a request path; a
///    hung PDP hangs the caller thread. Adapters MUST enforce their own deadline
///    and map expiry to `Err(Refused)`.
/// 3. **Do not panic.** The release profile uses `panic = "abort"` (workspace
///    `Cargo.toml [profile.release]`), so a panic in production terminates the
///    process rather than being catchable. The [`PayrollAuthzProvider`] wrapper
///    calls `catch_unwind` as a **test/debug-only best-effort** backstop that
///    works only when the panic strategy is `unwind` (i.e. in tests); it MUST NOT
///    be relied upon in production. Adapters MUST NOT panic — use `Err(Refused)`
///    for every recoverable and unrecoverable fault.
pub trait MoneyMutationAuthorizer: Send + Sync {
    /// Authorize `principal` to perform the money mutation on `resource`, or
    /// refuse.
    ///
    /// # Errors
    /// [`MoneyMutationAuthorizationError`] on an explicit deny or any PDP fault
    /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
    /// the caller maps this to HTTP 403).
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &MoneyMutationResource,
    ) -> Result<(), MoneyMutationAuthorizationError>;
}

/// The authz provider the boundary depends on: a principal verifier PORT plus a
/// money-mutation authorizer PORT. The composition root REFUSES to serve without
/// one configured (no default-allow fallback, no `Default` impl).
#[derive(Clone)]
pub struct PayrollAuthzProvider {
    verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: Arc<dyn MoneyMutationAuthorizer>, // data_class: INTERNAL_ONLY
}

impl PayrollAuthzProvider {
    /// Assemble the provider from a principal verifier and a money authorizer.
    #[must_use]
    pub fn new(
        verifier: Arc<dyn PrincipalVerifier>,
        authorizer: Arc<dyn MoneyMutationAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    /// Verify the caller principal. Returns the authoritative identity or a
    /// fail-closed 401-class refusal. Delegates to the [`PrincipalVerifier`]
    /// port — the headers / body are never trusted as identity.
    ///
    /// # Errors
    /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    /// Authorize the verified principal for the money resource via the PDP port.
    /// Default-deny / fail-closed.
    ///
    /// ## Panic / fault handling
    ///
    /// This wrapper calls `catch_unwind` as a **test/debug-only best-effort**
    /// backstop for panicking authorizer implementations. In production the
    /// release profile sets `panic = "abort"`, so `catch_unwind` has NO effect
    /// and the process aborts. The real fail-closed guarantee comes from the
    /// [`MoneyMutationAuthorizer`] adapter contract: adapters MUST map every
    /// fault to `Err(Refused)` and MUST NOT panic.
    ///
    /// # Errors
    /// [`MoneyMutationAuthorizationError`] — caller maps to HTTP 403.
    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &MoneyMutationResource,
    ) -> Result<(), MoneyMutationAuthorizationError> {
        let authorizer = Arc::clone(&self.authorizer);
        let principal = principal.clone();
        let resource = resource.clone();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, &resource)
        }))
        .unwrap_or(Err(MoneyMutationAuthorizationError::Refused))
    }
}

/// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
/// timing-probed. Mirrors `intelligence/adapters/rest/src/lib.rs`
/// `constant_time_eq` — NEVER use a naive `==` on secret material.
///
/// **Residual:** the length of both inputs is visible from the XOR seed
/// (`a.len() ^ b.len()`); accepted, as the repo reference. Use a MAC if
/// length-hiding is required.
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
/// identity from the configured mapping (NOT from the caller headers/body).
///
/// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
///
/// This adapter binds ONE static `(principal_id, tenant_id)` pair to a single
/// shared secret. It is suitable only as a **single-principal break-glass**
/// credential or for integration tests. The production W5 adapter is the
/// cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561), which derives the
/// principal and tenant from the verified peer certificate.
///
/// Construction REFUSES an empty bearer secret or empty bound identity so a
/// process that cannot prove a credential root can never authenticate a caller
/// (mirrors the cloud-pdp boot-refusal doctrine).
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

/// Map a matched route template to the money-mutation action it gates. `None`
/// for non-mutation routes (e.g. health), which the middleware passes through.
#[must_use]
pub fn action_for_template(template: &str) -> Option<MoneyMutationAction> {
    match template {
        crate::PAYROLL_TRIAL_CLOSE_PATH => Some(MoneyMutationAction::TrialClose),
        crate::PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH => Some(MoneyMutationAction::JournalDraft),
        crate::PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH => {
            Some(MoneyMutationAction::HrLeaveImpactIntake)
        }
        _ => None,
    }
}

/// AUTHN-BEFORE-BODY middleware: verifies the caller principal and runs the PDP
/// decision on money-mutation routes BEFORE the terminal handler deserializes
/// the body. Short-circuits 401 (unauthenticated) / 403 (unauthorized or PDP
/// fault) fail-closed. Non-mutation routes (health) pass through unauthenticated.
///
/// On success it injects the VERIFIED tenant into `path_captures` under
/// [`VERIFIED_TENANT_CAPTURE_KEY`] so the handler can reject any body whose
/// `tenant_id` does not match (no cross-tenant body substitution).
pub struct PayrollAuthzMiddleware {
    provider: PayrollAuthzProvider,
}

impl PayrollAuthzMiddleware {
    #[must_use]
    pub fn new(provider: PayrollAuthzProvider) -> Self {
        Self { provider }
    }

    fn unauthorized_401() -> HttpResponse {
        HttpResponse::new(401)
            .with_header("content-type", "application/json")
            .with_header("www-authenticate", "Bearer")
            .with_body(
                br#"{"error":{"code":"UNAUTHENTICATED","message":"payroll money mutation requires a verified principal"}}"#
                    .to_vec(),
            )
    }

    fn forbidden_403() -> HttpResponse {
        HttpResponse::new(403)
            .with_header("content-type", "application/json")
            .with_body(
                br#"{"error":{"code":"FORBIDDEN","message":"principal not authorized for this payroll money mutation"}}"#
                    .to_vec(),
            )
    }
}

impl Middleware<HttpRequest, HttpResponse> for PayrollAuthzMiddleware {
    fn handle(
        &self,
        mut request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        // The matched template is the trusted route binding set by the router at
        // dispatch. Fall back to the raw path only when no template was matched.
        let template = request
            .matched_template
            .clone()
            .unwrap_or_else(|| request.path.clone());

        // Non-mutation routes (health) need no authz.
        let Some(action) = action_for_template(&template) else {
            return next.run(request);
        };

        // 1. AUTHN: verify the caller principal from the Authorization header,
        //    BEFORE the handler deserializes the body. Missing/invalid -> 401.
        let credential = CallerCredential {
            authorization: request.headers.get(AUTHORIZATION_HEADER).cloned(),
        };
        let principal = match self.provider.verify_principal(&credential) {
            Ok(principal) => principal,
            Err(_) => return Self::unauthorized_401(),
        };

        // 2. AUTHZ: PDP decision bound to the VERIFIED tenant (not body input).
        //    Deny / fault -> 403 (fail-closed).
        let resource = MoneyMutationResource {
            tenant_id: principal.tenant_id().to_owned(),
            action,
            route_template: template,
        };
        if self
            .provider
            .ensure_authorized(&principal, &resource)
            .is_err()
        {
            return Self::forbidden_403();
        }

        // 3. Inject the verified tenant so the handler rejects any body whose
        //    tenant_id does not match (true blast radius / no IDOR).
        request.path_captures.insert(
            VERIFIED_TENANT_CAPTURE_KEY.to_owned(),
            principal.tenant_id().to_owned(),
        );
        next.run(request)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConfiguredBearerPrincipalVerifier, MoneyMutationAction, MoneyMutationAuthorizationError,
        MoneyMutationAuthorizer, MoneyMutationResource, PrincipalVerificationError,
        PrincipalVerifier, VerifiedPrincipal, action_for_template, constant_time_eq,
    };
    use crate::PayrollAuthzProvider;
    use std::sync::Arc;

    const SECRET: &str = "payroll-break-glass";

    fn verifier() -> ConfiguredBearerPrincipalVerifier {
        match ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_payroll", "ten_acme") {
            Ok(verifier) => verifier,
            Err(error) => panic!("verifier construction failed: {error}"),
        }
    }

    fn credential(authorization: Option<&str>) -> super::CallerCredential {
        super::CallerCredential {
            authorization: authorization.map(str::to_string),
        }
    }

    struct AllowAll;
    impl MoneyMutationAuthorizer for AllowAll {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedPrincipal,
            _resource: &MoneyMutationResource,
        ) -> Result<(), MoneyMutationAuthorizationError> {
            Ok(())
        }
    }

    struct Panicker;
    impl MoneyMutationAuthorizer for Panicker {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedPrincipal,
            _resource: &MoneyMutationResource,
        ) -> Result<(), MoneyMutationAuthorizationError> {
            panic!("boom");
        }
    }

    fn resource() -> MoneyMutationResource {
        MoneyMutationResource {
            tenant_id: "ten_acme".to_string(),
            action: MoneyMutationAction::TrialClose,
            route_template: crate::PAYROLL_TRIAL_CLOSE_PATH.to_string(),
        }
    }

    #[test]
    fn verifier_refuses_empty_secret_at_construction() {
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new("  ", "sp_payroll", "ten_acme"),
            Err(super::AuthzProviderConfigError::EmptyBearerSecret)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new(SECRET, "", "ten_acme"),
            Err(super::AuthzProviderConfigError::EmptyBoundIdentity)
        ));
    }

    #[test]
    fn verifier_binds_identity_from_config_not_input() {
        let verifier = verifier();
        let verified = verifier
            .verify_principal(&credential(Some(&format!("Bearer {SECRET}"))))
            .unwrap();
        assert_eq!(verified.principal_id(), "sp_payroll");
        assert_eq!(verified.tenant_id(), "ten_acme");
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
        let provider = PayrollAuthzProvider::new(Arc::new(verifier()), Arc::new(Panicker));
        let principal = VerifiedPrincipal::new_for_test("sp_payroll", "ten_acme");
        assert_eq!(
            provider
                .ensure_authorized(&principal, &resource())
                .unwrap_err(),
            MoneyMutationAuthorizationError::Refused
        );
    }

    #[test]
    fn provider_allows_when_authorizer_allows() {
        let provider = PayrollAuthzProvider::new(Arc::new(verifier()), Arc::new(AllowAll));
        let principal = VerifiedPrincipal::new_for_test("sp_payroll", "ten_acme");
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
        assert_eq!(
            MoneyMutationAction::TrialClose.surface(),
            "payroll.run.trial_close"
        );
        assert_eq!(
            MoneyMutationAction::JournalDraft.surface(),
            "payroll.run.journal_draft"
        );
        assert_eq!(
            MoneyMutationAction::HrLeaveImpactIntake.surface(),
            "payroll.run.hr_leave_intake"
        );
    }

    #[test]
    fn action_for_template_only_maps_mutation_routes() {
        assert_eq!(
            action_for_template(crate::PAYROLL_TRIAL_CLOSE_PATH),
            Some(MoneyMutationAction::TrialClose)
        );
        assert_eq!(
            action_for_template(crate::PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH),
            Some(MoneyMutationAction::JournalDraft)
        );
        assert_eq!(
            action_for_template(crate::PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH),
            Some(MoneyMutationAction::HrLeaveImpactIntake)
        );
        assert_eq!(action_for_template(crate::PAYROLL_HEALTH_PATH), None);
    }
}
