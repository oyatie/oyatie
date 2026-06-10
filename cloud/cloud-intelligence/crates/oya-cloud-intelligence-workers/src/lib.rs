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
            name: "cloud-intelligence-gateway",
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
}

impl ProviderClass {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::AnthropicSubscription => "anthropic-subscription",
            Self::OpenAiCompatible => "openai-compatible",
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

fn validate_secret_reference(value: &str) -> Result<(), WorkerConfigError> {
    if value.starts_with("secret-ref://") || value.starts_with("kms-ref://") {
        Ok(())
    } else {
        Err(WorkerConfigError::InvalidSecretReference)
    }
}
