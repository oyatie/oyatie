//! Users subsystem: SCIM 2.0 provisioning surface (RFC 7644) over
//! `oya-shared-scim-server-kernel`.
//!
//! Tenant-scoped base path `/scim/v2/{tenant}` (the Zitadel/Okta provisioning
//! pattern; the kernel's `meta.location` URLs use the same shape). The kernel
//! `ReferenceScimServer` owns all SCIM semantics over the `UserStore`/
//! `GroupStore` PORTS — the durable store lands behind those ports (G03).
//!
//! ## Fail-closed guard
//! Every route requires a Bearer workload token validated against the SAME
//! static JWKS + issuer/audience as the authorize surface AND carrying the
//! `scim.manage` scope: missing/invalid credential -> `401`, missing scope ->
//! `403`. The guard then binds the verified token tenant to the path tenant and
//! runs the normal workload-identity PEP path (repository lookup, denylist, and
//! Cedar action `identity.scim.Manage`) before any SCIM store mutation/read.
//!
//! ## Audit emission (PRD §3.3 / AC-W-13)
//! Every guard run that carries a credential emits exactly one immutable
//! `AuditRecord` through the workload [`AuditSink`] port: the PEP hot path
//! (`authorize_token_for`) emits the decision record itself, and the guard's
//! own pre-authorize refusals (invalid credential, missing scope, tenant
//! mismatch) each emit one deny-class record here. A request with no
//! `Authorization` header carries no credential and renders no decision, so —
//! like a body that fails to parse on the `/authorize` surfaces — it emits no
//! record. Audit emission never fails the request path (the sink contract
//! swallows sink errors); a refusal stands regardless of sink health.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};

use oya_shared_scim_server_kernel::{
    CounterIdGen, GroupStore, ListQuery, NewGroup, NewUser, PatchOp, ReferenceScimServer,
    ScimError, ScimId, ScimServer, TenantId, UserStore,
};

use iam_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository,
};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{Action, ClaimValue, Resource};
use iam_identity_workload_oidc::{Jwks, ValidationConfig, validate_workload_token};
use iam_identity_workload_rest::{AuditEvent, AuditRecord, AuditSink, WorkloadAuthzState};

/// The SCIM scope a caller's workload token must carry.
pub const SCIM_MANAGE_SCOPE: &str = "scim.manage";

/// Tenant-scoped SCIM base path (RFC 7644 §3.2 endpoints hang off this).
pub const SCIM_BASE: &str = "/scim/v2";
/// Tenant-scoped account registration base path for the product-facing
/// identity API. This is a thin API-native wrapper over the same SCIM create
/// path; SCIM remains the canonical provisioning contract underneath.
pub const ACCOUNT_REGISTRATION_BASE: &str = "/identity/v1";
const SCIM_USERS_ROUTE: &str = "/scim/v2/{tenant}/Users";
const SCIM_USER_ROUTE: &str = "/scim/v2/{tenant}/Users/{id}";
const SCIM_GROUPS_ROUTE: &str = "/scim/v2/{tenant}/Groups";
const SCIM_GROUP_ROUTE: &str = "/scim/v2/{tenant}/Groups/{id}";
const ACCOUNT_REGISTRATION_ROUTE: &str = "/identity/v1/{tenant}/account-registrations";

/// The composed SCIM surface state: the kernel reference server over the
/// [`UserStore`]/[`GroupStore`] PORTS + the offline token-validation material.
/// Generic over the same four workload-identity ports as [`WorkloadAuthzState`]
/// (mirroring the REST router) PLUS the two SCIM store ports `U`/`G`, so the
/// composition root selects in-memory (dev) or the durable G005 Postgres stores
/// behind the SAME traits — the kernel `ReferenceScimServer` and the
/// `UserStore`/`GroupStore` ports are UNCHANGED (cutover litmus: the owned-data
/// store swaps the adapter later with no change here).
pub struct ScimSurfaceState<R, D, A, S, U, G>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    server: ReferenceScimServer<U, G, CounterIdGen>,
    jwks: Jwks,
    validation: ValidationConfig,
    now_provider: fn() -> i64,
    authz: Arc<WorkloadAuthzState<R, D, A, S>>,
}

impl<R, D, A, S, U, G> ScimSurfaceState<R, D, A, S, U, G>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    /// Assemble the surface over the supplied SCIM store ports. `base_url` is the
    /// externally visible SCIM base (issuer + `/scim/v2`), used for
    /// `meta.location` URLs. The composition root passes either the in-memory
    /// stores (dev) or the durable Postgres stores (G005) — both satisfy the
    /// `UserStore`/`GroupStore` ports, so this signature is store-agnostic.
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        jwks: Jwks,
        validation: ValidationConfig,
        now_provider: fn() -> i64,
        authz: Arc<WorkloadAuthzState<R, D, A, S>>,
        users: U,
        groups: G,
    ) -> Self {
        Self {
            server: ReferenceScimServer::new(users, groups, CounterIdGen::new(), base_url),
            jwks,
            validation,
            now_provider,
            authz,
        }
    }
}

/// Shared handle for the SCIM routes.
pub type SharedScimState<R, D, A, S, U, G> = Arc<ScimSurfaceState<R, D, A, S, U, G>>;

// =====================================================================
// Fail-closed Bearer guard
// =====================================================================

/// The Cedar action every SCIM guard decision authorizes.
const SCIM_MANAGE_ACTION: &str = "identity.scim.Manage";

/// The Cedar resource type the guard scopes decisions to.
const SCIM_TENANT_RESOURCE_TYPE: &str = "ScimTenant";

/// Emit the single deny-class audit record for a guard refusal that occurred
/// BEFORE the PEP hot path ran (invalid credential / missing scope / tenant
/// mismatch). Same record shape + vocabulary as the authorize surfaces; the
/// raw token never reaches the record. The `detail` carries the refusal
/// stage (`scim-guard` for a rejected credential, `scope-missing`,
/// `tenant-mismatch`) — deliberately richer than the PEP's `token-rejected`
/// record (`detail: None`) so forensics can tell WHICH surface refused.
fn audit_guard_refusal<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    workload_id: Option<String>,
    outcome: &'static str,
    detail: &str,
    tenant: &str,
) where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    state.authz.audit().record(
        AuditRecord::new(
            AuditEvent::Authorize,
            workload_id,
            outcome,
            Some(detail.to_owned()),
        )
        .with_authorization_target(SCIM_MANAGE_ACTION, SCIM_TENANT_RESOURCE_TYPE, tenant),
    );
}

/// Validate the Bearer credential and require [`SCIM_MANAGE_SCOPE`].
/// `Err` is the ready-to-return refusal response (401/403) — never a pass.
/// Exactly one audit record is emitted per credential-bearing guard run:
/// pre-authorize refusals emit here, the PEP decision emits inside
/// `authorize_token_for` (see the module-level audit section).
fn authorize_scim<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    headers: &HeaderMap,
    tenant: &str,
) -> Result<(), Box<Response>>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        // No credential -> no decision rendered -> no audit record (module doc).
        .ok_or_else(|| scim_refusal(StatusCode::UNAUTHORIZED, "missing bearer credential"))?;
    let principal = validate_workload_token(
        token,
        &state.jwks,
        &state.validation,
        (state.now_provider)(),
    )
    .map_err(|_| {
        audit_guard_refusal(state, None, "token-rejected", "scim-guard", tenant);
        scim_refusal(StatusCode::UNAUTHORIZED, "invalid bearer credential")
    })?;
    if !principal.has_scope(SCIM_MANAGE_SCOPE) {
        audit_guard_refusal(
            state,
            Some(principal.workload_id().as_str().to_owned()),
            "deny",
            "scope-missing",
            tenant,
        );
        return Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential lacks the scim.manage scope",
        ));
    }
    if principal.tenant_id().as_str() != tenant {
        audit_guard_refusal(
            state,
            Some(principal.workload_id().as_str().to_owned()),
            "deny",
            "tenant-mismatch",
            tenant,
        );
        return Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential tenant does not match the SCIM tenant",
        ));
    }
    let mut context = BTreeMap::new();
    context.insert(
        "scim_tenant".to_owned(),
        ClaimValue::Text(tenant.to_owned()),
    );
    match state.authz.authorize_token_for(
        token,
        Action::new(SCIM_MANAGE_ACTION),
        Resource::new(SCIM_TENANT_RESOURCE_TYPE, tenant.to_owned()),
        context,
    ) {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => Ok(()),
        AuthorizeOutcome::TokenRejected => Err(scim_refusal(
            StatusCode::UNAUTHORIZED,
            "invalid bearer credential",
        )),
        AuthorizeOutcome::StoreUnavailable => Err(scim_refusal(
            StatusCode::SERVICE_UNAVAILABLE,
            "identity authorization store unavailable",
        )),
        AuthorizeOutcome::PrincipalUnknown => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential principal is not registered",
        )),
        AuthorizeOutcome::Revoked => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential principal is revoked",
        )),
        AuthorizeOutcome::Decided(_) => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential is not permitted to manage SCIM",
        )),
    }
}

fn scim_refusal(status: StatusCode, detail: &str) -> Box<Response> {
    Box::new((status, Json(ScimError::new(status.as_u16(), None, detail))).into_response())
}

fn scim_error_response(error: ScimError) -> Response {
    let status = StatusCode::from_u16(error.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(error)).into_response()
}

// =====================================================================
// Handlers
// =====================================================================

/// Parse `startIndex`/`count`/`filter` per RFC 7644 §3.4.2.4 (1-indexed)
/// from the request URI. Hand-rolled key=value split rather than the serde
/// `Query` extractor: the workspace axum build excludes the `query` feature.
/// Unparseable values fall back to the kernel defaults (a malformed page
/// param must not fail provisioning reads).
fn page_query(uri: &Uri) -> ListQuery {
    let mut query = ListQuery::default();
    for pair in uri.query().unwrap_or_default().split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "startIndex" => {
                if let Ok(value) = value.parse() {
                    query.start_index = value;
                }
            }
            "count" => {
                if let Ok(value) = value.parse() {
                    query.items_per_page = value;
                }
            }
            "filter" if !value.is_empty() => {
                query.filter = Some(value.to_owned());
            }
            _ => {}
        }
    }
    query
}

macro_rules! guard {
    ($state:expr, $headers:expr, $tenant:expr) => {
        if let Err(refusal) = authorize_scim(&$state, &$headers, &$tenant) {
            return *refusal;
        }
    };
}

async fn list_users<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .list_users(&TenantId(tenant), &page_query(&uri))
        .await
    {
        Ok(listing) => Json(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn create_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .create_user(&TenantId(tenant), input, (state.now_provider)())
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn register_account<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .create_user(&TenantId(tenant), input, (state.now_provider)())
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn get_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state.server.get_user(&TenantId(tenant), &ScimId(id)).await {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn replace_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .replace_user(
            &TenantId(tenant),
            &ScimId(id),
            input,
            (state.now_provider)(),
        )
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn patch_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(op): Json<PatchOp>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .patch_user(&TenantId(tenant), &ScimId(id), &op, (state.now_provider)())
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn delete_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .delete_user(&TenantId(tenant), &ScimId(id))
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn list_groups<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .list_groups(&TenantId(tenant), &page_query(&uri))
        .await
    {
        Ok(listing) => Json::<_>(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn create_group<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .create_group(&TenantId(tenant), input, (state.now_provider)())
        .await
    {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn get_group<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state.server.get_group(&TenantId(tenant), &ScimId(id)).await {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn patch_group<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(op): Json<PatchOp>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .patch_group(&TenantId(tenant), &ScimId(id), &op, (state.now_provider)())
        .await
    {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn delete_group<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    match state
        .server
        .delete_group(&TenantId(tenant), &ScimId(id))
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}

/// Build the SCIM router (mounted at the service root; paths carry
/// [`SCIM_BASE`]).
pub fn build_scim_router<R, D, A, S, U, G>(state: SharedScimState<R, D, A, S, U, G>) -> Router
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    Router::new()
        .route(SCIM_USERS_ROUTE, get(list_users::<R, D, A, S, U, G>))
        .route(SCIM_USERS_ROUTE, post(create_user::<R, D, A, S, U, G>))
        .route(
            ACCOUNT_REGISTRATION_ROUTE,
            post(register_account::<R, D, A, S, U, G>),
        )
        .route(SCIM_USER_ROUTE, get(get_user::<R, D, A, S, U, G>))
        .route(SCIM_USER_ROUTE, put(replace_user::<R, D, A, S, U, G>))
        .route(SCIM_USER_ROUTE, patch(patch_user::<R, D, A, S, U, G>))
        .route(SCIM_USER_ROUTE, delete(delete_user::<R, D, A, S, U, G>))
        .route(SCIM_GROUPS_ROUTE, get(list_groups::<R, D, A, S, U, G>))
        .route(SCIM_GROUPS_ROUTE, post(create_group::<R, D, A, S, U, G>))
        .route(SCIM_GROUP_ROUTE, get(get_group::<R, D, A, S, U, G>))
        .route(SCIM_GROUP_ROUTE, patch(patch_group::<R, D, A, S, U, G>))
        .route(SCIM_GROUP_ROUTE, delete(delete_group::<R, D, A, S, U, G>))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use oya_shared_scim_server_kernel::{InMemoryGroupStore, InMemoryUserStore};
    use tower::ServiceExt as _;

    use iam_identity_workload_app::{
        InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository,
        WorkloadPrincipalRepository,
    };
    use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
    use iam_identity_workload_domain::{WorkloadPrincipal, WorkloadState};
    use iam_identity_workload_oidc::Jwk;
    use iam_identity_workload_rest::{BearerCallerVerifier, InMemoryAuditSink, WorkloadAuthzState};

    use crate::decision_authz::TenantScopedDecisionAuthorizer;
    use crate::lifecycle_authz::TenantScopedLifecycleAuthorizer;

    const ISSUER: &str = "https://idp.oyatie.com";
    const AUDIENCE: &str = "oya-identity-scim";
    const KID: &str = "kid-scim-1";
    const NOW: i64 = 1_700_000_000;

    struct Fixture {
        router: Router,
        token: String,
        wrong_scope_token: String,
        unknown_principal_token: String,
        /// Registered + scoped principal in `ten_other` — passes every guard
        /// pre-check, then Cedar default-denies (the policy permits `ten_acme`
        /// only): the pure policy-deny path.
        cedar_denied_token: String,
        /// Inspection handle onto the SAME audit log the state emits to
        /// (clones share the underlying append-only store).
        audit: InMemoryAuditSink,
    }

    fn b64(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn mint(
        key_pair: &EcdsaKeyPair,
        rng: &SystemRandom,
        tenant: &str,
        workload: &str,
        scope: &str,
    ) -> String {
        let claims = format!(
            r#"{{"iss":"{ISSUER}","aud":"{AUDIENCE}","exp":{},"iat":{NOW},"tenant_id":"{tenant}","sub":"{workload}","owning_capability":"cap.identity.scim","scope":"{scope}"}}"#,
            NOW + 300
        );
        let header = format!(r#"{{"alg":"ES256","typ":"JWT","kid":"{KID}"}}"#);
        let input = format!("{}.{}", b64(header.as_bytes()), b64(claims.as_bytes()));
        let sig = key_pair.sign(rng, input.as_bytes()).expect("sign");
        format!("{input}.{}", b64(sig.as_ref()))
    }

    fn fixture() -> Fixture {
        let rng = SystemRandom::new();
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).expect("pkcs8");
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
            .expect("key");
        let public = key_pair.public_key().as_ref();
        let jwks =
            Jwks::new().add_key(Jwk::ec_p256(KID, b64(&public[1..33]), b64(&public[33..65])));
        let mut repository = InMemoryWorkloadPrincipalRepository::new();
        for (tenant, workload) in [
            ("ten_acme", "wl_provisioner"),
            ("ten_other", "wl_other_provisioner"),
        ] {
            let mut provisioner =
                WorkloadPrincipal::provision(tenant, workload, "cap.identity.scim")
                    .expect("principal");
            provisioner
                .transition_to(WorkloadState::Active)
                .expect("activate");
            repository.save(&provisioner).expect("seed principal");
        }
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("permit-scim-manage")
            permit(
              principal,
              action == Action::"identity.scim.Manage",
              resource is ScimTenant
            ) when {
              principal.tenant_id == "ten_acme"
            };
            "#,
        )
        .expect("policy");
        let audit = InMemoryAuditSink::new();
        let authz = Arc::new(WorkloadAuthzState::with_clock(
            repository,
            InMemoryRevocationDenylist::new(),
            authorizer,
            jwks.clone(),
            ValidationConfig::new(ISSUER, AUDIENCE),
            audit.clone(),
            Arc::new(BearerCallerVerifier::new(
                "scim-test-lifecycle-bearer",
                "ten_acme",
                "scim-test-control-plane",
            )),
            Arc::new(TenantScopedLifecycleAuthorizer::new()),
            Arc::new(TenantScopedDecisionAuthorizer::new()),
            || NOW,
        ));
        let state = Arc::new(ScimSurfaceState::new(
            format!("{ISSUER}{SCIM_BASE}"),
            jwks,
            ValidationConfig::new(ISSUER, AUDIENCE),
            || NOW,
            authz,
            InMemoryUserStore::default(),
            InMemoryGroupStore::default(),
        ));
        Fixture {
            router: build_scim_router(state),
            token: mint(
                &key_pair,
                &rng,
                "ten_acme",
                "wl_provisioner",
                SCIM_MANAGE_SCOPE,
            ),
            wrong_scope_token: mint(
                &key_pair,
                &rng,
                "ten_acme",
                "wl_provisioner",
                "cloud.kms.decrypt",
            ),
            unknown_principal_token: mint(
                &key_pair,
                &rng,
                "ten_acme",
                "wl_not_seeded",
                SCIM_MANAGE_SCOPE,
            ),
            cedar_denied_token: mint(
                &key_pair,
                &rng,
                "ten_other",
                "wl_other_provisioner",
                SCIM_MANAGE_SCOPE,
            ),
            audit,
        }
    }

    async fn send(
        router: &Router,
        method: &str,
        uri: &str,
        bearer: Option<&str>,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = axum::http::Request::builder().method(method).uri(uri);
        if let Some(token) = bearer {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        let request = match body {
            Some(json) => request
                .header("content-type", "application/json")
                .body(axum::body::Body::from(json.to_string())),
            None => request.body(axum::body::Body::empty()),
        }
        .expect("request");
        let response = router.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .expect("body");
        let value = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("json body")
        };
        (status, value)
    }

    /// RFC 7644 camelCase wire shape, absent members omitted — what a real
    /// SCIM client (Okta/Entra) sends. The legacy snake_case alias shape is
    /// covered by the kernel tests and the service E2E.
    fn new_user_payload(user_name: &str) -> serde_json::Value {
        serde_json::json!({
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "userName": user_name,
            "displayName": "Provisioned User",
            "active": true,
        })
    }

    #[tokio::test]
    async fn refuses_missing_and_invalid_credentials_with_401() {
        let fixture = fixture();
        let (status, body) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["schemas"][0], ScimError::SCHEMA);
        // No credential presented -> no decision rendered -> no audit record.
        assert!(fixture.audit.is_empty());

        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some("not-a-jwt"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        // An invalid credential IS audited: one token-rejected record.
        assert_eq!(fixture.audit.len(), 1);
        let record = &fixture.audit.records()[0];
        assert_eq!(record.event(), AuditEvent::Authorize);
        assert_eq!(record.workload_id(), None);
        assert_eq!(record.outcome(), "token-rejected");
        assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
        assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
        assert_eq!(record.resource_id(), Some("ten_acme"));
    }

    #[tokio::test]
    async fn refuses_missing_scope_with_403() {
        let fixture = fixture();
        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some(&fixture.wrong_scope_token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        // Exactly one deny record for the scope refusal.
        assert_eq!(fixture.audit.len(), 1);
        let record = &fixture.audit.records()[0];
        assert_eq!(record.outcome(), "deny");
        assert_eq!(record.detail(), Some("scope-missing"));
        assert_eq!(record.workload_id(), Some("wl_provisioner"));
    }

    #[tokio::test]
    async fn refuses_unregistered_or_cross_tenant_scim_principals() {
        let fixture = fixture();
        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some(&fixture.unknown_principal_token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        // The PEP hot path audited the fail-closed unknown-principal deny.
        assert_eq!(fixture.audit.len(), 1);
        assert_eq!(fixture.audit.records()[0].outcome(), "deny");
        assert_eq!(
            fixture.audit.records()[0].detail(),
            Some("principal-unknown")
        );

        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_other/Users",
            Some(&fixture.token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        // One additional deny record for the tenant-bind refusal.
        assert_eq!(fixture.audit.len(), 2);
        let record = &fixture.audit.records()[1];
        assert_eq!(record.outcome(), "deny");
        assert_eq!(record.detail(), Some("tenant-mismatch"));
        assert_eq!(record.workload_id(), Some("wl_provisioner"));
        assert_eq!(record.resource_id(), Some("ten_other"));
    }

    /// AC-W-13 on the SCIM surface: a SCIM ALLOW emits exactly one audit
    /// record carrying the right principal/action/resource/outcome — and no
    /// token material.
    #[tokio::test]
    async fn scim_allow_emits_exactly_one_audit_record_with_target() {
        let fixture = fixture();
        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some(&fixture.token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(fixture.audit.len(), 1);
        let record = &fixture.audit.records()[0];
        assert_eq!(record.event(), AuditEvent::Authorize);
        assert_eq!(record.workload_id(), Some("wl_provisioner"));
        assert_eq!(record.outcome(), "allow");
        assert_eq!(record.detail(), Some("permit-scim-manage"));
        assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
        assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
        assert_eq!(record.resource_id(), Some("ten_acme"));
        // The bearer credential must never reach the audit chain: no segment
        // of the JWT may appear anywhere in the sealed record.
        let rendered = format!("{record:?}");
        assert!(!rendered.contains(&fixture.token));
        for segment in fixture.token.split('.') {
            assert!(
                !rendered.contains(segment),
                "audit record leaked token material"
            );
        }
    }

    /// AC-W-13 on the SCIM surface: a SCIM policy DENY (registered, scoped,
    /// tenant-bound principal that Cedar default-denies) emits exactly one
    /// audit record carrying the right principal/action/resource/outcome.
    #[tokio::test]
    async fn scim_cedar_deny_emits_exactly_one_audit_record_with_target() {
        let fixture = fixture();
        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_other/Users",
            Some(&fixture.cedar_denied_token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(fixture.audit.len(), 1);
        let record = &fixture.audit.records()[0];
        assert_eq!(record.event(), AuditEvent::Authorize);
        assert_eq!(record.workload_id(), Some("wl_other_provisioner"));
        assert_eq!(record.outcome(), "deny");
        assert_eq!(record.detail(), Some("default-deny"));
        assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
        assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
        assert_eq!(record.resource_id(), Some("ten_other"));
        let rendered = format!("{record:?}");
        for segment in fixture.cedar_denied_token.split('.') {
            assert!(
                !rendered.contains(segment),
                "audit record leaked token material"
            );
        }
    }

    #[tokio::test]
    async fn user_lifecycle_and_tenant_isolation() {
        let fixture = fixture();
        let token = fixture.token.as_str();

        // Create -> 201 with a kernel-assigned id.
        let (status, created) = send(
            &fixture.router,
            "POST",
            "/scim/v2/ten_acme/Users",
            Some(token),
            Some(new_user_payload("amara@acme.example")),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let id = created["id"].as_str().expect("assigned id").to_owned();

        // Get + list in the owning tenant.
        let (status, fetched) = send(
            &fixture.router,
            "GET",
            &format!("/scim/v2/ten_acme/Users/{id}"),
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(fetched["userName"], "amara@acme.example");

        let (status, listing) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(listing["totalResults"].as_u64(), Some(1));

        // Cross-tenant read fails at the SCIM route guard before the store is
        // consulted: the token tenant must equal the path tenant.
        let (status, _) = send(
            &fixture.router,
            "GET",
            &format!("/scim/v2/ten_other/Users/{id}"),
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Delete -> 204; subsequent get -> 404.
        let (status, _) = send(
            &fixture.router,
            "DELETE",
            &format!("/scim/v2/ten_acme/Users/{id}"),
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = send(
            &fixture.router,
            "GET",
            &format!("/scim/v2/ten_acme/Users/{id}"),
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Exactly one audit record per credential-bearing guard run across the
        // whole mixed flow: 5 allows (the post-delete 404 still authorized at
        // the guard) + 1 tenant-mismatch deny.
        assert_eq!(fixture.audit.len(), 6);
        let records = fixture.audit.records();
        let outcomes: Vec<&str> = records.iter().map(AuditRecord::outcome).collect();
        assert_eq!(
            outcomes,
            ["allow", "allow", "allow", "deny", "allow", "allow"]
        );
    }

    #[tokio::test]
    async fn account_registration_wrapper_reuses_scim_create_and_guard() {
        let fixture = fixture();
        let payload = new_user_payload("register@acme.example");

        let (status, _) = send(
            &fixture.router,
            "POST",
            "/identity/v1/ten_acme/account-registrations",
            None,
            Some(payload.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let (status, _) = send(
            &fixture.router,
            "POST",
            "/identity/v1/ten_acme/account-registrations",
            Some(&fixture.wrong_scope_token),
            Some(payload.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        let (status, created) = send(
            &fixture.router,
            "POST",
            "/identity/v1/ten_acme/account-registrations",
            Some(&fixture.token),
            Some(payload),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(created["userName"], "register@acme.example");
        let id = created["id"].as_str().expect("assigned id").to_owned();

        let (status, fetched) = send(
            &fixture.router,
            "GET",
            &format!("/scim/v2/ten_acme/Users/{id}"),
            Some(&fixture.token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(fetched["id"], id);

        let (status, _) = send(
            &fixture.router,
            "POST",
            "/identity/v1/ten_other/account-registrations",
            Some(&fixture.token),
            Some(new_user_payload("cross-tenant@acme.example")),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn group_create_and_list() {
        let fixture = fixture();
        let token = fixture.token.as_str();
        let (status, created) = send(
            &fixture.router,
            "POST",
            "/scim/v2/ten_acme/Groups",
            Some(token),
            Some(serde_json::json!({"displayName": "platform-admins"})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(created["id"].as_str().is_some());

        let (status, listing) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Groups",
            Some(token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(listing["totalResults"].as_u64(), Some(1));
    }
}
