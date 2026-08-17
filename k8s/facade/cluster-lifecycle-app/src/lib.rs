#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use axum::Router;
use axum::extract::{FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use k8s_cluster_lifecycle_api::{
    ClusterLifecycle, ClusterResourceRequest, LifecycleError, LifecycleProvisioningResult,
};
use k8s_cluster_lifecycle_kernel::{DesiredTier, LifecycleRequest};
use k8s_control_plane_host_adapter_inmemory::InMemoryControlPlaneHost;
use k8s_tenant_quota_adapter_inmemory::InMemoryQuotaStore;
use k8s_tenant_quota_kernel::TenantQuota;
use serde::{Deserialize, Serialize};

pub mod authz;

pub use authz::{
    AuthzProviderConfigError, CallerCredential, CedarClusterAuthorizer, ClusterAction,
    ClusterAuthorizationError, ClusterAuthorizer, ClusterAuthzProvider,
    ConfiguredBearerPrincipalVerifier, PrincipalVerificationError, PrincipalVerifier,
    VerifiedPrincipal,
};

/// Shared application state injected into every axum handler.
///
/// The [`ClusterAuthzProvider`] is REQUIRED and non-optional: there is no
/// authz-less constructor, so the router can NEVER be built without a configured
/// principal-verification + PDP authorization seam (fail-closed; GitHub #979).
#[derive(Clone)]
pub struct AppState {
    quota: InMemoryQuotaStore,
    provisioning: std::sync::Arc<InMemoryControlPlaneHost>,
    authz: ClusterAuthzProvider,
}

#[derive(Debug)]
pub enum BootError {
    ProductionAdapterUnavailable,
    InvalidStaticQuota(String),
    /// The authorization provider could not be composed (empty bearer secret /
    /// bound identity, or a Cedar policy compile failure). Fail-closed: the
    /// service REFUSES to serve — there is no default-allow fallback.
    Authz(String),
    Bind(String),
    Serve(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProductionAdapterUnavailable => f.write_str(
                "production cluster-lifecycle adapters are not wired; set OYA_CLUSTER_LIFECYCLE_ENABLE_INMEMORY=true only for dev/test bring-up",
            ),
            Self::InvalidStaticQuota(error) => write!(f, "invalid dev static quota: {error}"),
            Self::Authz(error) => write!(f, "authz provider boot refused: {error}"),
            Self::Bind(error) => write!(f, "bind error: {error}"),
            Self::Serve(error) => write!(f, "serve error: {error}"),
        }
    }
}

impl std::error::Error for BootError {}

impl From<AuthzProviderConfigError> for BootError {
    fn from(error: AuthzProviderConfigError) -> Self {
        Self::Authz(error.to_string())
    }
}

pub const ENABLE_INMEMORY_ENV: &str = "OYA_CLUSTER_LIFECYCLE_ENABLE_INMEMORY";

/// Env var carrying the break-glass platform-operator bearer secret. Fail-closed:
/// boot is REFUSED if this is empty (no provable credential root, no service).
pub const ENV_BEARER_TOKEN: &str = "K8S_CLUSTER_LIFECYCLE_BEARER_TOKEN";
/// The break-glass operator identity bound to the configured bearer. The
/// principal/tenant ids follow the iam workload-identity shapes the PDP validates.
const BREAK_GLASS_PRINCIPAL_ID: &str = "wl_k8s_cluster_lifecycle_operator";
const BREAK_GLASS_TENANT_ID: &str = "ten_platform";
/// Platform-operator scope: may create clusters for any tenant.
const BREAK_GLASS_SCOPE: &str = "cluster:platform:write";

impl AppState {
    #[must_use]
    pub fn new(
        quota: InMemoryQuotaStore,
        provisioning: InMemoryControlPlaneHost,
        authz: ClusterAuthzProvider,
    ) -> Self {
        Self {
            quota,
            provisioning: std::sync::Arc::new(provisioning),
            authz,
        }
    }
}

/// Build the fail-closed authz provider from the environment, REFUSING to boot
/// on an empty bearer secret.
///
/// # Errors
/// [`BootError::Authz`] when the bearer secret is empty or the Cedar policy set
/// fails to compile.
pub fn authz_from_env() -> Result<ClusterAuthzProvider, BootError> {
    let bearer = std::env::var(ENV_BEARER_TOKEN).unwrap_or_default();
    let authz = ClusterAuthzProvider::from_bearer_secret(
        bearer,
        BREAK_GLASS_PRINCIPAL_ID,
        BREAK_GLASS_TENANT_ID,
        vec![BREAK_GLASS_SCOPE.to_string()],
    )?;
    Ok(authz)
}

// ============================================================
// VerifiedCaller — authn-BEFORE-body extractor
// ============================================================

/// A request extractor that authenticates the caller from the verified bearer
/// credential over the request PARTS — i.e. BEFORE the body is read. Placed
/// ahead of `Json(body)` in a handler signature it GUARANTEES authn precedes
/// body deserialization (401 on no/bad bearer without the body ever parsed).
pub struct VerifiedCaller(pub VerifiedPrincipal);

impl FromRequestParts<AppState> for VerifiedCaller {
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let credential = CallerCredential {
            authorization: parts
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned()),
        };
        match state.authz.verify_principal(&credential) {
            Ok(principal) => Ok(Self(principal)),
            Err(_) => Err(error_response(
                StatusCode::UNAUTHORIZED,
                "missing_caller_principal",
                "a verified caller credential is required".to_string(),
            )
            .into_response()),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClusterCreateBody {
    pub tenant_id: String,
    pub cluster_name: String,
    #[serde(default)]
    pub desired_tier: Option<String>,
    #[serde(default)]
    pub nodes: Option<u32>,
    #[serde(default)]
    pub vcpu: Option<u32>,
    #[serde(default)]
    pub ram_gib: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClusterCreateResponse {
    pub tenant_id: String,
    pub cluster_name: String,
    pub desired_tier: String,
    pub control_plane_tier: String,
    pub control_plane_handle: String,
}

impl From<LifecycleProvisioningResult> for ClusterCreateResponse {
    fn from(value: LifecycleProvisioningResult) -> Self {
        Self {
            tenant_id: value.tenant_id,
            cluster_name: value.cluster_name,
            desired_tier: value.desired_tier.as_str().to_string(),
            control_plane_tier: value.control_plane_tier.as_str().to_string(),
            control_plane_handle: value.control_plane_ref.handle,
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/clusters", post(create_cluster_handler))
        .route("/healthz", get(healthz_handler))
        .with_state(state)
}

async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn create_cluster_handler(
    State(state): State<AppState>,
    VerifiedCaller(principal): VerifiedCaller,
    axum::Json(body): axum::Json<ClusterCreateBody>,
) -> axum::response::Response {
    // The verified principal's tenant is authoritative; `body.tenant_id` is a
    // PDP-checked RESOURCE SELECTOR. The Cedar decision denies a cross-tenant
    // create unless the principal holds the platform scope (replaces the old
    // forgeable `x-oya-tenant-id == body.tenant_id` header-trust). Authn ran in
    // the `VerifiedCaller` extractor before this body was parsed; the PDP
    // decision runs before any provisioning — fail-closed.
    if state
        .authz
        .ensure_authorized(&principal, ClusterAction::Create, &body.tenant_id)
        .is_err()
    {
        return error_response(
            StatusCode::FORBIDDEN,
            "tenant_create_forbidden",
            "caller is not authorized to create a cluster for this tenant".to_string(),
        )
        .into_response();
    }
    let tier = match body.desired_tier.as_deref() {
        None => DesiredTier::Hosted,
        Some(slug) => match DesiredTier::parse(slug) {
            Some(tier) => tier,
            None => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_tier",
                    format!("unknown desired_tier {slug:?}"),
                )
                .into_response();
            }
        },
    };
    let defaults = ClusterResourceRequest::default_small();
    let resources = ClusterResourceRequest::new(
        body.nodes.unwrap_or(defaults.nodes),
        body.vcpu.unwrap_or(defaults.vcpu),
        body.ram_gib.unwrap_or(defaults.ram_gib),
    );
    let request = match LifecycleRequest::new(body.tenant_id, body.cluster_name, tier, resources) {
        Ok(request) => request,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                error.to_string(),
            )
            .into_response();
        }
    };
    let lifecycle = ClusterLifecycle::new(&state.quota, state.provisioning.as_ref());
    match lifecycle.provision_cluster(&request).await {
        Ok(result) => (
            StatusCode::CREATED,
            axum::Json(ClusterCreateResponse::from(result)),
        )
            .into_response(),
        Err(error) => lifecycle_error_response(&error).into_response(),
    }
}

fn lifecycle_error_response(error: &LifecycleError) -> (StatusCode, axum::Json<serde_json::Value>) {
    match error {
        LifecycleError::InvalidRequest(_) => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            error.to_string(),
        ),
        LifecycleError::QuotaDenied(_) => {
            error_response(StatusCode::FORBIDDEN, "quota_denied", error.to_string())
        }
        LifecycleError::QuotaUnavailable(_) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "quota_unavailable",
            error.to_string(),
        ),
        LifecycleError::ProvisioningFailed(_) => error_response(
            StatusCode::BAD_GATEWAY,
            "provisioning_failed",
            error.to_string(),
        ),
    }
}

fn error_response(
    code: StatusCode,
    kind: &'static str,
    detail: String,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    (
        code,
        axum::Json(serde_json::json!({ "error": { "kind": kind, "detail": detail } })),
    )
}

pub fn build_state_in_memory(authz: ClusterAuthzProvider) -> Result<AppState, BootError> {
    let quota = TenantQuota::new("ten_zero", 5, 10, 32, 128)
        .map_err(|error| BootError::InvalidStaticQuota(error.to_string()))?;
    Ok(AppState::new(
        InMemoryQuotaStore::new().with_quota(quota),
        InMemoryControlPlaneHost::new(),
        authz,
    ))
}

pub fn build_state_from_env_value(value: Option<&str>) -> Result<AppState, BootError> {
    match value {
        Some("true") => build_state_in_memory(authz_from_env()?),
        _ => Err(BootError::ProductionAdapterUnavailable),
    }
}

pub fn build_state_from_env() -> Result<AppState, BootError> {
    build_state_from_env_value(std::env::var(ENABLE_INMEMORY_ENV).ok().as_deref())
}

pub async fn serve(listen_addr: &str, router: Router) -> Result<(), BootError> {
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|error| BootError::Bind(format!("{listen_addr}: {error}")))?;
    tracing::info!(
        addr = listen_addr,
        "managed-k8s cluster-lifecycle listening"
    );
    axum::serve(listener, router)
        .await
        .map_err(|error| BootError::Serve(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tower::ServiceExt;

    const TEST_TOKEN: &str = "test-break-glass-secret";

    /// Break-glass platform-operator authz: any tenant create allowed (matches
    /// the production composition).
    fn platform_authz() -> ClusterAuthzProvider {
        ClusterAuthzProvider::from_bearer_secret(
            TEST_TOKEN,
            "wl_platform_op",
            "ten_platform",
            vec!["cluster:platform:write".to_owned()],
        )
        .unwrap()
    }

    /// Tenant-admin authz bound to `tenant` (own-tenant create allowed,
    /// cross-tenant denied by the PDP).
    fn tenant_admin_authz(tenant: &str) -> ClusterAuthzProvider {
        ClusterAuthzProvider::from_bearer_secret(
            TEST_TOKEN,
            "wl_tenant_admin",
            tenant,
            vec!["cluster:write".to_owned()],
        )
        .unwrap()
    }

    /// A faulting authorizer: every decision is a fail-closed PDP refusal (403).
    struct FaultAuthorizer;
    impl ClusterAuthorizer for FaultAuthorizer {
        fn ensure_authorized(
            &self,
            _p: &VerifiedPrincipal,
            _a: ClusterAction,
            _t: &str,
        ) -> Result<(), ClusterAuthorizationError> {
            Err(ClusterAuthorizationError::Refused)
        }
    }

    fn fault_authz() -> ClusterAuthzProvider {
        let verifier = Arc::new(
            ConfiguredBearerPrincipalVerifier::new(TEST_TOKEN, "wl_op", "ten_platform", vec![])
                .unwrap(),
        );
        ClusterAuthzProvider::new(verifier, Arc::new(FaultAuthorizer))
    }

    fn bearer() -> String {
        format!("Bearer {TEST_TOKEN}")
    }

    async fn body_str(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[tokio::test]
    async fn create_cluster_default_hosted_flow_returns_201() {
        let app = build_router(build_state_in_memory(platform_authz()).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header("authorization", bearer())
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let text = body_str(resp.into_body()).await;
        assert!(text.contains("hosted_kamaji"));
        assert!(text.contains("dogfood-a"));
    }

    #[tokio::test]
    async fn create_cluster_without_bearer_returns_401() {
        let app = build_router(build_state_in_memory(platform_authz()).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    // no authorization header
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_cluster_cross_tenant_returns_403() {
        // A tenant-admin bound to ten_zero requesting a cluster for ten_other is
        // cross-tenant: the verified tenant is authoritative and the PDP denies
        // the forged `body.tenant_id` selector => 403. This replaces the old
        // forgeable header==body trust.
        let app = build_router(build_state_in_memory(tenant_admin_authz("ten_zero")).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_other","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header("authorization", bearer())
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn create_cluster_same_tenant_admin_returns_201() {
        // A tenant-admin bound to ten_zero creating a cluster for ten_zero is
        // authorized (own tenant).
        let app = build_router(build_state_in_memory(tenant_admin_authz("ten_zero")).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header("authorization", bearer())
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn create_cluster_pdp_fault_returns_403_not_5xx() {
        let app = build_router(build_state_in_memory(fault_authz()).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header("authorization", bearer())
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn create_cluster_unknown_tenant_fails_closed() {
        // Authorized (platform operator) but the tenant has no quota record: the
        // downstream lifecycle fails closed with quota_unavailable (503).
        let app = build_router(build_state_in_memory(platform_authz()).unwrap());
        let body = serde_json::json!({"tenant_id":"ten_missing","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header("authorization", bearer())
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let text = body_str(resp.into_body()).await;
        assert!(text.contains("quota_unavailable"));
    }

    #[test]
    fn boot_refuses_empty_bearer_secret() {
        assert!(
            ClusterAuthzProvider::from_bearer_secret("", "wl_op", "ten_platform", vec![]).is_err(),
            "an empty bearer secret must refuse provider construction"
        );
    }
}

#[cfg(test)]
mod boot_tests {
    use super::*;

    #[test]
    fn production_boot_without_explicit_dev_flag_fails_closed() {
        assert!(matches!(
            build_state_from_env_value(None),
            Err(BootError::ProductionAdapterUnavailable)
        ));
        assert!(matches!(
            build_state_from_env_value(Some("false")),
            Err(BootError::ProductionAdapterUnavailable)
        ));
    }
}
