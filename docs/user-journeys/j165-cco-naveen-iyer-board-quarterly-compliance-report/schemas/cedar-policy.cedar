// j165 — Tessellate Health AI Q1-2027 board compliance report Cedar policy
// Bundle id: cedar-bundle-board-compliance-report-v1
// Deployed to: tessellate-health-ai-inc + audit-chain-spine-tessellate-q1-2027
// ADR-0244 + ADR-0263 + ADR-0251 + ADR-0250 + ADR-0253

// ============================================================
// PERMIT — CCO: workflow initiation + draft authoring
// ============================================================

permit (
    principal == User::"naveen.iyer@tessellate-health-ai-inc",
    action in [
        Action::"governance.board_report_workflow_initiate",
        Action::"compliance.cross_pack_evidence_pull",
        Action::"compliance.report_draft",
        Action::"intelligence.executive_summary_assist",
        Action::"governance.merkle_per_pack_compute",
        Action::"governance.merkle_super_root_compute",
        Action::"compliance.sec_8k_trigger_evaluate",
        Action::"workflow_engine.transition_propose",
        Action::"workflow_engine.transition_confirm_cco"
    ],
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "chief_compliance_officer" &&
    resource.tenant_id == "tessellate-health-ai-inc" &&
    resource.report_class == "quarterly_board_compliance" &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — General Counsel: counsel review + transition to AC
// ============================================================

permit (
    principal == User::"hampton.reese@tessellate-health-ai-inc",
    action in [
        Action::"governance.counsel_review",
        Action::"governance.counsel_redline_submit",
        Action::"workflow_engine.transition_counsel_to_audit_committee"
    ],
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "general_counsel" &&
    context.cco_signoff_present == true &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — Audit committee members: sign-off
// ============================================================

permit (
    principal in Group::"audit_committee_members",
    action == Action::"governance.audit_committee_signoff",
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") in [
        "audit_committee_chair",
        "audit_committee_independent"
    ] &&
    context.counsel_review_present == true &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — Audit committee chair: transition to board
// ============================================================

permit (
    principal == User::"jasmine.wells-okafor@tessellate-health-ai-inc",
    action == Action::"workflow_engine.transition_audit_committee_to_board",
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "audit_committee_chair" &&
    context.counsel_review_present == true &&
    context.audit_committee_quorum_reached == true &&
    context.audit_committee_quorum_count >= 3 &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — Drive WORM archive (final report)
// ============================================================

permit (
    principal in Role::"oya.system.governance-archive",
    action in [
        Action::"drive.write_room",
        Action::"drive.engage_worm_lock"
    ],
    resource is DriveRoom
) when {
    resource.room_path == "tessellate/board/2027/q1/compliance-report" &&
    context.cco_signoff_present == true &&
    context.counsel_review_present == true &&
    context.audit_committee_quorum_reached == true &&
    context.retention_years >= 7 &&
    context.retention_authority startsWith "SEC-pre-IPO-adapted"
};

// ============================================================
// PERMIT — External transparency log anchor
// ============================================================

permit (
    principal in Role::"oya.system.audit-chain-anchor",
    action == Action::"audit-chain.external_transparency_log_anchor",
    resource is BoardComplianceReport
) when {
    context.super_merkle_root_present == true &&
    context.drive_archive_complete == true
};

// ============================================================
// PERMIT — Pre-read distribution to board members
// ============================================================

permit (
    principal == User::"naveen.iyer@tessellate-health-ai-inc",
    action == Action::"drive.distribute_pre_read",
    resource is DriveRoom
) when {
    resource.room_path == "tessellate/board/2027/q1/compliance-report" &&
    context.report_in_state == "board_presentation" &&
    context.audit_committee_signoff_complete == true
};

// ============================================================
// FORBID — Non-CCO attempts workflow initiation
// ============================================================

forbid (
    principal,
    action == Action::"governance.board_report_workflow_initiate",
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant(resource.tenant_id) != "chief_compliance_officer"
};

// ============================================================
// FORBID — AC sign-off without counsel review
// ============================================================

forbid (
    principal,
    action == Action::"governance.audit_committee_signoff",
    resource is BoardComplianceReport
) when {
    !context.counsel_review_present
};

// ============================================================
// FORBID — Board transition without AC quorum
// ============================================================

forbid (
    principal,
    action == Action::"workflow_engine.transition_audit_committee_to_board",
    resource is BoardComplianceReport
) when {
    !context.audit_committee_quorum_reached ||
    context.audit_committee_quorum_count < 3
};

// ============================================================
// FORBID — Cross-region evidence material transfer (only hashes allowed)
// ============================================================

forbid (
    principal,
    action == Action::"compliance.evidence_material_export_cross_region",
    resource is EvidenceLedger
) when {
    resource.region != context.target_region &&
    !context.hash_only_transfer
};

// ============================================================
// FORBID — Modification of archived report after WORM lock
// ============================================================

forbid (
    principal,
    action in [
        Action::"drive.modify",
        Action::"drive.delete"
    ],
    resource is DriveObject
) when {
    resource.worm_lock_engaged == true &&
    resource.worm_until > context.now
};

// ============================================================
// PERMIT — LLM-assisted exec summary draft (intelligence µservice)
// ============================================================

permit (
    principal in Role::"oya.system.intelligence-llm",
    action == Action::"intelligence.llm_executive_summary_assist",
    resource is BoardComplianceReport
) when {
    context.model_identity_declared == true &&
    context.eu_ai_act_article_50_declaration_present == true &&
    context.human_finalization_required == true &&
    context.cco_signoff_required == true
};

// ============================================================
// PERMIT — SEC trigger evaluator (compliance µservice)
// ============================================================

permit (
    principal in Role::"oya.system.compliance-sec-evaluator",
    action == Action::"compliance.sec_8k_trigger_evaluate",
    resource is BoardComplianceReport
) when {
    resource.tenant_id == "tessellate-health-ai-inc" &&
    context.evaluation_mode in ["pre_ipo_shadow", "post_ipo_enforcement"]
};
