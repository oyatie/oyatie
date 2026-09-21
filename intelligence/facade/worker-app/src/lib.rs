use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum WorkerKind {
    GatewayDeployment,
    ControllerDeployment,
    WorkerDeployment,
    CronJob,
    Job,
    OpsDeployment,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkerOwnership {
    pub name: &'static str,
    pub kind: WorkerKind,
    pub reconciles: &'static [&'static str],
    pub writes: &'static [&'static str],
    pub hot_path: bool,
    pub writes_raw_prompts_or_secrets: bool,
}

pub fn default_worker_ownership() -> Vec<WorkerOwnership> {
    vec![
        WorkerOwnership {
            name: "intelligence-app-gateway",
            kind: WorkerKind::GatewayDeployment,
            reconciles: &["cached-route-snapshot", "provider-seat-cache"],
            writes: &["redacted-usage-events"],
            hot_path: true,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "route-controller",
            kind: WorkerKind::ControllerDeployment,
            reconciles: &[
                "ProviderBackend",
                "ModelRoute",
                "ModelAliasSet",
                "WireProfile",
            ],
            writes: &["route-snapshot-status"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "model-inventory-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["ProviderBackend", "ModelAliasSet"],
            writes: &["ModelInventorySnapshot"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "credential-refresh-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["SubscriptionSeat"],
            writes: &["credential-handle-status"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "drift-parity-worker",
            kind: WorkerKind::CronJob,
            reconciles: &["CapabilityParityBaseline", "WireProfile"],
            writes: &["parity-drift-report"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "analytics-metering-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["llm.usage.v1", "llm.audit.v1"],
            writes: &["burn-rate-aggregate"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "circuit-breaker-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["GatewayCircuitBreaker"],
            writes: &["RetryAfterHint"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "compatibility-worker",
            kind: WorkerKind::Job,
            reconciles: &["ToolCompatibilityProfile"],
            writes: &["compatibility-canary-result"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "agent-runtime-controller",
            kind: WorkerKind::ControllerDeployment,
            reconciles: &[
                "AgentRuntimeProfile",
                "AgentMemoryBinding",
                "AgentSkillBundle",
                "AgentWorkspaceBinding",
            ],
            writes: &["agent-runtime-status", "redacted-runtime-events"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "agent-scheduler-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["AgentSchedule", "AgentRuntimeProfile"],
            writes: &["agent-schedule-status", "agent-run-request"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "agent-delegation-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &["AgentDelegationPolicy", "AgentRuntimeProfile", "ModelRoute"],
            writes: &["delegation-status", "route-advice-event"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "safety-enforcement-controller",
            kind: WorkerKind::ControllerDeployment,
            reconciles: &[
                "GuardrailDetectionProfile",
                "SafetySignalPolicy",
                "ManualReviewEscalation",
            ],
            writes: &["safety-enforcement-status", "redacted-safety-signals"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "guardrail-detection-worker",
            kind: WorkerKind::WorkerDeployment,
            reconciles: &[
                "GuardrailDetectionProfile",
                "InTransitRedactionProfile",
                "AgentRuntimeProfile",
            ],
            writes: &["guardrail-signal", "redacted-secondary-review-request"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "evidence-retention-controller",
            kind: WorkerKind::ControllerDeployment,
            reconciles: &["EvidenceRetentionProfile", "ManualReviewEscalation"],
            writes: &["sealed-evidence-handle", "evidence-retention-status"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
        WorkerOwnership {
            name: "ops-api",
            kind: WorkerKind::OpsDeployment,
            reconciles: &["admin-status-cache"],
            writes: &["read-only-response"],
            hot_path: false,
            writes_raw_prompts_or_secrets: false,
        },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProviderClass {
    AnthropicSubscription,
    OpenAiCompatible,
    GeminiNative,
}

impl ProviderClass {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::AnthropicSubscription => "anthropic-subscription",
            Self::OpenAiCompatible => "openai-compatible",
            Self::GeminiNative => "gemini-native",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderBackendSpec {
    pub name: String,                  // data_class: INTERNAL_ONLY
    pub provider_class: ProviderClass, // data_class: INTERNAL_ONLY
    pub base_url: String,              // data_class: INTERNAL_ONLY
    pub credential_handle: String,     // data_class: SECRET_REFERENCE
    pub weight: u16,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerConfigError {
    InvalidName,
    InvalidUrl,
    InvalidSecretReference,
    EmptyRegistry,
    RawSecretValue,
    UnauthorizedRouteMutation,
    InvalidPolicyValue,
    InvalidTypedReference,
    InvalidAdapterSet,
}

impl ProviderBackendSpec {
    pub fn new_openai_compatible(
        name: &str,
        base_url: &str,
        credential_handle: &str,
        weight: u16,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if !(base_url.starts_with("https://") || base_url.starts_with("http://127.0.0.1")) {
            return Err(WorkerConfigError::InvalidUrl);
        }
        validate_secret_reference(credential_handle)?;
        Ok(Self {
            name: name.to_string(),
            provider_class: ProviderClass::OpenAiCompatible,
            base_url: base_url.to_string(),
            credential_handle: credential_handle.to_string(),
            weight,
        })
    }

    pub fn new_gemini_native(
        name: &str,
        base_url: &str,
        credential_handle: &str,
        weight: u16,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if !(base_url.starts_with("https://") || base_url.starts_with("http://127.0.0.1")) {
            return Err(WorkerConfigError::InvalidUrl);
        }
        validate_secret_reference(credential_handle)?;
        Ok(Self {
            name: name.to_string(),
            provider_class: ProviderClass::GeminiNative,
            base_url: base_url.to_string(),
            credential_handle: credential_handle.to_string(),
            weight,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentRuntimeProfileSpec {
    pub kind: &'static str,
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub name: String,                           // data_class: INTERNAL_ONLY
    pub model_route_ref: String,                // data_class: INTERNAL_ONLY
    pub prompt_profile_ref: String,             // data_class: INTERNAL_ONLY
    pub thinking_policy_ref: String,            // data_class: INTERNAL_ONLY
    pub tool_compatibility_profile_ref: String, // data_class: INTERNAL_ONLY
    pub sandbox_policy_ref: String,             // data_class: INTERNAL_ONLY
    pub intelligence_app_owned_control_plane: bool,
    pub embeds_model_runtime: bool,
    pub installs_cli_or_tui_surface: bool,
}

impl AgentRuntimeProfileSpec {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        name: &str,
        model_route_ref: &str,
        prompt_profile_ref: &str,
        thinking_policy_ref: &str,
        tool_compatibility_profile_ref: &str,
        sandbox_policy_ref: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        for resource_ref in [
            model_route_ref,
            prompt_profile_ref,
            thinking_policy_ref,
            tool_compatibility_profile_ref,
            sandbox_policy_ref,
        ] {
            validate_resource_name(resource_ref)?;
        }
        Ok(Self {
            kind: "AgentRuntimeProfile",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            model_route_ref: model_route_ref.to_string(),
            prompt_profile_ref: prompt_profile_ref.to_string(),
            thinking_policy_ref: thinking_policy_ref.to_string(),
            tool_compatibility_profile_ref: tool_compatibility_profile_ref.to_string(),
            sandbox_policy_ref: sandbox_policy_ref.to_string(),
            intelligence_app_owned_control_plane: true,
            embeds_model_runtime: false,
            installs_cli_or_tui_surface: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentMemoryBindingSpec {
    pub kind: &'static str,
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub name: String,       // data_class: INTERNAL_ONLY
    pub memory_ref: String, // data_class: INTERNAL_ONLY (opaque typed ref)
    pub durable_state_externalized: bool,
    pub stores_prompt_or_completion_body: bool,
}

impl AgentMemoryBindingSpec {
    pub fn new(tenant_id: &str, name: &str, memory_ref: &str) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        validate_typed_reference(memory_ref, "memory-ref://")?;
        Ok(Self {
            kind: "AgentMemoryBinding",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            memory_ref: memory_ref.to_string(),
            durable_state_externalized: true,
            stores_prompt_or_completion_body: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentWorkspaceBindingSpec {
    pub kind: &'static str,
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub name: String,          // data_class: INTERNAL_ONLY
    pub workspace_ref: String, // data_class: INTERNAL_ONLY (opaque typed ref)
    pub durable_state_externalized: bool,
    pub mounts_host_paths: bool,
}

impl AgentWorkspaceBindingSpec {
    pub fn new(
        tenant_id: &str,
        name: &str,
        workspace_ref: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        validate_typed_reference(workspace_ref, "workspace-ref://")?;
        Ok(Self {
            kind: "AgentWorkspaceBinding",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            workspace_ref: workspace_ref.to_string(),
            durable_state_externalized: true,
            mounts_host_paths: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentSkillBundleSpec {
    pub kind: &'static str,
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub name: String,                           // data_class: INTERNAL_ONLY
    pub skillbundle_ref: String,                // data_class: INTERNAL_ONLY (opaque typed ref)
    pub tool_compatibility_profile_ref: String, // data_class: INTERNAL_ONLY
    pub policy_gated: bool,
    pub installs_local_hooks: bool,
}

impl AgentSkillBundleSpec {
    pub fn new(
        tenant_id: &str,
        name: &str,
        skillbundle_ref: &str,
        tool_compatibility_profile_ref: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        validate_resource_name(tool_compatibility_profile_ref)?;
        validate_typed_reference(skillbundle_ref, "skillbundle-ref://")?;
        Ok(Self {
            kind: "AgentSkillBundle",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            skillbundle_ref: skillbundle_ref.to_string(),
            tool_compatibility_profile_ref: tool_compatibility_profile_ref.to_string(),
            policy_gated: true,
            installs_local_hooks: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentScheduleSpec {
    pub kind: &'static str,
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub name: String,                // data_class: INTERNAL_ONLY
    pub schedule_ref: String,        // data_class: INTERNAL_ONLY (opaque typed ref)
    pub runtime_profile_ref: String, // data_class: INTERNAL_ONLY
    pub execution_externalized_to_controller: bool,
    pub embeds_local_cron: bool,
}

impl AgentScheduleSpec {
    pub fn new(
        tenant_id: &str,
        name: &str,
        schedule_ref: &str,
        runtime_profile_ref: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        validate_resource_name(runtime_profile_ref)?;
        validate_typed_reference(schedule_ref, "schedule-ref://")?;
        Ok(Self {
            kind: "AgentSchedule",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            schedule_ref: schedule_ref.to_string(),
            runtime_profile_ref: runtime_profile_ref.to_string(),
            execution_externalized_to_controller: true,
            embeds_local_cron: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentDelegationPolicySpec {
    pub kind: &'static str,
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub name: String,                             // data_class: INTERNAL_ONLY
    pub allowed_generation_adapters: Vec<String>, // data_class: INTERNAL_ONLY
    pub policy_engine_port: String,               // data_class: INTERNAL_ONLY
    pub policy_gated: bool,
    pub allows_routing_advisor_generation: bool,
}

impl AgentDelegationPolicySpec {
    pub fn new(
        tenant_id: &str,
        name: &str,
        allowed_generation_adapters: &[&str],
        policy_engine_port: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if policy_engine_port != "owned-policy-engine-port" {
            return Err(WorkerConfigError::InvalidPolicyValue);
        }
        let mut adapters = allowed_generation_adapters
            .iter()
            .copied()
            .map(str::to_string)
            .collect::<Vec<_>>();
        adapters.sort();
        adapters.dedup();
        if adapters.is_empty()
            || adapters
                .iter()
                .any(|adapter| !matches!(adapter.as_str(), "claude" | "codex" | "gemini"))
        {
            return Err(WorkerConfigError::InvalidAdapterSet);
        }
        Ok(Self {
            kind: "AgentDelegationPolicy",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            allowed_generation_adapters: adapters,
            policy_engine_port: policy_engine_port.to_string(),
            policy_gated: true,
            allows_routing_advisor_generation: false,
        })
    }
}

/// Internal coding-agent workflow composed exclusively from intelligence-app
/// resource refs. Status ownership for the `/admin/v1/agent-runtimes` and
/// `llm.agent_runtime.v1` read surfaces; execution stays controller-owned.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalCodingAgentWorkflowPlan {
    pub kind: &'static str,
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub name: String,                           // data_class: INTERNAL_ONLY
    pub runtime_profile_ref: String,            // data_class: INTERNAL_ONLY
    pub schedule_name: String,                  // data_class: INTERNAL_ONLY
    pub schedule_ref: String,                   // data_class: INTERNAL_ONLY
    pub delegation_policy_ref: String,          // data_class: INTERNAL_ONLY
    pub guardrail_profile_ref: String,          // data_class: INTERNAL_ONLY
    pub evidence_retention_profile_ref: String, // data_class: INTERNAL_ONLY
    pub redaction_profile_ref: String,          // data_class: INTERNAL_ONLY
    pub generation_adapters: Vec<String>,       // data_class: INTERNAL_ONLY
    pub routing_advisor_scope: &'static str,    // data_class: INTERNAL_ONLY
    pub policy_engine_port: &'static str,       // data_class: INTERNAL_ONLY
    pub evidence_visibility: &'static str,      // data_class: INTERNAL_ONLY
    pub intelligence_app_primitive_only: bool,
    pub requires_policy_engine_decision: bool,
    pub requires_secondary_review_for_critical_blocks: bool,
    pub uses_redacted_evidence_handles: bool,
    pub embeds_product_workflow: bool,
    pub installs_cli_or_tui_surface: bool,
    pub stores_raw_prompt_or_completion: bool,
}

impl InternalCodingAgentWorkflowPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn dogfood_default(
        tenant_id: &str,
        name: &str,
        runtime_profile_ref: &str,
        schedule_name: &str,
        delegation_policy_ref: &str,
        guardrail_profile_ref: &str,
        evidence_retention_profile_ref: &str,
        redaction_profile_ref: &str,
    ) -> Result<Self, WorkerConfigError> {
        for resource_name in [
            name,
            runtime_profile_ref,
            schedule_name,
            delegation_policy_ref,
            guardrail_profile_ref,
            evidence_retention_profile_ref,
            redaction_profile_ref,
        ] {
            validate_resource_name(resource_name)?;
        }

        Ok(Self {
            kind: "AgentWorkflowPlan",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            runtime_profile_ref: runtime_profile_ref.to_string(),
            schedule_name: schedule_name.to_string(),
            schedule_ref: format!("schedule-ref://{tenant_id}/{schedule_name}"),
            delegation_policy_ref: delegation_policy_ref.to_string(),
            guardrail_profile_ref: guardrail_profile_ref.to_string(),
            evidence_retention_profile_ref: evidence_retention_profile_ref.to_string(),
            redaction_profile_ref: redaction_profile_ref.to_string(),
            generation_adapters: vec![
                "claude".to_string(),
                "codex".to_string(),
                "gemini".to_string(),
            ],
            routing_advisor_scope: "routing-decision-only",
            policy_engine_port: "owned-policy-engine-port",
            evidence_visibility: "redacted-structured-evidence",
            intelligence_app_primitive_only: true,
            requires_policy_engine_decision: true,
            requires_secondary_review_for_critical_blocks: true,
            uses_redacted_evidence_handles: true,
            embeds_product_workflow: false,
            installs_cli_or_tui_surface: false,
            stores_raw_prompt_or_completion: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GuardrailDetectionProfileSpec {
    pub kind: &'static str,
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub name: String,                     // data_class: INTERNAL_ONLY
    pub policy_engine_port: String,       // data_class: INTERNAL_ONLY
    pub critical_categories: Vec<String>, // data_class: INTERNAL_ONLY
    pub automatic_block_and_quarantine: bool,
    pub mandatory_secondary_agentic_review: bool,
    pub manual_review_required_after_secondary_review: bool,
    pub tenant_may_weaken_platform_floor: bool,
}

impl GuardrailDetectionProfileSpec {
    pub fn platform_default(
        tenant_id: &str,
        name: &str,
        policy_engine_port: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if policy_engine_port != "owned-policy-engine-port" {
            return Err(WorkerConfigError::InvalidPolicyValue);
        }
        Ok(Self {
            kind: "GuardrailDetectionProfile",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            policy_engine_port: policy_engine_port.to_string(),
            critical_categories: [
                "prompt-injection-or-jailbreak",
                "data-exfiltration-or-breach",
                "credential-or-secret-probe",
                "sandbox-escape-or-destructive-action",
                "self-harm-or-harm-to-others",
                "privacy-violation",
                "tenant-boundary-violation",
                "fraud-or-hostile-pattern",
                "fault-or-anomaly",
                "unsafe-scheduled-or-delegated-execution",
                "child-safety-or-abuse",
                "security-exploit-or-breach",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            automatic_block_and_quarantine: true,
            mandatory_secondary_agentic_review: true,
            manual_review_required_after_secondary_review: true,
            tenant_may_weaken_platform_floor: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRetentionProfileSpec {
    pub kind: &'static str,
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub name: String,                 // data_class: INTERNAL_ONLY
    pub secret_provider_port: String, // data_class: INTERNAL_ONLY
    pub stores_raw_payload_on_normal_path: bool,
    pub encrypted_handle_on_guardrail_trigger: bool,
    pub fixed_ttl_by_data_class: bool,
    pub regulatory_classification_required: bool,
    pub default_reviewer_visibility: String, // data_class: INTERNAL_ONLY
    pub raw_access_requires_audited_break_glass: bool,
}

impl EvidenceRetentionProfileSpec {
    pub fn platform_default(
        tenant_id: &str,
        name: &str,
        secret_provider_port: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if secret_provider_port != "owned-secret-provider-port" {
            return Err(WorkerConfigError::InvalidPolicyValue);
        }
        Ok(Self {
            kind: "EvidenceRetentionProfile",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            secret_provider_port: secret_provider_port.to_string(),
            stores_raw_payload_on_normal_path: false,
            encrypted_handle_on_guardrail_trigger: true,
            fixed_ttl_by_data_class: true,
            regulatory_classification_required: true,
            default_reviewer_visibility: "redacted-structured-evidence".to_string(),
            raw_access_requires_audited_break_glass: true,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InTransitRedactionProfileSpec {
    pub kind: &'static str,
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub name: String,      // data_class: INTERNAL_ONLY
    pub blocks_sensitive_classes: bool,
    pub redacts_trivial_personal_data: bool,
    pub reversible_tokens_require_tenant_policy: bool,
    pub default_token_lifetime: String, // data_class: INTERNAL_ONLY
    pub restore_only_after_model_output: bool,
    pub provider_receives_raw_token_values: bool,
    pub routing_advisor_receives_raw_token_values: bool,
}

impl InTransitRedactionProfileSpec {
    pub fn platform_default(tenant_id: &str, name: &str) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        Ok(Self {
            kind: "InTransitRedactionProfile",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            blocks_sensitive_classes: true,
            redacts_trivial_personal_data: true,
            reversible_tokens_require_tenant_policy: true,
            default_token_lifetime: "ephemeral-run".to_string(),
            restore_only_after_model_output: true,
            provider_receives_raw_token_values: false,
            routing_advisor_receives_raw_token_values: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManualReviewEscalationSpec {
    pub kind: &'static str,
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub name: String,      // data_class: INTERNAL_ONLY
    pub required_for_critical_blocks: bool,
    pub secondary_agentic_review_must_run_first: bool,
    pub default_evidence_visibility: String, // data_class: INTERNAL_ONLY
    pub raw_payload_break_glass_only: bool,
}

impl ManualReviewEscalationSpec {
    pub fn platform_default(tenant_id: &str, name: &str) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        Ok(Self {
            kind: "ManualReviewEscalation",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            required_for_critical_blocks: true,
            secondary_agentic_review_must_run_first: true,
            default_evidence_visibility: "redacted-structured-evidence".to_string(),
            raw_payload_break_glass_only: true,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SafetySignalPolicySpec {
    pub kind: &'static str,
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub name: String,      // data_class: INTERNAL_ONLY
    pub platform_automatic_enforcement: bool,
    pub tenant_policy_receives_signals: bool,
    pub tenant_policy_receives_recommendations: bool,
    pub tenant_can_override_platform_critical_block: bool,
}

impl SafetySignalPolicySpec {
    pub fn platform_default(tenant_id: &str, name: &str) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        Ok(Self {
            kind: "SafetySignalPolicy",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            platform_automatic_enforcement: true,
            tenant_policy_receives_signals: true,
            tenant_policy_receives_recommendations: true,
            tenant_can_override_platform_critical_block: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReferenceCiPatternCatalog {
    pub adopted_patterns: Vec<String>,
    pub rejected_patterns: Vec<String>,
}

impl ReferenceCiPatternCatalog {
    pub fn cloud_native_adoptions() -> Self {
        Self {
            adopted_patterns: [
                "path-scoped-compatibility-canaries",
                "drift-detection-with-artifacted-reports",
                "watcher-liveness-watchdog",
                "infra-vs-drift-vs-inconclusive-status-separation",
                "self-healing-pr-or-task-on-delta",
                "redaction-and-wire-fixture-regression-matrix",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            rejected_patterns: ["local-cli-smoke-surface", "local-tui-test-surface"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoutingAdvisorPurpose {
    RoutingDecisionOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoutingAdvisorProfile {
    pub provider_label: &'static str,   // data_class: INTERNAL_ONLY
    pub model_hint: &'static str,       // data_class: INTERNAL_ONLY
    pub purpose: RoutingAdvisorPurpose, // data_class: INTERNAL_ONLY
    pub adapter_backed: bool,           // data_class: INTERNAL_ONLY
    pub may_execute_generation: bool,   // data_class: INTERNAL_ONLY
    pub receives_redacted_route_metadata: bool, // data_class: INTERNAL_ONLY
    pub receives_raw_prompts_or_secrets: bool, // data_class: INTERNAL_ONLY
}

pub fn default_routing_advisor_profiles() -> Vec<RoutingAdvisorProfile> {
    vec![
        RoutingAdvisorProfile {
            provider_label: "openai-compatible",
            model_hint: "chatgpt-spark",
            purpose: RoutingAdvisorPurpose::RoutingDecisionOnly,
            adapter_backed: true,
            may_execute_generation: false,
            receives_redacted_route_metadata: true,
            receives_raw_prompts_or_secrets: false,
        },
        RoutingAdvisorProfile {
            provider_label: "gemini-native",
            model_hint: "gemini-3.1-flash-lite",
            purpose: RoutingAdvisorPurpose::RoutingDecisionOnly,
            adapter_backed: true,
            may_execute_generation: false,
            receives_redacted_route_metadata: true,
            receives_raw_prompts_or_secrets: false,
        },
        RoutingAdvisorProfile {
            provider_label: "external-free-frontier",
            model_hint: "nemotron-3-ultra-550b-a55b",
            purpose: RoutingAdvisorPurpose::RoutingDecisionOnly,
            adapter_backed: true,
            may_execute_generation: false,
            receives_redacted_route_metadata: true,
            receives_raw_prompts_or_secrets: false,
        },
    ]
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackendRegistry {
    backends: Vec<ProviderBackendSpec>, // data_class: INTERNAL_ONLY
}

impl BackendRegistry {
    pub fn from_specs(backends: Vec<ProviderBackendSpec>) -> Result<Self, WorkerConfigError> {
        if backends.is_empty() {
            return Err(WorkerConfigError::EmptyRegistry);
        }
        Ok(Self { backends })
    }

    pub fn len(&self) -> usize {
        self.backends.len()
    }

    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    pub fn weighted_fallback_order(&self) -> Vec<String> {
        let mut ordered = self.backends.clone();
        ordered.sort_by(|left, right| {
            right
                .weight
                .cmp(&left.weight)
                .then_with(|| left.name.cmp(&right.name))
        });
        ordered.into_iter().map(|backend| backend.name).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRouteSpec {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub route_name: String,                // data_class: INTERNAL_ONLY
    pub backend_name: String,              // data_class: INTERNAL_ONLY
    pub model_override: String,            // data_class: INTERNAL_ONLY
    pub normalized_effort: Option<String>, // data_class: INTERNAL_ONLY
    pub max_output_tokens: Option<u32>,    // data_class: INTERNAL_ONLY
    pub policy_authorized: bool,           // data_class: INTERNAL_ONLY
}

impl ModelRouteSpec {
    pub fn policy_authorized_override(
        tenant_id: &str,
        route_name: &str,
        backend_name: &str,
        model_override: &str,
        effort: &str,
        max_output_tokens: u32,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(route_name)?;
        validate_resource_name(backend_name)?;
        let normalized_effort = match effort {
            "low" | "medium" | "high" => Some(effort.to_string()),
            _ => return Err(WorkerConfigError::InvalidPolicyValue),
        };
        Ok(Self {
            tenant_id: tenant_id.to_string(),
            route_name: route_name.to_string(),
            backend_name: backend_name.to_string(),
            model_override: model_override.to_string(),
            normalized_effort,
            max_output_tokens: Some(max_output_tokens),
            policy_authorized: true,
        })
    }

    pub fn unauthorized_override(
        tenant_id: &str,
        route_name: &str,
        backend_name: &str,
        model_override: &str,
    ) -> Result<Self, WorkerConfigError> {
        let _ = (tenant_id, route_name, backend_name, model_override);
        Err(WorkerConfigError::UnauthorizedRouteMutation)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OAuthLifecyclePlan {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub provider_class: ProviderClass, // data_class: INTERNAL_ONLY
    pub refresh_token_handle: String,  // data_class: SECRET_REFERENCE
    pub worker_safe: bool,             // data_class: INTERNAL_ONLY
    pub uses_browser_automation: bool, // data_class: INTERNAL_ONLY
}

impl OAuthLifecyclePlan {
    pub fn manual_headless_enrollment(
        tenant_id: &str,
        provider_class: ProviderClass,
        refresh_token_handle: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_secret_reference(refresh_token_handle)?;
        Ok(Self {
            tenant_id: tenant_id.to_string(),
            provider_class,
            refresh_token_handle: refresh_token_handle.to_string(),
            worker_safe: true,
            uses_browser_automation: false,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoolActivation {
    SingleSeat,
    MultiSeatActive,
}

impl PoolActivation {
    pub const fn from_seat_count(seat_count: usize) -> Self {
        if seat_count >= 2 {
            Self::MultiSeatActive
        } else {
            Self::SingleSeat
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CredentialRefreshPlan {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub provider_class: ProviderClass,  // data_class: INTERNAL_ONLY
    pub secret_handle: String,          // data_class: SECRET_REFERENCE
    pub singleflight_group_key: String, // data_class: INTERNAL_ONLY
    pub stores_plaintext_secret: bool,  // data_class: INTERNAL_ONLY
}

impl CredentialRefreshPlan {
    pub fn singleflight(
        tenant_id: &str,
        provider_class: ProviderClass,
        secret_handle: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_secret_reference(secret_handle)?;
        Ok(Self {
            tenant_id: tenant_id.to_string(),
            provider_class,
            secret_handle: secret_handle.to_string(),
            singleflight_group_key: format!("{tenant_id}:{}", provider_class.slug()),
            stores_plaintext_secret: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudAuthRequirements {
    pub requires_tenant_authn: bool,
    pub requires_policy_engine_decision: bool,
    pub requires_mtls_or_api_key_at_edge: bool,
    pub cors_requires_policy_review: bool,
    pub upstream_proxy_requires_policy_review: bool,
}

impl CloudAuthRequirements {
    pub const fn non_loopback_default() -> Self {
        Self {
            requires_tenant_authn: true,
            requires_policy_engine_decision: true,
            requires_mtls_or_api_key_at_edge: true,
            cors_requires_policy_review: true,
            upstream_proxy_requires_policy_review: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ConfigSource {
    ServiceDefault = 0,
    TenantDefault = 10,
    ModelRoute = 20,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigLayer {
    pub source: ConfigSource, // data_class: INTERNAL_ONLY
    pub key: String,          // data_class: INTERNAL_ONLY
    pub value: String,        // data_class: INTERNAL_ONLY
}

impl ConfigLayer {
    pub fn new(source: ConfigSource, key: &str, value: &str) -> Self {
        Self {
            source,
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn validate_no_raw_secret(&self) -> Result<(), WorkerConfigError> {
        if self.key.to_ascii_lowercase().contains("key")
            && !(self.value.starts_with("secret-ref://") || self.value.starts_with("kms-ref://"))
        {
            return Err(WorkerConfigError::RawSecretValue);
        }
        Ok(())
    }
}

pub fn resolve_config_precedence(
    layers: impl IntoIterator<Item = ConfigLayer>,
) -> Result<BTreeMap<String, String>, WorkerConfigError> {
    let mut ordered: Vec<_> = layers.into_iter().collect();
    for layer in &ordered {
        layer.validate_no_raw_secret()?;
    }
    ordered.sort_by_key(|layer| layer.source);
    let mut resolved = BTreeMap::new();
    for layer in ordered {
        resolved.insert(layer.key, layer.value);
    }
    Ok(resolved)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DriftParityPlan {
    pub artifact_family: String,         // data_class: INTERNAL_ONLY
    pub commit_sha: String,              // data_class: INTERNAL_ONLY
    pub kind: &'static str,              // data_class: INTERNAL_ONLY
    pub probes: Vec<String>,             // data_class: INTERNAL_ONLY
    pub audit_event_required: bool,      // data_class: INTERNAL_ONLY
    pub opens_pr_or_task_on_delta: bool, // data_class: INTERNAL_ONLY
}

impl DriftParityPlan {
    pub fn for_pinned_baseline(artifact_family: &str, commit_sha: &str) -> Self {
        Self {
            artifact_family: artifact_family.to_string(),
            commit_sha: commit_sha.to_string(),
            kind: "CapabilityParityBaseline",
            probes: vec![
                "wire-profile-drift".to_string(),
                "package-baseline-drift".to_string(),
                "compatibility-canary-drift".to_string(),
            ],
            audit_event_required: true,
            opens_pr_or_task_on_delta: true,
        }
    }

    pub fn compatibility_canaries(&self) -> Vec<String> {
        vec![
            "route-matrix".to_string(),
            "streaming-fixtures".to_string(),
            "pool-failover".to_string(),
            "security-redaction".to_string(),
        ]
    }
}

/// Scheduled, controller-owned parity/drift canary plan backing the
/// `/admin/v1/parity/canaries` and `llm.parity_canary.v1` status surfaces.
/// Deltas open governed work items; the plan never embeds a local cron runner.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScheduledParityDriftCanaryPlan {
    pub kind: &'static str,
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub name: String,                        // data_class: INTERNAL_ONLY
    pub schedule_ref: String,                // data_class: INTERNAL_ONLY
    pub artifact_family: String,             // data_class: INTERNAL_ONLY
    pub baseline_commit_sha: String,         // data_class: INTERNAL_ONLY
    pub probes: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub compatibility_canaries: Vec<String>, // data_class: INTERNAL_ONLY
    pub controller_owned: bool,
    pub opens_pr_or_task_on_delta: bool,
    pub audit_event_required: bool,
    pub embeds_local_cron: bool,
    pub writes_raw_prompts_or_secrets: bool,
}

impl ScheduledParityDriftCanaryPlan {
    pub fn for_internal_coding_agent(
        tenant_id: &str,
        name: &str,
        artifact_family: &str,
        baseline_commit_sha: &str,
    ) -> Result<Self, WorkerConfigError> {
        validate_resource_name(name)?;
        if artifact_family != "external-proxy-reference" || baseline_commit_sha.len() != 40 {
            return Err(WorkerConfigError::InvalidPolicyValue);
        }
        Ok(Self {
            kind: "ScheduledParityDriftCanaryPlan",
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            schedule_ref: format!("schedule-ref://{tenant_id}/{name}"),
            artifact_family: artifact_family.to_string(),
            baseline_commit_sha: baseline_commit_sha.to_string(),
            probes: vec![
                "capability-parity".to_string(),
                "wire-profile-drift".to_string(),
                "adapter-translation-drift".to_string(),
            ],
            compatibility_canaries: vec![
                "route-matrix".to_string(),
                "streaming-fixtures".to_string(),
                "security-redaction".to_string(),
            ],
            controller_owned: true,
            opens_pr_or_task_on_delta: true,
            audit_event_required: true,
            embeds_local_cron: false,
            writes_raw_prompts_or_secrets: false,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParityCanaryStatusState {
    Passed,
    Failed,
    Running,
    Inconclusive,
}

/// Redacted status projection for a [`ScheduledParityDriftCanaryPlan`]. The
/// worker owns emission; REST/proto/AsyncAPI surfaces only relay it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParityCanaryStatusSpec {
    pub kind: &'static str,
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub plan_ref: String,                           // data_class: INTERNAL_ONLY
    pub state: ParityCanaryStatusState,             // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u32>,           // data_class: INTERNAL_ONLY
    pub evidence_visibility: &'static str,          // data_class: INTERNAL_ONLY
    pub sealed_evidence_handle_ref: Option<String>, // data_class: SECRET_REFERENCE
    pub raw_payload_included: bool,
}

impl ParityCanaryStatusSpec {
    pub fn from_plan(
        plan: &ScheduledParityDriftCanaryPlan,
        state: ParityCanaryStatusState,
    ) -> Self {
        Self {
            kind: "ParityCanaryStatus",
            tenant_id: plan.tenant_id.clone(),
            plan_ref: format!("parity-canary-plan-ref://{}/{}", plan.tenant_id, plan.name),
            state,
            retry_after_seconds: match state {
                ParityCanaryStatusState::Passed => None,
                ParityCanaryStatusState::Failed
                | ParityCanaryStatusState::Running
                | ParityCanaryStatusState::Inconclusive => Some(300),
            },
            evidence_visibility: "redacted-structured-evidence",
            sealed_evidence_handle_ref: Some(format!(
                "sealed-evidence-ref://{}/{}/parity-canary",
                plan.tenant_id, plan.name
            )),
            raw_payload_included: false,
        }
    }
}

fn validate_resource_name(name: &str) -> Result<(), WorkerConfigError> {
    if name.is_empty()
        || name.len() > 63
        || name.starts_with('-')
        || name.ends_with('-')
        || !name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(WorkerConfigError::InvalidName);
    }
    Ok(())
}

fn validate_typed_reference(value: &str, required_prefix: &str) -> Result<(), WorkerConfigError> {
    if value.starts_with(required_prefix) && value.len() > required_prefix.len() {
        Ok(())
    } else {
        Err(WorkerConfigError::InvalidTypedReference)
    }
}

fn validate_secret_reference(value: &str) -> Result<(), WorkerConfigError> {
    if value.starts_with("secret-ref://") || value.starts_with("kms-ref://") {
        Ok(())
    } else {
        Err(WorkerConfigError::InvalidSecretReference)
    }
}
