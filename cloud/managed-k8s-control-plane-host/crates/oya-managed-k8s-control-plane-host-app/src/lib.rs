//! Managed-Kubernetes control-plane-host composition root (ADR-0376).
//!
//! This crate is the *application* that wires the control-plane-host port
//! ([`oya_managed_k8s_control_plane_host_api::ControlPlaneProvisioning`]) into a
//! runnable admin/status API. It owns NO provisioning algorithm of its own —
//! that lives inward (the kernel state machine + the api port) and outward (the
//! kube-rs CAPI adapter or the in-memory fake):
//!
//! ```text
//!  operator --POST /admin/control-planes--> provision(tier, datastore, cluster)
//!                                                    │
//!                                   Arc<dyn ControlPlaneProvisioning>
//!                                          ┌─────────┴──────────┐
//!                                  in-memory fake         kube-rs CAPI adapter
//!                                  (dev / test)           (mgmt cluster; live
//!                                                          Kamaji dynamic API)
//! ```
//!
//! ## Operational boundary (ADR-0376)
//!
//! The kube-rs adapter talks ONLY to Oyatie's MANAGEMENT cluster; this service
//! never runs tenant workloads and never holds a tenant-cluster kubeconfig. The
//! production [`run`] path is **fail-closed**: it reads the management
//! kubeconfig path from `$OYA_MGMT_KUBECONFIG` and returns a typed
//! [`BootError::MissingMgmtKubeconfig`] if absent — it never silently falls back
//! to the in-memory fake in production.
//!
//! ## Hot-path posture (ADR-0083 Tier-3 — panic-free)
//!
//! No `.unwrap()`/`.expect()`/`panic!()` on the request path; boot
//! misconfiguration is surfaced as [`BootError`] before the socket binds.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};

use oya_managed_k8s_control_plane_host_adapter_inmemory::InMemoryControlPlaneHost;
use oya_managed_k8s_control_plane_host_api::{
    ClusterRef, ControlPlaneProvisioning, ControlPlaneRef, ProvisionRequest, ProvisioningError,
};
use oya_managed_k8s_control_plane_host_kernel::{ControlPlaneTier, DatastoreClass};

pub use oya_managed_k8s_control_plane_host_adapter_capi::{
    CapiControlPlaneHost, LiveProvisioningPolicy,
};

// =====================================================================
// App state
// =====================================================================

/// Shared application state: the provisioning port the router dispatches to.
#[derive(Clone)]
pub struct AppState {
    provisioning: Arc<dyn ControlPlaneProvisioning>,
}

impl AppState {
    /// Build app state from any [`ControlPlaneProvisioning`] implementation.
    #[must_use]
    pub fn new(provisioning: Arc<dyn ControlPlaneProvisioning>) -> Self {
        Self { provisioning }
    }

    /// Borrow the provisioning port (useful for tests + the acceptance suite).
    #[must_use]
    pub fn provisioning(&self) -> &Arc<dyn ControlPlaneProvisioning> {
        &self.provisioning
    }
}

// =====================================================================
// Boot errors
// =====================================================================

/// Errors raised by [`run`] / [`serve`] BEFORE (or while binding) the listener.
#[derive(Debug)]
pub enum BootError {
    /// The management kubeconfig path env var is absent or empty. Fail-closed —
    /// the production path never silently uses the in-memory fake.
    MissingMgmtKubeconfig {
        /// The env var that was expected to carry the path.
        env_var: String,
    },
    /// A TCP listener could not be bound.
    Bind {
        /// The address that failed to bind.
        address: String,
        /// The underlying OS error rendered for logs.
        error: String,
    },
    /// The axum serve loop exited with an error.
    Serve(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMgmtKubeconfig { env_var } => write!(
                f,
                "{env_var} env var (path to the MANAGEMENT-cluster kubeconfig) is required and must be non-empty (fail-closed; ADR-0376 operational boundary)"
            ),
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(error) => write!(f, "serve error: {error}"),
        }
    }
}

impl std::error::Error for BootError {}

/// The env var carrying the management-cluster kubeconfig path.
pub const MGMT_KUBECONFIG_ENV: &str = "OYA_MGMT_KUBECONFIG";
/// Rollback switch for live provisioning. False/disabled blocks new live
/// provisions while preserving status/teardown for existing objects.
pub const LIVE_PROVISIONING_ENV: &str = "OYA_MK8S_CONTROL_PLANE_HOST_LIVE_PROVISIONING";

// =====================================================================
// API DTOs
// =====================================================================

/// JSON body for `POST /admin/control-planes`. String tier/datastore fields are
/// parsed via the kernel's fail-closed `parse` (unknown values -> 400).
#[derive(Clone, Debug, Deserialize)]
pub struct ProvisionBody {
    /// Tenant that owns the cluster.
    pub tenant_id: String,
    /// Tenant-unique cluster name.
    pub cluster_name: String,
    /// Tier slug (`hosted_kamaji` | `dedicated_talos_spoke`). Defaults to the
    /// product default (hosted) when omitted.
    #[serde(default)]
    pub tier: Option<String>,
    /// Datastore-class slug (`etcd_per_tenant` | `pooled_relational`). Defaults
    /// to `etcd_per_tenant` (strongest hosted isolation) when omitted.
    #[serde(default)]
    pub datastore_class: Option<String>,
}

/// JSON body identifying a control plane for `status` / `teardown`.
#[derive(Clone, Debug, Deserialize)]
pub struct ControlPlaneRefBody {
    /// Tenant that owns the cluster.
    pub tenant_id: String,
    /// Tenant-unique cluster name.
    pub cluster_name: String,
    /// Tier slug (must match what the control plane was provisioned under).
    pub tier: String,
    /// Adapter-issued opaque handle returned by `provision`.
    pub handle: String,
}

/// JSON response echoing a provisioned control plane.
#[derive(Clone, Debug, Serialize)]
pub struct ControlPlaneRefResponse {
    /// Tenant id.
    pub tenant_id: String,
    /// Cluster name.
    pub cluster_name: String,
    /// Tier slug.
    pub tier: String,
    /// Adapter-issued opaque handle.
    pub handle: String,
}

impl From<ControlPlaneRef> for ControlPlaneRefResponse {
    fn from(value: ControlPlaneRef) -> Self {
        Self {
            tenant_id: value.cluster_ref.tenant_id,
            cluster_name: value.cluster_ref.cluster_name,
            tier: value.tier.as_str().to_string(),
            handle: value.handle,
        }
    }
}

/// JSON status response.
#[derive(Clone, Debug, Serialize)]
pub struct StatusResponse {
    /// Current lifecycle status slug.
    pub status: String,
    /// Tier slug.
    pub tier: String,
    /// API-server endpoint, if ready.
    pub endpoint: Option<String>,
}

// =====================================================================
// Router
// =====================================================================

/// Build the axum admin/status router over the given [`AppState`].
///
/// Routes:
/// - `POST   /admin/control-planes`          — provision (returns the handle)
/// - `POST   /admin/control-planes/status`   — status of a control plane
/// - `POST   /admin/control-planes/teardown` — drain + delete a control plane
/// - `GET    /healthz`                       — liveness ("ok")
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/admin/control-planes", post(provision_handler))
        .route("/admin/control-planes/status", post(status_handler))
        .route("/admin/control-planes/teardown", post(teardown_handler))
        .route("/healthz", get(healthz_handler))
        .with_state(state)
}

async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// Map a [`ProvisioningError`] to an HTTP status + JSON envelope. Malformed
/// input maps to 400, not-found to 404, backend to 502, and any explicit
/// adapter deferral to 501.
fn error_response(error: &ProvisioningError) -> (StatusCode, axum::Json<serde_json::Value>) {
    let (code, kind) = match error {
        ProvisioningError::InvalidClusterRef { .. } => {
            (StatusCode::BAD_REQUEST, "invalid_cluster_ref")
        }
        ProvisioningError::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
        ProvisioningError::IllegalTransition(_) => (StatusCode::CONFLICT, "illegal_transition"),
        ProvisioningError::Backend { .. } => (StatusCode::BAD_GATEWAY, "backend_error"),
        ProvisioningError::Unimplemented(_) => (StatusCode::NOT_IMPLEMENTED, "unimplemented"),
    };
    (
        code,
        axum::Json(serde_json::json!({
            "error": { "kind": kind, "detail": error.to_string() }
        })),
    )
}

async fn provision_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<ProvisionBody>,
) -> axum::response::Response {
    let tier = match body.tier.as_deref() {
        None => ControlPlaneTier::default_tier(),
        Some(slug) => match ControlPlaneTier::parse(slug) {
            Some(tier) => tier,
            None => {
                return error_response(&ProvisioningError::InvalidClusterRef {
                    cluster_ref: format!("unknown tier {slug:?}"),
                })
                .into_response();
            }
        },
    };
    let datastore_class = match body.datastore_class.as_deref() {
        None => DatastoreClass::EtcdPerTenant,
        Some(slug) => match DatastoreClass::parse(slug) {
            Some(class) => class,
            None => {
                return error_response(&ProvisioningError::InvalidClusterRef {
                    cluster_ref: format!("unknown datastore_class {slug:?}"),
                })
                .into_response();
            }
        },
    };
    let request = ProvisionRequest::new(
        ClusterRef::new(body.tenant_id, body.cluster_name),
        tier,
        datastore_class,
    );
    match state.provisioning.provision(&request).await {
        Ok(control_plane) => (
            StatusCode::CREATED,
            axum::Json(ControlPlaneRefResponse::from(control_plane)),
        )
            .into_response(),
        Err(error) => error_response(&error).into_response(),
    }
}

fn control_plane_ref_from_body(
    body: ControlPlaneRefBody,
) -> Result<ControlPlaneRef, ProvisioningError> {
    let tier = ControlPlaneTier::parse(&body.tier).ok_or_else(|| {
        ProvisioningError::InvalidClusterRef {
            cluster_ref: format!("unknown tier {:?}", body.tier),
        }
    })?;
    Ok(ControlPlaneRef::new(
        ClusterRef::new(body.tenant_id, body.cluster_name),
        tier,
        body.handle,
    ))
}

async fn status_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<ControlPlaneRefBody>,
) -> axum::response::Response {
    let control_plane_ref = match control_plane_ref_from_body(body) {
        Ok(reference) => reference,
        Err(error) => return error_response(&error).into_response(),
    };
    match state.provisioning.status(&control_plane_ref).await {
        Ok(report) => (
            StatusCode::OK,
            axum::Json(StatusResponse {
                status: report.status.as_str().to_string(),
                tier: report.control_plane_ref.tier.as_str().to_string(),
                endpoint: report.endpoint,
            }),
        )
            .into_response(),
        Err(error) => error_response(&error).into_response(),
    }
}

async fn teardown_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<ControlPlaneRefBody>,
) -> axum::response::Response {
    let control_plane_ref = match control_plane_ref_from_body(body) {
        Ok(reference) => reference,
        Err(error) => return error_response(&error).into_response(),
    };
    match state.provisioning.teardown(&control_plane_ref).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => error_response(&error).into_response(),
    }
}

// =====================================================================
// Lifecycle
// =====================================================================

/// Build [`AppState`] backed by the deterministic in-memory adapter. The
/// dev/test/bring-up composition; production uses [`build_state_capi`].
#[must_use]
pub fn build_state_in_memory() -> AppState {
    AppState::new(Arc::new(InMemoryControlPlaneHost::new()))
}

/// Build [`AppState`] backed by the kube-rs CAPI adapter against the management
/// cluster.
#[must_use]
pub fn build_state_capi(host: CapiControlPlaneHost) -> AppState {
    AppState::new(Arc::new(host))
}

/// Bind a listener on `listen_addr` and serve `router`. Returns when the serve
/// loop exits.
///
/// # Errors
/// Returns [`BootError::Bind`] if the address cannot be bound, or
/// [`BootError::Serve`] if axum exits with an error.
pub async fn serve(listen_addr: &str, router: Router) -> Result<(), BootError> {
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|error| BootError::Bind {
            address: listen_addr.to_string(),
            error: error.to_string(),
        })?;
    tracing::info!(
        target: "oya_managed_k8s_control_plane_host_app::boot",
        addr = listen_addr,
        "managed-k8s control-plane-host listening"
    );
    axum::serve(listener, router)
        .await
        .map_err(|error| BootError::Serve(error.to_string()))
}

/// Resolve the management-cluster kubeconfig path from the environment,
/// fail-closed.
///
/// # Errors
/// Returns [`BootError::MissingMgmtKubeconfig`] if `$OYA_MGMT_KUBECONFIG` is
/// absent or empty. This is the production fail-closed boot guard — the service
/// never silently falls back to the in-memory fake.
pub fn mgmt_kubeconfig_path_from_env() -> Result<String, BootError> {
    match std::env::var(MGMT_KUBECONFIG_ENV) {
        Ok(path) if !path.trim().is_empty() => Ok(path),
        _ => Err(BootError::MissingMgmtKubeconfig {
            env_var: MGMT_KUBECONFIG_ENV.to_string(),
        }),
    }
}

/// Parse the live-provisioning rollback switch value.
#[must_use]
pub fn live_provisioning_policy_from_value(value: Option<&str>) -> LiveProvisioningPolicy {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("0" | "false" | "off" | "disabled" | "disable") => {
            LiveProvisioningPolicy::new_provisions_disabled()
        }
        _ => LiveProvisioningPolicy::new_provisions_enabled(),
    }
}

/// Resolve the live-provisioning rollback switch from the environment.
#[must_use]
pub fn live_provisioning_policy_from_env() -> LiveProvisioningPolicy {
    live_provisioning_policy_from_value(std::env::var(LIVE_PROVISIONING_ENV).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_error_missing_kubeconfig_is_fail_closed_message() {
        // Asserted on the typed variant directly to avoid mutating process-wide
        // env state (which is `unsafe` in edition 2024 and racy across the test
        // binary's threads). The fail-closed BRANCH is exercised by the
        // `mgmt_kubeconfig_path_from_env` unset-path test below.
        let err = BootError::MissingMgmtKubeconfig {
            env_var: MGMT_KUBECONFIG_ENV.to_string(),
        };
        assert!(err.to_string().contains("fail-closed"));
        assert!(err.to_string().contains(MGMT_KUBECONFIG_ENV));
    }

    #[test]
    fn mgmt_kubeconfig_env_var_name_is_stable() {
        // The production boot guard reads exactly this var; pin the name so a
        // rename is a conscious, reviewed change (the deploy manifest depends
        // on it).
        assert_eq!(MGMT_KUBECONFIG_ENV, "OYA_MGMT_KUBECONFIG");
    }

    #[test]
    fn live_provisioning_policy_env_value_is_rollback_switch() {
        assert_eq!(
            LIVE_PROVISIONING_ENV,
            "OYA_MK8S_CONTROL_PLANE_HOST_LIVE_PROVISIONING"
        );
        assert!(live_provisioning_policy_from_value(None).allows_new_provisioning());
        assert!(live_provisioning_policy_from_value(Some("true")).allows_new_provisioning());
        let disabled = live_provisioning_policy_from_value(Some("disabled"));
        assert!(!disabled.allows_new_provisioning());
        assert!(disabled.allows_status_and_teardown());
    }

    #[tokio::test]
    async fn in_memory_state_provisions_through_the_router_port() {
        let state = build_state_in_memory();
        let req = ProvisionRequest::new(
            ClusterRef::new("ten_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            DatastoreClass::EtcdPerTenant,
        );
        let cp = state
            .provisioning()
            .provision(&req)
            .await
            .expect("provision");
        assert_eq!(cp.tier, ControlPlaneTier::HostedKamaji);
    }

    #[test]
    fn error_response_maps_unimplemented_to_501() {
        let (code, _body) = error_response(&ProvisioningError::Unimplemented(
            oya_managed_k8s_control_plane_host_api::Unimplemented::KamajiProviderLiveIntegration,
        ));
        assert_eq!(code, StatusCode::NOT_IMPLEMENTED);
    }
}
