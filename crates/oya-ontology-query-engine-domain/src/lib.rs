//! Ontology query-engine domain foundation.
//!
//! This crate implements the source-level, in-memory query semantics for the
//! preview Knowledge Graph contract. It intentionally stays adapter-free: cloud
//! storage, query languages, distributed execution, authz enforcement, and SLO
//! runtime evidence are future slices. The implemented semantics are bounded,
//! tenant-scoped, deterministic outbound traversal over validated link
//! instances.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oya_ontology_kernel::ObjectGraph;

/// Hard cap for source-level recursive traversal in this preview foundation.
pub const MAX_QUERY_DEPTH: u32 = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeGraphLinkInstance {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub from_entity_id: String,         // data_class: INTERNAL_ONLY
    pub to_entity_id: String,           // data_class: INTERNAL_ONLY
    pub edge_type_id: String,           // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeGraphQueryRequest {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub query_id: String,                   // data_class: INTERNAL_ONLY
    pub root_entity_id: String,             // data_class: INTERNAL_ONLY
    pub edge_type_ids: Vec<String>,         // data_class: INTERNAL_ONLY
    pub max_depth: u32,                     // data_class: INTERNAL_ONLY
    pub freshness_floor_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeGraphQueryResponse {
    pub query_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub nodes: Vec<KnowledgeGraphNode>, // data_class: INTERNAL_ONLY
    pub edges: Vec<KnowledgeGraphEdge>, // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
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
    InvalidMaxDepth,
    MissingRootEntity,
    DanglingLinkEndpoint { entity_id: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KnowledgeGraphQueryEngine {
    links: BTreeMap<KnowledgeGraphLinkKey, KnowledgeGraphLinkInstance>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct KnowledgeGraphLinkKey {
    tenant_id: String,
    from_entity_id: String,
    edge_type_id: String,
    to_entity_id: String,
}

impl KnowledgeGraphLinkInstance {
    pub fn new(
        tenant_id: impl Into<String>,
        from_entity_id: impl Into<String>,
        to_entity_id: impl Into<String>,
        edge_type_id: impl Into<String>,
        observed_at_epoch_seconds: u64,
    ) -> Result<Self, KnowledgeGraphQueryError> {
        let link = Self {
            tenant_id: tenant_id.into(),
            from_entity_id: from_entity_id.into(),
            to_entity_id: to_entity_id.into(),
            edge_type_id: edge_type_id.into(),
            observed_at_epoch_seconds,
        };
        validate_tenant_id(&link.tenant_id)?;
        validate_entity_id(&link.from_entity_id)?;
        validate_entity_id(&link.to_entity_id)?;
        validate_edge_type_id(&link.edge_type_id)?;
        Ok(link)
    }

    pub fn as_contract_edge(&self) -> KnowledgeGraphEdge {
        KnowledgeGraphEdge {
            from_entity_id: self.from_entity_id.clone(),
            to_entity_id: self.to_entity_id.clone(),
            edge_type_id: self.edge_type_id.clone(),
        }
    }
}

impl KnowledgeGraphQueryRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: impl Into<String>,
        query_id: impl Into<String>,
        root_entity_id: impl Into<String>,
        edge_type_ids: Vec<impl Into<String>>,
        max_depth: u32,
        freshness_floor_epoch_seconds: u64,
        observed_at_epoch_seconds: u64,
    ) -> Result<Self, KnowledgeGraphQueryError> {
        let request = Self {
            tenant_id: tenant_id.into(),
            query_id: query_id.into(),
            root_entity_id: root_entity_id.into(),
            edge_type_ids: edge_type_ids.into_iter().map(Into::into).collect(),
            max_depth,
            freshness_floor_epoch_seconds,
            observed_at_epoch_seconds,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), KnowledgeGraphQueryError> {
        validate_tenant_id(&self.tenant_id)?;
        validate_query_id(&self.query_id)?;
        validate_entity_id(&self.root_entity_id)?;
        validate_max_depth(self.max_depth)?;
        for edge_type_id in &self.edge_type_ids {
            validate_edge_type_id(edge_type_id)?;
        }
        Ok(())
    }

    fn edge_filter(&self) -> BTreeSet<&str> {
        self.edge_type_ids.iter().map(String::as_str).collect()
    }
}

impl KnowledgeGraphQueryEngine {
    pub fn upsert_link(
        &mut self,
        graph: &ObjectGraph,
        link: KnowledgeGraphLinkInstance,
    ) -> Result<KnowledgeGraphLinkUpsertOutcome, KnowledgeGraphQueryError> {
        validate_link_endpoints(graph, &link)?;
        let key = link.key();
        let outcome = if self.links.insert(key, link).is_some() {
            KnowledgeGraphLinkUpsertOutcome::Updated
        } else {
            KnowledgeGraphLinkUpsertOutcome::Inserted
        };
        Ok(outcome)
    }

    pub fn query_graph_slice(
        &self,
        graph: &ObjectGraph,
        request: KnowledgeGraphQueryRequest,
    ) -> Result<KnowledgeGraphQueryResponse, KnowledgeGraphQueryError> {
        request.validate()?;
        if graph
            .get(&request.tenant_id, &request.root_entity_id)
            .is_none()
        {
            return Err(KnowledgeGraphQueryError::MissingRootEntity);
        }

        let edge_filter = request.edge_filter();
        let mut queue = VecDeque::from([(request.root_entity_id.clone(), 0_u32)]);
        let mut seen_nodes = BTreeSet::from([request.root_entity_id.clone()]);
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeSet::new();
        insert_node(
            graph,
            &request.tenant_id,
            &request.root_entity_id,
            &mut nodes,
        )?;

        while let Some((entity_id, depth)) = queue.pop_front() {
            if depth >= request.max_depth {
                continue;
            }

            for link in self.outbound_links(&request.tenant_id, &entity_id) {
                if !edge_filter.is_empty() && !edge_filter.contains(link.edge_type_id.as_str()) {
                    continue;
                }
                if link.observed_at_epoch_seconds < request.freshness_floor_epoch_seconds {
                    continue;
                }
                validate_link_endpoints(graph, link)?;

                edges.insert(link.as_contract_edge());
                insert_node(graph, &request.tenant_id, &link.to_entity_id, &mut nodes)?;
                if seen_nodes.insert(link.to_entity_id.clone()) {
                    queue.push_back((link.to_entity_id.clone(), depth + 1));
                }
            }
        }

        Ok(KnowledgeGraphQueryResponse {
            query_id: request.query_id,
            tenant_id: request.tenant_id,
            nodes: nodes.into_values().collect(),
            edges: edges.into_iter().collect(),
            observed_at_epoch_seconds: request.observed_at_epoch_seconds,
        })
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    fn outbound_links<'a>(
        &'a self,
        tenant_id: &'a str,
        from_entity_id: &'a str,
    ) -> impl Iterator<Item = &'a KnowledgeGraphLinkInstance> + 'a {
        self.links
            .range(
                KnowledgeGraphLinkKey {
                    tenant_id: tenant_id.to_string(),
                    from_entity_id: from_entity_id.to_string(),
                    edge_type_id: String::new(),
                    to_entity_id: String::new(),
                }..,
            )
            .map_while(move |(key, link)| {
                ((key.tenant_id == tenant_id) && (key.from_entity_id == from_entity_id))
                    .then_some(link)
            })
    }
}

impl KnowledgeGraphLinkInstance {
    fn key(&self) -> KnowledgeGraphLinkKey {
        KnowledgeGraphLinkKey {
            tenant_id: self.tenant_id.clone(),
            from_entity_id: self.from_entity_id.clone(),
            edge_type_id: self.edge_type_id.clone(),
            to_entity_id: self.to_entity_id.clone(),
        }
    }
}

fn insert_node(
    graph: &ObjectGraph,
    tenant_id: &str,
    entity_id: &str,
    nodes: &mut BTreeMap<String, KnowledgeGraphNode>,
) -> Result<(), KnowledgeGraphQueryError> {
    let entity = graph.get(tenant_id, entity_id).ok_or_else(|| {
        KnowledgeGraphQueryError::DanglingLinkEndpoint {
            entity_id: entity_id.to_string(),
        }
    })?;
    nodes.insert(
        entity_id.to_string(),
        KnowledgeGraphNode {
            entity_id: entity_id.to_string(),
            entity_type_id: entity.entity_type.value.clone(),
        },
    );
    Ok(())
}

fn validate_link_endpoints(
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

fn validate_tenant_id(tenant_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > "ten_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidTenantId)
    }
}

fn validate_query_id(query_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if query_id.starts_with("kgq_") && query_id.len() > "kgq_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidQueryId)
    }
}

fn validate_entity_id(entity_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if entity_id.starts_with("ent_") && entity_id.len() > "ent_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidEntityId)
    }
}

fn validate_edge_type_id(edge_type_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if edge_type_id.starts_with("lty_") && edge_type_id.len() > "lty_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidEdgeTypeId)
    }
}

fn validate_max_depth(max_depth: u32) -> Result<(), KnowledgeGraphQueryError> {
    if (1..=MAX_QUERY_DEPTH).contains(&max_depth) {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::InvalidMaxDepth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClass, PrivacyDataClass};
    use oya_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyTier};

    fn property(name: &str) -> ObjectProperty {
        ObjectProperty::new(
            name.to_string(),
            "value".to_string(),
            PropertyTier::Scalar,
            PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
        )
    }

    fn graph() -> ObjectGraph {
        let mut graph = ObjectGraph::default();
        for (tenant_id, entity_id, entity_type) in [
            ("ten_alpha", "ent_root", "ety_account"),
            ("ten_alpha", "ent_contact", "ety_contact"),
            ("ten_alpha", "ent_case", "ety_case"),
            ("ten_alpha", "ent_cycle", "ety_case"),
            ("ten_beta", "ent_beta_root", "ety_account"),
        ] {
            graph
                .upsert_entity(
                    ObjectEntity::new(
                        tenant_id.to_string(),
                        entity_id.to_string(),
                        entity_type.to_string(),
                        vec![property("name")],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        graph
    }

    fn request(
        root_entity_id: &str,
        edge_type_ids: Vec<&str>,
        max_depth: u32,
        freshness_floor_epoch_seconds: u64,
    ) -> KnowledgeGraphQueryRequest {
        KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_two_hop",
            root_entity_id,
            edge_type_ids,
            max_depth,
            freshness_floor_epoch_seconds,
            12,
        )
        .unwrap()
    }

    #[test]
    fn bounded_query_returns_deterministic_two_hop_subgraph() {
        let graph = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        engine
            .upsert_link(
                &graph,
                KnowledgeGraphLinkInstance::new(
                    "ten_alpha",
                    "ent_root",
                    "ent_contact",
                    "lty_owns",
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        engine
            .upsert_link(
                &graph,
                KnowledgeGraphLinkInstance::new(
                    "ten_alpha",
                    "ent_contact",
                    "ent_case",
                    "lty_related",
                    11,
                )
                .unwrap(),
            )
            .unwrap();

        let response = engine
            .query_graph_slice(&graph, request("ent_root", vec![], 2, 0))
            .unwrap();

        assert_eq!(
            response
                .nodes
                .iter()
                .map(|node| node.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["ent_case", "ent_contact", "ent_root"]
        );
        assert_eq!(
            response
                .edges
                .iter()
                .map(|edge| {
                    (
                        edge.from_entity_id.as_str(),
                        edge.edge_type_id.as_str(),
                        edge.to_entity_id.as_str(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                ("ent_contact", "lty_related", "ent_case"),
                ("ent_root", "lty_owns", "ent_contact")
            ]
        );
        assert_eq!(response.observed_at_epoch_seconds, 12);
    }

    #[test]
    fn edge_type_filter_and_freshness_floor_prune_traversal() {
        let graph = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        for (from, to, edge, observed_at) in [
            ("ent_root", "ent_contact", "lty_owns", 100),
            ("ent_root", "ent_case", "lty_related", 100),
            ("ent_contact", "ent_case", "lty_owns", 10),
        ] {
            engine
                .upsert_link(
                    &graph,
                    KnowledgeGraphLinkInstance::new("ten_alpha", from, to, edge, observed_at)
                        .unwrap(),
                )
                .unwrap();
        }

        let response = engine
            .query_graph_slice(&graph, request("ent_root", vec!["lty_owns"], 2, 50))
            .unwrap();

        assert_eq!(
            response
                .nodes
                .iter()
                .map(|node| node.entity_id.as_str())
                .collect::<Vec<_>>(),
            vec!["ent_contact", "ent_root"]
        );
        assert_eq!(response.edges.len(), 1);
        assert_eq!(response.edges[0].edge_type_id, "lty_owns");
    }

    #[test]
    fn tenant_isolation_blocks_cross_tenant_and_query_leakage() {
        let graph = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        let cross_tenant_link = KnowledgeGraphLinkInstance::new(
            "ten_alpha",
            "ent_root",
            "ent_beta_root",
            "lty_owns",
            1,
        )
        .unwrap();

        assert_eq!(
            engine.upsert_link(&graph, cross_tenant_link),
            Err(KnowledgeGraphQueryError::DanglingLinkEndpoint {
                entity_id: "ent_beta_root".to_string()
            })
        );

        engine
            .upsert_link(
                &graph,
                KnowledgeGraphLinkInstance::new(
                    "ten_alpha",
                    "ent_root",
                    "ent_contact",
                    "lty_owns",
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let beta_response = engine
            .query_graph_slice(
                &graph,
                KnowledgeGraphQueryRequest::new(
                    "ten_beta",
                    "kgq_beta",
                    "ent_beta_root",
                    Vec::<&str>::new(),
                    2,
                    0,
                    2,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(beta_response.nodes.len(), 1);
        assert_eq!(beta_response.nodes[0].entity_id, "ent_beta_root");
        assert!(beta_response.edges.is_empty());
    }

    #[test]
    fn validation_rejects_bad_ids_missing_root_and_unbounded_depth() {
        assert_eq!(
            KnowledgeGraphQueryRequest::new(
                "tenant_alpha",
                "kgq_bad_tenant",
                "ent_root",
                Vec::<&str>::new(),
                1,
                0,
                1,
            ),
            Err(KnowledgeGraphQueryError::InvalidTenantId)
        );
        assert_eq!(
            KnowledgeGraphQueryRequest::new(
                "ten_alpha",
                "query_bad",
                "ent_root",
                Vec::<&str>::new(),
                1,
                0,
                1,
            ),
            Err(KnowledgeGraphQueryError::InvalidQueryId)
        );
        assert_eq!(
            KnowledgeGraphQueryRequest::new(
                "ten_alpha",
                "kgq_bad_depth",
                "ent_root",
                Vec::<&str>::new(),
                MAX_QUERY_DEPTH + 1,
                0,
                1,
            ),
            Err(KnowledgeGraphQueryError::InvalidMaxDepth)
        );

        let engine = KnowledgeGraphQueryEngine::default();
        assert_eq!(
            engine.query_graph_slice(&graph(), request("ent_missing", Vec::<&str>::new(), 1, 0)),
            Err(KnowledgeGraphQueryError::MissingRootEntity)
        );
    }

    #[test]
    fn cycle_edges_are_reported_without_unbounded_revisit() {
        let graph = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        for (from, to) in [
            ("ent_root", "ent_contact"),
            ("ent_contact", "ent_cycle"),
            ("ent_cycle", "ent_root"),
        ] {
            engine
                .upsert_link(
                    &graph,
                    KnowledgeGraphLinkInstance::new("ten_alpha", from, to, "lty_owns", 1).unwrap(),
                )
                .unwrap();
        }

        let response = engine
            .query_graph_slice(&graph, request("ent_root", Vec::<&str>::new(), 16, 0))
            .unwrap();

        assert_eq!(response.nodes.len(), 3);
        assert_eq!(response.edges.len(), 3);
        assert!(
            response
                .edges
                .iter()
                .any(|edge| edge.from_entity_id == "ent_cycle" && edge.to_entity_id == "ent_root")
        );
    }

    #[test]
    fn upsert_link_updates_observed_time_for_same_tenant_edge_key() {
        let graph = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        let first =
            KnowledgeGraphLinkInstance::new("ten_alpha", "ent_root", "ent_contact", "lty_owns", 1)
                .unwrap();
        let updated =
            KnowledgeGraphLinkInstance::new("ten_alpha", "ent_root", "ent_contact", "lty_owns", 99)
                .unwrap();

        assert_eq!(
            engine.upsert_link(&graph, first),
            Ok(KnowledgeGraphLinkUpsertOutcome::Inserted)
        );
        assert_eq!(
            engine.upsert_link(&graph, updated),
            Ok(KnowledgeGraphLinkUpsertOutcome::Updated)
        );
        let response = engine
            .query_graph_slice(&graph, request("ent_root", Vec::<&str>::new(), 1, 50))
            .unwrap();

        assert_eq!(engine.link_count(), 1);
        assert_eq!(response.edges.len(), 1);
    }
}
