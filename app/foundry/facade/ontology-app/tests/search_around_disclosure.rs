mod facade_support;
#[path = "search_disclosure/faults.rs"]
mod faults;
#[path = "search_disclosure/replay.rs"]
mod replay;
#[path = "search_disclosure/support.rs"]
mod support;

use axum::http::StatusCode;
use serde_json::json;
use support::{GraphFixture, body, links, objects, refusal};

#[tokio::test]
async fn linked_directions_self_edges_and_metadata_remain_canonical() {
    for (direction, expected) in [("outbound", 2), ("inbound", 2), ("both", 3)] {
        let fixture = GraphFixture::new(direction, objects(), links());
        let mut request = body(direction);
        request["direction"] = json!(direction);
        let (status, graph) = fixture.walk(&request).await;
        assert_eq!(status, StatusCode::OK, "{graph}");
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(graph["edges"].as_array().unwrap().len(), expected);
        assert_eq!(graph["truncated"], false);
        for node in graph["nodes"].as_array().unwrap() {
            assert_eq!(node.as_object().unwrap().len(), 2);
        }
        for edge in graph["edges"].as_array().unwrap() {
            assert_eq!(edge.as_object().unwrap().len(), 3);
            match edge["edge_type_id"].as_str().unwrap() {
                "lty_member" => assert_eq!(
                    (
                        edge["from_entity_id"].as_str(),
                        edge["to_entity_id"].as_str()
                    ),
                    (Some("ent_record"), Some("ent_team"))
                ),
                "lty_owns" => assert_eq!(
                    (
                        edge["from_entity_id"].as_str(),
                        edge["to_entity_id"].as_str()
                    ),
                    (Some("ent_team"), Some("ent_record"))
                ),
                "lty_peer" => assert_eq!(edge["from_entity_id"], edge["to_entity_id"]),
                other => panic!("unexpected relationship {other}"),
            }
        }
        assert!(!graph.to_string().contains("private fixture title"));
        assert_eq!(
            fixture.policy.resources.lock().unwrap().as_slice(),
            ["ent_tenant_view", "ent_record", "ent_team"]
        );
        assert_eq!(fixture.counters().await, (1, 0));
    }
}

#[tokio::test]
async fn present_historic_endpoints_must_match_both_declared_roles() {
    for (direction, kind) in [("outbound", "lty_member"), ("inbound", "lty_owns")] {
        let mut wrong = objects();
        wrong[1] = support::object(facade_support::TENANT, "ent_team", "ety_record");
        let fixture = GraphFixture::new("historic-type", wrong, links());
        let mut request = body(direction);
        request["direction"] = json!(direction);
        request["edge_types"] = json!([kind]);
        let (status, reply) = fixture.walk(&request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
        refusal(&reply, "surface");
        assert_eq!(fixture.counters().await, (0, 1));
    }
}

#[tokio::test]
async fn unregistered_historic_relationship_is_not_disclosed() {
    let mut unknown = links();
    unknown[0].link_type = "lty_unregistered".into();
    let fixture = GraphFixture::new("unregistered-edge", objects(), unknown);
    let (status, reply) = fixture.walk(&body("unregistered")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    refusal(&reply, "surface");
    assert_eq!(fixture.counters().await, (0, 1));
}

#[tokio::test]
async fn actual_root_and_neighbor_permissions_gate_disclosure_and_policy_faults_deny() {
    for (resource, outage) in [
        ("ent_record", false),
        ("ent_team", false),
        ("ent_team", true),
    ] {
        let fixture = GraphFixture::new("endpoint-policy", objects(), links());
        if outage {
            *fixture.policy.unavailable.lock().unwrap() = Some(resource);
        } else {
            *fixture.policy.denied.lock().unwrap() = Some(resource);
        }
        let (status, reply) = fixture.walk(&body("endpoint-policy")).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{reply}");
        refusal(&reply, "authorization");
        assert_eq!(fixture.counters().await, (0, 1));
        let tenant = fixture.state.tenants[facade_support::TENANT].lock().await;
        assert_eq!(
            tenant.queries.receipt_count(),
            1,
            "response refusal preserves completed query"
        );
    }
}

#[tokio::test]
async fn empty_consent_still_requires_current_root_read_permission() {
    let fixture = GraphFixture::new("empty-consent", objects(), links());
    let mut request = body("no-grants");
    request["consent"] = json!({"granted":[]});
    let (status, graph) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(graph["edges"], json!([]));
    *fixture.policy.denied.lock().unwrap() = Some("ent_record");
    let (status, reply) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{reply}");
    refusal(&reply, "authorization");
    assert_eq!(fixture.counters().await, (1, 1));
}
