#![allow(dead_code)]

#[path = "human_reads/denials.rs"]
mod denials;
mod facade_support;
mod failing_log;
mod failing_store;
#[path = "human_reads/failures.rs"]
mod failures;
#[path = "human_status/support.rs"]
mod human_support;
#[path = "human_reads/support.rs"]
mod read_support;
#[path = "human_reads/routes.rs"]
mod routes;

use axum::http::StatusCode;
use std::sync::atomic::Ordering;

use facade_support::{Fixture, get, write_a_record};
use human_support::{FakeSessions, body, cookie, instrument, request, view};

#[tokio::test]
async fn valid_human_reads_match_the_native_operator_results() {
    let fixture = Fixture::new("human-read-results");
    write_a_record(&fixture, "ent_alpha", "idem_human_alpha").await;
    write_a_record(&fixture, "ent_beta", "idem_human_beta").await;
    let sessions = FakeSessions::new(Ok(view()));
    let (router, _, pdp) = instrument(fixture.state(), sessions.clone());
    for suffix in [
        "/types",
        "/objects?type=ety_record&revision=1&limit=1",
        "/objects/ent_alpha?revision=1",
        "/objects/ent_alpha/history",
        "/audit",
    ] {
        let human = body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::OK,
        )
        .await;
        let (status, operator) = get(
            &fixture,
            Some(fixture.operator_token()),
            &format!("/v1{suffix}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            human,
            serde_json::from_str::<serde_json::Value>(&operator).unwrap()
        );
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 5);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn current_schema_read_preserves_explicit_metadata_and_old_type_wire() {
    let fixture = Fixture::new("human-schema-results");
    let sessions = FakeSessions::new(Ok(view()));
    let (router, _, pdp) = instrument(fixture.state(), sessions.clone());
    let schema = body(
        request(&router, "/v1/tenants/ten_acme/schemas", &[&cookie()]).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        schema,
        serde_json::json!([{"entity_type":"ety_record","revision":1,"properties":[
        {"name":"name","tier":"scalar","data_class":"INTERNAL_ONLY","required":true,"value_type":null},
        {"name":"note","tier":"scalar","data_class":"INTERNAL_ONLY","required":false,"value_type":null}
    ],"primary_key_property":null,"title_property":"name"}])
    );
    let (status, types) = get(&fixture, Some(fixture.operator_token()), "/v1/types").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        types,
        r#"[{"entity_type":"ety_record","revision":1,"properties":["name","note"]}]"#
    );
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 1);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 1);
}
