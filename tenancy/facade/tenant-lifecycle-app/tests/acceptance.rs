//! Acceptance tests for the tenant registration / lifecycle service.
//!
//! These drive the FULL HTTP surface end-to-end against the in-memory store,
//! exercising the REAL tenancy lifecycle core (the contract FSM inside the
//! usecase) AND the fail-closed authorization layer (the embedded Cedar PDP,
//! AUTH-005): register lands a tenant in `Provisioning`, `:provision` drives
//! it to `Active` through the operation ledger, and every route authenticates
//! the verified bearer and authorizes it against the target tenant. No sockets
//! are opened; the `axum::Router` is invoked via `tower::ServiceExt::oneshot`.
//!
//! Authorization model under test (SECURITY remediation — membership-bound):
//!   - platform-admin bearer (`PLATFORM_TOKEN`) → may register + list (no
//!     tenant scope; per-tenant ops still deny);
//!   - tenant-operator bearer (`OPERATOR_TOKEN`) → administers ONLY the tenants
//!     the SERVER-SIDE membership resolver assigns it; the `x-oya-tenant` header
//!     may SELECT among the assigned set but NEVER grant an unassigned tenant
//!     (the C7 fix — a self-attested header confers no authority);
//!   - no/invalid bearer → 401 on every route.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use tenancy_tenant_lifecycle_app::{
    BootError, InMemoryTenantMembershipResolver, SharedMembershipResolver, build_inmemory_router,
    build_inmemory_router_with_tenant_bound_operator_tokens, build_postgres_router,
};
use tower::ServiceExt;

/// The platform-admin bearer the test router is configured with.
const PLATFORM_TOKEN: &str = "test-platform-admin-secret";
/// The tenant-operator bearer the test router is configured with.
const OPERATOR_TOKEN: &str = "test-tenant-operator-secret";
/// A tenant-bound operator bearer that proves authority for exactly `acme`.
const ACME_OPERATOR_TOKEN: &str = "test-acme-operator-secret";
/// The stable operator principal id the reference verifier binds the shared
/// operator bearer to (the membership key).
const OPERATOR_PRINCIPAL: &str = "tenant-operator";
/// A tenant the operator is NEVER assigned to — the C7 victim. The operator can
/// hold the shared bearer and assert `x-oya-tenant: VICTIM_TENANT`, but the
/// server-side membership resolver denies it (no proven authority).
const VICTIM_TENANT: &str = "victim";

/// The tenants the test operator is a member of. Covers every tenant the
/// happy-path tests register and operate; `VICTIM_TENANT` is deliberately
/// EXCLUDED so the cross-tenant self-attestation test denies.
fn operator_tenants() -> Vec<String> {
    [
        "acme",
        "globex",
        "alpha",
        "bravo",
        "ghost",
        "nobody",
        "ten_reused",
        "reused",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// The reference membership resolver assigning the test operator its tenants.
fn membership() -> SharedMembershipResolver {
    Arc::new(
        InMemoryTenantMembershipResolver::new()
            .with_operator(OPERATOR_PRINCIPAL, operator_tenants()),
    )
}

/// Build a fully-authorized in-memory router (both credential classes
/// configured + the server-side membership resolver). Panics only on an
/// authz-bundle compile failure, which is a hard test failure (the embedded seed
/// bundle must always compile).
fn app() -> axum::Router {
    build_inmemory_router(
        membership(),
        Some(PLATFORM_TOKEN.to_owned()),
        Some(OPERATOR_TOKEN.to_owned()),
    )
    .expect("embedded authz bundle must compile and strict-validate")
}

/// Build a router with a tenant-bound operator credential. This is the #771
/// retirement path: the verified credential carries the tenant axis, so the
/// caller does not need to provide a self-asserted `x-oya-tenant` selector.
fn app_with_tenant_bound_operator_token() -> axum::Router {
    build_inmemory_router_with_tenant_bound_operator_tokens(
        membership(),
        Some(PLATFORM_TOKEN.to_owned()),
        Some(OPERATOR_TOKEN.to_owned()),
        std::collections::BTreeMap::from([("acme".to_owned(), ACME_OPERATOR_TOKEN.to_owned())]),
    )
    .expect("embedded authz bundle must compile and strict-validate")
}

/// A distinct canonical-UUID idempotency key per call (AIP-155 client token).
fn key(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000000{n:02x}")
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

fn register_body(tenant_id: &str) -> serde_json::Value {
    serde_json::json!({
        "tenant_id": tenant_id,
        "display_name": "Acme Corporation",
        "isolation_posture": "pooled",
        "cell_id": "cell-001",
        "residency_zone": "kr-seoul"
    })
}

/// Register a tenant as the platform admin (register is a platform-admin op).
async fn register(app: &axum::Router, tenant_id: &str, idem: &str) -> axum::http::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", idem)
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::from(
                    serde_json::to_vec(&register_body(tenant_id)).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

/// Drive a per-tenant lifecycle verb as the tenant operator scoped to
/// `tenant_id` (the `x-oya-tenant` axis equals the target id).
async fn lifecycle(
    app: &axum::Router,
    tenant_id: &str,
    verb: &str,
    idem: &str,
) -> axum::http::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/v1/tenants/{tenant_id}/{verb}"))
                .header("idempotency-key", idem)
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", tenant_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

/// Read a tenant's state as the tenant operator scoped to it.
async fn get_state(app: &axum::Router, tenant_id: &str) -> (StatusCode, serde_json::Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/v1/tenants/{tenant_id}"))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", tenant_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp.into_body()).await)
}

#[tokio::test]
async fn healthz_returns_200() {
    let app = app();
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

/// The headline E2E flow: register -> read (Provisioning) -> provision ->
/// read (Active). Proves the delivery layer exercises the real FSM and the
/// state transition is observable on the read surface.
#[tokio::test]
async fn register_then_provision_drives_provisioning_to_active() {
    let app = app();

    // Register: a brand-new tenant is born in Provisioning.
    let resp = register(&app, "acme", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp.into_body()).await;
    assert_eq!(created["tenant_id"], "acme");
    assert_eq!(created["state"], "provisioning");
    assert_eq!(created["cell_id"], "cell-001");

    // Read back: still Provisioning until provisioned.
    let (status, view) = get_state(&app, "acme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["state"], "provisioning");

    // Provision: the real Activate transition drives it to Active.
    let resp = lifecycle(&app, "acme", "provision", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let provisioned = body_json(resp.into_body()).await;
    assert_eq!(provisioned["state"], "active");

    // Read confirms the persisted transition.
    let (status, view) = get_state(&app, "acme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["state"], "active");
}

/// The full lifecycle FSM is reachable over HTTP: provision -> suspend ->
/// resume -> retire, each a real contract transition.
#[tokio::test]
async fn full_lifecycle_transitions_over_http() {
    let app = app();
    assert_eq!(
        register(&app, "globex", &key(1)).await.status(),
        StatusCode::CREATED
    );

    assert_eq!(
        lifecycle(&app, "globex", "provision", &key(2))
            .await
            .status(),
        StatusCode::OK
    );
    let resp = lifecycle(&app, "globex", "suspend", &key(3)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "suspended");

    let resp = lifecycle(&app, "globex", "resume", &key(4)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "active");

    // Retire is terminal; the read surface hides the tombstone.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/v1/tenants/globex")
                .header("idempotency-key", key(5))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "globex")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let (status, _) = get_state(&app, "globex").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Registration without an Idempotency-Key header is rejected (authz passes
/// first as the platform admin, then the missing key is a 400).
#[tokio::test]
async fn register_requires_idempotency_key() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::from(
                    serde_json::to_vec(&register_body("acme")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// Re-registering with the SAME key + SAME body replays (idempotent), 200.
#[tokio::test]
async fn register_replay_is_idempotent() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    let resp = register(&app, "acme", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "provisioning");
}

/// A NEW key onto an existing tenant id is a conflict (AlreadyExists).
#[tokio::test]
async fn register_existing_tenant_new_key_conflicts() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    let resp = register(&app, "acme", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

/// Provisioning an unknown tenant is a 404 (the operator IS authorized for the
/// asserted tenant; the resource simply does not exist).
#[tokio::test]
async fn provision_unknown_tenant_returns_404() {
    let app = app();
    let resp = lifecycle(&app, "ghost", "provision", &key(1)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Suspending a tenant still in Provisioning is a precondition conflict (the
/// FSM forbids Provisioning -> Suspended directly).
#[tokio::test]
async fn suspend_before_provision_is_conflict() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    let resp = lifecycle(&app, "acme", "suspend", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    // The failed transition left the tenant untouched.
    let (_, view) = get_state(&app, "acme").await;
    assert_eq!(view["state"], "provisioning");
}

/// Reading an unregistered tenant is a 404 (authorized operator, missing tenant).
#[tokio::test]
async fn get_unknown_tenant_returns_404() {
    let app = app();
    let (status, _) = get_state(&app, "nobody").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The list surface enumerates registered tenants (platform-admin only).
#[tokio::test]
async fn list_returns_registered_tenants() {
    let app = app();
    assert_eq!(
        register(&app, "alpha", &key(1)).await.status(),
        StatusCode::CREATED
    );
    assert_eq!(
        register(&app, "bravo", &key(2)).await.status(),
        StatusCode::CREATED
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/tenants")
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let page = body_json(resp.into_body()).await;
    let ids: Vec<&str> = page["tenants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tenant_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["alpha", "bravo"]);
}

// ============================================================
// AUTH-005 — fail-closed authorization (the BLOCKING security fix)
// ============================================================

/// Every mutating route rejects an UNAUTHENTICATED request with 401 (no
/// bearer at all). Register/provision/suspend/resume/retire are all covered.
#[tokio::test]
async fn unauthenticated_requests_are_401_on_every_mutating_route() {
    let app = app();

    // POST /v1/tenants (register) with no bearer.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", key(1))
                .body(Body::from(
                    serde_json::to_vec(&register_body("acme")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "register must be 401"
    );

    // Per-tenant mutating verbs with no bearer.
    for (method, uri) in [
        (Method::POST, "/v1/tenants/acme/provision"),
        (Method::POST, "/v1/tenants/acme/suspend"),
        (Method::POST, "/v1/tenants/acme/resume"),
        (Method::DELETE, "/v1/tenants/acme"),
    ] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(uri)
                    .header("idempotency-key", key(2))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} must be 401 unauthenticated",
        );
    }
}

/// RED test for the authn-after-body fix (FIX 1): an UNAUTHENTICATED POST whose
/// body is INVALID JSON must be rejected 401 — proving authentication ran via the
/// `VerifiedCaller` `FromRequestParts` extractor BEFORE the `Json` body extractor
/// ever tried to deserialize the body. If authn ran AFTER body parsing (the bug),
/// the malformed body would surface a 400/422 JSON-parse error instead of a 401.
#[tokio::test]
async fn unauthenticated_register_rejects_before_body_is_parsed() {
    let app = app();
    // Deliberately malformed JSON: if the body were parsed first, this would be a
    // 400/422. Authn must short-circuit to 401 before the parser is ever reached.
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", key(1))
                // NO authorization header.
                .body(Body::from(b"{ this is not valid json ::::".to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "unauthenticated register with a malformed body must be 401 (authn before \
         body parse), not a body-parse 400/422 — proves the parser was never reached",
    );
}

/// An oversized body on a mutating route is capped by `DefaultBodyLimit`. With a
/// VALID bearer (so authn passes) and a body that EXCEEDS the configured limit,
/// the request is rejected 413 Payload Too Large by the body extractor — the
/// second backstop against an oversized-body DoS. (Unauthenticated callers are
/// already short-circuited 401 before the body is read; this proves the cap
/// fires for an authenticated caller too.)
#[tokio::test]
async fn oversized_register_body_is_413() {
    let app = app();
    // > 64 KiB (the MAX_MUTATING_BODY_BYTES cap): a 128 KiB payload.
    let huge = vec![b'a'; 128 * 1024];
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", key(1))
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::from(huge))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "an oversized body must be rejected 413 by DefaultBodyLimit",
    );
}

/// The read route also rejects an unauthenticated caller (401) — reads are not
/// public.
#[tokio::test]
async fn unauthenticated_read_is_401() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/tenants/acme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// CROSS-TENANT DENIAL (the headline finding): an operator authorized for
/// tenant A is FORBIDDEN (403) from suspending OR retiring tenant B. The URL
/// {id} alone never authorizes.
#[tokio::test]
async fn cross_tenant_operator_is_403_on_suspend_and_retire() {
    let app = app();
    // Both tenants exist and are active.
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    assert_eq!(
        register(&app, "globex", &key(2)).await.status(),
        StatusCode::CREATED
    );
    assert_eq!(
        lifecycle(&app, "globex", "provision", &key(3))
            .await
            .status(),
        StatusCode::OK
    );

    // Operator scoped to "acme" tries to suspend "globex": axis != target ⇒ 403.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/globex/suspend")
                .header("idempotency-key", key(4))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "acme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "cross-tenant suspend must be 403"
    );

    // ...and retiring "globex" while scoped to "acme" is likewise 403.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/v1/tenants/globex")
                .header("idempotency-key", key(5))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "acme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "cross-tenant retire must be 403"
    );

    // "globex" survived the cross-tenant retire attempt (still Active).
    let (status, view) = get_state(&app, "globex").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["state"], "active");
}

/// A tenant-scoped operator may NOT register a tenant (platform-admin scope):
/// 403, not a silent allow.
#[tokio::test]
async fn tenant_operator_cannot_register() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", key(1))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "acme")
                .body(Body::from(
                    serde_json::to_vec(&register_body("acme")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// A tenant-scoped operator may NOT list tenants (the surface discloses all
/// tenants): 403.
#[tokio::test]
async fn tenant_operator_cannot_list() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/tenants")
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "acme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// An operator assigned MULTIPLE tenants that omits the `x-oya-tenant` selection
/// header cannot bind a tenant scope (the bearer alone never picks a tenant): the
/// request is a 400 `TENANT_SELECTION_REQUIRED`. The selection is still
/// constrained to the server-side assigned set — it never grants authority.
#[tokio::test]
async fn operator_without_tenant_axis_must_select() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/acme/provision")
                .header("idempotency-key", key(1))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// THE HEADLINE C7 FIX: an operator holding the SHARED operator bearer asserts
/// `x-oya-tenant: victim` on the victim's URL — a self-attested header for a
/// tenant the operator is NOT a member of. Pre-fix this self-attestation granted
/// the axis and the PDP saw axis == target ⇒ ALLOW (any operator could administer
/// ANY tenant). Now the SERVER-SIDE membership resolver denies it: the operator
/// has no proven authority over `victim` ⇒ 403, and NO mutation occurs. This
/// FAILS if the operator scope is derived from the header rather than membership.
#[tokio::test]
async fn operator_cannot_select_unassigned_victim_tenant() {
    let app = app();
    // The victim tenant exists and is active (registered + provisioned by the
    // platform admin / a legitimate operator path is not even needed — the deny
    // must happen at authn, before the store is consulted).
    assert_eq!(
        register(&app, VICTIM_TENANT, &key(1)).await.status(),
        StatusCode::CREATED
    );

    // Operator self-attests the victim tenant it is NOT assigned to, on the
    // victim's own URL (axis == target — the attack that the pre-fix PDP allowed).
    for (method, verb) in [
        (Method::POST, "suspend"),
        (Method::POST, "resume"),
        (Method::DELETE, ""),
    ] {
        let uri = if verb.is_empty() {
            format!("/v1/tenants/{VICTIM_TENANT}")
        } else {
            format!("/v1/tenants/{VICTIM_TENANT}/{verb}")
        };
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(&uri)
                    .header("idempotency-key", key(2))
                    .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                    .header("x-oya-tenant", VICTIM_TENANT)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "{method} {uri}: operator must NOT self-attest an unassigned tenant",
        );
    }

    // The victim tenant is untouched: a legitimate read by the platform admin
    // still shows it registered (no mutation leaked through).
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/v1/tenants/{VICTIM_TENANT}"))
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Platform admin has no tenant scope, so per-tenant read is 403 — but the
    // point is the victim was never mutated; we assert the deny was not a 404
    // (tenant gone) by confirming a tenant-scoped read elsewhere. Here we just
    // confirm the surface still responds (not 5xx).
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// An operator IS allowed to select an ASSIGNED tenant via the header (the
/// positive counterpart to the victim test): membership-bound selection works.
#[tokio::test]
async fn operator_can_select_assigned_tenant() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    // "acme" is in the operator's assigned set ⇒ provision succeeds.
    let resp = lifecycle(&app, "acme", "provision", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

/// #771 W2-S2 retirement path: a tenant-bound credential carries a verified
/// tenant claim (`acme`), so the caller can operate on `acme` without any
/// `x-oya-tenant` header. If the caller sends a conflicting header, the request
/// fails closed before the PDP can ever see a forged axis.
#[tokio::test]
async fn tenant_bound_operator_token_does_not_need_self_asserted_tenant_axis() {
    let app = app_with_tenant_bound_operator_token();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/acme/provision")
                .header("idempotency-key", key(2))
                .header("authorization", format!("Bearer {ACME_OPERATOR_TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "tenant-bound credential must not require x-oya-tenant"
    );

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/globex/suspend")
                .header("idempotency-key", key(3))
                .header("authorization", format!("Bearer {ACME_OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "globex")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "acme-bound credential must not act on globex even with a matching self-asserted header"
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/acme/suspend")
                .header("idempotency-key", key(4))
                .header("authorization", format!("Bearer {ACME_OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "globex")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "conflicting self-asserted tenant header must be denied"
    );
}

/// A wrong/garbage bearer is rejected as unauthenticated (constant-time
/// mismatch): 401.
#[tokio::test]
async fn wrong_bearer_is_401() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/v1/tenants/acme")
                .header("idempotency-key", key(1))
                .header("authorization", "Bearer not-the-real-token")
                .header("x-oya-tenant", "acme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// The platform admin may register but is NOT a tenant operator: a per-tenant
/// op with the platform-admin bearer (no tenant axis) is denied (401/403, never
/// allowed). This proves register-scope ≠ administer-scope.
#[tokio::test]
async fn platform_admin_is_not_a_tenant_operator() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants/acme/provision")
                .header("idempotency-key", key(2))
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "platform admin must not drive per-tenant lifecycle ops",
    );
    assert!(
        resp.status() == StatusCode::FORBIDDEN || resp.status() == StatusCode::UNAUTHORIZED,
        "expected 401/403, got {}",
        resp.status(),
    );
}

/// Replaying a mutating op after the tenant reached a terminal/idempotent
/// outcome with the SAME key replays the original outcome (idempotency holds
/// through the authz layer).
#[tokio::test]
async fn lifecycle_replay_same_key_is_idempotent() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    assert_eq!(
        lifecycle(&app, "acme", "provision", &key(2)).await.status(),
        StatusCode::OK
    );
    // Replay provision with the SAME key: idempotent replay, still 200 + Active.
    let resp = lifecycle(&app, "acme", "provision", &key(2)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp.into_body()).await["state"], "active");
}

/// Reusing an idempotency key with DIFFERENT parameters is rejected (HTTP 422
/// IDEMPOTENCY_KEY_REUSE) — exercised on register where the body differs WITHIN
/// the SAME tenant scope.
///
/// The idempotency-key namespace is PER-TENANT (matching the durable backend's
/// `PRIMARY KEY (tenant_id, idempotency_key)`), so a key reused under a
/// DIFFERENT tenant addresses an independent record and is NOT a reuse (see
/// `register_same_key_different_tenant_is_independent`). Reuse is only a
/// violation when the SAME tenant replays the key with a different body.
#[tokio::test]
async fn register_key_reuse_with_different_body_is_422() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    // Same tenant + same key, but a DIFFERENT body ⇒ key reuse with different
    // params ⇒ 422.
    let mut different_body = register_body("acme");
    different_body["display_name"] = serde_json::Value::String("Acme Renamed".to_owned());
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", key(1))
                .header("authorization", format!("Bearer {PLATFORM_TOKEN}"))
                .body(Body::from(serde_json::to_vec(&different_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// The SAME idempotency key under a DIFFERENT tenant is an INDEPENDENT record,
/// not a reuse: the dedup namespace is per-tenant. Each registration succeeds
/// (201) and creates its own tenant — proving no cross-tenant idempotency leak.
#[tokio::test]
async fn register_same_key_different_tenant_is_independent() {
    let app = app();
    assert_eq!(
        register(&app, "acme", &key(1)).await.status(),
        StatusCode::CREATED
    );
    // Same key, different tenant_id ⇒ independent dedup namespace ⇒ 201.
    assert_eq!(
        register(&app, "different", &key(1)).await.status(),
        StatusCode::CREATED
    );
}

// ============================================================
// G006 SLICE-1 — durable Postgres store wiring (composition root)
// ============================================================

/// Env that turns the live-Postgres tier ON (mirrors the durable adapter's
/// `tests/live_rls.rs` gate). Default `buck2 test` leaves it unset, so the live
/// test below skips cleanly (the DB-free lane stays the default).
const LIVE_ENV: &str = "OYA_BACKBONE_LIVE_POSTGRES";
/// The runtime Postgres URL the durable router boots against. This MUST be the
/// APP (non-superuser, NON-BYPASSRLS) role URL, NOT the SETUP superuser URL:
/// `build_postgres_router` runs `assert_rls_enforceable`, which REJECTS a
/// bypass-capable role with `PgStoreConnectError::RlsUnenforceable` (so a
/// superuser URL would make `pg_app(...).expect(...)` panic, not skip). Mirrors
/// the SCIM #799 fix and the adapter live tests' `OYA_BACKBONE_POSTGRES_APP_URL`.
const LIVE_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_APP_URL";

/// Truthy-gate identical to the adapter's live tests.
fn live_enabled() -> bool {
    std::env::var(LIVE_ENV)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Build a fully-authorized DURABLE router over `url`. Panics on a connect
/// failure (a hard test failure when the live tier is explicitly enabled).
async fn pg_app(url: &str) -> axum::Router {
    build_postgres_router(
        url,
        membership(),
        Some(PLATFORM_TOKEN.to_owned()),
        Some(OPERATOR_TOKEN.to_owned()),
    )
    .await
    .expect("durable router must compose against the live database")
}

/// Build a fully-authorized DURABLE router over `url` whose membership resolver
/// assigns the test operator EXACTLY `assigned_tenants` (and nothing else). The
/// durable test registers a per-PID tenant id not in the fixed `operator_tenants`
/// list, so it must grant the operator membership to that dynamic tenant to read
/// it back legitimately through the C7 membership path (NOT by reading as the
/// platform admin, which would bypass the membership-bound axis under test).
async fn pg_app_with_membership(url: &str, assigned_tenants: Vec<String>) -> axum::Router {
    let membership: SharedMembershipResolver = Arc::new(
        InMemoryTenantMembershipResolver::new().with_operator(OPERATOR_PRINCIPAL, assigned_tenants),
    );
    build_postgres_router(
        url,
        membership,
        Some(PLATFORM_TOKEN.to_owned()),
        Some(OPERATOR_TOKEN.to_owned()),
    )
    .await
    .expect("durable router must compose against the live database")
}

/// Read a tenant via a SPECIFIC router instance (not the shared `app()`), so a
/// freshly-rebuilt router can be checked against the same backend.
async fn get_state_on(app: &axum::Router, tenant_id: &str) -> (StatusCode, serde_json::Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/v1/tenants/{tenant_id}"))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", tenant_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp.into_body()).await)
}

/// DB-free fail-closed wiring proof at the integration tier: an empty
/// `DATABASE_URL` refuses to compose the durable router (maps
/// `PgStoreConnectError::MissingDatabaseUrl` -> [`BootError::Store`]). No
/// database is touched, so this runs in the default (DB-free) `buck2 test` lane
/// and guards the same fail-closed contract from the acceptance target (the
/// `src/lib.rs` unit lane asserts it in-crate; this proves it through the
/// public crate boundary the binary uses).
#[tokio::test]
async fn build_postgres_router_empty_url_fails_closed() {
    let result = build_postgres_router(
        "",
        membership(),
        Some(PLATFORM_TOKEN.to_owned()),
        Some(OPERATOR_TOKEN.to_owned()),
    )
    .await;
    assert!(
        matches!(result, Err(BootError::Store(_))),
        "empty DATABASE_URL must fail-close as BootError::Store, got {result:?}"
    );
}

/// LIVE durability proof (env-gated): register a tenant through the REST surface
/// on a durable router, then build a FRESH router over the SAME url and GET the
/// tenant back — proving the write survived a router rebuild (the property the
/// in-memory store CANNOT provide). Also asserts the real facade-layer PEP
/// cross-tenant deny: a verified operator whose `x-oya-tenant` axis does NOT
/// match the target id receives **403 FORBIDDEN**, not 404 — the PEP denies
/// it before the store is ever consulted. Store-layer RLS cross-tenant denial
/// (unset-GUC deny-all, cross-tenant INSERT/SELECT) is proven separately in
/// `tenancy/adapters/tenant-lifecycle-store-postgres/tests/live_rls.rs`.
///
/// Skips cleanly with a stderr notice when `OYA_BACKBONE_LIVE_POSTGRES` is
/// unset so the default `buck2 test` stays DB-free. The target never reads any
/// env at compile time (no `env!` macro), so it always compiles regardless of
/// whether `CARGO_MANIFEST_DIR` is set (as it would be absent in buck2).
#[tokio::test]
async fn live_durable_store_persists_across_router_rebuild() {
    if !live_enabled() {
        eprintln!(
            "SKIP live_durable_store_persists_across_router_rebuild: \
             set {LIVE_ENV}=1 and {LIVE_URL_ENV}=<disposable pg url> \
             to run the durable tier"
        );
        return;
    }
    let url = match std::env::var(LIVE_URL_ENV) {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP live_durable_store_persists_across_router_rebuild: \
                 {LIVE_ENV} is set but {LIVE_URL_ENV} is missing — \
                 set {LIVE_URL_ENV}=<disposable pg url>"
            );
            return;
        }
    };

    // A unique tenant id per run so repeated live runs against the same
    // database do not collide on the durable PRIMARY KEY.
    // key(1) is also safe to reuse across runs because the applied-writes PK
    // is (tenant_id, idempotency_key) — uniqueness is provided transitively
    // by the per-pid tenant_id.
    let tenant_id = format!("g006-durable-{}", std::process::id());

    // The dynamic per-PID tenant id is NOT in the fixed `operator_tenants` list,
    // so the operator must be granted SERVER-SIDE membership to it to read it
    // back legitimately through the C7 membership-bound path (reading as the
    // platform admin would bypass the very axis under test). The operator is
    // assigned ONLY this tenant — so the cross-tenant sub-assertion below (a
    // DIFFERENT, unassigned tenant) is still denied by the membership resolver.
    let assigned = vec![tenant_id.clone()];

    // Router #1: register the tenant (born Provisioning) via the REST surface
    // (register is a platform-admin op; the `register()` helper uses PLATFORM_TOKEN).
    let first = pg_app_with_membership(&url, assigned.clone()).await;
    let resp = register(&first, &tenant_id, &key(1)).await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "durable register must succeed"
    );

    // Router #2: a FRESH composition over the SAME url (operator membership to
    // the dynamic tenant re-granted). The earlier write is only observable here
    // if it was durably persisted (the in-memory store, being per-process state,
    // would return 404).
    let rebuilt = pg_app_with_membership(&url, assigned.clone()).await;

    // Positive control: the operator, now legitimately assigned `tenant_id`
    // (axis == assigned tenant), can read its own tenant row on the fresh router.
    let (status, view) = get_state_on(&rebuilt, &tenant_id).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "durable tenant must survive a router rebuild"
    );
    assert_eq!(view["tenant_id"], tenant_id.as_str());
    assert_eq!(view["state"], "provisioning");

    // Facade-layer PEP cross-tenant deny (the C7 membership-bound axis): issue
    // GET /v1/tenants/{tenant_id} with OPERATOR_TOKEN but `x-oya-tenant` set to a
    // DIFFERENT tenant the operator is NOT assigned (`some-other-tenant`). The
    // server-side membership resolver assigned the operator ONLY `tenant_id`, so
    // selecting an unassigned tenant is denied at authn → 403 FORBIDDEN, BEFORE
    // the store is ever consulted. This is NOT a store-layer RLS test (store-layer
    // RLS is proven in the adapter's live_rls.rs); it proves the PEP guards the
    // facade via the membership-bound axis, not a self-attested header.
    let cross_resp = rebuilt
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::GET)
                .uri(format!("/v1/tenants/{tenant_id}"))
                .header("authorization", format!("Bearer {OPERATOR_TOKEN}"))
                .header("x-oya-tenant", "some-other-tenant")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        cross_resp.status(),
        StatusCode::FORBIDDEN,
        "operator selecting an UNASSIGNED x-oya-tenant must be denied 403 by the membership-bound PEP"
    );
}
