//! Tenant-scoped SCIM provisioning over the native server kernel.
//! Every route verifies its workload and runs the IAM PDP before store access.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};

use iam_scim_server_kernel::{
    GroupStore, IdGen, ListQuery, NewGroup, NewUser, PatchOp, ReferenceScimServer, ScimError,
    ScimId, ScimServer, TenantId, UserStore,
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
pub const NATIVE_USER_ADMIN_SCOPE: &str = "identity.user.administer";

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

/// A process restart must not recycle SCIM identifiers already stored in
/// Postgres. Random UUIDs keep opaque IDs unique across replicas.
struct ProductionIdGen;

impl IdGen for ProductionIdGen {
    fn next_id(&self) -> ScimId {
        ScimId(format!("urn:oya:scim:{}", uuid::Uuid::new_v4()))
    }
}

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
    server: ReferenceScimServer<U, G, ProductionIdGen>,
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
            server: ReferenceScimServer::new(users, groups, ProductionIdGen, base_url),
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
const NATIVE_USER_ADMIN_ACTION: &str = "identity.user.Administer";

/// The Cedar resource type the guard scopes decisions to.
const SCIM_TENANT_RESOURCE_TYPE: &str = "ScimTenant";

macro_rules! guard {
    ($state:expr, $headers:expr, $tenant:expr) => {
        if let Err(refusal) = authorize_scim(&$state, &$headers, &$tenant) {
            return *refusal;
        }
    };
}

macro_rules! guard_native_admin {
    ($state:expr, $headers:expr, $tenant:expr) => {
        if let Err(refusal) = authorize_native_user_admin(&$state, &$headers, &$tenant) {
            return *refusal;
        }
    };
}

mod authority;
mod groups;
mod guards;
mod user_reads;
mod user_writes;
use authority::*;
use groups::*;
use guards::*;
pub(crate) use guards::{authorize_native_user_admin, authorize_scim};
use user_reads::*;
use user_writes::*;

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
mod tests;
