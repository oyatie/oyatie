//! Deterministic model-routing value kernel for the Intelligence substrate.
//!
//! The kernel evaluates an already-validated request against a catalog of
//! provider route profiles. It has no network, credential, clock, or storage
//! dependency; callers provide the catalog snapshot and persist any decision
//! evidence outside this crate.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelProvider {
    Anthropic,
    AzureOpenAi,
    Gemini,
    Local,
    OpenAi,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelCapability {
    ChatCompletion,
    Embedding,
    JsonMode,
    ToolUse,
    Vision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CredentialMode {
    BringYourOwnKey,
    Disabled,
    PlatformManaged,
    TenantScoped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntelligenceDataClass {
    BehavioralTenantProduct,
    InternalOnly,
    Phi,
    PiiIdentifying,
    Public,
    SearchQuery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RequestAudience {
    ExternalEndUser,
    InternalAutomation,
    TenantOperator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRouteRequest {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub capability: ModelCapability,       // data_class: PUBLIC
    pub credential_mode: CredentialMode,   // data_class: INTERNAL_ONLY
    pub data_class: IntelligenceDataClass, // data_class: INTERNAL_ONLY
    pub audience: RequestAudience,         // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRouteProfile {
    pub provider: ModelProvider,                    // data_class: PUBLIC
    pub model_id: String,                           // data_class: INTERNAL_ONLY
    pub enabled: bool,                              // data_class: INTERNAL_ONLY
    pub priority: u16,                              // data_class: INTERNAL_ONLY
    pub capabilities: BTreeSet<ModelCapability>,    // data_class: PUBLIC
    pub credential_modes: BTreeSet<CredentialMode>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: BTreeSet<IntelligenceDataClass>, // data_class: INTERNAL_ONLY
    pub allowed_audiences: BTreeSet<RequestAudience>, // data_class: INTERNAL_ONLY
    pub allowed_tenants: BTreeSet<String>,          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteSelection {
    pub provider: ModelProvider,         // data_class: PUBLIC
    pub model_id: String,                // data_class: INTERNAL_ONLY
    pub credential_mode: CredentialMode, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RouteDenialReason {
    NoEnabledProvider,
    CapabilityUnavailable,
    CredentialModeUnavailable,
    DataClassNotAllowed,
    AudienceNotAllowed,
    TenantNotAllowed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteDenial {
    pub reasons: BTreeSet<RouteDenialReason>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteDecision {
    Allow(RouteSelection),
    Deny(RouteDenial),
}

pub fn decide_route(
    request: &ModelRouteRequest,
    catalog: &[ProviderRouteProfile],
) -> RouteDecision {
    let mut denial_reasons = BTreeSet::new();
    let mut denial_evidence_refs = vec![request.request_evidence_ref.clone()];
    let mut candidates = Vec::new();

    for profile in catalog {
        denial_evidence_refs.extend(profile.evidence_refs.iter().cloned());

        let profile_denial_reasons = profile_denials(request, profile);
        if profile_denial_reasons.is_empty() {
            candidates.push(profile);
        } else {
            denial_reasons.extend(profile_denial_reasons);
        }
    }

    if let Some(profile) = select_highest_ranked_candidate(candidates) {
        return RouteDecision::Allow(selection_for_profile(request, profile));
    }

    if catalog.is_empty() || !catalog.iter().any(|profile| profile.enabled) {
        denial_reasons.insert(RouteDenialReason::NoEnabledProvider);
    }

    RouteDecision::Deny(RouteDenial {
        reasons: denial_reasons,
        evidence_refs: sorted_unique(denial_evidence_refs),
    })
}

fn profile_denials(
    request: &ModelRouteRequest,
    profile: &ProviderRouteProfile,
) -> BTreeSet<RouteDenialReason> {
    let mut reasons = BTreeSet::new();

    if !profile.enabled {
        reasons.insert(RouteDenialReason::NoEnabledProvider);
    }
    if !profile.capabilities.contains(&request.capability) {
        reasons.insert(RouteDenialReason::CapabilityUnavailable);
    }
    if !profile.credential_modes.contains(&request.credential_mode) {
        reasons.insert(RouteDenialReason::CredentialModeUnavailable);
    }
    if !profile.allowed_data_classes.contains(&request.data_class) {
        reasons.insert(RouteDenialReason::DataClassNotAllowed);
    }
    if !profile.allowed_audiences.contains(&request.audience) {
        reasons.insert(RouteDenialReason::AudienceNotAllowed);
    }
    if !profile.allowed_tenants.is_empty() && !profile.allowed_tenants.contains(&request.tenant_id)
    {
        reasons.insert(RouteDenialReason::TenantNotAllowed);
    }

    reasons
}

fn select_highest_ranked_candidate(
    mut candidates: Vec<&ProviderRouteProfile>,
) -> Option<&ProviderRouteProfile> {
    candidates.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.provider.cmp(&right.provider))
            .then_with(|| left.model_id.cmp(&right.model_id))
    });
    candidates.into_iter().next()
}

fn selection_for_profile(
    request: &ModelRouteRequest,
    profile: &ProviderRouteProfile,
) -> RouteSelection {
    let mut evidence_refs = vec![request.request_evidence_ref.clone()];
    evidence_refs.extend(profile.evidence_refs.iter().cloned());

    RouteSelection {
        provider: profile.provider,
        model_id: profile.model_id.clone(),
        credential_mode: request.credential_mode,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ModelRouteRequest {
        ModelRouteRequest {
            tenant_id: "ten_a".to_owned(),
            capability: ModelCapability::ToolUse,
            credential_mode: CredentialMode::TenantScoped,
            data_class: IntelligenceDataClass::InternalOnly,
            audience: RequestAudience::TenantOperator,
            request_evidence_ref: "req:42".to_owned(),
        }
    }

    fn profile(provider: ModelProvider, model_id: &str, priority: u16) -> ProviderRouteProfile {
        ProviderRouteProfile {
            provider,
            model_id: model_id.to_owned(),
            enabled: true,
            priority,
            capabilities: BTreeSet::from([ModelCapability::ToolUse]),
            credential_modes: BTreeSet::from([CredentialMode::TenantScoped]),
            allowed_data_classes: BTreeSet::from([IntelligenceDataClass::InternalOnly]),
            allowed_audiences: BTreeSet::from([RequestAudience::TenantOperator]),
            allowed_tenants: BTreeSet::new(),
            evidence_refs: vec![format!("catalog:{provider:?}")],
        }
    }

    #[test]
    fn selects_lowest_priority_candidate_deterministically_with_evidence() {
        let decision = decide_route(
            &request(),
            &[
                profile(ModelProvider::OpenAi, "gpt", 10),
                profile(ModelProvider::Anthropic, "claude", 1),
            ],
        );

        assert_eq!(
            decision,
            RouteDecision::Allow(RouteSelection {
                provider: ModelProvider::Anthropic,
                model_id: "claude".to_owned(),
                credential_mode: CredentialMode::TenantScoped,
                evidence_refs: vec!["catalog:Anthropic".to_owned(), "req:42".to_owned()],
            })
        );
    }

    #[test]
    fn denies_when_no_profile_supports_required_audience() {
        let mut blocked_profile = profile(ModelProvider::OpenAi, "gpt", 1);
        blocked_profile.allowed_audiences = BTreeSet::from([RequestAudience::InternalAutomation]);

        let decision = decide_route(&request(), &[blocked_profile]);

        assert_eq!(
            decision,
            RouteDecision::Deny(RouteDenial {
                reasons: BTreeSet::from([RouteDenialReason::AudienceNotAllowed]),
                evidence_refs: vec!["catalog:OpenAi".to_owned(), "req:42".to_owned()],
            })
        );
    }
}
