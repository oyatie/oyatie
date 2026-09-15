//! Search-around: `POST /v1/search-around` walks the knowledge graph from an
//! object set.
//!
//! The seed is materialized through the spine's pinned view, so the walk starts
//! only from objects the reading tenant holds at a revision it accepted — the
//! same guard `GET /v1/objects` and the set page read under. The walk itself is
//! the query usecase's, executed against the DURABLE projection.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use foundry_ontology_query_domain::{KnowledgeGraphQueryError, KnowledgeGraphQueryResponse};
use foundry_ontology_query_usecase::{
    OntologyQueryDenialKind, OntologyQueryExecutionInput, OntologyQueryExecutionStatus,
    OntologyQueryPolicyDecision,
};
use foundry_projection_draft::PageRequest;
use foundry_spine::{MAX_SET_MEMBERS, ObjectSet, materialize_object_set};
use serde::Serialize;

use crate::composition::AppState;
use crate::object_set::refuse_set_error;
use crate::reads::{TENANT_SCOPED_RESOURCE, authorized, declared_type, refuse};
use crate::search_around_body::SearchAroundRequest;
use crate::search_around_refusals::{refuse_denial, refuse_query_error};

/// The query surface this process serves. The usecase refuses a surface outside
/// the declared set; this route names its own, so that refusal guards a future
/// caller rather than this one.
const SEARCH_AROUND_SURFACE: &str = "foundry.ontology.search-around";

/// The depth this process permits, BELOW the domain's own `MAX_QUERY_DEPTH` of
/// 16. A request between this and that is well formed and refused by policy,
/// which is what makes the policy ceiling a decision rather than a restatement
/// of the domain's bound.
const QUERY_DEPTH_CEILING: u32 = 8;

#[derive(Debug, Serialize)]
struct NodeBody {
    entity_id: String,      // data_class: INTERNAL_ONLY
    entity_type_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug, Serialize)]
struct EdgeBody {
    from_entity_id: String, // data_class: INTERNAL_ONLY
    to_entity_id: String,   // data_class: INTERNAL_ONLY
    edge_type_id: String,   // data_class: INTERNAL_ONLY
}

/// The graph a walk reached. `truncated` is the domain's own signal that the
/// result is a prefix; a caller must treat one as incomplete.
#[derive(Debug, Serialize)]
struct GraphBody {
    nodes: Vec<NodeBody>,
    edges: Vec<EdgeBody>,
    observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    truncated: bool,                // data_class: INTERNAL_ONLY
}

pub async fn search_around(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let (caller, decision, tenant) = match authorized(&state, &headers, TENANT_SCOPED_RESOURCE) {
        Ok(authorized) => authorized,
        Err(response) => return *response,
    };
    let request = match SearchAroundRequest::parse(&body) {
        Ok(request) => request,
        Err(refusal) => {
            state.metrics.read_refused();
            return refuse(StatusCode::BAD_REQUEST, "surface", refusal.cause());
        }
    };
    let mut tenant = tenant.lock().await;
    let Some(entity_type) = declared_type(
        &tenant.projection.engine,
        &caller.tenant_id,
        &request.entity_type,
    ) else {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "this tenant declares no entity type by that id",
        );
    };
    let set = ObjectSet {
        entity_type,
        definition: request.seed.clone(),
    };
    let seeded = match materialize_object_set(
        &*tenant.projection_store,
        &tenant.projection.engine,
        &caller.tenant_id,
        &set,
        request.revision,
        &PageRequest::first(MAX_SET_MEMBERS),
    ) {
        Ok(page) => page,
        Err(error) => return refuse_set_error(&state, error),
    };
    let roots: Vec<String> = seeded.objects.into_iter().map(|(id, _)| id).collect();
    // A seed that holds nothing cannot be walked: the domain refuses a walk
    // from a root it cannot find, so there is no graph to answer and no
    // execution to record. Judged before the walk's own fields, so a request
    // wrong in both ways is told about its seed.
    if roots.is_empty() {
        state.metrics.read_refused();
        return refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the seed holds no object to walk from",
        );
    }
    let idempotency_key = request.idempotency_key.clone();
    // The decision this key was first authorized under, so a replay is the same
    // intent. This attempt was authorized on its own above.
    let decision_id = tenant
        .query_decisions
        .entry((caller.principal_id.clone(), idempotency_key.clone()))
        .or_insert(decision.decision_id)
        .clone();
    let query = match request.into_query(&caller.tenant_id, roots) {
        Ok(query) => query,
        Err(error) => {
            state.metrics.read_refused();
            return refuse_query_error(&error);
        }
    };
    let input = OntologyQueryExecutionInput {
        idempotency_key,
        principal_id: caller.principal_id.clone(),
        query_surface: SEARCH_AROUND_SURFACE.to_owned(),
        request_evidence_ref: decision_id.clone(),
        trace_context_ref: decision_id.clone(),
        request: query,
        policy_decision: OntologyQueryPolicyDecision {
            decision_id: decision_id.clone(),
            tenant_id: caller.tenant_id.clone(),
            principal_id: caller.principal_id.clone(),
            allowed_query_surfaces: vec![SEARCH_AROUND_SURFACE.to_owned()],
            max_depth_ceiling: QUERY_DEPTH_CEILING,
            evidence_ref: decision_id,
        },
    };
    let (queries, store) = tenant.query_handles();
    let receipt = queries.execute(store, input);
    match receipt.status {
        OntologyQueryExecutionStatus::Completed => match receipt.response {
            Some(response) => {
                state.metrics.read_served();
                Json(graph_of(response)).into_response()
            }
            // A completed execution always carries its response; answering an
            // absent one as an empty graph would report a walk that never ran.
            None => {
                state.metrics.read_refused();
                refuse(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "store",
                    "the query completed without a result",
                )
            }
        },
        OntologyQueryExecutionStatus::Denied => {
            state.metrics.read_refused();
            refuse_denial(receipt.denial_kind)
        }
        OntologyQueryExecutionStatus::Failed => {
            state.metrics.read_refused();
            match receipt.failure {
                Some(error) => refuse_query_error(&error),
                None => refuse(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "store",
                    "the query failed without a reason",
                ),
            }
        }
    }
}

fn graph_of(response: KnowledgeGraphQueryResponse) -> GraphBody {
    GraphBody {
        nodes: response
            .nodes
            .into_iter()
            .map(|node| NodeBody {
                entity_id: node.entity_id,
                entity_type_id: node.entity_type_id,
            })
            .collect(),
        edges: response
            .edges
            .into_iter()
            .map(|edge| EdgeBody {
                from_entity_id: edge.from_entity_id,
                to_entity_id: edge.to_entity_id,
                edge_type_id: edge.edge_type_id,
            })
            .collect(),
        observed_at_epoch_seconds: response.observed_at_epoch_seconds,
        truncated: response.result_truncated,
    }
}
