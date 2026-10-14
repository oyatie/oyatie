use axum::http::StatusCode;
use foundry_projection_draft::{AppliedEntry, EntryOutcome, KeyDesignations};
use serde_json::json;

use crate::facade_support::TENANT;
use crate::support::{GraphFixture, body, links, object, objects, refusal};

#[tokio::test]
async fn cached_receipt_rechecks_endpoint_policy_without_rewriting_query_or_projection() {
    let fixture = GraphFixture::new("cached-policy", objects(), links());
    let request = body("cached-policy");
    let (status, graph) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    let before = {
        let tenant = fixture.state.tenants[TENANT].lock().await;
        (
            tenant.queries.clone(),
            tenant.query_decisions.clone(),
            tenant.projection.clone(),
        )
    };
    *fixture.policy.denied.lock().unwrap() = Some("ent_team");
    let (status, reply) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{reply}");
    refusal(&reply, "authorization");
    *fixture.policy.denied.lock().unwrap() = None;
    let (status, replay) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(graph, replay);
    let tenant = fixture.state.tenants[TENANT].lock().await;
    assert_eq!(
        (
            tenant.queries.clone(),
            tenant.query_decisions.clone(),
            tenant.projection.clone()
        ),
        before
    );
    assert_eq!(fixture.fixture.log_head(), 0);
    assert_eq!(fixture.fixture.denial_head(), 0);
    drop(tenant);
    assert_eq!(fixture.counters().await, (2, 1));
}

#[tokio::test]
async fn cached_graph_rechecks_nonroot_existence_type_and_current_canonical_edge() {
    for changed in ["missing-neighbor", "wrong-neighbor-type", "missing-edge"] {
        let fixture = GraphFixture::new(changed, objects(), links());
        let request = body(changed);
        let (status, graph) = fixture.walk(&request).await;
        assert_eq!(status, StatusCode::OK, "{graph}");
        let before = {
            let tenant = fixture.state.tenants[TENANT].lock().await;
            (
                tenant.queries.clone(),
                tenant.query_decisions.clone(),
                tenant.projection.clone(),
            )
        };
        let mut current = objects();
        if changed == "missing-neighbor" {
            current.pop();
        }
        if changed == "wrong-neighbor-type" {
            current[1] = object(TENANT, "ent_team", "ety_record");
        }
        fixture
            .replace(
                current,
                if changed == "missing-edge" {
                    vec![]
                } else {
                    links()
                },
            )
            .await;
        let (status, reply) = fixture.walk(&request).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{changed}: {reply}");
        refusal(&reply, "surface");
        let tenant = fixture.state.tenants[TENANT].lock().await;
        assert_eq!(
            (
                tenant.queries.clone(),
                tenant.query_decisions.clone(),
                tenant.projection.clone()
            ),
            before
        );
        assert_eq!(fixture.fixture.log_head(), 0);
        assert_eq!(fixture.fixture.denial_head(), 0);
        drop(tenant);
        assert_eq!(fixture.counters().await, (1, 1));
    }
}

#[tokio::test]
async fn foreign_only_endpoint_cannot_satisfy_a_cached_tenant_graph() {
    let fixture = GraphFixture::new("cached-foreign", objects(), links());
    let request = body("foreign");
    assert_eq!(fixture.walk(&request).await.0, StatusCode::OK);
    fixture
        .replace(vec![object(TENANT, "ent_record", "ety_record")], links())
        .await;
    {
        let mut tenant = fixture.state.tenants[TENANT].lock().await;
        tenant
            .projection_store
            .apply(
                AppliedEntry {
                    tenant_id: "ten_other".into(),
                    ordinal: 1,
                    outcome: EntryOutcome::Applied {
                        objects: vec![object("ten_other", "ent_team", "ety_team")],
                        links: vec![],
                    },
                },
                &KeyDesignations::default(),
            )
            .unwrap();
        assert!(
            tenant
                .projection_store
                .get("ten_other", "ent_team")
                .unwrap()
                .is_some()
        );
    }
    let (status, reply) = fixture.walk(&request).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    refusal(&reply, "surface");
    assert_eq!(fixture.counters().await, (1, 1));
}

#[tokio::test]
async fn changed_intent_keeps_native_conflict_and_excluded_edges_stay_excluded() {
    let fixture = GraphFixture::new("conflict-control", objects(), links());
    let request = body("conflict");
    assert_eq!(fixture.walk(&request).await.0, StatusCode::OK);
    let mut changed = request.clone();
    changed["direction"] = json!("inbound");
    let (status, reply) = fixture.walk(&changed).await;
    assert_eq!(status, StatusCode::CONFLICT, "{reply}");
    refusal(&reply, "surface");
    *fixture.policy.denied.lock().unwrap() = Some("ent_team");
    changed["idempotency_key"] = json!("excluded");
    changed["consent"] = json!({"granted":["lty_peer"]});
    let (status, graph) = fixture.walk(&changed).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "excluded neighbor requires no disclosure: {graph}"
    );
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(graph["edges"].as_array().unwrap().len(), 1);
    assert_eq!(fixture.counters().await, (2, 1));
}
