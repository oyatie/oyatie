//! Ontology query-engine domain foundation.
//!
//! This crate implements the source-level, in-memory query semantics for the
//! preview Knowledge Graph contract. It intentionally stays adapter-free: cloud
//! storage, query languages, distributed execution, authz enforcement, and SLO
//! runtime evidence are future slices. The implemented semantics are bounded,
//! tenant-scoped, deterministic traversal (outbound, inbound, or both) over
//! validated link instances.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use data_ontology_kernel::ObjectGraph;

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
pub struct KnowledgeGraphLinkInstance {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub from_entity_id: String,         // data_class: INTERNAL_ONLY
    pub to_entity_id: String,           // data_class: INTERNAL_ONLY
    pub edge_type_id: String,           // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeGraphQueryRequest {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub query_id: String,                     // data_class: INTERNAL_ONLY
    pub root_entity_id: String,               // data_class: INTERNAL_ONLY
    pub edge_type_ids: Vec<String>,           // data_class: INTERNAL_ONLY
    pub max_depth: u32,                       // data_class: INTERNAL_ONLY
    pub freshness_floor_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub consented_edge_type_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub direction: TraversalDirection,        // data_class: INTERNAL_ONLY
}

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KnowledgeGraphQueryEngine {
    links: BTreeMap<KnowledgeGraphLinkKey, KnowledgeGraphLinkInstance>, // data_class: INTERNAL_ONLY
    inbound: BTreeMap<KnowledgeGraphLinkInboundKey, KnowledgeGraphLinkInstance>, // data_class: INTERNAL_ONLY
}

/// Primary (outbound) index key: (tenant, from, edge_type, to).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct KnowledgeGraphLinkKey {
    tenant_id: String,
    from_entity_id: String,
    edge_type_id: String,
    to_entity_id: String,
}

/// Secondary (inbound) index key: (tenant, to, edge_type, from).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct KnowledgeGraphLinkInboundKey {
    tenant_id: String,
    to_entity_id: String,
    edge_type_id: String,
    from_entity_id: String,
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
        consented_edge_type_ids: Vec<impl Into<String>>,
        direction: TraversalDirection,
    ) -> Result<Self, KnowledgeGraphQueryError> {
        let request = Self {
            tenant_id: tenant_id.into(),
            query_id: query_id.into(),
            root_entity_id: root_entity_id.into(),
            edge_type_ids: edge_type_ids.into_iter().map(Into::into).collect(),
            max_depth,
            freshness_floor_epoch_seconds,
            observed_at_epoch_seconds,
            consented_edge_type_ids: consented_edge_type_ids
                .into_iter()
                .map(Into::into)
                .collect(),
            direction,
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
        for grant_id in &self.consented_edge_type_ids {
            validate_consent_grant_id(grant_id)?;
        }
        Ok(())
    }

    fn edge_filter(&self) -> BTreeSet<&str> {
        self.edge_type_ids.iter().map(String::as_str).collect()
    }

    pub fn consent_filter(&self) -> BTreeSet<&str> {
        self.consented_edge_type_ids
            .iter()
            .map(String::as_str)
            .collect()
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
        let inbound_key = link.inbound_key();
        let outcome = if self.links.insert(key, link.clone()).is_some() {
            KnowledgeGraphLinkUpsertOutcome::Updated
        } else {
            KnowledgeGraphLinkUpsertOutcome::Inserted
        };
        self.inbound.insert(inbound_key, link);
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
        let consent_filter = request.consent_filter();
        let mut queue = VecDeque::from([(request.root_entity_id.clone(), 0_u32)]);
        let mut seen_nodes = BTreeSet::from([request.root_entity_id.clone()]);
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeSet::new();
        let mut result_truncated = false;
        insert_node(
            graph,
            &request.tenant_id,
            &request.root_entity_id,
            &mut nodes,
        )?;

        'bfs: while let Some((entity_id, depth)) = queue.pop_front() {
            if depth >= request.max_depth {
                continue;
            }

            // Collect candidate links for this entity based on traversal direction.
            // Both outbound and inbound iterators borrow &self so we materialise
            // the inbound candidates into a Vec to avoid simultaneous borrows.
            let outbound_links: Vec<&KnowledgeGraphLinkInstance> = if matches!(
                request.direction,
                TraversalDirection::Outbound | TraversalDirection::Both
            ) {
                self.outbound_links(&request.tenant_id, &entity_id)
                    .collect()
            } else {
                vec![]
            };
            let inbound_links: Vec<&KnowledgeGraphLinkInstance> = if matches!(
                request.direction,
                TraversalDirection::Inbound | TraversalDirection::Both
            ) {
                self.inbound_links(&request.tenant_id, &entity_id).collect()
            } else {
                vec![]
            };

            for link in outbound_links.into_iter().chain(inbound_links) {
                if !edge_filter.is_empty() && !edge_filter.contains(link.edge_type_id.as_str()) {
                    continue;
                }
                if link.observed_at_epoch_seconds < request.freshness_floor_epoch_seconds {
                    continue;
                }
                if !consent_filter.is_empty()
                    && !consent_filter.contains(link.edge_type_id.as_str())
                {
                    continue;
                }
                validate_link_endpoints(graph, link)?;

                // Determine the neighbor node (the side we haven't visited yet).
                let neighbor_id = if link.from_entity_id == entity_id {
                    &link.to_entity_id
                } else {
                    &link.from_entity_id
                };

                // Node cap: stop before emitting an edge to a node that cannot
                // be included in the response. Returning an edge without both
                // endpoints would create a dangling/orphaned graph slice.
                if !nodes.contains_key(neighbor_id.as_str())
                    && nodes.len() >= MAX_QUERY_RESULT_NODES
                {
                    result_truncated = true;
                    break 'bfs;
                }

                // Edge cap: stop before inserting when at limit.
                if edges.len() >= MAX_QUERY_RESULT_EDGES {
                    result_truncated = true;
                    break 'bfs;
                }
                // Emit edge in canonical from→to orientation regardless of traversal direction.
                edges.insert(link.as_contract_edge());

                if !nodes.contains_key(neighbor_id.as_str()) {
                    insert_node(graph, &request.tenant_id, neighbor_id, &mut nodes)?;
                }

                if seen_nodes.insert(neighbor_id.clone()) {
                    queue.push_back((neighbor_id.clone(), depth + 1));
                }
            }
        }

        Ok(KnowledgeGraphQueryResponse {
            query_id: request.query_id,
            tenant_id: request.tenant_id,
            nodes: nodes.into_values().collect(),
            edges: edges.into_iter().collect(),
            observed_at_epoch_seconds: request.observed_at_epoch_seconds,
            result_truncated,
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

    fn inbound_links<'a>(
        &'a self,
        tenant_id: &'a str,
        to_entity_id: &'a str,
    ) -> impl Iterator<Item = &'a KnowledgeGraphLinkInstance> + 'a {
        self.inbound
            .range(
                KnowledgeGraphLinkInboundKey {
                    tenant_id: tenant_id.to_string(),
                    to_entity_id: to_entity_id.to_string(),
                    edge_type_id: String::new(),
                    from_entity_id: String::new(),
                }..,
            )
            .map_while(move |(key, link)| {
                ((key.tenant_id == tenant_id) && (key.to_entity_id == to_entity_id)).then_some(link)
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

    fn inbound_key(&self) -> KnowledgeGraphLinkInboundKey {
        KnowledgeGraphLinkInboundKey {
            tenant_id: self.tenant_id.clone(),
            to_entity_id: self.to_entity_id.clone(),
            edge_type_id: self.edge_type_id.clone(),
            from_entity_id: self.from_entity_id.clone(),
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
    if max_depth == 0 {
        return Err(KnowledgeGraphQueryError::InvalidMaxDepth);
    }
    if max_depth > MAX_QUERY_DEPTH {
        return Err(KnowledgeGraphQueryError::DepthCeilingExceeded);
    }
    Ok(())
}

fn validate_consent_grant_id(grant_id: &str) -> Result<(), KnowledgeGraphQueryError> {
    if grant_id.starts_with("lty_") && grant_id.len() > "lty_".len() {
        Ok(())
    } else {
        Err(KnowledgeGraphQueryError::MalformedConsentGrantId {
            id: grant_id.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyTier};
    use data_boundary_kernel::{DataClass, PrivacyDataClass};

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
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap()
    }

    fn assert_every_edge_endpoint_is_returned(response: &KnowledgeGraphQueryResponse) {
        let node_ids: BTreeSet<&str> = response
            .nodes
            .iter()
            .map(|node| node.entity_id.as_str())
            .collect();
        for edge in &response.edges {
            assert!(
                node_ids.contains(edge.from_entity_id.as_str()),
                "edge source {} must be present in response nodes",
                edge.from_entity_id
            );
            assert!(
                node_ids.contains(edge.to_entity_id.as_str()),
                "edge target {} must be present in response nodes",
                edge.to_entity_id
            );
        }
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
                    Vec::<&str>::new(),
                    TraversalDirection::Outbound,
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
                Vec::<&str>::new(),
                TraversalDirection::Outbound,
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
                Vec::<&str>::new(),
                TraversalDirection::Outbound,
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
                Vec::<&str>::new(),
                TraversalDirection::Outbound,
            ),
            Err(KnowledgeGraphQueryError::DepthCeilingExceeded)
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

    // ---- ST1: result-cardinality ceilings + result_truncated signal ----

    /// ST1-a: A star graph with leaf_count > MAX_QUERY_RESULT_NODES triggers
    /// truncation.  The response MUST set result_truncated = true and return
    /// at most MAX_QUERY_RESULT_NODES + 1 nodes (cap + root).  Running the
    /// same query twice MUST return identical node and edge counts
    /// (determinism guarantee).
    #[test]
    fn node_cap_triggers_result_truncated_deterministically() {
        let cap = MAX_QUERY_RESULT_NODES; // constant must exist
        let leaf_count = cap + 1;

        let mut g = ObjectGraph::default();
        g.upsert_entity(
            ObjectEntity::new(
                "ten_alpha".to_string(),
                "ent_root".to_string(),
                "ety_account".to_string(),
                vec![property("name")],
            )
            .unwrap(),
        )
        .unwrap();
        for i in 0..leaf_count {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    format!("ent_leaf_{i:04}"),
                    "ety_contact".to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }

        let mut engine = KnowledgeGraphQueryEngine::default();
        for i in 0..leaf_count {
            engine
                .upsert_link(
                    &g,
                    KnowledgeGraphLinkInstance::new(
                        "ten_alpha",
                        "ent_root",
                        format!("ent_leaf_{i:04}"),
                        "lty_owns",
                        1_u64,
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_node_cap",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            0,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap();

        let r1 = engine.query_graph_slice(&g, req.clone()).unwrap();
        let r2 = engine.query_graph_slice(&g, req).unwrap();

        // result_truncated field must exist and be true
        assert!(
            r1.result_truncated,
            "first run: node cap must set result_truncated"
        );
        assert!(
            r2.result_truncated,
            "second run: node cap must set result_truncated"
        );
        // determinism: identical counts across repeated calls
        assert_eq!(
            r1.nodes.len(),
            r2.nodes.len(),
            "node count must be deterministic"
        );
        assert_eq!(
            r1.edges.len(),
            r2.edges.len(),
            "edge count must be deterministic"
        );
        // returned node set must not exceed cap + root
        assert!(
            r1.nodes.len() <= cap + 1,
            "nodes must not exceed cap + root"
        );
        assert_every_edge_endpoint_is_returned(&r1);
    }

    /// ST1-b: A star graph with leaf_count > MAX_QUERY_RESULT_EDGES triggers
    /// truncation via the edge ceiling.  result_truncated must be true and
    /// the returned edge count must not exceed MAX_QUERY_RESULT_EDGES.
    #[test]
    fn edge_cap_triggers_result_truncated() {
        let edge_cap = MAX_QUERY_RESULT_EDGES; // constant must exist
        let leaf_count = edge_cap + 1;

        let mut g = ObjectGraph::default();
        g.upsert_entity(
            ObjectEntity::new(
                "ten_alpha".to_string(),
                "ent_root".to_string(),
                "ety_account".to_string(),
                vec![property("name")],
            )
            .unwrap(),
        )
        .unwrap();
        for i in 0..leaf_count {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    format!("ent_leaf_{i:05}"),
                    "ety_contact".to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }

        let mut engine = KnowledgeGraphQueryEngine::default();
        for i in 0..leaf_count {
            engine
                .upsert_link(
                    &g,
                    KnowledgeGraphLinkInstance::new(
                        "ten_alpha",
                        "ent_root",
                        format!("ent_leaf_{i:05}"),
                        "lty_owns",
                        1_u64,
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_edge_cap",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            0,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap();

        let response = engine.query_graph_slice(&g, req).unwrap();
        assert!(
            response.result_truncated,
            "edge cap must set result_truncated"
        );
        assert!(
            response.edges.len() <= edge_cap,
            "returned edges must not exceed MAX_QUERY_RESULT_EDGES"
        );
    }

    /// ST1-c: A small graph (3 nodes, 2 edges) — well under both caps —
    /// MUST return result_truncated = false and complete results.
    #[test]
    fn under_cap_query_returns_complete_results_with_result_truncated_false() {
        let g = graph();
        let mut engine = KnowledgeGraphQueryEngine::default();
        engine
            .upsert_link(
                &g,
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
                &g,
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
            .query_graph_slice(&g, request("ent_root", vec![], 2, 0))
            .unwrap();

        // result_truncated field must exist and be false for small graphs
        assert!(
            !response.result_truncated,
            "under-cap result must not be truncated"
        );
        assert_eq!(response.nodes.len(), 3);
        assert_eq!(response.edges.len(), 2);
    }

    // ---- ST2: DepthCeilingExceeded error variant ----

    /// ST2-a: max_depth > MAX_QUERY_DEPTH must be rejected with the new
    /// DepthCeilingExceeded variant, NOT with InvalidMaxDepth.
    #[test]
    fn max_depth_above_ceiling_returns_depth_ceiling_exceeded_not_invalid_max_depth() {
        let result = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_over_depth",
            "ent_root",
            Vec::<&str>::new(),
            MAX_QUERY_DEPTH + 1,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        );
        // DepthCeilingExceeded variant must exist and be returned here
        assert_eq!(
            result,
            Err(KnowledgeGraphQueryError::DepthCeilingExceeded),
            "max_depth > MAX_QUERY_DEPTH must return DepthCeilingExceeded"
        );
    }

    /// ST2-b: max_depth == MAX_QUERY_DEPTH (exactly at ceiling) must be
    /// accepted — Ok result, not an error.
    #[test]
    fn max_depth_at_ceiling_is_accepted() {
        assert!(
            KnowledgeGraphQueryRequest::new(
                "ten_alpha",
                "kgq_at_ceiling",
                "ent_root",
                Vec::<&str>::new(),
                MAX_QUERY_DEPTH,
                0,
                1,
                Vec::<&str>::new(),
                TraversalDirection::Outbound,
            )
            .is_ok(),
            "max_depth == MAX_QUERY_DEPTH must be accepted"
        );
    }

    /// ST2-c: max_depth == 0 must still return InvalidMaxDepth (not
    /// DepthCeilingExceeded), preserving the existing structural-invalidity
    /// distinction.
    #[test]
    fn max_depth_zero_returns_invalid_max_depth_not_depth_ceiling_exceeded() {
        assert_eq!(
            KnowledgeGraphQueryRequest::new(
                "ten_alpha",
                "kgq_zero_depth",
                "ent_root",
                Vec::<&str>::new(),
                0,
                0,
                1,
                Vec::<&str>::new(),
                TraversalDirection::Outbound,
            ),
            Err(KnowledgeGraphQueryError::InvalidMaxDepth),
            "max_depth == 0 must return InvalidMaxDepth"
        );
    }

    // ---- Consent grant scope (ST1 + ST2) — RED tests ----
    // These tests reference:
    //   * the `consented_edge_type_ids` parameter added to `KnowledgeGraphQueryRequest::new`
    //   * the `KnowledgeGraphQueryError::MalformedConsentGrantId` variant
    //   * the `consent_filter()` helper method on `KnowledgeGraphQueryRequest`
    //   * the BFS consent gate in `KnowledgeGraphQueryEngine::query_graph_slice`
    // None of the above exist yet, so these tests MUST fail to compile (red stage).

    /// Builds a consent-scoped graph used by the consent gate tests:
    ///   ent_root --lty_partner--> ent_b --lty_partner--> ent_c
    ///   ent_root --lty_member-->  ent_d
    fn consent_graph() -> ObjectGraph {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [
            ("ent_root", "ety_account"),
            ("ent_b", "ety_contact"),
            ("ent_c", "ety_contact"),
            ("ent_d", "ety_contact"),
        ] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        g
    }

    /// Builds the consent-scoped engine for the consent gate tests.
    fn consent_engine(g: &ObjectGraph) -> KnowledgeGraphQueryEngine {
        let mut engine = KnowledgeGraphQueryEngine::default();
        for (from, to, edge_type) in [
            ("ent_root", "ent_b", "lty_partner"),
            ("ent_b", "ent_c", "lty_partner"),
            ("ent_root", "ent_d", "lty_member"),
        ] {
            engine
                .upsert_link(
                    g,
                    KnowledgeGraphLinkInstance::new("ten_alpha", from, to, edge_type, 1).unwrap(),
                )
                .unwrap();
        }
        engine
    }

    // ST1 acceptance: a malformed consent grant id (no `lty_` prefix) must be
    // rejected with `MalformedConsentGrantId`.
    #[test]
    fn malformed_consent_grant_id_rejected() {
        let result = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_bad_grant",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            1,
            vec!["bad_id"],
            TraversalDirection::Outbound,
        );
        assert_eq!(
            result,
            Err(KnowledgeGraphQueryError::MalformedConsentGrantId {
                id: "bad_id".to_string()
            }),
            "a consent grant id without the lty_ prefix must return MalformedConsentGrantId"
        );
    }

    // ST1 acceptance: a well-formed consent grant id (`lty_partner`) must be
    // accepted without error.
    #[test]
    fn well_formed_consent_grant_id_accepted() {
        let result = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_good_grant",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            1,
            vec!["lty_partner"],
            TraversalDirection::Outbound,
        );
        assert!(
            result.is_ok(),
            "a consent grant id with a valid lty_ prefix must be accepted"
        );
    }

    // ST1 acceptance: consent_filter() on a non-empty scope returns the
    // expected BTreeSet of string slices.
    #[test]
    fn consent_filter_returns_set_of_consented_edge_type_ids() {
        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_filter_set",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            1,
            vec!["lty_partner", "lty_member"],
            TraversalDirection::Outbound,
        )
        .unwrap();
        let filter = req.consent_filter();
        assert!(
            filter.contains("lty_partner"),
            "consent_filter must contain lty_partner"
        );
        assert!(
            filter.contains("lty_member"),
            "consent_filter must contain lty_member"
        );
        assert!(
            !filter.contains("lty_owns"),
            "consent_filter must not contain lty_owns"
        );
    }

    // ST2 acceptance: when a non-empty consent scope is supplied, the BFS must
    // prune edges whose edge_type_id is absent from the scope, so downstream
    // nodes reachable only via those edges are absent from the response.
    //
    // Graph:
    //   ent_root --lty_partner--> ent_b --lty_partner--> ent_c
    //   ent_root --lty_member-->  ent_d
    // Scope: ["lty_partner"]
    // Expected: ent_b and ent_c present; ent_d absent.
    //           lty_partner edges present; lty_member edge absent.
    #[test]
    fn consent_filter_prunes_non_consented_edges() {
        let g = consent_graph();
        let engine = consent_engine(&g);

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_consent_prune",
            "ent_root",
            Vec::<&str>::new(),
            3,
            0,
            1,
            vec!["lty_partner"],
            TraversalDirection::Outbound,
        )
        .unwrap();

        let response = engine.query_graph_slice(&g, req).unwrap();

        let node_ids: Vec<&str> = response
            .nodes
            .iter()
            .map(|n| n.entity_id.as_str())
            .collect();
        assert!(
            node_ids.contains(&"ent_b"),
            "ent_b (reached via consented lty_partner) must be in response nodes"
        );
        assert!(
            node_ids.contains(&"ent_c"),
            "ent_c (reached via consented lty_partner hop) must be in response nodes"
        );
        assert!(
            !node_ids.contains(&"ent_d"),
            "ent_d (reachable only via non-consented lty_member) must be absent from response nodes"
        );

        let member_edges: Vec<_> = response
            .edges
            .iter()
            .filter(|e| e.edge_type_id == "lty_member")
            .collect();
        assert!(
            member_edges.is_empty(),
            "no lty_member edge must appear in response edges when lty_member is not in consent scope"
        );

        let partner_edges: Vec<_> = response
            .edges
            .iter()
            .filter(|e| e.edge_type_id == "lty_partner")
            .collect();
        assert_eq!(
            partner_edges.len(),
            2,
            "both lty_partner edges (root->b, b->c) must appear in response edges"
        );
    }

    // ST2 acceptance: an empty consent scope must preserve prior traversal
    // behaviour — all reachable nodes and edges are returned.
    //
    // Graph (same as above):
    //   ent_root --lty_partner--> ent_b --lty_partner--> ent_c
    //   ent_root --lty_member-->  ent_d
    // Scope: [] (empty)
    // Expected: ent_b, ent_c, and ent_d all present; all 3 edges present.
    #[test]
    fn empty_consent_scope_preserves_prior_behavior() {
        let g = consent_graph();
        let engine = consent_engine(&g);

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_no_consent_filter",
            "ent_root",
            Vec::<&str>::new(),
            3,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap();

        let response = engine.query_graph_slice(&g, req).unwrap();

        let node_ids: Vec<&str> = response
            .nodes
            .iter()
            .map(|n| n.entity_id.as_str())
            .collect();
        assert!(
            node_ids.contains(&"ent_b"),
            "ent_b must be present with empty consent scope"
        );
        assert!(
            node_ids.contains(&"ent_c"),
            "ent_c must be present with empty consent scope"
        );
        assert!(
            node_ids.contains(&"ent_d"),
            "ent_d must be present with empty consent scope"
        );

        assert_eq!(
            response.edges.len(),
            3,
            "all 3 edges must be returned when consent scope is empty"
        );
    }

    // ST2 acceptance: consent gate fires before the cardinality cap checks, so
    // pruned (non-consented) edges do not count toward the cap.  With a small
    // graph well under caps, result_truncated must remain false.
    #[test]
    fn consent_gate_fires_before_cap_check_and_does_not_set_result_truncated() {
        let g = consent_graph();
        let engine = consent_engine(&g);

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_consent_no_trunc",
            "ent_root",
            Vec::<&str>::new(),
            3,
            0,
            1,
            vec!["lty_partner"],
            TraversalDirection::Outbound,
        )
        .unwrap();

        let response = engine.query_graph_slice(&g, req).unwrap();
        assert!(
            !response.result_truncated,
            "result_truncated must be false when pruning reduces result well below caps"
        );
    }

    // ST2 acceptance: consent gate fires after the freshness filter, so a
    // stale consented edge is still dropped by freshness before the consent
    // check could pass it through.
    #[test]
    fn freshness_filter_still_applies_to_consented_edges() {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [("ent_root", "ety_account"), ("ent_b", "ety_contact")] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        // Insert a consented edge that is stale (observed_at = 5, freshness_floor = 10).
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new("ten_alpha", "ent_root", "ent_b", "lty_partner", 5)
                    .unwrap(),
            )
            .unwrap();

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_stale_consented",
            "ent_root",
            Vec::<&str>::new(),
            2,
            10, // freshness floor is 10
            1,
            vec!["lty_partner"], // lty_partner is consented, but observed_at=5 < floor=10
            TraversalDirection::Outbound,
        )
        .unwrap();

        let response = engine.query_graph_slice(&g, req).unwrap();
        let node_ids: Vec<&str> = response
            .nodes
            .iter()
            .map(|n| n.entity_id.as_str())
            .collect();
        assert!(
            !node_ids.contains(&"ent_b"),
            "a consented but stale edge must be pruned by freshness; ent_b must be absent"
        );
    }

    // ---- TraversalDirection: Inbound / Both tests ----

    /// Builds a directed chain graph for direction traversal tests:
    ///   ent_pred --lty_owns--> ent_root --lty_owns--> ent_succ
    ///
    /// Outbound from ent_root reaches ent_succ only.
    /// Inbound from ent_root reaches ent_pred only.
    /// Both from ent_root reaches ent_pred and ent_succ.
    fn dir_graph() -> ObjectGraph {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [
            ("ent_pred", "ety_account"),
            ("ent_root", "ety_account"),
            ("ent_succ", "ety_account"),
        ] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        g
    }

    fn dir_engine(g: &ObjectGraph) -> KnowledgeGraphQueryEngine {
        let mut engine = KnowledgeGraphQueryEngine::default();
        for (from, to) in [("ent_pred", "ent_root"), ("ent_root", "ent_succ")] {
            engine
                .upsert_link(
                    g,
                    KnowledgeGraphLinkInstance::new("ten_alpha", from, to, "lty_owns", 1).unwrap(),
                )
                .unwrap();
        }
        engine
    }

    /// Inbound traversal from ent_root reaches ent_pred (predecessor) but NOT
    /// ent_succ (successor). Outbound would not reach ent_pred.
    #[test]
    fn inbound_reaches_predecessors_outbound_cannot() {
        let g = dir_graph();
        let engine = dir_engine(&g);

        let inbound_req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Inbound,
        )
        .unwrap();
        let inbound_resp = engine.query_graph_slice(&g, inbound_req).unwrap();
        let inbound_nodes: Vec<&str> = inbound_resp
            .nodes
            .iter()
            .map(|n| n.entity_id.as_str())
            .collect();

        assert!(
            inbound_nodes.contains(&"ent_pred"),
            "Inbound must reach predecessor ent_pred"
        );
        assert!(
            !inbound_nodes.contains(&"ent_succ"),
            "Inbound must NOT reach successor ent_succ"
        );

        let outbound_req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_outbound_dir",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap();
        let outbound_resp = engine.query_graph_slice(&g, outbound_req).unwrap();
        let outbound_nodes: Vec<&str> = outbound_resp
            .nodes
            .iter()
            .map(|n| n.entity_id.as_str())
            .collect();

        assert!(
            !outbound_nodes.contains(&"ent_pred"),
            "Outbound must NOT reach predecessor ent_pred"
        );
        assert!(
            outbound_nodes.contains(&"ent_succ"),
            "Outbound must reach successor ent_succ"
        );
    }

    /// Both direction from ent_root yields the union: ent_pred and ent_succ
    /// both visible, with no duplicate nodes or edges.
    #[test]
    fn both_yields_union_of_outbound_and_inbound() {
        let g = dir_graph();
        let engine = dir_engine(&g);

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_both",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Both,
        )
        .unwrap();
        let resp = engine.query_graph_slice(&g, req).unwrap();
        let node_ids: Vec<&str> = resp.nodes.iter().map(|n| n.entity_id.as_str()).collect();

        assert!(node_ids.contains(&"ent_pred"), "Both must include ent_pred");
        assert!(node_ids.contains(&"ent_root"), "Both must include ent_root");
        assert!(node_ids.contains(&"ent_succ"), "Both must include ent_succ");
        // No duplicate nodes
        assert_eq!(node_ids.len(), 3, "Both must not duplicate nodes");
        // Both edges present with canonical from->to orientation
        assert_eq!(resp.edges.len(), 2, "Both must return exactly 2 edges");
        assert!(
            resp.edges
                .iter()
                .any(|e| e.from_entity_id == "ent_pred" && e.to_entity_id == "ent_root"),
            "pred->root edge must be present in canonical orientation"
        );
        assert!(
            resp.edges
                .iter()
                .any(|e| e.from_entity_id == "ent_root" && e.to_entity_id == "ent_succ"),
            "root->succ edge must be present in canonical orientation"
        );
    }

    /// Omitting an explicit direction (using Outbound default) reproduces the
    /// same result as an explicit Outbound request byte-for-byte.
    #[test]
    fn default_direction_reproduces_outbound_result() {
        let g = dir_graph();
        let engine = dir_engine(&g);

        let explicit = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_explicit_out",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .unwrap();
        let default_dir = KnowledgeGraphQueryRequest {
            query_id: "kgq_default_dir".to_string(),
            ..explicit.clone()
        };

        let r_explicit = engine.query_graph_slice(&g, explicit).unwrap();
        let r_default = engine.query_graph_slice(&g, default_dir).unwrap();

        assert_eq!(
            r_explicit.nodes, r_default.nodes,
            "default direction must produce same nodes as explicit Outbound"
        );
        assert_eq!(
            r_explicit.edges, r_default.edges,
            "default direction must produce same edges as explicit Outbound"
        );
        assert_eq!(
            r_explicit.result_truncated, r_default.result_truncated,
            "default direction must produce same result_truncated as explicit Outbound"
        );
    }

    /// Consent scope prunes correctly under Inbound traversal.
    /// Graph: ent_pred --lty_owns--> ent_root <--lty_partner-- ent_other
    /// With consent scope ["lty_partner"], inbound traversal from ent_root
    /// must see ent_other (via consented lty_partner) but not ent_pred (via
    /// non-consented lty_owns).
    #[test]
    fn inbound_consent_prunes_non_consented_edges() {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [
            ("ent_pred", "ety_account"),
            ("ent_root", "ety_account"),
            ("ent_other", "ety_account"),
        ] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new("ten_alpha", "ent_pred", "ent_root", "lty_owns", 1)
                    .unwrap(),
            )
            .unwrap();
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new(
                    "ten_alpha",
                    "ent_other",
                    "ent_root",
                    "lty_partner",
                    1,
                )
                .unwrap(),
            )
            .unwrap();

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound_consent",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            vec!["lty_partner"],
            TraversalDirection::Inbound,
        )
        .unwrap();
        let resp = engine.query_graph_slice(&g, req).unwrap();
        let node_ids: Vec<&str> = resp.nodes.iter().map(|n| n.entity_id.as_str()).collect();

        assert!(
            node_ids.contains(&"ent_other"),
            "ent_other (via consented lty_partner inbound) must be present"
        );
        assert!(
            !node_ids.contains(&"ent_pred"),
            "ent_pred (via non-consented lty_owns inbound) must be absent"
        );
    }

    /// Freshness floor prunes stale inbound edges correctly.
    #[test]
    fn inbound_freshness_floor_prunes_stale_edges() {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [("ent_pred", "ety_account"), ("ent_root", "ety_account")] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        // stale inbound edge: observed_at=5, freshness_floor=10
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new("ten_alpha", "ent_pred", "ent_root", "lty_owns", 5)
                    .unwrap(),
            )
            .unwrap();

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound_stale",
            "ent_root",
            Vec::<&str>::new(),
            2,
            10,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Inbound,
        )
        .unwrap();
        let resp = engine.query_graph_slice(&g, req).unwrap();
        let node_ids: Vec<&str> = resp.nodes.iter().map(|n| n.entity_id.as_str()).collect();

        assert!(
            !node_ids.contains(&"ent_pred"),
            "stale inbound edge must be pruned by freshness floor"
        );
    }

    /// Node cardinality cap triggers result_truncated under Inbound traversal.
    #[test]
    fn inbound_node_cap_triggers_result_truncated() {
        let cap = MAX_QUERY_RESULT_NODES;
        let pred_count = cap + 1;

        let mut g = ObjectGraph::default();
        g.upsert_entity(
            ObjectEntity::new(
                "ten_alpha".to_string(),
                "ent_root".to_string(),
                "ety_account".to_string(),
                vec![property("name")],
            )
            .unwrap(),
        )
        .unwrap();
        for i in 0..pred_count {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    format!("ent_pred_{i:04}"),
                    "ety_account".to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }

        let mut engine = KnowledgeGraphQueryEngine::default();
        for i in 0..pred_count {
            engine
                .upsert_link(
                    &g,
                    KnowledgeGraphLinkInstance::new(
                        "ten_alpha",
                        format!("ent_pred_{i:04}"),
                        "ent_root",
                        "lty_owns",
                        1_u64,
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound_cap",
            "ent_root",
            Vec::<&str>::new(),
            1,
            0,
            0,
            Vec::<&str>::new(),
            TraversalDirection::Inbound,
        )
        .unwrap();
        let resp = engine.query_graph_slice(&g, req).unwrap();

        assert!(
            resp.result_truncated,
            "Inbound node cap must set result_truncated"
        );
        assert!(
            resp.nodes.len() <= cap + 1,
            "Inbound nodes must not exceed cap + root"
        );
        assert_every_edge_endpoint_is_returned(&resp);
    }

    /// Tenant isolation: inbound links from a different tenant are never returned.
    /// This is structurally enforced because upsert_link validates both endpoints
    /// exist in the same-tenant ObjectGraph. This test confirms the BFS inbound
    /// scan only returns same-tenant predecessors.
    #[test]
    fn inbound_tenant_isolation() {
        let mut g = ObjectGraph::default();
        for (tenant, entity_id, entity_type) in [
            ("ten_alpha", "ent_root", "ety_account"),
            ("ten_alpha", "ent_pred", "ety_account"),
            ("ten_beta", "ent_beta_pred", "ety_account"),
            ("ten_beta", "ent_beta_root", "ety_account"),
        ] {
            g.upsert_entity(
                ObjectEntity::new(
                    tenant.to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        // ten_alpha: ent_pred -> ent_root
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new("ten_alpha", "ent_pred", "ent_root", "lty_owns", 1)
                    .unwrap(),
            )
            .unwrap();
        // ten_beta: ent_beta_pred -> ent_beta_root (different tenant — must not leak)
        engine
            .upsert_link(
                &g,
                KnowledgeGraphLinkInstance::new(
                    "ten_beta",
                    "ent_beta_pred",
                    "ent_beta_root",
                    "lty_owns",
                    1,
                )
                .unwrap(),
            )
            .unwrap();

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound_isolation",
            "ent_root",
            Vec::<&str>::new(),
            2,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Inbound,
        )
        .unwrap();
        let resp = engine.query_graph_slice(&g, req).unwrap();
        let node_ids: Vec<&str> = resp.nodes.iter().map(|n| n.entity_id.as_str()).collect();

        assert!(
            node_ids.contains(&"ent_pred"),
            "same-tenant predecessor ent_pred must be visible"
        );
        assert!(
            !node_ids.contains(&"ent_beta_pred"),
            "cross-tenant ent_beta_pred must not be visible"
        );
        assert!(
            !node_ids.contains(&"ent_beta_root"),
            "cross-tenant ent_beta_root must not be visible"
        );
    }

    /// Cyclic inbound graph does not cause unbounded revisit.
    /// Graph (forming a cycle): ent_a -> ent_b -> ent_c -> ent_a
    /// Inbound from ent_a: should visit ent_c (direct predecessor), then ent_b,
    /// then back to ent_a (already seen), stopping. No infinite loop.
    #[test]
    fn inbound_cycle_no_unbounded_revisit() {
        let mut g = ObjectGraph::default();
        for (entity_id, entity_type) in [
            ("ent_a", "ety_account"),
            ("ent_b", "ety_account"),
            ("ent_c", "ety_account"),
        ] {
            g.upsert_entity(
                ObjectEntity::new(
                    "ten_alpha".to_string(),
                    entity_id.to_string(),
                    entity_type.to_string(),
                    vec![property("name")],
                )
                .unwrap(),
            )
            .unwrap();
        }
        let mut engine = KnowledgeGraphQueryEngine::default();
        for (from, to) in [("ent_a", "ent_b"), ("ent_b", "ent_c"), ("ent_c", "ent_a")] {
            engine
                .upsert_link(
                    &g,
                    KnowledgeGraphLinkInstance::new("ten_alpha", from, to, "lty_owns", 1).unwrap(),
                )
                .unwrap();
        }

        let req = KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_inbound_cycle",
            "ent_a",
            Vec::<&str>::new(),
            16,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Inbound,
        )
        .unwrap();
        // Must terminate and return the 3 cycle nodes.
        let resp = engine.query_graph_slice(&g, req).unwrap();
        assert_eq!(
            resp.nodes.len(),
            3,
            "inbound cycle must terminate and return 3 nodes"
        );
        assert_eq!(resp.edges.len(), 3, "inbound cycle must return 3 edges");
    }
}
