//! Fix-4: Body-size limit — POST /v1/messages with body > 1 MiB receives 413.
//!
//! Also covers D7 — data-plane cross-tenant ingress forbid (AUTH-005 / ADR-0573):
//! the REST boundary must feed the VERIFIED principal's tenant to the gate so the
//! cross-tenant Cedar forbid actually fires.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use httpmock::prelude::*;
use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{
    AppState, ConfiguredBearerIngressAuthenticator, EventSink, LlmGatewayEvent, PoolRegistry,
    RestAdapterError, SecretProviderStore, TokenRefreshSingleflight, build_router,
};
use tower::ServiceExt; // for `oneshot`

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct AlwaysAllow;
impl intelligence_kernel::AuthzGate for AlwaysAllow {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

struct NoopSink;
impl EventSink for NoopSink {
    fn emit(&self, _: LlmGatewayEvent) {}
}

struct StubStore;
impl SecretProviderStore for StubStore {
    fn fetch_refresh_token(&self, _: &str) -> Result<String, RestAdapterError> {
        Ok("stub-token".to_string())
    }
    fn store_refresh_token(&self, _: &str, _: &str) -> Result<(), RestAdapterError> {
        Ok(())
    }
}

fn make_state() -> Arc<AppState> {
    let tenant = TenantId::new("t-body-limit").unwrap();
    let mut pool = SubscriptionPool::new(
        tenant.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(OAuthSubscription::new(
        tenant.clone(),
        SeatId::new("seat-bl-1").unwrap(),
        SubscriptionId::new("sub-bl-1").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        "t-body-limit/seat-bl-1".to_string(),
        0,
    ))
    .unwrap();
    Arc::new(AppState {
        pool: Arc::new(Mutex::new(pool)),
        pool_registry: PoolRegistry::new(),
        gate: Arc::new(AlwaysAllow),
        sink: Arc::new(NoopSink),
        secret_store: Arc::new(StubStore),
        anthropic_base_url: "http://127.0.0.1:1".to_string(),
        openai_compatible_base_url: "http://127.0.0.1:1".to_string(),
        codex_oauth_base_url: "http://127.0.0.1:1".to_string(),
        gemini_base_url: "http://127.0.0.1:1".to_string(),
        tenant_id: tenant.clone(),
        ingress_authenticator: Arc::new(ConfiguredBearerIngressAuthenticator::new(
            "ingress-token",
            tenant,
        )),
        admin_bearer_token: None,
        environment: "test".to_string(),
        oauth_approved_providers: std::collections::HashSet::new(),
        token_singleflight: Arc::new(TokenRefreshSingleflight::new()),
        http_client: Arc::new(reqwest::Client::new()),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// POST /v1/messages with a body larger than 1 MiB must return 413.
#[tokio::test]
async fn post_exceeding_1mib_returns_413() {
    let state = make_state();
    let router = build_router(state);

    // 1 MiB + 1 byte.
    let oversized = Bytes::from(vec![b'x'; 1024 * 1024 + 1]);
    let req = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("authorization", "Bearer ingress-token")
        .header("content-type", "application/json")
        .header("x-agent-id", "agent-limit-test")
        .body(Body::from(oversized))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "expected 413 for oversized body"
    );
}

/// POST /v1/messages with a body exactly at the limit (1 MiB) must NOT return 413.
/// (It will fail for another reason since the upstream is unreachable, but not 413.)
#[tokio::test]
async fn post_at_exactly_1mib_does_not_return_413() {
    let state = make_state();
    let router = build_router(state);

    let at_limit = Bytes::from(vec![b'x'; 1024 * 1024]);
    let req = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("authorization", "Bearer ingress-token")
        .header("content-type", "application/json")
        .header("x-agent-id", "agent-limit-test")
        .body(Body::from(at_limit))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "1 MiB body should not be rejected"
    );
}

/// GET /healthz is not subject to body limit rejections.
#[tokio::test]
async fn healthz_returns_200() {
    let state = make_state();
    let router = build_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

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
    let mut pool = SubscriptionPool::new(t.clone(), Provider::Anthropic, SelectionStrategy::RoundRobin);
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

/// Cross-tenant: bearer bound to tenant-b on the tenant-a service => 403.
#[tokio::test]
async fn cross_tenant_ingress_principal_is_forbidden() {
    let server = MockServer::start();
    let state =
        make_cross_tenant_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
            ConfiguredBearerIngressAuthenticator::new("ingress-b", TenantId::new("tenant-b").unwrap()),
        ));
    let app = build_router(Arc::new(state));
    let resp = app.oneshot(messages_request(Some("ingress-b"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "cross-tenant must be 403");
}

/// Same-tenant: bearer bound to tenant-a on the tenant-a service => 200.
#[tokio::test]
async fn same_tenant_ingress_principal_is_allowed() {
    let server = MockServer::start();
    let _token = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok","refresh_token":"rt2","expires_in":3600}"#);
    });
    let _msg = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-1","type":"message"}"#);
    });
    let state =
        make_cross_tenant_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
            ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
        ));
    let app = build_router(Arc::new(state));
    let resp = app.oneshot(messages_request(Some("ingress-a"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "same-tenant must reach lease/proxy");
}

/// No bearer => 401 (default-deny before authz).
#[tokio::test]
async fn absent_bearer_is_unauthorized() {
    let server = MockServer::start();
    let state =
        make_cross_tenant_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
            ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
        ));
    let app = build_router(Arc::new(state));
    let resp = app.oneshot(messages_request(None)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Wrong bearer => 401 (constant-time compare fails).
#[tokio::test]
async fn wrong_bearer_is_unauthorized() {
    let server = MockServer::start();
    let state =
        make_cross_tenant_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
            ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
        ));
    let app = build_router(Arc::new(state));
    let resp = app.oneshot(messages_request(Some("wrong-token"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
