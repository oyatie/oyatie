// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_tenancy_api::{
    TENANT_CREATE_OPENAPI_CONTRACT, TENANT_CREATE_SURFACE, TenantApiAuthorization,
    TenantApiBoundaryContext, TenantApiPrincipal, TenantCreateApiError, TenantCreateApiRequest,
    TenantCreateApiStatus, TenantCreateIdempotencyLedger, TenantCreateRequest, TenantDirectory,
    TenantRegulatoryPackRef, create_tenant_from_api,
};

const REQUEST_ID: &str = "req_tenant_create_001";
const IDEMPOTENCY_KEY: &str = "idem_tenant_create_001";
const OPERATOR_TENANT_ID: &str = "ten_platform";
const TARGET_TENANT_ID: &str = "ten_alpha";

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

#[test]
fn tenant_create_creates_once_and_replays_same_idempotent_result() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request(REQUEST_ID, IDEMPOTENCY_KEY, TARGET_TENANT_ID);

    let first = create_tenant_from_api(&mut directory, &mut idempotency, request.clone())
        .expect("first tenant creation succeeds");
    let second = create_tenant_from_api(&mut directory, &mut idempotency, request)
        .expect("same tenant creation request replays");

    assert_eq!(first, second);
    assert_eq!(directory.len(), 1);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.data.tenant_id, TARGET_TENANT_ID);
    assert_eq!(first.data.legal_name, "KR Tenant Ltd");
    assert_eq!(first.data.home_region, "home-region");
    assert_eq!(first.data.residency_class, "strict_home");
    assert_eq!(first.data.regulatory_packs[0].value, "oya-pack-alpha");
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

    let error = create_tenant_from_api(&mut directory, &mut idempotency, request)
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
fn tenant_create_separates_missing_principal_from_denied_authorization() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut unauthenticated =
        tenant_request("req_tenant_authn", "idem_tenant_authn", TARGET_TENANT_ID);
    unauthenticated.principal.principal_id.clear();

    let authn_error = create_tenant_from_api(&mut directory, &mut idempotency, unauthenticated)
        .expect_err("missing principal is authentication failure");
    assert_eq!(
        authn_error.tenant_create_status(),
        TenantCreateApiStatus::Unauthorized
    );

    let mut denied = tenant_request("req_tenant_authz", "idem_tenant_authz", TARGET_TENANT_ID);
    denied.authorization.allowed_surfaces = vec!["identity.token.issue".to_string()];
    let authz_error = create_tenant_from_api(&mut directory, &mut idempotency, denied)
        .expect_err("missing tenant.create grant is authorization failure");
    assert!(matches!(
        authz_error,
        TenantCreateApiError::AuthorizationDenied { ref surface }
            if surface == TENANT_CREATE_SURFACE
    ));
    assert_eq!(
        authz_error.tenant_create_status(),
        TenantCreateApiStatus::Forbidden
    );
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
        tenant_request("req_tenant_first", "idem_tenant_first", TARGET_TENANT_ID),
    )
    .expect("initial tenant creation succeeds");

    let duplicate = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
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
        create_tenant_from_api(&mut directory, &mut idempotency, invalid_residency),
        Err(TenantCreateApiError::InvalidResidencyClass { .. })
    ));

    let mut bad_home_region = tenant_request(
        "req_tenant_bad_region",
        "idem_tenant_bad_region",
        "ten_bad_region",
    );
    bad_home_region.body.home_region = "failover-region".to_string();
    assert!(matches!(
        create_tenant_from_api(&mut directory, &mut idempotency, bad_home_region),
        Err(TenantCreateApiError::Tenant(_))
    ));
    assert_eq!(directory.len(), 1);
}

#[test]
fn tenant_create_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut request = tenant_request("req_tenant_reused", "idem_tenant_reused", "ten_reused");

    create_tenant_from_api(&mut directory, &mut idempotency, request.clone())
        .expect("first idempotent tenant creation succeeds");

    request.body.legal_name = "Changed Tenant Ltd".to_string();
    let error = create_tenant_from_api(&mut directory, &mut idempotency, request)
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

fn tenant_request(
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
        principal: TenantApiPrincipal {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
        },
        authorization: TenantApiAuthorization {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
            decision_id: "authz_tenant_create".to_string(),
            allowed_surfaces: vec![TENANT_CREATE_SURFACE.to_string()],
        },
        body: TenantCreateRequest {
            tenant_id: tenant_id.to_string(),
            legal_name: "KR Tenant Ltd".to_string(),
            home_region: "home-region".to_string(),
            residency_class: "strict_home".to_string(),
            regulatory_packs: vec![TenantRegulatoryPackRef {
                value: "oya-pack-alpha".to_string(),
            }],
        },
    }
}
