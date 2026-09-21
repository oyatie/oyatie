use intelligence_worker_app::{
    EvidenceRetentionProfileSpec, GuardrailDetectionProfileSpec, InTransitRedactionProfileSpec,
    ManualReviewEscalationSpec, SafetySignalPolicySpec, default_worker_ownership,
};

#[test]
fn safety_guardrail_resources_encode_platform_floor_and_secondary_review() {
    let guardrail = GuardrailDetectionProfileSpec::platform_default(
        "tenant-a",
        "platform-critical-guardrails",
        "owned-policy-engine-port",
    )
    .expect("platform guardrail profile");
    assert_eq!(guardrail.kind, "GuardrailDetectionProfile");
    assert!(guardrail.automatic_block_and_quarantine);
    assert!(guardrail.mandatory_secondary_agentic_review);
    assert!(guardrail.manual_review_required_after_secondary_review);
    assert!(!guardrail.tenant_may_weaken_platform_floor);
    assert!(
        guardrail
            .critical_categories
            .contains(&"prompt-injection-or-jailbreak".to_string())
    );
    assert!(
        guardrail
            .critical_categories
            .contains(&"data-exfiltration-or-breach".to_string())
    );
    assert!(
        guardrail
            .critical_categories
            .contains(&"self-harm-or-harm-to-others".to_string())
    );

    assert!(
        GuardrailDetectionProfileSpec::platform_default(
            "tenant-a",
            "bad-policy-port",
            "cedar-direct",
        )
        .is_err(),
        "guardrails must use owned policy-engine port, not a concrete transient adapter"
    );
}

#[test]
fn evidence_retention_and_manual_review_default_to_redacted_break_glass() {
    let evidence = EvidenceRetentionProfileSpec::platform_default(
        "tenant-a",
        "platform-evidence-retention",
        "owned-secret-provider-port",
    )
    .expect("evidence retention profile");
    assert_eq!(evidence.kind, "EvidenceRetentionProfile");
    assert!(!evidence.stores_raw_payload_on_normal_path);
    assert!(evidence.encrypted_handle_on_guardrail_trigger);
    assert!(evidence.fixed_ttl_by_data_class);
    assert!(evidence.regulatory_classification_required);
    assert_eq!(
        evidence.default_reviewer_visibility,
        "redacted-structured-evidence"
    );
    assert!(evidence.raw_access_requires_audited_break_glass);

    let review = ManualReviewEscalationSpec::platform_default("tenant-a", "critical-manual-review")
        .expect("manual review profile");
    assert_eq!(review.kind, "ManualReviewEscalation");
    assert!(review.required_for_critical_blocks);
    assert_eq!(
        review.default_evidence_visibility,
        "redacted-structured-evidence"
    );
    assert!(review.raw_payload_break_glass_only);
    assert!(review.secondary_agentic_review_must_run_first);
}

#[test]
fn in_transit_redaction_blocks_sensitive_and_allows_policy_approved_tokens() {
    let redaction =
        InTransitRedactionProfileSpec::platform_default("tenant-a", "in-transit-data-protection")
            .expect("redaction profile");
    assert_eq!(redaction.kind, "InTransitRedactionProfile");
    assert!(redaction.blocks_sensitive_classes);
    assert!(redaction.redacts_trivial_personal_data);
    assert!(redaction.reversible_tokens_require_tenant_policy);
    assert_eq!(redaction.default_token_lifetime, "ephemeral-run");
    assert!(redaction.restore_only_after_model_output);
    assert!(!redaction.provider_receives_raw_token_values);
    assert!(!redaction.routing_advisor_receives_raw_token_values);

    let signal_policy =
        SafetySignalPolicySpec::platform_default("tenant-a", "tenant-safety-signals")
            .expect("signal policy");
    assert_eq!(signal_policy.kind, "SafetySignalPolicy");
    assert!(signal_policy.platform_automatic_enforcement);
    assert!(signal_policy.tenant_policy_receives_signals);
    assert!(signal_policy.tenant_policy_receives_recommendations);
    assert!(!signal_policy.tenant_can_override_platform_critical_block);
}

#[test]
fn safety_workers_are_control_plane_only_and_never_write_raw_payloads() {
    let map = default_worker_ownership();
    for worker_name in [
        "safety-enforcement-controller",
        "guardrail-detection-worker",
        "evidence-retention-controller",
    ] {
        let worker = map
            .iter()
            .find(|worker| worker.name == worker_name)
            .unwrap_or_else(|| panic!("missing worker ownership row for {worker_name}"));
        assert!(!worker.hot_path);
        assert!(!worker.writes_raw_prompts_or_secrets);
        assert!(
            worker
                .writes
                .iter()
                .all(|write| !write.contains("raw-payload") && !write.contains("secret")),
            "safety workers write only redacted signals, sealed handles, or status"
        );
    }
}
