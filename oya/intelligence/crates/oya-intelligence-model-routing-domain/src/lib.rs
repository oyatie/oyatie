//! Domain validation for Intelligence model-routing requests.
//!
//! The domain layer fails closed before delegating to the deterministic kernel.
//! It validates tenant scoping, evidence presence, credential posture, and the
//! data-class/audience boundary that provider routing must not bypass.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub use oya_intelligence_model_routing_kernel::{
    CredentialMode, EnvTier, IntelligenceDataClass, ModelCapability, ModelDefaultClass,
    ModelProfileTag, ModelProvider, ModelRouteRequest, ProviderRouteProfile, RequestAudience,
    RouteDecision, RouteDenial, RouteDenialReason, RouteSelection, decide_route,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RequestValidationFailure {
    EmptyTenantId,
    InvalidTenantId,
    EmptyEvidenceRef,
    DisabledCredentialMode,
    ExternalAudienceSensitiveData,
    MissingEnvTier,
    MissingTierCostBudgetEvidence,
    FoundryLiveAuthorityResurrection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedRouteRequest {
    pub request: ModelRouteRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainRouteDecision {
    Routed(RouteDecision),
    Invalid(RouteDenial),
}

pub fn validate_route_request(
    request: ModelRouteRequest,
) -> Result<ValidatedRouteRequest, BTreeSet<RequestValidationFailure>> {
    let mut failures = BTreeSet::new();

    validate_tenant_id(&request.tenant_id, &mut failures);
    validate_request_evidence_ref(&request.request_evidence_ref, &mut failures);
    validate_credential_mode(request.credential_mode, &mut failures);
    validate_audience_data_boundary(request.audience, request.data_class, &mut failures);
    validate_env_tier_budget_contract(&request, &mut failures);

    if failures.is_empty() {
        Ok(ValidatedRouteRequest { request })
    } else {
        Err(failures)
    }
}

pub fn route_validated_request(
    request: ModelRouteRequest,
    catalog: &[ProviderRouteProfile],
) -> DomainRouteDecision {
    match validate_route_request(request) {
        Ok(validated) => DomainRouteDecision::Routed(decide_route(&validated.request, catalog)),
        Err(failures) => DomainRouteDecision::Invalid(validation_denial(failures)),
    }
}

fn validate_tenant_id(tenant_id: &str, failures: &mut BTreeSet<RequestValidationFailure>) {
    if tenant_id.trim().is_empty() {
        failures.insert(RequestValidationFailure::EmptyTenantId);
    } else if !tenant_id.starts_with("ten_") {
        failures.insert(RequestValidationFailure::InvalidTenantId);
    }
}

fn validate_request_evidence_ref(
    request_evidence_ref: &str,
    failures: &mut BTreeSet<RequestValidationFailure>,
) {
    if request_evidence_ref.trim().is_empty() {
        failures.insert(RequestValidationFailure::EmptyEvidenceRef);
    }
}

fn validate_credential_mode(
    credential_mode: CredentialMode,
    failures: &mut BTreeSet<RequestValidationFailure>,
) {
    if credential_mode == CredentialMode::Disabled {
        failures.insert(RequestValidationFailure::DisabledCredentialMode);
    }
}

fn validate_audience_data_boundary(
    audience: RequestAudience,
    data_class: IntelligenceDataClass,
    failures: &mut BTreeSet<RequestValidationFailure>,
) {
    if audience == RequestAudience::ExternalEndUser && is_sensitive_data_class(data_class) {
        failures.insert(RequestValidationFailure::ExternalAudienceSensitiveData);
    }
}

fn is_sensitive_data_class(data_class: IntelligenceDataClass) -> bool {
    matches!(
        data_class,
        IntelligenceDataClass::BehavioralTenantProduct
            | IntelligenceDataClass::Phi
            | IntelligenceDataClass::PiiIdentifying
            | IntelligenceDataClass::SearchQuery
    )
}

fn validate_env_tier_budget_contract(
    request: &ModelRouteRequest,
    failures: &mut BTreeSet<RequestValidationFailure>,
) {
    if request.env_tier.is_none() {
        failures.insert(RequestValidationFailure::MissingEnvTier);
    }
    if request.tier_cost_budget_policy_ref.trim().is_empty()
        || request
            .tier_cost_budget_evidence_ref
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        failures.insert(RequestValidationFailure::MissingTierCostBudgetEvidence);
    }
    if contains_retired_foundry_live_authority_ref(request) {
        failures.insert(RequestValidationFailure::FoundryLiveAuthorityResurrection);
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

fn validation_denial(failures: BTreeSet<RequestValidationFailure>) -> RouteDenial {
    let mut reasons = BTreeSet::new();
    let mut evidence_refs = Vec::new();

    for failure in failures {
        match failure {
            RequestValidationFailure::EmptyTenantId => {
                reasons.insert(RouteDenialReason::TenantNotAllowed);
                evidence_refs.push("validation:tenant-id-required".to_owned());
            }
            RequestValidationFailure::InvalidTenantId => {
                reasons.insert(RouteDenialReason::TenantNotAllowed);
                evidence_refs.push("validation:tenant-id-prefix".to_owned());
            }
            RequestValidationFailure::EmptyEvidenceRef => {
                reasons.insert(RouteDenialReason::NoEnabledProvider);
                evidence_refs.push("validation:evidence-ref-required".to_owned());
            }
            RequestValidationFailure::DisabledCredentialMode => {
                reasons.insert(RouteDenialReason::CredentialModeUnavailable);
                evidence_refs.push("validation:credential-mode-disabled".to_owned());
            }
            RequestValidationFailure::ExternalAudienceSensitiveData => {
                reasons.insert(RouteDenialReason::AudienceNotAllowed);
                reasons.insert(RouteDenialReason::DataClassNotAllowed);
                evidence_refs.push("validation:external-sensitive-data".to_owned());
            }
            RequestValidationFailure::MissingEnvTier => {
                reasons.insert(RouteDenialReason::EnvTierRequired);
                evidence_refs.push("env-tier:ENV-TIER-REQUIRED:missing_env_tier".to_owned());
            }
            RequestValidationFailure::MissingTierCostBudgetEvidence => {
                reasons.insert(RouteDenialReason::TierCostBudgetEvidenceMissing);
                evidence_refs.push(
                    "env-tier:TIER-BUDGET-EVIDENCE-REQUIRED:missing_per_tier_cost_budget_evidence"
                        .to_owned(),
                );
            }
            RequestValidationFailure::FoundryLiveAuthorityResurrection => {
                reasons.insert(RouteDenialReason::FoundryLiveAuthorityResurrection);
                evidence_refs.push(
                    "env-tier:FOUNDRY-LIVE-AUTHORITY-FORBIDDEN:foundry_live_authority_resurrection"
                        .to_owned(),
                );
            }
        }
    }

    evidence_refs.sort();
    evidence_refs.dedup();

    RouteDenial {
        reasons,
        evidence_refs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_TIER_RED_FIXTURES: &str =
        include_str!("../../../contracts/fixtures/env-tier-model-budget/red-fixtures.json");

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
            capability: ModelCapability::ChatCompletion,
            credential_mode: CredentialMode::TenantScoped,
            data_class: IntelligenceDataClass::InternalOnly,
            audience: RequestAudience::TenantOperator,
            request_evidence_ref: "req:route".to_owned(),
        }
    }

    fn profile() -> ProviderRouteProfile {
        ProviderRouteProfile {
            provider: ModelProvider::Anthropic,
            model_id: "claude-test".to_owned(),
            model_default_class: ModelDefaultClass::SmallCheap,
            profile_tags: BTreeSet::from([
                ModelProfileTag::CheapOrSmall,
                ModelProfileTag::SandboxOk,
                ModelProfileTag::NonProdOnly,
            ]),
            enabled: true,
            priority: 1,
            capabilities: BTreeSet::from([ModelCapability::ChatCompletion]),
            credential_modes: BTreeSet::from([CredentialMode::TenantScoped]),
            allowed_data_classes: BTreeSet::from([IntelligenceDataClass::InternalOnly]),
            allowed_audiences: BTreeSet::from([RequestAudience::TenantOperator]),
            allowed_tenants: BTreeSet::new(),
            evidence_refs: vec!["catalog:env-tier:test".to_owned()],
        }
    }

    #[test]
    fn validates_tenant_and_evidence_before_routing() {
        let mut invalid = request();
        invalid.tenant_id = " ".to_owned();
        invalid.request_evidence_ref.clear();

        assert_eq!(
            route_validated_request(invalid, &[]),
            DomainRouteDecision::Invalid(RouteDenial {
                reasons: BTreeSet::from([
                    RouteDenialReason::NoEnabledProvider,
                    RouteDenialReason::TenantNotAllowed,
                ]),
                evidence_refs: vec![
                    "validation:evidence-ref-required".to_owned(),
                    "validation:tenant-id-required".to_owned(),
                ],
            })
        );
    }

    #[test]
    fn rejects_sensitive_data_for_external_audience_before_provider_selection() {
        let mut invalid = request();
        invalid.audience = RequestAudience::ExternalEndUser;
        invalid.data_class = IntelligenceDataClass::PiiIdentifying;

        assert_eq!(
            validate_route_request(invalid),
            Err(BTreeSet::from([
                RequestValidationFailure::ExternalAudienceSensitiveData
            ]))
        );
    }

    #[test]
    fn env_tier_is_required_before_route_selection() {
        assert!(ENV_TIER_RED_FIXTURES.contains("missing_env_tier_denies_before_model_selection"));
        let mut invalid = request();
        invalid.env_tier = None;

        let DomainRouteDecision::Invalid(denial) = route_validated_request(invalid, &[profile()])
        else {
            panic!("missing env_tier must fail before route selection");
        };

        assert!(denial.reasons.contains(&RouteDenialReason::EnvTierRequired));
        assert!(
            denial
                .evidence_refs
                .contains(&"env-tier:ENV-TIER-REQUIRED:missing_env_tier".to_owned())
        );
    }

    #[test]
    fn test_tier_requires_small_cheap_non_prod_default() {
        assert!(ENV_TIER_RED_FIXTURES.contains("test_tier_rejects_production_grade_default"));
        let mut prod_default = profile();
        prod_default.model_default_class = ModelDefaultClass::ProductionGradeSelection;
        prod_default.profile_tags = BTreeSet::from([
            ModelProfileTag::ProductionGrade,
            ModelProfileTag::ProdApproved,
            ModelProfileTag::SloBacked,
        ]);

        let DomainRouteDecision::Routed(RouteDecision::Deny(denial)) =
            route_validated_request(request(), &[prod_default])
        else {
            panic!("test env_tier must reject production-grade defaults");
        };

        assert!(
            denial
                .reasons
                .contains(&RouteDenialReason::TierModelDefaultMismatch)
        );
    }

    #[test]
    fn retired_foundry_authority_refs_are_denied() {
        assert!(ENV_TIER_RED_FIXTURES.contains("foundry_live_authority_resurrection_is_rejected"));
        let mut invalid = request();
        invalid.model_default_policy_ref =
            "foundry:cost-budget.md#test_tier_model_default".to_owned();
        invalid.model_route_registry_snapshot_ref =
            "specs/microservices/foundry.json#live-authority".to_owned();

        let DomainRouteDecision::Invalid(denial) = route_validated_request(invalid, &[profile()])
        else {
            panic!("retired foundry live authority refs must fail closed");
        };

        assert!(
            denial
                .reasons
                .contains(&RouteDenialReason::FoundryLiveAuthorityResurrection)
        );
        assert!(
            denial.evidence_refs.contains(
                &"env-tier:FOUNDRY-LIVE-AUTHORITY-FORBIDDEN:foundry_live_authority_resurrection"
                    .to_owned()
            )
        );
    }
}
