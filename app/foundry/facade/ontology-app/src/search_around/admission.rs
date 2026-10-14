use std::collections::{BTreeMap, BTreeSet};

use axum::http::StatusCode;
use axum::response::Response;
use data_ontology_kernel::LinkTypeId;
use foundry_caller_draft::Caller;
use foundry_ontology_query_domain::KnowledgeGraphQueryResponse;

use crate::composition::{AppState, TenantState};
use crate::pdp::Surface;
use crate::reads::refuse;

pub(super) enum GraphReadRefusal {
    InvalidGraph,
    StoreUnreadable,
    Forbidden,
}

impl GraphReadRefusal {
    pub(super) fn response(self) -> Response {
        let (status, gate, cause) = match self {
            Self::InvalidGraph => (
                StatusCode::BAD_REQUEST,
                "surface",
                "the graph does not match the current projection and relationship declarations",
            ),
            Self::StoreUnreadable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "store",
                "the projection store could not be read",
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "authorization",
                "the policy decision point refused this read",
            ),
        };
        refuse(status, gate, cause)
    }
}

// Cached query receipts remain unchanged; every disclosure uses current objects and policy.
pub(super) fn admit(
    state: &AppState,
    caller: &Caller,
    tenant: &TenantState,
    graph: &KnowledgeGraphQueryResponse,
) -> Result<(), GraphReadRefusal> {
    let tenant_id = &tenant.projection.tenant_id;
    if graph.tenant_id != *tenant_id || caller.tenant_id != *tenant_id {
        return Err(GraphReadRefusal::InvalidGraph);
    }
    for node in &graph.nodes {
        state
            .pep
            .decide(caller, Surface::Use, &node.entity_id, tenant_id)
            .map_err(|_| GraphReadRefusal::Forbidden)?;
    }
    let mut nodes = BTreeMap::new();
    for node in &graph.nodes {
        let object = tenant
            .projection_store
            .get(tenant_id, &node.entity_id)
            .map_err(|_| GraphReadRefusal::StoreUnreadable)?
            .ok_or(GraphReadRefusal::InvalidGraph)?;
        if object.entity.tenant_id != *tenant_id
            || object.entity.id != node.entity_id
            || object.entity.entity_type.value != node.entity_type_id
            || nodes
                .insert(node.entity_id.as_str(), node.entity_type_id.as_str())
                .is_some()
        {
            return Err(GraphReadRefusal::InvalidGraph);
        }
    }
    let mut links: BTreeMap<&str, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for edge in &graph.edges {
        let kind =
            LinkTypeId::new(&edge.edge_type_id).map_err(|_| GraphReadRefusal::InvalidGraph)?;
        let definition = tenant
            .projection
            .engine
            .link_type(tenant_id, &kind)
            .ok_or(GraphReadRefusal::InvalidGraph)?;
        if nodes.get(edge.from_entity_id.as_str()).copied()
            != Some(definition.from_entity_type.value.as_str())
            || nodes.get(edge.to_entity_id.as_str()).copied()
                != Some(definition.to_entity_type.value.as_str())
            || !links
                .entry(&edge.from_entity_id)
                .or_default()
                .insert((&edge.edge_type_id, &edge.to_entity_id))
        {
            return Err(GraphReadRefusal::InvalidGraph);
        }
    }
    for (from, mut expected) in links {
        let stored = tenant
            .projection_store
            .links_from(tenant_id, from)
            .map_err(|_| GraphReadRefusal::StoreUnreadable)?;
        for edge in &stored {
            if edge.from_object_ref == from {
                expected.remove(&(edge.link_type.as_str(), edge.to_object_ref.as_str()));
            }
        }
        if !expected.is_empty() {
            return Err(GraphReadRefusal::InvalidGraph);
        }
    }
    Ok(())
}
