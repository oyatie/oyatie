// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Router-level tests for the Cedar policy publish REST edge.
//!
//! Each test drives the real axum router via `tower::ServiceExt::oneshot`,
//! asserting HTTP status + JSON body shape.  No business logic lives here;
//! the boundary-fn integration tests in `cedar_policy_publish_api.rs` cover
//! the domain rules end-to-end.
//!
//! The router is fail-closed by construction (task #124 / ADR-0572): every
//! router is built with a REQUIRED authz provider (a constant-time bearer
//! principal verifier + a PDP authorizer port), and the happy-path requests
//! present a valid `Authorization: Bearer` credential.  The AUTH-005 security
//! tests at the bottom prove the self-attestation bypass is closed.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

use iam_policy_cedar_api::authz::{
    CedarPolicyAuthzProvider, ConfiguredBearerPrincipalVerifier, PublishAuthorizationError,
    PublishAuthorizer, PublishResource, PublishScope, VerifiedPrincipal,
};
use iam_policy_cedar_api::rest::{CedarPolicyRestState, build_router};

// ── Test constants ──────────────────────────────────────────────────────────

const REQUEST_ID: &str = "req_rest_001";
const TENANT_ID: &str = "ten_platform";
const IDEMPOTENCY_KEY: &str = "idem_rest_001";
const PRINCIPAL_TENANT_ID: &str = "ten_platform";
const PRINCIPAL_ID: &str = "usr_platform_admin";
const DECISION_ID: &str = "authz_cedar_publish_001";
const POLICY_ID: &str = "pol_tenant_admin";
const VERSION: &str = "1.0.0";
const BEARER_SECRET: &str = "s3cr3t-cedar-publish-token";
const BEARER_HEADER: &str = "Bearer s3cr3t-cedar-publish-token";

// ── Test authz provider ───────────────────────────────────────────────────────

/// A [`PublishAuthorizer`] that models a platform-admin PDP decision:
/// - `Tenant`-scoped publish: allowed only when the verified principal's tenant
///   owns the resource tenant (cross-tenant publish → denied).
/// - `Global`-scoped publish: allowed only for the platform tenant principal
///   (platform-admin authority required; a non-platform principal → denied).
///
/// This is the authorizer that proves the scope is **explicit** in the
/// resource: the test `publish_global_scope_allowed_for_platform_admin` passes
/// because the authorizer grants `Global`, while
/// `publish_global_scope_denied_for_tenant_admin_returns_403` uses an
/// authorizer that denies `Global` even when it would allow `Tenant`.
struct PlatformTenantAuthorizer;

impl PublishAuthorizer for PlatformTenantAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError> {
        match &resource.scope {
            PublishScope::Tenant => {
                // Tenant-scoped: principal must own the resource tenant.
                if principal.tenant_id() == TENANT_ID && resource.tenant_id == TENANT_ID {
                    Ok(())
                } else {
                    Err(PublishAuthorizationError::Denied)
                }
            }
            PublishScope::Global => {
                // Global-scoped: requires platform-admin authority (principal
                // must be the platform tenant). A mere tenant-A admin sending
                // scope:global must be denied by a tenant-A-only authorizer.
                if principal.tenant_id() == TENANT_ID {
                    Ok(())
                } else {
                    Err(PublishAuthorizationError::Denied)
                }
            }
        }
    }
}

/// A [`PublishAuthorizer`] that allows ONLY tenant-scoped publish for the
/// platform tenant, but DENIES global-scoped publish entirely. Used to prove
/// the CRITICAL fix: a principal of tenant-A sending `scope:global` is denied
/// even when the same principal is allowed for tenant-A-scoped publish — i.e.,
/// global publish is NOT silently authorized as a per-tenant resource.
struct TenantOnlyAuthorizer;

impl PublishAuthorizer for TenantOnlyAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError> {
        match &resource.scope {
            // Allow tenant-scoped publish for the platform tenant.
            PublishScope::Tenant
                if principal.tenant_id() == TENANT_ID && resource.tenant_id == TENANT_ID =>
            {
                Ok(())
            }
            // Deny everything else — including Global, even for the platform tenant.
            _ => Err(PublishAuthorizationError::Denied),
        }
    }
}

/// A [`PublishAuthorizer`] that panics on every call. Used to prove that a
/// panicking PDP is mapped to 403 (fail-closed), not 500.
struct PanicAuthorizer;

impl PublishAuthorizer for PanicAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError> {
        panic!("PDP panic: simulating an authorizer that panics");
    }
}

/// A [`PublishAuthorizer`] that denies everything (default-deny).
struct DenyAllAuthorizer;

impl PublishAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &PublishResource,
    ) -> Result<(), PublishAuthorizationError> {
        Err(PublishAuthorizationError::Denied)
    }
}

/// Build an authz provider whose bearer verifier binds the platform principal,
/// paired with the supplied authorizer.
fn authz_provider(authorizer: Arc<dyn PublishAuthorizer>) -> CedarPolicyAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, PRINCIPAL_TENANT_ID)
            .expect("bearer verifier constructs with a non-empty secret"),
    );
    CedarPolicyAuthzProvider::new(verifier, authorizer)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Build a fresh router with default (empty) state and the platform authorizer.
fn fresh_router() -> axum::Router {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        PlatformTenantAuthorizer,
    ))));
    build_router(state)
}

/// Build a router whose authorizer denies every request (for 403 tests).
fn deny_all_router() -> axum::Router {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        DenyAllAuthorizer,
    ))));
    build_router(state)
}

/// Build a router whose authorizer allows tenant-scoped publish but DENIES
/// global-scoped publish. Used for the CRITICAL escalation RED test.
fn tenant_only_router() -> axum::Router {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        TenantOnlyAuthorizer,
    ))));
    build_router(state)
}

/// Build a router whose authorizer panics on every call. Used to prove
/// panicking PDP maps to 403 (fail-closed).
fn panic_authorizer_router() -> axum::Router {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        PanicAuthorizer,
    ))));
    build_router(state)
}

/// POST to the publish route with the canonical test headers and body.
async fn post_publish(
    router: axum::Router,
    policy_id: &str,
    version: &str,
    body: Value,
) -> (StatusCode, Value) {
    post_publish_with_overrides(router, policy_id, version, body, &[]).await
}

/// POST with optional header overrides (name, value) that REPLACE the defaults.
/// A default header is omitted entirely when an override supplies the same name.
async fn post_publish_with_overrides(
    router: axum::Router,
    policy_id: &str,
    version: &str,
    body: Value,
    overrides: &[(&str, &str)],
) -> (StatusCode, Value) {
    // Build a lookup of which default names are being overridden.
    let override_names: std::collections::HashSet<&str> =
        overrides.iter().map(|(name, _)| *name).collect();

    // Default headers — only included when not overridden.
    let defaults: &[(&str, &str)] = &[
        ("authorization", BEARER_HEADER),
        ("x-request-id", REQUEST_ID),
        ("x-tenant-id", TENANT_ID),
        ("idempotency-key", IDEMPOTENCY_KEY),
        ("x-principal-tenant-id", PRINCIPAL_TENANT_ID),
        ("x-principal-id", PRINCIPAL_ID),
        ("x-authorization-decision-id", DECISION_ID),
        ("x-authorization-tenant-id", TENANT_ID),
        ("x-authorization-principal-id", PRINCIPAL_ID),
        ("x-authorization-surfaces", "cedar.policy.publish"),
    ];

    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("/policies/{policy_id}/versions/{version}"))
        .header("content-type", "application/json");

    // Apply defaults not being overridden.
    for (name, value) in defaults {
        if !override_names.contains(*name) {
            builder = builder.header(*name, *value);
        }
    }
    // Apply overrides (only non-empty values are meaningful; empty strings
    // are intentionally omitted so the header is absent entirely).
    for (name, value) in overrides {
        if !value.is_empty() {
            builder = builder.header(*name, *value);
        }
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

/// Default valid request body (tenant-scoped to the platform tenant).
fn valid_body(policy_id: &str, version: &str) -> Value {
    json!({
        "policy_id": policy_id,
        "version": version,
        "scope": { "kind": "tenant", "tenant_id": "ten_platform" },
        "supersedes": null,
        "rules": [{
            "effect": "allow",
            "principal_role": "tenant-admin",
            "action": "tenant.settings.update",
            "resource_prefix": "tenant:",
            "required_attribute": { "key": "region", "value": "region-home" }
        }]
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn publish_happy_path_returns_201_created() {
    let (status, body) = post_publish(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["policy_id"], POLICY_ID);
    assert_eq!(body["data"]["version"], VERSION);
    assert_eq!(body["data"]["scope"]["kind"], "tenant");
    assert_eq!(body["data"]["scope"]["tenant_id"], "ten_platform");
    assert_eq!(body["data"]["rules"][0]["effect"], "allow");
    assert_eq!(body["data"]["schema_version"], 1);
    assert_eq!(body["metadata"]["request_id"], REQUEST_ID);
    assert_eq!(body["metadata"]["operator_tenant_id"], TENANT_ID);
    assert_eq!(body["metadata"]["principal_id"], PRINCIPAL_ID);
}

#[tokio::test]
async fn publish_idempotent_replay_returns_201_same_body() {
    // Use shared state so the second POST sees the ledger entry from the first.
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        PlatformTenantAuthorizer,
    ))));
    let router = build_router(state);

    let body = valid_body(POLICY_ID, VERSION);

    let (status1, body1) = post_publish(router.clone(), POLICY_ID, VERSION, body.clone()).await;
    let (status2, body2) = post_publish(router, POLICY_ID, VERSION, body).await;

    assert_eq!(status1, StatusCode::CREATED);
    assert_eq!(status2, StatusCode::CREATED);
    assert_eq!(body1["data"]["policy_id"], body2["data"]["policy_id"]);
    assert_eq!(body1["data"]["version"], body2["data"]["version"]);
}

#[tokio::test]
async fn publish_path_body_policy_id_mismatch_returns_400() {
    // Body has a different policy_id than path.
    let mut body = valid_body(POLICY_ID, VERSION);
    body["policy_id"] = json!("pol_other");

    let (status, resp_body) = post_publish(fresh_router(), POLICY_ID, VERSION, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_POLICY_PATH_BODY_MISMATCH"
    );
}

#[tokio::test]
async fn publish_authorization_denied_returns_403() {
    // Proves the PDP seam: the deny comes from the PDP authorizer (DenyAllAuthorizer
    // via deny_all_router()), NOT from the legacy x-authorization-surfaces
    // header-consistency check. A valid bearer is presented so the principal is
    // verified; the deny is purely from the PDP decision.
    let (status, resp_body) = post_publish(
        deny_all_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

#[tokio::test]
async fn publish_duplicate_version_returns_409_conflict() {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        PlatformTenantAuthorizer,
    ))));
    let router = build_router(state);
    let body = valid_body(POLICY_ID, VERSION);

    // First publish succeeds.
    let (status1, _) = post_publish(router.clone(), POLICY_ID, VERSION, body.clone()).await;
    assert_eq!(status1, StatusCode::CREATED);

    // Second publish of same version with a different idempotency key → conflict.
    let (status2, resp_body) = post_publish_with_overrides(
        router,
        POLICY_ID,
        VERSION,
        body,
        &[("idempotency-key", "idem_rest_002")],
    )
    .await;

    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_KERNEL_VERSION_ALREADY_EXISTS"
    );
}

#[tokio::test]
async fn publish_reused_idempotency_key_with_changed_body_returns_422() {
    let state = Arc::new(CedarPolicyRestState::with_authz(authz_provider(Arc::new(
        PlatformTenantAuthorizer,
    ))));
    let router = build_router(state);

    // First publish succeeds.
    let body = valid_body(POLICY_ID, VERSION);
    let (status1, _) = post_publish(router.clone(), POLICY_ID, VERSION, body).await;
    assert_eq!(status1, StatusCode::CREATED);

    // Same idempotency key but different body (changed action).
    let mut changed_body = valid_body(POLICY_ID, VERSION);
    changed_body["rules"][0]["action"] = json!("tenant.settings.read");

    let (status2, resp_body) = post_publish(router, POLICY_ID, VERSION, changed_body).await;

    assert_eq!(status2, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_IDEMPOTENCY_KEY_REUSED"
    );
}

#[tokio::test]
async fn publish_invalid_scope_kind_returns_400() {
    let mut body = valid_body(POLICY_ID, VERSION);
    body["scope"]["kind"] = json!("workspace");

    let (status, resp_body) = post_publish(fresh_router(), POLICY_ID, VERSION, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_SCOPE_KIND_INVALID"
    );
}

#[tokio::test]
async fn publish_global_scope_without_tenant_returns_201() {
    let mut body = valid_body("pol_global_reader", VERSION);
    body["policy_id"] = json!("pol_global_reader");
    body["scope"] = json!({ "kind": "global" });

    let (status, resp_body) =
        post_publish(fresh_router(), "pol_global_reader", VERSION, body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp_body["data"]["scope"]["kind"], "global");
    assert!(resp_body["data"]["scope"]["tenant_id"].is_null());
}

#[tokio::test]
async fn publish_missing_request_id_header_returns_400() {
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-request-id", "")],
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_REQUEST_ID_EMPTY");
}

// ── AUTH-005 security tests (task #124 / ADR-0572) ─────────────────────────────
//
// These prove the unauthenticated-control-plane bypass is closed: the verified
// principal — not the self-attested headers — gates the publish.

#[tokio::test]
async fn publish_without_bearer_returns_401() {
    // No Authorization header at all → unauthenticated → 401.
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("authorization", "")],
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_PRINCIPAL_UNVERIFIED"
    );
}

#[tokio::test]
async fn publish_with_wrong_bearer_returns_401() {
    // A presented but invalid bearer → unauthenticated → 401 (constant-time
    // compare; never naive ==).
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("authorization", "Bearer not-the-real-token")],
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_PRINCIPAL_UNVERIFIED"
    );
}

#[tokio::test]
async fn self_attested_headers_without_verified_principal_returns_401_not_ok() {
    // THE BYPASS PROOF: an attacker sets ALL the x-principal-*/x-authorization-*
    // headers consistently (which the legacy validate_authorization accepted),
    // but presents NO verified credential. This MUST be 401, NOT 201/Ok.
    let attacker_headers: &[(&str, &str)] = &[
        ("authorization", ""), // no credential
        ("x-principal-id", "usr_attacker"),
        ("x-principal-tenant-id", "ten_platform"),
        ("x-authorization-decision-id", "authz_forged_001"),
        ("x-authorization-tenant-id", "ten_platform"),
        ("x-authorization-principal-id", "usr_attacker"),
        ("x-authorization-surfaces", "cedar.policy.publish"),
    ];

    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        attacker_headers,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "self-attested headers without a verified principal must be 401, not Ok"
    );
    assert_eq!(
        resp_body["error"]["code"],
        "CEDAR_POLICY_PRINCIPAL_UNVERIFIED"
    );
}

#[tokio::test]
async fn publish_verified_principal_acting_as_other_tenant_returns_403() {
    // Verified principal is bound to ten_platform, but the request claims a
    // different operator tenant (x-tenant-id). The cross-tenant guard denies
    // with 403 — a verified principal of tenant A may not operate as tenant B.
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-tenant-id", "ten_other")],
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

#[tokio::test]
async fn cross_tenant_publish_attempt_is_denied_by_pdp() {
    // Principal of ten_platform attempts to publish a policy SCOPED to a
    // different tenant (ten_victim). The PDP authorizer keys on the resource
    // tenant and denies (403) — cross-tenant publish is forbidden.
    let mut body = valid_body(POLICY_ID, VERSION);
    body["scope"] = json!({ "kind": "tenant", "tenant_id": "ten_victim" });

    let (status, resp_body) = post_publish(fresh_router(), POLICY_ID, VERSION, body).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

#[tokio::test]
async fn publish_authenticated_but_pdp_denies_returns_403() {
    // Valid bearer (authenticated) but the PDP authorizer denies → 403.
    // This proves the PDP seam (not legacy header checks): the deny comes from
    // DenyAllAuthorizer which would otherwise pass header-consistency checks.
    let (status, resp_body) = post_publish(
        deny_all_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

#[tokio::test]
async fn forged_authorization_headers_do_not_affect_recorded_principal() {
    // Audit-evidence integrity: a caller who presents a valid bearer but forges
    // x-authorization-principal-id / x-authorization-tenant-id to claim a
    // different identity must NOT have that forged identity recorded. The
    // metadata in the 201 response must always reflect the VERIFIED principal,
    // not the caller-supplied headers.
    let (status, body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[
            ("x-authorization-principal-id", "usr_forged_attacker"),
            ("x-authorization-tenant-id", "ten_forged"),
        ],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "a valid bearer with forged authz headers should still publish (bearer is the gate)"
    );
    // The recorded principal_id MUST be the verified identity, not the forged header.
    assert_eq!(
        body["metadata"]["principal_id"], PRINCIPAL_ID,
        "metadata.principal_id must be derived from the verified credential, not x-authorization-principal-id"
    );
    assert_eq!(
        body["metadata"]["operator_tenant_id"], TENANT_ID,
        "metadata.operator_tenant_id must be derived from the verified credential, not x-authorization-tenant-id"
    );
}

#[tokio::test]
async fn absent_principal_id_header_returns_403() {
    // x-principal-id is required by this control plane's header contract.
    // Absent (empty) x-principal-id → 403: the bearer verified but the caller
    // did not assert a principal id for cross-checking.
    let (status, resp_body) = post_publish_with_overrides(
        fresh_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
        &[("x-principal-id", "")],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "absent x-principal-id must be 403 (header is required by this control plane)"
    );
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

// ── CRITICAL escalation RED test (task #124 / ADR-0572) ──────────────────────
//
// The CRITICAL fix: global-scope publish is presented to the PDP with
// `PublishScope::Global`, NOT flattened to the caller's own tenant. A
// tenant-admin who can publish tenant-scoped policies MUST NOT be able to
// publish a global policy just by setting `scope:global`.

#[tokio::test]
async fn publish_global_scope_denied_for_tenant_admin_returns_403() {
    // RED test for the CRITICAL cross-tenant escalation fix.
    //
    // Setup: TenantOnlyAuthorizer ALLOWS tenant-scoped publish for ten_platform
    // but DENIES global-scoped publish (even for the same principal).
    //
    // Attack: a verified ten_platform principal sends scope:global. Before the
    // fix, `publish_resource_tenant` mapped global → operator_tenant (ten_platform),
    // so the PDP was asked about {policy_id, tenant=ten_platform} which the
    // authorizer ALLOWS — granting platform-wide authz control. After the fix,
    // the PDP is asked about {policy_id, scope=Global} which TenantOnlyAuthorizer
    // DENIES → 403. This test FAILS if the global→own-tenant flattening returns.
    let mut global_body = valid_body("pol_global_attack", VERSION);
    global_body["policy_id"] = serde_json::json!("pol_global_attack");
    global_body["scope"] = serde_json::json!({ "kind": "global" });

    let (status, resp_body) = post_publish(
        tenant_only_router(),
        "pol_global_attack",
        VERSION,
        global_body,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "global-scope publish must be denied by an authorizer that only allows tenant-scope: \
         this FAILS if global is silently flattened to the caller's own tenant (the CRITICAL escalation)"
    );
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}

#[tokio::test]
async fn publish_pdp_panic_returns_403_not_500() {
    // In test builds (panic strategy = unwind) catch_unwind catches a panicking
    // authorizer and maps it to 403 (fail-closed), not an uncontrolled 500.
    // In release builds (panic = "abort", Cargo.toml profile.release) catch_unwind
    // is a no-op and a panicking adapter aborts the process — the real guarantee
    // in production is the PublishAuthorizer adapter contract: adapters MUST NOT
    // panic and MUST map every fault to Err(Refused). This test exercises the
    // test-environment backstop only.
    let (status, resp_body) = post_publish(
        panic_authorizer_router(),
        POLICY_ID,
        VERSION,
        valid_body(POLICY_ID, VERSION),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a panicking PDP must fail-closed to 403, not an uncontrolled 500"
    );
    assert_eq!(resp_body["error"]["code"], "CEDAR_POLICY_PUBLISH_FORBIDDEN");
}
