//! Workload-identity REST surface (ADR-0105 Layer 5).
//!
//! The I/O-bearing axum app for the workload-identity service. It mounts the
//! inward use-case core ([`oya_identity_workload_app`]) behind the HTTP
//! contract promised by `microservices/identity/workload-identity/PRD.md` §1.2
//! and serialized in `microservices/identity/contracts/openapi/workload.yaml`:
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
//! audit chain regardless of outcome.
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
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;

use oya_identity_workload_api::{
    ApiErrorEnvelope, AuthorizeRequest, AuthorizeResponse, AuthorizeWithTokenRequest,
    BatchAuthorizeRequest, BatchAuthorizeResponse, PrincipalLifecycleResponse,
    ValidateTokenRequest, ValidateTokenResponse,
};
use oya_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token,
    retire, suspend,
};
use oya_identity_workload_authz_cedar_adapter::WorkloadAuthorizer;
use oya_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, WorkloadId, WorkloadPrincipal,
    WorkloadState,
};
use oya_identity_workload_oidc_adapter::{Jwks, ValidationConfig, validate_workload_token};

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
        }
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
}

impl<R, D, A, S> WorkloadAuthzState<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    /// Assemble the state from its parts, using a real wall-clock for token
    /// temporal validation.
    #[must_use]
    pub fn new(
        repository: R,
        denylist: D,
        authorizer: A,
        jwks: Jwks,
        config: ValidationConfig,
        audit: S,
    ) -> Self {
        Self::with_clock(
            repository,
            denylist,
            authorizer,
            jwks,
            config,
            audit,
            default_now,
        )
    }

    /// Assemble the state with an explicit `now` provider (deterministic clock
    /// for tests).
    #[must_use]
    pub fn with_clock(
        repository: R,
        denylist: D,
        authorizer: A,
        jwks: Jwks,
        config: ValidationConfig,
        audit: S,
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
    match outcome {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "allow",
                decision_detail(decision),
            ));
            (StatusCode::OK, Json(AuthorizeResponse::from(decision))).into_response()
        }
        AuthorizeOutcome::Decided(decision) => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "deny",
                decision_detail(decision),
            ));
            (
                StatusCode::FORBIDDEN,
                Json(AuthorizeResponse::from(decision)),
            )
                .into_response()
        }
        AuthorizeOutcome::TokenRejected => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "token-rejected",
                None,
            ));
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiErrorEnvelope::token_invalid(None)),
            )
                .into_response()
        }
        AuthorizeOutcome::StoreUnavailable => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "store-unavailable",
                None,
            ));
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiErrorEnvelope::dependency_unavailable(None)),
            )
                .into_response()
        }
        AuthorizeOutcome::PrincipalUnknown => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "deny",
                Some("principal-unknown".to_owned()),
            ));
            // A deny — NOT a 404. An unknown subject is denied like any other.
            (
                StatusCode::FORBIDDEN,
                Json(AuthorizeResponse::from(
                    &AuthorizationDecision::default_deny(),
                )),
            )
                .into_response()
        }
        AuthorizeOutcome::Revoked => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "deny",
                Some("revoked".to_owned()),
            ));
            (
                StatusCode::FORBIDDEN,
                Json(AuthorizeResponse::from(
                    &AuthorizationDecision::default_deny(),
                )),
            )
                .into_response()
        }
    }
}

/// Extract a human-facing detail (policy id / stage) from a decision for the
/// audit record.
fn decision_detail(decision: &AuthorizationDecision) -> Option<String> {
    use oya_identity_workload_domain::DecisionReason;
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
    Json(request): Json<AuthorizeWithTokenRequest>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
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
    Json(request): Json<BatchAuthorizeRequest>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let mut decisions = Vec::with_capacity(request.requests.len());
    for item in &request.requests {
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
    Json(request): Json<AuthorizeRequest>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
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
    Json(request): Json<ValidateTokenRequest>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider)();
    match validate_workload_token(&request.token, &state.jwks, &state.config, now) {
        Ok(principal) => {
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
    lifecycle_transition(&state, id, op)
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

/// Shared suspend/retire control-plane flow: validate the id shape, run the
/// app use-case against the mutex-guarded store + denylist, and map the result
/// onto an HTTP response. A not-found principal is a `404`; an illegal
/// transition is a `409`; a store outage is `503`.
fn lifecycle_transition<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    id: &str,
    op: LifecycleOp,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
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
    use oya_identity_workload_app::LifecycleError;
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
