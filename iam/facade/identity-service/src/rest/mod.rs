//! REST subsystem: the service router.
//!
//! Mounts the workload-identity router from `identity-workload-rest`
//! (single decision core — no logic re-implemented here) and adds the
//! K8s-native health surface (`/healthz` liveness, `/readyz` readiness).

use axum::Json;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::get;

use iam_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_rest::{AuditSink, SharedState, build_router};

/// Global request-body cap for the service entry (ADR-0581 §"Body-limit"): the
/// decision handlers now read the raw body (`Bytes`) so the caller is verified
/// before deserialization; a bounded limit keeps an unauthenticated caller from
/// streaming an unbounded body at the socket. 256 KiB comfortably covers the
/// largest legitimate `/authorize:batch` payload.
const REQUEST_BODY_LIMIT_BYTES: usize = 256 * 1024;

/// `GET` — health probe (the Helm chart's readiness target).
pub const HEALTHZ_ROUTE: &str = "/healthz";
/// `GET` — liveness probe (the Helm chart's liveness target).
pub const LIVEZ_ROUTE: &str = "/livez";
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
        .route(LIVEZ_ROUTE, get(healthz))
        .route(READYZ_ROUTE, get(readyz))
        // Global body cap at the service entry (authn runs before the body is read).
        .layer(DefaultBodyLimit::max(REQUEST_BODY_LIMIT_BYTES))
}
