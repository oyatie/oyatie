use cell_routing::{CellRouter, CellTier};
use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    ResidencyClass,
};

use crate::*;

fn residency_class() -> ResidencyClass {
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-alpha1".to_string()],
            allowed_replica_regions: vec!["region-beta1".to_string()],
            forbidden_regions: vec!["region-gamma1".to_string()],
            regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                regulator_refs: vec!["regulator/cloud-region".to_string()],
                evidence_ref: "evidence/residency/cloud-region".to_string(),
            })
            .expect("regulator overlay fixture is valid"),
        })
        .expect("per-pack residency fixture is valid"),
    ))
}

fn region_create() -> CloudRegionCreate {
    CloudRegionCreate {
        code: "region-alpha1".to_string(),
        display_name: "Alpha Region".to_string(),
        regulatory_packs: vec!["pack-alpha".to_string()],
        state: RegionState::Preview,
        provider_facing: true,
        residency_strictness: residency_class(),
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn az_create() -> CloudAzCreate {
    CloudAzCreate {
        code: "region-alpha1-a".to_string(),
        region_code: "region-alpha1".to_string(),
        physical_ref: "dc/region-alpha1/a".to_string(),
        power_zones: vec!["pz-a1".to_string(), "pz-a2".to_string()],
        state: AzState::Active,
        created_at_epoch_seconds: 1_700_000_010,
    }
}

fn cell_create() -> CloudCellCreate {
    CloudCellCreate {
        id: "cell-region-alpha1-a-001".to_string(),
        region_code: "region-alpha1".to_string(),
        az_code: "region-alpha1-a".to_string(),
        state: CloudCellState::Active,
        tenant_density: TenantDensityClass::Shared,
        allowed_residency: vec![residency_class()],
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
        hsm_partition_ref: "hsm/region-alpha1/cell-region-alpha1-a-001".to_string(),
        created_at_epoch_seconds: 1_700_000_020,
    }
}

fn catalog_with_cell() -> CloudRegionCatalog {
    let mut catalog = CloudRegionCatalog::default();
    catalog
        .register_region(region_create())
        .expect("region fixture registers");
    catalog
        .register_az(az_create())
        .expect("AZ fixture registers");
    catalog
        .register_cell(cell_create())
        .expect("cell fixture registers");
    catalog
}

#[test]
fn maps_port_owned_location_errors_into_the_catalog_contract() {
    let location_error = RegionCode::new("region_alpha1").expect_err("invalid region denied");
    assert_eq!(
        CloudRegionError::from(location_error),
        CloudRegionError::InvalidRegionCode
    );
}

#[test]
fn registers_region_az_cell_and_produces_platform_cell_binding() {
    let catalog = catalog_with_cell();
    let mut router = CellRouter::default();

    let binding = catalog
        .bind_route_for_tenant(
            &mut router,
            TenantCellRouteRequest {
                tenant_id: "ten_alpha".to_string(),
                home_region_code: "region-alpha1".to_string(),
                residency_class: residency_class(),
                required_density: Some(TenantDensityClass::Shared),
            },
        )
        .expect("route should satisfy platform cell binding invariants");

    assert_eq!(binding.tenant_id, "ten_alpha");
    assert_eq!(binding.region, "region-alpha1");
    assert_eq!(binding.az.value, "region-alpha1-a");
    assert_eq!(binding.cell_id.value, "cell-region-alpha1-a-001");
    assert_eq!(binding.tier.value, CellTier::Shared);
    assert_eq!(binding.residency_class.value, residency_class());
}

#[test]
fn rejects_duplicate_region_az_and_cell_ids() {
    let mut catalog = CloudRegionCatalog::default();
    catalog
        .register_region(region_create())
        .expect("first region registers");
    assert_eq!(
        catalog
            .register_region(region_create())
            .expect_err("duplicate region denied"),
        CloudRegionError::DuplicateRegion
    );

    catalog
        .register_az(az_create())
        .expect("first AZ registers");
    assert_eq!(
        catalog
            .register_az(az_create())
            .expect_err("duplicate AZ denied"),
        CloudRegionError::DuplicateAz
    );

    catalog
        .register_cell(cell_create())
        .expect("first cell registers");
    assert_eq!(
        catalog
            .register_cell(cell_create())
            .expect_err("duplicate cell denied"),
        CloudRegionError::DuplicateCell
    );
}

#[test]
fn rejects_az_outside_region() {
    let error = CloudAz::new(CloudAzCreate {
        code: "region-gamma1-a".to_string(),
        ..az_create()
    })
    .expect_err("AZ code must sit under its region code");

    assert_eq!(error, CloudRegionError::AzRegionMismatch);
}

#[test]
fn rejects_cell_id_that_does_not_match_az_namespace() {
    let error = CloudCell::new(CloudCellCreate {
        id: "cell-region-alpha1-b-001".to_string(),
        ..cell_create()
    })
    .expect_err("cell ids must be namespaced under their AZ");

    assert_eq!(error, CloudRegionError::CellAzMismatch);
}

#[test]
fn rejects_cell_without_registered_region_or_az() {
    let mut catalog = CloudRegionCatalog::default();
    catalog
        .register_region(region_create())
        .expect("region registers");

    assert_eq!(
        catalog
            .register_cell(cell_create())
            .expect_err("cell must reference a registered AZ"),
        CloudRegionError::UnknownAz
    );
}

#[test]
fn rejects_cell_when_residency_is_not_allowed_in_region() {
    let error = CloudCell::new(CloudCellCreate {
        allowed_residency: vec![residency_class()],
        region_code: "region-gamma1".to_string(),
        az_code: "region-gamma1-a".to_string(),
        id: "cell-region-gamma1-a-001".to_string(),
        hsm_partition_ref: "hsm/region-gamma1/cell-region-gamma1-a-001".to_string(),
        ..cell_create()
    })
    .expect_err("pack residency cannot be advertised in a forbidden region");

    assert_eq!(error, CloudRegionError::CellResidencyNotAllowedInRegion);
}

#[test]
fn route_denies_residency_class_not_allowed_by_cell() {
    let catalog = catalog_with_cell();

    let error = catalog
        .route_for_tenant(TenantCellRouteRequest {
            tenant_id: "ten_global".to_string(),
            home_region_code: "region-alpha1".to_string(),
            residency_class: ResidencyClass::Global,
            required_density: None,
        })
        .expect_err("cell does not advertise pack residency");

    assert_eq!(error, CloudRegionError::NoCompatibleCell);
}

#[test]
fn rejects_cell_capacity_that_is_over_allocated() {
    let error = CloudCell::new(CloudCellCreate {
        utilization: CellUtilization {
            compute_vcpu_used: 129,
            memory_gb_used: 128,
            ssd_tb_used: 10,
            gpu_count_used: 0,
        },
        ..cell_create()
    })
    .expect_err("utilization cannot exceed declared capacity");

    assert_eq!(error, CloudRegionError::UtilizationExceedsCapacity);
}

#[test]
fn direct_cell_binding_revalidates_residency_against_cell_manifest() {
    let catalog = catalog_with_cell();
    let cell_id = CellId::new("cell-region-alpha1-a-001").expect("fixture cell id is valid");

    let error = catalog
        .binding_for_cell("ten_global".to_string(), ResidencyClass::Global, &cell_id)
        .expect_err("binding must not ignore the cell residency manifest");

    assert_eq!(error, CloudRegionError::CellResidencyDenied);
}
