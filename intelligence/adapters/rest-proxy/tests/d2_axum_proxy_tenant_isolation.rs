#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::body::Body;
use axum::http::Request;
use intelligence_kernel::{
    AuthzDecision, AuthzGate, AuthzRequest, EventSink, LlmGatewayEvent, OAuthSubscription,
    Provider, SeatId, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState,
    TenantId,
};
use intelligence_rest_proxy::{
    AppState, BearerBinding, ConfiguredBearerMapIngressAuthenticator, PoolRegistry, build_router,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tower::ServiceExt as _;

mod d2_axum_proxy_support;
use d2_axum_proxy_support::StubSecretStore;

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
