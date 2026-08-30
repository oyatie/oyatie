//! Execution inputs, receipts, statuses, and audit event shapes.

use data_ontology_query_engine_domain::{
    KnowledgeGraphQueryError, KnowledgeGraphQueryRequest, KnowledgeGraphQueryResponse,
    TraversalDirection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryPolicyDecision {
    pub decision_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub allowed_query_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
    pub max_depth_ceiling: u32,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryExecutionInput {
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub query_surface: String,               // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,           // data_class: INTERNAL_ONLY
    pub request: KnowledgeGraphQueryRequest, // data_class: INTERNAL_ONLY
    pub policy_decision: OntologyQueryPolicyDecision, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OntologyQueryExecutionStatus {
    Completed,
    Denied,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyQueryDenialKind {
    InvalidInput,
    IdempotencyConflict,
    TenantMismatch,
    PrincipalMismatch,
    SurfaceDenied,
    DepthCeilingExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryExecutionReceipt {
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub principal_id: String,                 // data_class: INTERNAL_ONLY
    pub status: OntologyQueryExecutionStatus, // data_class: PUBLIC
    pub denial_kind: Option<OntologyQueryDenialKind>, // data_class: INTERNAL_ONLY
    pub failure: Option<KnowledgeGraphQueryError>, // data_class: INTERNAL_ONLY
    pub response: Option<KnowledgeGraphQueryResponse>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OntologyQueryAuditEventKind {
    QueryRequestReceived,
    QueryDenied,
    QueryFailed,
    QueryCompleted,
    IdempotencyConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyQueryAuditEvent {
    pub kind: OntologyQueryAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub query_id: String,                  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}
