use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::facade_support::Fixture;
use crate::human_support::{FakeSessions, cookie, request, view};
use crate::read_support::SUFFIXES;

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
