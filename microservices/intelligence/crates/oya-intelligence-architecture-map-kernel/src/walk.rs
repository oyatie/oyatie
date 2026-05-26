//! Source walkers for the architecture-map kernel (M01-P16-IP-001).
//!
//! Each `walk_*` function takes a slice of **pre-parsed** typed records
//! and produces a partial `ArchitectureMap` covering one source layer.
//! Runners do the disk I/O + YAML/JSON parsing; the kernel stays
//! I/O-free so it can run in tests, CI, and the freshness lane without
//! any filesystem access.
//!
//! Maps from different walkers compose via `ArchitectureMap::merge`
//! (added in this slice).

use crate::{ArchitectureMap, Edge, EdgeKind, MapError, Node, NodeId, NodeKind};

/// Pre-parsed Cargo workspace metadata row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoPackage {
    pub name: String,                // data_class: INTERNAL_ONLY
    pub owning_team: Option<String>, // data_class: INTERNAL_ONLY
    pub dependencies: Vec<String>,   // data_class: INTERNAL_ONLY (workspace names only)
}

/// Pre-parsed OpenAPI contract row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiContractMeta {
    pub contract_path: String,           // data_class: INTERNAL_ONLY
    pub owning_bc_id: String,            // data_class: INTERNAL_ONLY
    pub cedar_fragment_ids: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Pre-parsed plan/frontmatter row (microservice or bounded context).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontmatterRecord {
    pub id: String,                  // data_class: INTERNAL_ONLY
    pub kind: FrontmatterKind,       // data_class: INTERNAL_ONLY
    pub label: String,               // data_class: INTERNAL_ONLY
    pub owning_team: Option<String>, // data_class: INTERNAL_ONLY
    /// Parent microservice id for BoundedContext rows; ignored otherwise.
    pub parent_microservice: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontmatterKind {
    Microservice,
    BoundedContext,
    Lane,
    CedarFragment,
}

impl FrontmatterKind {
    fn to_node_kind(self) -> NodeKind {
        match self {
            Self::Microservice => NodeKind::Microservice,
            Self::BoundedContext => NodeKind::BoundedContext,
            Self::Lane => NodeKind::Lane,
            Self::CedarFragment => NodeKind::CedarFragment,
        }
    }
}

impl ArchitectureMap {
    /// Merge another map into this one. Duplicate node ids surface as
    /// `MapError::DuplicateNode`; duplicate edges are accepted (edges
    /// are a multiset).
    pub fn merge(&mut self, other: ArchitectureMap) -> Result<(), MapError> {
        for node in other.nodes() {
            // We cannot reuse `add_node(node.clone())` here because
            // duplicate detection is "any existing entry", and a strict
            // policy makes idempotent re-walks expensive. Allow exact
            // duplicates (same id + same kind + same label).
            if let Some(existing) = self.node(&node.id) {
                if existing != node {
                    return Err(MapError::DuplicateNode {
                        id: node.id.clone(),
                    });
                }
                continue;
            }
            self.add_node(node.clone())?;
        }
        for edge in other.edges() {
            // Both endpoints exist by construction (other had them, and
            // we merged the node set above) — but defend anyway: skip
            // dangling refs after node-level deduplication.
            if self.node(&edge.source).is_some() && self.node(&edge.target).is_some() {
                self.add_edge(edge.clone())?;
            }
        }
        Ok(())
    }
}

/// Walk Cargo workspace packages into a partial map: one `Crate` node
/// per package, plus `DependsOn` edges between in-workspace packages.
pub fn walk_cargo_metadata(packages: &[CargoPackage]) -> Result<ArchitectureMap, MapError> {
    let mut map = ArchitectureMap::new();
    let mut workspace_names = std::collections::BTreeSet::new();
    for p in packages {
        if p.name.is_empty() {
            return Err(MapError::DuplicateNode {
                id: NodeId(String::new()),
            });
        }
        workspace_names.insert(p.name.as_str());
    }
    for p in packages {
        map.add_node(Node {
            id: NodeId(p.name.clone()),
            kind: NodeKind::Crate,
            label: p.name.clone(),
            owning_team: p.owning_team.clone(),
        })?;
    }
    for p in packages {
        for dep in &p.dependencies {
            if workspace_names.contains(dep.as_str()) && dep != &p.name {
                map.add_edge(Edge {
                    source: NodeId(p.name.clone()),
                    target: NodeId(dep.clone()),
                    kind: EdgeKind::DependsOn,
                })?;
            }
        }
    }
    Ok(map)
}

/// Walk OpenAPI contracts into a partial map: one `OpenApiContract`
/// node per contract path, plus `Exposes` edges from the owning BC
/// (which the caller must have already added via `walk_frontmatter`)
/// and `Governs` edges from any Cedar fragments named in the contract.
///
/// Nodes for the BC and Cedar fragments are **not** synthesized here;
/// the caller composes the maps via `ArchitectureMap::merge`. If a
/// referenced BC/fragment is absent at merge time, the `Exposes` /
/// `Governs` edge is dropped (see `merge` implementation).
pub fn walk_openapi(contracts: &[OpenApiContractMeta]) -> Result<ArchitectureMap, MapError> {
    let mut map = ArchitectureMap::new();
    for c in contracts {
        let contract_id = NodeId(c.contract_path.clone());
        map.add_node(Node {
            id: contract_id.clone(),
            kind: NodeKind::OpenApiContract,
            label: c.contract_path.clone(),
            owning_team: None,
        })?;
        // Owning BC + Cedar fragments are placeholder nodes; we add
        // them so the edges have valid endpoints inside *this* partial
        // map. On merge with the frontmatter walker, exact-duplicate
        // nodes collapse cleanly.
        let bc_id = NodeId(c.owning_bc_id.clone());
        if map.node(&bc_id).is_none() {
            map.add_node(Node {
                id: bc_id.clone(),
                kind: NodeKind::BoundedContext,
                label: c.owning_bc_id.clone(),
                owning_team: None,
            })?;
        }
        map.add_edge(Edge {
            source: bc_id,
            target: contract_id.clone(),
            kind: EdgeKind::Exposes,
        })?;
        for frag in &c.cedar_fragment_ids {
            let frag_id = NodeId(frag.clone());
            if map.node(&frag_id).is_none() {
                map.add_node(Node {
                    id: frag_id.clone(),
                    kind: NodeKind::CedarFragment,
                    label: frag.clone(),
                    owning_team: None,
                })?;
            }
            map.add_edge(Edge {
                source: frag_id,
                target: contract_id.clone(),
                kind: EdgeKind::Governs,
            })?;
        }
    }
    Ok(map)
}

/// Walk pre-parsed frontmatter records into a partial map. Microservice
/// rows produce `Microservice` nodes; BoundedContext rows produce
/// `BoundedContext` nodes plus a `Contains` edge from the parent
/// microservice when `parent_microservice` is set. Lane and
/// CedarFragment rows produce their respective nodes.
pub fn walk_frontmatter(records: &[FrontmatterRecord]) -> Result<ArchitectureMap, MapError> {
    let mut map = ArchitectureMap::new();
    let mut ids = std::collections::BTreeSet::new();
    for r in records {
        if r.id.is_empty() {
            return Err(MapError::DuplicateNode {
                id: NodeId(String::new()),
            });
        }
        ids.insert(r.id.as_str());
    }
    for r in records {
        map.add_node(Node {
            id: NodeId(r.id.clone()),
            kind: r.kind.to_node_kind(),
            label: r.label.clone(),
            owning_team: r.owning_team.clone(),
        })?;
    }
    for r in records {
        if matches!(r.kind, FrontmatterKind::BoundedContext)
            && let Some(parent) = &r.parent_microservice
            && ids.contains(parent.as_str())
        {
            map.add_edge(Edge {
                source: NodeId(parent.clone()),
                target: NodeId(r.id.clone()),
                kind: EdgeKind::Contains,
            })?;
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    // walk_cargo_metadata

    #[test]
    fn cargo_walker_adds_crate_node_per_package() {
        let pkgs = vec![
            CargoPackage {
                name: "oya-a".into(),
                owning_team: None,
                dependencies: vec![],
            },
            CargoPackage {
                name: "oya-b".into(),
                owning_team: None,
                dependencies: vec![],
            },
        ];
        let m = walk_cargo_metadata(&pkgs).unwrap();
        assert_eq!(m.node_count(), 2);
    }

    #[test]
    fn cargo_walker_creates_dep_edges() {
        let pkgs = vec![
            CargoPackage {
                name: "oya-a".into(),
                owning_team: None,
                dependencies: vec!["oya-b".into()],
            },
            CargoPackage {
                name: "oya-b".into(),
                owning_team: None,
                dependencies: vec![],
            },
        ];
        let m = walk_cargo_metadata(&pkgs).unwrap();
        assert_eq!(m.edge_count(), 1);
    }

    #[test]
    fn cargo_walker_drops_external_deps() {
        let pkgs = vec![CargoPackage {
            name: "oya-a".into(),
            owning_team: None,
            dependencies: vec!["serde".into(), "tokio".into()],
        }];
        let m = walk_cargo_metadata(&pkgs).unwrap();
        assert_eq!(m.edge_count(), 0);
    }

    #[test]
    fn cargo_walker_skips_self_loop() {
        let pkgs = vec![CargoPackage {
            name: "oya-a".into(),
            owning_team: None,
            dependencies: vec!["oya-a".into()],
        }];
        let m = walk_cargo_metadata(&pkgs).unwrap();
        assert_eq!(m.edge_count(), 0);
    }

    // walk_openapi

    #[test]
    fn openapi_walker_emits_contract_and_exposes_edge() {
        let m = walk_openapi(&[OpenApiContractMeta {
            contract_path: "contracts/api.yaml".into(),
            owning_bc_id: "ops/docs-portal".into(),
            cedar_fragment_ids: vec![],
        }])
        .unwrap();
        assert_eq!(m.node_count(), 2); // contract + placeholder BC
        assert_eq!(m.edge_count(), 1);
    }

    #[test]
    fn openapi_walker_emits_governs_edge_per_fragment() {
        let m = walk_openapi(&[OpenApiContractMeta {
            contract_path: "contracts/api.yaml".into(),
            owning_bc_id: "ops/docs-portal".into(),
            cedar_fragment_ids: vec!["ops-internal-public".into(), "ops-admin".into()],
        }])
        .unwrap();
        // 1 contract + 1 BC + 2 fragments = 4 nodes
        assert_eq!(m.node_count(), 4);
        // 1 exposes + 2 governs = 3 edges
        assert_eq!(m.edge_count(), 3);
    }

    // walk_frontmatter

    #[test]
    fn frontmatter_walker_adds_microservice_node() {
        let m = walk_frontmatter(&[FrontmatterRecord {
            id: "ops".into(),
            kind: FrontmatterKind::Microservice,
            label: "Ops".into(),
            owning_team: Some("sre".into()),
            parent_microservice: None,
        }])
        .unwrap();
        assert_eq!(m.node_count(), 1);
        assert_eq!(m.edge_count(), 0);
    }

    #[test]
    fn frontmatter_walker_contains_bc_under_microservice() {
        let m = walk_frontmatter(&[
            FrontmatterRecord {
                id: "ops".into(),
                kind: FrontmatterKind::Microservice,
                label: "Ops".into(),
                owning_team: None,
                parent_microservice: None,
            },
            FrontmatterRecord {
                id: "ops/docs-portal".into(),
                kind: FrontmatterKind::BoundedContext,
                label: "Docs Portal".into(),
                owning_team: None,
                parent_microservice: Some("ops".into()),
            },
        ])
        .unwrap();
        assert_eq!(m.node_count(), 2);
        assert_eq!(m.edge_count(), 1);
    }

    #[test]
    fn frontmatter_walker_drops_dangling_parent_pointer() {
        let m = walk_frontmatter(&[FrontmatterRecord {
            id: "ops/orphan".into(),
            kind: FrontmatterKind::BoundedContext,
            label: "Orphan".into(),
            owning_team: None,
            parent_microservice: Some("does-not-exist".into()),
        }])
        .unwrap();
        // BC node exists; no edge because parent isn't in this batch.
        assert_eq!(m.node_count(), 1);
        assert_eq!(m.edge_count(), 0);
    }

    #[test]
    fn frontmatter_walker_supports_lane_and_fragment() {
        let m = walk_frontmatter(&[
            FrontmatterRecord {
                id: "lane:provider-coupling".into(),
                kind: FrontmatterKind::Lane,
                label: "Provider coupling".into(),
                owning_team: None,
                parent_microservice: None,
            },
            FrontmatterRecord {
                id: "ops-internal-public".into(),
                kind: FrontmatterKind::CedarFragment,
                label: "ops-internal-public".into(),
                owning_team: None,
                parent_microservice: None,
            },
        ])
        .unwrap();
        assert_eq!(m.node_count(), 2);
    }

    // merge

    #[test]
    fn merge_combines_disjoint_maps() {
        let a = walk_cargo_metadata(&[CargoPackage {
            name: "oya-a".into(),
            owning_team: None,
            dependencies: vec![],
        }])
        .unwrap();
        let b = walk_cargo_metadata(&[CargoPackage {
            name: "oya-b".into(),
            owning_team: None,
            dependencies: vec![],
        }])
        .unwrap();
        let mut combined = a;
        combined.merge(b).unwrap();
        assert_eq!(combined.node_count(), 2);
    }

    #[test]
    fn merge_accepts_exact_duplicate_nodes() {
        let a = walk_frontmatter(&[FrontmatterRecord {
            id: "ops".into(),
            kind: FrontmatterKind::Microservice,
            label: "Ops".into(),
            owning_team: None,
            parent_microservice: None,
        }])
        .unwrap();
        let b = walk_frontmatter(&[FrontmatterRecord {
            id: "ops".into(),
            kind: FrontmatterKind::Microservice,
            label: "Ops".into(),
            owning_team: None,
            parent_microservice: None,
        }])
        .unwrap();
        let mut combined = a;
        combined.merge(b).unwrap();
        assert_eq!(combined.node_count(), 1);
    }

    #[test]
    fn merge_rejects_conflicting_node() {
        let a = walk_frontmatter(&[FrontmatterRecord {
            id: "ops".into(),
            kind: FrontmatterKind::Microservice,
            label: "Ops".into(),
            owning_team: None,
            parent_microservice: None,
        }])
        .unwrap();
        let b = walk_frontmatter(&[FrontmatterRecord {
            id: "ops".into(),
            kind: FrontmatterKind::Microservice,
            label: "Different label".into(),
            owning_team: None,
            parent_microservice: None,
        }])
        .unwrap();
        let mut combined = a;
        assert!(matches!(
            combined.merge(b),
            Err(MapError::DuplicateNode { .. })
        ));
    }

    #[test]
    fn three_walker_compose_produces_full_subgraph() {
        // Frontmatter walker provides BC + microservice.
        let fm = walk_frontmatter(&[
            FrontmatterRecord {
                id: "ops".into(),
                kind: FrontmatterKind::Microservice,
                label: "Ops".into(),
                owning_team: None,
                parent_microservice: None,
            },
            FrontmatterRecord {
                id: "ops/docs-portal".into(),
                kind: FrontmatterKind::BoundedContext,
                // Label matches the id; the OpenAPI walker generates a
                // placeholder BC with `label = bc_id`, so callers that
                // intend to compose across walkers must use the same
                // convention (or extend the merge step to reconcile).
                label: "ops/docs-portal".into(),
                owning_team: None,
                parent_microservice: Some("ops".into()),
            },
        ])
        .unwrap();
        // OpenAPI walker provides contract + exposes from the same BC.
        let oa = walk_openapi(&[OpenApiContractMeta {
            contract_path: "contracts/ops-docs.yaml".into(),
            owning_bc_id: "ops/docs-portal".into(),
            cedar_fragment_ids: vec![],
        }])
        .unwrap();
        // Cargo walker provides crate nodes (no edges across BC level).
        let cg = walk_cargo_metadata(&[CargoPackage {
            name: "oya-ops-docs-portal-kernel".into(),
            owning_team: None,
            dependencies: vec![],
        }])
        .unwrap();
        let mut all = fm;
        all.merge(oa).unwrap();
        all.merge(cg).unwrap();
        // 1 microservice + 1 BC + 1 contract + 1 crate = 4 nodes
        assert_eq!(all.node_count(), 4);
        // Contains(ops -> ops/docs-portal) + Exposes(ops/docs-portal -> contract)
        assert_eq!(all.edge_count(), 2);
    }
}
