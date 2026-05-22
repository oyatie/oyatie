// j171 — Halberd-Mercer ombudsperson-privileged cross-tenant mediation Cedar policy
// Bundle id: cedar-bundle-ombuds-Δ47
// Deployed to: halberd-mercer-holdings-corporate-sg + halberd-mercer-property-sg + priscilla-lim-personal-2018 (cross-tenant boundary)
// ADR-0243 + ADR-0244 + ADR-0245 + ADR-0246 + ADR-0251 + ADR-0263 + ADR-0253 + ADR-0252

// ============================================================
// PERMIT — Ombudsperson: privileged channel + WORM + audit-chain + community appeal handoff + escalation gate
// ============================================================

permit (
    principal == User::"felix.tan@halberd-mercer-holdings-corporate-sg",
    action in [
        Action::"messenger.privileged_channel_open",
        Action::"messenger.privileged_channel_send",
        Action::"messenger.privileged_channel_receive",
        Action::"drive.privileged_worm_create",
        Action::"drive.privileged_worm_write",
        Action::"drive.privileged_worm_read",
        Action::"audit_chain.privileged_anchor_emit",
        Action::"audit_chain.cedar_deny_coverage_report_read",
        Action::"community.moderator_appeal_handoff_receive",
        Action::"community.moderator_appeal_close_handoff_send",
        Action::"governance.escalation_gate_evaluate",
        Action::"governance.ombuds_outcome_record",
        Action::"notes.privileged_note_write",
        Action::"compliance.pack_manifest_assert",
        Action::"observability.redaction_attestation_emit",
        Action::"hr_transfer_bridge.transfer_activate"
    ],
    resource is OmbudsCase
) when {
    principal.role_in_tenant("halberd-mercer-holdings-corporate-sg") == "ombudsperson_certified_ioa" &&
    principal.title_attestation_id == "ioa-oco-2022-felix-tan-cert-9412" &&
    resource.privilege_class == "ombudsperson_privileged" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true
};

// ============================================================
// PERMIT — Complainant: cross-tenant intake from personal tenant + dyad messaging + WORM write
// ============================================================

permit (
    principal == User::"priscilla.lim@priscilla-lim-personal-2018",
    action in [
        Action::"community.appeal_initiate",
        Action::"community.appeal_target_select_ombudsperson_handoff",
        Action::"messenger.privileged_channel_send",
        Action::"messenger.privileged_channel_receive",
        Action::"drive.privileged_worm_write",
        Action::"drive.privileged_worm_read",
        Action::"complainant.mediation_outcome_confirm"
    ],
    resource is OmbudsCase
) when {
    principal.tenant_class == "personal" &&
    resource.complainant_principal == "priscilla.lim@priscilla-lim-personal-2018" &&
    resource.complainant_employer_record == "HMP-SG-2017-3082" &&
    context.passkey_assertion_present == true &&
    context.face_attestation_present == true
};

// ============================================================
// PERMIT — Privileged dyad channel send/receive (payload class allowlist)
// ============================================================

permit (
    principal,
    action in [
        Action::"messenger.privileged_channel_send",
        Action::"messenger.privileged_channel_receive"
    ],
    resource is PrivilegedChannel
) when {
    resource.channel_class == "ombudsperson_privileged_dyad" &&
    resource.member_count == 2 &&
    principal in resource.permitted_principals &&
    context.payload_class in [
        "ombudsperson_clarification_question",
        "ombudsperson_mediation_option",
        "ombudsperson_resolution_proposal",
        "complainant_narrative",
        "complainant_evidence_attachment",
        "complainant_decision_intent",
        "system_meta_message"
    ] &&
    context.mls_e2ee_envelope_intact == true &&
    context.mls_epoch_valid == true
};

// ============================================================
// FORBID — Channel enumeration by non-permitted principals
// ============================================================

forbid (
    principal,
    action in [
        Action::"messenger.channel_enumerate",
        Action::"messenger.channel_metadata_read",
        Action::"messenger.channel_member_list_read"
    ],
    resource is PrivilegedChannel
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    !(principal in resource.permitted_principals) &&
    context.regulator_compulsion_order_id == ""
};

// ============================================================
// FORBID — Cross-tenant principal leakage (employer HR cannot see personal-tenant principal)
// ============================================================

forbid (
    principal,
    action in [
        Action::"identity.cross_tenant_principal_map_read",
        Action::"ombuds_intake.complainant_employer_record_resolve"
    ],
    resource is CrossTenantPrincipalMapping
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    !(principal.role_in_tenant("halberd-mercer-holdings-corporate-sg") == "ombudsperson_certified_ioa")
};

// ============================================================
// PERMIT — WORM evidence write/read (dyad principals only)
// ============================================================

permit (
    principal,
    action in [
        Action::"drive.privileged_worm_write",
        Action::"drive.privileged_worm_read"
    ],
    resource is PrivilegedWORMRoom
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    principal in resource.write_principals &&
    resource.cell_primary == "eu-frankfurt-tier-1-privileged-worm" &&
    context.e2ee_at_rest_algo == "chacha20-poly1305" &&
    context.privileged_content_tag == true
};

// ============================================================
// FORBID — WORM evidence mutation post-seal
// ============================================================

forbid (
    principal,
    action in [
        Action::"drive.privileged_worm_update",
        Action::"drive.privileged_worm_delete",
        Action::"drive.privileged_worm_overwrite"
    ],
    resource is PrivilegedWORMItem
) when {
    resource.seal_state == "sealed"
};

// ============================================================
// PERMIT — Merkle privileged anchor emission
// ============================================================

permit (
    principal == ServicePrincipal::"drive-service",
    action == Action::"audit_chain.privileged_anchor_emit",
    resource is MerkleAnchor
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    resource.privileged_content_tag == "ombudsperson_privileged_no_payload_disclosure" &&
    resource.proof_class == "inclusion_proof_only_without_payload"
};

// ============================================================
// PERMIT — Regulator compulsion path (inclusion proof only)
// ============================================================

permit (
    principal,
    action == Action::"audit_chain.privileged_anchor_inclusion_proof_request",
    resource is MerkleAnchor
) when {
    context.regulator_compulsion_order_id != "" &&
    context.regulator_class in [
        "EU_WD_Article_22",
        "SOX_806_subpoena",
        "KR_ACRC_Article_13_demand",
        "SG_Court_Order"
    ] &&
    resource.privilege_class == "ombudsperson_privileged" &&
    context.corporate_counsel_review_present == true
    // NOTE: This permit returns inclusion-proof ONLY; payload disclosure requires separate court order + Cedar permit
};

// ============================================================
// FORBID — Payload disclosure without specific court order
// ============================================================

forbid (
    principal,
    action == Action::"audit_chain.privileged_anchor_payload_disclose",
    resource is MerkleAnchor
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    (context.specific_court_order_compelling_payload_disclosure == false ||
     context.complainant_explicit_consent == false)
};

// ============================================================
// PERMIT — Mandatory-reporter exception (privilege pierce, narrow)
// ============================================================

permit (
    principal == User::"felix.tan@halberd-mercer-holdings-corporate-sg",
    action == Action::"ombuds_intake.mandatory_reporter_exception_invoke",
    resource is OmbudsCase
) when {
    (context.child_safety_flag == true ||
     context.criminal_threat_flag == true ||
     context.imminent_harm_flag == true) &&
    context.secondary_ombudsperson_concurrence_present == true &&
    context.privilege_scope_narrowed_to_necessary_information == true
};

// ============================================================
// PERMIT — Confidential executive channel (ombuds recommendation transmission)
// ============================================================

permit (
    principal == User::"felix.tan@halberd-mercer-holdings-corporate-sg",
    action == Action::"messenger.confidential_executive_channel_send",
    resource is ConfidentialExecutiveChannel
) when {
    resource.recipients == [
        "adrian.cheng-whitford@halberd-mercer-holdings-corporate-sg",
        "sarojini.iyer-krishnan@halberd-mercer-holdings-corporate-sg"
    ] &&
    resource.payload_class == "ombudsperson_recommendation_no_identity" &&
    resource.complainant_identity_redacted == true &&
    resource.evidence_pointers_included == false
};

// ============================================================
// PERMIT — Supervised apology draft channel
// ============================================================

permit (
    principal == User::"aloysius.goh@halberd-mercer-property-sg",
    action == Action::"messenger.supervised_apology_draft_send",
    resource is SupervisedApologyChannel
) when {
    resource.supervisors == [
        "adrian.cheng-whitford@halberd-mercer-holdings-corporate-sg",
        "felix.tan@halberd-mercer-holdings-corporate-sg"
    ] &&
    context.both_supervisors_approval_present == true
};

// ============================================================
// PERMIT — Community moderator appeal review (no ombuds substance visibility)
// ============================================================

permit (
    principal,
    action == Action::"community.moderator_appeal_review",
    resource is CommunityAppeal
) when {
    principal.role_in_channel("channel-womenintech-halberd-property-sg") == "moderator" &&
    resource.appeal_path_community_moderator_review == true &&
    context.ombuds_handoff_content_NOT_visible == true
};

// ============================================================
// FORBID — Community moderator from ombuds handoff content
// ============================================================

forbid (
    principal,
    action in [
        Action::"community.ombuds_handoff_content_read",
        Action::"community.ombuds_handoff_payload_read"
    ],
    resource is CommunityAppeal
) when {
    !(principal.role_in_tenant("halberd-mercer-holdings-corporate-sg") == "ombudsperson_certified_ioa")
};

// ============================================================
// PERMIT — Anti-retaliation monitoring (24 months)
// ============================================================

permit (
    principal == ServicePrincipal::"anti-retaliation-monitor",
    action == Action::"hr_transfer_bridge.anti_retaliation_monitor",
    resource is TransferRecord
) when {
    resource.transfer_authority_path == "ombudsperson_mediated_outcome_Δ47" &&
    resource.anti_retaliation_monitoring_active == true &&
    context.elapsed_months_since_transfer <= 24
};

// ============================================================
// PERMIT — Observability redaction emission (ADR-0263)
// ============================================================

permit (
    principal == ServicePrincipal::"observability-emitter",
    action == Action::"observability.metric_emit",
    resource is MetricEmission
) when {
    resource.metric_class != "payload_content" &&
    context.redaction_rule_version == "adr-0263-redaction-rule-v3" &&
    context.payload_class_in_metric == false
};

// ============================================================
// FORBID — Payload class in observability metrics
// ============================================================

forbid (
    principal,
    action == Action::"observability.metric_emit",
    resource is MetricEmission
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    context.payload_class_in_metric == true
};

// ============================================================
// PERMIT — Pack manifest assertion (6 packs active)
// ============================================================

permit (
    principal == ServicePrincipal::"compliance-pack-asserter",
    action == Action::"compliance.pack_manifest_assert",
    resource is PackManifest
) when {
    resource.active_packs.containsAll([
        "pack-eu-wd-2019-1937-v2",
        "pack-sox-806-anti-retaliation-v3",
        "pack-kr-acrc-art-13",
        "pack-eeo-title-vii-2027",
        "pack-gdpr-article-9-special-category",
        "pack-ombudsperson-privilege-ioa-v2"
    ]) &&
    resource.cross_validation_state == "passed"
};

// ============================================================
// FORBID — All other access to ombudsperson-privileged resources
// ============================================================

forbid (
    principal,
    action,
    resource is OmbudsCase
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    !(principal == User::"felix.tan@halberd-mercer-holdings-corporate-sg") &&
    !(principal == User::"priscilla.lim@priscilla-lim-personal-2018") &&
    !(principal in [
        ServicePrincipal::"drive-service",
        ServicePrincipal::"audit-chain-service",
        ServicePrincipal::"compliance-pack-asserter",
        ServicePrincipal::"observability-emitter",
        ServicePrincipal::"anti-retaliation-monitor",
        ServicePrincipal::"ombuds-bridge-service"
    ]) &&
    context.regulator_compulsion_order_id == ""
};
