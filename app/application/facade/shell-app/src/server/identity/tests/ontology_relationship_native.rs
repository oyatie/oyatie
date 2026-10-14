use super::{ontology_native::NativeFixture, support::*};
use axum::{
    body::Body,
    http::{Method, StatusCode, header},
};
use foundry_model_api::ModelBaseline;
use foundry_projection_draft::{AppliedEntry, EntryOutcome, KeyDesignations, ProjectedLink};
use serde_json::json;

#[tokio::test]
async fn real_human_relationship_proxy_traverses_durable_links_and_rechecks_iam() {
    let iam = Fixture::new(&session()).await;
    let native = NativeFixture::new(&iam).await;
    {
        let mut tenant = native.state.tenants["ten_acme"].lock().await;
        let mut baseline = ModelBaseline::capture(&tenant.projection.engine, "ten_acme").unwrap();
        baseline
            .prior_entity_revisions
            .push(baseline.document.entities[0].clone());
        baseline.document.entities[0].revision = 2;
        baseline.document.links.push(serde_json::from_value(json!({"id":"lty_related","from_entity_type":"ety_record","to_entity_type":"ety_record","cardinality":"one_to_many","allow_cross_tenant":false,"revision":1,"display":null})).unwrap());
        tenant.projection.engine = baseline.registry("ten_acme").unwrap();
        let mut target = tenant
            .projection_store
            .get("ten_acme", "ent_alpha")
            .unwrap()
            .unwrap();
        target.entity.id = "ent_beta".into();
        target.schema_revision = 2;
        target.last_ordinal = 2;
        tenant
            .projection_store
            .apply(
                AppliedEntry {
                    tenant_id: "ten_acme".into(),
                    ordinal: 2,
                    outcome: EntryOutcome::Applied {
                        objects: vec![target],
                        links: vec![ProjectedLink {
                            link_type: "lty_related".into(),
                            from_object_ref: "ent_alpha".into(),
                            to_object_ref: "ent_beta".into(),
                            observed_at_epoch_ms: 1700000000000,
                        }],
                    },
                },
                &KeyDesignations::default(),
            )
            .unwrap();
    }
    let shell = native.shell(&iam);
    let query = json!({"idempotency_key":"real-linked","root":{"entity_type":"ety_record","object_ref":"ent_alpha","revision":1},"link_type":"lty_related","direction":"outbound","observed_at_epoch_seconds":1700000000});
    let response = call(
        shell.clone(),
        Method::POST,
        "/api/ontology/ten_acme/search-around",
        headers(),
        query.to_string(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    assert_eq!(response.headers()["x-oyatie-tenant"], "ten_acme");
    assert_eq!(response.headers()["x-oyatie-user"], "usr_alice");
    let graph = json(response).await;
    assert_eq!(
        graph,
        json!({"nodes":[{"entity_id":"ent_alpha","entity_type_id":"ety_record","written_revision":1},{"entity_id":"ent_beta","entity_type_id":"ety_record","written_revision":2}],"edges":[{"from_entity_id":"ent_alpha","to_entity_id":"ent_beta","edge_type_id":"lty_related"}],"observed_at_epoch_seconds":1700000000,"truncated":false})
    );
    let response = call(
        shell.clone(),
        Method::GET,
        "/api/ontology/ten_acme/objects/ent_beta?revision=2",
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&response, StatusCode::OK);
    let detail = json(response).await;
    assert_eq!(detail["requested_revision"], 2);
    assert_eq!(detail["object"]["written_revision"], 2);
    assert_eq!(detail["object"]["properties"]["name"]["value"], "Ada");
    assert_eq!(
        native.state.tenants["ten_acme"]
            .lock()
            .await
            .queries
            .receipt_count(),
        1
    );
    iam.reply.lock().unwrap().body = session().replace(r#"["foundry-operator"]"#, "[]");
    let response = call(
        shell,
        Method::POST,
        "/api/ontology/ten_acme/search-around",
        headers(),
        query.to_string(),
    )
    .await;
    expect_no_store(&response, StatusCode::FORBIDDEN);
    assert_eq!(json(response).await, json!({"error":"forbidden"}));
    let calls = iam.calls.lock().unwrap();
    assert_eq!(calls.len(), 7);
    for (path, headers, body) in calls.iter() {
        assert_eq!(path, "/identity/v1/ten_acme/session");
        assert_eq!(headers[header::COOKIE], cookie());
        assert!(!headers.contains_key(header::AUTHORIZATION));
        assert!(body.is_empty());
    }
    println!(
        "Real Application HTTP/proxy → Foundry human session/PDP → durable relationship traversal and own-revision detail PASS"
    );
}
