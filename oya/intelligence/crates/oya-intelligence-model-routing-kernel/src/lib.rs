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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EnvTier {
    Test,
    Staging,
    Prod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelDefaultClass {
    SmallCheap,
    PromotionCandidate,
    ProductionGradeSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelProfileTag {
    CheapOrSmall,
    SandboxOk,
    NonProdOnly,
    StagingApproved,
    EvalSnapshotBound,
    ProductionGrade,
    ProdApproved,
    SloBacked,
    EvalGatePassed,
    ProductionGradeOnly,
    ProdOnly,
    CheapOrSmallOnly,
    SandboxOnly,
    ProdOnlyWithoutPromotionEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRouteRequest {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub env_tier: Option<EnvTier>,                     // data_class: INTERNAL_ONLY
    pub model_default_policy_ref: String,              // data_class: INTERNAL_ONLY
    pub tier_cost_budget_policy_ref: String,           // data_class: INTERNAL_ONLY
    pub tier_cost_budget_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub model_route_registry_snapshot_ref: String,     // data_class: INTERNAL_ONLY
    pub capability: ModelCapability,                   // data_class: PUBLIC
    pub credential_mode: CredentialMode,               // data_class: INTERNAL_ONLY
    pub data_class: IntelligenceDataClass,             // data_class: INTERNAL_ONLY
    pub audience: RequestAudience,                     // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRouteProfile {
    pub provider: ModelProvider,                    // data_class: PUBLIC
    pub model_id: String,                           // data_class: INTERNAL_ONLY
    pub model_default_class: ModelDefaultClass,     // data_class: INTERNAL_ONLY
    pub profile_tags: BTreeSet<ModelProfileTag>,    // data_class: INTERNAL_ONLY
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
    EnvTierRequired,
    TierModelDefaultMismatch,
    TierCostBudgetEvidenceMissing,
    FoundryLiveAuthorityResurrection,
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
    let mut denial_evidence_refs = request_contract_evidence_refs(request);
    let mut candidates = Vec::new();

    for profile in catalog {
        denial_evidence_refs.extend(profile.evidence_refs.iter().cloned());

        let profile_denial_reasons = profile_denials(request, profile);
        if profile_denial_reasons.is_empty() {
            candidates.push(profile);
        } else {
            denial_evidence_refs.extend(evidence_refs_for_denial_reasons(&profile_denial_reasons));
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
    if request.env_tier.is_none() {
        reasons.insert(RouteDenialReason::EnvTierRequired);
    }
    if missing_tier_cost_budget_evidence(request) {
        reasons.insert(RouteDenialReason::TierCostBudgetEvidenceMissing);
    }
    if contains_retired_foundry_live_authority_ref(request) {
        reasons.insert(RouteDenialReason::FoundryLiveAuthorityResurrection);
    }
    if let Some(env_tier) = request.env_tier
        && !profile_satisfies_env_tier(env_tier, profile)
    {
        reasons.insert(RouteDenialReason::TierModelDefaultMismatch);
    }

    reasons
}

fn profile_satisfies_env_tier(env_tier: EnvTier, profile: &ProviderRouteProfile) -> bool {
    match env_tier {
        EnvTier::Test => {
            profile.model_default_class == ModelDefaultClass::SmallCheap
                && has_required_tags(
                    &profile.profile_tags,
                    &[
                        ModelProfileTag::CheapOrSmall,
                        ModelProfileTag::SandboxOk,
                        ModelProfileTag::NonProdOnly,
                    ],
                )
                && has_no_forbidden_tags(
                    &profile.profile_tags,
                    &[
                        ModelProfileTag::ProductionGrade,
                        ModelProfileTag::ProductionGradeOnly,
                        ModelProfileTag::ProdApproved,
                        ModelProfileTag::ProdOnly,
                    ],
                )
        }
        EnvTier::Staging => {
            profile.model_default_class == ModelDefaultClass::PromotionCandidate
                && has_required_tags(
                    &profile.profile_tags,
                    &[
                        ModelProfileTag::StagingApproved,
                        ModelProfileTag::EvalSnapshotBound,
                    ],
                )
                && has_no_forbidden_tags(
                    &profile.profile_tags,
                    &[ModelProfileTag::ProdOnlyWithoutPromotionEvidence],
                )
        }
        EnvTier::Prod => {
            profile.model_default_class == ModelDefaultClass::ProductionGradeSelection
                && has_required_tags(
                    &profile.profile_tags,
                    &[
                        ModelProfileTag::ProductionGrade,
                        ModelProfileTag::ProdApproved,
                        ModelProfileTag::SloBacked,
                        ModelProfileTag::EvalGatePassed,
                    ],
                )
                && has_no_forbidden_tags(
                    &profile.profile_tags,
                    &[
                        ModelProfileTag::CheapOrSmallOnly,
                        ModelProfileTag::SandboxOnly,
                        ModelProfileTag::NonProdOnly,
                    ],
                )
        }
    }
}

fn has_required_tags(tags: &BTreeSet<ModelProfileTag>, required: &[ModelProfileTag]) -> bool {
    required.iter().all(|tag| tags.contains(tag))
}

fn has_no_forbidden_tags(tags: &BTreeSet<ModelProfileTag>, forbidden: &[ModelProfileTag]) -> bool {
    forbidden.iter().all(|tag| !tags.contains(tag))
}

fn missing_tier_cost_budget_evidence(request: &ModelRouteRequest) -> bool {
    if request.tier_cost_budget_policy_ref.trim().is_empty() {
        return true;
    }
    match &request.tier_cost_budget_evidence_ref {
        Some(evidence_ref) => evidence_ref.trim().is_empty(),
        None => true,
    }
}

fn contains_retired_foundry_live_authority_ref(request: &ModelRouteRequest) -> bool {
    [
        request.model_default_policy_ref.as_str(),
        request.tier_cost_budget_policy_ref.as_str(),
        request.model_route_registry_snapshot_ref.as_str(),
        request
            .tier_cost_budget_evidence_ref
            .as_deref()
            .unwrap_or_default(),
    ]
    .into_iter()
    .any(is_retired_foundry_live_authority_ref)
}

fn is_retired_foundry_live_authority_ref(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("foundry:cost-budget.md")
        || lower.contains("foundry cost-budget.md")
        || lower.contains("specs/microservices/foundry.json#live-authority")
        || lower.contains("foundry.json#live-authority")
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
    let mut evidence_refs = request_contract_evidence_refs(request);
    evidence_refs.extend(profile.evidence_refs.iter().cloned());

    RouteSelection {
        provider: profile.provider,
        model_id: profile.model_id.clone(),
        credential_mode: request.credential_mode,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn request_contract_evidence_refs(request: &ModelRouteRequest) -> Vec<String> {
    let mut evidence_refs = vec![
        request.request_evidence_ref.clone(),
        request.model_default_policy_ref.clone(),
        request.tier_cost_budget_policy_ref.clone(),
        request.model_route_registry_snapshot_ref.clone(),
    ];
    if let Some(evidence_ref) = &request.tier_cost_budget_evidence_ref {
        evidence_refs.push(evidence_ref.clone());
    }
    evidence_refs
}

fn evidence_refs_for_denial_reasons(reasons: &BTreeSet<RouteDenialReason>) -> Vec<String> {
    let mut evidence_refs = Vec::new();
    for reason in reasons {
        match reason {
            RouteDenialReason::EnvTierRequired => {
                evidence_refs.push("env-tier:ENV-TIER-REQUIRED:missing_env_tier".to_owned());
            }
            RouteDenialReason::TierModelDefaultMismatch => {
                evidence_refs.push(
                    "env-tier:TIER-MODEL-DEFAULT-MATCH:wrong_model_default_for_tier".to_owned(),
                );
            }
            RouteDenialReason::TierCostBudgetEvidenceMissing => {
                evidence_refs.push(
                    "env-tier:TIER-BUDGET-EVIDENCE-REQUIRED:missing_per_tier_cost_budget_evidence"
                        .to_owned(),
                );
            }
            RouteDenialReason::FoundryLiveAuthorityResurrection => {
                evidence_refs.push(
                    "env-tier:FOUNDRY-LIVE-AUTHORITY-FORBIDDEN:foundry_live_authority_resurrection"
                        .to_owned(),
                );
            }
            RouteDenialReason::NoEnabledProvider
            | RouteDenialReason::CapabilityUnavailable
            | RouteDenialReason::CredentialModeUnavailable
            | RouteDenialReason::DataClassNotAllowed
            | RouteDenialReason::AudienceNotAllowed
            | RouteDenialReason::TenantNotAllowed => {}
        }
    }
    evidence_refs
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
            env_tier: Some(EnvTier::Test),
            model_default_policy_ref: "policy:intelligence.env-tier.model-default.test.v1"
                .to_owned(),
            tier_cost_budget_policy_ref: "policy:intelligence.env-tier.cost-budget.test.v1"
                .to_owned(),
            tier_cost_budget_evidence_ref: Some("budget:intelligence:test:unit".to_owned()),
            model_route_registry_snapshot_ref: "route-registry:intelligence:env-tier:test"
                .to_owned(),
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
            model_default_class: ModelDefaultClass::SmallCheap,
            profile_tags: BTreeSet::from([
                ModelProfileTag::CheapOrSmall,
                ModelProfileTag::SandboxOk,
                ModelProfileTag::NonProdOnly,
            ]),
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
                evidence_refs: vec![
                    "budget:intelligence:test:unit".to_owned(),
                    "catalog:Anthropic".to_owned(),
                    "policy:intelligence.env-tier.cost-budget.test.v1".to_owned(),
                    "policy:intelligence.env-tier.model-default.test.v1".to_owned(),
                    "req:42".to_owned(),
                    "route-registry:intelligence:env-tier:test".to_owned(),
                ],
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
                evidence_refs: vec![
                    "budget:intelligence:test:unit".to_owned(),
                    "catalog:OpenAi".to_owned(),
                    "policy:intelligence.env-tier.cost-budget.test.v1".to_owned(),
                    "policy:intelligence.env-tier.model-default.test.v1".to_owned(),
                    "req:42".to_owned(),
                    "route-registry:intelligence:env-tier:test".to_owned(),
                ],
            })
        );
    }
}
