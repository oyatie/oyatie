// j175 — LP K-1 reconciliation Cedar policy
// Bundle id: cedar-bundle-lp-k1-aanya-fy2026
// Deployed to: aanya-kapoor-personal-2008 + 4 fund-GP tenants + IRS + 8 state revenue + Wells Goldman CPA tenant
// ADR-0243 + ADR-0244 + ADR-0245 + ADR-0247 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0263

// ============================================================
// PERMIT — LP: K-1 reconciliation session operate
// ============================================================

permit (
    principal == User::"aanya.kapoor@aanya-kapoor-personal-2008",
    action in [
        Action::"lp_cockpit.open",
        Action::"finops_portal.lp_capital_account_reconcile",
        Action::"finops_portal.tax_character_categorize",
        Action::"finops_portal.section_199a_compute",
        Action::"finops_portal.section_1411_niit_compute",
        Action::"compliance.state_apportionment_compute",
        Action::"compliance.foreign_tax_credit_compute",
        Action::"compliance.amt_compute",
        Action::"compliance.pack_manifest_assert",
        Action::"payments.irs_direct_pay_dispatch",
        Action::"payments.state_revenue_dispatch",
        Action::"drive.k1_worm_archive",
        Action::"drive.cpa_package_share",
        Action::"connect.gp_lp_channel_open",
        Action::"connect.gp_lp_channel_send",
        Action::"audit_chain.k1_attestation_emit"
    ],
    resource is LPK1ReconciliationSession
) when {
    principal.tenant_class == "personal" &&
    principal.accredited_investor_attestation_id != "" &&
    principal.qualified_purchaser_attestation_id != "" &&
    context.passkey_assertion_present == true &&
    context.face_attestation_present == true
};

// ============================================================
// PERMIT — Fund GPs: K-1 dispatch + GP-LP channel
// ============================================================

permit (
    principal,
    action in [
        Action::"finops_portal.k1_dispatch",
        Action::"connect.gp_lp_channel_open",
        Action::"connect.gp_lp_channel_send"
    ],
    resource is K1Dispatch
) when {
    principal.tenant_id in [
        "andreessen-horowitz-fund-vii",
        "sequoia-capital-us-growth-fund-ix",
        "kkr-asian-fund-v",
        "insight-venture-partners-xii"
    ] &&
    resource.recipient_lp_principal == "aanya.kapoor@aanya-kapoor-personal-2008"
};

// ============================================================
// PERMIT — GP-LP channel (per-fund dyad/tetrad)
// ============================================================

permit (
    principal,
    action in [
        Action::"connect.gp_lp_channel_send",
        Action::"connect.gp_lp_channel_receive"
    ],
    resource is GPLPChannel
) when {
    principal in resource.permitted_principals &&
    resource.channel_class == "gp_lp_quarterly_investor_relations" &&
    context.payload_class in [
        "k1_clarification_question",
        "k1_clarification_answer",
        "capital_call_notice",
        "distribution_notice",
        "partner_allocation_clarification",
        "annual_meeting_logistics"
    ]
};

forbid (
    principal,
    action == Action::"connect.gp_lp_channel_enumerate",
    resource is GPLPChannel
) when {
    !(principal in resource.permitted_principals)
};

// ============================================================
// PERMIT — CPA receives package (read-only)
// ============================================================

permit (
    principal,
    action in [
        Action::"drive.cpa_package_read",
        Action::"drive.cpa_package_list"
    ],
    resource is CPAPackage
) when {
    principal.tenant_id == "wells-goldman-cpa" &&
    resource.recipient_principal == "patricia.wells-goldman@wells-goldman-cpa" &&
    context.cpa_engagement_letter_active == true
};

forbid (
    principal,
    action in [
        Action::"drive.cpa_package_update",
        Action::"drive.cpa_package_overwrite"
    ],
    resource is CPAPackage
) when {
    !(principal == User::"aanya.kapoor@aanya-kapoor-personal-2008")
};

// ============================================================
// PERMIT — IRS Direct Pay
// ============================================================

permit (
    principal,
    action == Action::"payments.irs_direct_pay_dispatch",
    resource is EstimatedTaxPayment
) when {
    principal.tenant_class == "personal" &&
    resource.recipient_tenant == "irs-direct-pay" &&
    context.sanctions_screening_irs_clean_by_definition == true
};

// ============================================================
// PERMIT — State revenue dispatch (8 states)
// ============================================================

permit (
    principal,
    action == Action::"payments.state_revenue_dispatch",
    resource is EstimatedTaxPayment
) when {
    resource.state in ["CA", "NY", "MA", "TX", "WA", "CO", "TN", "FL"] &&
    resource.recipient_tenant in [
        "ca-ftb",
        "ny-dtf",
        "ma-dor",
        "tx-no-income-tax",
        "wa-no-income-tax",
        "co-dor",
        "tn-no-income-tax",
        "fl-no-income-tax"
    ]
};

// ============================================================
// PERMIT — K-1 PDF WORM archival
// ============================================================

permit (
    principal,
    action == Action::"drive.k1_worm_archive",
    resource is K1WORMArtifact
) when {
    resource.worm_cell == "us-west-tier-1-worm-irs-retention" &&
    resource.seal_class == "irs-aligned-worm-class-1" &&
    resource.retention_years_minimum >= 7 &&
    resource.indelible_storage_attestation == true &&
    resource.time_stamp_authority_id != ""
};

forbid (
    principal,
    action in [
        Action::"drive.k1_worm_update",
        Action::"drive.k1_worm_delete",
        Action::"drive.k1_worm_overwrite"
    ],
    resource is K1WORMArtifact
) when {
    resource.seal_state == "sealed"
};

// ============================================================
// PERMIT — Tax-authority compulsion (IRS or state DOR)
// ============================================================

permit (
    principal,
    action == Action::"audit_chain.k1_inclusion_proof_request",
    resource is MerkleAnchor
) when {
    context.tax_authority_compulsion_order_id != "" &&
    context.tax_authority_class in [
        "IRS",
        "CA_FTB",
        "NY_DTF",
        "MA_DOR",
        "CO_DOR"
    ] &&
    resource.privilege_class == "k1_tax_substantiation"
    // proof only — payload disclosure requires specific court order
};

// ============================================================
// PERMIT — Pack manifest assertion (10 packs)
// ============================================================

permit (
    principal == ServicePrincipal::"compliance-pack-asserter",
    action == Action::"compliance.pack_manifest_assert",
    resource is PackManifest
) when {
    resource.active_packs.containsAll([
        "pack-irs-schedule-k-1-1065-v3",
        "pack-irs-section-199a-qbi-2026",
        "pack-irs-section-1411-niit-2026",
        "pack-state-tax-apportionment-multi-2026",
        "pack-irc-section-754-step-up",
        "pack-amt-2026",
        "pack-foreign-tax-credit-form-1116-2026",
        "pack-eu-aifmd-non-eu-fund-marketing",
        "pack-uk-nppr-non-eu-fund",
        "pack-accredited-investor-reg-501-rule-144a"
    ]) &&
    resource.cross_validation_state == "passed"
};

// ============================================================
// FORBID — Default deny on LP K-1 session
// ============================================================

forbid (
    principal,
    action,
    resource is LPK1ReconciliationSession
) when {
    !(principal == User::"aanya.kapoor@aanya-kapoor-personal-2008") &&
    !(principal.tenant_id in [
        "andreessen-horowitz-fund-vii",
        "sequoia-capital-us-growth-fund-ix",
        "kkr-asian-fund-v",
        "insight-venture-partners-xii",
        "wells-goldman-cpa"
    ]) &&
    !(principal in [
        ServicePrincipal::"finops-portal-service",
        ServicePrincipal::"compliance-pack-asserter",
        ServicePrincipal::"payments-service",
        ServicePrincipal::"drive-service",
        ServicePrincipal::"connect-service",
        ServicePrincipal::"audit-chain-service",
        ServicePrincipal::"intelligence-tax-scenario",
        ServicePrincipal::"observability-emitter"
    ]) &&
    context.tax_authority_compulsion_order_id == ""
};
