//! Domain validation for Intelligence model-routing requests.
//!
//! The domain layer fails closed before delegating to the deterministic kernel.
//! It validates tenant scoping, evidence presence, credential posture, and the
//! data-class/audience boundary that provider routing must not bypass.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub use intelligence_model_routing_kernel::{
    CredentialMode, IntelligenceDataClass, ModelCapability, ModelProvider, ModelRouteRequest,
    ProviderRouteProfile, RequestAudience, RouteDecision, RouteDenial, RouteDenialReason,
    RouteSelection, decide_route,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RequestValidationFailure {
    EmptyTenantId,
    InvalidTenantId,
    EmptyEvidenceRef,
    DisabledCredentialMode,
    ExternalAudienceSensitiveData,
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

    fn request() -> ModelRouteRequest {
        ModelRouteRequest {
            tenant_id: "ten_a".to_owned(),
            capability: ModelCapability::ChatCompletion,
            credential_mode: CredentialMode::TenantScoped,
            data_class: IntelligenceDataClass::InternalOnly,
            audience: RequestAudience::TenantOperator,
            request_evidence_ref: "req:route".to_owned(),
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
}
