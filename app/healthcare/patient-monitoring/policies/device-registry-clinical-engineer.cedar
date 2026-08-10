// device-registry-clinical-engineer.cedar
// Clinical engineers (biomedical) may read and write the device registry,
// calibration records, and firmware-revision metadata.

permit (
    principal in oya::patient_monitoring::Role::"clinical_engineer",
    action in [
        oya::patient_monitoring::Action::"device_registry_read",
        oya::patient_monitoring::Action::"device_registry_write",
        oya::patient_monitoring::Action::"calibration_write",
        oya::patient_monitoring::Action::"firmware_metadata_write"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id
    && principal.authority_paused == false
};

// Forbid bedside alarm-ack and suppression (they cannot act as clinician)
forbid (
    principal in oya::patient_monitoring::Role::"clinical_engineer",
    action in [
        oya::patient_monitoring::Action::"acknowledge_alarm",
        oya::patient_monitoring::Action::"suppress_alarm"
    ],
    resource
);

// Cross-tenant guard
forbid (
    principal in oya::patient_monitoring::Role::"clinical_engineer",
    action,
    resource
) when {
    principal.tenant_id != resource.tenant_id
};
