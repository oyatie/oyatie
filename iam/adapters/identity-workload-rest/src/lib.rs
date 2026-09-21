//! Workload-identity REST surface (ADR-0105 Layer 5).
//!
//! The I/O-bearing axum app for the workload-identity service. It mounts the
//! inward use-case core ([`iam_identity_workload_app`]) behind the HTTP
//! contract promised by `iam/identity/workload-identity/PRD.md` §1.2
//! and serialized in `iam/identity/contracts/openapi/workload.yaml`:
//!
//! | Method + path                       | Use-case                                  |
//! |-------------------------------------|-------------------------------------------|
//! | `POST /authorize`                   | authorize an already-verified principal   |
//! | `POST /authorize:batch`             | authorize a batch of token requests       |
//! | `POST /authorize-with-token`        | validate-then-authorize a raw workload JWT|
//! | `POST /tokens/validate`             | validate a raw workload JWT               |
//! | `POST /principals/{id}:suspend`     | suspend (revoke) a principal              |
//! | `POST /principals/{id}:retire`      | retire (terminal) a principal             |
//!
//! ## Fail-closed PEP status mapping (PRD §3.4/§3.5/§5)
//!
//! This service is a Policy Enforcement Point. Every error path denies:
//! - an authorization **deny** is `403 Forbidden` — *never* a `404` (a deny
//!   must not be downgraded to "not found", which would leak existence and
//!   weaken default-deny);
//! - a **token validation failure** is `422 Unprocessable Entity` (the request
//!   was well-formed JSON but the credential inside it did not validate); the
//!   policy engine is never consulted;
//! - a **store / JWKS unavailable** condition is `503 Service Unavailable` and
//!   is treated as a hard deny (the PEP fails closed, never open).
//!
//! ## Audit emission (PRD §3.3 / AC-W-13)
//!
//! Every authorize call and every token-validation emits exactly one immutable
//! [`AuditRecord`] through the [`AuditSink`] port before the response is
//! returned, so the decision (and the stage a deny fail-closed at) is on the
//! audit chain regardless of outcome. Service-local guards that authorize
//! through [`WorkloadAuthzState::authorize_token_for`] (e.g. the SCIM
//! provisioning surface) get the same per-decision emission structurally —
//! the choke point emits, so a delivery surface cannot forget to.
//!
//! ## Layering invariant (ADR-0131 / architecture-boundaries gate)
//!
//! `rest` ring: depends inward on `app` (use-cases), `api` (DTOs), `domain`,
//! and the two adapters. No inner layer depends back out.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); tests
// may use unwrap/expect/panic under the cfg(test) exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod grpc;

use std::sync::{Arc, Mutex};

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;

use iam_identity_workload_api::{
    ApiErrorEnvelope, AuthorizeRequest, AuthorizeResponse, AuthorizeWithTokenRequest,
    BatchAuthorizeRequest, BatchAuthorizeResponse, PrincipalLifecycleResponse,
    ValidateTokenRequest, ValidateTokenResponse,
};
use iam_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token,
    retire, suspend,
};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, Resource, WorkloadId,
    WorkloadPrincipal, WorkloadState,
};
use iam_identity_workload_oidc::{Jwks, ValidationConfig, validate_workload_token};

// =====================================================================
// Route constants (also consumed by the openapi-rest-route-parity gate)
// =====================================================================

/// `POST` — authorize an already-verified principal.
pub const AUTHORIZE_ROUTE: &str = "/authorize";
/// `POST` — authorize a batch of token-bearing requests.
pub const AUTHORIZE_BATCH_ROUTE: &str = "/authorize:batch";
/// `POST` — validate a raw workload JWT, then authorize.
pub const AUTHORIZE_WITH_TOKEN_ROUTE: &str = "/authorize-with-token";
/// `POST` — validate a raw workload JWT (no authorization).
pub const TOKENS_VALIDATE_ROUTE: &str = "/tokens/validate";
/// `POST` — suspend (revoke) a principal by id. The documented contract path
/// (an AIP-136 custom method); the colon-suffix shape is also what the OpenAPI
/// surface declares.
pub const PRINCIPAL_SUSPEND_ROUTE: &str = "/principals/{id}:suspend";
/// `POST` — retire (terminal) a principal by id (AIP-136 custom method).
pub const PRINCIPAL_RETIRE_ROUTE: &str = "/principals/{id}:retire";

/// The axum/matchit registration pattern backing both lifecycle custom methods.
/// matchit forbids a partial-segment param (`{id}:suspend`), so the whole final
/// segment — `<id>:<verb>` — is captured in one param and split in the handler.
/// The two public `*_ROUTE` constants above remain the documented/OpenAPI shape.
const PRINCIPAL_LIFECYCLE_PATTERN: &str = "/principals/{id_and_verb}";

// =====================================================================
// Caller authorization seam (AUTH-005 fail-closed control plane)
// =====================================================================
//
// The two mutating lifecycle routes (`:suspend`, `:retire`) are a control plane:
// without a guard, any caller who reaches the socket can revoke/terminate any
// principal. ADR-0581 closes that by requiring, for every lifecycle mutation,
// BOTH a verified UNFORGEABLE caller credential (failure-class (b)) AND a
// fail-closed PDP authorization decision scoped to the TARGET principal's real
// tenant (failure-class (d)). Both are clean PORTS owned by this boundary crate;
// the concrete iam PDP client + credential store are adapters outside it
// (owned-W5 shape). The guard is enforced at the in-crate choke point
// [`lifecycle_transition`], not only at the HTTP edge (failure-class (c)).

/// A caller whose credential the [`CallerVerifier`] has VERIFIED. The inner
/// fields are private and there is no public constructor, so a `VerifiedCaller`
/// can ONLY be minted by a verifier that proved an unforgeable credential — a
/// handler cannot fabricate one from caller-supplied headers (failure-class (b)).
/// It is the type-level proof that authn ran before any mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCaller {
    /// The tenant the verified caller acts within (derived from the credential,
    /// NEVER from a caller-supplied `x-*` header).
    caller_tenant: String, // data_class: INTERNAL_ONLY
    /// A stable identity label for the caller (e.g. the credential subject), for
    /// the PDP request + audit detail.
    caller_id: String, // data_class: INTERNAL_ONLY
}

impl VerifiedCaller {
    /// The verified caller's tenant.
    #[must_use]
    pub fn caller_tenant(&self) -> &str {
        &self.caller_tenant
    }

    /// The verified caller's identity label.
    #[must_use]
    pub fn caller_id(&self) -> &str {
        &self.caller_id
    }
}

/// Caller-authentication PORT: derive a [`VerifiedCaller`] from the request
/// headers by checking an UNFORGEABLE credential (a constant-time bearer compare,
/// or — in a production adapter — mTLS/SPIFFE peer identity). Returns `None` when
/// no valid credential is present: the handler maps that to `401` (default-deny).
/// Caller-supplied `x-principal-*` / `x-authorization-*` headers MUST NOT
/// authorize — only a verified credential mints a [`VerifiedCaller`].
pub trait CallerVerifier: Send + Sync {
    /// Verify the caller's credential. `None` ⇒ no verified principal ⇒ `401`.
    fn verify_principal(&self, headers: &HeaderMap) -> Option<VerifiedCaller>;
}

/// Which lifecycle mutation a [`LifecycleAuthorizer`] is being asked to permit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleAction {
    /// `POST /principals/{id}:suspend`.
    Suspend,
    /// `POST /principals/{id}:retire`.
    Retire,
}

impl LifecycleAction {
    /// Stable wire label (mirrors the Cedar action names).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suspend => "identity.workload.Suspend",
            Self::Retire => "identity.workload.Retire",
        }
    }
}

/// A fully-bound lifecycle authorization request handed to the PDP. The
/// `caller_tenant` is the verified caller's tenant; the `target_tenant` is the
/// TARGET principal's real tenant, derived from the trusted store after loading
/// it (NEVER the caller's own tenant, NEVER a header). A PDP that enforces
/// tenant isolation can therefore DENY a cross-tenant suspend/retire (caller in
/// tenant A retiring tenant B's principal) — closing the IDOR/blast-radius axis
/// (failure-class (d)).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleAuthzRequest<'a> {
    /// Verified caller's tenant (trusted).
    pub caller_tenant: &'a str,
    /// Verified caller's identity label (trusted).
    pub caller_id: &'a str,
    /// The mutation requested.
    pub action: LifecycleAction,
    /// The TARGET principal's tenant, derived from the loaded store record.
    pub target_tenant: &'a str,
    /// The TARGET principal's workload id.
    pub target_workload_id: &'a str,
}

/// A fail-closed PDP fault. Any adapter error/timeout maps to this and the
/// handler denies (`403`) — a PDP outage never allows and never 500s
/// (failure-class (e)).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthzFault {
    detail: String, // data_class: INTERNAL_ONLY
}

impl AuthzFault {
    /// Construct a fault with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Lifecycle authorization PORT (the PDP seam). Returns `Ok(true)` to PERMIT,
/// `Ok(false)` to DENY, and `Err(AuthzFault)` for any adapter fault — the caller
/// maps both `Ok(false)` and `Err(_)` to `403` (fail-closed; failure-class (e)).
/// Cross-tenant requests MUST be denied regardless of how many allow rules match
/// (deny is authoritative at the service boundary). A production adapter is the
/// iam Cedar PDP client.
///
/// # Adapter contract (MUST be upheld by every implementation)
///
/// 1. **Fault mapping**: every error, timeout, or unavailability MUST map to
///    `Err(AuthzFault)` — never panic, never return `Ok(true)` on failure.
/// 2. **Deadline enforcement**: the adapter MUST enforce its own call deadline.
///    `decide()` is called WITHOUT any repository or denylist lock held; a slow
///    adapter stalls only the in-flight request, not the service mutex.
/// 3. **No panics**: the adapter MUST NOT panic. Production release builds use
///    `panic = "abort"`, so `catch_unwind` is not a backstop — a panicking
///    adapter aborts the process. Surface all failures as `Err(AuthzFault)`.
pub trait LifecycleAuthorizer: Send + Sync {
    /// Decide whether the verified caller may perform the lifecycle mutation on
    /// the target principal.
    ///
    /// Called with NO locks held. See the trait-level adapter contract above.
    ///
    /// # Errors
    /// Returns [`AuthzFault`] on any PDP adapter failure; the caller denies.
    fn decide(&self, request: &LifecycleAuthzRequest<'_>) -> Result<bool, AuthzFault>;
}

/// A fully-bound DECISION authorization request handed to the [`DecisionAuthorizer`].
/// This is the read-decision sibling of [`LifecycleAuthzRequest`]: it gates the
/// authorize / token-validation surfaces (`/authorize`, `/authorize-with-token`,
/// `/authorize:batch`, `/tokens/validate`) so a VERIFIED caller can only obtain a
/// decision scoped to its OWN tenant. The `subject_tenant` is the tenant the
/// decision is ABOUT — derived from the request body for `/authorize`, or from the
/// VALIDATED token for the token-bearing surfaces — never the caller's own tenant
/// and never an unverified header. A caller in tenant A asking for a decision over
/// tenant B's subject (forged body / stolen cross-tenant token) is therefore
/// deniable at the boundary (cross-tenant entitlement / IDOR; AUTH-005).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionAuthzRequest<'a> {
    /// Verified caller's tenant (trusted; from the credential, never a header).
    pub caller_tenant: &'a str,
    /// Verified caller's identity label (trusted).
    pub caller_id: &'a str,
    /// The SUBJECT's tenant the decision concerns (body field for `/authorize`,
    /// validated-token tenant for the token-bearing surfaces). NEVER the caller's.
    pub subject_tenant: &'a str,
    /// The SUBJECT's workload id.
    pub subject_workload_id: &'a str,
    /// The requested action (PARC).
    pub action: &'a str,
    /// The target resource type.
    pub resource_type: &'a str,
    /// The target resource id.
    pub resource_id: &'a str,
}

/// Decision-authorization PORT (the caller-authz seam for the READ decision
/// surfaces). Returns `Ok(true)` to PERMIT, `Ok(false)` to DENY, and
/// `Err(AuthzFault)` for any adapter fault — the caller maps both `Ok(false)` and
/// `Err(_)` to `403` (fail-closed; a PDP outage never allows and never 5xx).
/// Cross-tenant requests MUST be denied regardless of how many allow rules match
/// (deny is authoritative at the boundary). The reference adapter is a
/// same-tenant check in the composition root; a richer iam Cedar PDP swaps
/// in behind this port without touching the delivery surfaces.
///
/// The trait-level adapter contract of [`LifecycleAuthorizer`] (fault mapping, no
/// locks held, no panics) applies identically here.
pub trait DecisionAuthorizer: Send + Sync {
    /// Decide whether the verified caller may obtain the decision over the subject.
    ///
    /// # Errors
    /// Returns [`AuthzFault`] on any PDP adapter failure; the caller denies.
    fn decide(&self, request: &DecisionAuthzRequest<'_>) -> Result<bool, AuthzFault>;
}

/// Constant-time byte comparison — NEVER `==` for a credential (timing-safe).
/// Mirrors the proven `intelligence/adapters/rest` helper; kept local so the
/// boundary crate has no extra dependency.
#[must_use]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

/// Reference [`CallerVerifier`] adapter: a single configured bearer token bound
/// to one caller identity + tenant, compared in constant time. Production swaps
/// in an mTLS/SPIFFE or iam credential-store adapter behind the same port.
/// An empty/unset configured token verifies NOTHING (every caller is `401`):
/// there is no allow-all path.
#[derive(Clone, Debug)]
pub struct BearerCallerVerifier {
    token: String,         // data_class: SECRET
    caller_tenant: String, // data_class: INTERNAL_ONLY
    caller_id: String,     // data_class: INTERNAL_ONLY
}

impl BearerCallerVerifier {
    /// Build a verifier for one configured bearer credential. The bound caller
    /// identity + tenant are what a successful verify returns (never headers).
    #[must_use]
    pub fn new(
        token: impl Into<String>,
        caller_tenant: impl Into<String>,
        caller_id: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            caller_tenant: caller_tenant.into(),
            caller_id: caller_id.into(),
        }
    }
}

impl CallerVerifier for BearerCallerVerifier {
    fn verify_principal(&self, headers: &HeaderMap) -> Option<VerifiedCaller> {
        let configured = self.token.trim();
        if configured.is_empty() {
            // No configured credential ⇒ no caller can be verified (fail-closed).
            return None;
        }
        let presented = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))?;
        if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
            Some(VerifiedCaller {
                caller_tenant: self.caller_tenant.clone(),
                caller_id: self.caller_id.clone(),
            })
        } else {
            None
        }
    }
}

// =====================================================================
// Audit sink (PRD §3.3 / AC-W-13)
// =====================================================================

/// The kind of operation an [`AuditRecord`] captures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditEvent {
    /// An authorize decision (any of the three authorize endpoints).
    Authorize,
    /// A token validation (`/tokens/validate`).
    TokenValidation,
}

impl AuditEvent {
    /// Stable wire label for the event, mirroring the AsyncAPI channel names.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Authorize => "workload.decision.v1",
            Self::TokenValidation => "workload.token-validation.v1",
        }
    }
}

/// One immutable decision-log record. Construction-only (no setters) so a
/// record cannot be mutated after the fact — it is sealed at emission time.
/// Records carry decision METADATA only: never token material, signatures,
/// or claim payloads (the bearer credential must not reach the audit chain).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    event: AuditEvent,
    /// The workload id the record concerns, when known (a forged token may have
    /// no resolvable subject, in which case this is `None`).
    workload_id: Option<String>, // data_class: PII_IDENTIFYING
    /// Machine outcome label (e.g. `allow`, `deny`, `token-rejected`,
    /// `store-unavailable`, `validated`, `validation-failed`).
    outcome: String, // data_class: INTERNAL_ONLY
    /// Optional decision detail (policy id / deny stage / error class).
    detail: Option<String>, // data_class: INTERNAL_ONLY
    /// Authorization target, when the emitting surface scopes the decision to
    /// one action/resource (mirrors the optional `action`/`resource_type`/
    /// `resource_id` members of the `WorkloadDecisionPayload` AsyncAPI schema).
    action: Option<String>, // data_class: INTERNAL_ONLY
    resource_type: Option<String>, // data_class: INTERNAL_ONLY
    resource_id: Option<String>,   // data_class: INTERNAL_ONLY
    /// Verified caller identity that authorized (or attempted) the operation.
    /// Populated on lifecycle-control-plane decisions where a verified caller
    /// is present; `None` on token-validation records.
    caller_id: Option<String>, // data_class: INTERNAL_ONLY
    caller_tenant: Option<String>, // data_class: INTERNAL_ONLY
}

impl AuditRecord {
    /// Seal a new record.
    #[must_use]
    pub fn new(
        event: AuditEvent,
        workload_id: Option<String>,
        outcome: impl Into<String>,
        detail: Option<String>,
    ) -> Self {
        Self {
            event,
            workload_id,
            outcome: outcome.into(),
            detail,
            action: None,
            resource_type: None,
            resource_id: None,
            caller_id: None,
            caller_tenant: None,
        }
    }

    /// Attach the authorization target (action + resource) before sealing.
    /// Builder-style so existing emit sites stay construction-only.
    #[must_use]
    pub fn with_authorization_target(
        mut self,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.action = Some(action.into());
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Attach the verified caller identity that authorized (or attempted) the
    /// operation. Must be set for all lifecycle-control-plane audit records so
    /// incident response can answer "who authorized the retire/suspend".
    #[must_use]
    pub fn with_caller(
        mut self,
        caller_id: impl Into<String>,
        caller_tenant: impl Into<String>,
    ) -> Self {
        self.caller_id = Some(caller_id.into());
        self.caller_tenant = Some(caller_tenant.into());
        self
    }

    /// The event kind.
    #[must_use]
    pub fn event(&self) -> AuditEvent {
        self.event
    }

    /// The subject workload id, if resolvable.
    #[must_use]
    pub fn workload_id(&self) -> Option<&str> {
        self.workload_id.as_deref()
    }

    /// The machine outcome label.
    #[must_use]
    pub fn outcome(&self) -> &str {
        &self.outcome
    }

    /// The optional decision detail.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The authorized action, when the surface attached a target.
    #[must_use]
    pub fn action(&self) -> Option<&str> {
        self.action.as_deref()
    }

    /// The resource type of the authorization target, when attached.
    #[must_use]
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// The resource id of the authorization target, when attached.
    #[must_use]
    pub fn resource_id(&self) -> Option<&str> {
        self.resource_id.as_deref()
    }

    /// The verified caller id, when attached (lifecycle control-plane records).
    #[must_use]
    pub fn caller_id(&self) -> Option<&str> {
        self.caller_id.as_deref()
    }

    /// The verified caller's tenant, when attached (lifecycle control-plane records).
    #[must_use]
    pub fn caller_tenant(&self) -> Option<&str> {
        self.caller_tenant.as_deref()
    }
}

/// Decision-log emission port. One record is emitted per authorize and per
/// token-validation. Implementations append immutably (audit-chain bridge in
/// production; an in-memory log for tests/bring-up). Emission must not fail the
/// request path — a sink error is swallowed after best-effort, never surfaced
/// as an allow.
pub trait AuditSink: Send + Sync {
    /// Append a sealed record.
    fn record(&self, record: AuditRecord);
}

/// In-memory [`AuditSink`] backed by a mutex-guarded append-only vector. The
/// reference sink for tests and single-node bring-up.
#[derive(Clone, Debug, Default)]
pub struct InMemoryAuditSink {
    records: Arc<Mutex<Vec<AuditRecord>>>, // data_class: AUDIT
}

impl InMemoryAuditSink {
    /// Build an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the recorded log (clone). Order is emission order.
    #[must_use]
    pub fn records(&self) -> Vec<AuditRecord> {
        self.records
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Number of records emitted so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.lock().map(|guard| guard.len()).unwrap_or(0)
    }

    /// Whether no records have been emitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AuditSink for InMemoryAuditSink {
    fn record(&self, record: AuditRecord) {
        // A poisoned lock must not panic the request path (ADR-0083 Tier 3):
        // recover the guard and append regardless.
        match self.records.lock() {
            Ok(mut guard) => guard.push(record),
            Err(poisoned) => poisoned.into_inner().push(record),
        }
    }
}

// =====================================================================
// Shared state
// =====================================================================

/// Shared application state behind the axum router. Generic over the three
/// app-layer ports + the audit sink so the in-memory reference adapters drive
/// tests/bring-up and a production deployment swaps in sharded stores behind the
/// same traits. The repository + denylist are mutex-guarded because the
/// lifecycle use-cases mutate them; the authorizer, JWKS, and config are
/// read-only shared.
pub struct WorkloadAuthzState<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    repository: Mutex<R>,
    denylist: Mutex<D>,
    authorizer: A,
    jwks: Jwks,
    config: ValidationConfig,
    audit: S,
    now_provider: fn() -> i64,
    /// Caller-authentication port for the mutating lifecycle control plane.
    /// REQUIRED (non-optional): the router cannot serve `:suspend`/`:retire`
    /// without it, and there is no allow-all default (AUTH-005 default-deny).
    caller_verifier: Arc<dyn CallerVerifier>,
    /// Lifecycle authorization (PDP) port for the mutating control plane.
    /// REQUIRED (non-optional): a missing/erroring decision is fail-closed deny.
    lifecycle_authorizer: Arc<dyn LifecycleAuthorizer>,
    /// Decision authorization (PDP) port for the READ decision surfaces
    /// (`/authorize`, `/authorize-with-token`, `/authorize:batch`,
    /// `/tokens/validate`). REQUIRED (non-optional): a caller can obtain a
    /// decision only within its own tenant; a missing/erroring decision is
    /// fail-closed deny (AUTH-005 default-deny).
    decision_authorizer: Arc<dyn DecisionAuthorizer>,
}

impl<R, D, A, S> WorkloadAuthzState<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    /// Assemble the state from its parts, using a real wall-clock for token
    /// temporal validation. The caller-verifier + lifecycle-authorizer ports are
    /// REQUIRED — the mutating control plane cannot be built without them.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repository: R,
        denylist: D,
        authorizer: A,
        jwks: Jwks,
        config: ValidationConfig,
        audit: S,
        caller_verifier: Arc<dyn CallerVerifier>,
        lifecycle_authorizer: Arc<dyn LifecycleAuthorizer>,
        decision_authorizer: Arc<dyn DecisionAuthorizer>,
    ) -> Self {
        Self::with_clock(
            repository,
            denylist,
            authorizer,
            jwks,
            config,
            audit,
            caller_verifier,
            lifecycle_authorizer,
            decision_authorizer,
            default_now,
        )
    }

    /// Assemble the state with an explicit `now` provider (deterministic clock
    /// for tests). The caller-verifier + lifecycle-authorizer ports are REQUIRED.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn with_clock(
        repository: R,
        denylist: D,
        authorizer: A,
        jwks: Jwks,
        config: ValidationConfig,
        audit: S,
        caller_verifier: Arc<dyn CallerVerifier>,
        lifecycle_authorizer: Arc<dyn LifecycleAuthorizer>,
        decision_authorizer: Arc<dyn DecisionAuthorizer>,
        now_provider: fn() -> i64,
    ) -> Self {
        Self {
            repository: Mutex::new(repository),
            denylist: Mutex::new(denylist),
            authorizer,
            jwks,
            config,
            audit,
            now_provider,
            caller_verifier,
            lifecycle_authorizer,
            decision_authorizer,
        }
    }

    /// Borrow the audit sink (e.g. to inspect emitted records in tests).
    #[must_use]
    pub fn audit(&self) -> &S {
        &self.audit
    }

    // ------------------------------------------------------------------
    // Accessors used by the gRPC delivery module (src/grpc/) so it can
    // delegate to the same state without duplicating fields.
    // ------------------------------------------------------------------

    /// Borrow the authorizer (read-only shared; used by the gRPC Authorize path).
    #[must_use]
    pub(crate) fn authorizer_ref(&self) -> &A {
        &self.authorizer
    }

    /// Borrow the JWKS (read-only; used by the gRPC ValidateToken path).
    #[must_use]
    pub(crate) fn jwks_ref(&self) -> &Jwks {
        &self.jwks
    }

    /// Borrow the validation config (read-only; used by gRPC paths).
    #[must_use]
    pub(crate) fn config_ref(&self) -> &ValidationConfig {
        &self.config
    }

    /// Return the `now` provider fn (used by gRPC paths for token temporal checks).
    #[must_use]
    pub(crate) fn now_provider_ref(&self) -> fn() -> i64 {
        self.now_provider
    }

    /// Borrow the caller-verifier port (used by the gRPC paths to authenticate the
    /// caller from request metadata; mirrors the REST header authn).
    #[must_use]
    pub(crate) fn caller_verifier_ref(&self) -> &dyn CallerVerifier {
        self.caller_verifier.as_ref()
    }

    /// Borrow the decision-authorizer port (used by the gRPC paths for the
    /// per-decision same-tenant caller-authz gate; mirrors the REST gate).
    #[must_use]
    pub(crate) fn decision_authorizer_ref(&self) -> &dyn DecisionAuthorizer {
        self.decision_authorizer.as_ref()
    }

    /// Lock the repository for reading (gRPC delegate; mirrors REST helper).
    pub(crate) fn repository_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, R>, std::sync::PoisonError<std::sync::MutexGuard<'_, R>>>
    {
        self.repository.lock()
    }

    /// Lock the denylist for reading (gRPC delegate; mirrors REST helper).
    pub(crate) fn denylist_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, D>, std::sync::PoisonError<std::sync::MutexGuard<'_, D>>>
    {
        self.denylist.lock()
    }

    /// Run the same validate -> repository -> denylist -> policy hot path used
    /// by `/authorize-with-token` for service-local delivery surfaces that need
    /// to guard their own routes (for example SCIM). Emits exactly one
    /// immutable [`AuditRecord`] per call (PRD §3.3 / AC-W-13) with the
    /// authorization target attached, using the SAME outcome vocabulary as the
    /// REST authorize handlers. Emission cannot fail the decision: the outcome
    /// is computed first and returned regardless, and the [`AuditSink`]
    /// contract swallows sink errors after best-effort (a sink failure is the
    /// sink's own loud signal, never a fail-open). Callers own only the
    /// route-specific response envelope.
    #[must_use]
    pub fn authorize_token_for(
        &self,
        token: &str,
        action: Action,
        resource: Resource,
        context: std::collections::BTreeMap<String, ClaimValue>,
    ) -> AuthorizeOutcome {
        let now = (self.now_provider)();
        // Best-effort audit subject; a token that does not validate has no
        // trustworthy subject (mirrors `workload_id_from_token`).
        let workload_id = validate_workload_token(token, &self.jwks, &self.config, now)
            .ok()
            .map(|principal| principal.workload_id().as_str().to_owned());
        let target = (
            action.as_str().to_owned(),
            resource.resource_type().to_owned(),
            resource.resource_id().to_owned(),
        );
        let outcome = {
            let repo_guard = match self.repository.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let denylist_guard = match self.denylist.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            authorize_with_token(
                &*repo_guard,
                &*denylist_guard,
                &self.authorizer,
                &self.jwks,
                &self.config,
                now,
                token,
                action,
                resource,
                context,
            )
        };
        let (label, detail) = authorize_audit_parts(&outcome);
        self.audit.record(
            AuditRecord::new(AuditEvent::Authorize, workload_id, label, detail)
                .with_authorization_target(target.0, target.1, target.2),
        );
        outcome
    }
}

/// Default wall-clock `now` in epoch seconds. Saturates to 0 before the Unix
/// epoch rather than panicking (ADR-0083 Tier 3 panic-free).
#[must_use]
fn default_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// The axum state handle: an `Arc` of [`WorkloadAuthzState`].
pub type SharedState<R, D, A, S> = Arc<WorkloadAuthzState<R, D, A, S>>;

// =====================================================================
// Router
// =====================================================================

/// Build the workload-identity REST router over the shared state.
pub fn build_router<R, D, A, S>(state: SharedState<R, D, A, S>) -> Router
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    Router::new()
        .route(AUTHORIZE_ROUTE, post(authorize_handler::<R, D, A, S>))
        .route(
            AUTHORIZE_BATCH_ROUTE,
            post(authorize_batch_handler::<R, D, A, S>),
        )
        .route(
            AUTHORIZE_WITH_TOKEN_ROUTE,
            post(authorize_with_token_handler::<R, D, A, S>),
        )
        .route(
            TOKENS_VALIDATE_ROUTE,
            post(tokens_validate_handler::<R, D, A, S>),
        )
        .route(
            PRINCIPAL_LIFECYCLE_PATTERN,
            // The lifecycle handler reads only request Parts (headers + path params);
            // there is no body extractor, so DefaultBodyLimit would never fire a 413.
            // Body-limit enforcement is deferred to the global axum layer at the
            // service entry point (ADR-0581 §"Body-limit").
            post(principal_lifecycle_handler::<R, D, A, S>),
        )
        .with_state(state)
}

// =====================================================================
// Handlers
// =====================================================================

/// Map an [`AuthorizeOutcome`] onto an HTTP response + audit record.
///
/// - `Decided(permit)` -> `200 OK` with the decision (effect ALLOW).
/// - `Decided(deny)`   -> `403 Forbidden` with the decision (forbid/default).
/// - `TokenRejected`   -> `422 Unprocessable Entity` (PRD §3.4).
/// - `StoreUnavailable`-> `503 Service Unavailable` (fail-closed; PRD §5).
/// - `PrincipalUnknown`/`Revoked` -> `403 Forbidden` (a deny, NEVER a 404).
fn respond_to_outcome<S: AuditSink>(
    audit: &S,
    workload_id: Option<String>,
    outcome: &AuthorizeOutcome,
) -> Response {
    let (label, detail) = authorize_audit_parts(outcome);
    audit.record(AuditRecord::new(
        AuditEvent::Authorize,
        workload_id,
        label,
        detail,
    ));
    match outcome {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => {
            (StatusCode::OK, Json(AuthorizeResponse::from(decision))).into_response()
        }
        AuthorizeOutcome::Decided(decision) => (
            StatusCode::FORBIDDEN,
            Json(AuthorizeResponse::from(decision)),
        )
            .into_response(),
        AuthorizeOutcome::TokenRejected => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiErrorEnvelope::token_invalid(None)),
        )
            .into_response(),
        AuthorizeOutcome::StoreUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiErrorEnvelope::dependency_unavailable(None)),
        )
            .into_response(),
        // A deny — NOT a 404. An unknown/revoked subject is denied like any other.
        AuthorizeOutcome::PrincipalUnknown | AuthorizeOutcome::Revoked => (
            StatusCode::FORBIDDEN,
            Json(AuthorizeResponse::from(
                &AuthorizationDecision::default_deny(),
            )),
        )
            .into_response(),
    }
}

/// The single audit `(outcome label, detail)` mapping for a single-decision
/// authorize: shared by the REST handlers above (via [`respond_to_outcome`])
/// and the service-local guard path in
/// [`WorkloadAuthzState::authorize_token_for`], so a new surface cannot fork
/// the audit vocabulary. The batch handler and the gRPC module retain their
/// own pre-existing label mappings (same outcome words, `detail: None`) —
/// unifying those is a separate change; their tests pin today's shape.
fn authorize_audit_parts(outcome: &AuthorizeOutcome) -> (&'static str, Option<String>) {
    match outcome {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => {
            ("allow", decision_detail(decision))
        }
        AuthorizeOutcome::Decided(decision) => ("deny", decision_detail(decision)),
        AuthorizeOutcome::TokenRejected => ("token-rejected", None),
        AuthorizeOutcome::StoreUnavailable => ("store-unavailable", None),
        AuthorizeOutcome::PrincipalUnknown => ("deny", Some("principal-unknown".to_owned())),
        AuthorizeOutcome::Revoked => ("deny", Some("revoked".to_owned())),
    }
}

/// Extract a human-facing detail (policy id / stage) from a decision for the
/// audit record.
fn decision_detail(decision: &AuthorizationDecision) -> Option<String> {
    use iam_identity_workload_domain::DecisionReason;
    match decision.reason() {
        DecisionReason::ExplicitPermit { policy_id }
        | DecisionReason::ExplicitForbid { policy_id } => Some(policy_id.clone()),
        DecisionReason::DefaultDeny => Some("default-deny".to_owned()),
        DecisionReason::PrincipalNotOperational { .. } => Some("not-operational".to_owned()),
    }
}

/// `POST /authorize-with-token`: validate the raw JWT, resolve the persisted
/// principal, then authorize — the full fail-closed hot path.
async fn authorize_with_token_handler<R, D, A, S>(
    State(state): State<SharedState<R, D, A, S>>,
    // `HeaderMap` + raw `Bytes` (not `Json`) so the caller is VERIFIED before any
    // body is deserialized (AUTH-005 authn-before-body-parse).
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let Some(caller) = state.caller_verifier.verify_principal(&headers) else {
        return unverified_caller_response(state.audit(), AuditEvent::Authorize);
    };
    let request: AuthorizeWithTokenRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => return bad_request_response(error.to_string()),
    };
    // Same-tenant gate: subject tenant comes from the VALIDATED token (never a
    // header). A token that does not validate has no trustworthy subject — the
    // gate is skipped and the existing flow fail-closes it to a 422.
    if let Some((subject_tenant, subject_workload_id)) = validated_subject(&state, &request.token)
        && let Some(response) = decision_gate(
            &state,
            AuditEvent::Authorize,
            &caller,
            &subject_tenant,
            &subject_workload_id,
            &request.action,
            &request.resource.resource_type,
            &request.resource.resource_id,
        )
    {
        return response;
    }
    let outcome = run_authorize_with_token(&state, &request);
    let workload_id = workload_id_from_token(&state, &request.token);
    respond_to_outcome(state.audit(), workload_id, &outcome)
}

/// `POST /authorize:batch`: one decision per request, in order. A store/JWKS
/// outage on any single request fail-closes that request to a 503-class deny in
/// the body decision, but the batch itself returns `200` with the per-item
/// decisions (each fail-closed item is a DENY decision).
async fn authorize_batch_handler<R, D, A, S>(
    State(state): State<SharedState<R, D, A, S>>,
    // Authn-before-body-parse: verify the caller before deserializing the batch.
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let Some(caller) = state.caller_verifier.verify_principal(&headers) else {
        return unverified_caller_response(state.audit(), AuditEvent::Authorize);
    };
    let request: BatchAuthorizeRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => return bad_request_response(error.to_string()),
    };
    let mut decisions = Vec::with_capacity(request.requests.len());
    for item in &request.requests {
        // Per-item same-tenant gate: a cross-tenant (or PDP-faulted) item collapses
        // to a DENY decision in the body (fail-closed), never leaking an allow. The
        // gate emits the item's deny audit record; a permitted item falls through to
        // the normal per-item flow which emits its own record.
        if let Some((subject_tenant, subject_workload_id)) = validated_subject(&state, &item.token)
            && decision_gate(
                &state,
                AuditEvent::Authorize,
                &caller,
                &subject_tenant,
                &subject_workload_id,
                &item.action,
                &item.resource.resource_type,
                &item.resource.resource_id,
            )
            .is_some()
        {
            decisions.push(AuthorizeResponse::from(
                &AuthorizationDecision::default_deny(),
            ));
            continue;
        }
        let outcome = run_authorize_with_token(&state, item);
        let workload_id = workload_id_from_token(&state, &item.token);
        // Each item emits its own audit record (one per authorize).
        let outcome_label = batch_outcome_label(&outcome);
        state.audit().record(AuditRecord::new(
            AuditEvent::Authorize,
            workload_id,
            outcome_label,
            None,
        ));
        // Every non-permit item collapses to a DENY decision in the body so a
        // batch never leaks an allow on a fail-closed stage.
        decisions.push(AuthorizeResponse::from(&outcome.decision()));
    }
    (StatusCode::OK, Json(BatchAuthorizeResponse { decisions })).into_response()
}

/// `POST /authorize`: authorize an already-verified principal supplied
/// explicitly by a trusted PEP (no token validation). The principal is built as
/// Active with the supplied scopes/claims; the Cedar engine decides.
async fn authorize_handler<R, D, A, S>(
    State(state): State<SharedState<R, D, A, S>>,
    // AUTH-005 keystone: `HeaderMap` + raw `Bytes` (NOT `Json`) so the caller is
    // VERIFIED before the body is deserialized. Before this guard the authorized
    // principal was built ENTIRELY from caller-supplied body fields over plain
    // TCP — a forged body authorized arbitrary cross-tenant ALLOWs. Now: (1) an
    // unforgeable caller credential is required (401 on miss), then (2) the body
    // is parsed, then (3) a fail-closed same-tenant decision gate denies a caller
    // asking for a decision over another tenant's subject (403), and only then is
    // the principal built and the Cedar engine consulted.
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    // (1) AUTHN — unforgeable verified caller, BEFORE any body deserialize.
    let Some(caller) = state.caller_verifier.verify_principal(&headers) else {
        return unverified_caller_response(state.audit(), AuditEvent::Authorize);
    };
    // (2) Parse the body AFTER authn.
    let request: AuthorizeRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => return bad_request_response(error.to_string()),
    };
    // (3) Same-tenant decision gate: the subject tenant is the BODY's tenant_id;
    // a caller may only obtain a decision within its own tenant (fail-closed).
    if let Some(response) = decision_gate(
        &state,
        AuditEvent::Authorize,
        &caller,
        &request.tenant_id,
        &request.workload_id,
        &request.action,
        &request.resource.resource_type,
        &request.resource.resource_id,
    ) {
        return response;
    }
    // (4) Build the asserted principal and let the Cedar engine decide.
    let principal = match build_active_principal(&request) {
        Ok(principal) => principal,
        Err(envelope) => {
            // A malformed principal body is a 400 (not a deny decision) — the
            // request itself could not be parsed into a principal.
            return (StatusCode::BAD_REQUEST, Json(envelope)).into_response();
        }
    };
    let workload_id = principal.workload_id().as_str().to_owned();
    let mut authz_request = AuthorizationRequest::new(
        principal,
        Action::new(request.action.clone()),
        request.resource.clone().into_domain(),
    );
    authz_request.context = request
        .context
        .iter()
        .map(|(key, value)| (key.clone(), ClaimValue::from(value.clone())))
        .collect();
    let decision = state.authorizer.authorize(&authz_request);
    // /authorize never validates a token, so its outcomes are only permit/deny.
    let outcome = AuthorizeOutcome::Decided(decision);
    respond_to_outcome(state.audit(), Some(workload_id), &outcome)
}

/// `POST /tokens/validate`: validate a raw JWT and return the projected
/// principal identity. A validation failure is a 422 (PRD §3.4).
async fn tokens_validate_handler<R, D, A, S>(
    State(state): State<SharedState<R, D, A, S>>,
    // Authn-before-body-parse: verify the caller before deserializing the body.
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let Some(caller) = state.caller_verifier.verify_principal(&headers) else {
        return unverified_caller_response(state.audit(), AuditEvent::TokenValidation);
    };
    let request: ValidateTokenRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => return bad_request_response(error.to_string()),
    };
    let now = (state.now_provider)();
    match validate_workload_token(&request.token, &state.jwks, &state.config, now) {
        Ok(principal) => {
            // Same-tenant gate: a caller may only introspect a token within its own
            // tenant (cross-tenant introspection is a 403, fail-closed).
            if let Some(response) = decision_gate(
                &state,
                AuditEvent::TokenValidation,
                &caller,
                principal.tenant_id().as_str(),
                principal.workload_id().as_str(),
                "identity.workload.ValidateToken",
                "Workload",
                principal.workload_id().as_str(),
            ) {
                return response;
            }
            state.audit().record(AuditRecord::new(
                AuditEvent::TokenValidation,
                Some(principal.workload_id().as_str().to_owned()),
                "validated",
                None,
            ));
            let response = ValidateTokenResponse {
                tenant_id: principal.tenant_id().as_str().to_owned(),
                workload_id: principal.workload_id().as_str().to_owned(),
                owning_capability: principal.owning_capability().as_str().to_owned(),
                trust_domain: principal.trust_domain().as_str().to_owned(),
                state: state_label(principal.state()).to_owned(),
                scopes: principal.scopes().to_vec(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => {
            state.audit().record(AuditRecord::new(
                AuditEvent::TokenValidation,
                None,
                "validation-failed",
                Some(error.to_string()),
            ));
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiErrorEnvelope::token_invalid(None)),
            )
                .into_response()
        }
    }
}

/// `POST /principals/{id}:suspend|retire`: the AIP-136 custom-method lifecycle
/// surface. The captured `<id>:<verb>` segment is split on the FINAL colon (a
/// workload id never contains `:`); an unknown verb is a `404` (no such custom
/// method).
async fn principal_lifecycle_handler<R, D, A, S>(
    State(state): State<SharedState<R, D, A, S>>,
    // `HeaderMap` is a `FromRequestParts` extractor — it runs BEFORE any body is
    // read, so the caller is verified before an arbitrary body is deserialized
    // (AUTH-005 failure-class (e): authn before body). The lifecycle envelope is
    // intentionally empty, so this handler never deserializes a body at all.
    headers: HeaderMap,
    Path(id_and_verb): Path<String>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let Some((id, verb)) = id_and_verb.rsplit_once(':') else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiErrorEnvelope::not_found(Some(
                "expected /principals/{id}:suspend or :retire".to_owned(),
            ))),
        )
            .into_response();
    };
    let op = match verb {
        "suspend" => LifecycleOp::Suspend,
        "retire" => LifecycleOp::Retire,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiErrorEnvelope::not_found(Some(format!(
                    "unknown lifecycle method: {verb}"
                )))),
            )
                .into_response();
        }
    };
    lifecycle_transition(&state, &headers, id, op)
}

// =====================================================================
// Handler helpers (synchronous core; panic-free)
// =====================================================================

/// Run the token-bearing authorize use-case against the (mutex-guarded) state.
/// A poisoned lock is recovered rather than panicked (fail-closed posture).
fn run_authorize_with_token<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    request: &AuthorizeWithTokenRequest,
) -> AuthorizeOutcome
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider)();
    let repo_guard = match state.repository.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let denylist_guard = match state.denylist.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    authorize_with_token(
        &*repo_guard,
        &*denylist_guard,
        &state.authorizer,
        &state.jwks,
        &state.config,
        now,
        &request.token,
        request.action(),
        request.resource.clone().into_domain(),
        request.context_domain(),
    )
}

/// Best-effort resolution of the workload id carried by a token, for the audit
/// record's subject field. Returns `None` for a token that does not validate
/// (a forged token has no trustworthy subject).
fn workload_id_from_token<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    token: &str,
) -> Option<String>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider)();
    validate_workload_token(token, &state.jwks, &state.config, now)
        .ok()
        .map(|principal| principal.workload_id().as_str().to_owned())
}

/// The `401` response for a request with no verified caller, emitting the single
/// fail-closed audit record (a denied authorize/validation call is on the audit
/// chain regardless of outcome). A self-attested header can never reach a `Some`
/// caller — [`CallerVerifier`] ignores caller-supplied identity headers.
fn unverified_caller_response<S: AuditSink>(audit: &S, event: AuditEvent) -> Response {
    audit.record(AuditRecord::new(
        event,
        None,
        "deny",
        Some("unverified-caller".to_owned()),
    ));
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorEnvelope::unauthorized(None)),
    )
        .into_response()
}

/// The `400` response for a body that authenticated but did not deserialize. A
/// malformed body is a `400` (the request itself is unparseable), NOT a deny
/// decision — and it is reached only AFTER caller authn (authn-before-body-parse).
fn bad_request_response(detail: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorEnvelope::validation(
            "invalid request body",
            Some(detail.into()),
        )),
    )
        .into_response()
}

/// Best-effort `(subject_tenant, subject_workload_id)` of the principal a token
/// attests, for the per-decision same-tenant gate. `None` for a token that does
/// not validate — a forged token has no trustworthy subject, and the downstream
/// flow fail-closes such a request on its own (a `422`/deny), so the gate is
/// simply skipped rather than allowed.
fn validated_subject<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    token: &str,
) -> Option<(String, String)>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider)();
    validate_workload_token(token, &state.jwks, &state.config, now)
        .ok()
        .map(|principal| {
            (
                principal.tenant_id().as_str().to_owned(),
                principal.workload_id().as_str().to_owned(),
            )
        })
}

/// Per-decision caller-authz gate (AUTH-005). A VERIFIED `caller` may obtain a
/// decision ONLY within its own tenant: the [`DecisionAuthorizer`] is consulted
/// with the SUBJECT's tenant (the body's `tenant_id` for `/authorize`, the
/// VALIDATED token's tenant for the token-bearing surfaces) so a caller in tenant
/// A cannot obtain a decision scoped to tenant B (cross-tenant entitlement /
/// IDOR). Returns `Some(403)` on an `Ok(false)` policy deny (`decision-forbidden`)
/// or an `Err` PDP fault (`decision-pdp-fault`) — BOTH fail-closed, never `5xx` —
/// and `None` on permit. A deny emits exactly one audit record with caller
/// attribution; the permit path emits NOTHING so the downstream decision handler
/// keeps the one-record-per-decision invariant.
#[allow(clippy::too_many_arguments)]
fn decision_gate<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    event: AuditEvent,
    caller: &VerifiedCaller,
    subject_tenant: &str,
    subject_workload_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
) -> Option<Response>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let request = DecisionAuthzRequest {
        caller_tenant: caller.caller_tenant(),
        caller_id: caller.caller_id(),
        subject_tenant,
        subject_workload_id,
        action,
        resource_type,
        resource_id,
    };
    // Both Ok(false) and Err(_) deny (fail-closed) but emit DISTINCT details so a
    // PDP outage is distinguishable from an intentional cross-tenant deny.
    let detail = match state.decision_authorizer.decide(&request) {
        Ok(true) => return None,
        Ok(false) => "decision-forbidden",
        Err(_fault) => "decision-pdp-fault",
    };
    state.audit.record(
        AuditRecord::new(
            event,
            Some(subject_workload_id.to_owned()),
            "deny",
            Some(detail.to_owned()),
        )
        .with_authorization_target(action, resource_type, resource_id)
        .with_caller(caller.caller_id(), caller.caller_tenant()),
    );
    Some(
        (
            StatusCode::FORBIDDEN,
            Json(ApiErrorEnvelope::forbidden(None)),
        )
            .into_response(),
    )
}

/// Build an Active principal from an `/authorize` request's explicit fields.
/// `pub(crate)` so the gRPC delivery module can reuse it without duplicating logic.
pub(crate) fn build_active_principal(
    request: &AuthorizeRequest,
) -> Result<WorkloadPrincipal, ApiErrorEnvelope> {
    let mut principal = WorkloadPrincipal::provision(
        request.tenant_id.clone(),
        request.workload_id.clone(),
        request.owning_capability.clone(),
    )
    .map_err(|error| ApiErrorEnvelope::validation("invalid principal", Some(error.to_string())))?;
    // A caller asserting an already-verified principal is asserting it is live.
    principal
        .transition_to(WorkloadState::Active)
        .map_err(|error| {
            ApiErrorEnvelope::validation("principal not activatable", Some(error.to_string()))
        })?;
    for scope in &request.scopes {
        principal.grant_scope(scope.clone()).map_err(|error| {
            ApiErrorEnvelope::validation("invalid scope", Some(error.to_string()))
        })?;
    }
    for (name, value) in &request.claims {
        principal
            .set_claim(name.clone(), ClaimValue::from(value.clone()))
            .map_err(|error| {
                ApiErrorEnvelope::validation("invalid claim", Some(error.to_string()))
            })?;
    }
    Ok(principal)
}

/// Which lifecycle operation a control-plane handler performs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LifecycleOp {
    Suspend,
    Retire,
}

impl LifecycleOp {
    /// The PDP action this operation authorizes against.
    fn action(self) -> LifecycleAction {
        match self {
            Self::Suspend => LifecycleAction::Suspend,
            Self::Retire => LifecycleAction::Retire,
        }
    }
}

/// Shared suspend/retire control-plane flow — the FAIL-CLOSED choke point for the
/// mutating lifecycle routes (ADR-0581 / AUTH-005). Enforced HERE (not only at the
/// HTTP edge) so any in-crate caller of a lifecycle mutation passes the same gate
/// (failure-class (c)). Order is deliberate and fail-closed:
///
/// 1. Verify the caller's UNFORGEABLE credential ([`CallerVerifier`]); no
///    verified principal ⇒ `401` (failure-class (b): caller headers never
///    authorize). Done before any store work.
/// 2. Validate the id shape (`400` on a malformed id).
/// 3. Acquire the repository + denylist locks, LOAD the target principal to
///    derive its REAL tenant from the trusted store (NOT the caller's tenant,
///    NOT a header) (failure-class (d): true blast radius / no IDOR), then
///    **release the locks**. An unknown principal is a `404` after authn.
/// 4. Authorize via the PDP ([`LifecycleAuthorizer`]) — **WITHOUT any lock
///    held** — bound to `caller_tenant` + the TARGET's tenant + action. A slow
///    or hung PDP adapter stalls only this request, not the service mutex.
///    `Ok(false)` ⇒ `403` (policy deny). `Err(_)` ⇒ `403` (PDP fault —
///    fail-closed, never `500`/allow; failure-class (e)). The two outcomes
///    emit DISTINCT audit details for incident-response separability.
/// 5. Re-acquire the locks, re-load the target to guard against the
///    load→authorize→mutate TOCTOU window, then run the use-case.
///
///    The load (step 3) and re-load (step 5) intentionally re-read from the
///    same mutex-guarded store so a concurrent state change is never silently
///    clobbered.
///    An illegal transition is `409`; a store outage is `503`.
///
/// Exactly one audit record is emitted for the authorize decision (preserving
/// the per-decision audit invariant). The record includes the verified caller
/// identity (caller_id + caller_tenant) for incident-response attribution.
fn lifecycle_transition<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    headers: &HeaderMap,
    id: &str,
    op: LifecycleOp,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    // (1) AUTHN — unforgeable verified caller, BEFORE any store work or body read.
    let Some(caller) = state.caller_verifier.verify_principal(headers) else {
        // No verified principal ⇒ default-deny 401. A self-attested header cannot
        // reach this branch's `Some` (the verifier ignores them).
        state.audit.record(
            AuditRecord::new(
                AuditEvent::Authorize,
                None,
                "deny",
                Some("unverified-caller".to_owned()),
            )
            .with_authorization_target(op.action().as_str(), "Workload", id),
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiErrorEnvelope::unauthorized(None)),
        )
            .into_response();
    };

    // (2) Validate the id shape.
    let workload_id = match WorkloadId::new(id.to_owned()) {
        Ok(id) => id,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorEnvelope::validation(
                    "invalid workload id",
                    Some(error.to_string()),
                )),
            )
                .into_response();
        }
    };

    // (3) Acquire locks, load the target to derive its REAL tenant, then DROP
    // locks before calling the PDP. A slow/hung PDP adapter must not hold the
    // repository or denylist lock — it stalls only the current request.
    let target_tenant = {
        let repo_guard = match state.repository.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match repo_guard.load(&workload_id) {
            Ok(Some(principal)) => principal.tenant_id().as_str().to_owned(),
            Ok(None) => {
                // Authenticated caller, but no such principal: 404. An UNVERIFIED
                // caller never reaches here — it 401s at step 1. See ADR-0581
                // §"Existence oracle" for the verified-caller-only residual.
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiErrorEnvelope::not_found(None)),
                )
                    .into_response();
            }
            Err(_) => {
                // Store read failure ⇒ fail-closed 503 (never allow the mutation).
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiErrorEnvelope::dependency_unavailable(None)),
                )
                    .into_response();
            }
        }
        // repo_guard drops here — locks released before PDP call below.
    };
    let action = op.action();

    // (4) PDP decision — NO locks held. The LifecycleAuthorizer adapter contract
    // requires adapters to enforce their own deadline and surface all faults as
    // Err(AuthzFault) rather than panicking. See trait-level doc for the full
    // contract. `Ok(true)` = permit; `Ok(false)` = policy deny; `Err(_)` = PDP
    // fault. The last two both return 403 (fail-closed) but emit DISTINCT audit
    // details so a PDP outage is distinguishable from a real policy deny.
    let authz_request = LifecycleAuthzRequest {
        caller_tenant: caller.caller_tenant(),
        caller_id: caller.caller_id(),
        action,
        target_tenant: &target_tenant,
        target_workload_id: workload_id.as_str(),
    };
    let decide_result = state.lifecycle_authorizer.decide(&authz_request);
    match decide_result {
        Ok(true) => {} // fall through to re-acquire + mutate
        Ok(false) => {
            // Intentional policy deny.
            state.audit.record(
                AuditRecord::new(
                    AuditEvent::Authorize,
                    Some(workload_id.as_str().to_owned()),
                    "deny",
                    Some("lifecycle-forbidden".to_owned()),
                )
                .with_authorization_target(action.as_str(), "Workload", workload_id.as_str())
                .with_caller(caller.caller_id(), caller.caller_tenant()),
            );
            return (
                StatusCode::FORBIDDEN,
                Json(ApiErrorEnvelope::forbidden(None)),
            )
                .into_response();
        }
        Err(_fault) => {
            // PDP fault/outage — fail-closed 403, DISTINCT detail from policy deny.
            state.audit.record(
                AuditRecord::new(
                    AuditEvent::Authorize,
                    Some(workload_id.as_str().to_owned()),
                    "deny",
                    Some("lifecycle-pdp-fault".to_owned()),
                )
                .with_authorization_target(action.as_str(), "Workload", workload_id.as_str())
                .with_caller(caller.caller_id(), caller.caller_tenant()),
            );
            return (
                StatusCode::FORBIDDEN,
                Json(ApiErrorEnvelope::forbidden(None)),
            )
                .into_response();
        }
    }

    // Authorized: emit the allow record (with caller attribution) before mutating.
    state.audit.record(
        AuditRecord::new(
            AuditEvent::Authorize,
            Some(workload_id.as_str().to_owned()),
            "allow",
            Some("lifecycle-permitted".to_owned()),
        )
        .with_authorization_target(action.as_str(), "Workload", workload_id.as_str())
        .with_caller(caller.caller_id(), caller.caller_tenant()),
    );

    // (5) Re-acquire locks and re-load the target to guard the TOCTOU window
    // between step 3 (load for tenant derivation) and the mutation: a concurrent
    // state change must not be silently clobbered.
    let mut repo_guard = match state.repository.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut denylist_guard = match state.denylist.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let result = match op {
        LifecycleOp::Suspend => suspend(&mut *repo_guard, &mut *denylist_guard, &workload_id),
        LifecycleOp::Retire => retire(&mut *repo_guard, &mut *denylist_guard, &workload_id),
    };
    use iam_identity_workload_app::LifecycleError;
    match result {
        Ok(principal) => (
            StatusCode::OK,
            Json(PrincipalLifecycleResponse::new(
                principal.workload_id().as_str(),
                principal.state(),
            )),
        )
            .into_response(),
        Err(LifecycleError::PrincipalNotFound { .. }) => (
            StatusCode::NOT_FOUND,
            Json(ApiErrorEnvelope::not_found(None)),
        )
            .into_response(),
        Err(LifecycleError::PrincipalAlreadyExists { .. }) => (
            StatusCode::CONFLICT,
            Json(ApiErrorEnvelope::validation(
                "principal already exists",
                None,
            )),
        )
            .into_response(),
        Err(LifecycleError::Domain(error)) => (
            // An illegal lifecycle transition (e.g. suspend from Provisioned) is
            // a conflict with the current state, not a server fault.
            StatusCode::CONFLICT,
            Json(ApiErrorEnvelope::validation(
                "illegal lifecycle transition",
                Some(error.to_string()),
            )),
        )
            .into_response(),
        Err(LifecycleError::Repository(_)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiErrorEnvelope::dependency_unavailable(None)),
        )
            .into_response(),
    }
}

/// Machine outcome label for a batch item's audit record.
fn batch_outcome_label(outcome: &AuthorizeOutcome) -> &'static str {
    match outcome {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => "allow",
        AuthorizeOutcome::Decided(_) => "deny",
        AuthorizeOutcome::TokenRejected => "token-rejected",
        AuthorizeOutcome::PrincipalUnknown => "principal-unknown",
        AuthorizeOutcome::Revoked => "revoked",
        AuthorizeOutcome::StoreUnavailable => "store-unavailable",
    }
}

/// Lowercase lifecycle label mirroring the authz layer / `identity.cedar`.
fn state_label(state: WorkloadState) -> &'static str {
    match state {
        WorkloadState::Provisioned => "provisioned",
        WorkloadState::Active => "active",
        WorkloadState::Suspended => "suspended",
        WorkloadState::Retired => "retired",
    }
}
