use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::facade_support::Fixture;
use crate::human_support::{FakeSessions, body, cookie, request, view};
use crate::read_support::SUFFIXES;

#[tokio::test]
async fn seed_link_catalog_is_empty_and_each_read_rechecks_human_membership() {
    let fixture = Fixture::new("human-link-catalog");
    let sessions = FakeSessions::new(Ok(view()));
    let enabled = foundry_ontology_app::router_with_user_sessions(
        Arc::new(fixture.state()),
        sessions.clone(),
    );
    for _ in 0..2 {
        assert_eq!(
            body(
                request(&enabled, "/v1/tenants/ten_acme/link-types", &[&cookie()]).await,
                StatusCode::OK
            )
            .await,
            serde_json::json!([])
        );
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.log_head(), 0);
}

#[tokio::test]
async fn model_catalog_refusals_precede_access_to_the_native_registry() {
    let fixture = Fixture::new("human-link-guard-before-registry");
    let sessions = FakeSessions::new(Ok(view()));
    let state = Arc::new(fixture.state());
    let _held_registry = state.tenants.get("ten_acme").unwrap().lock().await;
    let router = foundry_ontology_app::router_with_user_sessions(state.clone(), sessions.clone());
    for (member, expected) in [
        (view(), StatusCode::UNAUTHORIZED),
        (
            {
                let mut member = view();
                member.roles.clear();
                member
            },
            StatusCode::FORBIDDEN,
        ),
        (
            {
                let mut member = view();
                member.tenant_id = "ten_other".to_owned();
                member
            },
            StatusCode::FORBIDDEN,
        ),
    ] {
        *sessions.answer.lock().unwrap() = Ok(member);
        let cookies = if expected == StatusCode::UNAUTHORIZED {
            vec![]
        } else {
            vec![cookie()]
        };
        let values: Vec<_> = cookies.iter().map(String::as_str).collect();
        for suffix in ["link-types", "type-labels"] {
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                request(&router, &format!("/v1/tenants/ten_acme/{suffix}"), &values),
            )
            .await
            .expect("a refused read must not wait for the held registry");
            body(response, expected).await;
        }
    }
}

#[tokio::test]
async fn native_record_label_rechecks_membership_and_keeps_the_declared_revision() {
    let fixture = Fixture::new("human-native-type-labels");
    let sessions = FakeSessions::new(Ok(view()));
    let router = foundry_ontology_app::router_with_user_sessions(
        Arc::new(fixture.state()),
        sessions.clone(),
    );
    for _ in 0..2 {
        assert_eq!(
            body(
                request(&router, "/v1/tenants/ten_acme/type-labels", &[&cookie()]).await,
                StatusCode::OK
            )
            .await,
            serde_json::json!([{"entity_type":"ety_record","revision":1,"display_name":"Record"}])
        );
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.log_head(), 0);
}

#[tokio::test]
async fn default_router_keeps_human_reads_disabled_and_human_routes_offer_no_writes() {
    let fixture = Fixture::new("human-read-disabled");
    let disabled = foundry_ontology_app::router(fixture.state());
    for suffix in SUFFIXES {
        assert_eq!(
            request(
                &disabled,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()]
            )
            .await
            .status(),
            StatusCode::NOT_FOUND
        );
    }
    let sessions = FakeSessions::new(Ok(view()));
    let enabled = foundry_ontology_app::router_with_user_sessions(
        Arc::new(fixture.state()),
        sessions.clone(),
    );
    for suffix in SUFFIXES {
        let response = enabled
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/tenants/ten_acme{suffix}"))
                    .header("cookie", cookie())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers()["cache-control"], "no-store");
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.log_head(), 0);
}
