//! The traversal engine and its private link indexes.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use data_ontology_kernel::ObjectGraph;

use crate::contract::*;
use crate::link::KnowledgeGraphLinkInstance;
use crate::request::KnowledgeGraphQueryRequest;

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

pub(crate) fn insert_node(
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
