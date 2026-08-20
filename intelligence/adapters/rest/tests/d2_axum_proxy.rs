//! D2 axum reverse-proxy contract tests (Stage-5 GREEN).
//!
//! Tests verify the proxy wire types, pool seat selection, tenant isolation,
//! and that the adapter methods return proper error types (no longer todo!()).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::body::Body;
use axum::http::Request;
use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, EventSink, LlmGatewayEvent, OAuthSubscription,
    Provider, SeatId, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState,
    TenantId,
};
use intelligence_rest::{
    AnthropicAdapter, AppState, BearerBinding, ConfiguredBearerMapIngressAuthenticator,
    PoolRegistry, ProxyRequest, ProxyResponse, RestAdapterError, SecretProviderStore, build_router,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};
use tower::ServiceExt as _;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct AlwaysAllowGate;
impl intelligence_kernel::AuthzGate for AlwaysAllowGate {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

struct StubSecretStore;
impl SecretProviderStore for StubSecretStore {
    fn fetch_refresh_token<'a>(
        &'a self,
        _handle: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, String> {
        Box::pin(async { Ok("stub-refresh-token".to_string()) })
    }
    fn store_refresh_token<'a>(
        &'a self,
        _handle: &'a str,
        _plaintext: &'a str,
    ) -> intelligence_rest::SecretProviderFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn make_proxy_request(tenant_id: TenantId) -> ProxyRequest {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers,
        body: br#"{"model":"claude-opus-4-5","max_tokens":100,"messages":[]}"#.to_vec(),
        tenant_id,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// D2-1: ProxyRequest round-trips without data loss.
#[test]
fn d2_proxy_request_fields_preserved() {
    let tenant = TenantId::new("tenant-acme").unwrap();
    let req = make_proxy_request(tenant.clone());
    assert_eq!(req.method, "POST");
    assert_eq!(req.path, "/v1/messages");
    assert_eq!(req.headers.get("content-type").unwrap(), "application/json");
    assert!(!req.body.is_empty());
    assert_eq!(req.tenant_id, tenant);
}

/// D2-2: ProxyResponse carries status + headers + body.
#[test]
fn d2_proxy_response_fields_preserved() {
    let mut headers = BTreeMap::new();
    headers.insert("x-request-id".to_string(), "req-123".to_string());
    let resp = ProxyResponse {
        status: 200,
        headers,
        body: b"{}".to_vec(),
    };
    assert_eq!(resp.status, 200);
    assert_eq!(resp.headers.get("x-request-id").unwrap(), "req-123");
    assert_eq!(resp.body, b"{}");
}

/// D2-3: AnthropicAdapter::proxy returns a network error when no real server
/// is reachable (Stage-6 GREEN: async, uses &reqwest::Client).
#[tokio::test]
async fn d2_proxy_returns_error_without_server() {
    // Point at a guaranteed-unreachable address so reqwest fails fast.
    let adapter =
        AnthropicAdapter::with_base_url(StubSecretStore, "http://127.0.0.1:1".to_string());
    let client = reqwest::Client::new();
    let tenant = TenantId::new("tenant-acme").unwrap();
    let req = make_proxy_request(tenant);
    let result = adapter.proxy(&client, &req, "handle-1").await;
    // Must be an error — either OAuthRefreshFailed (token endpoint unreachable)
    // or UpstreamError (messages endpoint unreachable).
    assert!(
        result.is_err(),
        "expected proxy to fail without a real server, got Ok"
    );
    match result.unwrap_err() {
        RestAdapterError::OAuthRefreshFailed(_) | RestAdapterError::UpstreamError { .. } => {}
        other => panic!("unexpected error variant: {other:?}"),
    }
}

/// D2-4: Pool selects a seat before the proxy path is entered.
#[test]
fn d2_pool_select_before_proxy() {
    use std::time::Instant;
    let tenant = TenantId::new("tenant-acme").unwrap();
    let mut pool = SubscriptionPool::new(
        tenant.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    let seat_id = SeatId::new("seat-001").unwrap();
    pool.add_seat(OAuthSubscription::new(
        tenant.clone(),
        seat_id.clone(),
        SubscriptionId::new("sub-001").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        "handle-1".to_string(),
        0,
    ))
    .unwrap();
    let agent = AgentId::new("agent-bot").unwrap();
    let gate = AlwaysAllowGate;
    let selected = pool.select(&tenant, &agent, &gate, Instant::now()).unwrap();
    assert_eq!(selected, seat_id);
}

/// D2-5: Proxy path respects tenant isolation — cross-tenant request must
/// be rejected by the AuthzGate before reaching the proxy.
#[test]
fn d2_cross_tenant_request_forbidden() {
    use std::time::Instant;
    let tenant_a = TenantId::new("tenant-a").unwrap();
    let _tenant_b = TenantId::new("tenant-b").unwrap();
    let mut pool = SubscriptionPool::new(
        tenant_a.clone(),
        Provider::Anthropic,
        SelectionStrategy::FillFirst,
    );
    pool.add_seat(OAuthSubscription::new(
        tenant_a.clone(),
        SeatId::new("seat-a1").unwrap(),
        SubscriptionId::new("sub-a1").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        "handle-a1".to_string(),
        0,
    ))
    .unwrap();

    struct ForbidGate;
    impl intelligence_kernel::AuthzGate for ForbidGate {
        fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Forbid
        }
    }
    let agent_b = AgentId::new("agent-b").unwrap();
    let result = pool.select(&tenant_a, &agent_b, &ForbidGate, Instant::now());
    assert!(result.is_err());
}

/// D2-6: ProxyRequest body is not modified by construction (no serialization
/// side effects at wire-type level).
#[test]
fn d2_proxy_request_body_untouched() {
    let tenant = TenantId::new("tenant-acme").unwrap();
    let body = br#"{"model":"claude-opus-4-5","max_tokens":256}"#.to_vec();
    let req = ProxyRequest {
        method: "POST".to_string(),
        path: "/v1/messages".to_string(),
        headers: BTreeMap::new(),
        body: body.clone(),
        tenant_id: tenant,
    };
    assert_eq!(req.body, body);
}

/// D2-7: Multiple distinct ProxyRequests for different tenants are independent.
#[test]
fn d2_proxy_requests_are_tenant_scoped() {
    let t1 = TenantId::new("tenant-one").unwrap();
    let t2 = TenantId::new("tenant-two").unwrap();
    let r1 = make_proxy_request(t1.clone());
    let r2 = make_proxy_request(t2.clone());
    assert_ne!(r1.tenant_id, r2.tenant_id);
    assert_eq!(r1.tenant_id, t1);
    assert_eq!(r2.tenant_id, t2);
}

/// D2-8: exchange_authorization_code returns a network error without a real
/// server (Stage-6 GREEN: async, uses &reqwest::Client).
#[tokio::test]
async fn d2_exchange_auth_code_returns_error_without_server() {
    let adapter =
        AnthropicAdapter::with_base_url(StubSecretStore, "http://127.0.0.1:1".to_string());
    let client = reqwest::Client::new();
    let tenant = TenantId::new("tenant-acme").unwrap();
    let seat = SeatId::new("seat-001").unwrap();
    let result = adapter
        .exchange_authorization_code(&client, &tenant, &seat, "auth-code-xyz")
        .await;
    assert!(
        result.is_err(),
        "expected exchange_authorization_code to fail without a real server"
    );
    assert!(
        matches!(result.unwrap_err(), RestAdapterError::OAuthRefreshFailed(_)),
        "expected OAuthRefreshFailed"
    );
}

// ---------------------------------------------------------------------------
// AUTH-005 increment-3: multi-tenant data-plane closure (RED integration).
//
// These exercise the real router: the verified principal tenant keys the pool
// lookup (primary cross-tenant barrier) and the kernel `authz_allows` backstop
// catches a mis-keyed pool. RED before the fix: the new authenticator types did
// not exist and the handler leased the single `state.pool` for every caller.
// ---------------------------------------------------------------------------

/// Deny-wins gate mirroring the Cedar tenant-isolation contract: forbids any
/// request whose principal tenant differs from the resource tenant.
struct TenantIsolationGate;
impl AuthzGate for TenantIsolationGate {
    fn decide(&self, request: &AuthzRequest<'_>) -> AuthzDecision {
        if request.principal_tenant == request.resource_tenant {
            AuthzDecision::Allow
        } else {
            AuthzDecision::Forbid
        }
    }
}

struct NoopSink;
impl EventSink for NoopSink {
    fn emit(&self, _event: LlmGatewayEvent) {}
}

fn pool_with_seat(tenant: &str, seat: &str) -> Arc<Mutex<SubscriptionPool>> {
    let tenant_id = TenantId::new(tenant).unwrap();
    let mut pool = SubscriptionPool::new(
        tenant_id.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(OAuthSubscription::new(
        tenant_id,
        SeatId::new(seat).unwrap(),
        SubscriptionId::new(format!("{seat}-sub")).unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("secret-ref://{tenant}/anthropic/{seat}"),
        0,
    ))
    .unwrap();
    Arc::new(Mutex::new(pool))
}

/// Build a multi-tenant AppState with a `tokA=>tenant-a, tokB=>tenant-b` bearer
/// map and a deny-wins gate (so the below-edge kernel backstop is exercised).
fn two_tenant_state(
    registry: PoolRegistry,
    default_pool: Arc<Mutex<SubscriptionPool>>,
) -> Arc<AppState> {
    let map = ConfiguredBearerMapIngressAuthenticator::new(vec![
        BearerBinding {
            token: "tokA".to_string(),
            tenant: TenantId::new("tenant-a").unwrap(),
            agent: None,
        },
        BearerBinding {
            token: "tokB".to_string(),
            tenant: TenantId::new("tenant-b").unwrap(),
            agent: None,
        },
    ]);
    let state = AppState::new_with_pool_registry(
        default_pool,
        registry,
        Arc::new(TenantIsolationGate),
        Arc::new(NoopSink),
        Arc::new(StubSecretStore),
        "http://127.0.0.1:1".to_string(),
        TenantId::new("tenant-a").unwrap(),
        None,
        None,
        "test".to_string(),
        HashSet::new(),
    )
    .unwrap()
    .with_ingress_authenticator(Arc::new(map));
    Arc::new(state)
}

fn anthropic_request(bearer: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("authorization", format!("Bearer {bearer}"))
        .header("content-type", "application/json")
        .header("x-agent-id", "agent-x")
        .body(Body::from(
            r#"{"model":"claude-opus-4-5","max_tokens":100,"messages":[]}"#,
        ))
        .unwrap()
}

/// (i) Caller A's bearer must NOT lease tenant-B's seats. tenant-B owns the only
/// populated pool (also the default `state.pool`); caller A's verified tenant
/// resolves to NO tenant-a pool => 503, never a tenant-B seat. Caller B leases
/// its own seat and reaches the (unreachable) upstream => NOT 503.
#[tokio::test]
async fn auth005_inc3_caller_a_cannot_lease_tenant_b_pool() {
    let pool_b = pool_with_seat("tenant-b", "seat-b");
    let registry = PoolRegistry::new();
    registry.insert_pool(
        TenantId::new("tenant-b").unwrap(),
        Provider::Anthropic,
        Arc::clone(&pool_b),
    );
    // Default `state.pool` is tenant-B's populated pool — caller A must not fall
    // back to it (the pre-fix bug).
    let state = two_tenant_state(registry, Arc::clone(&pool_b));

    let resp_a = build_router(Arc::clone(&state))
        .oneshot(anthropic_request("tokA"))
        .await
        .unwrap();
    assert_eq!(
        resp_a.status().as_u16(),
        503,
        "caller A must not reach tenant-B's seats"
    );

    let resp_b = build_router(state)
        .oneshot(anthropic_request("tokB"))
        .await
        .unwrap();
    assert_ne!(
        resp_b.status().as_u16(),
        503,
        "caller B owns a leasable tenant-B seat"
    );
}

/// (ii) Unknown bearer => 401 (fail-closed, no allow-all path).
#[tokio::test]
async fn auth005_inc3_unknown_bearer_is_unauthorized() {
    let pool_b = pool_with_seat("tenant-b", "seat-b");
    let registry = PoolRegistry::new();
    registry.insert_pool(
        TenantId::new("tenant-b").unwrap(),
        Provider::Anthropic,
        Arc::clone(&pool_b),
    );
    let state = two_tenant_state(registry, pool_b);

    let resp = build_router(state)
        .oneshot(anthropic_request("tok-unknown"))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

/// (iii) Mis-keyed registry: tenant-B's pool registered under tenant-A's key.
/// Caller A passes the edge (principal==resource==tenant-a) but the kernel
/// backstop sees principal=tenant-a vs the pool's own tenant-b => Forbid => 403.
/// Proves the below-edge defense-in-depth catches a cross-tenant pool mis-route.
#[tokio::test]
async fn auth005_inc3_miskeyed_pool_is_forbidden_by_kernel_backstop() {
    let pool_b = pool_with_seat("tenant-b", "seat-b");
    let registry = PoolRegistry::new();
    // BUG INJECTION: tenant-B's pool registered under tenant-A's key.
    registry.insert_pool(
        TenantId::new("tenant-a").unwrap(),
        Provider::Anthropic,
        Arc::clone(&pool_b),
    );
    let state = two_tenant_state(registry, pool_b);

    let resp = build_router(state)
        .oneshot(anthropic_request("tokA"))
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        403,
        "kernel backstop must forbid a mis-keyed cross-tenant pool"
    );
}
