#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cell_routing::{CellBinding, CellBindingCreate, CellError, CellRouter, CellTier};
use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegionJurisdiction, RegionRef, RegionRefCreate,
    RegulatorOverlay, RegulatorOverlayCreate, ResidencyClass,
};

fn region(region_id: &str) -> RegionRef {
    RegionRef::new(RegionRefCreate {
        region_id: region_id.to_string(),
        jurisdiction: RegionJurisdiction::Other,
        cell_group_ref: format!("cells/{region_id}"),
    })
    .expect("region fixture is valid")
}

fn cell_binding_create() -> CellBindingCreate {
    CellBindingCreate {
        tenant_id: "ten_alpha".to_string(),
        region: region("primary-region"),
        residency_class: ResidencyClass::Global,
        az: "primary-region-a".to_string(),
        cell_id: "cell-primary-region-a-001".to_string(),
        tier: CellTier::Pooled,
        hsm_partition_ref: "hsm/primary-region/cell-001".to_string(),
    }
}

fn residency_class_for_home(home_region: &str) -> ResidencyClass {
    let regulator_overlay = RegulatorOverlay::new(RegulatorOverlayCreate {
        regulator_refs: vec!["neutral-authority".to_string()],
        evidence_ref: "regional-pack/neutral/residency".to_string(),
    })
    .expect("overlay fixture is valid");
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec![home_region.to_string()],
            allowed_replica_regions: vec![home_region.to_string()],
            forbidden_regions: Vec::new(),
            regulator_overlay,
        })
        .expect("residency fixture is valid"),
    ))
}

#[test]
fn binding_captures_region_residency_tier_and_hsm_partition() {
    let binding = CellBinding::new(cell_binding_create()).expect("cell binding should build");

    assert_eq!(binding.region, "primary-region");
    assert_eq!(binding.residency_class.value, ResidencyClass::Global);
    assert_eq!(binding.tier.value, CellTier::Pooled);
    assert_eq!(binding.schema_version.value, 1);
}

#[test]
fn router_keeps_tenant_cell_binding_immutable() {
    let mut router = CellRouter::default();
    router
        .bind(cell_binding_create())
        .expect("first binding succeeds");

    let error = router
        .bind(CellBindingCreate {
            az: "primary-region-b".to_string(),
            cell_id: "cell-primary-region-b-001".to_string(),
            ..cell_binding_create()
        })
        .expect_err("tenant cannot be rebound to another cell");

    assert_eq!(error, CellError::AlreadyBound);
}

#[test]
fn rejects_az_outside_declared_region() {
    let error = CellBinding::new(CellBindingCreate {
        az: "alternate-region-a".to_string(),
        ..cell_binding_create()
    })
    .expect_err("AZ must belong to declared region");

    assert_eq!(error, CellError::AzRegionMismatch);
}

#[test]
fn rejects_residency_region_mismatch() {
    let error = CellBinding::new(CellBindingCreate {
        residency_class: residency_class_for_home("alternate-region"),
        ..cell_binding_create()
    })
    .expect_err("cell region must match residency class");

    assert_eq!(error, CellError::ResidencyRegionMismatch);
}

#[test]
fn requires_per_cell_hsm_partition_reference() {
    let error = CellBinding::new(CellBindingCreate {
        hsm_partition_ref: "".to_string(),
        ..cell_binding_create()
    })
    .expect_err("per-cell HSM partition is mandatory");

    assert_eq!(error, CellError::EmptyHsmPartition);
}
