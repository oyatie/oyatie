//! `POST /v1/actions` — the write surface.
//!
//! The order here is the whole security argument, and it runs
//! authenticate → authorize → submit. Nothing reaches the log before a
//! policy decision exists, and the decision this process hands the writer
//! is the PDP's own, so the appended entry is attributable to the
//! authorization that permitted it.
//!
//! This handler never decides what an object may carry. It converts the
//! request, and the registry — through the writer's gates and the fold's
//! re-check — decides everything else. A refusal is reported with the gate
//! that produced it so an operator knows whether to look at the roster,
//! the seed, or the schema.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use data_ontology_kernel::{ActionInvocationRequest, ActionTypeId};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_spine::{ActionSubmission, ApplyOutcome, WriteError, submit};

use crate::auth::{authenticate, bearer_token};
use crate::composition::AppState;
use crate::dto::{RefusalBody, SubmitRequest, SubmitResponse};
use crate::pdp::Surface;

/// Submit one Action.
pub async fn submit_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let Some(token) = bearer_token(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    ) else {
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        );
    };
    let Some(caller) = authenticate(&state.operators, token) else {
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        );
    };
    let Ok(request) = serde_json::from_str::<SubmitRequest>(&body) else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the request body is not a well-formed submission",
        );
    };
    let Ok(action_id) = ActionTypeId::new(request.action_type.clone()) else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the action type is not an action id",
        );
    };

    // AUTHORIZE before anything is built: the decision is the PDP's, and a
    // refusal ends the request here, with nothing appended anywhere.
    let Ok(decision) = state
        .pep
        .decide(&caller, Surface::Invoke, &request.object_ref)
    else {
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this invocation",
        );
    };

    // The tenant is the CREDENTIAL's. Nothing in the body can move it.
    let Some(tenant) = state.tenants.get(&caller.tenant_id) else {
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        );
    };
    let mut tenant = tenant.lock().await;

    let Ok(edits) = edits_for(&request) else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the submission carries no representable edit",
        );
    };
    let submission = ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: caller.tenant_id.clone(),
            principal_id: caller.principal_id.clone(),
            action_id,
            entity_id: request.object_ref.clone(),
            idempotency_key: request.idempotency_key.clone(),
            requested_at_epoch_seconds: request.occurred_at_epoch_seconds,
        },
        decision,
        parameters: Vec::new(),
        edits,
    };

    let (log, denial_log, projection) = tenant.write_handles();
    match submit(submission, log, denial_log, projection) {
        Ok(ApplyOutcome::Applied { receipt }) => Json(SubmitResponse {
            outcome: "applied",
            ordinal: receipt.ordinal,
            deduplicated: receipt.deduplicated,
            poison_reason: None,
        })
        .into_response(),
        Ok(ApplyOutcome::Poisoned { receipt, reason }) => Json(SubmitResponse {
            outcome: "poisoned",
            ordinal: receipt.ordinal,
            deduplicated: receipt.deduplicated,
            poison_reason: Some(format!("{reason:?}")),
        })
        .into_response(),
        Err(WriteError::Refused(refused)) => refuse(
            StatusCode::FORBIDDEN,
            &format!("{:?}", refused.gate).to_lowercase(),
            refused.cause,
        ),
        // A divergent reuse of a spent key is the caller's conflict to
        // resolve, not something to retry into.
        Err(WriteError::Log(_)) => refuse(
            StatusCode::CONFLICT,
            "log",
            "this idempotency key is already spent on different content",
        ),
    }
}

/// The edit set is a PURE FUNCTION OF THE REQUEST, and that is the retry
/// contract, not a simplification. An earlier version chose the edit kind
/// by asking whether the projection already held the object — which made
/// the same request produce different bytes before and after it landed, so
/// a retry arrived as divergent content under a spent key and conflicted
/// instead of deduplicating. Nothing about how the payload is built may
/// depend on state the request cannot see.
///
/// This surface therefore writes whole records. Partial-update semantics
/// belong to their own action type, declared in the registry, rather than
/// to a facade inferring intent from what it happens to know.
fn edits_for(request: &SubmitRequest) -> Result<EditSet, ()> {
    let properties: Vec<WireProperty> = request
        .properties
        .iter()
        .filter_map(|(name, value)| {
            WireProperty::new(
                name,
                WireTier::Scalar,
                WireDataClass::InternalOnly,
                WireValue::String(value.clone()),
            )
            .ok()
        })
        .collect();
    if properties.len() != request.properties.len() {
        return Err(());
    }
    let edit = OntologyEdit::create_object(SEEDED_ENTITY_TYPE, properties).map_err(|_| ())?;
    EditSet::new(vec![edit]).map_err(|_| ())
}

/// The entity type the seeded registry declares. The surface admits one
/// type while the registry is compiled in; the Ontology Manager vertical
/// owns making that a runtime choice.
const SEEDED_ENTITY_TYPE: &str = "ety_record";

fn refuse(status: StatusCode, gate: &str, cause: &str) -> Response {
    (
        status,
        Json(RefusalBody {
            gate: gate.to_owned(),
            cause: cause.to_owned(),
        }),
    )
        .into_response()
}
