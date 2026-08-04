//! REST decision surface (axum).
//!
//! Routes (the COMPLETE surface — anything else is 404, default-deny):
//! - `POST /v1/authorize` — decide one PARC request against the supplied
//!   entity slice. `200` carries an [`AuthorizationResponse`] (allow OR
//!   deny — a deny is a decision, not an error). ANY non-200 is a refusal
//!   with a machine `error_code`; PEPs MUST treat it as deny (fail-closed).
//! - `POST /v1/platform/authorize/<grpc-service>/<method>` — Envoy HTTP
//!   `ext_authz` adapter for NativeLink. Only the exact `nativelink-edge`
//!   transport SVID may delegate the sanitized caller SVID; attributable,
//!   current allows map to 200 and every other result maps to 403.
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

use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use os_trustd_domain::TrustBundle;
use os_trustd_domain::signer::EcdsaP256Signer;
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PdpError};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, PlatformAction,
    PlatformAuthorizeRequest, PlatformResourceKind, PolicyVersion,
};

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

const PLATFORM_EDGE_SERVICE: &str = "nativelink-edge";

fn header(headers: &HeaderMap, name: &'static str) -> Result<String, Response> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| caller_auth_refusal("platform authorization request is malformed"))
}

fn platform_action(grpc_path: &str) -> Option<PlatformAction> {
    match grpc_path {
        "build.bazel.remote.execution.v2.Capabilities/GetCapabilities" => {
            Some(PlatformAction::ReCapabilities)
        }
        "build.bazel.remote.execution.v2.Execution/Execute"
        | "build.bazel.remote.execution.v2.Execution/WaitExecution"
        | "build.bazel.remote.execution.v2.ContentAddressableStorage/FindMissingBlobs"
        | "build.bazel.remote.execution.v2.ContentAddressableStorage/BatchUpdateBlobs"
        | "build.bazel.remote.execution.v2.ContentAddressableStorage/BatchReadBlobs"
        | "build.bazel.remote.execution.v2.ContentAddressableStorage/GetTree"
        | "build.bazel.remote.execution.v2.ActionCache/GetActionResult"
        | "google.bytestream.ByteStream/Read"
        | "google.bytestream.ByteStream/Write"
        | "google.bytestream.ByteStream/QueryWriteStatus" => Some(PlatformAction::ReExecute),
        _ => None,
    }
}

fn platform_request(
    headers: &HeaderMap,
    grpc_path: &str,
) -> Result<PlatformAuthorizeRequest, Response> {
    if header(headers, "x-oya-edge-role")? != "re-input-client" {
        return Err(caller_auth_refusal(
            "platform authorization request is malformed",
        ));
    }
    let action = platform_action(grpc_path)
        .ok_or_else(|| caller_auth_refusal("platform authorization request is malformed"))?;
    let min_policy_version = match headers
        .get("x-oya-min-policy-version")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(
            PolicyVersion::new(value)
                .map_err(|_| caller_auth_refusal("platform authorization request is malformed"))?,
        ),
        None => None,
    };
    let request = PlatformAuthorizeRequest {
        request_id: header(headers, "x-request-id")?,
        delegated_principal: header(headers, "x-oya-delegated-spiffe-principal")?,
        action,
        resource_kind: PlatformResourceKind::RemoteExecutionCell,
        resource_id: header(headers, "x-oya-resource-id")?,
        build_class: header(headers, "x-oya-build-class")?,
        min_policy_version,
    };
    request
        .validate()
        .map_err(|_| caller_auth_refusal("platform authorization request is malformed"))?;
    Ok(request)
}

fn platform_parc(request: &PlatformAuthorizeRequest) -> (AuthorizationRequest, EntitySlice) {
    let principal = EntityRef {
        entity_type: "OyaPlatform::PlatformPrincipal".to_owned(),
        entity_id: request.delegated_principal.clone(),
    };
    let resource = EntityRef {
        entity_type: request.resource_kind.cedar_type().to_owned(),
        entity_id: request.resource_id.clone(),
    };
    let entities = EntitySlice {
        entities: vec![
            EntityRecord {
                uid: principal.clone(),
                attributes: std::collections::BTreeMap::from([(
                    "spiffe_id".to_owned(),
                    serde_json::Value::String(request.delegated_principal.clone()),
                )]),
                parents: vec![],
            },
            EntityRecord {
                uid: resource.clone(),
                attributes: std::collections::BTreeMap::from([
                    (
                        "resource_kind".to_owned(),
                        serde_json::Value::String("remote_execution_cell".to_owned()),
                    ),
                    (
                        "build_class".to_owned(),
                        serde_json::Value::String(request.build_class.clone()),
                    ),
                ]),
                parents: vec![],
            },
        ],
    };
    (
        AuthorizationRequest {
            request_id: request.request_id.clone(),
            tenant_id: "platform".to_owned(),
            principal,
            action: request.action.as_str().to_owned(),
            resource,
            context: std::collections::BTreeMap::new(),
            min_policy_version: request.min_policy_version.clone(),
        },
        entities,
    )
}

fn platform_ext_authz_status(
    result: Result<&AuthorizationResponse, &PdpError>,
    required_version: Option<&PolicyVersion>,
) -> StatusCode {
    match result {
        Ok(response)
            if response.decision == Decision::Allow
                && !response.determining_policy_ids.is_empty()
                && required_version
                    .is_none_or(|version| response.satisfies_exact_version(version)) =>
        {
            StatusCode::OK
        }
        Ok(_) | Err(_) => StatusCode::FORBIDDEN,
    }
}

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

async fn platform_authorize(
    State(state): State<Arc<PdpState>>,
    bundle: Option<Extension<CallerAuthBundle>>,
    cell: Option<Extension<CallerAuthCell>>,
    peer: Option<Extension<PeerCertInfo>>,
    Path(grpc_path): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(Extension(bundle)) = bundle else {
        return caller_auth_refusal("platform PEP mTLS identity required");
    };
    let Some(expected_cell) = cell.and_then(|Extension(cell)| cell.0) else {
        return caller_auth_refusal("platform PEP mTLS identity required");
    };
    let expected_pep = format!("spiffe://{expected_cell}/platform/{PLATFORM_EDGE_SERVICE}");
    let peer_leaf = peer
        .as_ref()
        .and_then(|Extension(info)| info.leaf_der.as_deref());
    let pep = match SpiffeCallerAuth::with_cell_pin(&bundle, &expected_cell) {
        Ok(pep) => pep,
        Err(error) => return caller_auth_refusal(&error.to_string()),
    };
    let transport_principal =
        match pep.authenticate_platform_pep(peer_leaf, &expected_pep, now_secs()) {
            Ok(principal) => principal,
            Err(error) => return caller_auth_refusal(error.public_message()),
        };
    let request = match platform_request(&headers, grpc_path.trim_start_matches('/')) {
        Ok(request) => request,
        Err(response) => return response,
    };
    let (parc, entities) = platform_parc(&request);
    match state.decide(&parc, &entities) {
        Ok(response)
            if platform_ext_authz_status(Ok(&response), request.min_policy_version.as_ref())
                == StatusCode::OK =>
        {
            tracing::info!(
                target: "oya_cloud_iam_pdp::platform_decision",
                request_id = %request.request_id,
                decision_id = %response.decision_id,
                transport_principal = %transport_principal,
                delegated_principal = %request.delegated_principal,
                decision = "allow",
                "platform authorization decision"
            );
            StatusCode::OK.into_response()
        }
        Ok(response) => {
            tracing::info!(
                target: "oya_cloud_iam_pdp::platform_decision",
                request_id = %request.request_id,
                decision_id = %response.decision_id,
                transport_principal = %transport_principal,
                delegated_principal = %request.delegated_principal,
                decision = "deny",
                "platform authorization decision"
            );
            StatusCode::FORBIDDEN.into_response()
        }
        Err(error) => {
            PdpState::log_refusal(&request.request_id, &error);
            tracing::warn!(
                target: "oya_cloud_iam_pdp::platform_decision",
                request_id = %request.request_id,
                transport_principal = %transport_principal,
                delegated_principal = %request.delegated_principal,
                decision = "refused",
                error = %error,
                "platform authorization refusal"
            );
            StatusCode::FORBIDDEN.into_response()
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
        .route(
            "/v1/platform/authorize/{*grpc_path}",
            post(platform_authorize),
        )
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

    #[test]
    fn purpose_adapter_rejects_unknown_rpc_role_and_unsanitized_identity() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", "req-re-1".parse().unwrap());
        headers.insert("x-oya-edge-role", "re-input-client".parse().unwrap());
        headers.insert(
            "x-oya-delegated-spiffe-principal",
            "spiffe://oyatie.cell-build/platform/ci-re-input-client"
                .parse()
                .unwrap(),
        );
        headers.insert("x-oya-resource-id", "cell-build".parse().unwrap());
        headers.insert("x-oya-build-class", "trusted-dev".parse().unwrap());

        let request = platform_request(
            &headers,
            "build.bazel.remote.execution.v2.Execution/Execute",
        )
        .unwrap();
        assert_eq!(request.action, PlatformAction::ReExecute);

        headers.insert("x-oya-edge-role", "worker-writer".parse().unwrap());
        assert_eq!(
            platform_request(
                &headers,
                "build.bazel.remote.execution.v2.Execution/Execute"
            )
            .unwrap_err()
            .status(),
            StatusCode::FORBIDDEN
        );
        headers.insert("x-oya-edge-role", "re-input-client".parse().unwrap());
        headers.insert(
            "x-oya-delegated-spiffe-principal",
            "spiffe://oyatie.cell-build/platform/a,spiffe://evil/platform/b"
                .parse()
                .unwrap(),
        );
        assert!(
            platform_request(
                &headers,
                "build.bazel.remote.execution.v2.Execution/Execute"
            )
            .is_err()
        );
        assert!(platform_request(&headers, "unknown.Service/Method").is_err());
    }

    #[test]
    fn purpose_adapter_maps_only_attributable_current_allow_to_200() {
        let current = PolicyVersion::new("psv-current").unwrap();
        let stale = PolicyVersion::new("psv-stale").unwrap();
        let mut response = AuthorizationResponse {
            decision_id: "dec-re-1".to_owned(),
            request_id: "req-re-1".to_owned(),
            decision: Decision::Allow,
            policy_version: current.clone(),
            determining_policy_ids: vec!["re-execute-trusted-input-client".to_owned()],
            obligations: vec![],
        };
        assert_eq!(
            platform_ext_authz_status(Ok(&response), Some(&current)),
            StatusCode::OK
        );
        assert_eq!(
            platform_ext_authz_status(Ok(&response), Some(&stale)),
            StatusCode::FORBIDDEN
        );
        response.decision = Decision::Deny;
        assert_eq!(
            platform_ext_authz_status(Ok(&response), None),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            platform_ext_authz_status(Err(&PdpError::RuntimeTimeout { deadline_ms: 250 }), None),
            StatusCode::FORBIDDEN
        );
    }
}
