// j172 — Helios Industries AGM Cedar policy
// Bundle id: cedar-bundle-agm-helios-2027
// Deployed to: helios-industries-inc-nyse-hlos + computershare-registrar-services + carl-hagberg-inspectors-of-elections + abc-linguistic-services-geneva + ~280 broker tenants
// ADR-0243 + ADR-0244 + ADR-0245 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0254 + ADR-0263

// ============================================================
// PERMIT — IR Director: AGM session operate
// ============================================================

permit (
    principal == User::"lev.kahn@helios-industries-inc-nyse-hlos",
    action in [
        Action::"meet.agm_livestream_open",
        Action::"meet.agm_livestream_close",
        Action::"meet.agm_livestream_operate",
        Action::"meet.multi_language_caption_enable",
        Action::"meet.language_stream_activate",
        Action::"meet.q_and_a_queue_manage",
        Action::"meet.reg_fd_simultaneous_gate_arm",
        Action::"meet.reg_fd_simultaneous_gate_fire",
        Action::"governance.agm_state_transition",
        Action::"governance.vote_tally_certify_request",
        Action::"governance.proposal_open_voting",
        Action::"governance.proposal_close_voting",
        Action::"drive.sec_17a_4f_worm_seal",
        Action::"audit_chain.agm_anchor_emit",
        Action::"community.retail_question_handoff_receive",
        Action::"compliance.pack_manifest_assert",
        Action::"observability.latency_report_read"
    ],
    resource is AGMSession
) when {
    principal.role_in_tenant("helios-industries-inc-nyse-hlos") == "investor_relations_director" &&
    resource.tenant_id == "helios-industries-inc-nyse-hlos" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true &&
    context.cfo_co_sign_or_delegate_present == true
};

// ============================================================
// FORBID — Reg FD violations
// ============================================================

forbid (
    principal,
    action in [
        Action::"meet.multi_language_caption_disable_for_subset",
        Action::"meet.material_info_disclose_single_language_only",
        Action::"meet.material_info_disclose_pre_gate_fire",
        Action::"governance.tally_disclose_partial_share_class",
        Action::"governance.tally_disclose_aggregated_obscuring_founder_class"
    ],
    resource is AGMSession
) when {
    resource.reg_fd_simultaneous_disclosure_required == true
};

// ============================================================
// PERMIT — Reg FD gate fire (gate window enforced)
// ============================================================

permit (
    principal,
    action == Action::"meet.reg_fd_simultaneous_gate_fire",
    resource is RegFDGate
) when {
    resource.window_ms <= 200 &&
    resource.release_paths_all_armed == true &&
    resource.interpreter_cues_all_received_OR_auto_detector_triggered == true &&
    resource.actual_window_ms <= 200
};

forbid (
    principal,
    action == Action::"meet.reg_fd_simultaneous_gate_fire",
    resource is RegFDGate
) when {
    resource.actual_window_ms > 200
};

// ============================================================
// PERMIT — Computershare registrar + Carl Hagberg inspector dual-sign
// ============================================================

permit (
    principal in [
        ServicePrincipal::"computershare-registrar-tally-service",
        User::"karen.adebola-park@computershare-registrar-services"
    ],
    action == Action::"governance.vote_tally_certify_computershare_sign",
    resource is VoteTally
) when {
    resource.share_class in ["common_A", "common_B_founder"] &&
    resource.rolling_certification_state == "pending_computershare_sign" &&
    resource.merkle_anchor_emitted == true
};

permit (
    principal in [
        ServicePrincipal::"carl-hagberg-inspector-of-elections",
        User::"carl.hagberg@carl-hagberg-inspectors-of-elections"
    ],
    action == Action::"governance.vote_tally_certify_inspector_sign",
    resource is VoteTally
) when {
    resource.share_class in ["common_A", "common_B_founder"] &&
    resource.computershare_sign_present == true &&
    resource.merkle_anchor_emitted == true
};

forbid (
    principal,
    action == Action::"governance.vote_tally_certify_final",
    resource is VoteTally
) when {
    !(resource.computershare_sign_present == true) ||
    !(resource.carl_hagberg_sign_present == true)
};

// ============================================================
// PERMIT — Shareholder authentication (per-path)
// ============================================================

permit (
    principal,
    action == Action::"identity.agm_auth_direct_registered",
    resource is AGMSession
) when {
    context.computershare_proxyview_sso_token_valid == true &&
    context.passkey_assertion_present == true
};

permit (
    principal,
    action == Action::"identity.agm_auth_street_name",
    resource is AGMSession
) when {
    context.broker_sso_token_valid == true &&
    context.broker_control_number_valid == true &&
    context.broker_tenant in [
        "schwab",
        "fidelity",
        "vanguard",
        "interactive-brokers",
        "etrade",
        "robinhood",
        "tdameritrade",
        "merrill",
        "morgan-stanley-individual",
        "ubs-individual"
    ]
};

permit (
    principal,
    action == Action::"identity.agm_auth_institutional",
    resource is AGMSession
) when {
    context.computershare_proxyview_token_valid == true &&
    context.institutional_holdings_records_present == true
};

permit (
    principal,
    action == Action::"identity.agm_auth_proxy_advisor",
    resource is AGMSession
) when {
    context.proxy_advisor in ["iss-institutional-shareholder-services", "glass-lewis"] &&
    context.proxy_advisor_credential_valid == true
};

permit (
    principal,
    action == Action::"identity.agm_auth_press",
    resource is AGMSession
) when {
    context.press_credential_id != "" &&
    context.press_outlet in [
        "wsj", "bloomberg", "reuters", "ft", "nikkei",
        "caixin", "handelsblatt", "le-monde", "yonhap", "la-stampa",
        "korea-economic-daily", "mainichi", "globe-and-mail", "afr"
    ]
};

// ============================================================
// PERMIT — Interpreter principal (ABC LS)
// ============================================================

permit (
    principal,
    action in [
        Action::"meet.language_stream_interpret_simultaneous",
        Action::"meet.material_utterance_cue_button_tap"
    ],
    resource is LanguageStream
) when {
    principal.tenant_id == "abc-linguistic-services-geneva" &&
    context.interpreter_credential_valid == true &&
    context.interpretation_mode == "simultaneous"
};

// ============================================================
// PERMIT — Ombudsperson (community retail filter)
// ============================================================

permit (
    principal == User::"naveen.iyer-krishnamurthy@helios-industries-inc-nyse-hlos",
    action in [
        Action::"community.retail_question_review",
        Action::"community.retail_question_civility_filter",
        Action::"community.retail_question_reg_fd_filter",
        Action::"community.retail_question_promote_to_primary",
        Action::"community.retail_question_re_route_to_written"
    ],
    resource is RetailQuestion
) when {
    principal.role_in_tenant("helios-industries-inc-nyse-hlos") == "chief_compliance_officer_ombudsperson" &&
    resource.agm_session_id == "agm-helios-2027-fy2026"
};

// ============================================================
// PERMIT — GC + CFO + CEO board/executive channel
// ============================================================

permit (
    principal in [
        User::"theodore.chen-walsh@helios-industries-inc-nyse-hlos",
        User::"marguerite.vasquez-ortiz@helios-industries-inc-nyse-hlos",
        User::"lakshmi.subramanian-brodsky@helios-industries-inc-nyse-hlos",
        User::"hideki.akiyama-holt@helios-industries-inc-nyse-hlos",
        User::"lev.kahn@helios-industries-inc-nyse-hlos"
    ],
    action in [
        Action::"messenger.executive_channel_send",
        Action::"messenger.executive_channel_receive"
    ],
    resource is ExecutiveChannel
) when {
    resource.channel_class == "agm_executive_coordination" &&
    resource.agm_session_id == "agm-helios-2027-fy2026"
};

forbid (
    principal,
    action in [
        Action::"messenger.channel_enumerate",
        Action::"messenger.channel_metadata_read"
    ],
    resource is ExecutiveChannel
) when {
    resource.channel_class == "agm_executive_coordination" &&
    !(principal in resource.permitted_principals)
};

// ============================================================
// PERMIT — SEC 17a-4(f) WORM seal
// ============================================================

permit (
    principal,
    action == Action::"drive.sec_17a_4f_worm_seal",
    resource is WORMArtifact
) when {
    resource.worm_cell == "us-east-tier-1-worm-sec-17a-4f" &&
    resource.seal_class == "sec-17a-4f-helios-class-A" &&
    resource.retention_years_minimum >= 6 &&
    resource.indelible_storage_attestation == true &&
    resource.time_stamp_authority_id != "" &&
    resource.audit_trail_attached == true
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
// PERMIT — Merkle anchor emission (per share class per proposal)
// ============================================================

permit (
    principal == ServicePrincipal::"governance-service",
    action == Action::"audit_chain.agm_anchor_emit",
    resource is MerkleAnchor
) when {
    resource.share_class in ["common_A", "common_B_founder"] &&
    resource.proposal_id != "" &&
    resource.proof_class == "inclusion_proof" &&
    resource.external_transparency_log_batch != ""
};

// ============================================================
// PERMIT — Pack manifest assertion (8 packs)
// ============================================================

permit (
    principal == ServicePrincipal::"compliance-pack-asserter",
    action == Action::"compliance.pack_manifest_assert",
    resource is PackManifest
) when {
    resource.active_packs.containsAll([
        "pack-sec-reg-fd-2024",
        "pack-sec-17a-4f-worm-v3",
        "pack-nyse-listed-company-manual-2027",
        "pack-sox-302-404",
        "pack-gdpr-eu-shareholders-v4",
        "pack-eu-mar-article-17",
        "pack-delaware-gcl-shareholder-meetings",
        "pack-ias-1-fy2026"
    ]) &&
    resource.cross_validation_state == "passed"
};

// ============================================================
// FORBID — Default deny for AGM session
// ============================================================

forbid (
    principal,
    action,
    resource is AGMSession
) when {
    !(principal in [
        User::"lev.kahn@helios-industries-inc-nyse-hlos",
        User::"theodore.chen-walsh@helios-industries-inc-nyse-hlos",
        User::"marguerite.vasquez-ortiz@helios-industries-inc-nyse-hlos",
        User::"lakshmi.subramanian-brodsky@helios-industries-inc-nyse-hlos",
        User::"hideki.akiyama-holt@helios-industries-inc-nyse-hlos",
        User::"naveen.iyer-krishnamurthy@helios-industries-inc-nyse-hlos",
        User::"sarah.chen-marlowe@helios-industries-inc-nyse-hlos",
        User::"priya.iyer-bhatt@helios-industries-inc-nyse-hlos",
        User::"marcus.holloway-reid@helios-industries-inc-nyse-hlos"
    ]) &&
    !(principal in [
        ServicePrincipal::"meet-service",
        ServicePrincipal::"governance-service",
        ServicePrincipal::"drive-service",
        ServicePrincipal::"audit-chain-service",
        ServicePrincipal::"community-service",
        ServicePrincipal::"identity-service",
        ServicePrincipal::"compliance-pack-asserter",
        ServicePrincipal::"observability-emitter",
        ServicePrincipal::"computershare-registrar-tally-service",
        ServicePrincipal::"carl-hagberg-inspector-of-elections"
    ]) &&
    !(principal.tenant_id == "abc-linguistic-services-geneva" && context.interpreter_credential_valid == true) &&
    context.shareholder_auth_token == ""
};
