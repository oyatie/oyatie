//! Also covers D8 — admin-plane cross-tenant forbid + accounts scoping (AUTH-005 / ADR-0573):
//! the admin credential must be bound to the configured tenant (never the
//! x-admin-tenant header); cross-tenant path => 403; gate is consulted and
//! fail-closed; /admin/v1/accounts scoped to the verified tenant only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use intelligence_kernel::{
    AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest_proxy::{AppState, PoolRegistry, build_router};
use tower::ServiceExt; // for `oneshot`

mod d4_body_limit_support;
use d4_body_limit_support::*;

// ---------------------------------------------------------------------------
// D8 — admin-plane cross-tenant forbid + accounts scoping (AUTH-005 / ADR-0573)
// ---------------------------------------------------------------------------

/// Forbids every request — used to prove the gate IS consulted and fails closed.
struct RecordingForbidGate {
    called: Arc<Mutex<bool>>,
}
impl AuthzGate for RecordingForbidGate {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        *self.called.lock().unwrap() = true;
        AuthzDecision::Forbid
    }
}

fn make_admin_state(
    admin_tenant: &str,
    admin_bearer: Option<&str>,
    gate: Arc<dyn AuthzGate + Send + Sync>,
) -> Arc<AppState> {
    let tid = TenantId::new(admin_tenant).unwrap();
    let pool = Arc::new(Mutex::new(SubscriptionPool::new(
        tid.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    )));
    let registry = PoolRegistry::new();
    registry.insert_pool(tid.clone(), Provider::Anthropic, Arc::clone(&pool));
    Arc::new(
        AppState::new_with_pool_registry(
            pool,
            registry,
            gate,
            Arc::new(NoopSink),
            Arc::new(StubStore),
            "http://127.0.0.1:1".to_string(),
            tid,
            Some("ingress".to_string()),
            admin_bearer.map(str::to_string),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap(),
    )
}

fn admin_register_request(path_tenant: &str, bearer: Option<&str>) -> Request<Body> {
    let body = format!(
        r#"{{"seat_id":"seat-x","subscription_id":"sub-x","credential_mode":"oauth_subscription","secret_handle":"secret-ref://{path_tenant}/anthropic/seat-x"}}"#
    );
    let mut b = Request::builder()
        .method("POST")
        .uri(format!(
            "/admin/v1/tenants/{path_tenant}/providers/anthropic/subscriptions"
        ))
        .header("content-type", "application/json")
        .header("idempotency-key", "11111111-1111-4111-8111-111111111111");
    if let Some(tok) = bearer {
        b = b.header("authorization", format!("Bearer {tok}"));
    }
    b.body(Body::from(body)).unwrap()
}

/// Cross-tenant IDOR: admin bound to tenant-a must NOT register under tenant-b.
#[tokio::test]
async fn admin_cross_tenant_register_is_forbidden() {
    let state = make_admin_state("tenant-a", Some("admin-a"), Arc::new(AlwaysAllow));
    let resp = build_router(state)
        .oneshot(admin_register_request("tenant-b", Some("admin-a")))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "cross-tenant admin must be 403"
    );
}

/// Missing admin bearer => 401.
#[tokio::test]
async fn admin_missing_bearer_is_unauthorized() {
    let state = make_admin_state("tenant-a", Some("admin-a"), Arc::new(AlwaysAllow));
    let resp = build_router(state)
        .oneshot(admin_register_request("tenant-a", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Wrong admin bearer => 401.
#[tokio::test]
async fn admin_invalid_bearer_is_unauthorized() {
    let state = make_admin_state("tenant-a", Some("admin-a"), Arc::new(AlwaysAllow));
    let resp = build_router(state)
        .oneshot(admin_register_request("tenant-a", Some("wrong-token")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Gate is consulted on admin path; Forbid => 403 (fail-closed).
#[tokio::test]
async fn admin_path_consults_gate_and_fails_closed() {
    let called = Arc::new(Mutex::new(false));
    let state = make_admin_state(
        "tenant-a",
        Some("admin-a"),
        Arc::new(RecordingForbidGate {
            called: Arc::clone(&called),
        }),
    );
    let resp = build_router(state)
        .oneshot(admin_register_request("tenant-a", Some("admin-a")))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "gate Forbid must deny"
    );
    assert!(
        *called.lock().unwrap(),
        "gate must be consulted on admin path"
    );
}

/// /admin/v1/accounts scoped to verified tenant — must not expose tenant-b rows.
#[tokio::test]
async fn admin_accounts_scoped_to_verified_tenant() {
    let tenant_a = TenantId::new("tenant-a").unwrap();
    let tenant_b = TenantId::new("tenant-b").unwrap();
    let pool_a = Arc::new(Mutex::new(SubscriptionPool::new(
        tenant_a.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    )));
    let mut pool_b_inner = SubscriptionPool::new(
        tenant_b.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool_b_inner
        .add_seat(OAuthSubscription::new(
            tenant_b.clone(),
            SeatId::new("seat-b").unwrap(),
            SubscriptionId::new("sub-b").unwrap(),
            Provider::Anthropic,
            SubscriptionState::Active,
            "secret-ref://tenant-b/anthropic/seat-b".to_string(),
            0,
        ))
        .unwrap();
    let pool_b = Arc::new(Mutex::new(pool_b_inner));
    let registry = PoolRegistry::new();
    registry.insert_pool(tenant_a.clone(), Provider::Anthropic, Arc::clone(&pool_a));
    registry.insert_pool(tenant_b.clone(), Provider::Anthropic, Arc::clone(&pool_b));
    let state = Arc::new(
        AppState::new_with_pool_registry(
            pool_a,
            registry,
            Arc::new(AlwaysAllow),
            Arc::new(NoopSink),
            Arc::new(StubStore),
            "http://127.0.0.1:1".to_string(),
            tenant_a,
            Some("ingress".to_string()),
            Some("admin-a".to_string()),
            "development".to_string(),
            std::collections::HashSet::new(),
        )
        .unwrap(),
    );
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/v1/accounts")
                .header("authorization", "Bearer admin-a")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let s = std::str::from_utf8(&body).unwrap();
    assert!(
        !s.contains("tenant-b"),
        "admin-a must not see tenant-b accounts; got: {s}"
    );
}
