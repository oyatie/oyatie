//! REST subsystem: the service router.
//!
//! Mounts the workload-identity router from `oya-identity-workload-rest`
//! (single decision core — no logic re-implemented here) and adds the
//! K8s-native health surface (`/healthz` liveness, `/readyz` readiness).

use axum::Json;
use axum::Router;
use axum::routing::get;

use oya_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use oya_identity_workload_authz_cedar_adapter::WorkloadAuthorizer;
use oya_identity_workload_rest::{AuditSink, SharedState, build_router};

/// `GET` — liveness probe.
pub const HEALTHZ_ROUTE: &str = "/healthz";
/// `GET` — readiness probe.
pub const READYZ_ROUTE: &str = "/readyz";

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz() -> Json<serde_json::Value> {
    // State construction is fail-fast at boot (JWKS/policies/seed all load
    // before bind), so a serving process is a ready process.
    Json(serde_json::json!({"status": "ready"}))
}

/// Build the full service router: workload-identity routes + health surface.
pub fn build_service_router<R, D, A, S>(state: SharedState<R, D, A, S>) -> Router
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    build_router(state)
        .route(HEALTHZ_ROUTE, get(healthz))
        .route(READYZ_ROUTE, get(readyz))
}
