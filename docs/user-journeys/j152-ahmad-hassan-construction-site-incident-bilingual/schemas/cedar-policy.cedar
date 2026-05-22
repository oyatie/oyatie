// Cedar policy fragment — journey j152
// ADR anchors:
//   ADR-0243 (Cedar as universal gate)
//   ADR-0298 (emergency-services bypass, life-safety)
//   ADR-0244 (tenant scoping invariant)
//   ADR-0311 (dual-tenant identity)
//   ADR-0317 (role-projection)

// ---------------------------------------------------------------------------
// Site-lead permit set (Ahmad Hassan)
// ---------------------------------------------------------------------------

permit (
    principal == User::"ahmad.hassan@halcyon-build.com",
    action in [
        Action::"incident.create",
        Action::"incident.stop_work_broadcast",
        Action::"incident.add_narrative_voice",
        Action::"incident.add_witness",
        Action::"incident.attach_photo",
        Action::"workplace_integration.paycom.write_incident"
    ],
    resource is Site
) when {
    principal.passkey_step_up_within_seconds(120) &&
    resource.tenant_id == "halcyon_build_llc" &&
    principal.role_on_site(resource.site_id) == "site_lead" &&
    context.incident_class in ["near_miss", "first_aid", "recordable", "serious", "escalate_to_911", "fatality"]
};

// ---------------------------------------------------------------------------
// ADR-0298 medical-excerpt bypass (acute window, 60 minutes)
// ---------------------------------------------------------------------------

permit (
    principal,
    action == Action::"incident.attach_medical_excerpt",
    resource is Site
) when {
    principal.passkey_step_up_within_seconds(120) &&
    resource.tenant_id == "halcyon_build_llc" &&
    principal.role_on_site(resource.site_id) == "site_lead" &&
    context.adr_0298_bypass_active == true &&
    context.acute_window_minutes <= 60 &&
    context.consent_token_present == true &&
    context.consent_token_scope == "allergy_excerpt" &&
    context.consent_token_revoked == false &&
    context.affected_worker_id != "" &&
    principal.has_consent_token(context.affected_worker_id, "allergy_excerpt") == true
};

// ---------------------------------------------------------------------------
// EMS share-link minting (connect)
// ---------------------------------------------------------------------------

permit (
    principal,
    action == Action::"connect.mint_ems_one_time_link",
    resource is Incident
) when {
    principal.passkey_step_up_within_seconds(120) &&
    resource.tenant_id == "halcyon_build_llc" &&
    principal.role_on_site(resource.site_id) == "site_lead" &&
    context.psap_dispatch_id != "" &&
    context.ttl_minutes <= 60 &&
    context.permitted_fields containsAll ["allergies", "current_medications"] &&
    context.permitted_fields notContains "full_medical_record"
};

// ---------------------------------------------------------------------------
// HSE officer permit set (Priya Mehrotra)
// ---------------------------------------------------------------------------

permit (
    principal == User::"priya.mehrotra@halcyon-build.com",
    action in [
        Action::"compliance.calosha.file_ruling",
        Action::"compliance.calosha.file_courtesy",
        Action::"incident.add_followup_task",
        Action::"workflow.cancel_timer"
    ],
    resource is Incident
) when {
    principal.passkey_step_up_within_seconds(180) &&
    resource.tenant_id == "halcyon_build_llc" &&
    principal.role == "hse_officer"
};

// ---------------------------------------------------------------------------
// Non-site-lead — explicit deny on medical excerpt
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"incident.attach_medical_excerpt",
    resource
) when {
    principal.role_on_site(resource.site_id) != "site_lead" ||
    !principal.passkey_step_up_within_seconds(120)
};

// ---------------------------------------------------------------------------
// Cross-tenant drive attachment — explicit deny
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"drive.attach_evidence",
    resource is Incident
) when {
    context.source_tenant_id != resource.tenant_id
};

// ---------------------------------------------------------------------------
// Stale-bypass — explicit deny
// ---------------------------------------------------------------------------

forbid (
    principal,
    action == Action::"incident.attach_medical_excerpt",
    resource is Site
) when {
    context.acute_window_minutes > 60 ||
    context.adr_0298_bypass_active == false
};
