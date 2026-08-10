// Cedar policy — physician.can-prescribe
// Owning µservice: emr
// Owning BC: medication
// Per ADR-0243 universal Cedar gate + ADR-0251 HIPAA-2024 pack
// Authored Wave 15M-B 2026-05-21

// Permit a credentialed physician to prescribe a medication
// when the patient is on their active care-team and consent is granted.
// Schedule II requires EPCS 2FA + PDMP query.

permit (
    principal in Group::"clinician.physician",
    action == Action::"emr.medication.prescribe",
    resource in ResourceType::"FHIR.MedicationRequest"
)
when {
    // Tenant scope (ADR-0244)
    principal.tenant_id == resource.tenant_id &&

    // Care-team membership (HIPAA §164.502 minimum necessary)
    context.care_team_member == true &&

    // Active encounter (or explicit cross-encounter care-team grant)
    (context.encounter_active == true || context.care_team_chronic == true) &&

    // Purpose of use
    context.purpose_of_use == "TREATMENT" &&

    // Minimum-necessary assessment
    context.minimum_necessary_assessed == true &&

    // Session assurance level (AAL2 minimum; AAL3 for Schedule II)
    (
        (context.dea_schedule != "II" && context.session_assurance_level in ["AAL2", "AAL3"]) ||
        (context.dea_schedule == "II" && context.session_assurance_level == "AAL3")
    ) &&

    // EPCS-attestation for controlled substances
    (
        context.dea_schedule == "N/A" ||
        (context.dea_schedule in ["II", "III", "IV", "V"] && context.epcs_2fa_passed == true)
    ) &&

    // PDMP query for Schedule II + III + IV (jurisdiction-pack-overlay decides
    // exact required set; default below)
    (
        context.dea_schedule in ["N/A", "V"] ||
        context.pdmp_queried == true
    ) &&

    // Patient consent state (consent-graph)
    context.consent_state == "GRANTED" &&

    // HIPAA pack active
    "HIPAA-2024" in context.installed_packs
};

// Deny: refuted-allergy conflict (CDS Hooks hard-stop)
forbid (
    principal,
    action == Action::"emr.medication.prescribe",
    resource
)
when {
    context.cds_hard_stop == true &&
    context.cds_hard_stop_reason == "allergy-conflict"
};
