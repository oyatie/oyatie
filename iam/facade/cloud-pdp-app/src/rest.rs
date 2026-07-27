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
//! | `AuditChainEmission`    | 500    | `audit_chain_emission`     |
//! | `RuntimeTimeout`        | 504    | `runtime_timeout`         |
//! | `RuntimePanic`          | 500    | `runtime_panic`           |
//! | `CircuitOpen`           | 503    | `circuit_open`            |

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use os_trustd_domain::TrustBundle;
use os_trustd_domain::signer::EcdsaP256Signer;
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PdpError};
use oya_shared_platform_contracts_kernel::pdp::AuthorizationRequest;

use crate::PdpState;
use crate::mtls::SpiffeCallerAuth;
use crate::mtls_transport::PeerCertInfo;

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
        PdpError::AuditChainEmission { .. } => {
            (StatusCode::INTERNAL_SERVER_ERROR, "audit_chain_emission")
        }
        PdpError::RuntimeTimeout { .. } => (StatusCode::GATEWAY_TIMEOUT, "runtime_timeout"),
        PdpError::RuntimePanic { .. } => (StatusCode::INTERNAL_SERVER_ERROR, "runtime_panic"),
        PdpError::CircuitOpen { .. } => (StatusCode::SERVICE_UNAVAILABLE, "circuit_open"),
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

/// The mTLS PEP context layered onto the router per connection: the trust bundle
/// (request-tenant authority) plus the captured peer leaf. Both are `Extension`s
/// so the plain-TCP router (no layers) runs the legacy verbatim-tenant path.
type CallerAuthBundle = Arc<TrustBundle<EcdsaP256Signer>>;

/// The PDP's own cell authority (`oyatie.cell-<id>`), layered alongside the trust
/// bundle so `authorize` can pin a caller's cell. Wrapped so its presence is an
/// `Extension` distinct from the bundle; the inner `Option` is `None` when the
/// server leaf carries no SPIFFE id (legacy node-style cert ⇒ no cell pin).
#[derive(Clone)]
struct CallerAuthCell(Option<String>);

/// Current wall-clock as epoch seconds (clock-before-epoch ⇒ `0` ⇒ every SVID
/// expired ⇒ fail-closed DENY, never a spurious accept).
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A 403 refusal carrying the coarse auth-failure class (never 404, never the
/// SVID/tenant detail — anti-enumeration, mirroring the gRPC PermissionDenied).
fn caller_auth_refusal(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error_code": "caller_unauthenticated",
            "detail": message,
        })),
    )
        .into_response()
}

async fn authorize(
    State(state): State<Arc<PdpState>>,
    bundle: Option<Extension<CallerAuthBundle>>,
    cell: Option<Extension<CallerAuthCell>>,
    peer: Option<Extension<PeerCertInfo>>,
    Json(mut body): Json<AuthorizeBody>,
) -> Response {
    // mTLS PEP (the #717 closure): when a trust bundle is layered (mTLS boot),
    // authenticate the caller by its verified peer SVID and bind the tenant from
    // the SVID path BEFORE deciding. Reject ⇒ 403, never 404, never a verbatim
    // fall-through. Plain-TCP boot (no bundle layer) keeps the legacy path.
    if let Some(Extension(bundle)) = bundle {
        let peer_leaf = peer
            .as_ref()
            .and_then(|Extension(info)| info.leaf_der.as_deref());
        // Pin the caller's cell to the PDP's own when the mTLS layer carries it.
        let expected_cell = cell.and_then(|Extension(c)| c.0);
        let pep_result = match &expected_cell {
            Some(cell) => SpiffeCallerAuth::with_cell_pin(&bundle, cell.as_str()),
            None => SpiffeCallerAuth::new(&bundle),
        };
        let pep = match pep_result {
            Ok(pep) => pep,
            Err(err) => return caller_auth_refusal(&err.to_string()),
        };
        match pep.authenticate_caller(peer_leaf, &body.request.tenant_id, now_secs()) {
            Ok(bound) => body.request.tenant_id = bound.as_str().to_owned(),
            Err(rej) => return caller_auth_refusal(rej.public_message()),
        }
    }

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

/// Build the REST router over the shared state (plain-TCP boot — NO caller
/// authentication; the legacy verbatim-tenant path).
pub fn build_router(state: Arc<PdpState>) -> Router {
    Router::new()
        .route("/v1/authorize", post(authorize))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .fallback(unknown_route)
        .with_state(state)
}

/// Build the REST router with the mTLS PEP enforced: the trust bundle (and the
/// PDP's own cell authority, for caller cell-pinning) are layered as `Extension`s
/// so `authorize` authenticates the caller by its peer SVID, pins its cell, and
/// binds the tenant from the SVID path. The per-connection peer leaf is injected
/// separately by the mTLS accept loop (`mtls_transport`).
pub fn build_router_mtls(
    state: Arc<PdpState>,
    bundle: Arc<TrustBundle<EcdsaP256Signer>>,
    expected_cell_authority: Option<String>,
) -> Router {
    build_router(state)
        .layer(Extension(CallerAuthCell(expected_cell_authority)))
        .layer(Extension(bundle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_refusals_have_fail_closed_rest_mappings() {
        assert_eq!(
            refusal_parts(&PdpError::RuntimeTimeout { deadline_ms: 25 }),
            (StatusCode::GATEWAY_TIMEOUT, "runtime_timeout")
        );
        assert_eq!(
            refusal_parts(&PdpError::RuntimePanic {
                detail: "panic".to_owned(),
            }),
            (StatusCode::INTERNAL_SERVER_ERROR, "runtime_panic")
        );
        assert_eq!(
            refusal_parts(&PdpError::AuditChainEmission {
                detail: "append failed".to_owned(),
            }),
            (StatusCode::INTERNAL_SERVER_ERROR, "audit_chain_emission")
        );
        assert_eq!(
            refusal_parts(&PdpError::CircuitOpen {
                consecutive_failures: 3,
            }),
            (StatusCode::SERVICE_UNAVAILABLE, "circuit_open")
        );
    }
}
