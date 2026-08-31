use crate::{
    BmsPointState, CableState, CoolingZoneState, DatacenterState, EquipmentLifecycle,
    FacilityZoneState, PowerZoneState, RackState, SecurityZoneState,
};

pub(crate) fn datacenter_transition_allowed(
    current: DatacenterState,
    next: DatacenterState,
) -> bool {
    matches!(
        (current, next),
        (DatacenterState::Planned, DatacenterState::Commissioning)
            | (DatacenterState::Commissioning, DatacenterState::Active)
            | (DatacenterState::Active, DatacenterState::Draining)
            | (DatacenterState::Draining, DatacenterState::Retired)
    )
}

pub(crate) fn facility_zone_transition_allowed(
    current: FacilityZoneState,
    next: FacilityZoneState,
) -> bool {
    matches!(
        (current, next),
        (FacilityZoneState::Planned, FacilityZoneState::Active)
            | (FacilityZoneState::Active, FacilityZoneState::Isolated)
            | (FacilityZoneState::Isolated, FacilityZoneState::Active)
            | (FacilityZoneState::Isolated, FacilityZoneState::Retired)
    )
}

pub(crate) fn power_zone_transition_allowed(current: PowerZoneState, next: PowerZoneState) -> bool {
    matches!(
        (current, next),
        (PowerZoneState::Planned, PowerZoneState::Energized)
            | (PowerZoneState::Energized, PowerZoneState::Maintenance)
            | (PowerZoneState::Maintenance, PowerZoneState::Energized)
            | (PowerZoneState::Maintenance, PowerZoneState::Retired)
    )
}

pub(crate) fn cooling_zone_transition_allowed(
    current: CoolingZoneState,
    next: CoolingZoneState,
) -> bool {
    matches!(
        (current, next),
        (CoolingZoneState::Planned, CoolingZoneState::Active)
            | (CoolingZoneState::Active, CoolingZoneState::Maintenance)
            | (CoolingZoneState::Maintenance, CoolingZoneState::Active)
            | (CoolingZoneState::Maintenance, CoolingZoneState::Retired)
    )
}

pub(crate) fn security_zone_transition_allowed(
    current: SecurityZoneState,
    next: SecurityZoneState,
) -> bool {
    matches!(
        (current, next),
        (SecurityZoneState::Planned, SecurityZoneState::Armed)
            | (SecurityZoneState::Armed, SecurityZoneState::Isolated)
            | (SecurityZoneState::Isolated, SecurityZoneState::Armed)
            | (SecurityZoneState::Isolated, SecurityZoneState::Retired)
    )
}

pub(crate) fn rack_transition_allowed(current: RackState, next: RackState) -> bool {
    matches!(
        (current, next),
        (RackState::Planned, RackState::Active)
            | (RackState::Active, RackState::Quarantined)
            | (RackState::Quarantined, RackState::Active)
            | (RackState::Quarantined, RackState::Retired)
    )
}

pub(crate) fn equipment_lifecycle_transition_allowed(
    current: EquipmentLifecycle,
    next: EquipmentLifecycle,
) -> bool {
    matches!(
        (current, next),
        (EquipmentLifecycle::Installed, EquipmentLifecycle::InService)
            | (
                EquipmentLifecycle::InService,
                EquipmentLifecycle::Maintenance
            )
            | (
                EquipmentLifecycle::Maintenance,
                EquipmentLifecycle::InService
            )
            | (
                EquipmentLifecycle::InService,
                EquipmentLifecycle::Decommissioning
            )
            | (
                EquipmentLifecycle::Maintenance,
                EquipmentLifecycle::Decommissioning
            )
            | (
                EquipmentLifecycle::Decommissioning,
                EquipmentLifecycle::Sanitized
            )
            | (
                EquipmentLifecycle::Sanitized,
                EquipmentLifecycle::EwasteTransferred
            )
    )
}

pub(crate) fn cable_transition_allowed(current: CableState, next: CableState) -> bool {
    matches!(
        (current, next),
        (CableState::Planned, CableState::Installed)
            | (CableState::Installed, CableState::Certified)
            | (CableState::Certified, CableState::Retired)
    )
}

pub(crate) fn bms_point_transition_allowed(current: BmsPointState, next: BmsPointState) -> bool {
    matches!(
        (current, next),
        (BmsPointState::Commissioning, BmsPointState::Enabled)
            | (BmsPointState::Enabled, BmsPointState::Disabled)
            | (BmsPointState::Disabled, BmsPointState::Enabled)
            | (BmsPointState::Disabled, BmsPointState::Retired)
    )
}
