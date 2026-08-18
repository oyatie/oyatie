#![allow(clippy::expect_used)]

use std::time::Duration;

use intelligence_kernel::safety::{
    CentralDataClassTaxonomy, CriticalSafetyCategory, EvidenceDataClass, EvidenceVisibility,
    ManualReviewState, RedactionDecision, RetentionOverlay, SafetySignal, SecondaryReviewVerdict,
    TenantDataClassOverlay, TokenLifetime, TokenizationPolicy, apply_secondary_review,
    classify_in_transit_payload, default_retention_policy_for, enforce_safety_signals,
    normal_evidence_capture_policy, triggered_evidence_capture_policy,
};

#[test]
fn critical_signals_block_quarantine_and_require_secondary_and_manual_review() {
    let decision = enforce_safety_signals(&[SafetySignal::critical(
        CriticalSafetyCategory::PromptInjectionOrJailbreak,
        EvidenceDataClass::PromptInjectionOrJailbreak,
    )]);

    assert!(decision.blocked);
    assert!(decision.quarantined);
    assert!(decision.secondary_agentic_review_required);
    assert_eq!(decision.manual_review, ManualReviewState::Required);
    assert!(!decision.tenant_may_override);
    assert!(decision.signals_to_tenant_policy);
}

#[test]
fn secondary_agentic_review_cannot_override_platform_critical_block() {
    let decision = enforce_safety_signals(&[SafetySignal::critical(
        CriticalSafetyCategory::SecurityExploitOrBreach,
        EvidenceDataClass::SecurityExploitOrBreach,
    )]);

    let safe_second_pass = apply_secondary_review(&decision, SecondaryReviewVerdict::Safe);
    assert!(safe_second_pass.blocked);
    assert!(safe_second_pass.quarantined);
    assert_eq!(safe_second_pass.manual_review, ManualReviewState::Required);

    let unsafe_second_pass = apply_secondary_review(&decision, SecondaryReviewVerdict::Unsafe);
    assert!(unsafe_second_pass.secondary_review_flagged_unsafe);
    assert_eq!(
        unsafe_second_pass.manual_review,
        ManualReviewState::Required
    );
}

#[test]
fn normal_path_never_stores_raw_payload_and_guardrail_uses_encrypted_handle() {
    let normal = normal_evidence_capture_policy();
    assert!(!normal.raw_payload_stored);
    assert!(normal.encrypted_evidence_handle.is_none());
    assert_eq!(
        normal.visibility,
        EvidenceVisibility::RedactedStructuredEvidenceOnly
    );

    let triggered = triggered_evidence_capture_policy(EvidenceDataClass::CredentialOrSecret);
    assert!(triggered.raw_payload_stored);
    assert!(triggered.encrypted_evidence_handle.is_some());
    assert_eq!(
        triggered.visibility,
        EvidenceVisibility::RedactedStructuredEvidenceOnly
    );
    assert!(triggered.raw_access_requires_audited_break_glass);
    assert_eq!(
        triggered.ttl,
        default_retention_policy_for(EvidenceDataClass::CredentialOrSecret).ttl
    );
}

#[test]
fn tenant_overlay_may_tighten_but_not_downgrade_data_classes_or_expand_access() {
    let taxonomy = CentralDataClassTaxonomy;

    let tightened = TenantDataClassOverlay::new(
        "tenant-a",
        EvidenceDataClass::PersonalData,
        EvidenceDataClass::CredentialOrSecret,
        RetentionOverlay::shorter_ttl(Duration::from_secs(6 * 60 * 60)),
    )
    .expect("tenant can tighten personal data into secret-like handling");
    assert!(taxonomy.validate_overlay(&tightened).is_ok());

    let downgrade = TenantDataClassOverlay::new(
        "tenant-a",
        EvidenceDataClass::CredentialOrSecret,
        EvidenceDataClass::PublicOrOperationalMetadata,
        RetentionOverlay::shorter_ttl(Duration::from_secs(6 * 60 * 60)),
    )
    .expect("shape parses before taxonomy policy rejects downgrade");
    assert!(taxonomy.validate_overlay(&downgrade).is_err());

    let expanded_access = TenantDataClassOverlay::new(
        "tenant-a",
        EvidenceDataClass::PaymentOrFinancialData,
        EvidenceDataClass::PaymentOrFinancialData,
        RetentionOverlay::raw_access_without_break_glass(),
    )
    .expect("shape parses before taxonomy policy rejects access expansion");
    assert!(taxonomy.validate_overlay(&expanded_access).is_err());
}

#[test]
fn in_transit_redaction_blocks_sensitive_and_tokenizes_trivial_personal_data() {
    let secret = classify_in_transit_payload(
        "api key sk-live-secret",
        EvidenceDataClass::CredentialOrSecret,
    );
    assert_eq!(secret.decision, RedactionDecision::BlockAndQuarantine);

    let email =
        classify_in_transit_payload("email jane@example.com", EvidenceDataClass::PersonalData);
    assert_eq!(email.decision, RedactionDecision::ReversibleTokenize);
    assert!(!email.model_receives_raw_value);
    assert!(!email.routing_advisor_receives_raw_value);
    assert!(!email.secondary_review_receives_raw_value);
}

#[test]
fn reversible_tokens_are_policy_gated_ephemeral_by_default() {
    let default_policy = TokenizationPolicy::tenant_approved_default("tenant-a", "support-reply");
    assert!(default_policy.tenant_policy_approved);
    assert_eq!(default_policy.lifetime, TokenLifetime::EphemeralRun);
    assert!(default_policy.restore_only_after_model_output);
    assert!(!default_policy.provider_visible);
    assert!(!default_policy.routing_advisor_visible);

    assert!(
        TokenizationPolicy::long_lived_without_named_workflow("tenant-a").is_err(),
        "long-lived reversible maps require named workflow plus TTL"
    );
}
