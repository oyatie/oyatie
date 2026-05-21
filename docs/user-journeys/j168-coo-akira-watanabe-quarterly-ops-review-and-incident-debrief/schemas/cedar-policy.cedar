// j168 — Cedar policy bundle for quarterly ops review + SEV-2 debrief + capex gate
// Implements ADR-0243 (Cedar gate), ADR-0244 (tenant scoping), ADR-0263 (audit dual-seal), ADR-0252 (TrueTime).

// =====================================================================
// SNAPSHOT READ — coo OR vp-eng OR ops-director
// =====================================================================
permit (
    principal,
    action == Action::"ops.snapshot_read",
    resource is QuarterlySnapshot
) when {
    principal.role in ["coo", "cto", "ceo", "vp_eng", "ops_director", "svp_cs"] &&
    principal.tenant == "aurelia-robotics-internacional-sa-de-cv-mx"
};

// =====================================================================
// SNAPSHOT WRITE — only at snapshot-generation-time by system principal
// =====================================================================
permit (
    principal,
    action == Action::"ops.snapshot_write",
    resource is QuarterlySnapshot
) when {
    principal == User::"system-principal.ops-dashboard.snapshot-generator" &&
    resource.state == "draft" &&
    context.snapshot_generation_window_open == true
};

// Forbid modification after seal
forbid (
    principal,
    action == Action::"ops.snapshot_modify_after_seal",
    resource is QuarterlySnapshot
) when {
    resource.state == "sealed"
};

// =====================================================================
// INCIDENT DEBRIEF OPEN — coo or ops-director + incident closed
// =====================================================================
permit (
    principal,
    action == Action::"incident.debrief_open",
    resource is IncidentDebrief
) when {
    principal.role in ["coo", "cto", "ops_director", "ceo"] &&
    resource.incident_state == "closed" &&
    context.business_hours_local == true
};

// =====================================================================
// FIVE-WHYS STEP SUBMIT — debrief attendees only
// =====================================================================
permit (
    principal,
    action == Action::"incident.five_whys_step_submit",
    resource is FiveWhysStep
) when {
    principal in Group::"debrief-attendees" &&
    resource.step >= 1 &&
    resource.step <= 5 &&
    resource.previous_step_complete == true
};

// =====================================================================
// CORRECTIVE ACTIONS BULK — debrief facilitator only
// =====================================================================
permit (
    principal,
    action == Action::"tasks.corrective_actions_bulk_create",
    resource is CorrectiveActionsBulk
) when {
    principal.role in ["coo", "ops_director"] &&
    resource.linked_debrief_state == "root_cause_identified"
};

// =====================================================================
// CROSS-TENANT CUSTOMER ATTESTATION — coo + MLS active + incident link
// =====================================================================
permit (
    principal,
    action == Action::"messenger.send_customer_attestation",
    resource is CustomerAttestation
) when {
    principal.role in ["coo", "ceo", "svp_cs"] &&
    resource.incident_id != null &&
    resource.target_tenant != principal.tenant &&
    context.mls_encryption_active == true &&
    context.unicode_normalization == "NFC"
};

// =====================================================================
// MERKLE ATTESTATION — coo or ops-director with joint QES signers
// =====================================================================
permit (
    principal,
    action == Action::"audit-chain.merkle_attest",
    resource is MerkleAttestation
) when {
    principal.role in ["coo", "ops_director"] &&
    resource.signer_count >= 2 &&
    resource.scope_includes_count >= 3
};

// =====================================================================
// CAPEX LINE ITEM > MXN 5M — 5-of-5 quorum required
// =====================================================================
permit (
    principal,
    action == Action::"governance.capex_line_item_approve",
    resource is CapexLineItem
) when {
    resource.amount_mxn > 5000000 &&
    resource.quorum_count >= 5 &&
    resource.okr_cycle == "Q1-2027" &&
    resource.cra_signed == true &&
    context.business_hours_cdt == true &&
    context.truetime_uncertainty_ms <= 10 &&
    principal in Group::"aurelia-capex-quorum-members"
};

// =====================================================================
// CAPEX LINE ITEM <= MXN 5M — bulk vote, 5-of-5 quorum
// =====================================================================
permit (
    principal,
    action == Action::"governance.capex_bulk_approve",
    resource is CapexBulkVote
) when {
    resource.max_line_item_amount_mxn <= 5000000 &&
    resource.quorum_count >= 5 &&
    resource.okr_cycle == "Q1-2027" &&
    context.business_hours_cdt == true &&
    principal in Group::"aurelia-capex-quorum-members"
};

// Forbid capex vote outside business hours
forbid (
    principal,
    action == Action::"governance.capex_line_item_approve",
    resource is CapexLineItem
) when {
    context.business_hours_cdt == false &&
    context.emergency_override_signed_by_ceo == false
};

// =====================================================================
// QUARTERLY REPORT SUBMIT TO AUDITORS — coo sign-off
// =====================================================================
permit (
    principal,
    action == Action::"compliance.quarterly_report_submit",
    resource is QuarterlyReport
) when {
    principal.role == "coo" &&
    resource.all_capex_approvals_sealed == true &&
    resource.merkle_root_computed == true &&
    resource.target_auditors in [
        Group::"aurelia-soc2-auditors",
        Group::"aurelia-ifrs-auditors",
        Group::"aurelia-eu-ai-act-notified-bodies"
    ]
};
