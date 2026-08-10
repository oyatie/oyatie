// nurse-can-view-bed-group.cedar
// Bound to ADR-0243 (Cedar universal gate) + ADR-0244 (tenant scoping) +
// ADR-0332 (clinical-realtime substrate authority chain).
//
// A nurse principal may view vitals + waveforms + alarms for beds in the
// assigned bed_group during their active shift.

permit (
    principal in oya::patient_monitoring::Role::"nurse_assigned_to_bed_group",
    action in [
        oya::patient_monitoring::Action::"view_bed_vitals",
        oya::patient_monitoring::Action::"view_bed_waveform",
        oya::patient_monitoring::Action::"view_bed_alarms",
        oya::patient_monitoring::Action::"acknowledge_alarm"
    ],
    resource
)
when {
    // Same tenant
    principal.tenant_id == resource.tenant_id
    // Resource bed is in nurse's assigned bed_group
    && resource.bed_id in principal.assigned_bed_group
    // Nurse is currently on shift
    && principal.shift_active == true
    // Tenant has not paused this nurse's authority
    && principal.authority_paused == false
};

// Forbid suppression — nurses cannot suppress, only acknowledge.
forbid (
    principal in oya::patient_monitoring::Role::"nurse_assigned_to_bed_group",
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
);

// Forbid cross-tenant view (defense in depth even within shuffle-sharded cell)
forbid (
    principal in oya::patient_monitoring::Role::"nurse_assigned_to_bed_group",
    action,
    resource
) when {
    principal.tenant_id != resource.tenant_id
};
