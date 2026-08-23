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

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use axum::Json;
use axum::Router;
use axum::extract::{DefaultBodyLimit, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};

use shared_platform_contracts_kernel::tenancy::{
    IsolationPosture, Tenant, TenantLifecycleOperation, TenantLifecycleState,
};
use shared_resource_provider_contract_kernel::{
    IdempotencyKey, Operation, OperationResult, PageRequest, PageToken, ProviderError,
    ResourceName, ResourceProvider,
};
use tenancy_tenant_lifecycle_authz_pdp::{
    NetworkPdpTenantLifecycleAuthorizer, PdpTenantLifecycleAuthorizer,
};
use tenancy_tenant_lifecycle_authz_port::{
    AuthorizationDecision, AuthorizationOutcome, AuthorizationQuery, CallerIdentity,
    TenantLifecycleAction, TenantLifecycleAuthorizer, TenantMembershipResolver,
};
use tenancy_tenant_lifecycle_kernel::TenantLifecycleStore;
use tenancy_tenant_lifecycle_store_inmemory::InMemoryTenantLifecycleStore;
use tenancy_tenant_lifecycle_store_postgres::PgTenantLifecycleStore;
use tenancy_tenant_lifecycle_usecase::{TENANT_COLLECTION, TenantLifecycleProvider};

/// The default page size used when a list request omits one.
const DEFAULT_PAGE_SIZE: u32 = 50;

/// Maximum accepted request-body size on the mutating routes (64 KiB). The
/// tenant-register/lifecycle bodies are tiny structured JSON; a sane cap turns
/// an oversized body into an early HTTP 413 (via [`DefaultBodyLimit`]) instead
/// of unbounded buffering. Authn still runs FIRST (a `FromRequestParts`
/// extractor precedes the `Json` body extractor), so an unauthenticated caller
/// is rejected 401 before any body is read; the limit is the second backstop
/// (a body-parse DoS surface defense). Mirrors the intelligence REST surface.
const MAX_MUTATING_BODY_BYTES: usize = 64 * 1024;

/// Env var carrying the platform-admin bearer token. A request presenting this
/// token (constant-time verified) is a platform admin: it may register and list
/// tenants, but it has NO tenant scope, so per-tenant ops still deny-by-default
/// at the PDP (the platform admin is not a tenant operator).
const ENV_PLATFORM_ADMIN_TOKEN: &str = "TENANCY_PLATFORM_ADMIN_TOKEN";

/// Env var carrying the tenant-operator bearer token. A request presenting this
/// token (constant-time verified) is *some* tenant operator — but the bearer
/// alone proves NEITHER which operator NOR which tenants it may act for (it is a
/// SHARED credential). The verified operator's authorized tenants are resolved
/// SERVER-SIDE from the [`TenantMembershipResolver`]; the `x-tenant` header
/// may at most SELECT among those assigned tenants. An operator can NEVER act for
/// a tenant it is not a member of (the C7 self-attestation fix), and the PDP then
/// backstops the membership-bound axis against the target {id}.
const ENV_TENANT_OPERATOR_TOKEN: &str = "TENANCY_TENANT_OPERATOR_TOKEN";

/// Env var carrying tenant-bound operator bearer credentials as
/// `tenantA:tokenA,tenantB:tokenB`. This is the #771 retirement path for the
/// shared operator bearer + self-asserted `x-tenant` selector: the verified
/// credential itself binds exactly one tenant axis. A conflicting
/// `x-tenant` header is denied, and an absent header is accepted because the
/// tenant came from the credential, not from caller data.
const ENV_TENANT_BOUND_OPERATOR_TOKENS: &str = "TENANCY_TENANT_BOUND_OPERATOR_TOKENS";

/// Env var carrying the comma-separated tenant-operator membership assignments
/// for the seed in-memory [`TenantMembershipResolver`], as
/// `operator_principal:tenantA|tenantB,operator2:tenantC`. The reference verifier
/// binds ALL operators presenting the shared bearer to one stable operator
/// principal id (`tenant-operator`); production swaps a per-credential verifier +
/// a cloud-iam membership adapter behind the unchanged port. An absent/empty
/// value means NO operator has any membership (every per-tenant op denies —
/// default-deny, never an open surface).
const ENV_TENANT_OPERATOR_MEMBERSHIPS: &str = "TENANCY_TENANT_OPERATOR_MEMBERSHIPS";

/// The stable principal id the reference bearer-verifier binds every caller
/// presenting the shared operator bearer to. Membership is resolved for THIS id.
/// Production replaces the shared bearer with a per-operator verified credential
/// (mTLS/SPIFFE/OIDC subject) whose subject is the membership key.
const REFERENCE_OPERATOR_PRINCIPAL_ID: &str = "tenant-operator";

/// Env var carrying the runtime Postgres connection URL for the durable
/// tenant-lifecycle store. When present (non-empty) the composition root wires
/// the durable [`PgTenantLifecycleStore`] behind the unchanged kernel port and
/// REFUSES to serve if the connection fails (fail-closed — never a silent
/// downgrade to in-memory). When absent, the in-memory store is used for
/// single-node dev bring-up. Reuses the repo-canonical Postgres URL env name
/// (`OYATIE_BACKBONE_POSTGRES_URL`, the same one the durable adapter's live tests
/// point at) so runtime and live-test config share one source of truth.
const ENV_DATABASE_URL: &str = "OYATIE_BACKBONE_POSTGRES_URL";

/// Optional network PDP endpoint/base URL for dogfooding the tenant lifecycle
/// PEP against the shared PDP REST surface. When present, boot composes a
/// network authorizer and REFUSES to fall back to embedded PDP if that endpoint
/// is invalid or unavailable.
const ENV_AUTHZ_PDP_URL: &str = "TENANCY_LIFECYCLE_AUTHZ_PDP_URL";

/// The header asserting which tenant a verified tenant-operator is acting as.
const HEADER_TENANT_AXIS: &str = "x-tenant";

/// The storage-port bound a served lifecycle provider needs: the kernel store
/// port, plus the thread/async-share markers axum handlers require. `Sync` is
/// required because the async `ResourceProvider` impl returns `&self`-borrowing
/// boxed futures that must be `Send` (a `&S` is `Send` only when `S: Sync`).
pub trait LifecycleStore: TenantLifecycleStore + Send + Sync + 'static {}

impl<T> LifecycleStore for T where T: TenantLifecycleStore + Send + Sync + 'static {}

/// The lifecycle provider behind a single composition-root lock.
/// `ResourceProvider` mutations take `&mut self`; the lock makes the provider
/// shareable across async axum handlers while keeping the operation ledger
/// single-writer (so idempotency holds).
///
/// Uses `tokio::sync::Mutex` (NOT `std::sync::Mutex`): the now-async provider
/// methods are `.await`ed while the guard is held, so the lock MUST be an async
/// lock whose guard is `Send` and is designed to be held across await points
/// (a `std::sync::MutexGuard` held across `.await` makes the handler future
/// `!Send` and is a deadlock-class footgun).
///
/// ## Concurrency seam (deliberate, single-node bring-up)
///
/// This is ONE coarse global lock over the whole provider — correct and simple
/// for the in-memory single-node bring-up, but it serializes all mutations.
/// Per-tenant / row-level concurrency is NOT this layer's concern: it moves
/// into the persistent store adapter behind the unchanged `TenantLifecycleStore`
/// port (the G03 data store does row-level locking / optimistic
/// concurrency), so the delivery surface keeps this single-writer invariant and
/// the store owns the contention model. Do not pre-optimize the in-memory lock.
pub type SharedProvider<S> = Arc<Mutex<TenantLifecycleProvider<S>>>;

/// The authorization decision port shared across handlers (the PEP's PDP).
pub type SharedAuthorizer = Arc<dyn TenantLifecycleAuthorizer>;

/// The server-side membership-resolution port shared across handlers. REQUIRED —
/// there is no "no resolver" variant; without it an operator's tenant axis could
/// only be self-attested, which is exactly the C7 finding this fix closes.
pub type SharedMembershipResolver = Arc<dyn TenantMembershipResolver>;

/// Tenant id -> tenant-bound operator bearer token. Tokens are boot-time config
/// only; production can replace this bridge with an OIDC/mTLS adapter while the
/// [`CallerIdentity`] and PDP boundary stay unchanged.
pub type TenantBoundOperatorTokens = BTreeMap<String, String>;

/// Application state injected into every handler.
pub struct AppState<S: LifecycleStore> {
    provider: SharedProvider<S>,
    /// The fail-closed authorizer (the embedded PDP). REQUIRED — there is no
    /// "no authorizer" variant: a service with no authz provider must never be
    /// constructed (the composition root refuses to build one).
    authorizer: SharedAuthorizer,
    /// The server-side tenant-membership resolver. REQUIRED — the verified
    /// operator's authorized tenants are resolved HERE, never self-attested via
    /// the `x-tenant` header (the C7 fix). There is no "no resolver" variant.
    membership: SharedMembershipResolver,
    /// Constant-time-verified platform-admin bearer token. `None` means no
    /// platform admin can authenticate (register/list deny-all), never an open
    /// surface.
    platform_admin_token: Option<String>,
    /// Constant-time-verified tenant-operator bearer token. `None` means no
    /// tenant operator can authenticate (per-tenant ops deny-all).
    tenant_operator_token: Option<String>,
    /// Constant-time-verified tenant-bound operator bearers. Each token binds a
    /// single verified tenant axis and does not require `x-tenant`.
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
}

impl<S: LifecycleStore> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
            authorizer: Arc::clone(&self.authorizer),
            membership: Arc::clone(&self.membership),
            platform_admin_token: self.platform_admin_token.clone(),
            tenant_operator_token: self.tenant_operator_token.clone(),
            tenant_bound_operator_tokens: self.tenant_bound_operator_tokens.clone(),
        }
    }
}

impl<S: LifecycleStore> AppState<S> {
    /// Wrap a lifecycle provider + the REQUIRED authorizer and bearer-token
    /// configuration for serving. There is no default-allow path: the
    /// authorizer is non-optional and the tokens gate authentication.
    #[must_use]
    fn new(
        provider: TenantLifecycleProvider<S>,
        authorizer: SharedAuthorizer,
        membership: SharedMembershipResolver,
        platform_admin_token: Option<String>,
        tenant_operator_token: Option<String>,
    ) -> Self {
        Self::new_with_tenant_bound_operator_tokens(
            provider,
            authorizer,
            membership,
            platform_admin_token,
            tenant_operator_token,
            BTreeMap::new(),
        )
    }

    /// Same as [`Self::new`], plus tenant-bound operator bearers for the #771
    /// migration path away from self-asserted tenant selection.
    #[must_use]
    fn new_with_tenant_bound_operator_tokens(
        provider: TenantLifecycleProvider<S>,
        authorizer: SharedAuthorizer,
        membership: SharedMembershipResolver,
        platform_admin_token: Option<String>,
        tenant_operator_token: Option<String>,
        tenant_bound_operator_tokens: TenantBoundOperatorTokens,
    ) -> Self {
        Self {
            provider: Arc::new(Mutex::new(provider)),
            authorizer,
            membership,
            platform_admin_token: normalize_token(platform_admin_token),
            tenant_operator_token: normalize_token(tenant_operator_token),
            tenant_bound_operator_tokens: normalize_tenant_bound_operator_tokens(
                tenant_bound_operator_tokens,
            ),
        }
    }

    /// Acquire the provider lock (async). `tokio::sync::Mutex` does NOT poison
    /// on a panic-while-held (unlike `std::sync::Mutex`), so there is no
    /// poison-recovery branch: a handler that panics simply releases the guard,
    /// and the service keeps serving (no single-panic denial of service). The
    /// contract FSM mutates the in-memory ledger transactionally per call, so
    /// the next acquirer always observes a consistent prior state.
    async fn lock(&self) -> MutexGuard<'_, TenantLifecycleProvider<S>> {
        self.provider.lock().await
    }
}

/// Trim a configured token and treat empty/whitespace-only as unset (so a blank
/// env var can never accidentally authenticate every caller).
fn normalize_token(token: Option<String>) -> Option<String> {
    token.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty())
}

fn normalize_tenant_bound_operator_tokens(
    tokens: TenantBoundOperatorTokens,
) -> TenantBoundOperatorTokens {
    tokens
        .into_iter()
        .filter_map(|(tenant, token)| {
            normalize_token(Some(token)).map(|token| (tenant.trim().to_owned(), token))
        })
        .filter(|(tenant, _)| !tenant.is_empty())
        .collect()
}

fn normalize_and_validate_credentials(
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Result<(Option<String>, Option<String>, TenantBoundOperatorTokens), BootError> {
    let platform_admin_token = normalize_token(platform_admin_token);
    let tenant_operator_token = normalize_token(tenant_operator_token);
    let tenant_bound_operator_tokens =
        normalize_tenant_bound_operator_tokens(tenant_bound_operator_tokens);
    validate_credentials(
        platform_admin_token.as_deref(),
        tenant_operator_token.as_deref(),
        &tenant_bound_operator_tokens,
    )?;
    Ok((
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
    ))
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

/// Borrow the optional tenant selector header. It is untrusted caller data:
/// shared-operator credentials may only use it to select an assigned tenant,
/// while tenant-bound credentials reject any conflicting value.
fn tenant_axis_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(HEADER_TENANT_AXIS)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|axis| !axis.is_empty())
}

/// Find the tenant bound to the presented tenant-bound operator bearer, scanning
/// the whole boot-time token map so the match position does not control the
/// loop length. Duplicate tokens are refused at boot by [`validate_credentials`].
fn matched_tenant_bound_operator<'a>(
    headers: &HeaderMap,
    tokens: &'a TenantBoundOperatorTokens,
) -> Option<&'a str> {
    let presented = bearer_token(headers)?;
    let mut matched_tenant = None;
    for (tenant, configured) in tokens {
        if constant_time_eq(presented.as_bytes(), configured.as_bytes()) {
            matched_tenant = Some(tenant.as_str());
        }
    }
    matched_tenant
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

/// Authenticate the caller from the verified bearer credential, fail-closed,
/// binding any tenant scope to SERVER-SIDE membership (never a self-attested
/// header).
///
/// Default-deny: a request with no matching bearer is UNAUTHENTICATED (401) — the
/// bearer is the ONLY authentication boundary, and it alone grants no tenant
/// axis. A platform-admin bearer yields a platform-scoped caller (no tenant
/// scope).
///
/// ## The C7 fix — membership-bound operator scope
///
/// A tenant-operator bearer proves only that the caller is *some* operator; it is
/// a SHARED credential and proves NOTHING about which tenants that operator may
/// act for. The `x-tenant` header therefore CANNOT grant a tenant axis. The
/// PEP resolves the verified operator's ASSIGNED tenants from the server-side
/// [`TenantMembershipResolver`] and binds the axis ONLY to a tenant in that set:
///   - no `x-tenant` header AND exactly one assigned tenant ⇒ bind to it;
///   - `x-tenant` present ⇒ SELECT it, but ONLY if it is in the assigned set;
///     an unassigned selection is 403 (the operator has no authority over it);
///   - no assigned tenants (unknown operator) ⇒ 403 (default-deny);
///   - a membership-store fault ⇒ 403 (fail-closed).
///
/// The URL `{id}` is never consulted here — it is the resource the authorizer
/// checks the membership-bound axis against, not a credential.
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
    if let Some(bound_tenant) =
        matched_tenant_bound_operator(headers, &state.tenant_bound_operator_tokens)
    {
        if let Some(requested) = tenant_axis_header(headers)
            && requested != bound_tenant
        {
            return Err(err(
                StatusCode::FORBIDDEN,
                "PERMISSION_DENIED",
                "tenant-bound credential does not match x-tenant selection",
            ));
        }

        return Ok(CallerIdentity {
            principal_id: format!("tenant-operator:{bound_tenant}"),
            // The tenant axis is credential-bound, not header-bound.
            tenant_scope: Some(bound_tenant.to_owned()),
            platform_admin: false,
        });
    }
    if bearer_matches(headers, state.tenant_operator_token.as_deref()) {
        // The verified operator's stable principal id is the membership key —
        // derived from the credential (the reference verifier binds the shared
        // bearer to one operator id), NEVER from a self-attested header.
        let operator_id = REFERENCE_OPERATOR_PRINCIPAL_ID;

        // Resolve the operator's ASSIGNED tenants SERVER-SIDE. A store fault is
        // fail-closed (403): a membership outage never grants a tenant axis.
        let assigned = state
            .membership
            .assigned_tenants(operator_id)
            .map_err(|fault| {
                tracing::warn!(
                    message = "tenancy.membership.fault",
                    operator_id = operator_id,
                    error = %fault,
                );
                err(
                    StatusCode::FORBIDDEN,
                    "PERMISSION_DENIED",
                    "tenant-operator membership could not be resolved",
                )
            })?;

        // An operator with NO assigned tenants is denied every per-tenant op
        // (default-deny; the shared bearer grants nothing on its own).
        if assigned.is_empty() {
            return Err(err(
                StatusCode::FORBIDDEN,
                "PERMISSION_DENIED",
                "tenant-operator has no assigned tenants",
            ));
        }

        // The header SELECTS among assigned tenants; it never grants one.
        let selected = match tenant_axis_header(headers) {
            Some(requested) => {
                // The selection MUST be a tenant the operator is assigned to.
                // A request for an UNASSIGNED tenant (the C7 attack: selecting a
                // victim tenant via the header) is forbidden — the bearer proves
                // no authority over it.
                if !assigned.iter().any(|t| t == requested) {
                    return Err(err(
                        StatusCode::FORBIDDEN,
                        "PERMISSION_DENIED",
                        "tenant-operator is not a member of the selected tenant",
                    ));
                }
                requested.to_owned()
            }
            // No selection header: only unambiguous when the operator is assigned
            // exactly one tenant. With several, the operator MUST disambiguate via
            // a header (which is still constrained to the assigned set).
            None => {
                if assigned.len() == 1 {
                    assigned[0].clone()
                } else {
                    return Err(err(
                        StatusCode::BAD_REQUEST,
                        "TENANT_SELECTION_REQUIRED",
                        "tenant-operator is assigned multiple tenants; an x-tenant \
                         selection (within the assigned set) is required",
                    ));
                }
            }
        };

        return Ok(CallerIdentity {
            principal_id: format!("tenant-operator:{selected}"),
            // The tenant axis is bound to a SERVER-SIDE-VERIFIED membership, not
            // the raw header — the PDP then backstops it against the target {id}.
            tenant_scope: Some(selected),
            platform_admin: false,
        });
    }
    Err(err(
        StatusCode::UNAUTHORIZED,
        "UNAUTHENTICATED",
        "a valid Bearer credential is required",
    ))
}

/// Enforce the authorization decision (the PEP) over an ALREADY-VERIFIED caller.
/// Asks the embedded PDP, emits a structured audit record, and maps the outcome:
/// deny (or any engine refusal) → 403 (fail-closed), allow → proceed.
///
/// The caller is authenticated UPSTREAM by the [`VerifiedCaller`]
/// `FromRequestParts` extractor, which runs BEFORE the request body is parsed —
/// so authn provably precedes body extraction (the C7/C8 authn-after-body fix).
/// This step runs ONLY the PDP/audit decision, preserving the membership-bound
/// axis, platform-admin path, deny-by-default, engine-refusal-as-deny, and the
/// single `"tenancy.authz.decision"` audit record unchanged.
///
/// Every decision — allow AND deny — emits ONE structured `tracing` event at
/// INFO level with message `"tenancy.authz.decision"` (AC-W-13). The event
/// carries the attributable fields from the [`AuthorizationOutcome`] returned
/// by the PDP; discarding the outcome is a policy violation.
fn authorize_decision<S>(
    state: &AppState<S>,
    caller: &CallerIdentity,
    action: TenantLifecycleAction,
    target_tenant_id: Option<&str>,
) -> Result<(), HandlerError>
where
    S: LifecycleStore,
{
    let query = AuthorizationQuery {
        caller,
        action,
        target_tenant_id,
    };
    match state.authorizer.authorize(&query) {
        Ok(AuthorizationOutcome {
            decision,
            decision_id,
            determining_policy_ids,
        }) => {
            // Emit ONE structured audit record for EVERY decision (AC-W-13).
            // Both allow and deny are audited — the audit trail is the forensic
            // surface; a missing record for a deny is a security gap.
            let decision_str = match decision {
                AuthorizationDecision::Allow => "allow",
                AuthorizationDecision::Deny => "deny",
            };
            tracing::info!(
                message = "tenancy.authz.decision",
                decision_id = %decision_id,
                principal_id = %caller.principal_id,
                action = %action.slug(),
                target_tenant = ?target_tenant_id,
                decision = %decision_str,
                determining_policy_ids = ?determining_policy_ids,
            );
            if decision == AuthorizationDecision::Allow {
                Ok(())
            } else {
                // Authenticated-but-unauthorized: 403 (the caller is known;
                // they simply may not do this).
                Err(err(
                    StatusCode::FORBIDDEN,
                    "PERMISSION_DENIED",
                    "caller is not authorized for this tenant action",
                ))
            }
        }
        // Fail-closed engine refusal: treat as deny, surface as 403. The PDP
        // minted no decision id (it refused before deciding), but the deny still
        // belongs on the forensic surface — emit it so a probe that induces
        // engine refusals is never invisible (AC-W-13: every deny is audited).
        Err(error) => {
            tracing::warn!(
                message = "tenancy.authz.decision",
                decision_id = "",
                principal_id = %caller.principal_id,
                action = %action.slug(),
                target_tenant = ?target_tenant_id,
                decision = "deny",
                reason = "engine-refused",
                error = %error,
            );
            Err(err(
                StatusCode::FORBIDDEN,
                "PERMISSION_DENIED",
                "caller is not authorized for this tenant action",
            ))
        }
    }
}

// ============================================================
// VerifiedCaller — authn-BEFORE-body extractor (the C7/C8 fix)
// ============================================================

/// A request extractor that authenticates the caller from the verified bearer
/// credential (and membership-binds any operator tenant axis) over the request
/// PARTS — i.e. BEFORE the request body is read.
///
/// ## Why this exists (the authn-after-body fix)
///
/// `FromRequestParts` extractors run BEFORE the body `FromRequest` extractor.
/// Placing `VerifiedCaller` ahead of `Json(body)` in a handler signature
/// therefore GUARANTEES authentication runs before the body is buffered or
/// deserialized: an unauthenticated caller is short-circuited 401 (or a
/// membership-denied operator 403) WITHOUT the body ever being parsed, closing
/// the pre-auth body-parse / parser-attack DoS surface. (A `DefaultBodyLimit`
/// on the route is the second backstop, capping body size to a 413.)
///
/// It carries the verified [`CallerIdentity`]; the handler then runs ONLY the
/// PDP decision via [`authorize_decision`] against the action + target — the
/// membership-bound axis, platform-admin path, audit, and fail-closed posture
/// are all preserved unchanged.
pub struct VerifiedCaller(pub CallerIdentity);

impl<S> FromRequestParts<AppState<S>> for VerifiedCaller
where
    S: LifecycleStore,
{
    type Rejection = HandlerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<S>,
    ) -> Result<Self, Self::Rejection> {
        // Authenticate over the request HEADERS only (parts) — the body is not
        // touched here, so authn provably precedes body extraction.
        let caller = authenticate_caller(state, &parts.headers)?;
        Ok(Self(caller))
    }
}

// ============================================================
// Handlers
// ============================================================

/// `POST /v1/tenants` — register a new tenant (born `Provisioning`).
///
/// `VerifiedCaller` (a `FromRequestParts` extractor) precedes `Json(body)` so
/// authn runs BEFORE the body is buffered/parsed: an unauthenticated request is
/// rejected 401 without the body ever being read (the authn-after-body fix).
pub async fn register_tenant<S>(
    State(state): State<AppState<S>>,
    VerifiedCaller(caller): VerifiedCaller,
    headers: HeaderMap,
    Json(body): Json<RegisterTenantBody>,
) -> Result<(StatusCode, Json<TenantView>), HandlerError>
where
    S: LifecycleStore,
{
    // Register is a platform-admin control-plane op (no target tenant): a
    // tenant-scoped caller can never reach it. The caller is ALREADY verified
    // by the `VerifiedCaller` extractor (which ran before this body was parsed);
    // run only the PDP decision here.
    authorize_decision(&state, &caller, TenantLifecycleAction::Register, None)?;
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
    let mut provider = state.lock().await;
    let outcome = provider
        .create(&name, tenant, &key)
        .await
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
    VerifiedCaller(caller): VerifiedCaller,
    Path(tenant_id): Path<String>,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    // Per-tenant read: authorize the verified caller against the TARGET id
    // BEFORE input validation — consistent with all mutating handlers so no
    // validation signal (e.g. INVALID_TENANT_ID vs PERMISSION_DENIED) leaks
    // information about resource existence to an unauthenticated caller.
    authorize_decision(
        &state,
        &caller,
        TenantLifecycleAction::Read,
        Some(&tenant_id),
    )?;
    let name = tenant_name(&tenant_id)?;
    let provider = state.lock().await;
    let tenant = provider.get(&name).await.map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `GET /v1/tenants` — list tenants (AIP-158 paged).
pub async fn list_tenants<S>(
    State(state): State<AppState<S>>,
    VerifiedCaller(caller): VerifiedCaller,
    uri: Uri,
) -> Result<Json<TenantListView>, HandlerError>
where
    S: LifecycleStore,
{
    // Listing discloses every tenant: platform-admin only (no target tenant).
    authorize_decision(&state, &caller, TenantLifecycleAction::List, None)?;
    let query = parse_list_query(&uri)?;
    let page_size = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let mut request = PageRequest::first(page_size)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "INVALID_PAGE_SIZE", e.to_string()))?;
    if let Some(token) = query.page_token {
        let token = PageToken::new(token)
            .map_err(|e| err(StatusCode::BAD_REQUEST, "INVALID_PAGE_TOKEN", e.to_string()))?;
        request = request.after(token);
    }
    let provider = state.lock().await;
    let page = provider
        .list(TENANT_COLLECTION, &request)
        .await
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
async fn run_lifecycle<S>(
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
        .await
        .map_err(map_provider_error)?;
    let terminal = provider
        .poll_operation(&started.name)
        .await
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
        Some(OperationResult::Response(_)) | None => match provider.observe_stored(name).await {
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
    VerifiedCaller(caller): VerifiedCaller,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize_decision(
        &state,
        &caller,
        TenantLifecycleAction::Provision,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock().await;
    run_lifecycle(
        &mut provider,
        &name,
        TenantLifecycleOperation::Activate,
        &key,
    )
    .await?;
    let tenant = provider.get(&name).await.map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `POST /v1/tenants/{id}/suspend` — `Active -> Suspended`.
pub async fn suspend_tenant<S>(
    State(state): State<AppState<S>>,
    VerifiedCaller(caller): VerifiedCaller,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize_decision(
        &state,
        &caller,
        TenantLifecycleAction::Suspend,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock().await;
    run_lifecycle(
        &mut provider,
        &name,
        TenantLifecycleOperation::Suspend,
        &key,
    )
    .await?;
    let tenant = provider.get(&name).await.map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `POST /v1/tenants/{id}/resume` — `Suspended -> Active`.
pub async fn resume_tenant<S>(
    State(state): State<AppState<S>>,
    VerifiedCaller(caller): VerifiedCaller,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TenantView>, HandlerError>
where
    S: LifecycleStore,
{
    authorize_decision(
        &state,
        &caller,
        TenantLifecycleAction::Resume,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock().await;
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Resume, &key).await?;
    let tenant = provider.get(&name).await.map_err(map_provider_error)?;
    Ok(Json(TenantView::from(tenant)))
}

/// `DELETE /v1/tenants/{id}` — retire (terminal; the id is never reused).
pub async fn retire_tenant<S>(
    State(state): State<AppState<S>>,
    VerifiedCaller(caller): VerifiedCaller,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, HandlerError>
where
    S: LifecycleStore,
{
    // Retire is terminal and irreversible: authorize the verified caller
    // against the TARGET id before any FSM move.
    authorize_decision(
        &state,
        &caller,
        TenantLifecycleAction::Retire,
        Some(&tenant_id),
    )?;
    let key = idempotency_key(&headers)?;
    let name = tenant_name(&tenant_id)?;
    let mut provider = state.lock().await;
    // Retire IS the delete transition: start + poll to terminal.
    run_lifecycle(&mut provider, &name, TenantLifecycleOperation::Retire, &key).await?;
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
fn build_router<S>(
    provider: TenantLifecycleProvider<S>,
    authorizer: SharedAuthorizer,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
) -> Router
where
    S: LifecycleStore + Sync,
{
    build_router_with_tenant_bound_operator_tokens(
        provider,
        authorizer,
        membership,
        platform_admin_token,
        tenant_operator_token,
        BTreeMap::new(),
    )
}

/// Build the axum router with tenant-bound operator bearers for the #771
/// migration path away from self-asserted tenant-axis selection.
fn build_router_with_tenant_bound_operator_tokens<S>(
    provider: TenantLifecycleProvider<S>,
    authorizer: SharedAuthorizer,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Router
where
    S: LifecycleStore + Sync,
{
    let state = AppState::new_with_tenant_bound_operator_tokens(
        provider,
        authorizer,
        membership,
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
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
        // Cap every request body to a sane bound (HTTP 413 past it). The body
        // is never even read on an unauthenticated request (authn runs in the
        // `VerifiedCaller` `FromRequestParts` extractor BEFORE body extraction);
        // this limit is the second backstop against an oversized-body DoS.
        .layer(DefaultBodyLimit::max(MAX_MUTATING_BODY_BYTES))
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
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
) -> Result<Router, BootError> {
    build_inmemory_router_with_tenant_bound_operator_tokens(
        membership,
        platform_admin_token,
        tenant_operator_token,
        BTreeMap::new(),
    )
}

/// Build an in-memory router with tenant-bound operator bearers.
pub fn build_inmemory_router_with_tenant_bound_operator_tokens(
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Result<Router, BootError> {
    let authorizer = PdpTenantLifecycleAuthorizer::from_seed_bundle()
        .map_err(|e| BootError::Authz(e.to_string()))?;
    build_inmemory_router_with_authorizer_and_tenant_bound_operator_tokens(
        Arc::new(authorizer),
        membership,
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
    )
}

/// Build an in-memory router with an explicit authorizer. This is the network
/// PDP dogfood seam: callers that configured a network PDP pass that authorizer
/// here; failures happen before this function, never as fallback to embedded.
pub fn build_inmemory_router_with_authorizer_and_tenant_bound_operator_tokens(
    authorizer: SharedAuthorizer,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Result<Router, BootError> {
    let (platform_admin_token, tenant_operator_token, tenant_bound_operator_tokens) =
        normalize_and_validate_credentials(
            platform_admin_token,
            tenant_operator_token,
            tenant_bound_operator_tokens,
        )?;
    Ok(build_router_with_tenant_bound_operator_tokens(
        TenantLifecycleProvider::new(InMemoryTenantLifecycleStore::new()),
        authorizer,
        membership,
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
    ))
}

/// Build a router backed by the DURABLE Postgres store + the embedded PDP
/// authorizer over the seed tenancy bundle. Mirrors [`build_inmemory_router`]
/// but composes [`PgTenantLifecycleStore`] (the ADR-0510 transitional durable
/// adapter behind the unchanged `TenantLifecycleStore` kernel port — cutover
/// litmus holds: the data/G003 owned store swaps the adapter later with no
/// change here). The bearer tokens gate authentication exactly as the in-memory
/// path; an absent token means that principal class cannot authenticate.
///
/// # Errors
/// - [`BootError::Store`] if the durable store cannot connect (empty URL or
///   sqlx connect failure), or if the RLS-enforceability guard fires:
///   - the connected role carries `rolsuper` or `rolbypassrls` (bypass-capable;
///     the adapter refuses to serve), OR
///   - the connected role is not a member of the `tenancy_lifecycle_runtime`
///     policy-subject role (policies would not apply; deny-all outage).
///     The caller MUST refuse to serve — there is NO fallback to in-memory.
///     Note: the guard is necessary but not sufficient for full tenant isolation;
///     full isolation additionally requires that `tenancy_lifecycle_runtime`
///     exists provisioned with NOBYPASSRLS (deferred `0000_runtime_role.sql`,
///     mirroring data-outbox-adapter-postgres).
/// - [`BootError::Authz`] if the embedded tenancy authz bundle fails to compile
///   or strict-validate (no default-allow).
pub async fn build_postgres_router(
    database_url: &str,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
) -> Result<Router, BootError> {
    build_postgres_router_with_tenant_bound_operator_tokens(
        database_url,
        membership,
        platform_admin_token,
        tenant_operator_token,
        BTreeMap::new(),
    )
    .await
}

/// Build a durable Postgres router with tenant-bound operator bearers.
pub async fn build_postgres_router_with_tenant_bound_operator_tokens(
    database_url: &str,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Result<Router, BootError> {
    let authorizer = PdpTenantLifecycleAuthorizer::from_seed_bundle()
        .map_err(|e| BootError::Authz(e.to_string()))?;
    build_postgres_router_with_authorizer_and_tenant_bound_operator_tokens(
        database_url,
        Arc::new(authorizer),
        membership,
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
    )
    .await
}

/// Build a durable Postgres router with an explicit authorizer. A configured
/// network PDP reaches this only after successful composition; store failures
/// still refuse boot and never downgrade to in-memory.
pub async fn build_postgres_router_with_authorizer_and_tenant_bound_operator_tokens(
    database_url: &str,
    authorizer: SharedAuthorizer,
    membership: SharedMembershipResolver,
    platform_admin_token: Option<String>,
    tenant_operator_token: Option<String>,
    tenant_bound_operator_tokens: TenantBoundOperatorTokens,
) -> Result<Router, BootError> {
    let (platform_admin_token, tenant_operator_token, tenant_bound_operator_tokens) =
        normalize_and_validate_credentials(
            platform_admin_token,
            tenant_operator_token,
            tenant_bound_operator_tokens,
        )?;
    let store = PgTenantLifecycleStore::connect(database_url)
        .await
        .map_err(|e| BootError::Store(e.to_string()))?;
    // Fail-closed RLS-enforceability guard: refuse to serve if the connected
    // role can bypass Postgres RLS (rolsuper or rolbypassrls), OR if it is
    // not a member of the tenancy_lifecycle_runtime policy-subject role (in
    // which case the tenant-isolation policies would not apply at all). This
    // check runs at boot so a mis-provisioned DSN fails loudly.
    //
    // Guard scope: necessary but not sufficient for full isolation. Full
    // isolation additionally requires tenancy_lifecycle_runtime to exist
    // provisioned with NOBYPASSRLS (deferred 0000_runtime_role.sql follow-up).
    store
        .assert_rls_enforceable()
        .await
        .map_err(|e| BootError::Store(e.to_string()))?;
    Ok(build_router_with_tenant_bound_operator_tokens(
        TenantLifecycleProvider::new(store),
        authorizer,
        membership,
        platform_admin_token,
        tenant_operator_token,
        tenant_bound_operator_tokens,
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
    /// The durable tenant store could not be composed: empty/invalid
    /// `DATABASE_URL`, an sqlx connect failure, or the RLS-enforceability
    /// guard fired (connected role carries `rolsuper`/`rolbypassrls`, or is
    /// not a member of the `tenancy_lifecycle_runtime` policy-subject role).
    /// The service REFUSES to serve rather than silently downgrade to the
    /// in-memory store or allow isolation to be bypassed.
    ///
    /// Note: the guard is necessary but not sufficient for full tenant
    /// isolation; full isolation additionally requires that
    /// `tenancy_lifecycle_runtime` exists provisioned with NOBYPASSRLS
    /// (deferred `0000_runtime_role.sql` follow-up).
    Store(String),
    /// No bearer credential is configured at all (platform-admin, legacy shared
    /// tenant-operator, or tenant-bound operator). With no way to authenticate
    /// ANY caller the service would deny every request — refuse to start rather
    /// than serve a control plane no one can ever drive (a misconfiguration, not
    /// a security posture).
    NoCredentialConfigured,
    /// Two credential authorities are configured with the same bearer token.
    /// Because `authenticate_caller` checks platform-admin first, then
    /// tenant-bound operator tokens, then the legacy shared tenant operator,
    /// a duplicate would silently bind one bearer to multiple authorities.
    /// Refuse to start — fail-closed at boot rather than serve an ambiguous
    /// credential.
    AmbiguousCredential,
    /// The tenant-bound operator token config is malformed. Refuse to serve
    /// rather than silently skip a credential and leave an operator unable to
    /// administer its tenant.
    InvalidTenantBoundOperatorTokenConfig(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(e) => write!(f, "serve error: {e}"),
            Self::Authz(e) => write!(
                f,
                "authorization provider unavailable, refusing to serve: {e}"
            ),
            Self::Store(e) => write!(f, "tenant store unavailable, refusing to serve: {e}"),
            Self::NoCredentialConfigured => write!(
                f,
                "no bearer credential configured ({ENV_PLATFORM_ADMIN_TOKEN} / \
                 {ENV_TENANT_OPERATOR_TOKEN} / {ENV_TENANT_BOUND_OPERATOR_TOKENS}); \
                 refusing to start"
            ),
            Self::AmbiguousCredential => write!(
                f,
                "{ENV_PLATFORM_ADMIN_TOKEN}, {ENV_TENANT_OPERATOR_TOKEN}, and/or \
                 {ENV_TENANT_BOUND_OPERATOR_TOKENS} contain an ambiguous token; a \
                 shared value would silently bind one bearer to multiple authorities — \
                 refusing to start"
            ),
            Self::InvalidTenantBoundOperatorTokenConfig(detail) => {
                write!(f, "{ENV_TENANT_BOUND_OPERATOR_TOKENS} is invalid: {detail}")
            }
        }
    }
}

impl std::error::Error for BootError {}

// ============================================================
// Reference in-memory membership resolver (composition-root adapter)
// ============================================================

/// Reference [`TenantMembershipResolver`] adapter: an in-memory map of operator
/// principal id → assigned tenant ids, for single-node bring-up and tests. This
/// is the SERVER-SIDE source of truth for which tenants an operator may act for;
/// production swaps a cloud-iam / OIDC membership-store adapter behind the
/// unchanged port. An operator absent from the map resolves to an EMPTY set
/// (default-deny — every per-tenant op for that operator is forbidden).
#[derive(Clone, Debug, Default)]
pub struct InMemoryTenantMembershipResolver {
    assignments: std::collections::BTreeMap<String, Vec<String>>,
}

impl InMemoryTenantMembershipResolver {
    /// Build an empty resolver (no operator has any membership → all per-tenant
    /// ops deny).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign `tenant_ids` to `operator_principal_id` (builder-style).
    #[must_use]
    pub fn with_operator(
        mut self,
        operator_principal_id: impl Into<String>,
        tenant_ids: impl IntoIterator<Item = String>,
    ) -> Self {
        self.assignments.insert(
            operator_principal_id.into(),
            tenant_ids.into_iter().collect(),
        );
        self
    }

    /// Parse the operator-membership env value into a resolver. The grammar is
    /// `operator:tenantA|tenantB,operator2:tenantC`. A `None`/empty value yields
    /// an empty resolver (default-deny); malformed entries are skipped (they
    /// grant nothing — fail-closed). The reference verifier binds every operator
    /// bearer to [`REFERENCE_OPERATOR_PRINCIPAL_ID`], so a value without an
    /// explicit operator prefix (`tenantA|tenantB`) is assigned to that id.
    #[must_use]
    pub fn from_env(raw: Option<&str>) -> Self {
        let mut resolver = Self::new();
        let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
            return resolver;
        };
        for entry in raw.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (operator, tenants_raw) = match entry.split_once(':') {
                Some((op, tenants)) => (op.trim().to_owned(), tenants),
                // No explicit operator prefix: bind to the reference operator id.
                None => (REFERENCE_OPERATOR_PRINCIPAL_ID.to_owned(), entry),
            };
            if operator.is_empty() {
                continue;
            }
            let tenants: Vec<String> = tenants_raw
                .split('|')
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(str::to_owned)
                .collect();
            if !tenants.is_empty() {
                resolver.assignments.insert(operator, tenants);
            }
        }
        resolver
    }
}

impl TenantMembershipResolver for InMemoryTenantMembershipResolver {
    fn assigned_tenants(
        &self,
        operator_principal_id: &str,
    ) -> Result<Vec<String>, tenancy_tenant_lifecycle_authz_port::MembershipFault> {
        // Unknown operator ⇒ empty set ⇒ default-deny. This adapter never faults
        // (in-memory), so the fail-closed `Err` branch is exercised by production
        // adapters; the contract is documented on the port.
        Ok(self
            .assignments
            .get(operator_principal_id)
            .cloned()
            .unwrap_or_default())
    }
}

/// The store backend selected by [`select_store_kind`] from the runtime config.
///
/// This enum is the authoritative decision socket: the no-fallback property
/// (a configured Postgres URL must NEVER silently degrade to in-memory) is
/// enforced structurally — only `None`/empty URL maps to `InMemory`.
#[derive(Debug, PartialEq, Eq)]
pub enum StoreSelection {
    /// Durable Postgres store; the contained URL is non-empty.
    Postgres(String),
    /// In-memory store (single-node dev bring-up; no URL configured).
    InMemory,
}

/// Pure store-selection function: maps the raw (pre-normalized) database URL
/// option onto a [`StoreSelection`]. `None`, empty, or whitespace-only strings
/// all select `InMemory`; any non-empty trimmed URL selects `Postgres`.
///
/// Extracted from `serve()` so the no-fallback decision can be unit-tested
/// without a network, a runtime, or any side effects.
pub fn select_store_kind(database_url: Option<String>) -> StoreSelection {
    match normalize_token(database_url) {
        Some(url) => StoreSelection::Postgres(url),
        None => StoreSelection::InMemory,
    }
}

/// The authorization backend selected by [`select_authorizer_kind`] from boot
/// config.
///
/// This is the no-fallback decision socket: only an absent/empty endpoint uses
/// the embedded PDP. A configured endpoint means network PDP or boot failure.
#[derive(Debug, PartialEq, Eq)]
pub enum AuthorizerSelection {
    /// Embedded in-process Cedar PDP over the seed tenancy bundle.
    Embedded,
    /// Network PDP REST endpoint/base URL.
    Network(String),
}

/// Pure authorization-backend selection function. `None`, empty, or
/// whitespace-only strings select the embedded PDP; any non-empty value selects
/// network PDP and must never silently fall back if composition fails.
pub fn select_authorizer_kind(pdp_url: Option<String>) -> AuthorizerSelection {
    match normalize_token(pdp_url) {
        Some(url) => AuthorizerSelection::Network(url),
        None => AuthorizerSelection::Embedded,
    }
}

fn compose_authorizer(
    selection: AuthorizerSelection,
) -> Result<(SharedAuthorizer, &'static str), BootError> {
    match selection {
        AuthorizerSelection::Embedded => {
            let authorizer = PdpTenantLifecycleAuthorizer::from_seed_bundle()
                .map_err(|e| BootError::Authz(e.to_string()))?;
            Ok((Arc::new(authorizer), "embedded-pdp"))
        }
        AuthorizerSelection::Network(endpoint_or_base_url) => {
            let authorizer =
                NetworkPdpTenantLifecycleAuthorizer::from_endpoint_with_readiness_preflight(
                    &endpoint_or_base_url,
                )
                .map_err(|e| BootError::Authz(e.to_string()))?;
            Ok((Arc::new(authorizer), "network-pdp"))
        }
    }
}

/// Parse tenant-bound operator tokens from boot config. Malformed entries are
/// boot-fatal rather than silently skipped.
fn parse_tenant_bound_operator_tokens(
    raw: Option<&str>,
) -> Result<TenantBoundOperatorTokens, BootError> {
    let mut tokens = BTreeMap::new();
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(tokens);
    };

    for entry in raw.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let Some((tenant_raw, token_raw)) = entry.split_once(':') else {
            return Err(BootError::InvalidTenantBoundOperatorTokenConfig(
                "expected entries shaped tenant_id:token".to_owned(),
            ));
        };
        let tenant = tenant_raw.trim();
        if tenant.is_empty() {
            return Err(BootError::InvalidTenantBoundOperatorTokenConfig(
                "tenant id is empty".to_owned(),
            ));
        }
        ResourceName::new(TENANT_COLLECTION, tenant).map_err(|e| {
            BootError::InvalidTenantBoundOperatorTokenConfig(format!(
                "tenant id {tenant:?} is invalid: {e}"
            ))
        })?;
        let token = normalize_token(Some(token_raw.to_owned())).ok_or_else(|| {
            BootError::InvalidTenantBoundOperatorTokenConfig(format!(
                "token for tenant {tenant:?} is empty"
            ))
        })?;
        if tokens.insert(tenant.to_owned(), token).is_some() {
            return Err(BootError::InvalidTenantBoundOperatorTokenConfig(format!(
                "tenant {tenant:?} is configured more than once"
            )));
        }
    }

    Ok(tokens)
}

/// Pure boot-time bearer-credential validation (no env, no I/O — unit-testable).
///
/// Fail-closed when no credential exists, or when one bearer value maps to more
/// than one authority. `authenticate_caller` checks platform-admin first, then
/// tenant-bound tokens, then the legacy shared operator token, so any duplicate
/// would silently bind the same bearer to the wrong authority unless boot
/// refuses it here.
fn validate_credentials(
    platform_admin_token: Option<&str>,
    tenant_operator_token: Option<&str>,
    tenant_bound_operator_tokens: &TenantBoundOperatorTokens,
) -> Result<(), BootError> {
    if platform_admin_token.is_none()
        && tenant_operator_token.is_none()
        && tenant_bound_operator_tokens.is_empty()
    {
        return Err(BootError::NoCredentialConfigured);
    }

    if let (Some(admin), Some(operator)) = (platform_admin_token, tenant_operator_token)
        && constant_time_eq(admin.as_bytes(), operator.as_bytes())
    {
        return Err(BootError::AmbiguousCredential);
    }

    for (tenant, token) in tenant_bound_operator_tokens {
        if let Some(admin) = platform_admin_token
            && constant_time_eq(token.as_bytes(), admin.as_bytes())
        {
            return Err(BootError::AmbiguousCredential);
        }
        if let Some(operator) = tenant_operator_token
            && constant_time_eq(token.as_bytes(), operator.as_bytes())
        {
            return Err(BootError::AmbiguousCredential);
        }

        // ponytail: boot-only env-sized list; O(n^2) avoids a token-index
        // abstraction. Replace with fingerprints if this becomes dynamic.
        for (other_tenant, other_token) in tenant_bound_operator_tokens {
            if tenant < other_tenant && constant_time_eq(token.as_bytes(), other_token.as_bytes()) {
                return Err(BootError::AmbiguousCredential);
            }
        }
    }

    Ok(())
}

/// Bind and serve the tenant lifecycle service on `listen_addr`, fail-closed.
///
/// ## Store selection (12-factor composition-root config, NOT a CLI surface)
///   - `ENV_DATABASE_URL` (`OYATIE_BACKBONE_POSTGRES_URL`) present + non-empty →
///     the DURABLE [`PgTenantLifecycleStore`] is composed. If the connection
///     fails or the RLS-enforceability guard fires (bypass-capable role, or
///     role not a member of `tenancy_lifecycle_runtime`), the service REFUSES
///     to serve ([`BootError::Store`]) — it NEVER falls back to in-memory.
///     Note: the guard is necessary but not sufficient for full isolation;
///     full isolation requires `tenancy_lifecycle_runtime` to exist
///     provisioned with NOBYPASSRLS (deferred `0000_runtime_role.sql`).
///   - absent / empty → the in-memory store (single-node dev bring-up).
///
/// In both cases a persistent store plugs in behind the unchanged kernel port,
/// so the owned-data cutover (G003) swaps the adapter with no change here.
///
/// The composition root additionally REFUSES to serve when:
///   - the embedded authz bundle cannot compile/strict-validate ([`BootError::Authz`]),
///     so a misconfigured policy never degrades to default-allow; or
///   - no bearer credential is configured at all ([`BootError::NoCredentialConfigured`]).
///
/// # Errors
/// Returns [`BootError`] on store/authz/credential misconfiguration, a bind
/// failure, or a serve-loop exit.
pub async fn serve(listen_addr: &str) -> Result<(), BootError> {
    let platform_admin_token = normalize_token(std::env::var(ENV_PLATFORM_ADMIN_TOKEN).ok());
    let tenant_operator_token = normalize_token(std::env::var(ENV_TENANT_OPERATOR_TOKEN).ok());
    let tenant_bound_operator_tokens = parse_tenant_bound_operator_tokens(
        std::env::var(ENV_TENANT_BOUND_OPERATOR_TOKENS)
            .ok()
            .as_deref(),
    )?;
    validate_credentials(
        platform_admin_token.as_deref(),
        tenant_operator_token.as_deref(),
        &tenant_bound_operator_tokens,
    )?;
    // The server-side membership resolver (REQUIRED). Seeded from the operator
    // membership env var; an absent value yields an empty resolver (every
    // per-tenant op denies — default-deny, never self-attested).
    let membership: SharedMembershipResolver =
        Arc::new(InMemoryTenantMembershipResolver::from_env(
            std::env::var(ENV_TENANT_OPERATOR_MEMBERSHIPS)
                .ok()
                .as_deref(),
        ));
    let (authorizer, authorizer_kind) = compose_authorizer(select_authorizer_kind(
        std::env::var(ENV_AUTHZ_PDP_URL).ok(),
    ))?;
    let (app, store_kind) = match select_store_kind(std::env::var(ENV_DATABASE_URL).ok()) {
        StoreSelection::Postgres(url) => (
            // FAIL-CLOSED: a connect failure or RLS-bypass role propagates;
            // NEVER fall back to in-memory when a durable backend was configured.
            build_postgres_router_with_authorizer_and_tenant_bound_operator_tokens(
                &url,
                Arc::clone(&authorizer),
                Arc::clone(&membership),
                platform_admin_token,
                tenant_operator_token,
                tenant_bound_operator_tokens,
            )
            .await?,
            "postgres",
        ),
        StoreSelection::InMemory => (
            build_inmemory_router_with_authorizer_and_tenant_bound_operator_tokens(
                Arc::clone(&authorizer),
                Arc::clone(&membership),
                platform_admin_token,
                tenant_operator_token,
                tenant_bound_operator_tokens,
            )?,
            "inmemory",
        ),
    };
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|e| BootError::Bind {
            address: listen_addr.to_owned(),
            error: e.to_string(),
        })?;
    tracing::info!(
        addr = listen_addr,
        store = store_kind,
        authz = authorizer_kind,
        "tenancy-tenant-lifecycle listening (fail-closed)"
    );
    axum::serve(listener, app)
        .await
        .map_err(|e| BootError::Serve(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_readyz_server() -> String {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind readyz");
        let addr = listener.local_addr().expect("readyz addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept readyz");
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            let body = r#"{"status":"ready"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write readyz");
        });
        format!("http://{addr}")
    }

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

    // --- select_store_kind unit tests (pure, no async, no DB) ----------------

    #[test]
    fn select_store_kind_non_empty_url_picks_postgres() {
        assert_eq!(
            select_store_kind(Some("postgres://host/db".to_owned())),
            StoreSelection::Postgres("postgres://host/db".to_owned()),
        );
    }

    #[test]
    fn select_store_kind_none_picks_inmemory() {
        assert_eq!(select_store_kind(None), StoreSelection::InMemory);
    }

    #[test]
    fn select_store_kind_empty_string_picks_inmemory() {
        // An empty DATABASE_URL is treated as "not configured" — the dev
        // in-memory path, not a fail-closed error. This is the same rule
        // normalize_token uses (trim + filter empty).
        assert_eq!(
            select_store_kind(Some(String::new())),
            StoreSelection::InMemory,
        );
    }

    #[test]
    fn select_store_kind_whitespace_only_picks_inmemory() {
        // Whitespace-only is also treated as "not configured".
        assert_eq!(
            select_store_kind(Some("   ".to_owned())),
            StoreSelection::InMemory,
        );
    }

    // --- select_authorizer_kind unit tests (pure, no socket) ------------------

    #[test]
    fn select_authorizer_kind_none_picks_embedded() {
        assert_eq!(select_authorizer_kind(None), AuthorizerSelection::Embedded);
    }

    #[test]
    fn select_authorizer_kind_non_empty_url_picks_network() {
        assert_eq!(
            select_authorizer_kind(Some("https://pdp.internal".to_owned())),
            AuthorizerSelection::Network("https://pdp.internal".to_owned()),
        );
    }

    #[test]
    fn compose_network_authorizer_never_falls_back_to_embedded_on_bad_endpoint() {
        let result = compose_authorizer(AuthorizerSelection::Network(
            "http://pdp.internal".to_owned(),
        ));

        assert!(matches!(result, Err(BootError::Authz(_))));
    }

    #[test]
    fn compose_loopback_network_authorizer_reports_network_kind() {
        let endpoint = spawn_readyz_server();
        let (_authorizer, kind) = compose_authorizer(AuthorizerSelection::Network(endpoint))
            .expect("loopback network PDP endpoint with readyz preflight is valid for tests");

        assert_eq!(kind, "network-pdp");
    }

    // --- DB-free fail-closed wiring proof ------------------------------------

    /// Fail-closed wiring proof WITHOUT a database: an empty `DATABASE_URL`
    /// maps from `PgStoreConnectError::MissingDatabaseUrl` to
    /// [`BootError::Store`], so the durable path refuses to compose a router
    /// rather than silently degrading. No socket / no Postgres is touched.
    #[tokio::test]
    async fn build_postgres_router_empty_url_is_store_error() {
        let membership: SharedMembershipResolver =
            Arc::new(InMemoryTenantMembershipResolver::new());
        let result = build_postgres_router(
            "",
            membership,
            Some("platform".to_owned()),
            Some("operator".to_owned()),
        )
        .await;
        assert!(
            matches!(result, Err(BootError::Store(_))),
            "empty DATABASE_URL must fail-close as BootError::Store, got {result:?}"
        );
    }

    // --- validate_credentials (FIX 3: ambiguous-credential boot guard) -------

    #[test]
    fn validate_credentials_distinct_tokens_ok() {
        assert!(
            validate_credentials(
                Some("admin-secret"),
                Some("operator-secret"),
                &BTreeMap::new()
            )
            .is_ok()
        );
    }

    #[test]
    fn validate_credentials_only_one_configured_ok() {
        // A single principal class is a valid posture (the other simply can't
        // authenticate); only ZERO-configured or EQUAL-both are boot errors.
        assert!(validate_credentials(Some("admin-secret"), None, &BTreeMap::new()).is_ok());
        assert!(validate_credentials(None, Some("operator-secret"), &BTreeMap::new()).is_ok());
        assert!(
            validate_credentials(
                None,
                None,
                &BTreeMap::from([("acme".to_owned(), "acme-secret".to_owned())])
            )
            .is_ok()
        );
    }

    #[test]
    fn validate_credentials_none_configured_is_no_credential() {
        assert!(matches!(
            validate_credentials(None, None, &BTreeMap::new()),
            Err(BootError::NoCredentialConfigured)
        ));
    }

    #[test]
    fn validate_credentials_equal_tokens_is_ambiguous_credential() {
        // A shared platform-admin/operator token would silently escalate every
        // operator to platform-admin (admin is checked first) — refuse to boot.
        assert!(matches!(
            validate_credentials(
                Some("shared-secret"),
                Some("shared-secret"),
                &BTreeMap::new()
            ),
            Err(BootError::AmbiguousCredential)
        ));
        assert!(matches!(
            validate_credentials(
                Some("shared-secret"),
                None,
                &BTreeMap::from([("acme".to_owned(), "shared-secret".to_owned())])
            ),
            Err(BootError::AmbiguousCredential)
        ));
        assert!(matches!(
            validate_credentials(
                None,
                Some("shared-secret"),
                &BTreeMap::from([("acme".to_owned(), "shared-secret".to_owned())])
            ),
            Err(BootError::AmbiguousCredential)
        ));
        assert!(matches!(
            validate_credentials(
                None,
                None,
                &BTreeMap::from([
                    ("acme".to_owned(), "shared-secret".to_owned()),
                    ("globex".to_owned(), "shared-secret".to_owned()),
                ])
            ),
            Err(BootError::AmbiguousCredential)
        ));
    }

    #[test]
    fn tenant_bound_operator_token_env_parses_and_rejects_malformed_entries() {
        let tokens =
            parse_tenant_bound_operator_tokens(Some("acme: acme-secret, globex:globex-secret"))
                .unwrap();
        assert_eq!(tokens.get("acme").map(String::as_str), Some("acme-secret"));
        assert_eq!(
            tokens.get("globex").map(String::as_str),
            Some("globex-secret")
        );

        assert!(parse_tenant_bound_operator_tokens(None).unwrap().is_empty());
        assert!(
            parse_tenant_bound_operator_tokens(Some("   "))
                .unwrap()
                .is_empty()
        );
        assert!(matches!(
            parse_tenant_bound_operator_tokens(Some("acme")),
            Err(BootError::InvalidTenantBoundOperatorTokenConfig(_))
        ));
        assert!(matches!(
            parse_tenant_bound_operator_tokens(Some("acme:")),
            Err(BootError::InvalidTenantBoundOperatorTokenConfig(_))
        ));
        assert!(matches!(
            parse_tenant_bound_operator_tokens(Some("bad tenant:secret")),
            Err(BootError::InvalidTenantBoundOperatorTokenConfig(_))
        ));
    }

    #[test]
    fn tenant_bound_public_builder_rejects_ambiguous_normalized_tokens() {
        let membership: SharedMembershipResolver =
            Arc::new(InMemoryTenantMembershipResolver::new());
        let result = build_inmemory_router_with_tenant_bound_operator_tokens(
            membership,
            None,
            None,
            BTreeMap::from([
                ("acme".to_owned(), "shared-secret".to_owned()),
                ("globex".to_owned(), " shared-secret ".to_owned()),
            ]),
        );
        assert!(matches!(result, Err(BootError::AmbiguousCredential)));
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

    // --- membership resolver (the C7 server-side binding) --------------------

    #[test]
    fn membership_resolver_unknown_operator_is_empty_default_deny() {
        let resolver = InMemoryTenantMembershipResolver::new()
            .with_operator("op-a", ["t1".to_owned(), "t2".to_owned()]);
        // A known operator resolves its assigned set.
        assert_eq!(
            resolver.assigned_tenants("op-a").unwrap(),
            vec!["t1".to_owned(), "t2".to_owned()]
        );
        // An UNKNOWN operator resolves to the empty set (default-deny), never a
        // wildcard — the self-attested header can never grant a tenant.
        assert!(resolver.assigned_tenants("op-unknown").unwrap().is_empty());
    }

    #[test]
    fn membership_from_env_parses_grammar_and_defaults_to_reference_operator() {
        // Explicit operator prefixes.
        let resolver = InMemoryTenantMembershipResolver::from_env(Some("op-a:t1|t2, op-b:t3"));
        assert_eq!(
            resolver.assigned_tenants("op-a").unwrap(),
            vec!["t1".to_owned(), "t2".to_owned()]
        );
        assert_eq!(
            resolver.assigned_tenants("op-b").unwrap(),
            vec!["t3".to_owned()]
        );

        // A bare `tenantA|tenantB` (no operator prefix) binds to the reference id.
        let bare = InMemoryTenantMembershipResolver::from_env(Some("acme|globex"));
        assert_eq!(
            bare.assigned_tenants(REFERENCE_OPERATOR_PRINCIPAL_ID)
                .unwrap(),
            vec!["acme".to_owned(), "globex".to_owned()]
        );

        // None / empty / whitespace ⇒ empty resolver (default-deny, no allow-all).
        for raw in [None, Some(""), Some("   ")] {
            let r = InMemoryTenantMembershipResolver::from_env(raw);
            assert!(
                r.assigned_tenants(REFERENCE_OPERATOR_PRINCIPAL_ID)
                    .unwrap()
                    .is_empty(),
                "empty/absent membership config must grant nothing, raw={raw:?}"
            );
        }
    }
}
