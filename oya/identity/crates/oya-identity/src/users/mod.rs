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

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};

use oya_shared_scim_server_kernel::{
    CounterIdGen, InMemoryGroupStore, InMemoryUserStore, ListQuery, NewGroup, NewUser, PatchOp,
    ReferenceScimServer, ScimError, ScimId, ScimServer, TenantId,
};

use oya_identity_workload_app::AuthorizeOutcome;
use oya_identity_workload_domain::{Action, ClaimValue, Resource};
use oya_identity_workload_oidc_adapter::{Jwks, ValidationConfig, validate_workload_token};

use crate::AppState;

/// The SCIM scope a caller's workload token must carry.
pub const SCIM_MANAGE_SCOPE: &str = "scim.manage";

/// Tenant-scoped SCIM base path (RFC 7644 §3.2 endpoints hang off this).
pub const SCIM_BASE: &str = "/scim/v2";

/// The composed SCIM surface state: the kernel reference server over the
/// in-memory store ports + the offline token-validation material.
pub struct ScimSurfaceState {
    server: ReferenceScimServer<InMemoryUserStore, InMemoryGroupStore, CounterIdGen>,
    jwks: Jwks,
    validation: ValidationConfig,
    now_provider: fn() -> i64,
    authz: Arc<AppState>,
}

impl ScimSurfaceState {
    /// Assemble the surface. `base_url` is the externally visible SCIM base
    /// (issuer + `/scim/v2`), used for `meta.location` URLs.
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        jwks: Jwks,
        validation: ValidationConfig,
        now_provider: fn() -> i64,
        authz: Arc<AppState>,
    ) -> Self {
        Self {
            server: ReferenceScimServer::new(
                InMemoryUserStore::default(),
                InMemoryGroupStore::default(),
                CounterIdGen::new(),
                base_url,
            ),
            jwks,
            validation,
            now_provider,
            authz,
        }
    }
}

/// Shared handle for the SCIM routes.
pub type SharedScimState = Arc<ScimSurfaceState>;

// =====================================================================
// Fail-closed Bearer guard
// =====================================================================

/// Validate the Bearer credential and require [`SCIM_MANAGE_SCOPE`].
/// `Err` is the ready-to-return refusal response (401/403) — never a pass.
fn authorize_scim(
    state: &ScimSurfaceState,
    headers: &HeaderMap,
    tenant: &str,
) -> Result<(), Response> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| scim_refusal(StatusCode::UNAUTHORIZED, "missing bearer credential"))?;
    let principal = validate_workload_token(
        token,
        &state.jwks,
        &state.validation,
        (state.now_provider)(),
    )
    .map_err(|_| scim_refusal(StatusCode::UNAUTHORIZED, "invalid bearer credential"))?;
    if !principal.has_scope(SCIM_MANAGE_SCOPE) {
        return Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential lacks the scim.manage scope",
        ));
    }
    if principal.tenant_id().as_str() != tenant {
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
        Action::new("identity.scim.Manage"),
        Resource::new("ScimTenant", tenant.to_owned()),
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

fn scim_refusal(status: StatusCode, detail: &str) -> Response {
    (status, Json(ScimError::new(status.as_u16(), None, detail))).into_response()
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
            return refusal;
        }
    };
}

async fn list_users(
    State(state): State<SharedScimState>,
    Path(tenant): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .list_users(&TenantId(tenant), &page_query(&uri))
    {
        Ok(listing) => Json(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn create_user(
    State(state): State<SharedScimState>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .create_user(&TenantId(tenant), input, (state.now_provider)())
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn get_user(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state.server.get_user(&TenantId(tenant), &ScimId(id)) {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn replace_user(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response {
    guard!(state, headers, tenant);
    match state.server.replace_user(
        &TenantId(tenant),
        &ScimId(id),
        input,
        (state.now_provider)(),
    ) {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn patch_user(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(op): Json<PatchOp>,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .patch_user(&TenantId(tenant), &ScimId(id), &op, (state.now_provider)())
    {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn delete_user(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state.server.delete_user(&TenantId(tenant), &ScimId(id)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn list_groups(
    State(state): State<SharedScimState>,
    Path(tenant): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .list_groups(&TenantId(tenant), &page_query(&uri))
    {
        Ok(listing) => Json::<_>(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn create_group(
    State(state): State<SharedScimState>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .create_group(&TenantId(tenant), input, (state.now_provider)())
    {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn get_group(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state.server.get_group(&TenantId(tenant), &ScimId(id)) {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn patch_group(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(op): Json<PatchOp>,
) -> Response {
    guard!(state, headers, tenant);
    match state
        .server
        .patch_group(&TenantId(tenant), &ScimId(id), &op, (state.now_provider)())
    {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

async fn delete_group(
    State(state): State<SharedScimState>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    guard!(state, headers, tenant);
    match state.server.delete_group(&TenantId(tenant), &ScimId(id)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}

/// Build the SCIM router (mounted at the service root; paths carry
/// [`SCIM_BASE`]).
pub fn build_scim_router(state: SharedScimState) -> Router {
    Router::new()
        .route(&format!("{SCIM_BASE}/{{tenant}}/Users"), get(list_users))
        .route(&format!("{SCIM_BASE}/{{tenant}}/Users"), post(create_user))
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Users/{{id}}"),
            get(get_user),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Users/{{id}}"),
            put(replace_user),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Users/{{id}}"),
            patch(patch_user),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Users/{{id}}"),
            delete(delete_user),
        )
        .route(&format!("{SCIM_BASE}/{{tenant}}/Groups"), get(list_groups))
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Groups"),
            post(create_group),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Groups/{{id}}"),
            get(get_group),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Groups/{{id}}"),
            patch(patch_group),
        )
        .route(
            &format!("{SCIM_BASE}/{{tenant}}/Groups/{{id}}"),
            delete(delete_group),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use tower::ServiceExt as _;

    use oya_identity_workload_app::{
        InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository,
        WorkloadPrincipalRepository,
    };
    use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
    use oya_identity_workload_domain::{WorkloadPrincipal, WorkloadState};
    use oya_identity_workload_oidc_adapter::Jwk;
    use oya_identity_workload_rest::WorkloadAuthzState;

    use crate::observability::TracingAuditSink;

    const ISSUER: &str = "https://idp.oyatie.com";
    const AUDIENCE: &str = "oya-identity-scim";
    const KID: &str = "kid-scim-1";
    const NOW: i64 = 1_700_000_000;

    struct Fixture {
        router: Router,
        token: String,
        wrong_scope_token: String,
        unknown_principal_token: String,
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
        let mut provisioner =
            WorkloadPrincipal::provision("ten_acme", "wl_provisioner", "cap.identity.scim")
                .expect("principal");
        provisioner
            .transition_to(WorkloadState::Active)
            .expect("activate");
        repository.save(&provisioner).expect("seed principal");
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("permit-scim-manage")
            permit(
              principal,
              action == Action::"identity.scim.Manage",
              resource is ScimTenant
            );
            "#,
        )
        .expect("policy");
        let authz = Arc::new(WorkloadAuthzState::with_clock(
            repository,
            InMemoryRevocationDenylist::new(),
            authorizer,
            jwks.clone(),
            ValidationConfig::new(ISSUER, AUDIENCE),
            TracingAuditSink::new(),
            || NOW,
        ));
        let state = Arc::new(ScimSurfaceState::new(
            format!("{ISSUER}{SCIM_BASE}"),
            jwks,
            ValidationConfig::new(ISSUER, AUDIENCE),
            || NOW,
            authz,
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

    fn new_user_payload(user_name: &str) -> serde_json::Value {
        serde_json::json!({
            "user_name": user_name,
            "external_id": null,
            "name": null,
            "display_name": "Provisioned User",
            "active": true,
            "emails": [],
            "enterprise": null,
            "oyatie": null,
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

        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_acme/Users",
            Some("not-a-jwt"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
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

        let (status, _) = send(
            &fixture.router,
            "GET",
            "/scim/v2/ten_other/Users",
            Some(&fixture.token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
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
            Some(serde_json::json!({"display_name": "platform-admins", "members": []})),
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
