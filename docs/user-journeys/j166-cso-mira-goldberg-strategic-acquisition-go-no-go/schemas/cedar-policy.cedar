// j166 — Skylark + MRT acquisition Cedar policy
// Bundle id: cedar-bundle-acquisition-v1
// Deployed to: skylark-logistics-solutions-inc + cross-tenant-channel-skylark-mrt-2027-q2
// ADR-0244 + ADR-0263 + ADR-0251 + ADR-0245 + ADR-0252 + ADR-0253 + ADR-0255

// ============================================================
// PERMIT — CSO: M&A workspace open + model + ML + analysis
// ============================================================

permit (
    principal == User::"mira.goldberg@skylark-logistics-solutions-inc",
    action in [
        Action::"governance.m_and_a_workspace_open",
        Action::"financial_planning.m_a_model_compute",
        Action::"intelligence.scenario_modeling",
        Action::"compliance.pack_manifest_cross_check",
        Action::"compliance.merger_filing_requirements_compute",
        Action::"connect.cross_tenant_nda_channel_send",
        Action::"connect.cross_tenant_nda_channel_receive",
        Action::"governance.acquisition_recommendation_propose",
        Action::"drive.cross_tenant_evidence_archive"
    ],
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") == "chief_strategy_officer" &&
    resource.tenant_id == "skylark-logistics-solutions-inc" &&
    resource.target_tenant_id == "mendelsohn-routing-technologies-inc-de" &&
    context.passkey_assertion_present == true &&
    context.nda_active_at_time == true
};

// ============================================================
// PERMIT — Cross-tenant NDA channel send/receive
// ============================================================

permit (
    principal,
    action in [
        Action::"connect.cross_tenant_nda_channel_send",
        Action::"connect.cross_tenant_nda_channel_receive"
    ],
    resource is CrossTenantChannel
) when {
    resource.channel_id == "cross-tenant-channel-skylark-mrt-2027-q2" &&
    context.nda_record_id == "nda-skylark-mrt-2027-03-08" &&
    context.nda_active == true &&
    context.payload_class in [
        "diligence_request",
        "diligence_response_anonymized",
        "diligence_response_named",
        "valuation_clarification",
        "regulatory_filing_input",
        "executive_meeting_logistics"
    ] &&
    context.payload_pii_scan_zero_hits == true &&
    context.payload_e2ee_envelope_intact == true &&
    context.sender_on_tenant_authorized_signer_list == true
};

// ============================================================
// FORBID — Cross-tenant flow with PII
// ============================================================

forbid (
    principal,
    action == Action::"connect.cross_tenant_nda_channel_send",
    resource is CrossTenantChannel
) when {
    !context.payload_pii_scan_zero_hits
};

// ============================================================
// FORBID — Cross-tenant flow without active NDA
// ============================================================

forbid (
    principal,
    action == Action::"connect.cross_tenant_nda_channel_send",
    resource is CrossTenantChannel
) when {
    !context.nda_active
};

// ============================================================
// PERMIT — General Counsel: counsel review
// ============================================================

permit (
    principal == User::"daphne.harrowgate@skylark-logistics-solutions-inc",
    action in [
        Action::"governance.counsel_review_m_and_a",
        Action::"governance.counsel_redline_submit"
    ],
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") == "general_counsel" &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — CFO sign-off on financial model
// ============================================================

permit (
    principal == User::"reginald.otis@skylark-logistics-solutions-inc",
    action == Action::"governance.cfo_signoff_m_and_a",
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") == "chief_financial_officer" &&
    context.financial_model_review_complete == true &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — Committee endorsement (strategy + audit)
// ============================================================

permit (
    principal in Group::"strategy_committee_members",
    action == Action::"governance.committee_endorsement_submit",
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") in [
        "strategy_committee_chair",
        "strategy_committee_member"
    ] &&
    context.passkey_assertion_present == true
};

permit (
    principal in Group::"audit_committee_members",
    action == Action::"governance.committee_endorsement_submit",
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") in [
        "audit_committee_chair",
        "audit_committee_independent"
    ] &&
    context.passkey_assertion_present == true &&
    context.counsel_review_present == true
};

// ============================================================
// PERMIT — Board go/no-go vote
// ============================================================

permit (
    principal in Group::"skylark_board_voting_members",
    action == Action::"governance.acquisition_go_no_go_vote",
    resource is AcquisitionDecision
) when {
    context.audit_committee_endorsement_present == true &&
    context.strategy_committee_endorsement_present == true &&
    context.counsel_review_present == true &&
    context.financial_model_signoff_cfo_present == true &&
    context.ml_scenarios_provenance_preserved == true &&
    context.pack_manifest_cross_check_no_blocker == true &&
    context.merger_filings_path_clear == true &&
    context.passkey_assertion_present == true
};

// ============================================================
// FORBID — Board vote without CFO sign-off
// ============================================================

forbid (
    principal,
    action == Action::"governance.acquisition_go_no_go_vote",
    resource is AcquisitionDecision
) when {
    !context.financial_model_signoff_cfo_present
};

// ============================================================
// FORBID — Non-board member attempts vote
// ============================================================

forbid (
    principal,
    action == Action::"governance.acquisition_go_no_go_vote",
    resource is AcquisitionDecision
) when {
    !(principal in Group::"skylark_board_voting_members")
};

// ============================================================
// PERMIT — Drive WORM archive (final decision)
// ============================================================

permit (
    principal in Role::"oya.system.governance-archive",
    action in [
        Action::"drive.write_room",
        Action::"drive.engage_worm_lock"
    ],
    resource is DriveRoom
) when {
    resource.room_path == "skylark/board/2027/q2/mrt-acquisition" &&
    context.cso_signoff_present == true &&
    context.counsel_review_present == true &&
    context.cfo_signoff_present == true &&
    context.board_vote_recorded == true &&
    context.retention_years >= 7
};

// ============================================================
// FORBID — NDA-bound diligence docs in decision bundle (return-or-destroy)
// ============================================================

forbid (
    principal,
    action == Action::"drive.bundle_include",
    resource is DriveObject
) when {
    resource.cross_tenant_evidence_flag == true &&
    resource.nda_return_or_destroy_obligation_active == true
};

// ============================================================
// PERMIT — Audit chain super-Merkle anchor
// ============================================================

permit (
    principal in Role::"oya.system.audit-chain-anchor",
    action == Action::"audit-chain.m_a_decision_anchor",
    resource is AcquisitionDecision
) when {
    context.super_merkle_root_present == true &&
    context.bundle_complete == true
};

// ============================================================
// PERMIT — Intelligence µservice ML pipelines
// ============================================================

permit (
    principal in Role::"oya.system.intelligence-ml",
    action == Action::"intelligence.m_a_scenario_modeling",
    resource is AcquisitionDecision
) when {
    context.eu_ai_act_article_50_declaration_present == true &&
    context.ml_model_provenance_declared == true &&
    context.deterministic_seed_provided == true
};

// ============================================================
// FORBID — Cross-region material data transfer
// ============================================================

forbid (
    principal,
    action == Action::"compliance.cross_region_evidence_material_transfer",
    resource is EvidenceLedger
) when {
    resource.region != context.target_region &&
    !context.hash_only_transfer
};
