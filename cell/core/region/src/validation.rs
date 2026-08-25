use std::collections::BTreeSet;

use cell_location::CELL_ID_PREFIX;
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};

use crate::model::{
    CellCapacity, CellUtilization, CloudRegion, CloudRegionError, HSM_PARTITION_PREFIX,
    REGIONAL_PACK_ID_PREFIX, TENANT_ID_PREFIX,
};
use crate::{AzCode, CellId, RegionCode};

pub(crate) fn region_allows_residency(
    region: &CloudRegion,
    residency_class: &ResidencyClass,
) -> bool {
    residency_class_allows_home_region_label(residency_class, &region.code.value.value)
}

pub(crate) fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudRegionError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudRegionError::AzRegionMismatch)
    }
}

pub(crate) fn validate_regulatory_packs(packs: &[String]) -> Result<(), CloudRegionError> {
    validate_non_empty_set(
        packs,
        CloudRegionError::EmptyRegulatoryPackSet,
        CloudRegionError::InvalidRegulatoryPack,
        CloudRegionError::DuplicateRegulatoryPack,
        |pack| pack.starts_with(REGIONAL_PACK_ID_PREFIX),
    )
}

pub(crate) fn validate_power_zones(power_zones: &[String]) -> Result<(), CloudRegionError> {
    validate_non_empty_set(
        power_zones,
        CloudRegionError::EmptyPowerZoneSet,
        CloudRegionError::InvalidPowerZone,
        CloudRegionError::DuplicatePowerZone,
        |_| true,
    )
}

pub(crate) fn validate_allowed_residency(
    region_code: &RegionCode,
    residency_classes: &[ResidencyClass],
) -> Result<(), CloudRegionError> {
    if residency_classes.is_empty() {
        return Err(CloudRegionError::EmptyAllowedResidencySet);
    }
    let mut seen = BTreeSet::new();
    for residency_class in residency_classes {
        if !seen.insert(residency_class.clone()) {
            return Err(CloudRegionError::DuplicateAllowedResidencyClass);
        }
        if !residency_class_allows_home_region_label(residency_class, &region_code.value) {
            return Err(CloudRegionError::CellResidencyNotAllowedInRegion);
        }
    }
    Ok(())
}

pub(crate) fn validate_capacity(
    capacity: CellCapacity,
    utilization: CellUtilization,
) -> Result<(), CloudRegionError> {
    if !capacity.has_required_capacity() {
        return Err(CloudRegionError::InvalidCapacity);
    }
    if !capacity.contains(utilization) {
        return Err(CloudRegionError::UtilizationExceedsCapacity);
    }
    Ok(())
}

pub(crate) fn validate_cell_id_namespace(
    cell_id: &CellId,
    az_code: &AzCode,
) -> Result<(), CloudRegionError> {
    let expected_prefix = format!("{CELL_ID_PREFIX}{}-", az_code.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudRegionError::CellAzMismatch)
    }
}

pub(crate) fn validate_hsm_partition_ref(
    value: &str,
    region_code: &RegionCode,
    cell_id: &CellId,
) -> Result<(), CloudRegionError> {
    validate_non_empty(value, CloudRegionError::InvalidHsmPartitionRef)?;
    let expected = format!(
        "{HSM_PARTITION_PREFIX}{}/{}",
        region_code.value, cell_id.value
    );
    if value == expected {
        Ok(())
    } else {
        Err(CloudRegionError::InvalidHsmPartitionRef)
    }
}

pub(crate) fn validate_tenant_id(value: &str) -> Result<(), CloudRegionError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudRegionError::InvalidTenantId)
    }
}

pub(crate) fn validate_non_empty(
    value: &str,
    error: CloudRegionError,
) -> Result<(), CloudRegionError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty_set(
    values: &[String],
    empty_error: CloudRegionError,
    invalid_error: CloudRegionError,
    duplicate_error: CloudRegionError,
    accepts: impl Fn(&str) -> bool,
) -> Result<(), CloudRegionError> {
    if values.is_empty() {
        return Err(empty_error);
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() || !accepts(value) {
            return Err(invalid_error);
        }
        if !seen.insert(value) {
            return Err(duplicate_error);
        }
    }
    Ok(())
}
