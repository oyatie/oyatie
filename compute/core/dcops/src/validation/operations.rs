use std::collections::BTreeSet;

use data_boundary_kernel::DataClass;

use super::{validate_positive_time, validate_ref_path};
use crate::{
    CloudDcopsError, DatacenterSiteId, EquipmentInstallPlan, EquipmentInstallation, EquipmentKind,
    PowerRedundancy,
};

const MIN_RACK_U_HEIGHT: u16 = 24;
const MAX_RACK_U_HEIGHT: u16 = 60;
const MAX_FIBER_LOSS_MILLI_DB: u32 = 30_000;

pub(crate) fn validate_power_redundancy(
    redundancy: PowerRedundancy,
    utility_feed_count: u8,
) -> Result<(), CloudDcopsError> {
    let required = match redundancy {
        PowerRedundancy::N => 1,
        PowerRedundancy::NPlusOne | PowerRedundancy::TwoN => 2,
        PowerRedundancy::TwoNPlusOne => 3,
    };
    if utility_feed_count >= required {
        Ok(())
    } else {
        Err(CloudDcopsError::InvalidRedundancy)
    }
}

pub(crate) fn validate_rack_shape(
    u_height: u16,
    rated_power_watts: u64,
    max_heat_watts: u64,
    max_weight_kg: u64,
) -> Result<(), CloudDcopsError> {
    if !(MIN_RACK_U_HEIGHT..=MAX_RACK_U_HEIGHT).contains(&u_height)
        || rated_power_watts == 0
        || max_heat_watts == 0
        || max_weight_kg == 0
    {
        Err(CloudDcopsError::InvalidCapacity)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_install_shape(
    input: &EquipmentInstallPlan,
    kind: EquipmentKind,
) -> Result<(), CloudDcopsError> {
    validate_positive_time(input.installed_at_epoch_seconds)?;
    if input.start_u == 0 || input.height_u == 0 || input.weight_kg == 0 {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    if kind.requires_power() && (input.power_watts == 0 || input.heat_watts == 0) {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    if !kind.requires_power() && input.heat_watts > 0 && input.power_watts == 0 {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    typed_network_drop_refs(&input.network_drop_refs)?;
    Ok(())
}

pub(crate) fn typed_network_drop_refs(values: &[String]) -> Result<Vec<String>, CloudDcopsError> {
    let mut seen = BTreeSet::new();
    let mut refs = Vec::with_capacity(values.len());
    for value in values {
        validate_ref_path(value, "netdrop", CloudDcopsError::InvalidInstallPlan)?;
        if !seen.insert(value.clone()) {
            return Err(CloudDcopsError::InvalidInstallPlan);
        }
        refs.push(value.clone());
    }
    Ok(refs)
}

pub(crate) fn installation_end_u(
    installation: &EquipmentInstallation,
) -> Result<u16, CloudDcopsError> {
    installation
        .start_u
        .checked_add(installation.height_u)
        .and_then(|value| value.checked_sub(1))
        .ok_or(CloudDcopsError::InvalidRackUnits)
}

pub(crate) fn u_ranges_overlap(start_a: u16, end_a: u16, start_b: u16, end_b: u16) -> bool {
    start_a <= end_b && start_b <= end_a
}

pub(crate) fn validate_port(value: &str) -> Result<(), CloudDcopsError> {
    if value.trim().is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.'))
    {
        Err(CloudDcopsError::InvalidPort)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_cable_loss(
    measured_milli_db: u32,
    budget_milli_db: u32,
) -> Result<(), CloudDcopsError> {
    if budget_milli_db == 0
        || budget_milli_db > MAX_FIBER_LOSS_MILLI_DB
        || measured_milli_db > budget_milli_db
    {
        Err(CloudDcopsError::InvalidCableLoss)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_unit(value: &str) -> Result<(), CloudDcopsError> {
    if matches!(
        value,
        "milli-celsius" | "milli-percent" | "milli-liter" | "milli-watt" | "boolean"
    ) {
        Ok(())
    } else {
        Err(CloudDcopsError::InvalidBmsReading)
    }
}

pub(crate) fn validate_work_order_data_class(data_class: DataClass) -> Result<(), CloudDcopsError> {
    match data_class {
        DataClass::InternalOnly | DataClass::PiiIdentifying | DataClass::PiiQuasiIdentifier => {
            Ok(())
        }
        _ => Err(CloudDcopsError::InvalidDataClass),
    }
}

pub(crate) fn validate_sustainability_data_class(
    data_class: DataClass,
) -> Result<(), CloudDcopsError> {
    match data_class {
        DataClass::InternalOnly | DataClass::Financial | DataClass::BehavioralTenantProduct => {
            Ok(())
        }
        _ => Err(CloudDcopsError::InvalidDataClass),
    }
}

pub(crate) fn exact_ratio_milli(numerator: u64, denominator: u64) -> Result<u64, CloudDcopsError> {
    let scaled = numerator
        .checked_mul(1_000)
        .ok_or(CloudDcopsError::InvalidTargetRatio)?;
    if denominator == 0 || scaled % denominator != 0 {
        return Err(CloudDcopsError::InvalidTargetRatio);
    }
    Ok(scaled / denominator)
}

pub(crate) fn validate_non_empty(value: &str) -> Result<(), CloudDcopsError> {
    if value.trim().is_empty() {
        Err(CloudDcopsError::InvalidText)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_same_site(
    expected: &DatacenterSiteId,
    actual: &DatacenterSiteId,
) -> Result<(), CloudDcopsError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CloudDcopsError::CrossSiteReference)
    }
}
