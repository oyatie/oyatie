//! Fix-4: Body-size limit — POST /v1/messages with body > 1 MiB receives 413.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use oya_llm_gateway_oauth_pool_kernel::{
    AgentId, AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use oya_llm_gateway_oauth_pool_rest::{
    AppState, EventSink, LlmGatewayEvent, OpenBaoSecretStore, RestAdapterError,
    TokenRefreshSingleflight, build_router,
};
use tower::ServiceExt; // for `oneshot`

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct AlwaysAllow;
impl oya_llm_gateway_oauth_pool_kernel::AuthzGate for AlwaysAllow {
    fn decide(&self, _: &AuthzRequest<'_>) -> AuthzDecision {
        AuthzDecision::Allow
    }
}

struct NoopSink;
impl EventSink for NoopSink {
    fn emit(&self, _: LlmGatewayEvent) {}
}

struct StubStore;
impl OpenBaoSecretStore for StubStore {
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
    pool.add_seat(OAuthSubscription {
        tenant_id: tenant.clone(),
        seat_id: SeatId::new("seat-bl-1").unwrap(),
        subscription_id: SubscriptionId::new("sub-bl-1").unwrap(),
        provider: Provider::Anthropic,
        state: SubscriptionState::Active,
        refresh_token_handle: "t-body-limit/seat-bl-1".to_string(),
        failure_count: 0,
    })
    .unwrap();
    Arc::new(AppState {
        pool: Arc::new(Mutex::new(pool)),
        gate: Arc::new(AlwaysAllow),
        sink: Arc::new(NoopSink),
        secret_store: Arc::new(StubStore),
        anthropic_base_url: "http://127.0.0.1:1".to_string(),
        tenant_id: tenant,
        token_singleflight: Arc::new(TokenRefreshSingleflight::new()),
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
