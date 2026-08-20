use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireProfile {
    allowed_provider_headers: HashSet<String>,
}

impl WireProfile {
    pub fn openai_compatible_default() -> Self {
        Self {
            allowed_provider_headers: ["openai-organization", "openai-project"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }
}

pub fn filter_wire_headers(
    profile: &WireProfile,
    headers: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let connection_tokens: HashSet<String> = headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("connection"))
        .flat_map(|(_, value)| {
            value
                .split(',')
                .map(|token| token.trim().to_ascii_lowercase())
        })
        .collect();
    let hop_by_hop: HashSet<&str> = [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
        "upgrade",
    ]
    .into_iter()
    .collect();

    let mut filtered = BTreeMap::new();
    for (key, value) in headers {
        let lower = key.to_ascii_lowercase();
        if lower == "authorization" || lower == "host" || lower == "content-length" {
            continue;
        }
        if hop_by_hop.contains(lower.as_str()) || connection_tokens.contains(&lower) {
            continue;
        }
        if (lower.starts_with("openai-") || lower.starts_with("x-openai-"))
            && !profile.allowed_provider_headers.contains(&lower)
        {
            continue;
        }
        filtered.insert(lower, value.clone());
    }
    filtered
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromptProfile {
    NamedResource { name: String, body: String },
}

impl PromptProfile {
    pub fn named_resource(name: &str, body: &str) -> Result<Self, WirePolicyError> {
        if name.trim().is_empty() || body.trim().is_empty() {
            return Err(WirePolicyError::InvalidPromptProfile);
        }
        Ok(Self::NamedResource {
            name: name.to_string(),
            body: body.to_string(),
        })
    }

    pub fn cluster_file_path(_path: &str, _body: &str) -> Result<Self, WirePolicyError> {
        Err(WirePolicyError::ClusterFilePathRejected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WirePolicyError {
    InvalidPromptProfile,
    ClusterFilePathRejected,
}

pub fn apply_prompt_profile(
    profile: &PromptProfile,
    mut payload: serde_json::Value,
) -> Result<serde_json::Value, WirePolicyError> {
    match profile {
        PromptProfile::NamedResource { body, .. } => {
            payload["system"] = serde_json::Value::String(body.clone());
            Ok(payload)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThinkingPolicy {
    ProviderCompatibleDefault,
    ExplicitClientPassthrough,
}

impl ThinkingPolicy {
    pub fn provider_compatible_default() -> Self {
        Self::ProviderCompatibleDefault
    }

    pub fn explicit_client_passthrough() -> Self {
        Self::ExplicitClientPassthrough
    }
}

pub fn apply_thinking_policy(
    policy: &ThinkingPolicy,
    mut payload: serde_json::Value,
) -> serde_json::Value {
    if matches!(policy, ThinkingPolicy::ProviderCompatibleDefault)
        && let Some(object) = payload.as_object_mut()
    {
        object.remove("thinking");
    }
    payload
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShimSupersession {
    pub capability_id: &'static str,
    pub status: &'static str,
    pub implementation_target: &'static str,
}

impl ShimSupersession {
    pub fn cloud_gateway_supersedes_local_patch() -> Self {
        Self {
            capability_id: "XPROXY-COMPAT-006",
            status: "superseded",
            implementation_target: "cloud-gateway-adapter-worker-model",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WireCaptureBaseline {
    pub profile_kind: &'static str,
    pub provider_profile: String,
    pub signature_ref: String,
    pub redacted_summary: String,
}

impl WireCaptureBaseline {
    pub fn signed_provider_capture(
        provider_profile: &str,
        signature_ref: &str,
        raw_prompt_sample: &str,
    ) -> Result<Self, WirePolicyError> {
        if !signature_ref.starts_with("sha256:") {
            return Err(WirePolicyError::InvalidPromptProfile);
        }
        Ok(Self {
            profile_kind: "WireProfile",
            provider_profile: provider_profile.to_string(),
            signature_ref: signature_ref.to_string(),
            redacted_summary: format!(
                "provider_profile={provider_profile};sample_len={}",
                raw_prompt_sample.len()
            ),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DriftProbePlan {
    pub provider_profile: String,
    pub worker_owned: bool,
    pub audit_event_required: bool,
}

impl DriftProbePlan {
    pub fn from_capture(capture: &WireCaptureBaseline) -> Self {
        Self {
            provider_profile: capture.provider_profile.clone(),
            worker_owned: true,
            audit_event_required: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransportFingerprintPolicy {
    pub adapter_isolated: bool,
    pub strict_fingerprint_replay: bool,
    pub approved_adapter: Option<String>,
}

impl Default for TransportFingerprintPolicy {
    fn default() -> Self {
        Self {
            adapter_isolated: true,
            strict_fingerprint_replay: false,
            approved_adapter: None,
        }
    }
}

impl TransportFingerprintPolicy {
    pub fn approved_adapter_isolated(adapter_name: &str) -> Self {
        Self {
            adapter_isolated: true,
            strict_fingerprint_replay: true,
            approved_adapter: Some(adapter_name.to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PacingPolicy {
    pub enabled: bool,
    pub think_time_millis: u64,
    pub jitter_millis: u64,
    pub compliance_approved: bool,
}

impl PacingPolicy {
    pub const fn default_off() -> Self {
        Self {
            enabled: false,
            think_time_millis: 0,
            jitter_millis: 0,
            compliance_approved: false,
        }
    }

    pub const fn compliance_approved(think_time_millis: u64, jitter_millis: u64) -> Self {
        Self {
            enabled: true,
            think_time_millis,
            jitter_millis,
            compliance_approved: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StreamLifecyclePolicy {
    pub drain_to_eof_on_disconnect: bool,
    pub retry_after_first_byte_allowed: bool,
}

impl StreamLifecyclePolicy {
    pub const fn default_no_drain() -> Self {
        Self {
            drain_to_eof_on_disconnect: false,
            retry_after_first_byte_allowed: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionAffinityPolicy {
    pub idle_ttl_seconds: u64,
    pub max_age_seconds: u64,
    pub rotation_jitter_seconds: u64,
}

impl SessionAffinityPolicy {
    pub fn sticky_with_rotation(
        idle_ttl_seconds: u64,
        max_age_seconds: u64,
        rotation_jitter_seconds: u64,
    ) -> Result<Self, WirePolicyError> {
        if idle_ttl_seconds == 0
            || max_age_seconds < idle_ttl_seconds
            || rotation_jitter_seconds > max_age_seconds
        {
            return Err(WirePolicyError::InvalidPromptProfile);
        }
        Ok(Self {
            idle_ttl_seconds,
            max_age_seconds,
            rotation_jitter_seconds,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptCachePolicy {
    allowed_headers: HashSet<String>,
    pub requires_provider_adapter: bool,
}

impl PromptCachePolicy {
    pub fn provider_allowlist<const N: usize>(headers: [&str; N]) -> Self {
        Self {
            allowed_headers: headers
                .into_iter()
                .map(|header| header.to_ascii_lowercase())
                .collect(),
            requires_provider_adapter: true,
        }
    }

    pub fn allows_header(&self, header: &str) -> bool {
        self.allowed_headers.contains(&header.to_ascii_lowercase())
    }
}
