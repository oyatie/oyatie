// Cedar policy — break-glass.emergency
// Owning µservice: emr
// Authored Wave 15M-B 2026-05-21
// Per ADR-0243 + ADR-0251 + PRD §C US-HIM-016 mandatory review

// Permit clinical-emergency break-glass invocation when a structured justification
// is provided and a mandatory retrospective review is queued.

permit (
    principal in Group::"clinician",
    action == Action::"emr.break-glass.invoke",
    resource
)
when {
    principal.tenant_id == resource.tenant_id &&

    // Structured justification mandatory
    context.justification_category in [
        "clinical-emergency",
        "patient-relationship-emergency",
        "IT-emergency"
    ] &&

    // Justification text minimum length
    context.justification_text_length >= 20 &&

    // AAL3 step-up required for break-glass
    context.session_assurance_level == "AAL3" &&

    // Review queue active
    context.review_workflow_armed == true &&

    "HIPAA-2024" in context.installed_packs
};

// Forbid: cannot break-glass without an active session and step-up
forbid (
    principal,
    action == Action::"emr.break-glass.invoke",
    resource
)
when {
    context.session_assurance_level != "AAL3" ||
    context.review_workflow_armed == false
};
