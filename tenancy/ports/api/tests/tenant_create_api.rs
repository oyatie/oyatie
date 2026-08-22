// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Tenant-create API boundary tests.
//!
//! AUTH-005 / SECURITY remediation: authorization is decided SERVER-SIDE by an
//! injected [`TenantCreateAuthorizer`] PDP against the VERIFIED caller and the
//! TARGET (path) tenant, never from a caller-supplied authorization grant. The
//! verified principal is UNFORGEABLE — minted ONLY by a credential verifier, not
//! deserialized from the request. These tests prove the fail-closed seam:
//!   - forged/absent credential ⇒ no verified principal ⇒ 401 (cannot even build
//!     a request);
//!   - verified caller, PDP-deny ⇒ 403;
//!   - verified caller, cross-tenant deny under an authorizer that WOULD allow
//!     same-tenant ⇒ 403 (blast-radius binding to the TARGET tenant);
//!   - PDP-fault ⇒ 403 (fail-closed);
//!   - happy path (verified + allow) ⇒ created.

use tenancy_api::{
    AuthzFault, BearerTenantPrincipalVerifier, TENANT_CREATE_OPENAPI_CONTRACT,
    TENANT_CREATE_SURFACE, TenantApiBoundaryContext, TenantCreateApiError, TenantCreateApiRequest,
    TenantCreateApiStatus, TenantCreateAuthorizer, TenantCreateAuthzRequest,
    TenantCreateIdempotencyLedger, TenantCreateRequest, TenantDirectory, TenantPrincipalVerifier,
    TenantRegulatoryPackRef, VerifiedTenantPrincipal, create_tenant_from_api,
};

const REQUEST_ID: &str = "req_tenant_create_001";
const IDEMPOTENCY_KEY: &str = "idem_tenant_create_001";
const OPERATOR_TENANT_ID: &str = "ten_platform";
const TARGET_TENANT_ID: &str = "ten_alpha";
const CALLER_PRINCIPAL: &str = "usr_platform_admin";
const VALID_BEARER: &str = "test-tenant-create-bearer-secret";

// ── Test PDP authorizers (deny-by-default; explicit shapes) ─────────────────

/// An authorizer that ALWAYS allows — used to prove the *other* gates (e.g.
/// cross-tenant binding) deny independently of the PDP's permit.
struct AllowAllAuthorizer;
impl TenantCreateAuthorizer for AllowAllAuthorizer {
    fn decide(&self, _request: &TenantCreateAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Ok(true)
    }
}

/// An authorizer that permits `tenant.create` ONLY when the verified caller's
/// tenant equals the TARGET tenant (same-tenant). A cross-tenant create is
/// denied AT THE PDP — proving blast-radius binding to the trusted target axis.
struct SameTenantAuthorizer;
impl TenantCreateAuthorizer for SameTenantAuthorizer {
    fn decide(&self, request: &TenantCreateAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Ok(request.surface == TENANT_CREATE_SURFACE
            && request.caller_tenant_id == request.target_tenant_id)
    }
}

/// An authorizer that always DENIES (explicit policy deny).
struct DenyAllAuthorizer;
impl TenantCreateAuthorizer for DenyAllAuthorizer {
    fn decide(&self, _request: &TenantCreateAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Ok(false)
    }
}

/// An authorizer that always FAULTS (PDP unavailable) — must map fail-closed.
struct FaultingAuthorizer;
impl TenantCreateAuthorizer for FaultingAuthorizer {
    fn decide(&self, _request: &TenantCreateAuthzRequest<'_>) -> Result<bool, AuthzFault> {
        Err(AuthzFault::new("pdp-unavailable"))
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// The reference verifier configured with the test bearer, bound to the caller
/// tenant the target tenant equals on the happy path.
fn verifier_for(caller_tenant: &str) -> BearerTenantPrincipalVerifier {
    BearerTenantPrincipalVerifier::new(VALID_BEARER, caller_tenant, CALLER_PRINCIPAL)
}

/// Mint a verified principal by presenting the VALID bearer to the reference
/// verifier — the ONLY supported way to obtain an unforgeable principal.
fn verified(caller_tenant: &str) -> VerifiedTenantPrincipal {
    verifier_for(caller_tenant)
        .verify(Some(VALID_BEARER))
        .expect("valid bearer mints a verified principal")
}

fn request_for(
    principal: VerifiedTenantPrincipal,
    request_id: &str,
    idempotency_key: &str,
    tenant_id: &str,
) -> TenantCreateApiRequest {
    TenantCreateApiRequest {
        path_tenant_id: tenant_id.to_string(),
        boundary: TenantApiBoundaryContext {
            request_id: request_id.to_string(),
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            idempotency_key: idempotency_key.to_string(),
        },
        principal,
        body: TenantCreateRequest {
            tenant_id: tenant_id.to_string(),
            legal_name: "Alpha Tenant Ltd".to_string(),
            home_region: "region-home".to_string(),
            residency_class: "strict_home_region".to_string(),
            regulatory_packs: vec![TenantRegulatoryPackRef {
                value: "pack-alpha".to_string(),
            }],
        },
    }
}

/// The happy-path request: caller tenant == target tenant, valid bearer.
fn tenant_request(
    request_id: &str,
    idempotency_key: &str,
    tenant_id: &str,
) -> TenantCreateApiRequest {
    request_for(verified(tenant_id), request_id, idempotency_key, tenant_id)
}

// ── Contract constants ──────────────────────────────────────────────────────

#[test]
fn tenant_create_contract_runtime_constants_are_covered() {
    assert_eq!(TENANT_CREATE_SURFACE, "tenant.create");
    assert_eq!(
        TENANT_CREATE_OPENAPI_CONTRACT,
        "contracts/openapi/platform/platform-tenant-v1.yaml"
    );
    assert_eq!(TenantCreateApiStatus::Created.code(), 201);
    assert_eq!(TenantCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(TenantCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(TenantCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(TenantCreateApiStatus::Conflict.code(), 409);
    assert_eq!(TenantCreateApiStatus::UnprocessableEntity.code(), 422);
}

// ── AUTH-005 fail-closed seam (the SECURITY tests) ──────────────────────────

/// A forged/absent bearer mints NO verified principal — the request cannot even
/// be constructed without authority. This is the type-level 401: there is no
/// caller-supplied authorization field to fabricate.
#[test]
fn forged_or_absent_credential_yields_no_verified_principal() {
    let verifier = verifier_for(TARGET_TENANT_ID);
    assert!(
        verifier.verify(Some("wrong-bearer")).is_none(),
        "a forged bearer must NOT mint a verified principal",
    );
    assert!(
        verifier.verify(None).is_none(),
        "an absent bearer must NOT mint a verified principal",
    );
    // An empty configured token is an allow-NOTHING verifier (no allow-all).
    let unconfigured = BearerTenantPrincipalVerifier::new("", TARGET_TENANT_ID, CALLER_PRINCIPAL);
    assert!(
        unconfigured.verify(Some(VALID_BEARER)).is_none(),
        "an unconfigured verifier must authenticate no one",
    );
}

/// HAPPY PATH: verified caller + PDP allow ⇒ created. Removing the PDP gate (an
/// allow-all) still passes here, so the GREEN test alone proves nothing — it is
/// the DENY tests below that prove the gate is load-bearing.
#[test]
fn verified_caller_allowed_by_pdp_creates_tenant() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request(REQUEST_ID, IDEMPOTENCY_KEY, TARGET_TENANT_ID);

    let created = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &SameTenantAuthorizer,
        request,
    )
    .expect("verified same-tenant caller is allowed");

    assert_eq!(created.data.tenant_id, TARGET_TENANT_ID);
    // Audit attribution reflects the VERIFIED caller, never a caller-supplied field.
    assert_eq!(created.metadata.principal_id, CALLER_PRINCIPAL);
    assert_eq!(created.metadata.operator_tenant_id, TARGET_TENANT_ID);
    assert_eq!(directory.len(), 1);
}

/// CROSS-TENANT DENY (blast-radius binding): the SAME verified caller (scoped to
/// `ten_attacker`) tries to create the VICTIM tenant `ten_alpha`. The PDP would
/// allow a *same-tenant* create, but the target axis is the path tenant, so the
/// PDP denies ⇒ 403. This fails if the boundary flattened the target to the
/// caller's own tenant (the IDOR the remediation forbids).
#[test]
fn verified_cross_tenant_create_is_denied_at_pdp() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    // Caller proven-scoped to a DIFFERENT tenant than the target it requests.
    let attacker = verified("ten_attacker");
    let request = request_for(attacker, "req_x", "idem_x", TARGET_TENANT_ID);

    let error = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &SameTenantAuthorizer,
        request,
    )
    .expect_err("cross-tenant create must be denied at the PDP");

    assert!(matches!(
        error,
        TenantCreateApiError::AuthorizationDenied { ref surface }
            if surface == TENANT_CREATE_SURFACE
    ));
    assert_eq!(
        error.tenant_create_status(),
        TenantCreateApiStatus::Forbidden
    );
    assert!(
        directory.is_empty(),
        "no mutation on a denied cross-tenant create"
    );
    assert!(idempotency.is_empty());
}

/// PDP explicit deny (`Ok(false)`) ⇒ 403, no mutation. Even with a perfectly
/// valid verified caller and matching tenants, a policy deny blocks the create.
#[test]
fn verified_caller_denied_by_pdp_is_forbidden() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request("req_deny", "idem_deny", TARGET_TENANT_ID);

    let error = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &DenyAllAuthorizer,
        request,
    )
    .expect_err("PDP deny must forbid the create");

    assert!(matches!(
        error,
        TenantCreateApiError::AuthorizationDenied { .. }
    ));
    assert_eq!(
        error.tenant_create_status(),
        TenantCreateApiStatus::Forbidden
    );
    assert!(directory.is_empty());
}

/// PDP fault (`Err`) ⇒ 403 fail-closed, never an allow, never a 500.
#[test]
fn pdp_fault_is_fail_closed_forbidden() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request("req_fault", "idem_fault", TARGET_TENANT_ID);

    let error = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &FaultingAuthorizer,
        request,
    )
    .expect_err("a PDP fault must fail-close to a deny");

    assert!(matches!(
        error,
        TenantCreateApiError::AuthorizationFault { ref detail } if detail == "pdp-unavailable"
    ));
    assert_eq!(
        error.tenant_create_status(),
        TenantCreateApiStatus::Forbidden
    );
    assert!(
        directory.is_empty(),
        "no mutation on a fail-closed PDP fault"
    );
}

// ── Functional behaviour (under an allow-all PDP) ───────────────────────────

#[test]
fn tenant_create_creates_once_and_replays_same_idempotent_result() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request(REQUEST_ID, IDEMPOTENCY_KEY, TARGET_TENANT_ID);

    let first = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        request.clone(),
    )
    .expect("first tenant creation succeeds");
    let second = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        request,
    )
    .expect("same tenant creation request replays");

    assert_eq!(first, second);
    assert_eq!(directory.len(), 1);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.data.tenant_id, TARGET_TENANT_ID);
    assert_eq!(first.data.legal_name, "Alpha Tenant Ltd");
    assert_eq!(first.data.home_region, "region-home");
    assert_eq!(first.data.residency_class, "strict_home_region");
    assert_eq!(first.data.regulatory_packs[0].value, "pack-alpha");
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.request_id, REQUEST_ID);
    assert!(directory.get(TARGET_TENANT_ID).is_some());
}

#[test]
fn tenant_create_rejects_path_body_drift_before_directory_mutation() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut request = tenant_request("req_tenant_drift", "idem_tenant_drift", TARGET_TENANT_ID);
    request.body.tenant_id = "ten_other".to_string();

    let error = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        request,
    )
    .expect_err("path/body tenant drift is rejected");

    assert!(matches!(
        error,
        TenantCreateApiError::TenantPathBodyMismatch { .. }
    ));
    assert_eq!(error.tenant_create_status_code(), 400);
    assert!(directory.is_empty());
    assert!(idempotency.is_empty());
}

#[test]
fn tenant_create_maps_duplicate_invalid_residency_and_kernel_errors() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        tenant_request("req_tenant_first", "idem_tenant_first", TARGET_TENANT_ID),
    )
    .expect("initial tenant creation succeeds");

    let duplicate = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        tenant_request(
            "req_tenant_duplicate",
            "idem_tenant_duplicate",
            TARGET_TENANT_ID,
        ),
    )
    .expect_err("duplicate tenant id conflicts");
    assert!(matches!(
        duplicate,
        TenantCreateApiError::DuplicateTenant { .. }
    ));
    assert_eq!(
        duplicate.tenant_create_status(),
        TenantCreateApiStatus::Conflict
    );

    let mut invalid_residency = tenant_request(
        "req_tenant_bad_residency",
        "idem_tenant_bad_residency",
        "ten_bad_residency",
    );
    invalid_residency.body.residency_class = "moon_base".to_string();
    assert!(matches!(
        create_tenant_from_api(
            &mut directory,
            &mut idempotency,
            &AllowAllAuthorizer,
            invalid_residency
        ),
        Err(TenantCreateApiError::InvalidResidencyClass { .. })
    ));

    let mut bad_home_region = tenant_request(
        "req_tenant_bad_region",
        "idem_tenant_bad_region",
        "ten_bad_region",
    );
    bad_home_region.body.home_region = "region-recovery".to_string();
    assert!(matches!(
        create_tenant_from_api(
            &mut directory,
            &mut idempotency,
            &AllowAllAuthorizer,
            bad_home_region
        ),
        Err(TenantCreateApiError::Tenant(_))
    ));
    assert_eq!(directory.len(), 1);
}

#[test]
fn tenant_create_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut request = tenant_request("req_tenant_reused", "idem_tenant_reused", "ten_reused");

    create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        request.clone(),
    )
    .expect("first idempotent tenant creation succeeds");

    request.body.legal_name = "Changed Tenant Ltd".to_string();
    let error = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        &AllowAllAuthorizer,
        request,
    )
    .expect_err("same idempotency key with changed body is rejected");

    assert_eq!(
        error,
        TenantCreateApiError::IdempotencyKeyReused {
            idempotency_key: "idem_tenant_reused".to_string()
        }
    );
    assert_eq!(
        error.tenant_create_status(),
        TenantCreateApiStatus::UnprocessableEntity
    );
    assert_eq!(directory.len(), 1);
    assert_eq!(idempotency.len(), 1);
}
