use crate::{Equipment, EquipmentInstallation, EquipmentLifecycle, Rack, RackCapacitySnapshot};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct InstallationAccounting {
    pub(crate) rack_capacity: RackCapacitySnapshot,
    pub(crate) power_zone_used_watts: u64,
    pub(crate) cooling_zone_used_watts: u64,
    pub(crate) rack_unit_overlap: bool,
}

pub(crate) fn equipment_counts_against_capacity(equipment: &Equipment) -> bool {
    equipment.lifecycle.value != EquipmentLifecycle::EwasteTransferred
        && equipment.installation.value.is_some()
}

pub(crate) fn add_installation_capacity(
    capacity: &mut RackCapacitySnapshot,
    installation: &EquipmentInstallation,
) {
    capacity.used_u = capacity.used_u.saturating_add(installation.height_u);
    capacity.used_power_watts = capacity
        .used_power_watts
        .saturating_add(installation.power_watts);
    capacity.used_heat_watts = capacity
        .used_heat_watts
        .saturating_add(installation.heat_watts);
    capacity.used_weight_kg = capacity
        .used_weight_kg
        .saturating_add(installation.weight_kg);
}

pub(crate) fn subtract_installation_capacity(
    capacity: &mut RackCapacitySnapshot,
    installation: &EquipmentInstallation,
) {
    capacity.used_u = capacity.used_u.saturating_sub(installation.height_u);
    capacity.used_power_watts = capacity
        .used_power_watts
        .saturating_sub(installation.power_watts);
    capacity.used_heat_watts = capacity
        .used_heat_watts
        .saturating_sub(installation.heat_watts);
    capacity.used_weight_kg = capacity
        .used_weight_kg
        .saturating_sub(installation.weight_kg);
}

pub(crate) fn finalize_rack_capacity(rack: &Rack, capacity: &mut RackCapacitySnapshot) {
    capacity.free_u = rack.u_height.value.saturating_sub(capacity.used_u);
    capacity.remaining_power_watts = rack
        .rated_power_watts
        .value
        .saturating_sub(capacity.used_power_watts);
    capacity.remaining_heat_watts = rack
        .max_heat_watts
        .value
        .saturating_sub(capacity.used_heat_watts);
    capacity.remaining_weight_kg = rack
        .max_weight_kg
        .value
        .saturating_sub(capacity.used_weight_kg);
}
