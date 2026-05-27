//! Acceptance tests for the managed-K8s tenant quota service.
//!
//! These tests drive the full HTTP surface using the in-memory store.
//! No sockets are opened; `axum::Router` is called via `axum_test` pattern
//! using `tower::ServiceExt`.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use oya_managed_k8s_tenant_quota_adapter_inmemory::InMemoryQuotaStore;
use oya_managed_k8s_tenant_quota_app::build_router;
use oya_managed_k8s_tenant_quota_kernel::{TenantQuota, TenantUsage};
use tower::ServiceExt;

fn make_store_with_quota(tenant: &str) -> InMemoryQuotaStore {
    InMemoryQuotaStore::new().with_quota(TenantQuota::new(tenant, 5, 10, 32, 128).unwrap())
}

async fn body_str(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

#[tokio::test]
async fn healthz_returns_200() {
    let app = build_router(InMemoryQuotaStore::new());
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
    let app = build_router(InMemoryQuotaStore::new());
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
    let app = build_router(InMemoryQuotaStore::new());
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_unknown/quota")
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
    let app = build_router(store);
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
    let app = build_router(store);
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
    let app = build_router(InMemoryQuotaStore::new());
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
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn put_quota_path_body_mismatch_returns_400() {
    let app = build_router(InMemoryQuotaStore::new());
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
    let app = build_router(store);
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/tenants/ten_acme/usage")
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
