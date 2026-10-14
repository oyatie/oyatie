use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use iam_identity_user_session::SessionRefusal;
use serde_json::json;
use std::sync::atomic::Ordering;
use tower::ServiceExt;

use crate::facade_support::TENANT;
use crate::human_support::{body, cookie, view};
use crate::support::{HumanGraph, request};

#[tokio::test]
async fn explicit_direction_and_single_relationship_never_widen_to_wildcard() {
    let fixture = HumanGraph::new("human-direction");
    for (direction, link, from, to) in [
        ("outbound", "lty_member", "ent_record", "ent_team"),
        ("inbound", "lty_owns", "ent_team", "ent_record"),
        ("outbound", "lty_peer", "ent_record", "ent_record"),
    ] {
        let mut query = request(link);
        query["link_type"] = json!(link);
        query["direction"] = json!(direction);
        let graph = fixture.walk(&query, StatusCode::OK).await;
        assert_eq!(
            graph["edges"],
            json!([{"from_entity_id":from,"to_entity_id":to,"edge_type_id":link}])
        );
    }
    let mut absent = request("absent");
    absent["root"]["object_ref"] = json!("ent_missing");
    fixture.walk(&absent, StatusCode::BAD_REQUEST).await;
    let mut wrong = request("wrong");
    wrong["root"]["object_ref"] = json!("ent_team");
    fixture.walk(&wrong, StatusCode::BAD_REQUEST).await;
    wrong = request("role");
    wrong["link_type"] = json!("lty_owns");
    fixture.walk(&wrong, StatusCode::BAD_REQUEST).await;
    wrong = request("pin");
    wrong["root"]["revision"] = json!(9);
    let reply = fixture.walk(&wrong, StatusCode::CONFLICT).await;
    assert_eq!(
        reply["cause"],
        "that revision was never accepted for this entity type"
    );
}

#[tokio::test]
async fn strict_bounded_human_body_refuses_grants_depth_wildcards_and_unknown_fields() {
    let fixture = HumanGraph::new("human-grammar");
    let mut invalid = Vec::new();
    for (field, value) in [
        ("edge_types", json!([])),
        ("consent", json!("unrestricted")),
        ("max_depth", json!(2)),
        ("tenant_id", json!(TENANT)),
        ("principal_id", json!("usr_admin")),
        ("direction", json!("both")),
        ("link_type", json!("")),
        ("idempotency_key", json!("has space")),
        ("idempotency_key", json!("x".repeat(129))),
        ("observed_at_epoch_seconds", json!(9007199254740992_u64)),
        ("observed_at_epoch_seconds", json!(-1)),
    ] {
        let mut query = request("bad");
        query[field] = value;
        invalid.push(query);
    }
    for (field, value) in [
        ("extra", json!(true)),
        ("revision", json!(1.2)),
        ("revision", json!(4294967296_u64)),
        ("entity_type", json!(" ety_record")),
        ("object_ref", json!("ent_\nsecret")),
        ("object_ref", json!(format!("ent_{}", "x".repeat(253)))),
    ] {
        let mut query = request("bad");
        query["root"][field] = value;
        invalid.push(query);
    }
    invalid.push(json!([
        "bad",
        ["ety_record", "ent_record", 1],
        "lty_member",
        "outbound",
        1700000000
    ]));
    let mut sequence = request("sequence");
    sequence["root"] = json!(["ety_record", "ent_record", 1]);
    invalid.push(sequence);
    invalid.push(json!([]));
    invalid.push(json!(null));
    for query in invalid {
        fixture.walk(&query, StatusCode::BAD_REQUEST).await;
    }
    assert_eq!(
        fixture.state.tenants[TENANT]
            .lock()
            .await
            .queries
            .receipt_count(),
        0
    );
}

#[tokio::test]
async fn cookie_membership_and_policy_refuse_before_the_query_and_on_replay() {
    let fixture = HumanGraph::new("human-session");
    for tenant in [TENANT, "ten_other"] {
        let response = fixture
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/tenants/{tenant}/search-around"))
                    .header(
                        "authorization",
                        format!("Bearer {}", fixture.fixture.operator_token()),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from("not json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        body(response, StatusCode::UNAUTHORIZED).await;
    }
    assert_eq!(fixture.sessions.calls.load(Ordering::SeqCst), 0);
    let query = request("session");
    fixture.walk(&query, StatusCode::OK).await;
    for (refusal, status) in [
        (SessionRefusal::Expired, StatusCode::UNAUTHORIZED),
        (SessionRefusal::InactiveMembership, StatusCode::FORBIDDEN),
        (
            SessionRefusal::MembershipUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    ] {
        *fixture.sessions.answer.lock().unwrap() = Err(refusal);
        fixture.walk(&query, status).await;
    }
    let mut revoked = view();
    revoked.roles.clear();
    *fixture.sessions.answer.lock().unwrap() = Ok(revoked);
    fixture.walk(&query, StatusCode::FORBIDDEN).await;
    *fixture.sessions.answer.lock().unwrap() = Ok(view());
    let response = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/tenants/ten_other/search-around")
                .header("cookie", cookie())
                .header("content-type", "application/json")
                .body(Body::from(query.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    body(response, StatusCode::FORBIDDEN).await;
}

#[tokio::test]
async fn json_protocol_and_stream_size_are_bounded_after_identity() {
    let fixture = HumanGraph::new("human-protocol");
    for (content_type, text, status) in [
        (
            "text/plain",
            request("plain").to_string(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "application/json",
            " ".repeat(4097),
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        ("application/json", "[".into(), StatusCode::BAD_REQUEST),
    ] {
        let response = fixture
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/tenants/ten_acme/search-around")
                    .header("cookie", cookie())
                    .header("content-type", content_type)
                    .body(Body::from(text))
                    .unwrap(),
            )
            .await
            .unwrap();
        body(response, status).await;
    }
}

#[tokio::test]
async fn denied_existing_and_absent_roots_have_identical_refusals_before_store_lookup() {
    let fixture = HumanGraph::new("human-root-disclosure");
    let mut replies = Vec::new();
    for root in ["ent_record", "ent_missing"] {
        *fixture.policy.denied.lock().unwrap() = Some(root);
        let mut query = request(root);
        query["root"]["object_ref"] = json!(root);
        replies.push(fixture.walk(&query, StatusCode::FORBIDDEN).await);
    }
    assert_eq!(replies[0], replies[1]);
    assert_eq!(
        fixture.state.tenants[TENANT]
            .lock()
            .await
            .queries
            .receipt_count(),
        0
    );
    let metrics = crate::facade_support::scrape(&crate::facade_support::Session::from_shared(
        fixture.state.clone(),
    ))
    .await;
    assert_eq!(
        crate::facade_support::value_of(&metrics, "foundry_read_refused_total"),
        2
    );
    assert_eq!(
        crate::facade_support::value_of(&metrics, "foundry_read_served_total"),
        0
    );
}
