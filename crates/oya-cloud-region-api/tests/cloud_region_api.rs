// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_region_api::{
    CLOUD_AZ_LIST_SURFACE, CLOUD_REGION_LIST_SURFACE, CloudAzListApiRequest,
    CloudRegionApiAuthorization, CloudRegionApiBoundaryContext, CloudRegionApiError,
    CloudRegionApiPrincipal, CloudRegionListApiRequest, CloudRegionListApiStatus,
    list_cloud_azs_from_api, list_cloud_regions_from_api,
};
use oya_cloud_region_domain::{
    AzState, CellCapacity, CellUtilization, CloudAzCreate, CloudCellCreate, CloudCellState,
    CloudRegionCatalog, CloudRegionCreate, RegionState, TenantDensityClass,
};
use oya_residency_domain::ResidencyClass;

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_REGION_LIST_SURFACE, "cloud.region.list");
    assert_eq!(CLOUD_AZ_LIST_SURFACE, "cloud.az.list");
    assert_eq!(CloudRegionListApiStatus::Ok.code(), 200);
    assert_eq!(CloudRegionListApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudRegionListApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudRegionListApiStatus::NotFound.code(), 404);
}

fn catalog() -> CloudRegionCatalog {
    let mut catalog = CloudRegionCatalog::default();
    catalog
        .register_region(CloudRegionCreate {
            code: "home-region".to_string(),
            display_name: "Home Region".to_string(),
            regulatory_packs: vec!["oya-pack-alpha".to_string()],
            state: RegionState::Preview,
            provider_facing: true,
            residency_strictness: ResidencyClass::StrictHome,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("home region fixture registers");
    catalog
        .register_az(CloudAzCreate {
            code: "home-region-a".to_string(),
            region_code: "home-region".to_string(),
            physical_ref: "dc/home-region/a".to_string(),
            power_zones: vec!["pz-a1".to_string(), "pz-a2".to_string()],
            state: AzState::Active,
            created_at_epoch_seconds: 1_700_000_010,
        })
        .expect("home AZ fixture registers");
    catalog
        .register_cell(CloudCellCreate {
            id: "cell-home-region-a-001".to_string(),
            region_code: "home-region".to_string(),
            az_code: "home-region-a".to_string(),
            state: CloudCellState::Active,
            tenant_density: TenantDensityClass::Dedicated,
            allowed_residency: vec![ResidencyClass::StrictHome],
            capacity: CellCapacity {
                compute_vcpu: 256,
                memory_gb: 1_024,
                ssd_tb: 96,
                gpu_count: 8,
            },
            utilization: CellUtilization {
                compute_vcpu_used: 64,
                memory_gb_used: 256,
                ssd_tb_used: 24,
                gpu_count_used: 1,
            },
            hsm_partition_ref: "hsm/home-region/cell-home-region-a-001".to_string(),
            created_at_epoch_seconds: 1_700_000_011,
        })
        .expect("home cell fixture registers");
    catalog
        .register_cell(CloudCellCreate {
            id: "cell-home-region-a-002".to_string(),
            region_code: "home-region".to_string(),
            az_code: "home-region-a".to_string(),
            state: CloudCellState::DrOnly,
            tenant_density: TenantDensityClass::Sovereign,
            allowed_residency: vec![ResidencyClass::StrictHome],
            capacity: CellCapacity {
                compute_vcpu: 128,
                memory_gb: 512,
                ssd_tb: 48,
                gpu_count: 0,
            },
            utilization: CellUtilization {
                compute_vcpu_used: 0,
                memory_gb_used: 0,
                ssd_tb_used: 0,
                gpu_count_used: 0,
            },
            hsm_partition_ref: "hsm/home-region/cell-home-region-a-002".to_string(),
            created_at_epoch_seconds: 1_700_000_012,
        })
        .expect("home DR cell fixture registers");
    catalog
        .register_region(CloudRegionCreate {
            code: "failover-region".to_string(),
            display_name: "Failover Region".to_string(),
            regulatory_packs: vec!["oya-pack-global".to_string()],
            state: RegionState::Ga,
            provider_facing: true,
            residency_strictness: ResidencyClass::Global,
            created_at_epoch_seconds: 1_700_000_020,
        })
        .expect("failover region fixture registers");
    catalog
        .register_az(CloudAzCreate {
            code: "failover-region-a".to_string(),
            region_code: "failover-region".to_string(),
            physical_ref: "dc/failover-region/a".to_string(),
            power_zones: vec!["pz-u1".to_string()],
            state: AzState::Planned,
            created_at_epoch_seconds: 1_700_000_030,
        })
        .expect("failover AZ fixture registers");
    catalog
        .register_region(CloudRegionCreate {
            code: "internal-ops".to_string(),
            display_name: "Internal Operations".to_string(),
            regulatory_packs: vec!["oya-pack-internal".to_string()],
            state: RegionState::Planned,
            provider_facing: false,
            residency_strictness: ResidencyClass::Global,
            created_at_epoch_seconds: 1_700_000_040,
        })
        .expect("internal region fixture registers");
    catalog
        .register_az(CloudAzCreate {
            code: "internal-ops-a".to_string(),
            region_code: "internal-ops".to_string(),
            physical_ref: "dc/internal-ops/a".to_string(),
            power_zones: vec!["pz-i1".to_string()],
            state: AzState::Planned,
            created_at_epoch_seconds: 1_700_000_050,
        })
        .expect("internal AZ fixture registers");
    catalog
}

fn boundary() -> CloudRegionApiBoundaryContext {
    CloudRegionApiBoundaryContext {
        request_id: "req-region-1".to_string(),
        tenant_id: "ten_catalog".to_string(),
    }
}

fn principal() -> CloudRegionApiPrincipal {
    CloudRegionApiPrincipal {
        tenant_id: "ten_catalog".to_string(),
        principal_id: "sp_region_reader".to_string(),
    }
}

fn authorization(surface: &str) -> CloudRegionApiAuthorization {
    CloudRegionApiAuthorization {
        tenant_id: "ten_catalog".to_string(),
        principal_id: "sp_region_reader".to_string(),
        decision_id: format!("dec_{surface}"),
        allowed_surfaces: vec![surface.to_string()],
    }
}

fn region_request() -> CloudRegionListApiRequest {
    CloudRegionListApiRequest {
        boundary: boundary(),
        principal: principal(),
        authorization: authorization(CLOUD_REGION_LIST_SURFACE),
    }
}

fn az_request(region_code: &str) -> CloudAzListApiRequest {
    CloudAzListApiRequest {
        path_region_code: region_code.to_string(),
        boundary: boundary(),
        principal: principal(),
        authorization: authorization(CLOUD_AZ_LIST_SURFACE),
    }
}

#[test]
fn region_list_rejects_missing_request_id_before_projection() {
    let mut request = region_request();
    request.boundary.request_id.clear();

    let error = list_cloud_regions_from_api(&catalog(), request)
        .expect_err("missing request id must be rejected");

    assert_eq!(error.list_status_code(), 400);
    assert!(matches!(error, CloudRegionApiError::EmptyRequestId));
}

#[test]
fn region_list_rejects_tenant_principal_drift() {
    let mut request = region_request();
    request.principal.tenant_id = "ten_other".to_string();

    let error = list_cloud_regions_from_api(&catalog(), request)
        .expect_err("tenant drift must be rejected");

    assert_eq!(error.list_status_code(), 403);
    assert!(matches!(error, CloudRegionApiError::TenantMismatch { .. }));
}

#[test]
fn region_list_rejects_empty_principal() {
    let mut request = region_request();
    request.principal.principal_id.clear();

    let error = list_cloud_regions_from_api(&catalog(), request)
        .expect_err("empty principal id must be rejected");

    assert_eq!(error.list_status_code(), 403);
    assert!(matches!(error, CloudRegionApiError::EmptyPrincipalId));
}

#[test]
fn region_list_rejects_authorization_denial() {
    let mut request = region_request();
    request.authorization.allowed_surfaces = vec![CLOUD_AZ_LIST_SURFACE.to_string()];

    let error = list_cloud_regions_from_api(&catalog(), request)
        .expect_err("wrong surface grant must be rejected");

    assert_eq!(error.list_status_code(), 403);
    assert!(matches!(
        error,
        CloudRegionApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn region_list_projects_public_region_catalog() {
    let response = list_cloud_regions_from_api(&catalog(), region_request())
        .expect("authorized region list succeeds");

    assert_eq!(response.metadata.request_id, "req-region-1");
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].code, "failover-region");
    assert_eq!(response.data[0].state, "ga");
    assert_eq!(response.data[1].code, "home-region");
    assert_eq!(response.data[1].azs[0].value, "home-region-a");
    assert_eq!(response.data[1].residency_strictness, "strict_home");
}

#[test]
fn az_list_rejects_invalid_region_code() {
    let error = list_cloud_azs_from_api(&catalog(), az_request("KR Seoul"))
        .expect_err("non-canonical region code must be rejected");

    assert_eq!(error.list_status_code(), 400);
    assert!(matches!(error, CloudRegionApiError::Region(_)));
    let response = error.error_response("req-region-1");
    assert_eq!(response.error.code, "CLOUD_REGION_INVALID_REQUEST");
}

#[test]
fn az_list_rejects_unknown_region_after_authorization() {
    let error = list_cloud_azs_from_api(&catalog(), az_request("secondary-region"))
        .expect_err("unknown region must be explicit");

    assert_eq!(error.list_status_code(), 404);
    assert!(matches!(error, CloudRegionApiError::Region(_)));
    let response = error.error_response("req-region-1");
    assert_eq!(response.error.code, "CLOUD_REGION_NOT_FOUND");
}

#[test]
fn az_list_rejects_unauthorized_unknown_region_without_existence_leak() {
    let mut request = az_request("secondary-region");
    request.authorization.allowed_surfaces = vec![CLOUD_REGION_LIST_SURFACE.to_string()];

    let error = list_cloud_azs_from_api(&catalog(), request)
        .expect_err("authorization denial must win over catalog existence checks");

    assert_eq!(error.list_status_code(), 403);
    assert!(matches!(
        error,
        CloudRegionApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn az_list_does_not_expose_non_provider_facing_region() {
    let error = list_cloud_azs_from_api(&catalog(), az_request("internal-ops"))
        .expect_err("non-provider-facing regions must not be externally enumerable");

    assert_eq!(error.list_status_code(), 404);
    let response = error.error_response("req-region-1");
    assert_eq!(response.error.code, "CLOUD_REGION_NOT_FOUND");
}

#[test]
fn az_list_projects_only_requested_region_azs() {
    let response = list_cloud_azs_from_api(&catalog(), az_request("home-region"))
        .expect("authorized AZ list succeeds");

    assert_eq!(response.metadata.request_id, "req-region-1");
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].code, "home-region-a");
    assert_eq!(response.data[0].region_code, "home-region");
    assert_eq!(response.data[0].power_zones[0].value, "pz-a1");
    assert_eq!(response.data[0].power_zones[1].value, "pz-a2");
    assert_eq!(response.data[0].state, "active");
}

#[test]
fn az_list_projects_per_cell_isolation_evidence_without_capacity_leakage() {
    let response = list_cloud_azs_from_api(&catalog(), az_request("home-region"))
        .expect("authorized AZ list succeeds");

    let az = &response.data[0];

    assert_eq!(az.cells.len(), 2);
    assert_eq!(az.cell_isolation_evidence.len(), 2);
    assert_eq!(
        az.cell_isolation_evidence[0].cell_id,
        "cell-home-region-a-001"
    );
    assert_eq!(az.cell_isolation_evidence[0].region_code, "home-region");
    assert_eq!(az.cell_isolation_evidence[0].az_code, "home-region-a");
    assert_eq!(az.cell_isolation_evidence[0].state, "active");
    assert_eq!(az.cell_isolation_evidence[0].tenant_density, "dedicated");
    assert_eq!(
        az.cell_isolation_evidence[0].allowed_residency,
        vec!["strict_home".to_string()]
    );
    assert_eq!(
        az.cell_isolation_evidence[0].evidence_ref,
        "cell-isolation://home-region/home-region-a/cell-home-region-a-001"
    );
    assert_eq!(az.cell_isolation_evidence[0].schema_version, 1);
    assert_eq!(
        az.cell_isolation_evidence[1].cell_id,
        "cell-home-region-a-002"
    );
    assert_eq!(az.cell_isolation_evidence[1].state, "dr_only");
    assert_eq!(az.cell_isolation_evidence[1].tenant_density, "sovereign");
}
