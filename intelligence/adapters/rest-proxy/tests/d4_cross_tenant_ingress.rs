//! Also covers D7 — data-plane cross-tenant ingress forbid (AUTH-005 / ADR-0573):
//! the REST boundary must feed the VERIFIED principal's tenant to the gate so the
//! cross-tenant Cedar forbid actually fires.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use intelligence_kernel::{
    AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest_proxy::{
    AppState, ConfiguredBearerIngressAuthenticator, PoolRegistry, build_router,
};
use scripted_http_server::{ScriptedResponse, ScriptedServer};
use tower::ServiceExt; // for `oneshot`

mod d4_body_limit_support;
use d4_body_limit_support::*;

// ---------------------------------------------------------------------------
// D7 — cross-tenant ingress forbid (AUTH-005 / ADR-0573)
// ---------------------------------------------------------------------------

/// Deny-wins gate: forbids when principal tenant differs from resource tenant.
/// Mirrors the Cedar cross-tenant forbid without pulling the cedar adapter in.
struct CrossTenantForbidGate;
impl AuthzGate for CrossTenantForbidGate {
    fn decide(&self, request: &AuthzRequest<'_>) -> AuthzDecision {
        if request.principal_tenant != request.resource_tenant {
            AuthzDecision::Forbid
        } else {
            AuthzDecision::Allow
        }
    }
}

fn make_pool_for(tenant: &str) -> Arc<Mutex<SubscriptionPool>> {
    let t = TenantId::new(tenant).unwrap();
    let mut pool = SubscriptionPool::new(
        t.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(OAuthSubscription::new(
        t.clone(),
        SeatId::new("seat-1").unwrap(),
        SubscriptionId::new("sub-1").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("{tenant}/seat-1"),
        0,
    ))
    .unwrap();
    Arc::new(Mutex::new(pool))
}

fn make_cross_tenant_state(base_url: String, service_tenant: &str) -> AppState {
    let tid = TenantId::new(service_tenant).unwrap();
    let pool = make_pool_for(service_tenant);
    let registry = PoolRegistry::new();
    registry.insert_pool(tid.clone(), Provider::Anthropic, Arc::clone(&pool));
    AppState::new_with_pool_registry(
        pool,
        registry,
        Arc::new(CrossTenantForbidGate),
        Arc::new(NoopSink),
        Arc::new(StubStore),
        base_url,
        tid,
        None,
        None,
        "development".to_string(),
        std::collections::HashSet::new(),
    )
    .unwrap()
}

fn messages_request(bearer: Option<&str>) -> Request<Body> {
    const BODY: &str = r#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#;
    let mut b = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .header("x-agent-id", "agent-x");
    if let Some(tok) = bearer {
        b = b.header("authorization", format!("Bearer {tok}"));
    }
    b.body(Body::from(BODY)).unwrap()
}

/// Cross-tenant: a bearer bound to tenant-b on a tenant-a-only instance mints
/// the VERIFIED tenant-b principal. AUTH-005 increment-3 keys the data-plane
/// pool by the verified principal tenant, so the request resolves to tenant-b's
/// pool — which is absent here => 503. It can never reach tenant-a's seats.
/// Cross-tenant isolation now lives in pool-keying + the kernel backstop (the
/// 403 deny-wins path is covered by the mis-keyed-pool fixture in d2 / d7); the
/// edge no longer 403s on the tenant axis (principal == resource == tenant-b).
#[tokio::test]
async fn cross_tenant_ingress_principal_gets_no_seat() {
    // Empty script: any upstream call at all is unexpected here, and the trace below
    // asserts none happened — which the httpmock original never checked.
    let server = ScriptedServer::start(vec![]);
    let state = make_cross_tenant_state(server.base_url().to_owned(), "tenant-a")
        .with_ingress_authenticator(Arc::new(ConfiguredBearerIngressAuthenticator::new(
            "ingress-b",
            TenantId::new("tenant-b").unwrap(),
        )));
    let app = build_router(Arc::new(state));
    let resp = app
        .oneshot(messages_request(Some("ingress-b")))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "cross-tenant principal has no pool on this instance => 503, never a tenant-a seat"
    );
}

/// Same-tenant: bearer bound to tenant-a on the tenant-a service => 200.
#[tokio::test]
async fn same_tenant_ingress_principal_is_allowed() {
    let server = ScriptedServer::start(vec![
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok","refresh_token":"rt2","expires_in":3600}"#),
        ScriptedResponse::ok()
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-1","type":"message"}"#),
    ]);
    let state = make_cross_tenant_state(server.base_url().to_owned(), "tenant-a")
        .with_ingress_authenticator(Arc::new(ConfiguredBearerIngressAuthenticator::new(
            "ingress-a",
            TenantId::new("tenant-a").unwrap(),
        )));
    let app = build_router(Arc::new(state));
    let resp = app
        .oneshot(messages_request(Some("ingress-a")))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "same-tenant must reach lease/proxy"
    );
    // The httpmock originals bound these two mocks to `_token` / `_msg` and never
    // asserted either one, so "must reach lease/proxy" was carried entirely by the
    // status code. Assert the upstream calls themselves.
    assert_eq!(
        server.request_lines(),
        vec!["POST /v1/oauth/token", "POST /v1/messages"],
        "same-tenant must exchange the token and then proxy upstream"
    );
}

/// No bearer => 401 (default-deny before authz).
#[tokio::test]
async fn absent_bearer_is_unauthorized() {
    // Empty script: any upstream call at all is unexpected here, and the trace below
    // asserts none happened — which the httpmock original never checked.
    let server = ScriptedServer::start(vec![]);
    let state = make_cross_tenant_state(server.base_url().to_owned(), "tenant-a")
        .with_ingress_authenticator(Arc::new(ConfiguredBearerIngressAuthenticator::new(
            "ingress-a",
            TenantId::new("tenant-a").unwrap(),
        )));
    let app = build_router(Arc::new(state));
    let resp = app.oneshot(messages_request(None)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Wrong bearer => 401 (constant-time compare fails).
#[tokio::test]
async fn wrong_bearer_is_unauthorized() {
    // Empty script: any upstream call at all is unexpected here, and the trace below
    // asserts none happened — which the httpmock original never checked.
    let server = ScriptedServer::start(vec![]);
    let state = make_cross_tenant_state(server.base_url().to_owned(), "tenant-a")
        .with_ingress_authenticator(Arc::new(ConfiguredBearerIngressAuthenticator::new(
            "ingress-a",
            TenantId::new("tenant-a").unwrap(),
        )));
    let app = build_router(Arc::new(state));
    let resp = app
        .oneshot(messages_request(Some("wrong-token")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
