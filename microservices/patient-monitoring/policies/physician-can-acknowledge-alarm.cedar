// physician-can-acknowledge-alarm.cedar
// Physician principals may view + acknowledge + suppress alarms for beds in
// their bed_group authority. Suppression requires justification (see
// alarm-suppression-requires-justification.cedar).

permit (
    principal in oya::patient_monitoring::Role::"physician_attending",
    action in [
        oya::patient_monitoring::Action::"view_bed_vitals",
        oya::patient_monitoring::Action::"view_bed_waveform",
        oya::patient_monitoring::Action::"view_bed_alarms",
        oya::patient_monitoring::Action::"acknowledge_alarm",
        oya::patient_monitoring::Action::"activate_code_blue"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && resource.bed_id in principal.authority_bed_group
    && principal.authority_paused == false
};

// Cross-tenant break-glass requires explicit emergency_view role grant
// (not granted by default).
forbid (
    principal in oya::patient_monitoring::Role::"physician_attending",
    action,
    resource
) when {
    principal.tenant_id != resource.tenant_id
};
