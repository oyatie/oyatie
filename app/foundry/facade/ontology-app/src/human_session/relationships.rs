use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use data_ontology_kernel::{EntityTypeId, LinkTypeId};
use foundry_ontology_query_usecase::{
    OntologyQueryDenialKind, OntologyQueryExecutionInput, OntologyQueryExecutionStatus,
    OntologyQueryPolicyDecision,
};
use foundry_projection_draft::PageRequest;
use foundry_spine::{ObjectSet, SetDefinition, materialize_object_set};

use super::{HumanReadState, authorized_membership, now_epoch_seconds, session_refusal};
use crate::reads::{TENANT_SCOPED_RESOURCE, declared_type, refuse};
use crate::search_around::admission::{self, GraphReadRefusal};
mod body;
mod transport;
pub(crate) use body::Request as HumanRelationshipIntent;
use body::{Direction, Request};

const SURFACE: &str = "foundry.ontology.search-around";

pub(crate) async fn search_around(
    State(state): State<HumanReadState>,
    Path(tenant_id): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let access =
        match authorized_membership(&state, &tenant_id, &headers, TENANT_SCOPED_RESOURCE).await {
            Ok(access) => access,
            Err(response) => return *response,
        };
    let body = match transport::request_body(&state, &headers, body).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let Some(request) = Request::parse(&body) else {
        return invalid(&state);
    };
    let mut tenant = access.tenant.lock().await;
    let Some(entity_type) = declared_type(
        &tenant.projection.engine,
        &tenant_id,
        &request.root.entity_type,
    ) else {
        return invalid(&state);
    };
    let Some(link) = LinkTypeId::new(&request.link_type)
        .ok()
        .and_then(|id| tenant.projection.engine.link_type(&tenant_id, &id))
    else {
        return invalid(&state);
    };
    let root_type = match request.direction {
        Direction::Outbound => &link.from_entity_type,
        Direction::Inbound => &link.to_entity_type,
    };
    if root_type != &entity_type {
        return invalid(&state);
    }
    if state
        .app
        .pep
        .decide(
            &access.caller,
            crate::pdp::Surface::Use,
            &request.root.object_ref,
            &tenant_id,
        )
        .is_err()
    {
        state.app.metrics.read_refused();
        return GraphReadRefusal::Forbidden.response();
    }
    let set = ObjectSet {
        entity_type,
        definition: SetDefinition::Named(vec![request.root.object_ref.clone()]),
    };
    let seeded = match materialize_object_set(
        &*tenant.projection_store,
        &tenant.projection.engine,
        &tenant_id,
        &set,
        request.root.revision,
        &PageRequest::first(1),
    ) {
        Ok(page) => page,
        Err(error) => return crate::object_set::refuse_set_error(&state.app, error),
    };
    if seeded.objects.len() != 1 || seeded.objects[0].0 != request.root.object_ref {
        return invalid(&state);
    }
    if let Err(error) = now_epoch_seconds().and_then(|now| access.membership.ensure_current(now)) {
        return session_refusal(&state.app, error);
    }
    let intent_key = (
        access.caller.principal_id.clone(),
        request.idempotency_key.clone(),
    );
    if let Some(previous) = tenant.human_query_intents.get(&intent_key) {
        if previous != &request {
            return invalid(&state);
        }
    } else {
        tenant
            .human_query_intents
            .insert(intent_key, request.clone());
    }
    let decision_id = tenant
        .query_decisions
        .entry((
            access.caller.principal_id.clone(),
            request.idempotency_key.clone(),
        ))
        .or_insert(access.decision.decision_id)
        .clone();
    let query = match request.query(&tenant_id) {
        Ok(query) => query,
        Err(_) => return invalid(&state),
    };
    let input = OntologyQueryExecutionInput {
        idempotency_key: request.idempotency_key.clone(),
        principal_id: access.caller.principal_id.clone(),
        query_surface: SURFACE.into(),
        request_evidence_ref: decision_id.clone(),
        trace_context_ref: decision_id.clone(),
        request: query,
        policy_decision: OntologyQueryPolicyDecision {
            decision_id: decision_id.clone(),
            tenant_id: tenant_id.clone(),
            principal_id: access.caller.principal_id.clone(),
            allowed_query_surfaces: vec![SURFACE.into()],
            max_depth_ceiling: 1,
            evidence_ref: decision_id,
        },
    };
    let (queries, store) = tenant.query_handles();
    let receipt = queries.execute(store, input);
    let response = match receipt.status {
        OntologyQueryExecutionStatus::Completed => match receipt.response {
            Some(response) => response,
            None => return unavailable(&state),
        },
        OntologyQueryExecutionStatus::Denied => {
            if receipt.denial_kind == Some(OntologyQueryDenialKind::IdempotencyConflict) {
                return invalid(&state);
            }
            state.app.metrics.read_refused();
            return crate::search_around_refusals::refuse_denial(receipt.denial_kind);
        }
        OntologyQueryExecutionStatus::Failed => {
            return match receipt.failure {
                Some(error) => {
                    state.app.metrics.read_refused();
                    crate::search_around_refusals::refuse_query_error(&error)
                }
                None => unavailable(&state),
            };
        }
    };
    let nodes = match admission::admit(&state.app, &access.caller, &tenant, &response) {
        Ok(nodes) => nodes,
        Err(error) => {
            state.app.metrics.read_refused();
            return error.response();
        }
    };
    // Only the human detail handoff requires each node's own retained written revision.
    if nodes.iter().any(|node| {
        EntityTypeId::new(&node.entity_type_id)
            .ok()
            .and_then(|id| {
                tenant.projection.engine.entity_type_at_revision(
                    &tenant_id,
                    &id,
                    node.written_revision,
                )
            })
            .is_none()
    }) {
        state.app.metrics.read_refused();
        return GraphReadRefusal::InvalidGraph.response();
    }
    if let Err(error) = now_epoch_seconds().and_then(|now| access.membership.ensure_current(now)) {
        return session_refusal(&state.app, error);
    }
    state.app.metrics.read_served();
    Json(serde_json::json!({"nodes":nodes,"edges":response.edges.into_iter().map(|edge| serde_json::json!({"from_entity_id":edge.from_entity_id,"to_entity_id":edge.to_entity_id,"edge_type_id":edge.edge_type_id})).collect::<Vec<_>>(), "observed_at_epoch_seconds":response.observed_at_epoch_seconds,"truncated":response.result_truncated})).into_response()
}
fn invalid(state: &HumanReadState) -> Response {
    state.app.metrics.read_refused();
    refuse(
        StatusCode::BAD_REQUEST,
        "surface",
        "a relationship read requires one declared root, accepted relationship and direction",
    )
}
fn unavailable(state: &HumanReadState) -> Response {
    state.app.metrics.read_refused();
    refuse(
        StatusCode::SERVICE_UNAVAILABLE,
        "store",
        "the query did not provide a result",
    )
}
