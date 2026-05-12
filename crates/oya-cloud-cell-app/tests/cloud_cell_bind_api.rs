use oya_cloud_cell_app::{
    bind_cloud_cell_from_api, CloudCellApiAuthorization, CloudCellApiBoundaryContext,
    CloudCellApiError, CloudCellApiPrincipal, CloudCellBindApiRequest, CloudCellBindApiStatus,
    CloudCellBindIdempotencyLedger, CloudCellBindRequest, CLOUD_CELL_BIND_SURFACE,
};
use oya_cloud_region_kernel::{
    AzState, CellCapacity, CellUtilization, CloudAzCreate, CloudCellCreate, CloudCellState,
    CloudRegionCatalog, CloudRegionCreate, RegionState, TenantDensityClass,
};
use oya_platform_cell_kernel::CellRouter;
use oya_platform_residency_kernel::ResidencyClass;

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_CELL_BIND_SURFACE, "cloud.cell.bind");
    assert_eq!(CloudCellBindApiStatus::Created.code(), 201);
    assert_eq!(CloudCellBindApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudCellBindApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudCellBindApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudCellBindApiStatus::NotFound.code(), 404);
    assert_eq!(CloudCellBindApiStatus::Conflict.code(), 409);
    assert_eq!(CloudCellBindApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn cell_bind_api_binds_once_and_replays_same_idempotent_result() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();
    let request = bind_request("req_cell_bind_ok", "idem_cell_bind_ok");

    let first = bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, request.clone())
        .expect("first request binds tenant to cell");
    let second = bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.az, "kr-seoul-a");
    assert_eq!(first.data.cell_id, "cell-kr-seoul-a-001");
    assert_eq!(first.data.residency_class, "strict_kr");
    assert_eq!(first.data.tier, "shared");
    assert_eq!(
        first.data.hsm_partition_ref,
        "hsm/kr-seoul/cell-kr-seoul-a-001"
    );
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.request_id, "req_cell_bind_ok");
    assert_eq!(first.metadata.tenant_id, "ten_kr");
    assert_eq!(first.metadata.region, "kr-seoul");
    assert_eq!(idempotency.len(), 1);
    assert!(router.get("ten_kr").is_some());
}

#[test]
fn cell_bind_api_rejects_path_body_and_header_tenant_drift_before_router_mutation() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();
    let mut request = bind_request("req_cell_bind_drift", "idem_cell_bind_drift");
    request.path_tenant_id = "ten_other".to_string();

    let error = bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, request)
        .expect_err("path/body tenant drift is rejected");

    assert!(matches!(error, CloudCellApiError::TenantMismatch { .. }));
    assert_eq!(error.cell_bind_status(), CloudCellBindApiStatus::Forbidden);
    assert!(idempotency.is_empty());
    assert!(router.get("ten_kr").is_none());
}

#[test]
fn cell_bind_api_separates_missing_authentication_from_denied_authorization() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();
    let mut unauthenticated = bind_request("req_cell_bind_unauthn", "idem_cell_bind_unauthn");
    unauthenticated.principal.principal_id.clear();

    let authn_error =
        bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, unauthenticated)
            .expect_err("missing principal is authentication failure");
    assert_eq!(
        authn_error.cell_bind_status(),
        CloudCellBindApiStatus::Unauthorized
    );

    let mut denied = bind_request("req_cell_bind_denied", "idem_cell_bind_denied");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.event.ingest".to_string()];
    let authz_error = bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, denied)
        .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        CloudCellApiError::AuthorizationDenied { ref surface } if surface == CLOUD_CELL_BIND_SURFACE
    ));
    assert_eq!(
        authz_error.cell_bind_status(),
        CloudCellBindApiStatus::Forbidden
    );
    assert!(idempotency.is_empty());
    assert!(router.get("ten_kr").is_none());
}

#[test]
fn cell_bind_api_rejects_invalid_residency_and_density_labels_before_kernel() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();
    let mut invalid_residency = bind_request(
        "req_cell_bind_bad_residency",
        "idem_cell_bind_bad_residency",
    );
    invalid_residency.body.residency_class = "per_pack".to_string();

    assert!(matches!(
        bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, invalid_residency),
        Err(CloudCellApiError::InvalidResidencyClassLabel { .. })
    ));

    let mut invalid_density =
        bind_request("req_cell_bind_bad_density", "idem_cell_bind_bad_density");
    invalid_density.body.required_density = Some("pooled".to_string());
    assert!(matches!(
        bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, invalid_density),
        Err(CloudCellApiError::InvalidTenantDensityLabel { .. })
    ));

    assert!(idempotency.is_empty());
    assert!(router.get("ten_kr").is_none());
}

#[test]
fn cell_bind_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();
    let mut request = bind_request("req_cell_bind_reused", "idem_cell_bind_reused");

    bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, request.clone())
        .expect("first request records idempotency result");

    request.body.required_density = Some("dedicated".to_string());
    let error = bind_cloud_cell_from_api(&catalog, &mut router, &mut idempotency, request)
        .expect_err("same idempotency key with changed body is rejected");

    assert_eq!(
        error,
        CloudCellApiError::IdempotencyKeyReused {
            idempotency_key: "idem_cell_bind_reused".to_string()
        }
    );
    assert_eq!(
        error.cell_bind_status(),
        CloudCellBindApiStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
}

#[test]
fn cell_bind_api_maps_kernel_duplicate_binding_and_no_compatible_cell_errors() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();
    let mut idempotency = CloudCellBindIdempotencyLedger::default();

    bind_cloud_cell_from_api(
        &catalog,
        &mut router,
        &mut idempotency,
        bind_request("req_cell_bind_first", "idem_cell_bind_first"),
    )
    .expect("first binding succeeds");

    let duplicate = bind_cloud_cell_from_api(
        &catalog,
        &mut router,
        &mut idempotency,
        bind_request("req_cell_bind_duplicate", "idem_cell_bind_duplicate"),
    )
    .expect_err("same tenant through new idempotency key conflicts");
    assert!(matches!(duplicate, CloudCellApiError::Region(_)));
    assert_eq!(
        duplicate.cell_bind_status(),
        CloudCellBindApiStatus::Conflict
    );

    let mut empty_router = CellRouter::default();
    let mut empty_idempotency = CloudCellBindIdempotencyLedger::default();
    let no_cell = bind_cloud_cell_from_api(
        &catalog_without_cells(),
        &mut empty_router,
        &mut empty_idempotency,
        bind_request("req_cell_bind_no_cell", "idem_cell_bind_no_cell"),
    )
    .expect_err("catalog without active compatible cells cannot route");
    assert!(matches!(no_cell, CloudCellApiError::Region(_)));
    assert_eq!(
        no_cell.cell_bind_status(),
        CloudCellBindApiStatus::UnprocessableEntity
    );
}

fn bind_request(request_id: &str, idempotency_key: &str) -> CloudCellBindApiRequest {
    CloudCellBindApiRequest {
        path_tenant_id: "ten_kr".to_string(),
        boundary: CloudCellApiBoundaryContext {
            request_id: request_id.to_string(),
            tenant_id: "ten_kr".to_string(),
            idempotency_key: idempotency_key.to_string(),
        },
        principal: CloudCellApiPrincipal {
            tenant_id: "ten_kr".to_string(),
            principal_id: "sp_cloud_cell_admin".to_string(),
        },
        authorization: CloudCellApiAuthorization {
            tenant_id: "ten_kr".to_string(),
            principal_id: "sp_cloud_cell_admin".to_string(),
            decision_id: format!("authz_{request_id}"),
            allowed_surfaces: vec![CLOUD_CELL_BIND_SURFACE.to_string()],
        },
        body: CloudCellBindRequest {
            tenant_id: "ten_kr".to_string(),
            home_region_code: "kr-seoul".to_string(),
            residency_class: "strict_kr".to_string(),
            required_density: Some("shared".to_string()),
        },
    }
}

fn catalog_with_cell() -> CloudRegionCatalog {
    let mut catalog = catalog_without_cells();
    catalog
        .register_cell(CloudCellCreate {
            id: "cell-kr-seoul-a-001".to_string(),
            region_code: "kr-seoul".to_string(),
            az_code: "kr-seoul-a".to_string(),
            state: CloudCellState::Active,
            tenant_density: TenantDensityClass::Shared,
            allowed_residency: vec![ResidencyClass::StrictKr],
            capacity: CellCapacity {
                compute_vcpu: 128,
                memory_gb: 512,
                ssd_tb: 40,
                gpu_count: 0,
            },
            utilization: CellUtilization {
                compute_vcpu_used: 32,
                memory_gb_used: 128,
                ssd_tb_used: 10,
                gpu_count_used: 0,
            },
            hsm_partition_ref: "hsm/kr-seoul/cell-kr-seoul-a-001".to_string(),
            created_at_epoch_seconds: 1_700_000_020,
        })
        .expect("cell fixture registers");
    catalog
}

fn catalog_without_cells() -> CloudRegionCatalog {
    let mut catalog = CloudRegionCatalog::default();
    catalog
        .register_region(CloudRegionCreate {
            code: "kr-seoul".to_string(),
            display_name: "Korea Seoul".to_string(),
            regulatory_packs: vec!["oya-pack-kr".to_string()],
            state: RegionState::Preview,
            provider_facing: true,
            residency_strictness: ResidencyClass::StrictKr,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("region fixture registers");
    catalog
        .register_az(CloudAzCreate {
            code: "kr-seoul-a".to_string(),
            region_code: "kr-seoul".to_string(),
            physical_ref: "dc/kr-seoul/a".to_string(),
            power_zones: vec!["pz-a1".to_string(), "pz-a2".to_string()],
            state: AzState::Active,
            created_at_epoch_seconds: 1_700_000_010,
        })
        .expect("AZ fixture registers");
    catalog
}
