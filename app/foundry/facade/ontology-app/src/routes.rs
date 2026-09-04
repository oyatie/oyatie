//! The HTTP surface this lane serves: two probes, metrics, and an operator
//! status endpoint that refuses.
//!
//! `/statusz` returning 403 is not a placeholder — it is the deny-by-default
//! posture stated in code. The endpoint is authorized by a policy decision
//! point that no lane has composed yet, and a surface with no authorizer
//! must refuse rather than serve. It opens when the decision point lands,
//! not before.

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use crate::composition::AppState;
use crate::metrics::prometheus_text;

/// Build the router over composed state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/statusz", get(statusz))
        .route("/metrics", get(metrics))
        .route("/v1/actions", post(crate::submit::submit_action))
        .route("/v1/objects/{object_ref}", get(crate::reads::object))
        .route(
            "/v1/objects/{object_ref}/history",
            get(crate::reads::history),
        )
        .route("/v1/audit", get(crate::reads::audit))
        .route("/v1/types", get(crate::reads::types))
        .with_state(Arc::new(state))
}

/// Liveness: the listener is bound and this process is answering. It asks
/// nothing about correctness, so it must not consult readiness state.
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok\n")
}

/// Readiness: every tenant's fold has consumed its whole log, AND every
/// tenant could be read. Poison never enters this answer.
///
/// The two refusals are distinct and say so. A tenant that is behind and a
/// tenant nobody could read are both "not ready", but reporting the second as
/// "lagging" would name a state the process never observed — the failure this
/// surface's own signal was rebuilt to stop.
async fn readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let seen = crate::observation::observe(&state);
    if seen.is_caught_up() {
        (StatusCode::OK, "ready\n")
    } else if seen.unknown > 0 {
        (StatusCode::SERVICE_UNAVAILABLE, "unobserved\n")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "lagging\n")
    }
}

/// The operator status surface: deny-by-default until an authorizer exists.
async fn statusz() -> impl IntoResponse {
    (
        StatusCode::FORBIDDEN,
        "no policy decision point is composed; this surface refuses\n",
    )
}

/// Metrics carry no tenant labels: the exposition surface is unauthenticated
/// by design, so it must not become a tenancy oracle.
async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, prometheus_text(&state))
}
