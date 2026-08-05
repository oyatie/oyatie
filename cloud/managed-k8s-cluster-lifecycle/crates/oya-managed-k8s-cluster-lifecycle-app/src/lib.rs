#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use oya_managed_k8s_cluster_lifecycle_api::{
    ClusterLifecycle, ClusterResourceRequest, LifecycleError, LifecycleProvisioningResult,
};
use oya_managed_k8s_cluster_lifecycle_kernel::{DesiredTier, LifecycleRequest};
use oya_managed_k8s_control_plane_host_adapter_inmemory::InMemoryControlPlaneHost;
use oya_managed_k8s_tenant_quota_adapter_inmemory::InMemoryQuotaStore;
use oya_managed_k8s_tenant_quota_kernel::TenantQuota;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct AppState {
    quota: InMemoryQuotaStore,
    provisioning: std::sync::Arc<InMemoryControlPlaneHost>,
}

#[derive(Debug)]
pub enum BootError {
    ProductionAdapterUnavailable,
    InvalidStaticQuota(String),
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
            Self::Bind(error) => write!(f, "bind error: {error}"),
            Self::Serve(error) => write!(f, "serve error: {error}"),
        }
    }
}

impl std::error::Error for BootError {}

pub const ENABLE_INMEMORY_ENV: &str = "OYA_CLUSTER_LIFECYCLE_ENABLE_INMEMORY";
pub const TENANT_HEADER: &str = "x-oya-tenant-id";

impl AppState {
    #[must_use]
    pub fn new(quota: InMemoryQuotaStore, provisioning: InMemoryControlPlaneHost) -> Self {
        Self {
            quota,
            provisioning: std::sync::Arc::new(provisioning),
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
    headers: HeaderMap,
    axum::Json(body): axum::Json<ClusterCreateBody>,
) -> axum::response::Response {
    let Some(header_tenant) = headers
        .get(TENANT_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "missing_tenant_principal",
            format!("{TENANT_HEADER} header is required"),
        )
        .into_response();
    };
    if header_tenant != body.tenant_id {
        return error_response(
            StatusCode::FORBIDDEN,
            "tenant_principal_mismatch",
            "tenant principal does not match request tenant_id".to_string(),
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
        LifecycleError::InvalidOperation(_) => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_operation",
            error.to_string(),
        ),
        LifecycleError::LedgerUnavailable(_) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "operation_ledger_unavailable",
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

pub fn build_state_in_memory() -> Result<AppState, BootError> {
    let quota = TenantQuota::new("ten_zero", 5, 10, 32, 128)
        .map_err(|error| BootError::InvalidStaticQuota(error.to_string()))?;
    Ok(AppState::new(
        InMemoryQuotaStore::new().with_quota(quota),
        InMemoryControlPlaneHost::new(),
    ))
}

pub fn build_state_from_env_value(value: Option<&str>) -> Result<AppState, BootError> {
    match value {
        Some("true") => build_state_in_memory(),
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
    use tower::ServiceExt;

    async fn body_str(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[tokio::test]
    async fn create_cluster_default_hosted_flow_returns_201() {
        let app = build_router(build_state_in_memory().unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header(TENANT_HEADER, "ten_zero")
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
    async fn create_cluster_missing_tenant_header_returns_401() {
        let app = build_router(build_state_in_memory().unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_cluster_tenant_header_mismatch_returns_403() {
        let app = build_router(build_state_in_memory().unwrap());
        let body = serde_json::json!({"tenant_id":"ten_zero","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header(TENANT_HEADER, "ten_other")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn create_cluster_unknown_tenant_fails_closed() {
        let app = build_router(build_state_in_memory().unwrap());
        let body = serde_json::json!({"tenant_id":"ten_missing","cluster_name":"dogfood-a"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/clusters")
                    .header("content-type", "application/json")
                    .header(TENANT_HEADER, "ten_missing")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let text = body_str(resp.into_body()).await;
        assert!(text.contains("quota_unavailable"));
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
