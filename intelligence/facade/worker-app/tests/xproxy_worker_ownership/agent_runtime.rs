use intelligence_worker_app::{
    AgentDelegationPolicySpec, AgentMemoryBindingSpec, AgentRuntimeProfileSpec, AgentScheduleSpec,
    AgentSkillBundleSpec, AgentWorkspaceBindingSpec, default_worker_ownership,
};

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
    assert!(runtime.intelligence_app_owned_control_plane);
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
