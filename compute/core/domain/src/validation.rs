use std::collections::BTreeSet;

use cell_region::{AzCode, CellId, RegionCode};
use compute_resource::{CloudResourceError, ResourceId, ResourceKind};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use network_domain::SecurityGroupId;
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};

use crate::CloudComputeError;

const TENANT_ID_PREFIX: &str = "ten_";

pub(crate) fn security_groups(
    input: Vec<String>,
) -> Result<Vec<SecurityGroupId>, CloudComputeError> {
    if input.is_empty() {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    let mut seen = BTreeSet::new();
    let mut groups = Vec::with_capacity(input.len());
    for id in input {
        let id = SecurityGroupId::new(id).map_err(|_| CloudComputeError::ResourceKindMismatch)?;
        if !seen.insert(id.clone()) {
            return Err(CloudComputeError::ResourceKindMismatch);
        }
        groups.push(id);
    }
    Ok(groups)
}

pub(crate) fn privacy_classes(
    input: Vec<DataClass>,
) -> Result<Vec<PrivacyDataClass>, CloudComputeError> {
    let mut seen = BTreeSet::new();
    let mut classes = Vec::with_capacity(input.len());
    for data_class in input {
        let data_class =
            PrivacyDataClass::new(data_class).map_err(|_| CloudComputeError::InvalidDataClass)?;
        if seen.insert(data_class) {
            classes.push(data_class);
        }
    }
    Ok(classes)
}

pub(crate) fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind: ResourceKind,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind.type_label() {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

pub(crate) fn resource_ref_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind_label: &str,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind_label {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

pub(crate) fn resource_id_for_kind_label(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind_label: &str,
) -> Result<ResourceId, CloudComputeError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudComputeError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudComputeError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind_label {
        return Err(CloudComputeError::ResourceKindMismatch);
    }
    Ok(id)
}

pub(crate) fn region_for(
    value: &str,
    residency: &ResidencyClass,
) -> Result<RegionCode, CloudComputeError> {
    let region =
        RegionCode::new(value.to_string()).map_err(|_| CloudComputeError::InvalidResourceId)?;
    if !residency_class_allows_home_region_label(residency, &region.value) {
        return Err(CloudComputeError::ResidencyRegionMismatch);
    }
    Ok(region)
}

pub(crate) fn validate_tenant_id(value: &str) -> Result<(), CloudComputeError> {
    let Some(suffix) = value.strip_prefix(TENANT_ID_PREFIX) else {
        return Err(CloudComputeError::InvalidTenantId);
    };
    if suffix.is_empty()
        || suffix.starts_with('-')
        || suffix.ends_with('-')
        || suffix.contains("--")
        || !suffix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(CloudComputeError::InvalidTenantId);
    }
    Ok(())
}

pub(crate) fn validate_az_region(
    az: &AzCode,
    region: &RegionCode,
) -> Result<(), CloudComputeError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudComputeError::AzRegionMismatch)
    }
}

pub(crate) fn validate_cell_az(cell_id: &CellId, az: &AzCode) -> Result<(), CloudComputeError> {
    let expected_prefix = format!("cell-{}-", az.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudComputeError::CellAzMismatch)
    }
}

pub(crate) fn validate_cell_region(
    cell_id: &CellId,
    region: &RegionCode,
) -> Result<(), CloudComputeError> {
    let expected_prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudComputeError::CellAzMismatch)
    }
}

pub(crate) fn safe_ref_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("..")
}

pub(crate) fn looks_secret_like(value: &str) -> bool {
    const SECRET_MARKERS: &[&str] = &[
        "password",
        "passwd",
        "secret",
        "credential",
        "private_key",
        "private-key",
        "api_key",
        "api-key",
        "access_key",
        "access-key",
        "secret_key",
        "secret-key",
        "session_token",
        "access_token",
        "refresh_token",
        "token=",
        "token:",
        "bearer ",
        "bearer_",
        "-----begin",
        "kubeconfig",
    ];
    let lower = value.to_ascii_lowercase();
    SECRET_MARKERS.iter().any(|marker| lower.contains(marker))
}

pub(crate) fn public_metadata_class(
    data_class: DataClass,
) -> Result<PrivacyDataClass, CloudComputeError> {
    if data_class != DataClass::Public {
        return Err(CloudComputeError::InvalidDataClass);
    }
    PrivacyDataClass::new(data_class).map_err(|_| CloudComputeError::InvalidDataClass)
}

pub(crate) fn prefixed_token(
    value: String,
    prefix: &str,
    error: CloudComputeError,
) -> Result<String, CloudComputeError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

pub(crate) fn map_resource_error(error: CloudResourceError) -> CloudComputeError {
    match error {
        CloudResourceError::InvalidResourceId => CloudComputeError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudComputeError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudComputeError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudComputeError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudComputeError::InvalidTenantId,
        _ => CloudComputeError::InvalidResourceId,
    }
}

pub(crate) fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

pub(crate) fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
