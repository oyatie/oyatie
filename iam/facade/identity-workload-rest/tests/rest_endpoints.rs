//! End-to-end tests for the workload-identity REST surface.
//!
//! Each test drives the real axum router via `tower::ServiceExt::oneshot` with
//! a genuinely-signed ES256 workload JWT (minted with `aws-lc-rs` and, for the
//! issuer/JWKS E2E, issuer-kernel claim validation), exercising the
//! full validate -> resolve -> authorize hot path through the REAL adapters and
//! asserting the fail-closed PEP status mapping + one-audit-record-per-call
//! invariant (PRD §1.2/§3.4/§3.5/§5, AC-W-13).
//!
//! Fixtures are shared with `grpc_authorize_deny.rs` via `tests/common/mod.rs`
//! so both surfaces provably exercise one shared setup with identical JWKS,
//! Cedar policies, and provisioned principal.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

use iam_identity_oidc_issuer_kernel::{
    Algorithm as IssuerAlgorithm, SigningKey, build_jwks as build_issuer_jwks,
};
use iam_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository, activate, provision,
};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_domain::{WorkloadId, WorkloadPrincipal};
use iam_identity_workload_oidc::{Jwk, JwkMaterial, Jwks, ValidationConfig};
use iam_identity_workload_rest::{
    AuditEvent, AuditRecord, InMemoryAuditSink, SharedState, WorkloadAuthzState, build_router,
};

use common::{
    AUDIENCE, AllowAllLifecycleAuthorizer, FailingRepository, FaultingDecisionAuthorizer,
    FaultingLifecycleAuthorizer, ISSUER, LIFECYCLE_BEARER, LIFECYCLE_CALLER_ID,
    LIFECYCLE_CALLER_TENANT, NOW, SameTenantDecisionAuthorizer, SameTenantLifecycleAuthorizer,
    lifecycle_verifier, mint_issuer_kernel_access_token, mint_token, permit_authorizer,
    provisioned_state, provisioned_state_with_jwks,
};
use iam_identity_workload_rest::BearerCallerVerifier;

async fn post_json(router: axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    post_json_inner(router, path, body, None).await
}

/// POST with an `Authorization: Bearer <token>` header — for the lifecycle
/// control-plane routes that require a verified caller.
async fn post_json_bearer(
    router: axum::Router,
    path: &str,
    body: Value,
    bearer: &str,
) -> (StatusCode, Value) {
    post_json_inner(router, path, body, Some(bearer)).await
}

async fn post_json_inner(
    router: axum::Router,
    path: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(bearer) = bearer {
        builder = builder.header("authorization", format!("Bearer {bearer}"));
    }
    let request = builder.body(Body::from(body.to_string())).expect("request");
    let response = router.oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

fn verifier_jwks_from_issuer_publication(jwk: &Jwk) -> Jwks {
    let JwkMaterial::EcP256 { x, y } = &jwk.material else {
        panic!("fixture mints ES256/P-256 only");
    };
    let mut components = BTreeMap::new();
    components.insert("crv".to_owned(), "P-256".to_owned());
    components.insert("x".to_owned(), x.clone());
    components.insert("y".to_owned(), y.clone());

    let mut signing_key =
        SigningKey::provision(&jwk.kid, IssuerAlgorithm::Es256, components).expect("signing key");
    signing_key.activate(NOW).expect("activate signing key");
    let issuer_jwks = build_issuer_jwks(&[signing_key]);
    let keys: Vec<Value> = issuer_jwks
        .keys()
        .iter()
        .map(|key| {
            let mut value = serde_json::Map::new();
            value.insert("kid".to_owned(), json!(key.kid));
            value.insert("kty".to_owned(), json!(key.kty));
            value.insert("alg".to_owned(), json!(key.alg));
            value.insert("use".to_owned(), json!(key.key_use));
            for (name, component) in &key.public_components {
                value.insert(name.clone(), json!(component));
            }
            Value::Object(value)
        })
        .collect();
    let issuer_jwks_json = json!({ "keys": keys }).to_string();
    Jwks::from_jwks_json(&issuer_jwks_json).expect("issuer JWKS parses for verifier")
}

#[tokio::test]
async fn issuer_published_es256_jwks_validates_offline_and_policy_deny_is_403() {
    let minted = mint_issuer_kernel_access_token();
    // The verifier receives only the issuer-published JWKS document. No
    // introspection/token-status callback is configured or needed on the hot path.
    let verifier_jwks = verifier_jwks_from_issuer_publication(&minted.jwk);
    let state = provisioned_state_with_jwks(verifier_jwks);
    let router = build_router(state);

    let (validate_status, validate_body) = post_json_bearer(
        router.clone(),
        "/tokens/validate",
        json!({ "token": minted.token }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(validate_status, StatusCode::OK);
    assert_eq!(validate_body["tenantId"], "ten_acme");
    assert_eq!(validate_body["workloadId"], "wl_secrets_sync");

    let (authorize_status, authorize_body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Encrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(authorize_status, StatusCode::FORBIDDEN);
    assert_eq!(authorize_body["effect"], "DENY");
    assert_eq!(authorize_body["reason"]["kind"], "defaultDeny");
}

#[tokio::test]
async fn authorize_with_token_permit_is_200_allow_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["effect"], "ALLOW");
    // Exactly one authorize audit record, marked allow.
    assert_eq!(audit.len(), 1);
    let record = &audit.records()[0];
    assert_eq!(record.event(), AuditEvent::Authorize);
    assert_eq!(record.outcome(), "allow");
    assert_eq!(record.workload_id(), Some("wl_secrets_sync"));
}

#[tokio::test]
async fn authorize_with_token_policy_deny_is_403_forbidden() {
    let minted = mint_token();
    // Empty authorizer -> default-deny.
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        CedarWorkloadAuthorizer::new(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    // A deny is a 403 — NEVER a 404.
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["effect"], "DENY");
    assert_eq!(body["reason"]["kind"], "defaultDeny");
}

#[tokio::test]
async fn invalid_token_on_authorize_is_422() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": "not-a-valid-jwt",
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");
    // Still emits exactly one audit record (token-rejected).
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].outcome(), "token-rejected");
}

#[tokio::test]
async fn unknown_subject_is_403_not_404() {
    let minted = mint_token();
    // Empty repository: the token validates but no persisted principal exists.
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        InMemoryWorkloadPrincipalRepository::new(),
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    // An unknown subject must NOT be a 404 — it is a fail-closed 403.
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn store_unavailable_is_503_fail_closed() {
    let minted = mint_token();
    let state: SharedState<FailingRepository, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        FailingRepository,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "DEPENDENCY_UNAVAILABLE");
}

#[tokio::test]
async fn tokens_validate_returns_principal_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/tokens/validate",
        json!({ "token": minted.token }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tenantId"], "ten_acme");
    assert_eq!(body["workloadId"], "wl_secrets_sync");
    assert_eq!(body["trustDomain"], "spiffe://ten_acme");
    assert_eq!(body["state"], "active");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].event(), AuditEvent::TokenValidation);
    assert_eq!(audit.records()[0].outcome(), "validated");
}

#[tokio::test]
async fn tokens_validate_failure_is_422_and_audits() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/tokens/validate",
        json!({ "token": "garbage" }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");
    assert_eq!(audit.records()[0].outcome(), "validation-failed");
}

#[tokio::test]
async fn authorize_already_verified_principal_permit() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_acme",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["effect"], "ALLOW");
}

#[tokio::test]
async fn authorize_batch_returns_per_item_decisions() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize:batch",
        json!({
            "requests": [
                {
                    "token": minted.token,
                    "action": "cloud.kms.Decrypt",
                    "resource": { "resourceType": "Secret", "resourceId": "db-password" }
                },
                {
                    "token": minted.token,
                    "action": "cloud.kms.Encrypt",
                    "resource": { "resourceType": "Secret", "resourceId": "db-password" }
                }
            ]
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // First permits (decrypt), second default-denies (encrypt not permitted).
    assert_eq!(body["decisions"][0]["effect"], "ALLOW");
    assert_eq!(body["decisions"][1]["effect"], "DENY");
    // One audit record per batch item.
    assert_eq!(audit.len(), 2);
}

#[tokio::test]
async fn suspend_then_authorize_is_403() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    // Suspend wl_secrets_sync (verified same-tenant caller).
    let (suspend_status, suspend_body) = post_json_bearer(
        router.clone(),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(suspend_status, StatusCode::OK);
    assert_eq!(suspend_body["state"], "suspended");

    // The now-revoked principal is denied on the hot path (403, not 404).
    let (status, _body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn retire_unknown_principal_is_404() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/principals/wl_ghost:retire",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn suspend_invalid_id_is_400() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/principals/not-a-wl-id:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// =====================================================================
// AUTH-005 / ADR-0581: mutating lifecycle control-plane authz seam proofs.
// These RED/GREEN tests fail if the verified-caller + PDP gate is removed.
// =====================================================================

/// BYPASS-CLOSED: a self-attested `x-principal-*` header (no verified bearer)
/// MUST NOT authorize a mutation — the route returns 401. This is the proof
/// that fabricated caller identity cannot reach the mutation.
#[tokio::test]
async fn lifecycle_self_attested_headers_are_401() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/principals/wl_secrets_sync:suspend")
        .header("content-type", "application/json")
        // Forged self-attested identity headers — must be ignored.
        .header("x-principal-id", "attacker")
        .header("x-principal-tenant", "ten_acme")
        .header("x-authorization-decision", "allow")
        .body(Body::from("{}"))
        .expect("request");
    let response = router.oneshot(request).await.expect("response");
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "self-attested headers must NOT authorize a mutation"
    );
}

/// No bearer at all -> 401 (default-deny without a verified principal).
#[tokio::test]
async fn lifecycle_no_bearer_is_401() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json(router, "/principals/wl_secrets_sync:suspend", json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
}

/// Wrong bearer -> 401 (constant-time compare rejects a non-matching token).
#[tokio::test]
async fn lifecycle_wrong_bearer_is_401() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        "not-the-right-token",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// CROSS-TENANT: a VERIFIED caller in a DIFFERENT tenant attempting to suspend
/// `ten_acme`'s principal -> 403. The verifier mints a caller in `ten_other`;
/// the tenant-scoped PDP denies because the handler binds the TARGET's tenant
/// (`ten_acme`), not the caller's. Proves no IDOR / cross-tenant blast radius.
#[tokio::test]
async fn lifecycle_cross_tenant_caller_is_403() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    // Verified caller is bound to a DIFFERENT tenant.
    let cross_tenant_verifier = Arc::new(BearerCallerVerifier::new(
        LIFECYCLE_BEARER,
        "ten_other",
        "other-tenant-control-plane",
    ));
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        cross_tenant_verifier,
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a verified cross-tenant caller must be denied"
    );
    assert_eq!(body["error"]["code"], "FORBIDDEN");
}

/// CROSS-TENANT BINDING (the strong proof): even with an ALLOW-ALL authorizer,
/// the handler must hand the PDP the TARGET's real tenant. We assert the happy
/// path still works with allow-all (sanity), then `lifecycle_cross_tenant_caller_is_403`
/// proves binding under the tenant-scoped authorizer. Together they prove the
/// handler does not flatten to the caller's tenant.
#[tokio::test]
async fn lifecycle_allow_all_permits_same_tenant() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(AllowAllLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "suspended");
}

/// PDP-DENY -> 403 (verified caller, explicit deny from the authorizer).
#[tokio::test]
async fn lifecycle_pdp_deny_is_403() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    // Verified same-tenant caller, but a DIFFERENT-tenant target binding via a
    // caller bound to a non-matching tenant forces the same-tenant PDP to deny.
    let deny_verifier = Arc::new(BearerCallerVerifier::new(
        LIFECYCLE_BEARER,
        "ten_mismatch",
        "control-plane",
    ));
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        deny_verifier,
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:retire",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

/// PDP-FAULT -> 403 (a PDP `Err`/outage maps to fail-closed deny, never 500/allow).
#[tokio::test]
async fn lifecycle_pdp_fault_is_403_not_500() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(FaultingLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a PDP fault must fail closed to 403, never 500 or allow"
    );
}

/// PDP-FAULT audit detail is DISTINCT from a policy-deny detail. Both return 403
/// but a PDP outage must be distinguishable from an intentional deny in the audit
/// chain so incident response can tell misconfiguration from a real policy block.
#[tokio::test]
async fn lifecycle_pdp_fault_audit_detail_is_distinct_from_policy_deny() {
    let minted = mint_token();

    // --- fault case: FaultingLifecycleAuthorizer ---
    let mut repo_fault = InMemoryWorkloadPrincipalRepository::new();
    provision(
        &mut repo_fault,
        "ten_acme",
        "wl_secrets_sync",
        "cap.cloud.kms",
    )
    .expect("provision");
    activate(
        &mut repo_fault,
        &WorkloadId::new("wl_secrets_sync").unwrap(),
    )
    .expect("activate");
    let audit_fault = InMemoryAuditSink::new();
    let state_fault: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo_fault,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        audit_fault.clone(),
        lifecycle_verifier(),
        Arc::new(FaultingLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let (status_fault, _) = post_json_bearer(
        build_router(state_fault),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status_fault, StatusCode::FORBIDDEN);
    let fault_records: Vec<AuditRecord> = audit_fault
        .records()
        .into_iter()
        .filter(|r| r.event() == AuditEvent::Authorize && r.outcome() == "deny")
        .collect();
    assert_eq!(fault_records.len(), 1, "exactly one deny audit record");
    assert_eq!(
        fault_records[0].detail(),
        Some("lifecycle-pdp-fault"),
        "PDP fault must emit 'lifecycle-pdp-fault', not 'lifecycle-forbidden'"
    );

    // --- policy-deny case: cross-tenant caller forces SameTenantLifecycleAuthorizer to deny ---
    let mut repo_deny = InMemoryWorkloadPrincipalRepository::new();
    provision(
        &mut repo_deny,
        "ten_acme",
        "wl_secrets_sync",
        "cap.cloud.kms",
    )
    .expect("provision");
    activate(&mut repo_deny, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let audit_deny = InMemoryAuditSink::new();
    let deny_verifier = Arc::new(BearerCallerVerifier::new(
        LIFECYCLE_BEARER,
        "ten_other", // different tenant -> SameTenant denies
        "other-plane",
    ));
    let state_deny: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo_deny,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        audit_deny.clone(),
        deny_verifier,
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let (status_deny, _) = post_json_bearer(
        build_router(state_deny),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status_deny, StatusCode::FORBIDDEN);
    let deny_records: Vec<AuditRecord> = audit_deny
        .records()
        .into_iter()
        .filter(|r| r.event() == AuditEvent::Authorize && r.outcome() == "deny")
        .collect();
    assert_eq!(deny_records.len(), 1, "exactly one deny audit record");
    assert_eq!(
        deny_records[0].detail(),
        Some("lifecycle-forbidden"),
        "policy deny must emit 'lifecycle-forbidden', not 'lifecycle-pdp-fault'"
    );
}

/// AUDIT ATTRIBUTION: the lifecycle audit record for an authorized allow (and
/// deny) must carry the verified caller's id and tenant — not None — so incident
/// response can answer "WHO authorized the retire/suspend".
#[tokio::test]
async fn lifecycle_audit_records_caller_attribution() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let audit = InMemoryAuditSink::new();
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        audit.clone(),
        lifecycle_verifier(), // bound to LIFECYCLE_CALLER_ID / LIFECYCLE_CALLER_TENANT
        Arc::new(AllowAllLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));

    let (status, _) = post_json_bearer(
        build_router(state),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let allow_records: Vec<AuditRecord> = audit
        .records()
        .into_iter()
        .filter(|r| r.event() == AuditEvent::Authorize && r.outcome() == "allow")
        .collect();
    assert_eq!(allow_records.len(), 1, "exactly one allow audit record");
    assert_eq!(
        allow_records[0].caller_id(),
        Some(LIFECYCLE_CALLER_ID),
        "audit record must carry the verified caller_id"
    );
    assert_eq!(
        allow_records[0].caller_tenant(),
        Some(LIFECYCLE_CALLER_TENANT),
        "audit record must carry the verified caller_tenant"
    );
}

// =====================================================================
// AUTH-005 keystone: the READ decision surfaces (/authorize,
// /authorize-with-token, /authorize:batch, /tokens/validate) require a
// VERIFIED caller and a fail-closed same-tenant decision gate. These RED/GREEN
// proofs fail on origin/dev today (where /authorize was unauthenticated and the
// authorized principal was built entirely from the caller-supplied body).
// =====================================================================

/// /authorize with NO bearer is 401 (the keystone: before the fix a forged body
/// over plain TCP authorized an arbitrary principal; now authn is required first).
#[tokio::test]
async fn authorize_no_bearer_is_401() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_acme",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
}

/// /authorize with a FORGED (wrong) bearer is 401 (constant-time compare rejects).
#[tokio::test]
async fn authorize_forged_bearer_is_401() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_acme",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        "forged-not-the-bearer",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// CROSS-TENANT ENTITLEMENT: a VERIFIED caller in `ten_acme` asserting a body
/// `tenantId: ten_evil` is 403 — the same-tenant decision gate denies a caller
/// asking for a decision over another tenant's subject (no forged-body ALLOW).
#[tokio::test]
async fn authorize_cross_tenant_body_is_403() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_evil",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a verified caller must not obtain a cross-tenant decision"
    );
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    // Exactly one deny record, carrying the distinct cross-tenant detail + caller.
    let deny: Vec<_> = audit
        .records()
        .into_iter()
        .filter(|r| r.outcome() == "deny")
        .collect();
    assert_eq!(deny.len(), 1);
    assert_eq!(deny[0].detail(), Some("decision-forbidden"));
    assert_eq!(deny[0].caller_tenant(), Some(LIFECYCLE_CALLER_TENANT));
}

/// PDP-FAULT on /authorize is a fail-closed 403, never a 5xx (a decision-PDP
/// outage never allows and never 500s).
#[tokio::test]
async fn authorize_decision_pdp_fault_is_403_not_500() {
    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    let audit = InMemoryAuditSink::new();
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        audit.clone(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(FaultingDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/authorize",
        json!({
            "tenantId": "ten_acme",
            "workloadId": "wl_secrets_sync",
            "owningCapability": "cap.cloud.kms",
            "scopes": ["cloud.kms.decrypt"],
            "claims": {},
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a decision-PDP fault must fail closed to 403, never 500 or allow"
    );
    let deny: Vec<_> = audit
        .records()
        .into_iter()
        .filter(|r| r.outcome() == "deny")
        .collect();
    assert_eq!(deny.len(), 1);
    assert_eq!(
        deny[0].detail(),
        Some("decision-pdp-fault"),
        "a PDP fault must be distinguishable from a policy deny in the audit chain"
    );
}

/// The token-bearing decision surfaces also require a verified caller: no bearer
/// is 401 on /authorize-with-token, /authorize:batch and /tokens/validate.
#[tokio::test]
async fn token_decision_surfaces_no_bearer_are_401() {
    let minted = mint_token();

    let state = provisioned_state(minted.jwk.clone());
    let (status, _) = post_json(
        build_router(state),
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "with-token requires a caller"
    );

    let state = provisioned_state(minted.jwk.clone());
    let (status, _) = post_json(
        build_router(state),
        "/authorize:batch",
        json!({ "requests": [
            { "token": minted.token, "action": "cloud.kms.Decrypt",
              "resource": { "resourceType": "Secret", "resourceId": "db-password" } }
        ] }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "batch requires a caller");

    let state = provisioned_state(minted.jwk.clone());
    let (status, _) = post_json(
        build_router(state),
        "/tokens/validate",
        json!({ "token": minted.token }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "validate requires a caller"
    );
}

/// CROSS-TENANT on /authorize-with-token: a verified `ten_acme` caller presenting
/// a (validly-signed) token whose subject is in a DIFFERENT tenant is 403 — the
/// subject tenant comes from the VALIDATED token, not the caller, so a stolen
/// cross-tenant token cannot be replayed by another tenant's control plane.
#[tokio::test]
async fn authorize_with_token_cross_tenant_caller_is_403() {
    let minted = mint_token(); // token subject is ten_acme
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").expect("provision");
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).expect("activate");
    // Verified caller is bound to a DIFFERENT tenant than the token's subject.
    let cross_tenant_verifier = Arc::new(BearerCallerVerifier::new(
        LIFECYCLE_BEARER,
        "ten_other",
        "other-control-plane",
    ));
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        cross_tenant_verifier,
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let router = build_router(state);

    let (status, _body) = post_json_bearer(
        router,
        "/authorize-with-token",
        json!({
            "token": minted.token,
            "action": "cloud.kms.Decrypt",
            "resource": { "resourceType": "Secret", "resourceId": "db-password" }
        }),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a cross-tenant token must not be authorized by another tenant's caller"
    );
}
