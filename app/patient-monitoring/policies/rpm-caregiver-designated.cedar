// rpm-caregiver-designated.cedar
// Family caregivers designated via consent-graph may view patient's RPM data.

permit (
    principal in oya::patient_monitoring::Role::"rpm_caregiver_designated",
    action in [
        oya::patient_monitoring::Action::"view_designated_vitals",
        oya::patient_monitoring::Action::"view_designated_dashboard",
        oya::patient_monitoring::Action::"view_designated_trends"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && resource.patient_id in principal.designated_patient_ids
    && principal.consent_active == true
    && principal.designation_expires_at_hlc > context.now_hlc
};

// Forbid acknowledge/suppress/code-blue by caregivers
forbid (
    principal in oya::patient_monitoring::Role::"rpm_caregiver_designated",
    action in [
        oya::patient_monitoring::Action::"acknowledge_alarm",
        oya::patient_monitoring::Action::"suppress_alarm",
        oya::patient_monitoring::Action::"activate_code_blue"
    ],
    resource
);
