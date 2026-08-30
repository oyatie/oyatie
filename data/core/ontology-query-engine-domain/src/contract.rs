//! Query contract: request/response shapes, limits, errors, validators.

use data_ontology_kernel::ObjectGraph;

use crate::link::KnowledgeGraphLinkInstance;

/// Direction of BFS traversal relative to the root entity.
///
/// `Outbound` (the default) follows edges from `from_entity_id` to `to_entity_id`.
/// `Inbound` follows edges in reverse — from `to_entity_id` back to `from_entity_id`.
/// `Both` is the union; edges are emitted in canonical `from→to` orientation in all cases.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TraversalDirection {
    /// Follow edges in the forward (from→to) direction. Default.
    #[default]
    Outbound,
    /// Follow edges in the reverse (to→from) direction.
    Inbound,
    /// Follow edges in both directions.
    Both,
}

/// Hard cap for source-level recursive traversal in this preview foundation.
pub const MAX_QUERY_DEPTH: u32 = 16;

/// Hard cap on nodes returned in a single query result to bound blast radius.
pub const MAX_QUERY_RESULT_NODES: usize = 1_000;

/// Hard cap on edges returned in a single query result to bound blast radius.
pub const MAX_QUERY_RESULT_EDGES: usize = 5_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeGraphQueryResponse {
    pub query_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub nodes: Vec<KnowledgeGraphNode>, // data_class: INTERNAL_ONLY
    pub edges: Vec<KnowledgeGraphEdge>, // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    /// True when the result was truncated by node or edge cardinality caps.
    /// Callers must treat a truncated result as incomplete. // data_class: INTERNAL_ONLY
    pub result_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KnowledgeGraphNode {
    pub entity_id: String,      // data_class: INTERNAL_ONLY
    pub entity_type_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KnowledgeGraphEdge {
    pub from_entity_id: String, // data_class: INTERNAL_ONLY
    pub to_entity_id: String,   // data_class: INTERNAL_ONLY
    pub edge_type_id: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeGraphLinkUpsertOutcome {
    Inserted,
    Updated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KnowledgeGraphQueryError {
    InvalidTenantId,
    InvalidQueryId,
    InvalidEntityId,
    InvalidEdgeTypeId,
    /// `max_depth` is structurally invalid (e.g. zero).
    InvalidMaxDepth,
    /// `max_depth` exceeds [`MAX_QUERY_DEPTH`]; reduce the requested depth.
    DepthCeilingExceeded,
    MissingRootEntity,
    DanglingLinkEndpoint {
        entity_id: String,
    },
    /// A consent grant id in `consented_edge_type_ids` is structurally invalid
    /// (e.g. missing the `lty_` prefix).
    MalformedConsentGrantId {
        id: String,
    },
}

pub(crate) fn validate_link_endpoints(
    graph: &ObjectGraph,
    link: &KnowledgeGraphLinkInstance,
) -> Result<(), KnowledgeGraphQueryError> {
    if graph.get(&link.tenant_id, &link.from_entity_id).is_none() {
        return Err(KnowledgeGraphQueryError::DanglingLinkEndpoint {
            entity_id: link.from_entity_id.clone(),
        });
    }
    if graph.get(&link.tenant_id, &link.to_entity_id).is_none() {
        return Err(KnowledgeGraphQueryError::DanglingLinkEndpoint {
            entity_id: link.to_entity_id.clone(),
        });
    }
    Ok(())
}

pub(crate) fn validate_tenant_id(tenant_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > "ten_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidTenantId)
    }
}

pub(crate) fn validate_query_id(query_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if query_id.starts_with("kgq_") && query_id.len() > "kgq_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidQueryId)
    }
}

pub(crate) fn validate_entity_id(entity_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if entity_id.starts_with("ent_") && entity_id.len() > "ent_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidEntityId)
    }
}

pub(crate) fn validate_edge_type_id(edge_type_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if edge_type_id.starts_with("lty_") && edge_type_id.len() > "lty_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidEdgeTypeId)
    }
}

pub(crate) fn validate_max_depth(max_depth: u32) -> Result<(), KnowledgeGraphQueryError> {
    if max_depth == 0 {
        return Err(KnowledgeGraphQueryError::InvalidMaxDepth);
    }
    if max_depth > MAX_QUERY_DEPTH {
        return Err(KnowledgeGraphQueryError::DepthCeilingExceeded);
    }
    Ok(())
}

pub(crate) fn validate_consent_grant_id(grant_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if grant_id.starts_with("lty_") && grant_id.len() > "lty_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::MalformedConsentGrantId {
            id: grant_id.to_string(),
        })
    }
}
