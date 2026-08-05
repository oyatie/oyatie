//! Intelligence dispatch-flow usecase foundation.
//!
//! This crate owns the first provider-port orchestration seam for the
//! Intelligence microservice. It deliberately moves only metadata references —
//! never raw prompt, credential, provider response, or model-output bytes — so
//! later cloud/provider adapters can plug in without changing routing,
//! guardrail, audit, or idempotency semantics.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use oya_intelligence_guardrails_domain::{
    DomainGuardrailRequest, GuardrailAudience, GuardrailDataClass, GuardrailDecision,
    GuardrailDeny, GuardrailFinding, GuardrailRequest, decide_domain_guardrail,
};
use oya_intelligence_model_routing_domain::{
    DomainRouteDecision, IntelligenceDataClass, ModelRouteRequest, ProviderRouteProfile,
    RequestAudience, RouteDecision, RouteDenial, RouteSelection, route_validated_request,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchInput {
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub route_request: ModelRouteRequest, // data_class: INTERNAL_ONLY
    pub content_ref: String,              // data_class: INTERNAL_ONLY
    pub pre_dispatch_findings: Vec<GuardrailFinding>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDispatchRequest {
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub content_ref: String,             // data_class: INTERNAL_ONLY
    pub route_selection: RouteSelection, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDispatchResponse {
    pub output_ref: String,            // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub output_guardrail_findings: Vec<GuardrailFinding>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDispatchFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchStatus {
    Completed,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchDenialKind {
    InvalidInput,
    RouteDenied,
    PreGuardrailDenied,
    ProviderFailed,
    OutputGuardrailDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchReceipt {
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub status: DispatchStatus,                  // data_class: PUBLIC
    pub denial_kind: Option<DispatchDenialKind>, // data_class: INTERNAL_ONLY
    pub denial_reasons: Vec<String>,             // data_class: INTERNAL_ONLY
    pub route_selection: Option<RouteSelection>, // data_class: INTERNAL_ONLY
    pub output_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchAuditEventKind {
    DispatchRequestReceived,
    RouteDenied,
    GuardrailDenied,
    ProviderDispatchFailed,
    ProviderDispatchCompleted,
    DispatchCompleted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchAuditEvent {
    pub kind: DispatchAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub idempotency_key: String,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

pub trait ProviderDispatchPort {
    fn dispatch(
        &mut self,
        request: ProviderDispatchRequest,
    ) -> Result<ProviderDispatchResponse, ProviderDispatchFailure>;
}

pub trait DispatchAuditSink {
    fn record(&mut self, event: DispatchAuditEvent);
}

#[derive(Debug)]
pub struct IntelligenceDispatchOrchestrator<P, A> {
    provider: P,
    audit_sink: A,
    receipts_by_idempotency_key: BTreeMap<String, DispatchReceipt>,
}

impl<P, A> IntelligenceDispatchOrchestrator<P, A>
where
    P: ProviderDispatchPort,
    A: DispatchAuditSink,
{
    pub fn new(provider: P, audit_sink: A) -> Self {
        Self {
            provider,
            audit_sink,
            receipts_by_idempotency_key: BTreeMap::new(),
        }
    }

    pub fn dispatch(
        &mut self,
        input: DispatchInput,
        catalog: &[ProviderRouteProfile],
    ) -> DispatchReceipt {
        if let Some(existing) = self.receipts_by_idempotency_key.get(&input.idempotency_key) {
            return existing.clone();
        }

        let receipt = self.dispatch_uncached(input, catalog);
        if !receipt.idempotency_key.trim().is_empty() {
            self.receipts_by_idempotency_key
                .insert(receipt.idempotency_key.clone(), receipt.clone());
        }
        receipt
    }

    pub fn into_parts(self) -> (P, A) {
        (self.provider, self.audit_sink)
    }

    fn dispatch_uncached(
        &mut self,
        input: DispatchInput,
        catalog: &[ProviderRouteProfile],
    ) -> DispatchReceipt {
        let idempotency_key = input.idempotency_key.clone();
        let tenant_id = input.route_request.tenant_id.clone();
        if idempotency_key.trim().is_empty() || input.content_ref.trim().is_empty() {
            return self.invalid_input_receipt(input);
        }

        self.record_event(
            DispatchAuditEventKind::DispatchRequestReceived,
            tenant_id.clone(),
            idempotency_key.clone(),
            vec![input.route_request.request_evidence_ref.clone()],
        );

        let route_selection = match route_validated_request(input.route_request.clone(), catalog) {
            DomainRouteDecision::Routed(RouteDecision::Allow(selection)) => selection,
            DomainRouteDecision::Routed(RouteDecision::Deny(denial)) => {
                return self.route_denied_receipt(input, denial);
            }
            DomainRouteDecision::Invalid(denial) => {
                return self.route_denied_receipt(input, denial);
            }
        };

        if let Some(receipt) = self.guardrail_denial_receipt(
            &input,
            &route_selection,
            input.content_ref.clone(),
            input.pre_dispatch_findings.clone(),
            Vec::new(),
            DispatchDenialKind::PreGuardrailDenied,
        ) {
            return receipt;
        }

        let provider_request = ProviderDispatchRequest {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: input.route_request.tenant_id.clone(),
            content_ref: input.content_ref.clone(),
            route_selection: route_selection.clone(),
            request_evidence_ref: input.route_request.request_evidence_ref.clone(),
        };

        let provider_response = match self.provider.dispatch(provider_request) {
            Ok(response) => response,
            Err(failure) => {
                return self.provider_failure_receipt(input, route_selection, failure);
            }
        };

        self.record_event(
            DispatchAuditEventKind::ProviderDispatchCompleted,
            input.route_request.tenant_id.clone(),
            input.idempotency_key.clone(),
            vec![provider_response.provider_evidence_ref.clone()],
        );

        if let Some(receipt) = self.guardrail_denial_receipt(
            &input,
            &route_selection,
            provider_response.output_ref.clone(),
            provider_response.output_guardrail_findings.clone(),
            vec![provider_response.provider_evidence_ref.clone()],
            DispatchDenialKind::OutputGuardrailDenied,
        ) {
            return receipt;
        }

        let receipt = DispatchReceipt {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: input.route_request.tenant_id.clone(),
            status: DispatchStatus::Completed,
            denial_kind: None,
            denial_reasons: Vec::new(),
            route_selection: Some(route_selection.clone()),
            output_ref: Some(provider_response.output_ref.clone()),
            evidence_refs: sorted_unique(collect_evidence_refs([
                vec![input.route_request.request_evidence_ref.clone()],
                route_selection.evidence_refs.clone(),
                vec![provider_response.provider_evidence_ref.clone()],
            ])),
        };
        self.record_event(
            DispatchAuditEventKind::DispatchCompleted,
            receipt.tenant_id.clone(),
            receipt.idempotency_key.clone(),
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn invalid_input_receipt(&mut self, input: DispatchInput) -> DispatchReceipt {
        let mut denial_reasons = Vec::new();
        if input.idempotency_key.trim().is_empty() {
            denial_reasons.push("idempotency key is required before dispatch".to_owned());
        }
        if input.content_ref.trim().is_empty() {
            denial_reasons.push("content reference is required before dispatch".to_owned());
        }
        let receipt = DispatchReceipt {
            idempotency_key: input.idempotency_key,
            tenant_id: input.route_request.tenant_id,
            status: DispatchStatus::Denied,
            denial_kind: Some(DispatchDenialKind::InvalidInput),
            denial_reasons: sorted_unique(denial_reasons),
            route_selection: None,
            output_ref: None,
            evidence_refs: sorted_unique(vec!["validation:intelligence-dispatch".to_owned()]),
        };
        self.record_event(
            DispatchAuditEventKind::RouteDenied,
            receipt.tenant_id.clone(),
            receipt.idempotency_key.clone(),
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn route_denied_receipt(
        &mut self,
        input: DispatchInput,
        denial: RouteDenial,
    ) -> DispatchReceipt {
        let reasons = denial
            .reasons
            .into_iter()
            .map(|reason| format!("{reason:?}"))
            .collect();
        let receipt = DispatchReceipt {
            idempotency_key: input.idempotency_key,
            tenant_id: input.route_request.tenant_id,
            status: DispatchStatus::Denied,
            denial_kind: Some(DispatchDenialKind::RouteDenied),
            denial_reasons: sorted_unique(reasons),
            route_selection: None,
            output_ref: None,
            evidence_refs: sorted_unique(denial.evidence_refs),
        };
        self.record_event(
            DispatchAuditEventKind::RouteDenied,
            receipt.tenant_id.clone(),
            receipt.idempotency_key.clone(),
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn guardrail_denial_receipt(
        &mut self,
        input: &DispatchInput,
        route_selection: &RouteSelection,
        content_ref: String,
        findings: Vec<GuardrailFinding>,
        additional_evidence_refs: Vec<String>,
        denial_kind: DispatchDenialKind,
    ) -> Option<DispatchReceipt> {
        let guardrail_request = DomainGuardrailRequest {
            guardrail_request: GuardrailRequest {
                tenant_id: input.route_request.tenant_id.clone(),
                content_ref,
                findings,
                request_evidence_ref: input.route_request.request_evidence_ref.clone(),
            },
            data_class: guardrail_data_class(input.route_request.data_class),
            audience: guardrail_audience(input.route_request.audience),
        };

        let GuardrailDecision::Deny(GuardrailDeny {
            refusal_reasons,
            evidence_refs,
        }) = decide_domain_guardrail(&guardrail_request)
        else {
            return None;
        };

        let receipt = DispatchReceipt {
            idempotency_key: input.idempotency_key.clone(),
            tenant_id: input.route_request.tenant_id.clone(),
            status: DispatchStatus::Denied,
            denial_kind: Some(denial_kind),
            denial_reasons: sorted_unique(refusal_reasons),
            route_selection: Some(route_selection.clone()),
            output_ref: None,
            evidence_refs: sorted_unique(collect_evidence_refs([
                vec![input.route_request.request_evidence_ref.clone()],
                route_selection.evidence_refs.clone(),
                evidence_refs,
                additional_evidence_refs,
            ])),
        };
        self.record_event(
            DispatchAuditEventKind::GuardrailDenied,
            receipt.tenant_id.clone(),
            receipt.idempotency_key.clone(),
            receipt.evidence_refs.clone(),
        );
        Some(receipt)
    }

    fn provider_failure_receipt(
        &mut self,
        input: DispatchInput,
        route_selection: RouteSelection,
        failure: ProviderDispatchFailure,
    ) -> DispatchReceipt {
        let receipt = DispatchReceipt {
            idempotency_key: input.idempotency_key,
            tenant_id: input.route_request.tenant_id,
            status: DispatchStatus::Denied,
            denial_kind: Some(DispatchDenialKind::ProviderFailed),
            denial_reasons: sorted_unique(vec![failure.reason]),
            route_selection: Some(route_selection.clone()),
            output_ref: None,
            evidence_refs: sorted_unique(collect_evidence_refs([
                vec![input.route_request.request_evidence_ref],
                route_selection.evidence_refs,
                vec![failure.evidence_ref],
            ])),
        };
        self.record_event(
            DispatchAuditEventKind::ProviderDispatchFailed,
            receipt.tenant_id.clone(),
            receipt.idempotency_key.clone(),
            receipt.evidence_refs.clone(),
        );
        receipt
    }

    fn record_event(
        &mut self,
        kind: DispatchAuditEventKind,
        tenant_id: String,
        idempotency_key: String,
        evidence_refs: Vec<String>,
    ) {
        self.audit_sink.record(DispatchAuditEvent {
            kind,
            tenant_id,
            idempotency_key,
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

fn guardrail_data_class(data_class: IntelligenceDataClass) -> GuardrailDataClass {
    match data_class {
        IntelligenceDataClass::BehavioralTenantProduct => {
            GuardrailDataClass::BehavioralTenantProduct
        }
        IntelligenceDataClass::InternalOnly => GuardrailDataClass::InternalOnly,
        IntelligenceDataClass::Phi => GuardrailDataClass::Phi,
        IntelligenceDataClass::PiiIdentifying => GuardrailDataClass::PiiIdentifying,
        IntelligenceDataClass::Public => GuardrailDataClass::Public,
        IntelligenceDataClass::SearchQuery => GuardrailDataClass::SearchQuery,
    }
}

fn guardrail_audience(audience: RequestAudience) -> GuardrailAudience {
    match audience {
        RequestAudience::ExternalEndUser => GuardrailAudience::ExternalEndUser,
        RequestAudience::InternalAutomation => GuardrailAudience::InternalAutomation,
        RequestAudience::TenantOperator => GuardrailAudience::TenantOperator,
    }
}

fn collect_evidence_refs<const N: usize>(sets: [Vec<String>; N]) -> Vec<String> {
    sets.into_iter().flatten().collect()
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use oya_intelligence_guardrails_domain::{GuardrailCategory, RiskLevel};
    use oya_intelligence_model_routing_domain::{
        CredentialMode, EnvTier, ModelCapability, ModelDefaultClass, ModelProfileTag,
        ModelProvider, ProviderRouteProfile,
    };

    #[derive(Debug, Default)]
    struct FakeProvider {
        calls: Vec<ProviderDispatchRequest>,
        response: Option<Result<ProviderDispatchResponse, ProviderDispatchFailure>>,
    }

    impl ProviderDispatchPort for FakeProvider {
        fn dispatch(
            &mut self,
            request: ProviderDispatchRequest,
        ) -> Result<ProviderDispatchResponse, ProviderDispatchFailure> {
            self.calls.push(request);
            self.response.clone().unwrap_or_else(|| {
                Ok(ProviderDispatchResponse {
                    output_ref: "output:default".to_owned(),
                    provider_evidence_ref: "provider:default".to_owned(),
                    output_guardrail_findings: vec![benign_finding("classifier:output")],
                })
            })
        }
    }

    #[derive(Debug, Default)]
    struct FakeAuditSink {
        events: Vec<DispatchAuditEvent>,
    }

    impl DispatchAuditSink for FakeAuditSink {
        fn record(&mut self, event: DispatchAuditEvent) {
            self.events.push(event);
        }
    }

    fn route_request() -> ModelRouteRequest {
        ModelRouteRequest {
            tenant_id: "ten_acme".to_owned(),
            env_tier: Some(EnvTier::Test),
            model_default_policy_ref: "policy:intelligence.env-tier.model-default.test.v1"
                .to_owned(),
            tier_cost_budget_policy_ref: "policy:intelligence.env-tier.cost-budget.test.v1"
                .to_owned(),
            tier_cost_budget_evidence_ref: Some("budget:intelligence:test:dispatch".to_owned()),
            model_route_registry_snapshot_ref: "route-registry:intelligence:env-tier:test"
                .to_owned(),
            capability: ModelCapability::ChatCompletion,
            credential_mode: CredentialMode::TenantScoped,
            data_class: IntelligenceDataClass::InternalOnly,
            audience: RequestAudience::TenantOperator,
            request_evidence_ref: "req:dispatch".to_owned(),
        }
    }

    fn profile() -> ProviderRouteProfile {
        ProviderRouteProfile {
            provider: ModelProvider::OpenAi,
            model_id: "gpt-preview".to_owned(),
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
            evidence_refs: vec!["catalog:openai".to_owned()],
        }
    }

    fn dispatch_input(idempotency_key: &str) -> DispatchInput {
        DispatchInput {
            idempotency_key: idempotency_key.to_owned(),
            route_request: route_request(),
            content_ref: "content:input".to_owned(),
            pre_dispatch_findings: vec![benign_finding("classifier:input")],
        }
    }

    fn benign_finding(evidence_ref: &str) -> GuardrailFinding {
        GuardrailFinding {
            category: GuardrailCategory::Benign,
            risk_level: RiskLevel::Low,
            reason: "benign".to_owned(),
            evidence_ref: evidence_ref.to_owned(),
        }
    }

    fn high_risk_finding(evidence_ref: &str) -> GuardrailFinding {
        GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::High,
            reason: "prompt injection attempt".to_owned(),
            evidence_ref: evidence_ref.to_owned(),
        }
    }

    fn orchestrator(
        response: Option<Result<ProviderDispatchResponse, ProviderDispatchFailure>>,
    ) -> IntelligenceDispatchOrchestrator<FakeProvider, FakeAuditSink> {
        IntelligenceDispatchOrchestrator::new(
            FakeProvider {
                calls: Vec::new(),
                response,
            },
            FakeAuditSink::default(),
        )
    }

    #[test]
    fn completes_successful_dispatch_with_provider_port_and_audit_events() {
        let mut orchestrator = orchestrator(None);

        let receipt = orchestrator.dispatch(dispatch_input("idem:1"), &[profile()]);
        let (provider, audit) = orchestrator.into_parts();

        assert_eq!(receipt.status, DispatchStatus::Completed);
        assert_eq!(receipt.output_ref, Some("output:default".to_owned()));
        assert_eq!(receipt.denial_kind, None);
        assert_eq!(provider.calls.len(), 1);
        assert_eq!(provider.calls[0].route_selection.model_id, "gpt-preview");
        assert_eq!(
            audit
                .events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                DispatchAuditEventKind::DispatchRequestReceived,
                DispatchAuditEventKind::ProviderDispatchCompleted,
                DispatchAuditEventKind::DispatchCompleted,
            ]
        );
    }

    #[test]
    fn invalid_route_denies_before_provider_dispatch() {
        let mut invalid = dispatch_input("idem:route-denied");
        invalid.route_request.tenant_id = " ".to_owned();
        invalid.route_request.request_evidence_ref.clear();
        let mut orchestrator = orchestrator(None);

        let receipt = orchestrator.dispatch(invalid, &[profile()]);
        let (provider, _audit) = orchestrator.into_parts();

        assert_eq!(receipt.status, DispatchStatus::Denied);
        assert_eq!(receipt.denial_kind, Some(DispatchDenialKind::RouteDenied));
        assert_eq!(provider.calls.len(), 0);
        assert!(receipt.output_ref.is_none());
    }

    #[test]
    fn pre_guardrail_denial_prevents_provider_dispatch() {
        let mut input = dispatch_input("idem:pre-guardrail");
        input.pre_dispatch_findings = vec![high_risk_finding("classifier:pre")];
        let mut orchestrator = orchestrator(None);

        let receipt = orchestrator.dispatch(input, &[profile()]);
        let (provider, _audit) = orchestrator.into_parts();

        assert_eq!(receipt.status, DispatchStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(DispatchDenialKind::PreGuardrailDenied)
        );
        assert_eq!(provider.calls.len(), 0);
        assert!(
            receipt
                .denial_reasons
                .contains(&"prompt injection attempt".to_owned())
        );
    }

    #[test]
    fn output_guardrail_denial_suppresses_provider_output_ref() {
        let response = ProviderDispatchResponse {
            output_ref: "output:sensitive".to_owned(),
            provider_evidence_ref: "provider:ok".to_owned(),
            output_guardrail_findings: vec![high_risk_finding("classifier:output")],
        };
        let mut orchestrator = orchestrator(Some(Ok(response)));

        let receipt = orchestrator.dispatch(dispatch_input("idem:output-guardrail"), &[profile()]);
        let (provider, _audit) = orchestrator.into_parts();

        assert_eq!(provider.calls.len(), 1);
        assert_eq!(receipt.status, DispatchStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(DispatchDenialKind::OutputGuardrailDenied)
        );
        assert_eq!(receipt.output_ref, None);
        assert!(receipt.evidence_refs.contains(&"provider:ok".to_owned()));
    }

    #[test]
    fn duplicate_idempotency_returns_original_receipt_without_second_provider_call() {
        let mut orchestrator = orchestrator(None);

        let first = orchestrator.dispatch(dispatch_input("idem:duplicate"), &[profile()]);
        let second = orchestrator.dispatch(dispatch_input("idem:duplicate"), &[profile()]);
        let (provider, _audit) = orchestrator.into_parts();

        assert_eq!(first, second);
        assert_eq!(provider.calls.len(), 1);
    }

    #[test]
    fn provider_failure_denies_closed_with_evidence() {
        let failure = ProviderDispatchFailure {
            reason: "provider circuit open".to_owned(),
            evidence_ref: "provider:circuit-open".to_owned(),
        };
        let mut orchestrator = orchestrator(Some(Err(failure)));

        let receipt = orchestrator.dispatch(dispatch_input("idem:provider-failure"), &[profile()]);

        assert_eq!(receipt.status, DispatchStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(DispatchDenialKind::ProviderFailed)
        );
        assert_eq!(receipt.output_ref, None);
        assert!(
            receipt
                .denial_reasons
                .contains(&"provider circuit open".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"provider:circuit-open".to_owned())
        );
    }
}
