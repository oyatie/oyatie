use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{LinkSchema, OntologyReadError};

pub const MAX_RELATIONSHIP_REQUEST_BYTES: usize = 4096;
// One link and direction permit at most one rooted edge per node. With 1000
// nodes and 256-byte IDs, even JSON-escaped canonical output fits within 4 MiB.
pub const MAX_RELATIONSHIP_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_RELATIONSHIP_NODES: usize = 1000;
pub const MAX_RELATIONSHIP_EDGES: usize = 5000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipDirection {
    Outbound,
    Inbound,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipRoot {
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub object_ref: String,  // data_class: TENANT_SCOPED
    pub revision: u32,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipQuery {
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub root: RelationshipRoot,
    pub link_type: String, // data_class: INTERNAL_ONLY
    pub direction: RelationshipDirection,
    /// Frozen intent metadata, not evidence of projection freshness.
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

pub(crate) fn identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value.len() <= 256
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

impl RelationshipQuery {
    pub fn decode(bytes: &[u8]) -> Result<Self, OntologyReadError> {
        if bytes.len() > MAX_RELATIONSHIP_REQUEST_BYTES {
            return Err(OntologyReadError::InvalidQuery);
        }
        let shape: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| OntologyReadError::InvalidQuery)?;
        if !shape.is_object() || !shape.get("root").is_some_and(serde_json::Value::is_object) {
            return Err(OntologyReadError::InvalidQuery);
        }
        // Decode the original bytes too: duplicate keys must not be erased by Value.
        let query: Self =
            serde_json::from_slice(bytes).map_err(|_| OntologyReadError::InvalidQuery)?;
        query.validate()?;
        Ok(query)
    }

    pub fn validate(&self) -> Result<(), OntologyReadError> {
        if self.idempotency_key.is_empty()
            || self.idempotency_key.len() > 128
            || !self
                .idempotency_key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
            || !identifier(&self.root.entity_type, "ety_")
            || !identifier(&self.root.object_ref, "ent_")
            || !identifier(&self.link_type, "lty_")
            || self.observed_at_epoch_seconds > MAX_SAFE_INTEGER
        {
            return Err(OntologyReadError::InvalidQuery);
        }
        Ok(())
    }

    pub fn validate_for(&self, link: &LinkSchema) -> Result<(), OntologyReadError> {
        self.validate()?;
        link.validate()?;
        if !identifier(&link.link_type, "lty_")
            || !identifier(&link.from_entity_type, "ety_")
            || !identifier(&link.to_entity_type, "ety_")
        {
            return Err(OntologyReadError::SourceUnavailable);
        }
        let root_type = match self.direction {
            RelationshipDirection::Outbound => &link.from_entity_type,
            RelationshipDirection::Inbound => &link.to_entity_type,
        };
        if self.link_type != link.link_type || &self.root.entity_type != root_type {
            return Err(OntologyReadError::InvalidQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipNode {
    pub entity_id: String,      // data_class: TENANT_SCOPED
    pub entity_type_id: String, // data_class: INTERNAL_ONLY
    /// This object's own admitted stored pin, independent of the root request pin.
    pub written_revision: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipEdge {
    pub from_entity_id: String, // data_class: TENANT_SCOPED
    pub to_entity_id: String,   // data_class: TENANT_SCOPED
    pub edge_type_id: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipGraph {
    pub nodes: Vec<RelationshipNode>,
    pub edges: Vec<RelationshipEdge>,
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub truncated: bool,                // data_class: INTERNAL_ONLY
}

impl RelationshipGraph {
    pub fn decode(
        bytes: &[u8],
        query: &RelationshipQuery,
        link: &LinkSchema,
    ) -> Result<Self, OntologyReadError> {
        let unavailable = || OntologyReadError::SourceUnavailable;
        if bytes.len() > MAX_RELATIONSHIP_RESPONSE_BYTES {
            return Err(unavailable());
        }
        let shape: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| unavailable())?;
        if !shape.is_object()
            || ["nodes", "edges"].iter().any(|key| {
                shape
                    .get(key)
                    .and_then(serde_json::Value::as_array)
                    .is_none_or(|rows| rows.iter().any(|row| !row.is_object()))
            })
        {
            return Err(unavailable());
        }
        let graph: Self = serde_json::from_slice(bytes).map_err(|_| unavailable())?;
        graph.validate(query, link)?;
        Ok(graph)
    }

    pub fn validate(
        &self,
        query: &RelationshipQuery,
        link: &LinkSchema,
    ) -> Result<(), OntologyReadError> {
        query.validate_for(link)?;
        let unavailable = || OntologyReadError::SourceUnavailable;
        if self.nodes.is_empty()
            || self.nodes.len() > MAX_RELATIONSHIP_NODES
            || self.edges.len() > MAX_RELATIONSHIP_EDGES
            || self.observed_at_epoch_seconds != query.observed_at_epoch_seconds
        {
            return Err(unavailable());
        }
        let mut nodes = BTreeMap::new();
        for node in &self.nodes {
            if !identifier(&node.entity_id, "ent_")
                || !identifier(&node.entity_type_id, "ety_")
                || nodes
                    .insert(&node.entity_id, &node.entity_type_id)
                    .is_some()
            {
                return Err(unavailable());
            }
        }
        if nodes.get(&query.root.object_ref).copied() != Some(&query.root.entity_type) {
            return Err(unavailable());
        }
        let mut edges = BTreeSet::new();
        let mut connected = BTreeSet::from([&query.root.object_ref]);
        for edge in &self.edges {
            let rooted = match query.direction {
                RelationshipDirection::Outbound => edge.from_entity_id == query.root.object_ref,
                RelationshipDirection::Inbound => edge.to_entity_id == query.root.object_ref,
            };
            if !rooted
                || edge.edge_type_id != query.link_type
                || nodes.get(&edge.from_entity_id).copied() != Some(&link.from_entity_type)
                || nodes.get(&edge.to_entity_id).copied() != Some(&link.to_entity_type)
                || !edges.insert((&edge.from_entity_id, &edge.to_entity_id, &edge.edge_type_id))
            {
                return Err(unavailable());
            }
            connected.insert(&edge.from_entity_id);
            connected.insert(&edge.to_entity_id);
        }
        if nodes.keys().any(|id| !connected.contains(id)) {
            return Err(unavailable());
        }
        Ok(())
    }
}
