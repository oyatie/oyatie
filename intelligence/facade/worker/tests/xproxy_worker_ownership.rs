#![allow(clippy::expect_used, clippy::panic)]

use intelligence_worker::{
    AgentDelegationPolicySpec, AgentMemoryBindingSpec, AgentRuntimeProfileSpec, AgentScheduleSpec,
    AgentSkillBundleSpec, AgentWorkspaceBindingSpec, BackendRegistry, CloudAuthRequirements,
    ConfigLayer, ConfigSource, CredentialRefreshPlan, DriftParityPlan,
    EvidenceRetentionProfileSpec, GuardrailDetectionProfileSpec, InTransitRedactionProfileSpec,
    InternalCodingAgentWorkflowPlan, ManualReviewEscalationSpec, ModelRouteSpec,
    OAuthLifecyclePlan, ParityCanaryStatusSpec, ParityCanaryStatusState, PoolActivation,
    ProviderBackendSpec, ProviderClass, ReferenceCiPatternCatalog, RoutingAdvisorPurpose,
    SafetySignalPolicySpec, ScheduledParityDriftCanaryPlan, WorkerKind,
    default_routing_advisor_profiles, default_worker_ownership, resolve_config_precedence,
};

#[test]
fn worker_ownership_map_keeps_hot_path_and_control_plane_separate() {
    let map = default_worker_ownership();
    assert!(
        map.iter()
            .any(|worker| worker.name == "cloud-intelligence-gateway"
                && worker.kind == WorkerKind::GatewayDeployment)
    );
    assert!(map.iter().any(|worker| worker.name == "route-controller"));
    assert!(
        map.iter()
            .any(|worker| worker.name == "model-inventory-worker")
    );
    assert!(
        map.iter()
            .any(|worker| worker.name == "drift-parity-worker")
    );
    assert!(map.iter().any(|worker| worker.kind == WorkerKind::CronJob));
    assert!(
        map.iter()
            .all(|worker| !worker.hot_path || worker.name == "cloud-intelligence-gateway")
    );
    assert!(
        map.iter()
            .all(|worker| !worker.writes_raw_prompts_or_secrets)
    );
}

fn manifest() -> String {
    // cargo test binaries run with CWD = package root (buck2 used repo-root CWD);
    // anchor the repo-relative manifest path at the repo root marker. Buck2 does not
    // define CARGO_MANIFEST_DIR, so fall back to the current directory (repo root under
    // buck2, package root under cargo) instead of a compile-time env!.
    let anchor = option_env!("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let mut root = anchor.clone();
    loop {
        if root.join("specs/root-hub-pointers.json").is_file() {
            break;
        }
        if !root.pop() {
            panic!("failed to locate repo root from {}", anchor.display());
        }
    }
    std::fs::read_to_string(root.join("intelligence/k8s/cloud-intelligence.yaml"))
        .expect("read cloud-intelligence manifest")
}

fn assert_manifest_contains(manifest: &str, needle: &str) {
    assert!(manifest.contains(needle), "manifest missing {needle}");
}

#[test]
fn k8s_manifest_declares_crds_workers_canaries_and_hardening() {
    let manifest = manifest();
    for crd in [
        "providerbackends.cloud-intelligence.oya.io",
        "modelroutes.cloud-intelligence.oya.io",
        "modelaliassets.cloud-intelligence.oya.io",
        "promptprofiles.cloud-intelligence.oya.io",
        "thinkingpolicies.cloud-intelligence.oya.io",
        "subscriptionseats.cloud-intelligence.oya.io",
        "wireprofiles.cloud-intelligence.oya.io",
        "toolcompatibilityprofiles.cloud-intelligence.oya.io",
        "gatewaycircuitbreakers.cloud-intelligence.oya.io",
        "capabilityparitybaselines.cloud-intelligence.oya.io",
        "agentruntimeprofiles.cloud-intelligence.oya.io",
        "agentmemorybindings.cloud-intelligence.oya.io",
        "agentskillbundles.cloud-intelligence.oya.io",
        "agentschedules.cloud-intelligence.oya.io",
        "agentdelegationpolicies.cloud-intelligence.oya.io",
        "agentworkspacebindings.cloud-intelligence.oya.io",
        "guardraildetectionprofiles.cloud-intelligence.oya.io",
        "evidenceretentionprofiles.cloud-intelligence.oya.io",
        "intransitredactionprofiles.cloud-intelligence.oya.io",
        "safetysignalpolicies.cloud-intelligence.oya.io",
        "manualreviewescalations.cloud-intelligence.oya.io",
    ] {
        assert_manifest_contains(&manifest, crd);
    }

    for deployment in [
        "name: cloud-intelligence-gateway",
        "name: route-controller",
        "name: model-inventory-worker",
        "name: credential-refresh-worker",
        "name: analytics-metering-worker",
        "name: circuit-breaker-worker",
        "name: ops-api",
        "name: agent-runtime-controller",
        "name: agent-scheduler-worker",
        "name: agent-delegation-worker",
        "name: safety-enforcement-controller",
        "name: guardrail-detection-worker",
        "name: evidence-retention-controller",
    ] {
        assert_manifest_contains(&manifest, deployment);
    }

    assert_manifest_contains(&manifest, "kind: CronJob");
    assert_manifest_contains(&manifest, "name: drift-parity-worker");
    assert_manifest_contains(&manifest, "kind: Job");
    assert_manifest_contains(&manifest, "name: compatibility-worker");
    assert_manifest_contains(&manifest, "kind: ServiceAccount");
    assert_manifest_contains(&manifest, "kind: Role");
    assert_manifest_contains(&manifest, "kind: RoleBinding");
    assert_manifest_contains(&manifest, "kind: NetworkPolicy");
    assert_manifest_contains(&manifest, "kind: PodDisruptionBudget");
    assert_manifest_contains(&manifest, "readOnlyRootFilesystem: true");
    assert_manifest_contains(&manifest, "allowPrivilegeEscalation: false");
    assert_manifest_contains(&manifest, "runAsNonRoot: true");
    assert_manifest_contains(&manifest, "path: /livez");
    assert_manifest_contains(&manifest, "path: /readyz");
    assert!(!manifest.contains("value: sk-"));
}

#[test]
fn agent_runtime_resources_are_first_class_but_durable_state_uses_refs() {
    let runtime = AgentRuntimeProfileSpec::new(
        "tenant-a",
        "dogfood-codex-runtime",
        "codex-default-route",
        "prompt-default",
        "thinking-default",
        "tool-compat-default",
        "sandbox-restricted",
    )
    .expect("first-class runtime profile");
    assert_eq!(runtime.kind, "AgentRuntimeProfile");
    assert!(runtime.cloud_intelligence_owned_control_plane);
    assert!(!runtime.embeds_model_runtime);
    assert!(!runtime.installs_cli_or_tui_surface);

    let memory = AgentMemoryBindingSpec::new(
        "tenant-a",
        "dogfood-memory",
        "memory-ref://tenant-a/agents/codex/default",
    )
    .expect("memory binding uses typed ref");
    assert_eq!(memory.kind, "AgentMemoryBinding");
    assert!(memory.durable_state_externalized);
    assert!(!memory.stores_prompt_or_completion_body);

    let workspace = AgentWorkspaceBindingSpec::new(
        "tenant-a",
        "dogfood-workspace",
        "workspace-ref://tenant-a/agents/codex/default",
    )
    .expect("workspace binding uses typed ref");
    assert_eq!(workspace.kind, "AgentWorkspaceBinding");
    assert!(workspace.durable_state_externalized);
    assert!(!workspace.mounts_host_paths);

    assert!(
        AgentMemoryBindingSpec::new("tenant-a", "bad-memory", "postgres://raw").is_err(),
        "memory bindings must not embed durable storage coordinates"
    );
    assert!(
        AgentWorkspaceBindingSpec::new("tenant-a", "bad-workspace", "/tmp/local-workspace")
            .is_err(),
        "workspace bindings must not point at local paths"
    );
}

#[test]
fn agent_skills_schedules_and_delegation_are_policy_gated_cloud_resources() {
    let skill = AgentSkillBundleSpec::new(
        "tenant-a",
        "analysis-skills",
        "skillbundle-ref://tenant-a/analysis/v1",
        "tool-compat-default",
    )
    .expect("skill bundle resource");
    assert_eq!(skill.kind, "AgentSkillBundle");
    assert!(skill.policy_gated);
    assert!(!skill.installs_local_hooks);

    let schedule = AgentScheduleSpec::new(
        "tenant-a",
        "nightly-drift-check",
        "schedule-ref://tenant-a/nightly-drift-check",
        "dogfood-codex-runtime",
    )
    .expect("schedule resource");
    assert_eq!(schedule.kind, "AgentSchedule");
    assert!(schedule.execution_externalized_to_controller);
    assert!(!schedule.embeds_local_cron);

    let delegation = AgentDelegationPolicySpec::new(
        "tenant-a",
        "codex-claude-gemini-delegation",
        &["codex", "claude", "gemini"],
        "owned-policy-engine-port",
    )
    .expect("delegation policy");
    assert_eq!(delegation.kind, "AgentDelegationPolicy");
    assert_eq!(
        delegation.allowed_generation_adapters,
        ["claude", "codex", "gemini"]
    );
    assert!(delegation.policy_gated);
    assert!(!delegation.allows_routing_advisor_generation);
}

#[test]
fn agent_runtime_workers_are_control_plane_only_and_redacted() {
    let map = default_worker_ownership();
    for worker_name in [
        "agent-runtime-controller",
        "agent-scheduler-worker",
        "agent-delegation-worker",
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
                .all(|write| !write.contains("prompt") && !write.contains("secret")),
            "runtime workers write only redacted/status resources"
        );
    }
}

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

#[test]
fn xproxy_route_004_005_006_model_and_backend_management_are_declarative_resources() {
    let route = ModelRouteSpec::policy_authorized_override(
        "tenant-a",
        "chat",
        "compat-gateway",
        "gpt-4o-mini",
        "medium",
        8192,
    )
    .expect("policy-authorized route override");
    assert_eq!(route.normalized_effort.as_deref(), Some("medium"));
    assert_eq!(route.max_output_tokens, Some(8192));
    assert!(route.policy_authorized);

    let rejected =
        ModelRouteSpec::unauthorized_override("tenant-a", "chat", "compat-gateway", "gpt-4o-mini");
    assert!(
        rejected.is_err(),
        "route overrides require policy authorization"
    );

    let backends = BackendRegistry::from_specs(vec![
        ProviderBackendSpec::new_openai_compatible(
            "compat-primary",
            "https://provider-a.example/v1",
            "secret-ref://tenant-a/provider/primary",
            90,
        )
        .unwrap(),
        ProviderBackendSpec::new_openai_compatible(
            "compat-secondary",
            "https://provider-b.example/v1",
            "secret-ref://tenant-a/provider/secondary",
            10,
        )
        .unwrap(),
    ])
    .expect("multiple named provider backends");

    assert_eq!(backends.len(), 2);
    assert_eq!(backends.weighted_fallback_order()[0], "compat-primary");
    assert!(
        ProviderBackendSpec::new_openai_compatible(
            "../unsafe",
            "https://provider.example/v1",
            "secret-ref://tenant-a/provider/unsafe",
            1,
        )
        .is_err()
    );
}

#[test]
fn xproxy_route_005_cheaper_model_advisors_are_routing_only_adapter_backed_and_redacted() {
    let advisors = default_routing_advisor_profiles();
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "chatgpt-spark")
    );
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "gemini-3.1-flash-lite")
    );
    assert!(
        advisors
            .iter()
            .any(|advisor| advisor.model_hint == "nemotron-3-ultra-550b-a55b")
    );

    for advisor in advisors {
        assert_eq!(advisor.purpose, RoutingAdvisorPurpose::RoutingDecisionOnly);
        assert!(advisor.adapter_backed);
        assert!(!advisor.may_execute_generation);
        assert!(!advisor.receives_raw_prompts_or_secrets);
        assert!(
            advisor.receives_redacted_route_metadata,
            "routing advisors should receive only redacted route metadata"
        );
    }
}

#[test]
fn xproxy_auth_001_002_006_007_008_lifecycle_auth_and_config_are_cloud_native() {
    let lifecycle = OAuthLifecyclePlan::manual_headless_enrollment(
        "tenant-a",
        ProviderClass::AnthropicSubscription,
        "secret-ref://tenant-a/oauth/seat-a",
    )
    .expect("worker-safe lifecycle plan");
    assert!(lifecycle.worker_safe);
    assert!(!lifecycle.uses_browser_automation);
    assert!(lifecycle.refresh_token_handle.starts_with("secret-ref://"));

    assert_eq!(
        PoolActivation::from_seat_count(1),
        PoolActivation::SingleSeat
    );
    assert_eq!(
        PoolActivation::from_seat_count(2),
        PoolActivation::MultiSeatActive
    );

    let refresh = CredentialRefreshPlan::singleflight(
        "tenant-a",
        ProviderClass::AnthropicSubscription,
        "secret-ref://tenant-a/oauth/seat-a",
    )
    .unwrap();
    assert_eq!(
        refresh.singleflight_group_key,
        "tenant-a:anthropic-subscription"
    );
    assert!(!refresh.stores_plaintext_secret);

    let auth = CloudAuthRequirements::non_loopback_default();
    assert!(auth.requires_tenant_authn);
    assert!(auth.requires_policy_engine_decision);
    assert!(auth.requires_mtls_or_api_key_at_edge);
    assert!(auth.cors_requires_policy_review);

    let resolved = resolve_config_precedence([
        ConfigLayer::new(ConfigSource::ServiceDefault, "model", "sonnet"),
        ConfigLayer::new(ConfigSource::TenantDefault, "model", "opus"),
        ConfigLayer::new(ConfigSource::ModelRoute, "model", "openai:gpt-4o"),
    ])
    .expect("deterministic cloud config precedence");
    assert_eq!(
        resolved.get("model").map(String::as_str),
        Some("openai:gpt-4o")
    );
    assert!(
        ConfigLayer::new(ConfigSource::ModelRoute, "provider_key", "sk-raw-value")
            .validate_no_raw_secret()
            .is_err()
    );
}

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
fn internal_coding_agent_workflow_composes_cloud_intelligence_resources_only() {
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
    assert!(workflow.cloud_intelligence_primitive_only);
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
