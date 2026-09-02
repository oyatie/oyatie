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
use foundry_records_draft::RecordsLogError;
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
        state.metrics.submit_refused();
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "no bearer credential",
        );
    };
    let Some(caller) = authenticate(&state.operators, token) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::UNAUTHORIZED,
            "credential",
            "the presented credential is not recognized",
        );
    };
    let Ok(request) = serde_json::from_str::<SubmitRequest>(&body) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the request body is not a well-formed submission",
        );
    };
    let Ok(action_id) = ActionTypeId::new(request.action_type.clone()) else {
        state.metrics.submit_refused();
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
        state.metrics.submit_refused();
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision point refused this invocation",
        );
    };

    // The tenant is the CREDENTIAL's. Nothing in the body can move it.
    let Some(tenant) = state.tenants.get(&caller.tenant_id) else {
        state.metrics.submit_refused();
        return refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the credential names a tenant this process does not serve",
        );
    };
    let mut tenant = tenant.lock().await;

    let Ok(edits) = edits_for(&request) else {
        state.metrics.submit_refused();
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
    let outcome = submit(submission, log, denial_log, projection);
    // Every submission lands in exactly one of served or refused, including
    // the ones refused before the writer was reached — an availability
    // denominator that omitted authorization failures would report a number
    // flatter than the service. A POISONED outcome counts as served: the log
    // accepted it and the projection refused it by law, which is the system
    // working, not an outage.
    match &outcome {
        Ok(_) => state.metrics.submit_served(),
        Err(_) => state.metrics.submit_refused(),
    }
    match outcome {
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
        Err(WriteError::Log(RecordsLogError::IdempotencyConflict { .. })) => refuse(
            StatusCode::CONFLICT,
            "log",
            "this idempotency key is already spent on different content",
        ),
        // The OTHER variant means the opposite thing. A storage fault is the
        // service failing, not the caller colliding with themselves, and it
        // is the one LOG failure here that a retry of the same bytes may get
        // past. (Not the only retryable refusal on this surface: a PDP
        // timeout or open circuit collapses to 403 at `authz.rs:79-82`, which
        // `authorizer_outage_is_deny`'s header settles as deliberate — no
        // test exercises a timeout or an open circuit.)
        //
        // The arm is COARSER than the error it answers: `Storage` funnels
        // every `rusqlite` failure, so a transient lock and a corrupt page
        // arrive identically and cannot be told apart here. 503 is chosen for
        // the common transient case; the message promises nothing about a
        // retry, because for the corrupt case no retry will help.
        //
        // The detail is not echoed. The 403 above echoes `refused.cause` and
        // the 200 echoes `poison_reason`, both INTERNAL_ONLY, so operator
        // reachability is not the axis. `refused.cause` is an authored
        // `&'static str`; `poison_reason` renders kernel errors whose STRING
        // payloads are the caller's own submitted names, going back to that
        // caller — the rest are ordinals, wire tags and revisions. Neither
        // carries anything from this process's environment.
        // `rusqlite::Error` does: it is unbounded and path-bearing.
        Err(WriteError::Log(RecordsLogError::Storage { .. })) => refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "log",
            "the action log could not be written; the submission was not accepted",
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
