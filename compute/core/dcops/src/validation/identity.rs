use cell_region::{AzCode, RegionCode};

use crate::CloudDcopsError;
use crate::identifiers::{DatacenterSiteId, PHYSICAL_REF_PREFIX};

pub(crate) fn validate_path(
    value: &str,
    prefix: &str,
    min_segments: usize,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() < min_segments || parts.first().copied() != Some(prefix) {
        return Err(error);
    }
    for part in parts.iter().skip(1) {
        validate_path_segment(part).map_err(|_| error.clone())?;
    }
    Ok(())
}

pub(crate) fn validate_child_id(
    value: &str,
    prefix: &str,
    parent_id: &str,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    let required = format!("{prefix}/{parent_id}/");
    if value.starts_with(&required) {
        Ok(())
    } else {
        Err(error)
    }
}

pub(crate) fn validate_path_segment(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Err(())
    } else {
        Ok(())
    }
}

pub(crate) fn site_region_from_path(value: &str) -> Result<&str, CloudDcopsError> {
    value
        .split('/')
        .nth(1)
        .ok_or(CloudDcopsError::InvalidDatacenterSiteId)
}

pub(crate) fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudDcopsError> {
    let prefix = format!("{}-", region.value);
    if az.value.starts_with(&prefix) && az.value.len() > prefix.len() {
        Ok(())
    } else {
        Err(CloudDcopsError::AzRegionMismatch)
    }
}

pub(crate) fn validate_physical_ref(value: &str) -> Result<(), CloudDcopsError> {
    validate_ref_path(
        value,
        PHYSICAL_REF_PREFIX,
        CloudDcopsError::InvalidPhysicalRef,
    )
}

pub(crate) fn validate_ref_path(
    value: &str,
    prefix: &str,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    validate_path(value, prefix, 3, error)
}

pub(crate) fn validate_positive_time(value: u64) -> Result<(), CloudDcopsError> {
    if value == 0 {
        Err(CloudDcopsError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_time_order(start: u64, end: u64) -> Result<(), CloudDcopsError> {
    if start == 0 || end <= start {
        Err(CloudDcopsError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_sustainability_targets(
    pue_target_milli: u32,
    wue_target_milli: u32,
    cue_target_milli: u32,
) -> Result<(), CloudDcopsError> {
    if !(1_000..=2_500).contains(&pue_target_milli)
        || wue_target_milli == 0
        || wue_target_milli > 10_000
        || cue_target_milli == 0
        || cue_target_milli > 10_000
    {
        Err(CloudDcopsError::InvalidTargetRatio)
    } else {
        Ok(())
    }
}
