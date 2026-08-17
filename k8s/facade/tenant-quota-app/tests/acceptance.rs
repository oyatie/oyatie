//! Acceptance tests for the managed-K8s tenant quota service.
//!
//! These tests drive the full HTTP surface using the in-memory store.
//! No sockets are opened; `axum::Router` is called via `tower::ServiceExt`.
//!
//! Every mutating/per-tenant route is fail-closed: an unauthenticated caller is
//! rejected 401 (the `VerifiedCaller` extractor runs before the body is parsed),
//! a cross-tenant caller is rejected 403 by the Cedar PDP, and a PDP fault is
//! rejected 403 (never a 5xx). The happy-path tests present a break-glass
//! platform-operator bearer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use k8s_tenant_quota_adapter_inmemory::InMemoryQuotaStore;
use k8s_tenant_quota_app::{
    ConfiguredBearerPrincipalVerifier, QuotaAction, QuotaAuthorizationError, QuotaAuthorizer,
    QuotaAuthzProvider, VerifiedPrincipal, build_router,
};
use k8s_tenant_quota_kernel::{TenantId, TenantQuota, TenantUsage};
use tower::ServiceExt;

const TEST_TOKEN: &str = "test-break-glass-secret";

/// A break-glass platform-operator authz provider: any tenant allowed (matches
/// the production composition). Used by the happy-path tests.
fn platform_authz() -> QuotaAuthzProvider {
    QuotaAuthzProvider::from_bearer_secret(
        TEST_TOKEN,
        "wl_platform_op",
        "ten_platform",
        vec!["quota:platform:write".to_owned()],
    )
    .unwrap()
}

/// A tenant-admin authz provider bound to `tenant` with `quota:write` +
/// `quota:read` scopes: own-tenant allowed, cross-tenant denied by the PDP.
fn tenant_admin_authz(tenant: &str) -> QuotaAuthzProvider {
    QuotaAuthzProvider::from_bearer_secret(
        TEST_TOKEN,
        "wl_tenant_admin",
        tenant,
        vec!["quota:write".to_owned(), "quota:read".to_owned()],
    )
    .unwrap()
}

/// A faulting authorizer: every decision is a fail-closed PDP refusal (=> 403).
struct FaultAuthorizer;
impl QuotaAuthorizer for FaultAuthorizer {
    fn ensure_authorized(
        &self,
        _p: &VerifiedPrincipal,
        _a: QuotaAction,
        _t: &TenantId,
    ) -> Result<(), QuotaAuthorizationError> {
        Err(QuotaAuthorizationError::Refused)
    }
}

fn fault_authz() -> QuotaAuthzProvider {
    let verifier =
        Arc::new(ConfiguredBearerPrincipalVerifier::new(TEST_TOKEN, "op", "t", vec![]).unwrap());
    QuotaAuthzProvider::new(verifier, Arc::new(FaultAuthorizer))
}

fn bearer() -> String {
    format!("Bearer {TEST_TOKEN}")
}

fn make_store_with_quota(tenant: &str) -> InMemoryQuotaStore {
    InMemoryQuotaStore::new().with_quota(TenantQuota::new(tenant, 5, 10, 32, 128).unwrap())
}

async fn body_str(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

#[tokio::test]
async fn healthz_returns_200() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn put_quota_then_get_quota_round_trip() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let body = serde_json::json!({
        "tenant_id": "ten_acme",
        "max_clusters": 3,
        "max_nodes_per_cluster": 5,
        "max_vcpu_per_cluster": 16,
        "max_ram_gib_per_cluster": 64
    });
    let put_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/tenants/ten_acme/quota")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(put_resp.status(), StatusCode::NO_CONTENT);

    let get_resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_acme/quota")
                .header("authorization", bearer())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let text = body_str(get_resp.into_body()).await;
    assert!(text.contains("ten_acme"));
    assert!(text.contains("3"));
}

#[tokio::test]
async fn get_quota_not_found_returns_404() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_unknown/quota")
                .header("authorization", bearer())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn check_quota_allow_within_limits() {
    let store = make_store_with_quota("ten_acme");
    let app = build_router(store, platform_authz());
    let body = serde_json::json!({
        "requested_clusters": 1,
        "requested_nodes_per_cluster": 3,
        "requested_vcpu_per_cluster": 8,
        "requested_ram_gib_per_cluster": 32
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/tenants/ten_acme/quota/check")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = body_str(resp.into_body()).await;
    assert!(text.contains("\"allowed\":true"));
}

#[tokio::test]
async fn check_quota_deny_cluster_exceeded() {
    let store = InMemoryQuotaStore::new()
        .with_quota(TenantQuota::new("ten_acme", 2, 10, 32, 128).unwrap())
        .with_usage(TenantUsage::new("ten_acme", 2, 0, 0, 0).unwrap());
    let app = build_router(store, platform_authz());
    let body = serde_json::json!({
        "requested_clusters": 1,
        "requested_nodes_per_cluster": 1,
        "requested_vcpu_per_cluster": 1,
        "requested_ram_gib_per_cluster": 1
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/tenants/ten_acme/quota/check")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = body_str(resp.into_body()).await;
    assert!(text.contains("\"allowed\":false"));
    assert!(text.contains("deny_reason"));
}

#[tokio::test]
async fn check_quota_unknown_tenant_returns_404() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let body = serde_json::json!({
        "requested_clusters": 1,
        "requested_nodes_per_cluster": 1,
        "requested_vcpu_per_cluster": 1,
        "requested_ram_gib_per_cluster": 1
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/tenants/ten_unknown/quota/check")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn put_quota_path_body_mismatch_returns_400() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let body = serde_json::json!({
        "tenant_id": "ten_other",  // mismatches path "ten_acme"
        "max_clusters": 3,
        "max_nodes_per_cluster": 5,
        "max_vcpu_per_cluster": 16,
        "max_ram_gib_per_cluster": 64
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/tenants/ten_acme/quota")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_usage_after_set_usage() {
    let store =
        InMemoryQuotaStore::new().with_usage(TenantUsage::new("ten_acme", 3, 5, 16, 64).unwrap());
    let app = build_router(store, platform_authz());
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_acme/usage")
                .header("authorization", bearer())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let text = body_str(resp.into_body()).await;
    assert!(text.contains("ten_acme"));
    assert!(text.contains("3")); // current_clusters
}

// ============================================================
// AUTH-005 fail-closed fixtures (RED before this fix; GREEN after)
// ============================================================

#[tokio::test]
async fn put_quota_without_bearer_returns_401() {
    let app = build_router(InMemoryQuotaStore::new(), platform_authz());
    let body = serde_json::json!({
        "tenant_id": "ten_acme",
        "max_clusters": 3,
        "max_nodes_per_cluster": 5,
        "max_vcpu_per_cluster": 16,
        "max_ram_gib_per_cluster": 64
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/tenants/ten_acme/quota")
                .header("content-type", "application/json")
                // no authorization header
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_quota_cross_tenant_returns_403() {
    // A tenant-admin bound to ten_acme reading ten_globex is cross-tenant: the
    // Cedar PDP denies it (tenant isolation) => 403, even though the bearer is
    // valid (authenticated but not authorized).
    let app = build_router(
        make_store_with_quota("ten_globex"),
        tenant_admin_authz("ten_acme"),
    );
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_globex/quota")
                .header("authorization", bearer())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn put_quota_same_tenant_admin_returns_2xx() {
    // A tenant-admin bound to ten_acme writing ten_acme is authorized.
    let app = build_router(InMemoryQuotaStore::new(), tenant_admin_authz("ten_acme"));
    let body = serde_json::json!({
        "tenant_id": "ten_acme",
        "max_clusters": 3,
        "max_nodes_per_cluster": 5,
        "max_vcpu_per_cluster": 16,
        "max_ram_gib_per_cluster": 64
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/tenants/ten_acme/quota")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn put_quota_pdp_fault_returns_403_not_5xx() {
    let app = build_router(InMemoryQuotaStore::new(), fault_authz());
    let body = serde_json::json!({
        "tenant_id": "ten_acme",
        "max_clusters": 3,
        "max_nodes_per_cluster": 5,
        "max_vcpu_per_cluster": 16,
        "max_ram_gib_per_cluster": 64
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/tenants/ten_acme/quota")
                .header("content-type", "application/json")
                .header("authorization", bearer())
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[test]
fn boot_refuses_empty_bearer_secret() {
    // Fail-closed boot: an empty bearer secret means no provable credential
    // root, so the provider (and therefore the service) refuses to construct.
    assert!(
        QuotaAuthzProvider::from_bearer_secret("", "op", "ten_platform", vec![]).is_err(),
        "an empty bearer secret must refuse provider construction"
    );
}
