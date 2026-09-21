use intelligence_worker_app::{
    DriftParityPlan, InternalCodingAgentWorkflowPlan, ParityCanaryStatusSpec,
    ParityCanaryStatusState, ReferenceCiPatternCatalog, ScheduledParityDriftCanaryPlan,
};

#[test]
fn reference_test_ci_patterns_are_adopted_as_cloud_native_canaries() {
    let catalog = ReferenceCiPatternCatalog::cloud_native_adoptions();
    for pattern in [
        "path-scoped-compatibility-canaries",
        "drift-detection-with-artifacted-reports",
        "watcher-liveness-watchdog",
        "infra-vs-drift-vs-inconclusive-status-separation",
        "self-healing-pr-or-task-on-delta",
        "redaction-and-wire-fixture-regression-matrix",
    ] {
        assert!(
            catalog.adopted_patterns.contains(&pattern.to_string()),
            "missing adopted pattern {pattern}"
        );
    }

    assert!(
        catalog
            .rejected_patterns
            .contains(&"local-cli-smoke-surface".to_string())
    );
    assert!(
        catalog
            .rejected_patterns
            .contains(&"local-tui-test-surface".to_string())
    );
}

#[test]
fn xproxy_drift_001_002_workers_emit_pinned_parity_and_canary_plans() {
    let plan = DriftParityPlan::for_pinned_baseline(
        "external-proxy-reference",
        "30fed94b362f5106cc7a4feaf37019fc0ccc007f",
    );
    assert_eq!(plan.kind, "CapabilityParityBaseline");
    assert!(plan.audit_event_required);
    assert!(plan.opens_pr_or_task_on_delta);
    assert!(plan.probes.contains(&"wire-profile-drift".to_string()));

    let canaries = plan.compatibility_canaries();
    assert!(canaries.contains(&"route-matrix".to_string()));
    assert!(canaries.contains(&"streaming-fixtures".to_string()));
    assert!(canaries.contains(&"pool-failover".to_string()));
    assert!(canaries.contains(&"security-redaction".to_string()));
}

#[test]
fn internal_coding_agent_workflow_composes_intelligence_app_resources_only() {
    let workflow = InternalCodingAgentWorkflowPlan::dogfood_default(
        "tenant-a",
        "oyatie-internal-coding-agent",
        "dogfood-fable-runtime",
        "nightly-drift-check",
        "claude-codex-gemini-delegation",
        "platform-critical-guardrails",
        "platform-evidence-retention",
        "in-transit-data-protection",
    )
    .expect("workflow plan should compose existing resource refs");

    assert_eq!(workflow.kind, "AgentWorkflowPlan");
    assert_eq!(workflow.runtime_profile_ref, "dogfood-fable-runtime");
    assert_eq!(
        workflow.schedule_ref,
        "schedule-ref://tenant-a/nightly-drift-check"
    );
    assert!(workflow.intelligence_app_primitive_only);
    assert!(workflow.requires_policy_engine_decision);
    assert!(workflow.requires_secondary_review_for_critical_blocks);
    assert!(workflow.uses_redacted_evidence_handles);
    assert!(!workflow.embeds_product_workflow);
    assert!(!workflow.installs_cli_or_tui_surface);
    assert!(!workflow.stores_raw_prompt_or_completion);
    assert_eq!(workflow.generation_adapters, ["claude", "codex", "gemini"]);
    assert_eq!(workflow.routing_advisor_scope, "routing-decision-only");

    assert!(
        InternalCodingAgentWorkflowPlan::dogfood_default(
            "tenant-a",
            "Invalid Name With Spaces",
            "dogfood-fable-runtime",
            "nightly-drift-check",
            "claude-codex-gemini-delegation",
            "platform-critical-guardrails",
            "platform-evidence-retention",
            "in-transit-data-protection",
        )
        .is_err(),
        "workflow resource refs must be valid resource names"
    );
}

#[test]
fn scheduled_parity_drift_canary_plan_is_controller_owned_and_status_only() {
    let plan = ScheduledParityDriftCanaryPlan::for_internal_coding_agent(
        "tenant-a",
        "nightly-drift-check",
        "external-proxy-reference",
        "30fed94b362f5106cc7a4feaf37019fc0ccc007f",
    )
    .expect("scheduled canary plan");

    assert_eq!(plan.kind, "ScheduledParityDriftCanaryPlan");
    assert_eq!(
        plan.schedule_ref,
        "schedule-ref://tenant-a/nightly-drift-check"
    );
    assert!(plan.controller_owned);
    assert!(plan.opens_pr_or_task_on_delta);
    assert!(plan.audit_event_required);
    assert!(!plan.embeds_local_cron);
    assert!(!plan.writes_raw_prompts_or_secrets);
    assert!(plan.probes.contains(&"capability-parity".to_string()));
    assert!(plan.probes.contains(&"wire-profile-drift".to_string()));
    assert!(
        plan.compatibility_canaries
            .contains(&"route-matrix".to_string())
    );
    assert!(
        plan.compatibility_canaries
            .contains(&"security-redaction".to_string())
    );

    assert!(
        ScheduledParityDriftCanaryPlan::for_internal_coding_agent(
            "tenant-a",
            "nightly-drift-check",
            "some-other-artifact-family",
            "30fed94b362f5106cc7a4feaf37019fc0ccc007f",
        )
        .is_err(),
        "canary plans are pinned to the external-proxy-reference artifact family"
    );
    assert!(
        ScheduledParityDriftCanaryPlan::for_internal_coding_agent(
            "tenant-a",
            "nightly-drift-check",
            "external-proxy-reference",
            "not-a-full-sha",
        )
        .is_err(),
        "canary plans require a full 40-char pinned baseline sha"
    );

    let status = ParityCanaryStatusSpec::from_plan(&plan, ParityCanaryStatusState::Passed);
    assert_eq!(status.kind, "ParityCanaryStatus");
    assert_eq!(status.state, ParityCanaryStatusState::Passed);
    assert_eq!(status.retry_after_seconds, None);
    assert_eq!(status.evidence_visibility, "redacted-structured-evidence");
    assert_eq!(
        status.plan_ref,
        "parity-canary-plan-ref://tenant-a/nightly-drift-check"
    );
    assert!(status.sealed_evidence_handle_ref.is_some());
    assert!(!status.raw_payload_included);

    let failed = ParityCanaryStatusSpec::from_plan(&plan, ParityCanaryStatusState::Failed);
    assert_eq!(failed.retry_after_seconds, Some(300));
    let running = ParityCanaryStatusSpec::from_plan(&plan, ParityCanaryStatusState::Running);
    assert_eq!(running.retry_after_seconds, Some(300));
    let inconclusive =
        ParityCanaryStatusSpec::from_plan(&plan, ParityCanaryStatusState::Inconclusive);
    assert_eq!(inconclusive.retry_after_seconds, Some(300));
}
