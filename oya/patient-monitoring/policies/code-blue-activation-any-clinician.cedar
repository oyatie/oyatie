// code-blue-activation-any-clinician.cedar
// Per ADR-0332 §B-7: code-blue activation must never be blocked by Cedar.
// Any authorized clinician (nurse, physician, central-station-operator,
// resuscitation-team-member) may activate code-blue for any bed in their tenant.

permit (
    principal in [
        oya::patient_monitoring::Role::"nurse_assigned_to_bed_group",
        oya::patient_monitoring::Role::"physician_attending",
        oya::patient_monitoring::Role::"central_station_operator",
        oya::patient_monitoring::Role::"resuscitation_team_member",
        oya::patient_monitoring::Role::"charge_nurse"
    ],
    action == oya::patient_monitoring::Action::"activate_code_blue",
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && principal.authority_paused == false
};

// Cross-tenant code-blue forbidden (clinical safety; tenant boundary respected
// even under emergency)
forbid (
    principal,
    action == oya::patient_monitoring::Action::"activate_code_blue",
    resource
) when {
    principal.tenant_id != resource.tenant_id
};
