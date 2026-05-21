// j174 — BHI Treasury EOD Cedar policy
// Bundle id: cedar-bundle-treasury-eod-bhi-2027
// Deployed to: bohlin-hjelmqvist-industries-ab-parent + 7 subsidiary + 8 bank tenants
// ADR-0243 + ADR-0244 + ADR-0248 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0254 + ADR-0263

// ============================================================
// PERMIT — Treasury operations manager: EOD session operate
// ============================================================

permit (
    principal == User::"sven.eriksson@bohlin-hjelmqvist-industries-ab-parent",
    action in [
        Action::"treasury.cockpit_open",
        Action::"treasury.cash_position_compute",
        Action::"treasury.fx_delta_hedge_compute",
        Action::"treasury.intercompany_netting_compute",
        Action::"treasury.cash_sweep_prepare",
        Action::"treasury.cash_sweep_execute",
        Action::"treasury.eod_post",
        Action::"payments.swift_mt940_ingest_request",
        Action::"payments.swift_mt942_intraday_ingest_request",
        Action::"finops_portal.lcr_compute",
        Action::"finops_portal.overnight_position_dashboard_read",
        Action::"audit_chain.eod_account_anchor_emit",
        Action::"observability.eod_latency_slo_read"
    ],
    resource is EODSession
) when {
    principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") == "treasury_operations_manager" &&
    principal.ctp_attestation_id != "" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true &&
    context.cut_off_window_open == true
};

// ============================================================
// PERMIT — Group Treasurer (Annika): co-sign + escalation
// ============================================================

permit (
    principal == User::"annika.lindqvist-holmberg@bohlin-hjelmqvist-industries-ab-parent",
    action in [
        Action::"treasury.cash_sweep_co_sign",
        Action::"treasury.fx_hedge_rebalance_authorize",
        Action::"messenger.executive_escalation_send",
        Action::"finops_portal.overnight_position_dashboard_read",
        Action::"audit_chain.eod_summary_read"
    ],
    resource is EODSession
) when {
    principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") == "group_treasurer" &&
    principal.act_fia_attestation_id != "" &&
    context.passkey_assertion_present == true
};

// ============================================================
// PERMIT — Cash sweep with co-sign requirement
// ============================================================

permit (
    principal,
    action == Action::"treasury.cash_sweep_execute",
    resource is CashSweep
) when {
    resource.target_overnight_amount_usd <= 500000000 &&  // $500M ceiling
    context.group_treasurer_co_sign_present == true &&
    context.sanctions_screening_on_destination_counterparty_clean == true &&
    context.eod_session_in_us_cut_or_later == true
};

forbid (
    principal,
    action == Action::"treasury.cash_sweep_execute",
    resource is CashSweep
) when {
    !(context.group_treasurer_co_sign_present == true) ||
    !(context.sanctions_screening_on_destination_counterparty_clean == true)
};

// ============================================================
// PERMIT — Bank principals: MT940/MT942 dispatch
// ============================================================

permit (
    principal,
    action in [
        Action::"payments.swift_mt940_dispatch",
        Action::"payments.swift_mt942_dispatch"
    ],
    resource is SwiftStatementMessage
) when {
    principal.tenant_class == "bank" &&
    resource.sender_bic == principal.bank_bic &&
    resource.receiver_bic == "BOHLSESS" &&  // BHI parent BIC
    principal.bank_tenant_id in [
        "nordea-bank-ab",
        "svenska-handelsbanken-ab",
        "jpmorgan-chase",
        "hsbc",
        "mizuho-bank",
        "sumitomo-mitsui-banking",
        "banco-bradesco",
        "dnb-bank-asa"
    ]
};

// ============================================================
// FORBID — Cross-tenant bank statement metadata read
// ============================================================

forbid (
    principal,
    action in [
        Action::"payments.swift_mt940_metadata_read",
        Action::"payments.swift_mt942_metadata_read",
        Action::"payments.bank_account_enumerate"
    ],
    resource is BankAccount
) when {
    !(principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") in [
        "treasury_operations_manager",
        "treasury_senior_analyst",
        "treasury_junior_analyst",
        "group_treasurer",
        "internal_audit"
    ])
};

// ============================================================
// PERMIT — Per-account Merkle anchor emission
// ============================================================

permit (
    principal == ServicePrincipal::"audit-chain-service",
    action == Action::"audit_chain.eod_account_anchor_emit",
    resource is MerkleAnchor
) when {
    resource.proof_class == "inclusion_proof" &&
    resource.ifrs_9_attestation == true &&
    resource.ias_21_fx_translation_attestation == true &&
    resource.external_transparency_log_batch != ""
};

// ============================================================
// PERMIT — LCR computation
// ============================================================

permit (
    principal,
    action == Action::"finops_portal.lcr_compute",
    resource is LCRComputeSession
) when {
    principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") in [
        "treasury_operations_manager",
        "group_treasurer",
        "cfo",
        "internal_audit",
        "external_auditor"
    ]
};

forbid (
    principal,
    action == Action::"finops_portal.lcr_threshold_breach_silence",
    resource is LCRComputeSession
) when {
    resource.lcr_ratio < 1.0
};

// ============================================================
// PERMIT — Pack manifest assertion
// ============================================================

permit (
    principal == ServicePrincipal::"compliance-pack-asserter",
    action == Action::"compliance.pack_manifest_assert",
    resource is PackManifest
) when {
    resource.active_packs.containsAll([
        "pack-ifrs-9-financial-instruments",
        "pack-ias-21-fx-effects",
        "pack-emir-article-9-trade-reporting",
        "pack-dodd-frank-title-vii-swap-reporting",
        "pack-mifid-ii-transaction-reporting",
        "pack-bis-basel-iii-lcr",
        "pack-iso-20022-mx-2026",
        "pack-ibor-transition-rfr-2027"
    ]) &&
    resource.cross_validation_state == "passed"
};

// ============================================================
// PERMIT — Cut-off window operations (per window)
// ============================================================

permit (
    principal,
    action in [
        Action::"treasury.cut_off_window_open",
        Action::"treasury.cut_off_window_close"
    ],
    resource is CutOffWindow
) when {
    principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") in [
        "treasury_operations_manager",
        "group_treasurer"
    ] &&
    resource.cut_window in [
        "asia_cut",
        "sg_cut",
        "europe_cut",
        "london_cut",
        "us_cut",
        "stockholm_night_cut"
    ]
};

// ============================================================
// FORBID — Default deny on EOD session
// ============================================================

forbid (
    principal,
    action,
    resource is EODSession
) when {
    !(principal in [
        User::"sven.eriksson@bohlin-hjelmqvist-industries-ab-parent",
        User::"annika.lindqvist-holmberg@bohlin-hjelmqvist-industries-ab-parent"
    ]) &&
    !(principal.tenant_class == "bank") &&
    !(principal in [
        ServicePrincipal::"payments-service",
        ServicePrincipal::"treasury-service",
        ServicePrincipal::"finops-portal-service",
        ServicePrincipal::"audit-chain-service",
        ServicePrincipal::"observability-emitter",
        ServicePrincipal::"compliance-pack-asserter",
        ServicePrincipal::"intelligence-fx-prediction"
    ])
};
