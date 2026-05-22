// j163 — Hartwell-Renshaw board meeting Cedar policy
// Bundle id: cedar-bundle-board-meeting-v1
// Deployed to: hartwell-renshaw-asset-mgmt-llc + meet-cross-region-evidence-spine
// ADR-0244 tenant scoping; ADR-0263 audit dual-seal; ADR-0248 cellular architecture
// ADR-0251 compliance-pack primitive; ADR-0255 intelligence two-layer

// ============================================================
// PERMIT — AV coordinator: pre-flight + recording envelope control
// ============================================================

permit (
    principal == User::"jordan.park@hartwell-renshaw-asset-mgmt-llc",
    action in [
        Action::"meet.pre_flight_initiate",
        Action::"meet.pre_flight_validate",
        Action::"meet.pre_flight_close",
        Action::"recordings.start_envelope",
        Action::"recordings.close_envelope",
        Action::"recordings.redact_segment",
        Action::"meet.breakout_create",
        Action::"meet.executive_session_engage",
        Action::"meet.executive_session_release",
        Action::"meet.consent_matrix_close",
        Action::"cell.warm_batch"
    ],
    resource is BoardMeeting
) when {
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") == "av_coordinator" &&
    resource.tenant_id == "hartwell-renshaw-asset-mgmt-llc" &&
    resource.meeting_class == "board_of_directors" &&
    context.passkey_assertion_present == true
};

// ============================================================
// FORBID — AV coordinator cannot listen during executive session
// ============================================================

forbid (
    principal,
    action == Action::"meet.audio_path_subscribe",
    resource is BoardMeeting
) when {
    principal.role_in_tenant(resource.tenant_id) == "av_coordinator" &&
    resource.executive_session_engaged == true
};

forbid (
    principal,
    action == Action::"intelligence.asr_subscribe",
    resource is BoardMeeting
) when {
    resource.executive_session_engaged == true &&
    principal.role_in_tenant(resource.tenant_id) in ["av_coordinator", "executive_assistant"]
};

// ============================================================
// PERMIT — Chair: executive session engagement + release + minutes signature
// ============================================================

permit (
    principal == User::"margaret.hartwell-renshaw@hartwell-renshaw-asset-mgmt-llc",
    action in [
        Action::"meet.executive_session_authorize",
        Action::"meet.executive_session_release",
        Action::"governance.minutes_sign",
        Action::"governance.minutes_archive_final"
    ],
    resource is BoardMeeting
) when {
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") == "chair" &&
    resource.tenant_id == "hartwell-renshaw-asset-mgmt-llc" &&
    context.passkey_assertion_present == true &&
    context.fido2_hardware_key_assertion_present == true
};

// ============================================================
// PERMIT — Board voting members in executive session
// ============================================================

permit (
    principal in Group::"board_voting_members",
    action == Action::"meet.executive_session_participant",
    resource is BoardMeeting
) when {
    resource.executive_session_engaged == true &&
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") in [
        "chair", "ceo", "coo", "general_counsel",
        "regional_vp", "regional_controller", "independent_director"
    ]
};

// ============================================================
// PERMIT — General Counsel: counsel review + Article 50 declaration owner
// ============================================================

permit (
    principal == User::"friedrich.holstein@hartwell-renshaw-asset-mgmt-llc",
    action in [
        Action::"governance.counsel_review",
        Action::"governance.eu_ai_act_article_50_publish",
        Action::"compliance.gdpr_dpo_validate"
    ],
    resource is BoardMeeting
) when {
    principal.role_in_tenant("hartwell-renshaw-asset-mgmt-llc") == "general_counsel" &&
    resource.tenant_id == "hartwell-renshaw-asset-mgmt-llc"
};

// ============================================================
// PERMIT — Recording envelope open: gated by consent matrix + Article 50
// ============================================================

permit (
    principal == User::"jordan.park@hartwell-renshaw-asset-mgmt-llc",
    action == Action::"recordings.start_envelope",
    resource is BoardMeeting
) when {
    context.consent_matrix_complete == true &&
    context.consent_matrix_jurisdiction_count >= 7 &&
    context.eu_ai_act_article_50_declaration_acknowledged_by_all_eu_participants == true &&
    context.declared_models contains "whisper-large-v3@openai-mit-fork-2025-08" &&
    context.declared_models contains "nllb-200-distilled-1.3B@meta-cc-by-nc-4.0-2024-11" &&
    context.kms_primary_root_engaged == true &&
    context.kms_mirror_root_engaged == true &&
    context.worm_retention_years >= 7
};

// ============================================================
// PERMIT — Recording segment redaction (post-meeting, requires chair + GC)
// ============================================================

permit (
    principal == User::"jordan.park@hartwell-renshaw-asset-mgmt-llc",
    action == Action::"recordings.redact_segment",
    resource is RecordingSegment
) when {
    context.chair_authorization_present == true &&
    context.general_counsel_authorization_present == true &&
    context.gdpr_article_17_request_id != "" &&
    context.sec_17a_4f_override_review_complete == true
};

// ============================================================
// PERMIT — Exec session segment unlock: chair + GC + board-vote-of-3
// ============================================================

permit (
    principal,
    action == Action::"recordings.unlock_exec_session_segment",
    resource is RecordingSegment
) when {
    resource.segment_class == "executive_session_segment" &&
    context.chair_authorization_present == true &&
    context.general_counsel_authorization_present == true &&
    context.board_vote_count >= 3 &&
    context.board_vote_quorum_reached == true
};

// ============================================================
// FORBID — External party access to exec session segment without unlock
// ============================================================

forbid (
    principal,
    action == Action::"recordings.read_segment",
    resource is RecordingSegment
) when {
    resource.segment_class == "executive_session_segment" &&
    !context.exec_session_unlock_evidence_present
};

// ============================================================
// FORBID — Non-operator attempts envelope control
// ============================================================

forbid (
    principal,
    action in [
        Action::"recordings.close_envelope",
        Action::"recordings.start_envelope",
        Action::"meet.executive_session_engage"
    ],
    resource is BoardMeeting
) when {
    principal.role_in_tenant(resource.tenant_id) != "av_coordinator"
};

// ============================================================
// PERMIT — Compliance evaluator: consent matrix collection
// ============================================================

permit (
    principal in Role::"oya.system.compliance-evaluator",
    action in [
        Action::"compliance.consent_sec_17a_4f",
        Action::"compliance.consent_gdpr",
        Action::"compliance.consent_kr_pipa_cross_border",
        Action::"compliance.consent_jp_appi_cross_border",
        Action::"compliance.consent_lgpd",
        Action::"compliance.consent_pdpa_sg",
        Action::"compliance.consent_ndpa_ng",
        Action::"compliance.transparency_eu_ai_act_article_50"
    ],
    resource is BoardMeeting
) when {
    context.system_initiated == true &&
    resource.tenant_id == "hartwell-renshaw-asset-mgmt-llc"
};

// ============================================================
// PERMIT — Drive WORM archive
// ============================================================

permit (
    principal == User::"theresa.holloway@hartwell-renshaw-asset-mgmt-llc",
    action in [
        Action::"drive.write_room",
        Action::"drive.engage_worm_lock"
    ],
    resource is DriveRoom
) when {
    resource.room_path == "hartwell-renshaw/board/2027/q1" &&
    context.signed_by_chair == true &&
    context.counsel_review_present == true &&
    context.retention_years >= 7
};

// ============================================================
// PERMIT — Governance Merkle anchor
// ============================================================

permit (
    principal in Role::"oya.system.governance-anchor",
    action in [
        Action::"governance.merkle_compute",
        Action::"governance.merkle_anchor_audit_chain_spine",
        Action::"governance.merkle_anchor_external_transparency_log"
    ],
    resource is BoardMeetingBundle
) when {
    context.bundle_components_count >= 6 &&
    context.bundle_includes_minutes == true &&
    context.bundle_includes_recording_metadata == true &&
    context.bundle_includes_consent_matrix == true &&
    context.bundle_includes_eu_ai_act_50_declaration == true &&
    context.bundle_includes_exec_session_resolution_log == true &&
    context.bundle_includes_cross_border_evidence == true
};

// ============================================================
// FORBID — Cross-region key migration
// ============================================================

forbid (
    principal,
    action == Action::"kms.export_key_material",
    resource is KMSKey
) when {
    resource.key_class in ["board_recording_primary", "board_recording_mirror"]
};

// ============================================================
// PERMIT — Caption fallback declaration
// ============================================================

permit (
    principal in Role::"oya.system.caption-quality-monitor",
    action == Action::"intelligence.caption_fallback_engage",
    resource is BoardMeeting
) when {
    context.drift_score_sustained_60s > 0.08 &&
    context.fallback_language == "en-US"
};
