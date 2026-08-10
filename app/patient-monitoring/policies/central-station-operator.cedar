// central-station-operator.cedar
// Central-station operators (charge nurse, unit clerk, tele-ICU operator) may
// view multi-bed unit-wide displays and escalate alarms. They cannot suppress.

permit (
    principal in oya::patient_monitoring::Role::"central_station_operator",
    action in [
        oya::patient_monitoring::Action::"subscribe_unit_view",
        oya::patient_monitoring::Action::"view_bed_vitals",
        oya::patient_monitoring::Action::"view_bed_waveform",
        oya::patient_monitoring::Action::"view_bed_alarms",
        oya::patient_monitoring::Action::"escalate_alarm",
        oya::patient_monitoring::Action::"activate_code_blue"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && resource.unit_id in principal.authority_unit_ids
    && principal.shift_active == true
};

// Forbid suppression
forbid (
    principal in oya::patient_monitoring::Role::"central_station_operator",
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
);

// Forbid device-registry mutation
forbid (
    principal in oya::patient_monitoring::Role::"central_station_operator",
    action == oya::patient_monitoring::Action::"device_registry_write",
    resource
);
