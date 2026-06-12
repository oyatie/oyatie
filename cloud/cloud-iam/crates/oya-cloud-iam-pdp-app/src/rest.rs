//! REST decision surface (axum).
//!
//! Routes (the COMPLETE surface — anything else is 404, default-deny):
//! - `POST /v1/authorize` — decide one PARC request against the supplied
//!   entity slice. `200` carries an [`AuthorizationResponse`] (allow OR
//!   deny — a deny is a decision, not an error). ANY non-200 is a refusal
//!   with a machine `error_code`; PEPs MUST treat it as deny (fail-closed).
//! - `GET /healthz` — liveness.
//! - `GET /readyz` — readiness: serving implies a strict-validated bundle is
//!   loaded (boot refuses otherwise), echoed as `policy_version`.
//!
//! Refusal status mapping (every code is fail-closed):
//!
//! | `PdpError`              | status | `error_code`              |
//! |-------------------------|--------|---------------------------|
//! | `InvalidRequest`        | 400    | `invalid_request`         |
//! | `UnknownAction`         | 400    | `unknown_action`          |
//! | `StalePolicyVersion`    | 409    | `stale_policy_version`    |
//! | `Evaluation`            | 422    | `evaluation_refused`      |
//! | `BundleRejected`        | 503    | `bundle_rejected`         |
//! | `DecisionIdUnavailable` | 500    | `decision_id_unavailable` |

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PdpError};
use oya_shared_platform_contracts_kernel::pdp::AuthorizationRequest;

use crate::PdpState;

/// `POST /v1/authorize` body: the locked-contract request plus the
/// PEP-assembled entity slice. Closed schema — unknown fields are rejected
/// (a smuggled field is a contract violation, not an extension point).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizeBody {
    request: AuthorizationRequest,
    #[serde(default)]
    entities: Vec<EntityRecord>,
}

/// Map a PDP refusal to its REST status + machine error code.
fn refusal_parts(error: &PdpError) -> (StatusCode, &'static str) {
    match error {
        PdpError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
        PdpError::UnknownAction { .. } => (StatusCode::BAD_REQUEST, "unknown_action"),
        PdpError::StalePolicyVersion { .. } => (StatusCode::CONFLICT, "stale_policy_version"),
        PdpError::Evaluation { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "evaluation_refused"),
        PdpError::BundleRejected { .. } => (StatusCode::SERVICE_UNAVAILABLE, "bundle_rejected"),
        PdpError::DecisionIdUnavailable { .. } => {
            (StatusCode::INTERNAL_SERVER_ERROR, "decision_id_unavailable")
        }
    }
}

fn refusal_response(error: &PdpError) -> Response {
    let (status, error_code) = refusal_parts(error);
    (
        status,
        Json(serde_json::json!({
            "error_code": error_code,
            "detail": error.to_string(),
        })),
    )
        .into_response()
}

async fn authorize(
    State(state): State<Arc<PdpState>>,
    Json(body): Json<AuthorizeBody>,
) -> Response {
    let entities = EntitySlice {
        entities: body.entities,
    };
    match state.decide(&body.request, &entities) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            PdpState::log_refusal(&body.request.request_id, &error);
            refusal_response(&error)
        }
    }
}

async fn healthz() -> Response {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

async fn readyz(State(state): State<Arc<PdpState>>) -> Response {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ready",
            "policy_version": state.loaded_policy_version().as_str(),
        })),
    )
        .into_response()
}

/// Any route outside the declared surface: 404 with a machine code
/// (default-deny — there is no implicit surface).
async fn unknown_route() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error_code": "unknown_route",
            "detail": "no such surface; the decision surface is POST /v1/authorize",
        })),
    )
        .into_response()
}

/// Build the REST router over the shared state.
#[must_use]
pub fn build_router(state: Arc<PdpState>) -> Router {
    Router::new()
        .route("/v1/authorize", post(authorize))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .fallback(unknown_route)
        .with_state(state)
}
