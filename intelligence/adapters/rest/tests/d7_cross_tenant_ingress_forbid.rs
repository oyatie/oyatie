//! D7 — data-plane cross-tenant ingress forbid (AUTH-005 / ADR-0573).
//!
//! Proves the REST boundary feeds the VERIFIED ingress principal's tenant to the
//! in-process authz gate (never `state.tenant_id`, never a caller header) so the
//! cross-tenant forbid actually fires.
//!
//! Setup: the service serves tenant-a (tenant-a pool). The gate forbids any
//! request whose principal tenant differs from the resource tenant (deny-wins,
//! mirroring the owned policy-engine + Cedar cross-tenant forbid). The boundary
//! is exercised through the real axum router.
//!
//! - Cross-tenant: a VALID bearer bound to tenant-b + x-agent-id => 403.
//! - Same-tenant:  a VALID bearer bound to tenant-a + x-agent-id => 200 (leases).
//! - No / wrong bearer => 401 (default-deny, before any authz).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::{Arc, Mutex};

use axum::body::Body;
use httpmock::prelude::*;
use intelligence_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState, TenantId,
};
use intelligence_rest::{
    AppState, ConfiguredBearerIngressAuthenticator, PoolRegistry, RestAdapterError,
    SecretProviderStore,
};

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

struct StubStore;
impl SecretProviderStore for StubStore {
    fn fetch_refresh_token(&self, _: &str) -> Result<String, RestAdapterError> {
        Ok("refresh-tok".to_string())
    }
    fn store_refresh_token(&self, _: &str, _: &str) -> Result<(), RestAdapterError> {
        Ok(())
    }
}

struct NoopSink;
impl intelligence_kernel::EventSink for NoopSink {
    fn emit(&self, _: intelligence_kernel::LlmGatewayEvent) {}
}

/// Deny-wins gate: forbids any request whose principal tenant differs from the
/// resource tenant. Mirrors the owned policy-engine / Cedar cross-tenant forbid
/// without pulling the concrete Cedar adapter into this crate's test deps (the
/// Cedar policy itself is tested in the cedar adapter crate). The boundary's
/// responsibility under test is feeding the CORRECT tenants to the gate.
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

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn make_pool(tenant: &str) -> Arc<Mutex<SubscriptionPool>> {
    let t = TenantId::new(tenant).unwrap();
    let mut pool = SubscriptionPool::new(t.clone(), Provider::Anthropic, SelectionStrategy::RoundRobin);
    let seat = SeatId::new("seat-1").unwrap();
    pool.add_seat(OAuthSubscription::new(
        t.clone(),
        seat.clone(),
        SubscriptionId::new("sub-1").unwrap(),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("{tenant}/{}", seat.as_str()),
        0,
    ))
    .unwrap();
    Arc::new(Mutex::new(pool))
}

/// Build an AppState for `tenant` with the cross-tenant-forbidding gate. The
/// caller installs the ingress authenticator (binding the verified principal
/// tenant) via `with_ingress_authenticator`.
fn make_state(base_url: String, tenant: &str) -> AppState {
    let tenant_id = TenantId::new(tenant).unwrap();
    let pool = make_pool(tenant);
    let registry = PoolRegistry::new();
    registry.insert_pool(tenant_id.clone(), Provider::Anthropic, Arc::clone(&pool));
    AppState::new_with_pool_registry(
        pool,
        registry,
        Arc::new(CrossTenantForbidGate),
        Arc::new(NoopSink),
        Arc::new(StubStore),
        base_url,
        tenant_id,
        None,
        None,
        "development".to_string(),
        std::collections::HashSet::new(),
    )
    .unwrap()
}

const BODY: &str = r#"{"model":"claude-opus-4-5","max_tokens":10,"messages":[]}"#;

fn request(bearer: Option<&str>) -> axum::http::Request<Body> {
    let mut builder = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .header("x-agent-id", "agent-x");
    if let Some(bearer) = bearer {
        builder = builder.header("authorization", format!("Bearer {bearer}"));
    }
    builder.body(Body::from(BODY)).unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Cross-tenant: a valid bearer bound to tenant-b hitting the tenant-a service
/// must be FORBIDDEN (403) — the boundary feeds principal=tenant-b vs
/// resource=tenant-a to the gate, which denies.
#[tokio::test]
async fn cross_tenant_ingress_principal_is_forbidden() {
    let server = MockServer::start();
    let state = make_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
        ConfiguredBearerIngressAuthenticator::new("ingress-b", TenantId::new("tenant-b").unwrap()),
    ));
    let app = intelligence_rest::build_router(Arc::new(state));

    let response = tower::ServiceExt::oneshot(app, request(Some("ingress-b")))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::FORBIDDEN,
        "tenant-b principal on the tenant-a data plane must be forbidden (cross-tenant)"
    );
}

/// Same-tenant: a valid bearer bound to tenant-a is permitted and reaches the
/// lease/proxy path (200).
#[tokio::test]
async fn same_tenant_ingress_principal_is_allowed() {
    let server = MockServer::start();
    let _token_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"access_token":"tok","refresh_token":"rt2","expires_in":3600}"#);
    });
    let _msg_mock = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"id":"msg-1","type":"message"}"#);
    });

    let state = make_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
        ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
    ));
    let app = intelligence_rest::build_router(Arc::new(state));

    let response = tower::ServiceExt::oneshot(app, request(Some("ingress-a")))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "same-tenant principal must be permitted and reach the lease/proxy path"
    );
}

/// No bearer => 401 (default-deny, before any authz).
#[tokio::test]
async fn absent_bearer_is_unauthorized() {
    let server = MockServer::start();
    let state = make_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
        ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
    ));
    let app = intelligence_rest::build_router(Arc::new(state));

    let response = tower::ServiceExt::oneshot(app, request(None))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

/// Wrong bearer => 401 (constant-time compare fails, no principal minted).
#[tokio::test]
async fn wrong_bearer_is_unauthorized() {
    let server = MockServer::start();
    let state = make_state(server.base_url(), "tenant-a").with_ingress_authenticator(Arc::new(
        ConfiguredBearerIngressAuthenticator::new("ingress-a", TenantId::new("tenant-a").unwrap()),
    ));
    let app = intelligence_rest::build_router(Arc::new(state));

    let response = tower::ServiceExt::oneshot(app, request(Some("wrong-token")))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}
