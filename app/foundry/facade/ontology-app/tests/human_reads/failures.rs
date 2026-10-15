use std::sync::Arc;

use axum::http::StatusCode;
use foundry_ontology_app::router_with_user_sessions;

use crate::facade_support::{
    Fixture, Session, TENANT, WRITE_BODY, scrape, value_of, write_a_record,
};
use crate::failing_store::{AlwaysFailingStore, ApplyRefusingStore};
use crate::human_support::{FakeSessions, body, cookie, request, view};

#[tokio::test]
async fn authorized_human_queries_keep_native_pins_pages_filters_and_refusals() {
    let fixture = Fixture::new("human-read-query-contract");
    write_a_record(&fixture, "ent_alpha", "idem_query_alpha").await;
    write_a_record(&fixture, "ent_beta", "idem_query_beta").await;
    let router =
        router_with_user_sessions(Arc::new(fixture.state()), FakeSessions::new(Ok(view())));
    let first = body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects?type=ety_record&revision=1&limit=1",
            &[&cookie()],
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(first["objects"][0]["object_ref"], "ent_alpha");
    assert_eq!(first["next"], "ent_alpha");
    let second = body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects?type=ety_record&revision=1&limit=1&cursor=ent_alpha",
            &[&cookie()],
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(second["objects"][0]["object_ref"], "ent_beta");
    assert_eq!(second["next"], serde_json::Value::Null);
    let filtered = body(request(&router, "/v1/tenants/ten_acme/objects?type=ety_record&revision=1&property=name&equals=text:Absent", &[&cookie()]).await, StatusCode::OK).await;
    assert_eq!(filtered["objects"], serde_json::json!([]));
    for (suffix, expected) in [
        ("/objects/ent_alpha", StatusCode::BAD_REQUEST),
        (
            "/objects/ent_alpha?type=ety_record&revision=invalid",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/objects/ent_alpha?type=ety_record&revision=9",
            StatusCode::CONFLICT,
        ),
        (
            "/objects/ent_missing?type=ety_record&revision=1",
            StatusCode::NOT_FOUND,
        ),
        (
            "/objects?type=ety_missing&revision=1",
            StatusCode::BAD_REQUEST,
        ),
        ("/objects?type=ety_record&revision=9", StatusCode::CONFLICT),
        (
            "/objects?type=ety_record&revision=1&limit=0",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/objects?type=ety_record&revision=1&limit=1001",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/objects?type=ety_record&revision=1&revision=1",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/objects?type=ety_record&revision=1&unknown=x",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/objects?type=ety_record&revision=1&property=name&from=int:1&to=int:2",
            StatusCode::CONFLICT,
        ),
    ] {
        let response = request(
            &router,
            &format!("/v1/tenants/ten_acme{suffix}"),
            &[&cookie()],
        )
        .await;
        assert_eq!(response.status(), expected, "{suffix}");
        let refusal = body(response, expected).await;
        assert_eq!(refusal["gate"], "surface");
    }
}

#[tokio::test]
async fn human_reads_preserve_store_and_log_faults_and_exact_refusal_metrics() {
    let fixture = Fixture::new("human-read-source-faults");
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .unwrap()
        .get_mut()
        .replace_projection_store(Box::new(AlwaysFailingStore {
            detail: "projection unreadable",
        }));
    let shared = Arc::new(state);
    let operator = Session::from_shared(shared.clone());
    let router = router_with_user_sessions(shared, FakeSessions::new(Ok(view())));
    for suffix in [
        "/objects/ent_alpha?type=ety_record&revision=1",
        "/objects?type=ety_record&revision=1",
    ] {
        let before = value_of(&scrape(&operator).await, "foundry_read_refused_total");
        let refusal = body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .await;
        assert_eq!(refusal["gate"], "store");
        assert_eq!(
            value_of(&scrape(&operator).await, "foundry_read_refused_total"),
            before + 1
        );
    }
    let state = crate::failing_log::state_with_a_failing_log(&fixture.config(), "log unreadable");
    let shared = Arc::new(state);
    let operator = Session::from_shared(shared.clone());
    let router = router_with_user_sessions(shared, FakeSessions::new(Ok(view())));
    for suffix in ["/objects/ent_alpha/history", "/audit"] {
        let before = value_of(&scrape(&operator).await, "foundry_read_refused_total");
        let refusal = body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .await;
        assert_eq!(
            refusal,
            serde_json::json!({"gate":"log","cause":"the action log could not be read; this view is unavailable"})
        );
        assert_eq!(
            value_of(&scrape(&operator).await, "foundry_read_refused_total"),
            before + 1
        );
    }
}

#[tokio::test]
async fn human_detail_uses_last_taken_projection_and_history_uses_live_log() {
    let fixture = Fixture::new("human-read-live-store-log");
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .unwrap()
        .get_mut()
        .replace_projection_store(Box::new(ApplyRefusingStore::taking_then_refusing(1, 1)));
    let shared = Arc::new(state);
    let operator = Session::from_shared(shared.clone());
    let router = router_with_user_sessions(shared, FakeSessions::new(Ok(view())));
    assert_eq!(
        operator
            .post(Some(fixture.operator_token()), WRITE_BODY)
            .await
            .0,
        StatusCode::OK
    );
    let later = WRITE_BODY.replace("Ada", "Bob").replace("idem_1", "idem_2");
    assert_eq!(
        operator
            .post(Some(fixture.operator_token()), &later)
            .await
            .0,
        StatusCode::OK
    );
    let detail = body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects/ent_alpha?type=ety_record&revision=1",
            &[&cookie()],
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["properties"]["name"]["value"], "Ada");
    assert_eq!(detail["written_revision"], 1);
    assert_eq!(detail["upcast_state"], "current");
    let history = body(
        request(
            &router,
            "/v1/tenants/ten_acme/objects/ent_alpha/history",
            &[&cookie()],
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(history.as_array().unwrap().len(), 2);
    assert_eq!(history[1]["ordinal"], 2);
    assert_eq!(history[1]["principal_id"], "prn_alice");
    let audit = body(
        request(&router, "/v1/tenants/ten_acme/audit", &[&cookie()]).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(audit.as_array().unwrap().len(), 2);
    assert_eq!(audit[1]["disposition"], "applied");
}
