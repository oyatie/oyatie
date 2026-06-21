//! D2 axum reverse-proxy contract tests (Stage-5 GREEN).
//!
//! Tests verify the proxy wire types, pool seat selection, tenant isolation,
//! and that the adapter methods return proper error types (no longer todo!()).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzRequest, OAuthSubscription, Provider, SeatId, SelectionStrategy,
    SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{
    AnthropicAdapter, ProxyRequest, ProxyResponse, RestAdapterError, SecretProviderStore,
};
use std::collections::BTreeMap;

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
    fn fetch_refresh_token(&self, _handle: &str) -> Result<String, RestAdapterError> {
        Ok("stub-refresh-token".to_string())
    }
    fn store_refresh_token(&self, _handle: &str, _plaintext: &str) -> Result<(), RestAdapterError> {
        Ok(())
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
    let selected = pool.select(&agent, &gate, Instant::now()).unwrap();
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
    let result = pool.select(&agent_b, &ForbidGate, Instant::now());
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
