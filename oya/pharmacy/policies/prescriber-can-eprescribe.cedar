// Prescriber-can-eprescribe Cedar policy
// Authority: ADR-0243 (Cedar universal gate); ADR-0332 (pharmacy substrate)
// Effect: only a DEA-bound prescriber with valid state license can ePrescribe a controlled substance.

@id("prescriber-can-eprescribe-noncontrolled")
permit (
    principal in Group::"pharmacy.prescribers",
    action == Action::"pharmacy.eprescribe",
    resource is EPrescription
)
when {
    principal has license &&
    principal.license.status == "active" &&
    principal.license.kind in ["MD", "DO", "NP", "PA", "DDS", "DMD", "OD", "DPM"] &&
    resource.tenant_id == principal.tenant_id &&
    (!(resource has schedule) || resource.schedule == "NONE")
};

// EPCS path: schedule II–V require DEA registration AND KMS-bound key.
@id("prescriber-can-eprescribe-controlled")
permit (
    principal in Group::"pharmacy.prescribers",
    action == Action::"pharmacy.eprescribe",
    resource is EPrescription
)
when {
    principal has license &&
    principal.license.status == "active" &&
    resource has schedule &&
    resource.schedule in ["CII", "CIII", "CIV", "CV"] &&
    principal has dea &&
    principal.dea.status == "active" &&
    principal has epcs_kms_key_binding &&
    principal.epcs_kms_key_binding.bound == true &&
    resource.epcs_two_factor_evidence_present == true
};
