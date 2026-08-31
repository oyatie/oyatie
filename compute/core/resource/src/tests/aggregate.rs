use std::collections::BTreeMap;

use data_boundary_kernel::DataClass;

use crate::aggregate::RESOURCE_SCHEMA_VERSION;
use crate::{CloudResourceError, Resource, ResourceCreate};

use super::fixture::compute_resource_create;

#[test]
fn creates_resource_aggregate_with_location_residency_and_metering_identity() {
    let resource = Resource::new(compute_resource_create()).expect("resource should be valid");

    assert_eq!(resource.tenant_id.value, "ten_alpha");
    assert_eq!(resource.region.value.value, "region-alpha1");
    assert_eq!(
        resource.az.value.expect("compute has AZ").value,
        "region-alpha1-a"
    );
    assert_eq!(resource.cell_id.value.value, "cell-region-alpha1-a-001");
    assert_eq!(resource.kind.value.type_label(), "instance");
    assert_eq!(
        resource.metering_tag.value.value,
        "oyatie:metering:ten_alpha:instance"
    );
    assert_eq!(resource.schema_version.value, RESOURCE_SCHEMA_VERSION);
}

#[test]
fn rejects_resource_id_that_disagrees_with_tenant_region_or_kind() {
    let tenant_error = Resource::new(ResourceCreate {
        id: "oyatie:cloud:region-alpha1:ten_other:instance:api-001".to_string(),
        ..compute_resource_create()
    })
    .expect_err("resource id tenant must match resource tenant");
    assert_eq!(tenant_error, CloudResourceError::ResourceIdTenantMismatch);

    let kind_error = Resource::new(ResourceCreate {
        id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:api-001".to_string(),
        ..compute_resource_create()
    })
    .expect_err("resource id kind must match resource kind");
    assert_eq!(kind_error, CloudResourceError::ResourceIdKindMismatch);
}

#[test]
fn rejects_az_scoped_resource_without_az() {
    let error = Resource::new(ResourceCreate {
        az: None,
        cell_id: "cell-region-alpha1-001".to_string(),
        ..compute_resource_create()
    })
    .expect_err("compute instances must declare AZ placement");

    assert_eq!(error, CloudResourceError::AzRequiredForResourceKind);
}

#[test]
fn rejects_location_tuple_drift_between_region_az_and_cell() {
    let az_error = Resource::new(ResourceCreate {
        az: Some("region-gamma1-a".to_string()),
        ..compute_resource_create()
    })
    .expect_err("AZ must belong to region");
    assert_eq!(az_error, CloudResourceError::AzRegionMismatch);

    let cell_error = Resource::new(ResourceCreate {
        cell_id: "cell-region-alpha1-b-001".to_string(),
        ..compute_resource_create()
    })
    .expect_err("cell must belong to AZ namespace");
    assert_eq!(cell_error, CloudResourceError::CellLocationMismatch);
}

#[test]
fn rejects_operational_labels_as_resource_payload_data_class() {
    let error = Resource::new(ResourceCreate {
        data_class: DataClass::Audit,
        ..compute_resource_create()
    })
    .expect_err("resource payload class must be a privacy-program class");

    assert_eq!(error, CloudResourceError::InvalidDataClass);
}

#[test]
fn rejects_reserved_or_empty_tenant_tags_and_duplicate_policy_ids() {
    let tag_error = Resource::new(ResourceCreate {
        tags: BTreeMap::from([("oyatie:internal".to_string(), "no".to_string())]),
        ..compute_resource_create()
    })
    .expect_err("tenant tags cannot use the reserved Oyatie prefix");
    assert_eq!(tag_error, CloudResourceError::InvalidTagKey);

    let policy_error = Resource::new(ResourceCreate {
        iam_policy_attachments: vec![
            "pol_cloud_compute_admin".to_string(),
            "pol_cloud_compute_admin".to_string(),
        ],
        ..compute_resource_create()
    })
    .expect_err("policy attachments must be unique");
    assert_eq!(policy_error, CloudResourceError::DuplicatePolicyId);
}

#[test]
fn rejects_residency_region_mismatch_and_wrong_metering_tag() {
    let residency_error = Resource::new(ResourceCreate {
        region: "region-gamma1".to_string(),
        az: Some("region-gamma1-a".to_string()),
        cell_id: "cell-region-gamma1-a-001".to_string(),
        id: "oyatie:cloud:region-gamma1:ten_alpha:instance:api-001".to_string(),
        ..compute_resource_create()
    })
    .expect_err("pack residency cannot move to a forbidden region");
    assert_eq!(residency_error, CloudResourceError::ResidencyRegionMismatch);

    let metering_error = Resource::new(ResourceCreate {
        metering_tag: "oyatie:metering:ten_alpha:bucket".to_string(),
        ..compute_resource_create()
    })
    .expect_err("metering tag must match tenant and kind");
    assert_eq!(metering_error, CloudResourceError::InvalidMeteringTag);
}
