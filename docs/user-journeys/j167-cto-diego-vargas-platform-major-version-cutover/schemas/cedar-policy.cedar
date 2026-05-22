// j167 — Cedar policy bundle for Aurelia Platform v3.x → v4.0 cutover
// Implements ADR-0243 (Cedar as universal gate), ADR-0244 (tenant scoping),
// ADR-0263 (audit dual-seal), ADR-0252 (TrueTime fence ≤ 10ms), ADR-0110 (changeset state machine).

// =====================================================================
// READINESS READ — restricted to the cutover quorum group
// =====================================================================
permit (
    principal,
    action == Action::"governance.readiness_read",
    resource is ChangeRecord
) when {
    resource.change_record_id like "CHG-V4-CUTOVER-*" &&
    principal in Group::"aurelia-cutover-quorum-members" &&
    principal.tenant == "aurelia-robotics-internacional-sa-de-cv-mx"
};

// =====================================================================
// COHORT TRANSITION VOTE — 4-of-4 quorum with multiple preconditions
// =====================================================================
permit (
    principal,
    action == Action::"governance.cohort_transition_vote",
    resource is CohortGate
) when {
    resource.target_cohort in ["cohort_a", "cohort_b", "cohort_c", "cohort_d"] &&
    resource.quorum_count >= 4 &&
    resource.previous_cohort_slo_green == true &&
    resource.cra_document_signed == true &&
    context.business_hours_cdt == true &&
    context.truetime_uncertainty_ms <= 10 &&
    context.unicode_normalization == "NFC" &&
    principal in Group::"aurelia-cutover-quorum-members" &&
    principal.passkey_attested == true
};

// Forbid cohort vote outside business hours unless emergency override
forbid (
    principal,
    action == Action::"governance.cohort_transition_vote",
    resource is CohortGate
) when {
    context.business_hours_cdt == false &&
    context.emergency_override_signed_by_ceo == false
};

// =====================================================================
// TERRAFORM APPLY — orchestrator-triggered, gated by cohort permit
// =====================================================================
permit (
    principal,
    action == Action::"cloud-iac.apply_terraform_module",
    resource is TerraformApply
) when {
    principal == User::"system-principal.governance.cutover-orchestrator" &&
    resource.module == "aurelia-platform-v4" &&
    resource.triggered_by.cohort_permit_seal like "EVT-J167-COHORT-*-PERMIT-*" &&
    context.truetime_uncertainty_ms <= 10
};

// =====================================================================
// FEATURE-FLAG TRAFFIC SPLIT — gated by cohort approval + business hours
// =====================================================================
permit (
    principal,
    action == Action::"feature-flags.update_traffic_split",
    resource is FlagRule
) when {
    principal in Group::"aurelia-cutover-quorum-members" ||
    principal == User::"system-principal.governance.cutover-orchestrator"
} when {
    resource.flag_id == "aurelia-fleet-coordinator-version" &&
    resource.cohort_approved == true &&
    context.business_hours_cdt == true
};

// =====================================================================
// PER-TENANT OVERRIDE — for customer hotfix (e.g., Cotrijal scenario)
// =====================================================================
permit (
    principal,
    action == Action::"feature-flags.set_tenant_override",
    resource is FlagTenantOverride
) when {
    principal in Group::"aurelia-customer-success-team" &&
    resource.tenant_id != "aurelia-robotics-internacional-sa-de-cv-mx" &&
    resource.override_expires_at != null &&
    resource.override_expires_at <= context.now + duration("P14D") &&
    resource.hotfix_ticket_id != null
};

// =====================================================================
// OBSERVABILITY SLO READ — operator role
// =====================================================================
permit (
    principal,
    action == Action::"observability.slo_read",
    resource is SLOMetric
) when {
    principal in Group::"aurelia-noc-operators" ||
    principal in Group::"aurelia-cutover-quorum-members"
};

// =====================================================================
// BYTECODE CACHE PRE-WARM — operator role with active incident context
// =====================================================================
permit (
    principal,
    action == Action::"policy-engine.bytecode_pre_warm",
    resource is PreWarmJob
) when {
    principal in Group::"aurelia-noc-operators" &&
    (resource.triggered_by_incident != null || resource.scheduled_maintenance == true)
};

// =====================================================================
// ROLLBACK VOTE — 3-of-4 quorum within 4-hour mitigation window
// =====================================================================
permit (
    principal,
    action == Action::"governance.cohort_rollback",
    resource is CohortGate
) when {
    resource.quorum_count >= 3 &&
    resource.active_sev_level in ["sev-1", "sev-2"] &&
    context.minutes_since_alarm <= 240 &&
    principal in Group::"aurelia-cutover-incident-commanders" &&
    context.truetime_uncertainty_ms <= 10
};

// Forbid rollback vote after 4-hour window
forbid (
    principal,
    action == Action::"governance.cohort_rollback",
    resource is CohortGate
) when {
    context.minutes_since_alarm > 240
};

// =====================================================================
// V3 SUNSET — global flip, requires cohort_d_stable + 72-hour stability + business hours
// =====================================================================
permit (
    principal,
    action == Action::"feature-flags.sunset_legacy_version",
    resource is FlagRule
) when {
    principal == User::"system-principal.governance.cutover-orchestrator" &&
    resource.flag_id == "v3_api_enabled" &&
    context.cohort_d_stable_hours >= 72 &&
    context.business_hours_cdt == true &&
    context.truetime_uncertainty_ms <= 10
};

// =====================================================================
// COMPLIANCE ATTESTATION SUBMIT — CTO sign-off + change-record-closed
// =====================================================================
permit (
    principal,
    action == Action::"compliance.cross_tenant_attestation_submit",
    resource is Attestation
) when {
    principal.role == "cto" &&
    principal.tenant == "aurelia-robotics-internacional-sa-de-cv-mx" &&
    resource.change_record_status == "closed" &&
    resource.target_tenant in [
        "pwc-mexico-soc2-auditor-tenant",
        "dekra-eu-ai-act-notified-body-tenant",
        "mx-finance-ministry-tenant",
        "aurelia-internal-audit-archive"
    ] &&
    resource.signer_qes_attestation != null
};

// =====================================================================
// AUDIT-CHAIN DUAL-SEAL INVARIANT (advisory; enforced by audit-chain µservice)
// =====================================================================
// Every cohort permit + rollback decision + sunset event MUST dual-seal in:
//   1. aurelia-robotics-internacional-sa-de-cv-mx (primary)
//   2. oya-governance-change-management-system-tenant (substrate)
// TrueTime uncertainty ≤ 10 ms required for the seal pair to be valid.

// =====================================================================
// CROSS-TENANT CUSTOMER COMMUNICATION (CS team only)
// =====================================================================
permit (
    principal,
    action == Action::"messenger.send_customer_sitrep",
    resource is CustomerCommunication
) when {
    principal in Group::"aurelia-customer-success-team" &&
    resource.target_tenant != principal.tenant &&
    resource.incident_id != null &&
    resource.content_locale in ["en-US", "es-MX", "pt-BR", "de-DE", "ja-JP"] &&
    context.mls_encryption_active == true
};
