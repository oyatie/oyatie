use axum::http::StatusCode;
use serde_json::json;

use crate::facade_support::TENANT;
use crate::graph_support::{links, object, objects, refusal};
use crate::support::{HumanGraph, request};

#[tokio::test]
async fn every_current_node_permission_is_required_on_first_and_cached_disclosure() {
    for resource in ["ent_record", "ent_team"] {
        let fixture = HumanGraph::new("human-cached-policy");
        let query = request("cached");
        fixture.walk(&query, StatusCode::OK).await;
        *fixture.policy.denied.lock().unwrap() = Some(resource);
        refusal(
            &fixture.walk(&query, StatusCode::FORBIDDEN).await,
            "authorization",
        );
        *fixture.policy.denied.lock().unwrap() = None;
        *fixture.policy.unavailable.lock().unwrap() = true;
        refusal(
            &fixture.walk(&query, StatusCode::FORBIDDEN).await,
            "authorization",
        );
    }
}

#[tokio::test]
async fn cached_graph_rechecks_current_endpoint_types_edges_and_target_revisions() {
    for changed in ["missing", "type", "edge", "revision"] {
        let fixture = HumanGraph::new(changed);
        let query = request("cached");
        fixture.walk(&query, StatusCode::OK).await;
        let mut current = objects();
        match changed {
            "missing" => {
                current.pop();
            }
            "type" => {
                current[1] = object(TENANT, "ent_team", "ety_record");
            }
            "revision" => {
                current[1].schema_revision = 777;
            }
            _ => {}
        }
        fixture
            .replace(current, if changed == "edge" { vec![] } else { links() })
            .await;
        refusal(
            &fixture.walk(&query, StatusCode::BAD_REQUEST).await,
            "surface",
        );
        assert_eq!(
            fixture.state.tenants[TENANT]
                .lock()
                .await
                .queries
                .receipt_count(),
            1
        );
    }
}

#[tokio::test]
async fn request_replays_freeze_timestamp_and_direction_but_new_key_reads_current_graph() {
    let fixture = HumanGraph::new("human-intent");
    let query = request("intent");
    let graph = fixture.walk(&query, StatusCode::OK).await;
    assert_eq!(fixture.walk(&query, StatusCode::OK).await, graph);
    let mut changed = query.clone();
    changed["observed_at_epoch_seconds"] = json!(1700000001);
    fixture.walk(&changed, StatusCode::BAD_REQUEST).await;
    changed = query.clone();
    changed["link_type"] = json!("lty_peer");
    fixture.walk(&changed, StatusCode::BAD_REQUEST).await;
    fixture.replace(objects(), vec![]).await;
    let mut fresh = query;
    fresh["idempotency_key"] = json!("fresh");
    let empty = fixture.walk(&fresh, StatusCode::OK).await;
    assert_eq!(empty["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(empty["edges"], json!([]));
    assert_eq!(empty["truncated"], false);
}

#[tokio::test]
async fn target_revision_is_the_current_written_revision_not_the_root_pin() {
    let fixture = HumanGraph::new("human-target-revision");
    let query = request("target-revision");
    let first = fixture.walk(&query, StatusCode::OK).await;
    assert_eq!(first["nodes"][1]["written_revision"], 1);
    {
        let mut tenant = fixture.state.tenants[TENANT].lock().await;
        let mut definition = tenant
            .projection()
            .engine
            .entity_type(
                TENANT,
                &data_ontology_kernel::EntityTypeId::new("ety_team").unwrap(),
            )
            .unwrap()
            .clone();
        definition.revision = 2;
        tenant
            .projection_for_injection()
            .engine
            .evolve_entity_type(definition)
            .unwrap();
    }
    let mut current = objects();
    current[1].schema_revision = 2;
    fixture.replace(current, links()).await;
    let graph = fixture.walk(&query, StatusCode::OK).await;
    assert_eq!(graph["nodes"][0]["written_revision"], 1);
    assert_eq!(graph["nodes"][1]["written_revision"], 2);
    assert_eq!(
        fixture.state.tenants[TENANT]
            .lock()
            .await
            .queries
            .receipt_count(),
        1,
        "cached graph metadata is enriched from current admitted objects"
    );
    let reply = crate::human_support::request(
        &fixture.router,
        "/v1/tenants/ten_acme/objects/ent_team?type=ety_team&revision=2",
        &[&crate::human_support::cookie()],
    )
    .await;
    let detail = crate::human_support::body(reply, StatusCode::OK).await;
    assert_eq!(detail["written_revision"], 2);
    assert!(
        detail.get("requested_revision").is_none(),
        "the native object wire is unchanged"
    );
}

#[tokio::test]
async fn human_retention_handoff_does_not_change_operator_wire_or_refusals() {
    let fixture = HumanGraph::new("human-operator-control");
    let mut current = objects();
    current[1].schema_revision = 777;
    fixture.replace(current, links()).await;
    fixture
        .walk(&request("human-retention"), StatusCode::BAD_REQUEST)
        .await;
    let operator = crate::facade_support::Session::from_shared(fixture.state.clone());
    let (status, raw) = operator
        .post_to(
            Some(fixture.fixture.operator_token()),
            "/v1/search-around",
            &crate::graph_support::body("operator-retention").to_string(),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{raw}");
    let graph: serde_json::Value = serde_json::from_str(&raw).unwrap();
    for node in graph["nodes"].as_array().unwrap() {
        assert_eq!(node.as_object().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn capped_human_relationship_result_remains_explicitly_incomplete() {
    let fixture = HumanGraph::new("human-cap");
    let mut current = vec![object(TENANT, "ent_record", "ety_record")];
    let mut selected = Vec::new();
    for i in 0..1000 {
        let reference = format!("ent_team_{i:04}");
        current.push(object(TENANT, &reference, "ety_team"));
        selected.push(foundry_projection_draft::ProjectedLink {
            link_type: "lty_member".into(),
            from_object_ref: "ent_record".into(),
            to_object_ref: reference,
            observed_at_epoch_ms: 1700000000000,
        });
    }
    fixture.replace(current, selected).await;
    let graph = fixture.walk(&request("cap"), StatusCode::OK).await;
    assert_eq!(graph["truncated"], true);
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 1000);
    assert!(!graph["edges"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn human_seed_pin_and_type_are_bound_to_the_same_process_local_query_intent() {
    let fixture = HumanGraph::new("human-seed-intent");
    let query = request("seed-intent");
    let first = fixture.walk(&query, StatusCode::OK).await;
    assert_eq!(fixture.walk(&query, StatusCode::OK).await, first);
    {
        let mut tenant = fixture.state.tenants[TENANT].lock().await;
        let id = data_ontology_kernel::EntityTypeId::new("ety_record").unwrap();
        let mut definition = tenant
            .projection()
            .engine
            .entity_type(TENANT, &id)
            .unwrap()
            .clone();
        definition.revision = 2;
        tenant
            .projection_for_injection()
            .engine
            .evolve_entity_type(definition)
            .unwrap();
    }
    let mut changed = query.clone();
    changed["root"]["revision"] = json!(2);
    fixture.walk(&changed, StatusCode::BAD_REQUEST).await;
    changed["idempotency_key"] = json!("new-pin");
    fixture.walk(&changed, StatusCode::OK).await;
    let mut current = objects();
    current[0] = object(TENANT, "ent_record", "ety_team");
    fixture.replace(current, vec![]).await;
    let mut changed = query;
    changed["root"]["entity_type"] = json!("ety_team");
    changed["link_type"] = json!("lty_owns");
    fixture.walk(&changed, StatusCode::BAD_REQUEST).await;
    changed["idempotency_key"] = json!("new-type");
    fixture.walk(&changed, StatusCode::OK).await;
    assert_eq!(
        fixture.state.tenants[TENANT]
            .lock()
            .await
            .queries
            .receipt_count(),
        3
    );
}
