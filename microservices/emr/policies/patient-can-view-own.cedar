// Cedar policy — patient.can-view-own
// Owning µservice: emr
// Owning BC: portal-session
// HIPAA §164.524 — patient right of access
// Authored Wave 15M-B 2026-05-21

// Permit a patient (or their valid proxy) to view their own chart resources.

permit (
    principal in Group::"patient",
    action in [
        Action::"emr.patient.read",
        Action::"emr.encounter.read",
        Action::"emr.problem.read",
        Action::"emr.medication.read",
        Action::"emr.medication.list",
        Action::"emr.allergy.search",
        Action::"emr.vital.search",
        Action::"emr.note.read",
        Action::"emr.result.list",
        Action::"emr.care-plan.read",
        Action::"emr.care-team.read",
        Action::"emr.immunization.list",
        Action::"emr.portal-session.bulk-export"
    ],
    resource
)
when {
    principal.id == resource.patient_id &&
    principal.tenant_id == resource.tenant_id
};

// Permit a verified proxy to view chart resources of the subject patient
permit (
    principal in Group::"patient.proxy",
    action in [
        Action::"emr.patient.read",
        Action::"emr.encounter.read",
        Action::"emr.problem.read",
        Action::"emr.medication.list",
        Action::"emr.allergy.search",
        Action::"emr.vital.search",
        Action::"emr.result.list",
        Action::"emr.care-plan.read"
    ],
    resource
)
when {
    principal.proxy_grant_active == true &&
    resource.patient_id == principal.subject_patient_id &&
    principal.tenant_id == resource.tenant_id &&
    context.session_assurance_level in ["AAL2", "AAL3"]
};

// Forbid: 42 CFR Part 2 SUD records require explicit consent re-grant per access
forbid (
    principal,
    action,
    resource
)
when {
    resource.data_class == "substance-use-disorder-42-cfr-part-2" &&
    context.consent_state != "GRANTED_42_CFR_PART_2"
};
