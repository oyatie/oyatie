use std::collections::BTreeMap;

use data_boundary_kernel::DataClass;
use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    ResidencyClass,
};

use crate::*;

pub(super) fn residency_class() -> ResidencyClass {
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-alpha1".to_string()],
            allowed_replica_regions: vec!["region-beta1".to_string()],
            forbidden_regions: vec!["region-gamma1".to_string()],
            regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                regulator_refs: vec!["regulator/cloud-resource".to_string()],
                evidence_ref: "evidence/residency/cloud-resource".to_string(),
            })
            .expect("regulator overlay fixture is valid"),
        })
        .expect("per-pack residency fixture is valid"),
    ))
}

pub(super) fn tags() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("cost-center".to_string(), "foundry".to_string()),
        ("env".to_string(), "preview".to_string()),
    ])
}

pub(super) fn compute_resource_create() -> ResourceCreate {
    ResourceCreate {
        id: "oyatie:cloud:region-alpha1:ten_alpha:instance:api-001".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha1".to_string(),
        az: Some("region-alpha1-a".to_string()),
        cell_id: "cell-region-alpha1-a-001".to_string(),
        kind: ResourceKind::ComputeInstance(InstanceFlavor::GeneralPurpose),
        data_class: DataClass::InternalOnly,
        owner_principal: "sp_foundry".to_string(),
        state: ResourceState::Pending,
        tags: tags(),
        iam_policy_attachments: vec!["pol_cloud_compute_admin".to_string()],
        metering_tag: "oyatie:metering:ten_alpha:instance".to_string(),
        residency: residency_class(),
        created_at_epoch_seconds: 1_700_000_000,
        updated_at_epoch_seconds: 1_700_000_000,
    }
}
