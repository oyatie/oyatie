//! Acceptance tests for the managed-Kubernetes control-plane-host composition
//! root (ADR-0376), grounded in the PRD + implementation-ready acceptance
//! criteria.
//!
//! These drive the FULL flow through the REAL kernel + api + in-memory adapter
//! plus the axum router over a localhost TCP socket (loopback only; no kube-rs,
//! no network). Both tiers (hosted Kamaji and dedicated Talos spoke) are
//! exercised end-to-end: provision then status(active) then teardown then
//! status(deleted).
//!
//! Mapped acceptance criteria:
//! AC-1 hosted-tier provision reaches `active` with an endpoint;
//! AC-2 dedicated-tier provision reaches `active`;
//! AC-3 teardown drains + deletes and status reflects `deleted`;
//! AC-4 malformed cluster ref / unknown tier returns 400 (fail-closed);
//! AC-5 healthz is mounted.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::SocketAddr;
use std::sync::Arc;

use k8s_control_plane_host_app::{
    ConfiguredBearerPrincipalVerifier, ConfiguredPlatformAdminAuthorizer, ControlPlaneAction,
    ControlPlaneAuthorizationError, ControlPlaneAuthzProvider, PlatformAdminAuthorizer,
    VerifiedPrincipal, build_router, build_state_in_memory,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const TEST_TOKEN: &str = "test-break-glass-secret";

/// Break-glass platform-operator authz provider (the production composition).
fn platform_authz() -> ControlPlaneAuthzProvider {
    ControlPlaneAuthzProvider::from_bearer_secret(TEST_TOKEN, "op", "ten_platform").unwrap()
}

/// A verifier that binds a NON-platform principal (no platform scope): the
/// bearer authenticates but the PDP denies the admin action (403).
fn non_admin_authz() -> ControlPlaneAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(TEST_TOKEN, "op", "ten_acme", vec![]).unwrap(),
    );
    ControlPlaneAuthzProvider::new(verifier, Arc::new(ConfiguredPlatformAdminAuthorizer::new()))
}

/// A faulting authorizer: every decision is a fail-closed PDP refusal (=> 403).
struct FaultAuthorizer;
impl PlatformAdminAuthorizer for FaultAuthorizer {
    fn ensure_authorized(
        &self,
        _p: &VerifiedPrincipal,
        _a: ControlPlaneAction,
    ) -> Result<(), ControlPlaneAuthorizationError> {
        Err(ControlPlaneAuthorizationError::Refused)
    }
}

fn fault_authz() -> ControlPlaneAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(TEST_TOKEN, "op", "ten_platform", vec![]).unwrap(),
    );
    ControlPlaneAuthzProvider::new(verifier, Arc::new(FaultAuthorizer))
}

/// Spawn the in-memory-backed router on a localhost port; return its address.
async fn spawn_app() -> SocketAddr {
    spawn_app_with(platform_authz()).await
}

async fn spawn_app_with(authz: ControlPlaneAuthzProvider) -> SocketAddr {
    let state = build_state_in_memory(authz);
    let router = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

/// Minimal HTTP/1.1 client over a raw TCP socket (no extra dev-dep). Sends one
/// request WITH the valid break-glass bearer and returns `(status_code, body)`.
async fn http(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    http_auth(addr, method, path, body, Some(TEST_TOKEN)).await
}

/// Like [`http`], but with an explicit (optional) bearer token.
async fn http_auth(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    bearer: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let auth_header = match bearer {
        Some(token) => format!("Authorization: Bearer {token}\r\n"),
        None => String::new(),
    };
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{auth_header}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8_lossy(&raw).to_string();
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);
    let body = text
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

/// Extract the JSON string value for `key` from a flat object body (avoids a
/// serde_json dev-dep; the bodies are small and flat).
fn json_str(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[tokio::test]
async fn healthz_is_mounted() {
    let addr = spawn_app().await;
    let (status, body) = http(addr, "GET", "/healthz", "").await;
    assert_eq!(status, 200);
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn hosted_tier_full_lifecycle() {
    let addr = spawn_app().await;

    // AC-1: provision hosted -> 201 + handle.
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a","tier":"hosted_kamaji","datastore_class":"etcd_per_tenant"}"#,
    )
    .await;
    assert_eq!(status, 201, "provision body: {body}");
    assert_eq!(json_str(&body, "tier").as_deref(), Some("hosted_kamaji"));
    let handle = json_str(&body, "handle").expect("handle present");

    // AC-1: status -> active with an endpoint.
    let ref_body = format!(
        r#"{{"tenant_id":"ten_zero","cluster_name":"dogfood-a","tier":"hosted_kamaji","handle":"{handle}"}}"#
    );
    let (status, body) = http(addr, "POST", "/admin/control-planes/status", &ref_body).await;
    assert_eq!(status, 200, "status body: {body}");
    assert_eq!(json_str(&body, "status").as_deref(), Some("active"));
    assert!(body.contains("endpoint"), "endpoint surfaced: {body}");

    // AC-3: teardown -> 204, then status -> deleted.
    let (status, _body) = http(addr, "POST", "/admin/control-planes/teardown", &ref_body).await;
    assert_eq!(status, 204);
    let (status, body) = http(addr, "POST", "/admin/control-planes/status", &ref_body).await;
    assert_eq!(status, 200);
    assert_eq!(json_str(&body, "status").as_deref(), Some("deleted"));
}

#[tokio::test]
async fn dedicated_tier_provisions_active() {
    let addr = spawn_app().await;
    // AC-2: dedicated tier provision -> 201, status active.
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"sovereign-1","tier":"dedicated_talos_spoke"}"#,
    )
    .await;
    assert_eq!(status, 201, "provision body: {body}");
    assert_eq!(
        json_str(&body, "tier").as_deref(),
        Some("dedicated_talos_spoke")
    );
    let handle = json_str(&body, "handle").expect("handle present");

    let ref_body = format!(
        r#"{{"tenant_id":"ten_zero","cluster_name":"sovereign-1","tier":"dedicated_talos_spoke","handle":"{handle}"}}"#
    );
    let (status, body) = http(addr, "POST", "/admin/control-planes/status", &ref_body).await;
    assert_eq!(status, 200);
    assert_eq!(json_str(&body, "status").as_deref(), Some("active"));
}

#[tokio::test]
async fn provision_defaults_to_hosted_tier_when_omitted() {
    let addr = spawn_app().await;
    // tier omitted -> product default is hosted (ADR-0376).
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"default-tier"}"#,
    )
    .await;
    assert_eq!(status, 201, "provision body: {body}");
    assert_eq!(json_str(&body, "tier").as_deref(), Some("hosted_kamaji"));
}

#[tokio::test]
async fn unknown_tier_is_rejected_fail_closed() {
    let addr = spawn_app().await;
    // AC-4: unknown tier -> 400.
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"c","tier":"gardener"}"#,
    )
    .await;
    assert_eq!(status, 400, "body: {body}");
    assert!(body.contains("invalid_cluster_ref"));
}

#[tokio::test]
async fn malformed_cluster_ref_is_rejected_fail_closed() {
    let addr = spawn_app().await;
    // AC-4: empty tenant/cluster -> 400.
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"","cluster_name":""}"#,
    )
    .await;
    assert_eq!(status, 400, "body: {body}");
}

// ============================================================
// AUTH-005 fail-closed fixtures (RED before this fix; GREEN after)
// ============================================================

#[tokio::test]
async fn provision_without_bearer_returns_401() {
    let addr = spawn_app().await;
    let (status, _body) = http_auth(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a"}"#,
        None,
    )
    .await;
    assert_eq!(status, 401);
}

#[tokio::test]
async fn provision_non_platform_principal_returns_403() {
    // A valid bearer whose principal lacks the platform scope is authenticated
    // but NOT authorized for the platform-level admin surface => 403.
    let addr = spawn_app_with(non_admin_authz()).await;
    let (status, _body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a"}"#,
    )
    .await;
    assert_eq!(status, 403);
}

#[tokio::test]
async fn provision_platform_operator_returns_201() {
    // The platform-operator break-glass bearer is authorized.
    let addr = spawn_app().await;
    let (status, body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a"}"#,
    )
    .await;
    assert_eq!(status, 201, "body: {body}");
}

#[tokio::test]
async fn provision_pdp_fault_returns_403_not_5xx() {
    let addr = spawn_app_with(fault_authz()).await;
    let (status, _body) = http(
        addr,
        "POST",
        "/admin/control-planes",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a"}"#,
    )
    .await;
    assert_eq!(status, 403);
}

#[tokio::test]
async fn teardown_without_bearer_returns_401() {
    let addr = spawn_app().await;
    let (status, _body) = http_auth(
        addr,
        "POST",
        "/admin/control-planes/teardown",
        r#"{"tenant_id":"ten_zero","cluster_name":"dogfood-a","tier":"hosted_kamaji","handle":"h"}"#,
        None,
    )
    .await;
    assert_eq!(status, 401);
}

#[test]
fn boot_refuses_empty_bearer_secret() {
    assert!(
        ControlPlaneAuthzProvider::from_bearer_secret("", "op", "ten_platform").is_err(),
        "an empty bearer secret must refuse provider construction"
    );
}
