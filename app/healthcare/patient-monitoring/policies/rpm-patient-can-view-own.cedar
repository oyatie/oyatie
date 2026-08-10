// rpm-patient-can-view-own.cedar
// RPM patients (B2C) may view their own vitals and dashboard.

permit (
    principal in oya::patient_monitoring::Role::"rpm_patient",
    action in [
        oya::patient_monitoring::Action::"view_own_vitals",
        oya::patient_monitoring::Action::"view_own_dashboard",
        oya::patient_monitoring::Action::"view_own_trends",
        oya::patient_monitoring::Action::"export_own_data"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && principal.patient_id == resource.patient_id
    && principal.consent_active == true
};

// Forbid view of other patients
forbid (
    principal in oya::patient_monitoring::Role::"rpm_patient",
    action,
    resource
) when {
    principal.patient_id != resource.patient_id
};

// Forbid alarm-suppression by patients
forbid (
    principal in oya::patient_monitoring::Role::"rpm_patient",
    action == oya::patient_monitoring::Action::"suppress_alarm",
    resource
);
