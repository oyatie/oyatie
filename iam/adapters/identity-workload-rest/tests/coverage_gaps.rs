//! Coverage-gap tests for identity-workload-rest.
//!
//! Fills the paths not exercised by rest_endpoints.rs and grpc_authorize_deny.rs:
//!
//! REST gaps:
//! - /authorize default-deny returns 403 (not just permit tested)
//! - /authorize:batch with a store-unavailable item returns 200 with per-item DENY
//! - /principals/{id}:retire happy path returns 200 + "retired" state
//! - /principals/{id}:retire -> authorize -> 403 (retire revokes on denylist)
//! - /principals/{id}:<unknown-verb> returns 404
//! - /principals/{id}:suspend -> suspend again -> 409 conflict
//!
//! gRPC gaps:
//! - Authorize with invalid principal fields -> fail-closed DENY (not tonic Err)
//! - AuthorizeBatch with store-unavailable item -> 200 with per-item DENY (not Err)
//! - ValidateToken expired token -> typed Expired error kind (ok=false, engine NOT consulted)
//! - ValidateToken audit record carries workload_id=None on failure
//! - AuthorizeWithToken for a revoked (suspended) principal -> DENY response
//!
//! Unit gaps:
//! - AuditEvent::label() returns the correct wire labels
//! - AuditRecord accessors (event, workload_id=None, outcome, detail=Some)
//! - InMemoryAuditSink::is_empty / len / records round-trip

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tonic::Request as TonicRequest;
use tower::ServiceExt as _;

use iam_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, activate, provision,
};
use iam_identity_workload_authz_cedar::CedarWorkloadAuthorizer;
use iam_identity_workload_domain::WorkloadId;
use iam_identity_workload_oidc::{Jwks, ValidationConfig};
use iam_identity_workload_rest::{
    AuditEvent, AuditRecord, AuditSink, InMemoryAuditSink, SharedState, WorkloadAuthzState,
    build_router,
    grpc::{
        WorkloadGrpcServer,
        proto::{
            AuthorizeRequest, AuthorizeWithTokenRequest, BatchAuthorizeRequest, DecisionEffect,
            ValidateTokenRequest, validate_token_response,
            workload_authorizer_server::WorkloadAuthorizer as _,
            workload_token_validator_server::WorkloadTokenValidator as _,
        },
    },
};

use common::{
    AUDIENCE, FailingRepository, ISSUER, LIFECYCLE_BEARER, NOW, SameTenantDecisionAuthorizer,
    SameTenantLifecycleAuthorizer, lifecycle_verifier, mint_token, permit_authorizer,
    provisioned_state,
};

/// Wrap a message in a tonic [`TonicRequest`] carrying the verified-caller bearer
/// in the `authorization` metadatum (gRPC analogue of the REST bearer header).
/// The caller is bound to `ten_acme`, so the same-tenant decision gate permits the
/// `ten_acme` fixtures below.
fn authed_request<T>(message: T) -> TonicRequest<T> {
    let mut request = TonicRequest::new(message);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {LIFECYCLE_BEARER}")
            .parse()
            .expect("ascii bearer"),
    );
    request
}

// =====================================================================
// Shared REST helper (mirrors rest_endpoints.rs)
// =====================================================================

async fn post_json(router: axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    post_json_inner(router, path, body, None).await
}

/// POST with an `Authorization: Bearer <token>` header (lifecycle control plane).
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

// =====================================================================
// Unit: AuditEvent::label()
// =====================================================================

#[test]
fn audit_event_authorize_label_is_workload_decision_v1() {
    assert_eq!(
        AuditEvent::Authorize.label(),
        "workload.decision.v1",
        "Authorize label must match AsyncAPI channel name"
    );
}

#[test]
fn audit_event_token_validation_label_is_workload_token_validation_v1() {
    assert_eq!(
        AuditEvent::TokenValidation.label(),
        "workload.token-validation.v1",
        "TokenValidation label must match AsyncAPI channel name"
    );
}

// =====================================================================
// Unit: AuditRecord accessors
// =====================================================================

#[test]
fn audit_record_accessors_with_no_workload_id_and_no_detail() {
    let rec = AuditRecord::new(AuditEvent::TokenValidation, None, "validation-failed", None);
    assert_eq!(rec.event(), AuditEvent::TokenValidation);
    assert_eq!(
        rec.workload_id(),
        None,
        "workload_id must be None for a forged-token record"
    );
    assert_eq!(rec.outcome(), "validation-failed");
    assert_eq!(rec.detail(), None);
}

#[test]
fn audit_record_accessors_with_workload_id_and_detail() {
    let rec = AuditRecord::new(
        AuditEvent::Authorize,
        Some("wl_secrets_sync".to_owned()),
        "deny",
        Some("default-deny".to_owned()),
    );
    assert_eq!(rec.workload_id(), Some("wl_secrets_sync"));
    assert_eq!(rec.detail(), Some("default-deny"));
}

// =====================================================================
// Unit: InMemoryAuditSink is_empty / len / records round-trip
// =====================================================================

#[test]
fn in_memory_audit_sink_is_empty_until_record_is_appended() {
    let sink = InMemoryAuditSink::new();
    assert!(sink.is_empty(), "new sink must report is_empty=true");
    assert_eq!(sink.len(), 0);
    sink.record(AuditRecord::new(AuditEvent::Authorize, None, "allow", None));
    assert!(!sink.is_empty());
    assert_eq!(sink.len(), 1);
    assert_eq!(sink.records().len(), 1);
}

// =====================================================================
// REST: /authorize default-deny returns 403 (forbidden principal)
// =====================================================================

#[tokio::test]
async fn authorize_already_verified_principal_default_deny_is_403() {
    let minted = mint_token();
    // No Cedar policies -> default deny.
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").unwrap();
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).unwrap();
    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        InMemoryRevocationDenylist::new(),
        CedarWorkloadAuthorizer::new(), // no policies -> default deny
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

    // A deny must be 403, never 404.
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["effect"], "DENY");
}

// =====================================================================
// REST: /authorize:batch store-unavailable item -> 200 with per-item DENY
// =====================================================================

#[tokio::test]
async fn authorize_batch_with_store_unavailable_returns_200_with_deny_per_item() {
    // The batch outer response is always 200; a store outage on an item
    // collapses to a DENY decision for that item (fail-closed, never an error).
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
                }
            ]
        }),
        LIFECYCLE_BEARER,
    )
    .await;

    // Outer response is 200 — the batch never errors.
    assert_eq!(status, StatusCode::OK);
    // The store-unavailable item collapses to DENY.
    assert_eq!(
        body["decisions"][0]["effect"], "DENY",
        "store-unavailable batch item must be a DENY decision"
    );
    // One audit record emitted even for the failing item.
    assert_eq!(
        audit.len(),
        1,
        "one audit record must be emitted per batch item"
    );
    assert_eq!(audit.records()[0].outcome(), "store-unavailable");
}

// =====================================================================
// REST: /principals/{id}:retire happy path -> 200 + "retired"
// =====================================================================

#[tokio::test]
async fn retire_known_principal_returns_200_with_retired_state() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:retire",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["state"], "retired",
        "retire response must report state=retired"
    );
    assert_eq!(body["workloadId"], "wl_secrets_sync");
}

// =====================================================================
// REST: retire -> authorize -> 403 (retire revokes on denylist)
// =====================================================================

#[tokio::test]
async fn retire_then_authorize_is_403_fail_closed() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    // Retire the principal (verified same-tenant caller).
    let (retire_status, _) = post_json_bearer(
        router.clone(),
        "/principals/wl_secrets_sync:retire",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(retire_status, StatusCode::OK);

    // The now-retired principal must be denied (fail-closed), never 404.
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

// =====================================================================
// REST: unknown lifecycle verb -> 404
// =====================================================================

#[tokio::test]
async fn unknown_lifecycle_verb_returns_404() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    let (status, body) = post_json(router, "/principals/wl_secrets_sync:delete", json!({})).await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "an unrecognised lifecycle verb must be 404"
    );
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

// =====================================================================
// REST: suspend twice -> 409 conflict (illegal transition)
// =====================================================================

#[tokio::test]
async fn suspend_already_suspended_principal_returns_409() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let router = build_router(state);

    // First suspend succeeds (verified same-tenant caller).
    let (first_status, _) = post_json_bearer(
        router.clone(),
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(first_status, StatusCode::OK);

    // Second suspend is an illegal domain transition -> 409 Conflict.
    let (second_status, body) = post_json_bearer(
        router,
        "/principals/wl_secrets_sync:suspend",
        json!({}),
        LIFECYCLE_BEARER,
    )
    .await;
    assert_eq!(
        second_status,
        StatusCode::CONFLICT,
        "suspending an already-suspended principal must be 409"
    );
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// =====================================================================
// gRPC: Authorize with invalid principal fields -> fail-closed DENY (not Err)
// =====================================================================

#[tokio::test]
async fn grpc_authorize_invalid_principal_fields_returns_deny_not_error() {
    let minted = mint_token();
    let state = provisioned_state(minted.jwk.clone());
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    // tenant_id without the required "ten_" prefix -> build_active_principal fails.
    let result = server
        .authorize(authed_request(AuthorizeRequest {
            // Same-tenant as the verified caller (so the AUTH-005 decision gate
            // permits), but an invalid workload_id (no `wl_` prefix) so
            // build_active_principal fails -> fail-closed DENY (not a tonic Err).
            tenant_id: "ten_acme".to_owned(),
            workload_id: "secrets_sync".to_owned(), // invalid: missing "wl_" prefix
            owning_capability: "cap.cloud.kms".to_owned(),
            scopes: vec!["cloud.kms.decrypt".to_owned()],
            claims: Default::default(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(iam_identity_workload_rest::grpc::proto::Resource {
                resource_type: "Secret".to_owned(),
                resource_id: "db-password".to_owned(),
                attributes: Default::default(),
            }),
            context: Default::default(),
        }))
        .await;

    // Must return Ok(Response{DENY}) — never a tonic Err (fail-closed PEP invariant).
    assert!(
        result.is_ok(),
        "an invalid principal must produce a DENY response, not a tonic Err"
    );
    assert_eq!(
        result.unwrap().into_inner().effect,
        DecisionEffect::Deny as i32,
        "invalid principal fields must fail-closed to DECISION_EFFECT_DENY"
    );
    // One audit record emitted for the deny.
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].outcome(), "deny");
    assert_eq!(
        audit.records()[0].detail(),
        Some("invalid-principal"),
        "invalid principal deny must carry 'invalid-principal' detail"
    );
}

// =====================================================================
// gRPC: AuthorizeBatch with store-unavailable item -> per-item DENY (not Err)
// =====================================================================

#[tokio::test]
async fn grpc_authorize_batch_store_unavailable_returns_per_item_deny_not_error() {
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
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let result = server
        .authorize_batch(authed_request(BatchAuthorizeRequest {
            requests: vec![AuthorizeWithTokenRequest {
                token: minted.token.clone(),
                action: "cloud.kms.Decrypt".to_owned(),
                resource: Some(iam_identity_workload_rest::grpc::proto::Resource {
                    resource_type: "Secret".to_owned(),
                    resource_id: "db-password".to_owned(),
                    attributes: Default::default(),
                }),
                context: Default::default(),
            }],
        }))
        .await;

    // AuthorizeBatch must never return Err — store outage -> per-item DENY.
    assert!(
        result.is_ok(),
        "batch must not return a tonic Err for a store outage"
    );
    let batch = result.unwrap().into_inner();
    assert_eq!(batch.decisions.len(), 1);
    assert_eq!(
        batch.decisions[0].effect,
        DecisionEffect::Deny as i32,
        "store-unavailable item must be DENY in the batch response"
    );
    // One audit record emitted for the failing item.
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].outcome(), "store-unavailable");
}

// =====================================================================
// gRPC: ValidateToken expired token -> typed Expired error, ok=false
// =====================================================================

#[tokio::test]
async fn grpc_validate_token_expired_returns_typed_expired_error() {
    use iam_identity_workload_rest::grpc::proto::ValidationErrorKind;

    // Mint a token signed at NOW (exp = NOW + 300) but drive the clock to
    // NOW + 1000 so the token is expired when the server evaluates it.
    let minted2 = mint_token();
    let state2: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        {
            let mut repo = InMemoryWorkloadPrincipalRepository::new();
            provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").unwrap();
            activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).unwrap();
            repo
        },
        InMemoryRevocationDenylist::new(),
        permit_authorizer(),
        Jwks::new().add_key(minted2.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        // clock is NOW + 1000, well past exp = NOW + 300
        || NOW + 1000,
    ));
    let audit2 = state2.audit().clone();
    let server2 = WorkloadGrpcServer::new(state2);

    let result = server2
        .validate_token(authed_request(ValidateTokenRequest {
            token: minted2.token.clone(),
        }))
        .await
        .expect("rpc must return Ok (typed error in body)");

    let resp = result.into_inner();
    assert!(!resp.ok, "ok must be false for an expired token");
    let err = match resp.outcome {
        Some(validate_token_response::Outcome::Error(e)) => e,
        other => panic!("expected Error outcome, got: {other:?}"),
    };
    assert_eq!(
        err.kind,
        ValidationErrorKind::Expired as i32,
        "expired token must produce ValidationErrorKind::Expired"
    );

    // Audit: one TokenValidation failure record, workload_id=None.
    assert_eq!(audit2.len(), 1);
    let rec = &audit2.records()[0];
    assert_eq!(rec.event(), AuditEvent::TokenValidation);
    assert_eq!(rec.outcome(), "validation-failed");
    assert_eq!(
        rec.workload_id(),
        None,
        "failed token validation must not carry a workload_id in the audit record"
    );
}

// =====================================================================
// gRPC: AuthorizeWithToken for a revoked (suspended) principal -> DENY
// =====================================================================

#[tokio::test]
async fn grpc_authorize_with_token_revoked_principal_returns_deny() {
    use iam_identity_workload_app::suspend;

    let minted = mint_token();
    let mut repo = InMemoryWorkloadPrincipalRepository::new();
    provision(&mut repo, "ten_acme", "wl_secrets_sync", "cap.cloud.kms").unwrap();
    activate(&mut repo, &WorkloadId::new("wl_secrets_sync").unwrap()).unwrap();

    // Suspend the principal before building the state — it lands on the denylist.
    let mut denylist = InMemoryRevocationDenylist::new();
    suspend(
        &mut repo,
        &mut denylist,
        &WorkloadId::new("wl_secrets_sync").unwrap(),
    )
    .unwrap();

    let state: SharedState<_, _, _, _> = Arc::new(WorkloadAuthzState::with_clock(
        repo,
        denylist,
        permit_authorizer(),
        Jwks::new().add_key(minted.jwk.clone()),
        ValidationConfig::new(ISSUER, AUDIENCE),
        InMemoryAuditSink::new(),
        lifecycle_verifier(),
        Arc::new(SameTenantLifecycleAuthorizer),
        Arc::new(SameTenantDecisionAuthorizer),
        || NOW,
    ));
    let audit = state.audit().clone();
    let server = WorkloadGrpcServer::new(state);

    let result = server
        .authorize_with_token(authed_request(AuthorizeWithTokenRequest {
            token: minted.token.clone(),
            action: "cloud.kms.Decrypt".to_owned(),
            resource: Some(iam_identity_workload_rest::grpc::proto::Resource {
                resource_type: "Secret".to_owned(),
                resource_id: "db-password".to_owned(),
                attributes: Default::default(),
            }),
            context: Default::default(),
        }))
        .await;

    // A revoked principal must be a DENY response value, never a tonic Err.
    assert!(
        result.is_ok(),
        "revoked principal must produce a DENY response, not a tonic Err"
    );
    assert_eq!(
        result.unwrap().into_inner().effect,
        DecisionEffect::Deny as i32,
        "suspended (revoked) principal must produce DECISION_EFFECT_DENY"
    );
    // Audit: deny emitted, label is "deny".
    assert_eq!(audit.len(), 1);
    assert_eq!(audit.records()[0].outcome(), "deny");
}
