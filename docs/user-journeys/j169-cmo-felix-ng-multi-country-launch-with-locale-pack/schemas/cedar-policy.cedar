// j169 — Cedar policy bundle for ASEAN-6 multi-country launch
// Implements ADR-0243 (Cedar gate), ADR-0244 (tenant scoping), ADR-0263 (audit dual-seal),
// ADR-0251 (compliance pack primitive), ADR-0252 (TrueTime), ADR-0255 (intelligence two-layer).

// =====================================================================
// READINESS READ — cmo / ceo / md / compliance
// =====================================================================
permit (
    principal,
    action == Action::"marketing-automation.launch_readiness_read",
    resource is LaunchReadiness
) when {
    principal.role in ["cmo", "ceo", "md_sg", "md_id", "md_th", "md_vn", "md_ph", "md_my", "compliance"] &&
    principal.tenant in [
        "veritem-health-asia-pte-ltd-sg",
        "veritem-health-singapore-sub-tenant",
        "veritem-health-indonesia-sub-tenant",
        "veritem-health-thailand-sub-tenant",
        "veritem-health-vietnam-sub-tenant",
        "veritem-health-philippines-sub-tenant",
        "veritem-health-malaysia-sub-tenant"
    ]
};

// =====================================================================
// CONTENT LOCALIZATION BATCH — cmo or system-orchestrator
// =====================================================================
permit (
    principal,
    action == Action::"intelligence.localize_content_batch",
    resource is LocalizationBatch
) when {
    (principal.role == "cmo" || principal == User::"system-principal.localization-orchestrator") &&
    resource.ai_content_transparency_required == true &&
    context.eu_ai_act_art_50_policy_active == true
};

// =====================================================================
// AI-CONTENT-TRANSPARENCY FORBID — every AI-translated string MUST have disclosure
// (EU-AI-Act-Art-50 pre-emptive compliance)
// =====================================================================
forbid (
    principal,
    action == Action::"intelligence.publish_localized_content",
    resource is LocalizedContent
) when {
    resource.translation_source == "ai-generated" &&
    resource.ai_content_transparency_disclosure_present == false
};

// =====================================================================
// AMBASSADOR CREDENTIAL ISSUE — cmo + ndab signed + consent
// =====================================================================
permit (
    principal,
    action == Action::"community.ambassador_credential_issue",
    resource is AmbassadorCredential
) when {
    principal.role in ["cmo", "ceo"] &&
    resource.ndab_signed == true &&
    resource.consent_gdpr_equivalent_signed == true &&
    resource.country_code in ["SG", "ID", "TH", "VN", "PH", "MY"]
};

// =====================================================================
// COHORT SPLIT — md + cmo + compliance + research-ethics
// =====================================================================
permit (
    principal,
    action == Action::"marketing-automation.cohort_rules_set",
    resource is CohortRules
) when {
    resource.signer_md != null &&
    resource.signer_cmo != null &&
    resource.signer_compliance != null &&
    resource.research_ethics_review_passed == true &&
    resource.cohorts_percentage_sum_pct >= 99.5 &&
    resource.cohorts_percentage_sum_pct <= 100.5
};

// =====================================================================
// GO/NO-GO — 8-of-8 quorum (CEO + CMO + 6 MDs)
// =====================================================================
permit (
    principal,
    action == Action::"marketing-automation.country_launch_approve",
    resource is CountryLaunch
) when {
    resource.country_code in ["SG", "ID", "TH", "VN", "PH", "MY"] &&
    resource.quorum_count >= 8 &&
    resource.locale_pack_activated == true &&
    resource.content_localization_qa_passed == true &&
    resource.cohort_split_signed == true &&
    resource.ambassadors_confirmed == true &&
    context.business_hours_sgt == true &&
    context.truetime_uncertainty_ms <= 10 &&
    principal in Group::"veritem-go-no-go-quorum-members"
};

// =====================================================================
// PER-COUNTRY LAUNCH FLAG FLIP — orchestrator-triggered after quorum
// =====================================================================
permit (
    principal,
    action == Action::"feature-flags.per_country_launch_flip",
    resource is PerCountryFlag
) when {
    principal == User::"system-principal.launch-orchestrator" &&
    resource.go_no_go_permit_seal like "EVT-J169-GO-LIVE-PERMIT-*" &&
    resource.effective_at_local == "08:00" &&
    context.truetime_uncertainty_ms <= 10
};

// =====================================================================
// PAYMENT PROCESSOR INIT — cmo + per-country compliance attestation
// =====================================================================
permit (
    principal,
    action == Action::"payments.processor_initialize",
    resource is PaymentProcessorInit
) when {
    principal.role in ["cmo", "finance_director"] &&
    resource.country_code != null &&
    resource.compliance_attestation != null
};

// =====================================================================
// LOCALE PACK AUTO-ACTIVATION (advisory; enforced by compliance µservice)
// =====================================================================
// Every subscriber tenant with `country_residency_*` attribute MUST auto-activate
// the corresponding country pack(s). No manual override allowed for B2C subscribers.
permit (
    principal,
    action == Action::"compliance.locale_pack_auto_activate",
    resource is LocalePackActivation
) when {
    principal == User::"system-principal.compliance-orchestrator" &&
    resource.country_residency_attribute != null &&
    resource.activation_mode == "auto"
};

forbid (
    principal,
    action == Action::"compliance.locale_pack_manual_override",
    resource is LocalePackActivation
) when {
    resource.subscriber_tier == "b2c" &&
    resource.justification == null
};

// =====================================================================
// CROSS-BORDER TRANSFER ATTESTATION — coo or cmo + consent + framework
// =====================================================================
permit (
    principal,
    action == Action::"compliance.cross_border_transfer_attestation",
    resource is CrossBorderAttestation
) when {
    principal.role in ["coo", "cmo", "ceo", "compliance"] &&
    resource.asean_privacy_framework_attestation == true &&
    resource.per_country_consent_obtained_pct >= 99.9
};
