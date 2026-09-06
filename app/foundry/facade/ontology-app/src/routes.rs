//! The HTTP surface this process serves: two probes, the exposition, an
//! operator status endpoint, and the ontology read and write routes.
//!
//! `/statusz` refused unconditionally while no policy decision point was
//! composed. One is, and has been for several lanes — so it now authorizes
//! like the other tenant-wide views (`authorized` then `tenant_of`) and
//! answers. Its deny-by-default posture is unchanged; what changed is that
//! there is finally something to deny BY.

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
    router_from(Arc::new(state))
}

/// The same, over state the caller already shares — so a test can act on the
/// very state the router serves rather than a copy of it.
pub fn router_from(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/statusz", get(crate::status::statusz))
        .route("/metrics", get(metrics))
        .route("/v1/actions", post(crate::submit::submit_action))
        .route(
            "/v1/migrations/attest",
            post(crate::migrate::attest::attest),
        )
        .route("/v1/migrations/run", post(crate::migrate::run::run))
        .route("/v1/objects/{object_ref}", get(crate::reads::object))
        .route(
            "/v1/objects/{object_ref}/history",
            get(crate::reads::history),
        )
        .route("/v1/audit", get(crate::reads::audit))
        .route("/v1/types", get(crate::reads::types))
        .with_state(state)
}

/// Liveness: the listener is bound and this process is answering. It asks
/// nothing about correctness, so it must not consult readiness state.
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok\n")
}

/// Readiness: every tenant's fold has consumed its whole log, AND every
/// tenant could be read. Poison never enters this answer.
///
/// The three refusals are distinct and say so. Behind, unreadable, and busy
/// are all "not ready", and collapsing them would name a state the process
/// never observed — the failure this surface's own signal was rebuilt to
/// stop. A busy tenant in particular WAS observable; only this pass missed
/// it, which is why it is not reported as unobserved.
///
/// Readiness fails closed on all three where the freshness indicator does not
/// fail closed on contention: one retried 503 is cheap, and an error budget
/// spent on the service being used is not.
///
/// ORDER IS PART OF THE ANSWER, and contention comes last because it is the
/// only one of the three that is not a fault. A process that is genuinely
/// behind AND happens to hold a lock has measured a fault; reporting it as
/// contended would name a non-fault for a state the process did observe,
/// which is the mirror of the error this surface was split up to stop.
/// Mixed states are pinned per adjacent pair, so the priority cannot be
/// reordered silently.
async fn readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let seen = crate::observation::observe(&state);
    if seen.is_caught_up() {
        (StatusCode::OK, "ready\n")
    } else if seen.unreadable > 0 {
        (StatusCode::SERVICE_UNAVAILABLE, "unobserved\n")
    } else if seen.lag > 0 {
        (StatusCode::SERVICE_UNAVAILABLE, "lagging\n")
    } else if seen.contended > 0 {
        (StatusCode::SERVICE_UNAVAILABLE, "contended\n")
    } else {
        // Unreachable while `is_caught_up` is exactly the conjunction above,
        // and named rather than folded into the arm before it: a fourth
        // readiness cause added later would otherwise be announced as a busy
        // tenant, which is the one word here that means nothing is wrong.
        (StatusCode::SERVICE_UNAVAILABLE, "not ready\n")
    }
}

/// Metrics carry no tenant labels: the exposition surface is unauthenticated
/// by design, so it must not become a tenancy oracle.
async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, prometheus_text(&state))
}
