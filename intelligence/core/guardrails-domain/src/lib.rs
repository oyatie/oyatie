//! Domain validation wrapper for Intelligence guardrail decisions.
//!
//! This layer validates request provenance and data-use boundaries before the
//! pure guardrail kernel evaluates classifier findings. Invalid requests deny
//! closed and carry validation evidence.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_guardrails_kernel::{
    GuardrailAllow, GuardrailCategory, GuardrailDecision, GuardrailDeny, GuardrailFinding,
    GuardrailRequest, RiskLevel, decide_guardrail,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GuardrailDataClass {
    BehavioralTenantProduct,
    InternalOnly,
    Phi,
    PiiIdentifying,
    Public,
    SearchQuery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GuardrailAudience {
    ExternalEndUser,
    InternalAutomation,
    TenantOperator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainGuardrailRequest {
    pub guardrail_request: GuardrailRequest, // data_class: INTERNAL_ONLY
    pub data_class: GuardrailDataClass,      // data_class: INTERNAL_ONLY
    pub audience: GuardrailAudience,         // data_class: INTERNAL_ONLY
}

pub fn decide_domain_guardrail(request: &DomainGuardrailRequest) -> GuardrailDecision {
    let validation_refusals = validate_domain_guardrail_request(request);
    if validation_refusals.is_empty() {
        return decide_guardrail(&request.guardrail_request);
    }

    GuardrailDecision::Deny(GuardrailDeny {
        refusal_reasons: sorted_unique(validation_refusals),
        evidence_refs: validation_evidence_refs(&request.guardrail_request),
    })
}

fn validate_domain_guardrail_request(request: &DomainGuardrailRequest) -> Vec<String> {
    let mut refusals = Vec::new();
    let guardrail_request = &request.guardrail_request;

    if guardrail_request.tenant_id.trim().is_empty() {
        refusals.push("tenant is required before guardrail evaluation".to_owned());
    } else if !guardrail_request.tenant_id.starts_with("ten_") {
        refusals.push("tenant id must use ten_ prefix before guardrail evaluation".to_owned());
    }

    if guardrail_request.content_ref.trim().is_empty() {
        refusals.push("content reference is required before guardrail evaluation".to_owned());
    }

    if guardrail_request.request_evidence_ref.trim().is_empty() {
        refusals
            .push("request evidence reference is required before guardrail evaluation".to_owned());
    }

    if request.audience == GuardrailAudience::ExternalEndUser
        && is_sensitive_data_class(request.data_class)
    {
        refusals.push("external audience cannot receive sensitive intelligence output".to_owned());
    }

    refusals
}

fn is_sensitive_data_class(data_class: GuardrailDataClass) -> bool {
    matches!(
        data_class,
        GuardrailDataClass::BehavioralTenantProduct
            | GuardrailDataClass::Phi
            | GuardrailDataClass::PiiIdentifying
            | GuardrailDataClass::SearchQuery
    )
}

fn validation_evidence_refs(request: &GuardrailRequest) -> Vec<String> {
    let mut evidence_refs = Vec::new();
    if !request.request_evidence_ref.trim().is_empty() {
        evidence_refs.push(request.request_evidence_ref.clone());
    }
    evidence_refs.push("validation:guardrails-domain".to_owned());
    sorted_unique(evidence_refs)
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain_request() -> DomainGuardrailRequest {
        DomainGuardrailRequest {
            guardrail_request: GuardrailRequest {
                tenant_id: "ten_a".to_owned(),
                content_ref: "content:1".to_owned(),
                findings: vec![GuardrailFinding {
                    category: GuardrailCategory::Benign,
                    risk_level: RiskLevel::Low,
                    reason: "benign".to_owned(),
                    evidence_ref: "classifier:1".to_owned(),
                }],
                request_evidence_ref: "req:1".to_owned(),
            },
            data_class: GuardrailDataClass::InternalOnly,
            audience: GuardrailAudience::TenantOperator,
        }
    }

    #[test]
    fn sensitive_external_output_fails_closed() {
        let mut request = domain_request();
        request.data_class = GuardrailDataClass::PiiIdentifying;
        request.audience = GuardrailAudience::ExternalEndUser;

        assert_eq!(
            decide_domain_guardrail(&request),
            GuardrailDecision::Deny(GuardrailDeny {
                refusal_reasons: vec![
                    "external audience cannot receive sensitive intelligence output".to_owned()
                ],
                evidence_refs: vec![
                    "req:1".to_owned(),
                    "validation:guardrails-domain".to_owned()
                ],
            })
        );
    }

    #[test]
    fn valid_domain_request_delegates_to_kernel() {
        assert_eq!(
            decide_domain_guardrail(&domain_request()),
            GuardrailDecision::Allow(GuardrailAllow {
                evidence_refs: vec!["classifier:1".to_owned(), "req:1".to_owned()],
            })
        );
    }
}
