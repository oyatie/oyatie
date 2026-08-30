//! Query-engine tests: part 2.

use super::support::*;

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
            KnowledgeGraphLinkInstance::new("ten_alpha", "ent_root", "ent_contact", "lty_owns", 10)
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
