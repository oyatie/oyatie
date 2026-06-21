//! Composition root for the tenant registration / lifecycle delivery layer.
//!
//! Wires the locked tenancy core into a live HTTP surface so a new tenant can
//! be REGISTERED, PROVISIONED (driven to Active through the real contract
//! FSM), READ, and moved across its lifecycle — closing the
//! delivery-chain gap recorded in the auth/onboarding E2E audit (tenant
//! registration: "FSM real but in-memory only; no rest/app crate").
//!
//! ## REST surface
//!
//! ```text
//! POST   /v1/tenants                  — register a tenant (born Provisioning)
//! GET    /v1/tenants/{id}             — read the tenant's current state
//! GET    /v1/tenants                  — list tenants (AIP-158 paged)
//! POST   /v1/tenants/{id}/provision   — drive Provisioning -> Active (FSM)
//! POST   /v1/tenants/{id}/suspend     — Active -> Suspended
//! POST   /v1/tenants/{id}/resume      — Suspended -> Active
//! DELETE /v1/tenants/{id}             — retire (terminal; id never reused)
//! GET    /healthz                     — liveness probe
//! ```
//!
//! Mutating requests carry a client-generated `Idempotency-Key` header
//! (canonical UUID, AIP-155 / AWS client-token shape): the same key replays
//! the original outcome; a reused key with different parameters is rejected.
//!
//! ## Layering (ADR-0131)
//!
//! facade -> { usecase (core), adapter (in-memory store) }, both path-inward.
//! This crate owns NO lifecycle algorithm: every state transition is decided
//! by the contract FSM inside the usecase. The in-memory store is a valid
//! ports/adapters realization for single-node bring-up; a persistent store
//! plugs in behind the same kernel port with no change here.
//!
//! ## Provision semantics
//!
//! Registration lands a tenant in `Provisioning`. `:provision` starts an
//! `Activate` lifecycle operation (AIP-151) and polls it to completion in the
//! same request, so the synchronous HTTP caller observes the converged
//! `Active` state. The async operation ledger is still the single source of
//! truth — a reconciler or a slower backend simply polls the returned
//! operation instead.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};

use oya_shared_platform_contracts_kernel::tenancy::{
    IsolationPosture, Tenant, TenantLifecycleOperation, TenantLifecycleState,
};
use oya_shared_resource_provider_contract_kernel::{
    IdempotencyKey, Operation, OperationResult, PageRequest, PageToken, ProviderError,
    ResourceName, ResourceProvider,
};
use tenancy_tenant_lifecycle_authz_port::{
    AuthorizationDecision, AuthorizationQuery, CallerIdentity, TenantLifecycleAction,
    TenantLifecycleAuthorizer,
};
use tenancy_tenant_lifecycle_authz_pdp::PdpTenantLifecycleAuthorizer;
use tenancy_tenant_lifecycle_kernel::TenantLifecycleStore;
use tenancy_tenant_lifecycle_store_inmemory::InMemoryTenantLifecycleStore;
use tenancy_tenant_lifecycle_usecase::{TENANT_COLLECTION, TenantLifecycleProvider};

/// The default page size used when a list request omits one.
const DEFAULT_PAGE_SIZE: u32 = 50;

/// Env var carrying the platform-admin bearer token. A request presenting this
/// token (constant-time verified) is a platform admin: it may register and list
/// tenants, but it has NO tenant scope, so per-tenant ops still deny-by-default
/// at the PDP (the platform admin is not a tenant operator).
const ENV_PLATFORM_ADMIN_TOKEN: &str = "TENANCY_PLATFORM_ADMIN_TOKEN";

/// Env var carrying the tenant-operator bearer token. A request presenting this
/// token (constant-time verified) AND the `x-oya-tenant` axis header is a
/// tenant operator scoped to that asserted tenant. The bearer alone NEVER
/// grants the tenant axis — the header binds the axis only AFTER the bearer is
/// verified, and the PDP then checks the axis against the target {id}.
const ENV_TENANT_OPERATOR_TOKEN: &str = "TENANCY_TENANT_OPERATOR_TOKEN";

/// The header asserting which tenant a verified tenant-operator is acting as.
const HEADER_TENANT_AXIS: &str = "x-oya-tenant";

/// The storage-port bound a served lifecycle provider needs: the kernel store
/// port, plus the thread/async-share markers axum handlers require.
pub trait LifecycleStore: TenantLifecycleStore + Send + 'static {}

impl<T> LifecycleStore for T where T: TenantLifecycleStore + Send + 'static {}

/// The lifecycle provider behind a single composition-root lock.
/// `ResourceProvider` mutations take `&mut self`; the lock makes the provider
/// shareable across async axum handlers while keeping the operation ledger
/// single-writer (so idempotency holds).
///
/// ## Concurrency seam (deliberate, single-node bring-up)
///
/// This is ONE coarse global lock over the whole provider — correct and simple
/// for the in-memory single-node bring-up, but it serializes all mutations.
/// Per-tenant / row-level concurrency is NOT this layer's concern: it moves
/// into the persistent store adapter behind the unchanged `TenantLifecycleStore`
/// port (the G03 oya-data store does row-level locking / optimistic
/// concurrency), so the delivery surface keeps this single-writer invariant and
/// the store owns the contention model. Do not pre-optimize the in-memory lock.
pub type SharedProvider<S> = Arc<Mutex<TenantLifecycleProvider<S>>>;

/// The authorization decision port shared across handlers (the PEP's PDP).
pub type SharedAuthorizer = Arc<dyn TenantLifecycleAuthorizer>;

/// Application state injected into every handler.
pub struct AppState<S: LifecycleStore> {
    provider: SharedProvider<S>,
    /// The fail-closed authorizer (the embedded PDP). REQUIRED — there is no
    /// "no authorizer" variant: a service with no authz provider must never be
    /// constructed (the composition root refuses to build one).
    authorizer: SharedAuthorizer,
    /// Constant-time-verified platform-admin bearer token. `None` means no
    /// platform admin can authenticate (register/list deny-all), never an open
    /// surface.
    platform_admin_token: Option<String>,
    /// Constant-time-verified tenant-operator bearer token. `None` means no
    /// tenant operator can authenticate (per-tenant ops deny-all).
    tenant_operator_token: Option<String>,
}

impl<S: LifecycleStore> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
            authorizer: Arc::clone(&self.authorizer),
            platform_admin_token: self.platform_admin_token.clone(),
            tenant_operator_token: self.tenant_operator_token.clone(),
        }
    }
}

impl<S: LifecycleStore> AppState<S> {
    /// Wrap a lifecycle provider + the REQUIRED authorizer and bearer-token
    /// configuration for serving. There is no default-allow path: the
    /// authorizer is non-optional and the tokens gate authentication.
    #[must_use]
    pub fn new(
        provider: TenantLifecycleProvider<S>,
        authorizer: SharedAuthorizer,
        platform_admin_token: Option<String>,
        tenant_operator_token: Option<String>,
    ) -> Self {
        Self {
            provider: Arc::new(Mutex::new(provider)),
            authorizer,
            platform_admin_token: normalize_token(platform_admin_token),
            tenant_operator_token: normalize_token(tenant_operator_token),
        }
    }

    /// Acquire the provider lock, RECOVERING from poisoning rather than
    /// propagating it (poison-DoS hardening): a handler that panicked while
    /// holding the lock must not brick every subsequent request. The contract
    /// FSM mutates the in-memory ledger transactionally per call, so a recovered
    /// guard observes a consistent prior state; one panicked request can fail,
    /// but the service keeps serving (no single-panic denial of service).
    fn lock(&self) -> MutexGuard<'_, TenantLifecycleProvider<S>> {
        self.provider
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Trim a configured token and treat empty/whitespace-only as unset (so a blank
/// env var can never accidentally authenticate every caller).
fn normalize_token(token: Option<String>) -> Option<String> {
    token
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// Constant-time byte comparison (no early-out on first mismatch): never leak
/// token length or content through timing. Mirrors the established intelligence
/// REST `constant_time_eq` doctrine — do NOT hand-roll a naive `==`.
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

/// Extract a `Bearer <token>` value from the Authorization header, if present.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

/// Whether a presented bearer matches a configured token (constant-time).
fn bearer_matches(headers: &HeaderMap, configured: Option<&str>) -> bool {
    let Some(configured) = configured else {
        return false;
    };
    let Some(presented) = bearer_token(headers) else {
        return false;
    };
    constant_time_eq(presented.as_bytes(), configured.as_bytes())
}

// ============================================================
// Wire DTOs
// ============================================================

/// Body for `POST /v1/tenants` (register a tenant).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterTenantBody {
    /// Stable tenant id (slug). The resource name is `tenants/<tenant_id>`.
    pub tenant_id: String,
    pub display_name: String,
    pub isolation_posture: IsolationPosture,
    /// The cell this tenant is pinned to (cell-based architecture).
    pub cell_id: String,
    /// Optional data-residency zone constraint.
    #[serde(default)]
    pub residency_zone: Option<String>,
}

/// The tenant projection returned on the read surface.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TenantView {
    pub tenant_id: String,
    pub display_name: String,
    pub state: TenantLifecycleState,
    pub isolation_posture: IsolationPosture,
    pub cell_id: String,
    pub residency_zone: Option<String>,
}

impl From<Tenant> for TenantView {
    fn from(tenant: Tenant) -> Self {
        Self {
            tenant_id: tenant.tenant_id,
            display_name: tenant.display_name,
            state: tenant.state,
            isolation_posture: tenant.isolation_posture,
            cell_id: tenant.cell_id,
            residency_zone: tenant.residency_zone,
        }
    }
}

/// One page of the tenant list.
#[derive(Clone, Debug, Serialize)]
pub struct TenantListView {
    pub tenants: Vec<TenantView>,
    pub next_page_token: Option<String>,
}

/// Parsed list query parameters (`page_size`, `page_token`).
///
/// Parsed from the raw request URI query string rather than axum's `Query`
/// extractor, which is gated behind the `query` cargo feature not enabled in
/// the repo's owned axum build. The two params are simple unreserved values,
/// so a `&`/`=` split with `+`→space is the complete, correct decode.
#[derive(Clone, Debug, Default)]
struct ListQuery {
    page_size: Option<u32>,
    page_token: Option<String>,
}

fn parse_list_query(uri: &Uri) -> Result<ListQuery, HandlerError> {
    let mut out = ListQuery::default();
    let Some(raw) = uri.query() else {
        return Ok(out);
    };
    for pair in raw.split('&').filter(|p| !p.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = value.replace('+', " ");
        match name {
            "page_size" => {
                let parsed = value.parse::<u32>().map_err(|_| {
                    err(
                        StatusCode::BAD_REQUEST,
                        "INVALID_PAGE_SIZE",
                        format!("page_size {value:?} is not a non-negative integer"),
                    )
                })?;
                out.page_size = Some(parsed);
            }
            "page_token" => {
                if !value.is_empty() {
                    out.page_token = Some(value);
                }
            }
            other => {
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    "UNKNOWN_QUERY_PARAM",
                    format!("unknown query parameter {other:?}"),
                ));
            }
        }
    }
    Ok(out)
}

/// Error response body (AIP-193-shaped: machine code + human message).
#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub error: String,
}

type HandlerError = (StatusCode, Json<ErrorBody>);

fn err(status: StatusCode, code: &str, message: impl Into<String>) -> HandlerError {
    (
        status,
        Json(ErrorBody {
            code: code.to_owned(),
            error: message.into(),
        }),
    )
}

/// Map a contract `ProviderError` onto an HTTP status + body.
fn map_provider_error(error: ProviderError) -> HandlerError {
    let status = match &error {
        ProviderError::AlreadyExists { .. } => StatusCode::CONFLICT,
        ProviderError::NotFound { .. } => StatusCode::NOT_FOUND,
        ProviderError::IdempotencyKeyReuse { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        ProviderError::InvalidArgument { .. } => StatusCode::BAD_REQUEST,
        ProviderError::FailedPrecondition { .. } => StatusCode::CONFLICT,
        ProviderError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let code = match &error {
        ProviderError::AlreadyExists { .. } => "ALREADY_EXISTS",
        ProviderError::NotFound { .. } => "NOT_FOUND",
        ProviderError::IdempotencyKeyReuse { .. } => "IDEMPOTENCY_KEY_REUSE",
        ProviderError::InvalidArgument { .. } => "INVALID_ARGUMENT",
        ProviderError::FailedPrecondition { .. } => "FAILED_PRECONDITION",
        ProviderError::Internal { .. } => "INTERNAL",
    };
    err(status, code, error.to_string())
}

/// Parse the `Idempotency-Key` header into a contract idempotency key.
fn idempotency_key(headers: &HeaderMap) -> Result<IdempotencyKey, HandlerError> {
    let raw = headers
        .get("idempotency-key")
        .ok_or_else(|| {
            err(
                StatusCode::BAD_REQUEST,
                "IDEMPOTENCY_KEY_REQUIRED",
                "Idempotency-Key header is required for mutating requests",
            )
        })?
        .to_str()
        .map_err(|_| {
            err(
                StatusCode::BAD_REQUEST,
                "IDEMPOTENCY_KEY_MALFORMED",
                "Idempotency-Key header must be ASCII",
            )
        })?;
    IdempotencyKey::new(raw).map_err(|e| {
        err(
            StatusCode::BAD_REQUEST,
            "IDEMPOTENCY_KEY_MALFORMED",
            e.to_string(),
        )
    })
}

/// Build the `tenants/<id>` resource name, rejecting malformed ids.
fn tenant_name(tenant_id: &str) -> Result<ResourceName, HandlerError> {
    ResourceName::new(TENANT_COLLECTION, tenant_id).map_err(|e| {
        err(
            StatusCode::BAD_REQUEST,
            "INVALID_TENANT_ID",
            format!("tenant id {tenant_id:?} is not a valid resource id: {e}"),
        )
    })
}

// ============================================================
// Authentication + authorization (PEP)
// ============================================================

/// Authenticate the caller from the verified bearer credential, fail-closed.
///
/// Default-deny: a request with no matching bearer is UNAUTHENTICATED (401) —
/// the bearer is the ONLY authentication boundary, and it alone grants no axis.
/// A platform-admin bearer yields a platform-scoped caller (no tenant scope); a
/// tenant-operator bearer yields a caller scoped to the `x-oya-tenant` axis the
/// header asserts (bound only AFTER the bearer is verified). The URL `{id}` is
/// never consulted here — it is the resource the authorizer checks the caller
/// against, not a credential.
fn authenticate_caller<S>(
    state: &AppState<S>,
    headers: &HeaderMap,
) -> Result<CallerIdentity, HandlerError>
where
    S: LifecycleStore,
{
    if bearer_matches(headers, state.platform_admin_token.as_deref()) {
        return Ok(CallerIdentity {
            principal_id: "platform-admin".to_owned(),
            tenant_scope: None,
            platform_admin: true,
        });
    }
    if bearer_matches(headers, state.tenant_operator_token.as_deref()) {
        // The verified operator asserts which tenant it acts as via the axis
        // header. A missing axis = unauthenticated for tenant scope (the bearer
        // alone never grants a tenant axis); the PDP would otherwise deny, but
        // failing here keeps the 401/403 boundary crisp.
        let Some(axis) = headers
            .get(HEADER_TENANT_AXIS)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|axis| !axis.is_empty())
        else {
            return Err(err(
                StatusCode::UNAUTHORIZED,
                "UNAUTHENTICATED",
                "tenant-operator credential requires an x-oya-tenant axis assertion",
            ));
        };
        return Ok(CallerIdentity {
            principal_id: format!("tenant-operator:{axis}"),
            tenant_scope: Some(axis.to_owned()),
            platform_admin: false,
        });
    }
    Err(err(
        StatusCode::UNAUTHORIZED,
        "UNAUTHENTICATED",
        "a valid Bearer credential is required",
    ))
}

/// Enforce the authorization decision (the PEP). Authenticates the caller, asks
/// the embedded PDP, and maps the outcome: unauthenticated → 401, deny (or any
/// engine refusal) → 403 (fail-closed), allow → proceed.
fn authorize<S>(
    state: &AppState<S>,
    headers: &HeaderMap,
    action: TenantLifecycleAction,
    target_tenant_id: Option<&str>,
) -> Result<(), HandlerError>
where
    S: LifecycleStore,
{
    let caller = authenticate_caller(state, headers)?;
    let query = AuthorizationQuery {
        caller: &caller,
        action,
        target_tenant_id,
    };
    match state.authorizer.authorize(&query) {
        Ok(AuthorizationDecision::Allow) => Ok(()),
        // Authenticated-but-unauthorized, OR a fail-closed engine refusal:
        // both are a 403 (the caller is known; they simply may not do this).
        Ok(AuthorizationDecision::Deny) | Err(_) => Err(err(
            StatusCode::FORBIDDEN,
            "PERMISSION_DENIED",
            "caller is not authorized for this tenant action",
        )),
    }
}

// ============================================================
// Handlers
// ============================================================

/// `POST /v1/tenants` — register a new tenant (born `Provisioning`).
pub async fn register_tenant<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Json(body): Json<RegisterTenantBody>,
) -> Result<(StatusCode, Json<TenantView>), HandlerError>
where
    S: LifecycleStore,
{
    // Register is a platform-admin control-plane op (no target tenant): a
    // tenant-scoped caller can never reach it.
    authorize(&state, &headers, TenantLifecycleAction::Register, None)?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&body.tenant_id)?;
    let tenant = Tenant {
        tenant_id: body.tenant_id,
        display_name: body.display_name,
        // Born in the initial state; lifecycle moves go through operations.
        state: TenantLifecycleState::initial(),
        isolation_posture: body.isolation_posture,
        cell_id: body.cell_id,
        residency_zone: body.residency_zone,
    };
    let mut provider = state.lock();
    let outcome = provider
        .create(&name, tenant, &key)
        .map_err(map_provider_error)?;
    let status = if outcome.replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, Json(TenantView::from(outcome.resource))))
}

/// `GET /v1/tenants/{id}` — read the tenant's current state.
pub async fn get_tenant<S>(
    State(state): State<AppState<S>>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    let name = tenant_name(&tenant_id)?;
    // Per-tenant read: authorize the verified caller against the TARGET id.
    authorize(
        &state,
        &headers,
        TenantLifecycleAction::Read,
        Some(&tenant_id),
    )?;
    let provider = state.lock();
    let tenant = provider.get(&name).map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `GET /v1/tenants` — list tenants (AIP-158 paged).
pub async fn list_tenants<S>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Json<TenantListView>, HandlerError>
where
    S: LifecycleStore,
{
    // Listing discloses every tenant: platform-admin only (no target tenant).
    authorize(&state, &headers, TenantLifecycleAction::List, None)?;
    let query = parse_list_query(&uri)?;
    let page_size = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let mut request = PageRequest::first(page_size)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "INVALID_PAGE_SIZE", e.to_string()))?;
    if let Some(token) = query.page_token {
        let token = PageToken::new(token)
            .map_err(|e| err(StatusCode::BAD_REQUEST, "INVALID_PAGE_TOKEN", e.to_string()))?;
        request = request.after(token);
    }
    let provider = state.lock();
    let page = provider
        .list(TENANT_COLLECTION, &request)
        .map_err(map_provider_error)?;
    Ok(Json(TenantListView {
        tenants: page
            .items
            .into_iter()
            .map(|entry| TenantView::from(entry.resource))
            .collect(),
        next_page_token: page.next_page_token.map(|t| t.as_str().to_owned()),
    }))
}

/// Drive one lifecycle operation to completion and return the resulting
/// tenant view. Starts the AIP-151 operation, polls it once (the in-memory
/// provider completes synchronously on poll), and surfaces the converged
/// tenant — or the terminal operation error mapped onto an HTTP status.
fn run_lifecycle<S>(
    provider: &mut TenantLifecycleProvider<S>,
    name: &ResourceName,
    operation: TenantLifecycleOperation,
    key: &IdempotencyKey,
) -> Result<TenantLifecycleState, HandlerError>
where
    S: LifecycleStore,
{
    let started: Operation = provider
        .apply_lifecycle(name, operation, key)
        .map_err(map_provider_error)?;
    let terminal = provider
        .poll_operation(&started.name)
        .map_err(map_provider_error)?;
    match terminal.result {
        Some(OperationResult::Error(op_error)) => {
            let status = match op_error.code.as_str() {
                "not_found" => StatusCode::NOT_FOUND,
                "failed_precondition" => StatusCode::CONFLICT,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            Err(err(
                status,
                &op_error.code.to_ascii_uppercase(),
                op_error.message,
            ))
        }
        // Success (or, defensively, a re-read pending op): the observed state
        // is the authoritative outcome.
        Some(OperationResult::Response(_)) | None => match provider.observe_stored(name) {
            Ok(Some(tenant)) => Ok(tenant.state),
            Ok(None) => Err(err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("{name} no longer exists"),
            )),
            Err(e) => Err(map_provider_error(e)),
        },
    }
}

/// `POST /v1/tenants/{id}/provision` — drive `Provisioning -> Active`.
pub async fn provision_tenant<S>(
    State(state): State<AppState<S>>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize(
        &state,
        &headers,
        TenantLifecycleAction::Provision,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock();
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Activate, &key)?;
    let tenant = provider.get(&name).map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `POST /v1/tenants/{id}/suspend` — `Active -> Suspended`.
pub async fn suspend_tenant<S>(
    State(state): State<AppState<S>>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize(
        &state,
        &headers,
        TenantLifecycleAction::Suspend,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock();
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Suspend, &key)?;
    let tenant = provider.get(&name).map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `POST /v1/tenants/{id}/resume` — `Suspended -> Active`.
pub async fn resume_tenant<S>(
    State(state): State<AppState<S>>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize(
        &state,
        &headers,
        TenantLifecycleAction::Resume,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock();
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Resume, &key)?;
    let tenant = provider.get(&name).map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `DELETE /v1/tenants/{id}` — retire (terminal; the id is never reused).
pub async fn retire_tenant<S>(
    State(state): State<AppState<S>>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, HandlerError>
where
    S: LifecycleStore,
{
    // Retire is terminal and irreversible: authorize the verified caller
    // against the TARGET id before any FSM move.
    authorize(
        &state,
        &headers,
        TenantLifecycleAction::Retire,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock();
    // Retire IS the delete transition: start + poll to terminal.
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Retire, &key)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /healthz` — liveness probe.
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

// ============================================================
// Router + serve
// ============================================================

/// Build the axum router for the tenant lifecycle service over `provider` with
/// the REQUIRED authorizer + bearer-token configuration. There is no
/// authorizer-less overload: the only way to mount the routes is to supply a
/// fail-closed authorizer (no default-allow surface can be constructed).
pub fn build_router<S>(
    provider: TenantLifecycleProvider<S>,
    authorizer: SharedAuthorizer,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
) -> Router
where
    S: LifecycleStore + Sync,
{
    let state = AppState::new(
        provider,
        authorizer,
        platform_admin_token,
        tenant_operator_token,
    );
    Router::new()
        .route("/v1/tenants", post(register_tenant::<S>))
        .route("/v1/tenants", get(list_tenants::<S>))
        .route("/v1/tenants/{id}", get(get_tenant::<S>))
        .route("/v1/tenants/{id}", delete(retire_tenant::<S>))
        .route("/v1/tenants/{id}/provision", post(provision_tenant::<S>))
        .route("/v1/tenants/{id}/suspend", post(suspend_tenant::<S>))
        .route("/v1/tenants/{id}/resume", post(resume_tenant::<S>))
        .route("/healthz", get(healthz))
        .with_state(state)
}

/// Build a router backed by the in-memory store + the embedded PDP authorizer
/// over the seed tenancy bundle (single-node bring-up and tests). The bearer
/// tokens gate authentication; an absent token means that principal class
/// cannot authenticate (deny-all for it), never an open surface.
///
/// # Errors
/// [`BootError::Authz`] if the embedded tenancy authz bundle fails to compile
/// or strict-validate — the caller MUST refuse to serve (no default-allow).
pub fn build_inmemory_router(
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
) -> Result<Router, BootError> {
    let authorizer = PdpTenantLifecycleAuthorizer::from_seed_bundle()
        .map_err(|e| BootError::Authz(e.to_string()))?;
    Ok(build_router(
        TenantLifecycleProvider::new(InMemoryTenantLifecycleStore::new()),
        Arc::new(authorizer),
        platform_admin_token,
        tenant_operator_token,
    ))
}

/// Boot errors.
#[derive(Debug)]
pub enum BootError {
    /// TCP listener bind failure.
    Bind { address: String, error: String },
    /// Axum serve loop exited with an error.
    Serve(String),
    /// The authorization provider could not be composed (bundle compile /
    /// strict-validation failure). The service REFUSES to serve — there is no
    /// default-allow fallback when authz is unavailable.
    Authz(String),
    /// No bearer credential is configured at all (neither platform-admin nor
    /// tenant-operator). With no way to authenticate ANY caller the service
    /// would deny every request — refuse to start rather than serve a control
    /// plane no one can ever drive (a misconfiguration, not a security posture).
    NoCredentialConfigured,
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(e) => write!(f, "serve error: {e}"),
            Self::Authz(e) => write!(f, "authorization provider unavailable, refusing to serve: {e}"),
            Self::NoCredentialConfigured => write!(
                f,
                "no bearer credential configured ({ENV_PLATFORM_ADMIN_TOKEN} / \
                 {ENV_TENANT_OPERATOR_TOKEN}); refusing to start"
            ),
        }
    }
}

impl std::error::Error for BootError {}

/// Bind and serve the tenant lifecycle service on `listen_addr` over the
/// in-memory store, fail-closed. Production swaps a persistent store behind the
/// same port. The composition root REFUSES to serve when:
///   - the embedded authz bundle cannot compile/strict-validate ([`BootError::Authz`]),
///     so a misconfigured policy never degrades to default-allow; or
///   - no bearer credential is configured at all ([`BootError::NoCredentialConfigured`]).
///
/// # Errors
/// Returns [`BootError`] on authz/credential misconfiguration, a bind failure,
/// or a serve-loop exit.
pub async fn serve(listen_addr: &str) -> Result<(), BootError> {
    let platform_admin_token = normalize_token(std::env::var(ENV_PLATFORM_ADMIN_TOKEN).ok());
    let tenant_operator_token = normalize_token(std::env::var(ENV_TENANT_OPERATOR_TOKEN).ok());
    if platform_admin_token.is_none() && tenant_operator_token.is_none() {
        return Err(BootError::NoCredentialConfigured);
    }
    let app = build_inmemory_router(platform_admin_token, tenant_operator_token)?;
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|e| BootError::Bind {
            address: listen_addr.to_owned(),
            error: e.to_string(),
        })?;
    tracing::info!(addr = listen_addr, "tenancy-tenant-lifecycle listening (authz: embedded-pdp, fail-closed)");
    axum::serve(listener, app)
        .await
        .map_err(|e| BootError::Serve(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_view_round_trips_from_tenant() {
        let tenant = Tenant {
            tenant_id: "acme".to_owned(),
            display_name: "Acme Corp".to_owned(),
            state: TenantLifecycleState::Active,
            isolation_posture: IsolationPosture::Pooled,
            cell_id: "cell-001".to_owned(),
            residency_zone: Some("kr-seoul".to_owned()),
        };
        let view = TenantView::from(tenant.clone());
        assert_eq!(view.tenant_id, tenant.tenant_id);
        assert_eq!(view.state, TenantLifecycleState::Active);
        assert_eq!(view.residency_zone.as_deref(), Some("kr-seoul"));
    }

    #[test]
    fn provider_error_maps_to_expected_status() {
        let (status, _) = map_provider_error(ProviderError::NotFound {
            name: "tenants/ghost".to_owned(),
        });
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = map_provider_error(ProviderError::AlreadyExists {
            name: "tenants/acme".to_owned(),
        });
        assert_eq!(status, StatusCode::CONFLICT);
        let (status, _) = map_provider_error(ProviderError::IdempotencyKeyReuse {
            key: "k".to_owned(),
        });
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
