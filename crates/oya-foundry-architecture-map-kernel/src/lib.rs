//! Foundry architecture-map kernel — pure graph model for the
//! visualization-as-code directive (2026-05-12).
//!
//! Walks Cargo workspace + OpenAPI contracts + capability-registry + cedar-
//! fragments + microservices/bounded-contexts registries to produce a
//! machine-readable architecture map. The map is graphed and visible per the
//! directive; freshness lane `oya-foundry-fitness-architecture-map-freshness`
//! enforces that the on-disk map matches the live workspace state.
//!
//! This crate is the kernel: pure std-only, takes parsed inputs as data,
//! returns the graph. The runtime `-app` crate (separate slice) walks the
//! filesystem + emits JSON.
//!
//! Node types: Microservice | BoundedContext | Crate | OpenApiContract |
//!             CedarFragment | Lane
//! Edge types: contains | exposes | governs | depends-on | enforces
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

pub mod emit;
pub mod plane;
pub mod walk;

pub use plane::{ArchitecturePlane, PlaneVerdict, ProofLevel, UnknownPlane};

/// Node kind taxonomy. Each variant maps to one row class in the
/// underlying registries:
///   Microservice ↔ registry/microservices.json
///   BoundedContext ↔ registry/bounded-contexts.json
///   Crate ↔ workspace members in root Cargo.toml
///   OpenApiContract ↔ contracts/*.openapi.yaml
///   CedarFragment ↔ registry/cedar-fragments.json
///   Lane ↔ registry/quality/lanes.yaml
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NodeKind {
    Microservice,
    BoundedContext,
    Crate,
    OpenApiContract,
    CedarFragment,
    Lane,
}

impl NodeKind {
    pub fn name(self) -> &'static str {
        match self {
            NodeKind::Microservice => "microservice",
            NodeKind::BoundedContext => "bounded-context",
            NodeKind::Crate => "crate",
            NodeKind::OpenApiContract => "openapi-contract",
            NodeKind::CedarFragment => "cedar-fragment",
            NodeKind::Lane => "lane",
        }
    }
}

/// Edge kind taxonomy.
///   Contains       Microservice → BoundedContext
///   Exposes        BoundedContext → OpenApiContract
///   Governs        CedarFragment → OpenApiContract (route-level Cedar gate)
///   DependsOn      Crate → Crate (Cargo dependency)
///   Enforces       Lane → Node (the lane's check protects that node from drift)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EdgeKind {
    Contains,
    Exposes,
    Governs,
    DependsOn,
    Enforces,
}

impl EdgeKind {
    pub fn name(self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::Exposes => "exposes",
            EdgeKind::Governs => "governs",
            EdgeKind::DependsOn => "depends-on",
            EdgeKind::Enforces => "enforces",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub id: NodeId,                  // data_class: INTERNAL_ONLY
    pub kind: NodeKind,              // data_class: INTERNAL_ONLY
    pub label: String,               // data_class: INTERNAL_ONLY (display name)
    pub owning_team: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edge {
    pub source: NodeId, // data_class: INTERNAL_ONLY
    pub target: NodeId, // data_class: INTERNAL_ONLY
    pub kind: EdgeKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArchitectureMap {
    nodes: BTreeMap<NodeId, Node>,
    edges: Vec<Edge>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
    DuplicateNode { id: NodeId },
    UnknownSource { id: NodeId },
    UnknownTarget { id: NodeId },
}

impl ArchitectureMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) -> Result<(), MapError> {
        if self.nodes.contains_key(&node.id) {
            return Err(MapError::DuplicateNode { id: node.id });
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn add_edge(&mut self, edge: Edge) -> Result<(), MapError> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(MapError::UnknownSource {
                id: edge.source.clone(),
            });
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(MapError::UnknownTarget {
                id: edge.target.clone(),
            });
        }
        self.edges.push(edge);
        Ok(())
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes_of_kind(&self, kind: NodeKind) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(move |n| n.kind == kind)
    }

    /// Return all edges whose source is `id`.
    pub fn outgoing<'a>(&'a self, id: &'a NodeId) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| &e.source == id)
    }

    /// Return all edges whose target is `id`.
    pub fn incoming<'a>(&'a self, id: &'a NodeId) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| &e.target == id)
    }

    /// Find nodes that have zero incoming + outgoing edges. Useful for the
    /// freshness lane: orphaned nodes likely indicate stale registries.
    pub fn orphans(&self) -> Vec<&Node> {
        let mut connected: BTreeSet<&NodeId> = BTreeSet::new();
        for edge in &self.edges {
            connected.insert(&edge.source);
            connected.insert(&edge.target);
        }
        self.nodes
            .values()
            .filter(|n| !connected.contains(&n.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, kind: NodeKind) -> Node {
        Node {
            id: NodeId(id.into()),
            kind,
            label: id.into(),
            owning_team: None,
        }
    }

    #[test]
    fn node_kind_names_round_trip() {
        let kinds = [
            (NodeKind::Microservice, "microservice"),
            (NodeKind::BoundedContext, "bounded-context"),
            (NodeKind::Crate, "crate"),
            (NodeKind::OpenApiContract, "openapi-contract"),
            (NodeKind::CedarFragment, "cedar-fragment"),
            (NodeKind::Lane, "lane"),
        ];
        for (k, n) in kinds {
            assert_eq!(k.name(), n);
        }
    }

    #[test]
    fn edge_kind_names_round_trip() {
        let kinds = [
            (EdgeKind::Contains, "contains"),
            (EdgeKind::Exposes, "exposes"),
            (EdgeKind::Governs, "governs"),
            (EdgeKind::DependsOn, "depends-on"),
            (EdgeKind::Enforces, "enforces"),
        ];
        for (k, n) in kinds {
            assert_eq!(k.name(), n);
        }
    }

    #[test]
    fn add_node_and_count() {
        let mut map = ArchitectureMap::new();
        map.add_node(node("ops", NodeKind::Microservice)).unwrap();
        assert_eq!(map.node_count(), 1);
    }

    #[test]
    fn add_duplicate_node_errors() {
        let mut map = ArchitectureMap::new();
        map.add_node(node("ops", NodeKind::Microservice)).unwrap();
        let result = map.add_node(node("ops", NodeKind::Microservice));
        assert!(matches!(result, Err(MapError::DuplicateNode { .. })));
    }

    #[test]
    fn add_edge_requires_existing_endpoints() {
        let mut map = ArchitectureMap::new();
        map.add_node(node("ops", NodeKind::Microservice)).unwrap();
        let result = map.add_edge(Edge {
            source: NodeId("ops".into()),
            target: NodeId("ops/docs-portal".into()),
            kind: EdgeKind::Contains,
        });
        assert!(matches!(result, Err(MapError::UnknownTarget { .. })));
    }

    #[test]
    fn add_edge_unknown_source_errors() {
        let mut map = ArchitectureMap::new();
        map.add_node(node("ops/docs-portal", NodeKind::BoundedContext))
            .unwrap();
        let result = map.add_edge(Edge {
            source: NodeId("ops".into()),
            target: NodeId("ops/docs-portal".into()),
            kind: EdgeKind::Contains,
        });
        assert!(matches!(result, Err(MapError::UnknownSource { .. })));
    }

    fn populated() -> ArchitectureMap {
        let mut map = ArchitectureMap::new();
        map.add_node(node("ops", NodeKind::Microservice)).unwrap();
        map.add_node(node("ops/docs-portal", NodeKind::BoundedContext))
            .unwrap();
        map.add_node(node("ops/workspace", NodeKind::BoundedContext))
            .unwrap();
        map.add_node(node(
            "contracts/ops-docs-v1.openapi.yaml",
            NodeKind::OpenApiContract,
        ))
        .unwrap();
        map.add_node(node("ops-internal-public", NodeKind::CedarFragment))
            .unwrap();
        map.add_edge(Edge {
            source: NodeId("ops".into()),
            target: NodeId("ops/docs-portal".into()),
            kind: EdgeKind::Contains,
        })
        .unwrap();
        map.add_edge(Edge {
            source: NodeId("ops".into()),
            target: NodeId("ops/workspace".into()),
            kind: EdgeKind::Contains,
        })
        .unwrap();
        map.add_edge(Edge {
            source: NodeId("ops/docs-portal".into()),
            target: NodeId("contracts/ops-docs-v1.openapi.yaml".into()),
            kind: EdgeKind::Exposes,
        })
        .unwrap();
        map.add_edge(Edge {
            source: NodeId("ops-internal-public".into()),
            target: NodeId("contracts/ops-docs-v1.openapi.yaml".into()),
            kind: EdgeKind::Governs,
        })
        .unwrap();
        map
    }

    #[test]
    fn outgoing_from_microservice() {
        let map = populated();
        let ops = NodeId("ops".into());
        let edges: Vec<&Edge> = map.outgoing(&ops).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|e| e.kind == EdgeKind::Contains));
    }

    #[test]
    fn incoming_to_openapi_contract() {
        let map = populated();
        let contract = NodeId("contracts/ops-docs-v1.openapi.yaml".into());
        let edges: Vec<&Edge> = map.incoming(&contract).collect();
        assert_eq!(edges.len(), 2);
        let kinds: BTreeSet<_> = edges.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&EdgeKind::Exposes));
        assert!(kinds.contains(&EdgeKind::Governs));
    }

    #[test]
    fn nodes_of_kind_filter() {
        let map = populated();
        let bcs: Vec<&Node> = map.nodes_of_kind(NodeKind::BoundedContext).collect();
        assert_eq!(bcs.len(), 2);
    }

    #[test]
    fn orphans_finds_disconnected_nodes() {
        let mut map = populated();
        map.add_node(node("lonely-fragment", NodeKind::CedarFragment))
            .unwrap();
        let orphans = map.orphans();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].id, NodeId("lonely-fragment".into()));
    }

    #[test]
    fn orphans_empty_when_all_connected() {
        let map = populated();
        let orphans = map.orphans();
        assert!(orphans.is_empty());
    }

    #[test]
    fn edge_count_matches_inserted() {
        let map = populated();
        assert_eq!(map.edge_count(), 4);
    }
}
