//! Fix-4: Body-size limit — POST /v1/messages with body > 1 MiB receives 413.
//!
//! Also covers D7 — data-plane cross-tenant ingress forbid (AUTH-005 / ADR-0573):
//! the REST boundary must feed the VERIFIED principal's tenant to the gate so the
//! cross-tenant Cedar forbid actually fires.
//!
//! Also covers D8 — admin-plane cross-tenant forbid + accounts scoping (AUTH-005 / ADR-0573):
//! the admin credential must be bound to the configured tenant (never the
//! x-admin-tenant header); cross-tenant path => 403; gate is consulted and
//! fail-closed; /admin/v1/accounts scoped to the verified tenant only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use http_body_util::BodyExt as _;
use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{
    AppState, ConfiguredBearerAdminAuthenticator, ConfiguredBearerIngressAuthenticator, EventSink,
    LlmGatewayEvent, PoolRegistry, RestAdapterError, SecretProviderStore,
    UpstreamOAuthSingleflight, build_router,
};
use scripted_http_server::{ScriptedResponse, ScriptedServer};
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
    fn fetch_refresh_token<'a>(
        &'a self,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, String> {
        Box::pin(async { Ok("stub-token".to_string()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
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
            tenant.clone(),
        )),
        admin_authenticator: Arc::new(ConfiguredBearerAdminAuthenticator::new(
            String::new(),
            tenant,
        )),
        admin_bearer_token: None,
        environment: "test".to_string(),
        oauth_approved_providers: std::collections::HashSet::new(),
        upstream_oauth_singleflight: Arc::new(UpstreamOAuthSingleflight::new()),
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
