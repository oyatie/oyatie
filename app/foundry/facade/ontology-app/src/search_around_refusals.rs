//! How a walk's refusals are answered: the policy's denials, and the domain's.
//!
//! Separated from the handler so each mapping is read beside its siblings
//! rather than beside the flow that reaches it.

use axum::http::StatusCode;
use axum::response::Response;
use foundry_ontology_query_domain::KnowledgeGraphQueryError;
use foundry_ontology_query_usecase::OntologyQueryDenialKind;

use crate::reads::refuse;

/// A policy denial is an authorization answer, except the one that is a replay
/// of a different intent under one key.
pub(crate) fn refuse_denial(kind: Option<OntologyQueryDenialKind>) -> Response {
    match kind {
        Some(OntologyQueryDenialKind::IdempotencyConflict) => refuse(
            StatusCode::CONFLICT,
            "surface",
            "that idempotency key is already answering another query",
        ),
        Some(OntologyQueryDenialKind::DepthCeilingExceeded) => refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "this process permits a shallower walk than that",
        ),
        Some(OntologyQueryDenialKind::InvalidInput) => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "an idempotency key must name the attempt it repeats",
        ),
        _ => refuse(
            StatusCode::FORBIDDEN,
            "authorization",
            "the policy decision does not permit this query",
        ),
    }
}

/// The domain's refusals. `Source` is the store failing to answer; every other
/// variant is the request's own shape, which the caller can correct.
pub(crate) fn refuse_query_error(error: &KnowledgeGraphQueryError) -> Response {
    match error {
        KnowledgeGraphQueryError::Source { .. } => refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "store",
            "the projection store could not be read",
        ),
        KnowledgeGraphQueryError::InvalidMaxDepth => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "a walk must go at least one step",
        ),
        KnowledgeGraphQueryError::DepthCeilingExceeded => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "a walk may go at most sixteen steps",
        ),
        KnowledgeGraphQueryError::InvalidEdgeTypeId
        | KnowledgeGraphQueryError::MalformedConsentGrantId { .. } => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "an edge type and a consent grant are both lty_ ids",
        ),
        _ => refuse(
            StatusCode::BAD_REQUEST,
            "surface",
            "the walk this names cannot be built",
        ),
    }
}
