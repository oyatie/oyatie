// j173 — Multi-jurisdictional trust restructure Cedar policy
// Bundle id: cedar-bundle-amht-2027
// Deployed to: halberd-mercer-private-wealth-difc + mishcon-de-reya-london + maples-group-grand-cayman + allen-gledhill-singapore + 4 banks + 7 family principals
// ADR-0243 + ADR-0244 + ADR-0245 + ADR-0247 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0263

// ============================================================
// PERMIT — Wealth manager: trust restructure operate
// ============================================================

permit (
    principal == User::"aamir.khan@halberd-mercer-private-wealth-difc",
    action in [
        Action::"wealth_mgmt.cockpit_open",
        Action::"clm.trust_workflow_open",
        Action::"clm.trust_document_draft",
        Action::"clm.trust_document_send_for_review",
        Action::"clm.family_review_request",
        Action::"clm.engagement_state_transition",
        Action::"compliance.pack_manifest_activate_cross_jurisdiction",
        Action::"compliance.pack_manifest_assert",
        Action::"compliance.crs_reissuance_initiate",
        Action::"compliance.fatca_form_reissuance_initiate",
        Action::"compliance.aml_screen_request",
        Action::"compliance.dtaa_attestation_emit",
        Action::"payments.consolidation_transfer_initiate",
        Action::"payments.sanctions_screen_request",
        Action::"audit_chain.tax_position_anchor_emit",
        Action::"drive.jurisdiction_worm_write",
        Action::"intelligence.tax_scenario_compute",
        Action::"messenger.step_privileged_channel_send",
        Action::"messenger.step_privileged_channel_receive",
        Action::"notes.step_privileged_note_write"
    ],
    resource is TrustRestructureEngagement
) when {
    principal.role_in_tenant("halberd-mercer-private-wealth-difc") == "senior_director_multi_family_office" &&
    principal.dfsa_smcr_attestation_id != "" &&
    principal.step_tep_attestation_id != "" &&
    resource.client_family_id == "client-family-al-maktoum-hartington-tan" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true &&
    context.client_engagement_letter_active == true
};

// ============================================================
// PERMIT — Counsel firms (per-jurisdiction bar attestation)
// ============================================================

permit (
    principal,
    action in [
        Action::"clm.trust_document_counsel_review",
        Action::"clm.trust_document_redline_submit",
        Action::"clm.trust_document_cross_counsel_comment",
        Action::"messenger.step_privileged_channel_send",
        Action::"messenger.step_privileged_channel_receive"
    ],
    resource is TrustDocument
) when {
    principal.tenant_id in [
        "mishcon-de-reya-london",
        "maples-group-grand-cayman",
        "allen-gledhill-singapore",
        "bin-suwaidan-co-difc"
    ] &&
    principal.bar_attestation_jurisdiction in resource.applicable_jurisdictions &&
    context.step_privileged_class == true
};

// ============================================================
// PERMIT — Family principal sign (passkey + KYC + MiFID II for UK)
// ============================================================

permit (
    principal,
    action == Action::"clm.trust_document_sign",
    resource is TrustDocument
) when {
    principal.tenant_class == "personal" &&
    principal in resource.family_signing_parties &&
    context.passkey_assertion_present == true &&
    context.kyc_attestation_id != "" &&
    context.kyc_attestation_jurisdiction_valid_for_principal == true &&
    (principal.domicile_class != "uk_domicile" || context.mifid_ii_suitability_acknowledgment_id != "")
};

forbid (
    principal,
    action == Action::"clm.trust_document_sign",
    resource is TrustDocument
) when {
    context.kyc_attestation_id == ""
};

// ============================================================
// PERMIT — STEP-privileged advisory channel (tetrad)
// ============================================================

permit (
    principal,
    action in [
        Action::"messenger.step_privileged_channel_send",
        Action::"messenger.step_privileged_channel_receive"
    ],
    resource is STEPPrivilegedChannel
) when {
    resource.channel_class == "step_privileged_advisory_tetrad" &&
    resource.member_count == 4 &&
    principal in resource.permitted_principals &&
    context.payload_class in [
        "counsel_clarification",
        "tax_position_drafting",
        "cross_reference_reconciliation",
        "family_principal_proxy_question",
        "hmrc_clearance_drafting"
    ] &&
    context.mls_e2ee_envelope_intact == true
};

forbid (
    principal,
    action in [
        Action::"messenger.channel_enumerate",
        Action::"messenger.channel_metadata_read"
    ],
    resource is STEPPrivilegedChannel
) when {
    !(principal in resource.permitted_principals) &&
    context.tax_authority_compulsion_order_id == ""
};

// ============================================================
// PERMIT — Bank principals + SWIFT MT103 dispatch
// ============================================================

permit (
    principal,
    action in [
        Action::"payments.swift_mt103_dispatch",
        Action::"payments.swift_mt202_cover_send"
    ],
    resource is SwiftMessage
) when {
    principal.tenant_class == "bank" &&
    resource.sender_bic == principal.bank_bic &&
    context.sanctions_screen_clean == true &&
    context.aml_screen_pass == true
};

forbid (
    principal,
    action == Action::"payments.consolidation_transfer_initiate",
    resource is ConsolidationTransfer
) when {
    !(context.sanctions_screen_clean == true) ||
    !(context.aml_screen_pass == true) ||
    !(resource.all_family_signatures_complete == true)
};

// ============================================================
// PERMIT — HMRC CGT clearance
// ============================================================

permit (
    principal,
    action in [
        Action::"hmrc.cgt_clearance_apply",
        Action::"hmrc.cgt_clearance_grant"
    ],
    resource is HMRCClearanceApplication
) when {
    (principal.tenant_id == "mishcon-de-reya-london" && action == Action::"hmrc.cgt_clearance_apply") ||
    (principal.tenant_id == "hmrc-customer-compliance" && action == Action::"hmrc.cgt_clearance_grant")
};

// ============================================================
// PERMIT — Per-document Merkle anchor emission
// ============================================================

permit (
    principal == ServicePrincipal::"audit-chain-service",
    action == Action::"audit_chain.tax_position_anchor_emit",
    resource is MerkleAnchor
) when {
    resource.proof_class == "inclusion_proof_only_without_payload" &&
    resource.external_transparency_log_batch != "" &&
    resource.compulsion_paths.size() >= 1
};

// ============================================================
// PERMIT — Tax-authority compulsion (proof only)
// ============================================================

permit (
    principal,
    action == Action::"audit_chain.tax_position_inclusion_proof_request",
    resource is MerkleAnchor
) when {
    context.tax_authority_compulsion_order_id != "" &&
    context.tax_authority_class in [
        "HMRC",
        "IRS_FATCA",
        "OECD_CRS_competent_authority",
        "DFSA_DIFC",
        "MAS_Singapore",
        "CIMA_Cayman"
    ] &&
    resource.privilege_class == "step_privileged_substantiation"
    // proof only — payload disclosure requires separate court order + counsel review
};

forbid (
    principal,
    action == Action::"audit_chain.tax_position_payload_disclose",
    resource is MerkleAnchor
) when {
    resource.privilege_class == "step_privileged_substantiation" &&
    (context.specific_court_order_compelling_payload_disclosure == false ||
     context.counsel_review_present == false)
};

// ============================================================
// PERMIT — Jurisdiction-aware WORM write
// ============================================================

permit (
    principal,
    action == Action::"drive.jurisdiction_worm_write",
    resource is JurisdictionWORMRoom
) when {
    resource.jurisdiction in ["UK", "UAE", "SG", "KY"] &&
    resource.worm_cell in [
        "eu-london-tier-1-worm-trust-retention",
        "me-dubai-tier-1-worm-trust-retention",
        "apac-singapore-tier-1-worm-trust-retention",
        "kyc-grand-cayman-tier-2-worm-trust-retention"
    ] &&
    resource.retention_years_minimum >= 6 &&
    resource.indelible_storage_attestation == true
};

forbid (
    principal,
    action in [
        Action::"drive.worm_artifact_update",
        Action::"drive.worm_artifact_delete",
        Action::"drive.worm_artifact_overwrite"
    ],
    resource is WORMArtifact
) when {
    resource.seal_state == "sealed"
};

// ============================================================
// PERMIT — Pack manifest assertion (8 packs cross-validated)
// ============================================================

permit (
    principal == ServicePrincipal::"compliance-pack-asserter",
    action == Action::"compliance.pack_manifest_assert",
    resource is PackManifest
) when {
    resource.active_packs.containsAll([
        "pack-oecd-crs-2025",
        "pack-fatca-form-w8-ben-e-v3",
        "pack-mifid-ii-suitability-2027",
        "pack-difc-trust-law-5-2018",
        "pack-uk-trustee-act-2000",
        "pack-sg-trust-companies-act-2005",
        "pack-cayman-trust-law-star",
        "pack-aml-5-cross-jurisdiction"
    ]) &&
    resource.cross_validation_state == "passed"
};

// ============================================================
// FORBID — Default deny on TrustRestructureEngagement
// ============================================================

forbid (
    principal,
    action,
    resource is TrustRestructureEngagement
) when {
    !(principal == User::"aamir.khan@halberd-mercer-private-wealth-difc") &&
    !(principal.tenant_id in [
        "halberd-mercer-private-wealth-difc",
        "mishcon-de-reya-london",
        "maples-group-grand-cayman",
        "allen-gledhill-singapore",
        "bin-suwaidan-co-difc"
    ]) &&
    !(principal in resource.family_signing_parties) &&
    !(principal in [
        ServicePrincipal::"clm-service",
        ServicePrincipal::"compliance-pack-asserter",
        ServicePrincipal::"payments-service",
        ServicePrincipal::"audit-chain-service",
        ServicePrincipal::"drive-service",
        ServicePrincipal::"intelligence-service",
        ServicePrincipal::"observability-emitter",
        ServicePrincipal::"hmrc-clearance-bridge"
    ]) &&
    context.tax_authority_compulsion_order_id == ""
};
