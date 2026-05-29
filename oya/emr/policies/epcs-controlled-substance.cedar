// Cedar policy — epcs.controlled-substance
// Owning µservice: emr
// Owning BC: medication
// Authored Wave 15M-B 2026-05-21
// Per DEA EPCS Rule (21 CFR §1311) + ADR-0188 passkey/WebAuthn

// Permit EPCS (Electronic Prescribing of Controlled Substances) when DEA
// requirements are met: AAL3 2FA, DEA-registrant attestation, PDMP query.

permit (
    principal in Group::"clinician.physician.dea-registrant",
    action == Action::"emr.medication.prescribe.controlled",
    resource
)
when {
    principal.dea_registration_active == true &&
    principal.dea_schedule_authorized in ["II", "III", "IV", "V"] &&
    context.dea_schedule in principal.dea_schedule_authorized &&

    // EPCS 2FA — passkey or hard-token (per DEA)
    context.epcs_2fa_passed == true &&
    context.session_assurance_level == "AAL3" &&

    // PDMP query mandatory at Schedule II + III + IV
    (context.dea_schedule == "V" || context.pdmp_queried == true) &&

    // Audit envelope present
    context.audit_envelope_complete == true &&

    // Tenant + pack
    principal.tenant_id == resource.tenant_id &&
    "HIPAA-2024" in context.installed_packs
};

// Forbid: lapsed DEA registration
forbid (
    principal,
    action == Action::"emr.medication.prescribe.controlled",
    resource
)
when {
    principal.dea_registration_active == false ||
    principal.dea_registration_expires_within_days < 0
};

// Forbid: AAL3 step-up not present
forbid (
    principal,
    action == Action::"emr.medication.prescribe.controlled",
    resource
)
when {
    context.session_assurance_level != "AAL3" ||
    context.epcs_2fa_passed == false
};
