//! Workflow Studio DSL emitter domain.
//!
//! Emits deterministic, canonical `workflow_spec.v1` JSON from typed value
//! objects. The domain is pure: callers own storage, signing, policy, and
//! transport concerns.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

const WORKFLOW_SPEC_SCHEMA_VERSION: &str = "workflow_spec.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowSpecNodeKind {
    Http,
    Transform,
    Branch,
    Join,
    CapabilityCall,
    HumanReview,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSpecNode {
    pub id: String,                 // data_class: INTERNAL_ONLY
    pub kind: WorkflowSpecNodeKind, // data_class: PUBLIC
    pub label: String,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSpecEdge {
    pub from: String, // data_class: INTERNAL_ONLY
    pub to: String,   // data_class: INTERNAL_ONLY
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSpec {
    pub schema_version: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub definition_id: String,        // data_class: INTERNAL_ONLY
    pub version: String,              // data_class: PUBLIC
    pub nodes: Vec<WorkflowSpecNode>, // data_class: INTERNAL_ONLY
    pub edges: Vec<WorkflowSpecEdge>, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub enum WorkflowSpecEmitError {
    InvalidSchemaVersion,
    InvalidTenantId,
    InvalidDefinitionId,
    InvalidVersion,
    EmptyNodeSet,
    EmptyNodeId,
    EmptyNodeName,
    DuplicateNodeId(String),
    EmptyEdgeEndpoint,
    DuplicateEdge(String),
    SelfLoop(String),
    DanglingEdgeSource(String),
    DanglingEdgeTarget(String),
    GraphCycle(String),
    UnreachableNode(String),
    Json(serde_json::Error),
}

impl PartialEq for WorkflowSpecEmitError {
    fn eq(&self, other: &Self) -> bool {
        use WorkflowSpecEmitError::*;
        match (self, other) {
            (InvalidSchemaVersion, InvalidSchemaVersion)
            | (InvalidTenantId, InvalidTenantId)
            | (InvalidDefinitionId, InvalidDefinitionId)
            | (InvalidVersion, InvalidVersion)
            | (EmptyNodeSet, EmptyNodeSet)
            | (EmptyNodeId, EmptyNodeId)
            | (EmptyNodeName, EmptyNodeName)
            | (EmptyEdgeEndpoint, EmptyEdgeEndpoint) => true,
            (DuplicateNodeId(left), DuplicateNodeId(right))
            | (DuplicateEdge(left), DuplicateEdge(right))
            | (SelfLoop(left), SelfLoop(right))
            | (DanglingEdgeSource(left), DanglingEdgeSource(right))
            | (DanglingEdgeTarget(left), DanglingEdgeTarget(right))
            | (GraphCycle(left), GraphCycle(right))
            | (UnreachableNode(left), UnreachableNode(right)) => left == right,
            (Json(left), Json(right)) => left.to_string() == right.to_string(),
            _ => false,
        }
    }
}

impl From<serde_json::Error> for WorkflowSpecEmitError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl std::fmt::Display for WorkflowSpecEmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WorkflowSpecEmitError::*;
        match self {
            InvalidSchemaVersion => write!(f, "invalid schema version"),
            InvalidTenantId => write!(f, "invalid tenant id: must be prefixed 'ten_'"),
            InvalidDefinitionId => write!(f, "invalid definition id: must be prefixed 'wfd_'"),
            InvalidVersion => write!(f, "invalid version: must be semver"),
            EmptyNodeSet => write!(f, "node set must not be empty"),
            EmptyNodeId => write!(f, "node id must be prefixed 'wfn_'"),
            EmptyNodeName => write!(f, "node label must not be blank"),
            DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            EmptyEdgeEndpoint => write!(f, "edge endpoint must be prefixed 'wfn_'"),
            DuplicateEdge(key) => write!(f, "duplicate edge: {key}"),
            SelfLoop(key) => write!(f, "self-loop edge: {key}"),
            DanglingEdgeSource(key) => write!(f, "dangling edge source: {key}"),
            DanglingEdgeTarget(key) => write!(f, "dangling edge target: {key}"),
            GraphCycle(id) => write!(f, "graph contains a cycle involving node: {id}"),
            UnreachableNode(id) => write!(f, "node unreachable from any entry node: {id}"),
            Json(err) => write!(f, "JSON serialisation error: {err}"),
        }
    }
}

impl WorkflowSpecNode {
    pub fn new(
        id: impl Into<String>,
        kind: WorkflowSpecNodeKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
        }
    }
}

impl WorkflowSpecEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>, condition: Option<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            condition,
        }
    }
}

impl WorkflowSpec {
    pub fn new(
        tenant_id: impl Into<String>,
        definition_id: impl Into<String>,
        version: impl Into<String>,
        nodes: Vec<WorkflowSpecNode>,
        edges: Vec<WorkflowSpecEdge>,
    ) -> Self {
        Self {
            schema_version: WORKFLOW_SPEC_SCHEMA_VERSION.to_string(),
            tenant_id: tenant_id.into(),
            definition_id: definition_id.into(),
            version: version.into(),
            nodes,
            edges,
        }
    }

    pub fn schema_version() -> &'static str {
        WORKFLOW_SPEC_SCHEMA_VERSION
    }

    pub fn validate(&self) -> Result<(), WorkflowSpecEmitError> {
        if self.schema_version != WORKFLOW_SPEC_SCHEMA_VERSION {
            return Err(WorkflowSpecEmitError::InvalidSchemaVersion);
        }
        if !is_prefixed_nonempty(&self.tenant_id, "ten_") {
            return Err(WorkflowSpecEmitError::InvalidTenantId);
        }
        if !is_prefixed_nonempty(&self.definition_id, "wfd_") {
            return Err(WorkflowSpecEmitError::InvalidDefinitionId);
        }
        if !is_semver(&self.version) {
            return Err(WorkflowSpecEmitError::InvalidVersion);
        }
        if self.nodes.is_empty() {
            return Err(WorkflowSpecEmitError::EmptyNodeSet);
        }

        let mut node_ids = BTreeSet::new();
        for node in &self.nodes {
            if !is_prefixed_nonempty(&node.id, "wfn_") {
                return Err(WorkflowSpecEmitError::EmptyNodeId);
            }
            if node.label.trim().is_empty() {
                return Err(WorkflowSpecEmitError::EmptyNodeName);
            }
            if !node_ids.insert(node.id.clone()) {
                return Err(WorkflowSpecEmitError::DuplicateNodeId(node.id.clone()));
            }
        }

        let mut edge_keys = BTreeSet::new();
        for edge in &self.edges {
            if !is_prefixed_nonempty(&edge.from, "wfn_") || !is_prefixed_nonempty(&edge.to, "wfn_")
            {
                return Err(WorkflowSpecEmitError::EmptyEdgeEndpoint);
            }
            let key = edge_key(edge);
            if !edge_keys.insert(key.clone()) {
                return Err(WorkflowSpecEmitError::DuplicateEdge(key));
            }
            if edge.from == edge.to {
                return Err(WorkflowSpecEmitError::SelfLoop(key));
            }
            if !node_ids.contains(&edge.from) {
                return Err(WorkflowSpecEmitError::DanglingEdgeSource(key));
            }
            if !node_ids.contains(&edge.to) {
                return Err(WorkflowSpecEmitError::DanglingEdgeTarget(key));
            }
        }

        // --- graph-integrity checks ---
        // Build forward adjacency list and in-degree map (BTreeMap for determinism).
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
        for node in &self.nodes {
            adjacency.entry(node.id.as_str()).or_default();
            in_degree.entry(node.id.as_str()).or_insert(0);
        }
        for edge in &self.edges {
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        }

        // Entry nodes: all nodes with in-degree 0 (no incoming edges).
        let entry_nodes: BTreeSet<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        // Unreachable-node detection (runs first so nodes only reachable via a cycle
        // are reported as UnreachableNode rather than GraphCycle).
        // BFS forward from all entry nodes; any unvisited node is unreachable.
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut frontier: Vec<&str> = entry_nodes.iter().copied().collect();
        while let Some(current) = frontier.pop() {
            if visited.insert(current) {
                if let Some(neighbours) = adjacency.get(current) {
                    for &neighbour in neighbours {
                        if !visited.contains(neighbour) {
                            frontier.push(neighbour);
                        }
                    }
                }
            }
        }
        if let Some(first_unreachable) = node_ids
            .iter()
            .find(|id| !visited.contains(id.as_str()))
        {
            return Err(WorkflowSpecEmitError::UnreachableNode(
                first_unreachable.clone(),
            ));
        }

        // Cycle detection via Kahn's BFS (topological sort).
        // If any node remains unprocessed after BFS, the graph contains a cycle.
        let mut queue: BTreeSet<&str> = entry_nodes;
        let mut processed: usize = 0;
        let mut in_degree_work = in_degree.clone();
        while let Some(&current) = queue.iter().next() {
            queue.remove(current);
            processed += 1;
            if let Some(neighbours) = adjacency.get(current) {
                for &neighbour in neighbours {
                    let deg = in_degree_work.entry(neighbour).or_insert(0);
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.insert(neighbour);
                    }
                }
            }
        }
        if processed < self.nodes.len() {
            // First unprocessed node by sorted ID is the reported cycle participant.
            let first_cycle_node = in_degree_work
                .iter()
                .find(|&(_, &deg)| deg > 0)
                .map(|(&id, _)| id)
                .unwrap_or("");
            return Err(WorkflowSpecEmitError::GraphCycle(
                first_cycle_node.to_string(),
            ));
        }

        Ok(())
    }

    pub fn canonicalized(&self) -> Result<Self, WorkflowSpecEmitError> {
        self.validate()?;
        let mut nodes = self.nodes.clone();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));
        let mut edges = self.edges.clone();
        edges.sort_by(|left, right| {
            (
                left.from.as_str(),
                left.to.as_str(),
                left.condition.as_deref(),
            )
                .cmp(&(
                    right.from.as_str(),
                    right.to.as_str(),
                    right.condition.as_deref(),
                ))
        });
        Ok(Self {
            schema_version: WORKFLOW_SPEC_SCHEMA_VERSION.to_string(),
            tenant_id: self.tenant_id.clone(),
            definition_id: self.definition_id.clone(),
            version: self.version.clone(),
            nodes,
            edges,
        })
    }
}

pub fn emit_canonical_json(spec: &WorkflowSpec) -> Result<String, WorkflowSpecEmitError> {
    serde_json::to_string(&spec.canonicalized()?).map_err(WorkflowSpecEmitError::from)
}

fn edge_key(edge: &WorkflowSpecEdge) -> String {
    format!(
        "{}->{} [{}]",
        edge.from,
        edge.to,
        edge.condition.as_deref().unwrap_or("")
    )
}

fn is_prefixed_nonempty(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix) && value.len() > prefix.len()
}

fn is_semver(value: &str) -> bool {
    let core = value
        .split_once('-')
        .map_or(value, |(candidate, _suffix)| candidate);
    let mut parts = core.split('.');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(major), Some(minor), Some(patch), None) => {
            is_ascii_digits(major) && is_ascii_digits(minor) && is_ascii_digits(patch)
        }
        _ => false,
    }
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workflow_spec() -> WorkflowSpec {
        WorkflowSpec::new(
            "ten_acme",
            "wfd_onboarding",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_transform", WorkflowSpecNodeKind::Transform, "Prepare"),
                WorkflowSpecNode::new("wfn_start", WorkflowSpecNodeKind::Http, "Start"),
                WorkflowSpecNode::new("wfn_review", WorkflowSpecNodeKind::HumanReview, "Approve"),
            ],
            vec![
                WorkflowSpecEdge::new("wfn_transform", "wfn_review", None),
                WorkflowSpecEdge::new("wfn_start", "wfn_transform", Some("ok".to_string())),
            ],
        )
    }

    #[test]
    fn emits_canonical_workflow_spec_v1_json_without_null_conditions() {
        let emitted = emit_canonical_json(&workflow_spec()).unwrap();

        assert!(emitted.contains("\"schema_version\":\"workflow_spec.v1\""));
        assert!(emitted.contains("\"id\":\"wfn_review\""));
        assert!(emitted.contains("\"condition\":\"ok\""));
        assert!(!emitted.contains("null"));
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let spec = WorkflowSpec::new(
            "ten_acme",
            "wfd_bad",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_one", WorkflowSpecNodeKind::Http, "One"),
                WorkflowSpecNode::new("wfn_one", WorkflowSpecNodeKind::Transform, "Two"),
            ],
            Vec::new(),
        );

        assert_eq!(
            spec.validate(),
            Err(WorkflowSpecEmitError::DuplicateNodeId(
                "wfn_one".to_string()
            ))
        );
    }

    #[test]
    fn validate_clean_dag_passes() {
        // 3-node linear chain: wfn_a -> wfn_b -> wfn_c
        let spec = WorkflowSpec::new(
            "ten_acme",
            "wfd_chain",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
                WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
                WorkflowSpecNode::new("wfn_c", WorkflowSpecNodeKind::Join, "C"),
            ],
            vec![
                WorkflowSpecEdge::new("wfn_a", "wfn_b", None),
                WorkflowSpecEdge::new("wfn_b", "wfn_c", None),
            ],
        );
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validate_cyclic_graph_returns_graph_cycle() {
        // wfn_a (entry, in-degree 0) -> wfn_b -> wfn_c -> wfn_b (back-edge).
        // All nodes reachable from wfn_a, so unreachable check passes.
        // wfn_b and wfn_c form a cycle, so Kahn's BFS processes only wfn_a (1 < 3).
        let spec = WorkflowSpec::new(
            "ten_acme",
            "wfd_cycle",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
                WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
                WorkflowSpecNode::new("wfn_c", WorkflowSpecNodeKind::Join, "C"),
            ],
            vec![
                WorkflowSpecEdge::new("wfn_a", "wfn_b", None),
                WorkflowSpecEdge::new("wfn_b", "wfn_c", None),
                WorkflowSpecEdge::new("wfn_c", "wfn_b", None),
            ],
        );
        let result = spec.validate();
        assert!(
            matches!(result, Err(WorkflowSpecEmitError::GraphCycle(_))),
            "expected GraphCycle, got {result:?}",
        );
    }

    #[test]
    fn validate_unreachable_node_returns_unreachable_node() {
        // wfn_a is the sole entry node (in-degree 0, no outgoing edges).
        // wfn_b and wfn_c form a 2-cycle: both have in-degree > 0, no path from wfn_a.
        // Unreachable check (runs before cycle check) fires on wfn_b (first sorted).
        let spec = WorkflowSpec::new(
            "ten_acme",
            "wfd_island",
            "1.0.0",
            vec![
                WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
                WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
                WorkflowSpecNode::new("wfn_c", WorkflowSpecNodeKind::Join, "C"),
            ],
            vec![
                WorkflowSpecEdge::new("wfn_b", "wfn_c", None),
                WorkflowSpecEdge::new("wfn_c", "wfn_b", None),
            ],
        );
        assert_eq!(
            spec.validate(),
            Err(WorkflowSpecEmitError::UnreachableNode("wfn_b".to_string())),
        );
    }

    #[test]
    fn validate_is_deterministic() {
        let spec = workflow_spec();
        let result1 = spec.validate();
        let result2 = spec.validate();
        assert_eq!(result1, result2);
    }

    #[test]
    fn rejects_dangling_edges() {
        let spec = WorkflowSpec::new(
            "ten_acme",
            "wfd_bad",
            "1.0.0",
            vec![WorkflowSpecNode::new(
                "wfn_start",
                WorkflowSpecNodeKind::Http,
                "Start",
            )],
            vec![WorkflowSpecEdge::new("wfn_start", "wfn_missing", None)],
        );

        assert_eq!(
            spec.validate(),
            Err(WorkflowSpecEmitError::DanglingEdgeTarget(
                "wfn_start->wfn_missing []".to_string()
            ))
        );
    }
}
