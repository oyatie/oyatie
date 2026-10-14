#![allow(dead_code)]

mod facade_support;
#[path = "human_point/grammar.rs"]
mod grammar;
#[path = "human_status/support.rs"]
mod human_support;
#[path = "human_point/support.rs"]
mod point_support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use foundry_ontology_app::router_with_user_sessions;

use facade_support::{Fixture, Session, TENANT, scrape, value_of, write_a_record};
use human_support::{FakeSessions, body, cookie, now, request, view};
use point_support::{ChangingStore, declare, point, send};

#[tokio::test]
async fn a_human_point_read_refuses_another_expected_type() {
    let fixture = Fixture::new("human-point-type-mismatch");
    write_a_record(&fixture, "ent_alpha", "idem_point_type").await;
    let router =
        router_with_user_sessions(Arc::new(fixture.state()), FakeSessions::new(Ok(view())));
    body(
        request(&router, &point("ety_other", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::NOT_FOUND,
    )
    .await;
}

#[tokio::test]
async fn an_admitted_unlinked_retype_cannot_replace_the_selected_types_detail() {
    let fixture = Fixture::new("human-point-admitted-retype");
    let mut state = fixture.state();
    declare(&mut state, "ety_other", "aty_other", "other_name");
    let state = Arc::new(state);
    let router = router_with_user_sessions(state.clone(), FakeSessions::new(Ok(view())));
    send(
        &state,
        "ety_record",
        "aty_record_write",
        "name",
        "Ada",
        "first",
    )
    .await;
    let listed = body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects?type=ety_record&revision=1",
            &[&cookie()],
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["objects"][0]["object_ref"], "ent_alpha");
    let before = body(
        request(&router, &point("ety_record", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(before["properties"]["name"]["value"], "Ada");
    send(
        &state,
        "ety_other",
        "aty_other",
        "other_name",
        "B-private",
        "retype",
    )
    .await;
    let mismatched = body(
        request(&router, &point("ety_record", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let absent = body(
        request(&router, &point("ety_record", "ent_absent", 1), &[&cookie()]).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(mismatched, absent);
    assert_eq!(
        body(
            request(&router, &point("ety_record", "ent_alpha", 9), &[&cookie()]).await,
            StatusCode::NOT_FOUND
        )
        .await,
        absent
    );
    let current = body(
        request(&router, &point("ety_other", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::OK,
    )
    .await;
    assert!(current["properties"].get("name").is_none());
    assert_eq!(current["properties"]["other_name"]["value"], "B-private");
    body(
        request(&router, &point("ety_other", "ent_alpha", 9), &[&cookie()]).await,
        StatusCode::CONFLICT,
    )
    .await;
    let operator = Session::from_shared(state);
    let (status, legacy) = operator
        .get(
            Some(fixture.operator_token()),
            "/v1/objects/ent_alpha?revision=1",
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&legacy).unwrap(),
        current
    );
    assert_eq!(
        fixture.log_head(),
        2,
        "the actual writer admitted both writes"
    );
}

#[tokio::test]
async fn type_and_fields_are_derived_from_the_same_single_store_value() {
    let fixture = Fixture::new("human-point-one-store-value");
    let mut state = fixture.state();
    declare(&mut state, "ety_other", "aty_other", "other_name");
    let state = Arc::new(state);
    send(
        &state,
        "ety_record",
        "aty_record_write",
        "name",
        "A",
        "first",
    )
    .await;
    let original = state.tenants[TENANT]
        .lock()
        .await
        .projection_store
        .get(TENANT, "ent_alpha")
        .unwrap()
        .unwrap();
    send(
        &state,
        "ety_other",
        "aty_other",
        "other_name",
        "B",
        "retype",
    )
    .await;
    let mut state = Arc::try_unwrap(state).ok().unwrap();
    let reads = ChangingStore::install(&mut state, Some(original), std::time::Duration::ZERO);
    let router = router_with_user_sessions(Arc::new(state), FakeSessions::new(Ok(view())));
    let detail = body(
        request(&router, &point("ety_record", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    assert_eq!(detail["properties"]["name"]["value"], "A");
    assert!(detail["properties"].get("other_name").is_none());
    body(
        request(&router, &point("ety_record", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(reads.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn an_expiry_during_the_point_read_cannot_count_or_serve_a_payload() {
    let fixture = Fixture::new("human-point-inflight-expiry");
    write_a_record(&fixture, "ent_alpha", "idem_expiry").await;
    let mut state = fixture.state();
    let reads = ChangingStore::install(&mut state, None, std::time::Duration::from_secs(3));
    let state = Arc::new(state);
    let operator = Session::from_shared(state.clone());
    let initial = scrape(&operator).await;
    let mut membership = view();
    membership.expires_at = now() + 2;
    let router = router_with_user_sessions(state, FakeSessions::new(Ok(membership)));
    let refused = body(
        request(&router, &point("ety_record", "ent_alpha", 1), &[&cookie()]).await,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_eq!(refused["gate"], "session");
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    let after = scrape(&operator).await;
    assert_eq!(
        value_of(&after, "foundry_read_served_total"),
        value_of(&initial, "foundry_read_served_total")
    );
    assert_eq!(
        value_of(&after, "foundry_read_refused_total"),
        value_of(&initial, "foundry_read_refused_total") + 1
    );
}
