#![allow(dead_code)]
#[path = "human_relationship/disclosure.rs"]
mod disclosure;
mod facade_support;
mod failing_store;
#[path = "human_relationship/faults.rs"]
mod faults;
#[path = "human_relationship/grammar.rs"]
mod grammar;
#[path = "search_disclosure/support.rs"]
mod graph_support;
#[path = "human_status/support.rs"]
mod human_support;
#[path = "human_relationship/support.rs"]
mod support;

use axum::http::StatusCode;
use serde_json::json;
use support::{HumanGraph, request};

#[tokio::test]
async fn human_explicit_relationship_walk_returns_minimal_authorized_nodes() {
    let fixture = HumanGraph::new("human-relationship");
    let graph = fixture.walk(&request("explicit"), StatusCode::OK).await;
    assert_eq!(
        graph,
        json!({"nodes":[
        {"entity_id":"ent_record","entity_type_id":"ety_record","written_revision":1},
        {"entity_id":"ent_team","entity_type_id":"ety_team","written_revision":1}],
        "edges":[{"from_entity_id":"ent_record","to_entity_id":"ent_team","edge_type_id":"lty_member"}],
        "observed_at_epoch_seconds":1700000000,"truncated":false})
    );
    assert_eq!(
        fixture.policy.resources.lock().unwrap().as_slice(),
        ["ent_tenant_view", "ent_record", "ent_record", "ent_team"]
    );
    assert!(!graph.to_string().contains("private fixture title"));
    assert_eq!(fixture.fixture.log_head(), 0);
}
