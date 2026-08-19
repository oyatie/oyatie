//! Intelligence eval adapter foundation.
//!
//! This crate defines a deterministic, metadata-only adapter seam for future
//! hosted eval-runner integration. It builds runner request envelopes from the
//! eval domain/usecase metadata already authorized by the preview foundation and
//! maps runner outcome metadata into stable receipts. It performs no network
//! I/O, OpenAI Evals API calls, LLM-as-judge calls, dataset fetches, filesystem
//! access, credential resolution, durable idempotency, or durable audit-chain
//! emission.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_eval_usecase::{
    DomainEvalSetRequest, EvalCaseKind, EvalCaseOutcome, EvalCaseResult, EvalFailureKind,
    EvalPolicyDecision, EvalSet, EvalSetStatus, EvalSetThresholds, EvalUsecaseDenialKind,
    EvalUsecaseReceipt, EvalUsecaseStatus,
};

const EVAL_RUNNER_PATH: &str = "/v1/eval-runs";
const ADAPTER_REFERENCE_REF: &str = "spec://oyatie/intelligence/eval-adapter-foundation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalRunnerHttpMethod {
    Post,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalRunnerTransportMode {
    EnvelopeOnly,
    HostedRunner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerAdapterConfig {
    pub endpoint: String,                        // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,           // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                   // data_class: INTERNAL_ONLY
    pub runner_audience_ref: String,             // data_class: INTERNAL_ONLY
    pub transport_mode: EvalRunnerTransportMode, // data_class: INTERNAL_ONLY
}

impl EvalRunnerAdapterConfig {
    pub fn new(
        endpoint: impl Into<String>,
        credential_handle_ref: impl Into<String>,
        audit_tap_ref: impl Into<String>,
        runner_audience_ref: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            credential_handle_ref: credential_handle_ref.into(),
            audit_tap_ref: audit_tap_ref.into(),
            runner_audience_ref: runner_audience_ref.into(),
            transport_mode: EvalRunnerTransportMode::EnvelopeOnly,
        }
    }

    pub fn with_transport_mode(mut self, transport_mode: EvalRunnerTransportMode) -> Self {
        self.transport_mode = transport_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalRunnerAdapterConfigError {
    EmptyEndpoint,
    NonHttpsEndpoint,
    LocalEndpointDenied,
    EmptyCredentialHandleRef,
    RawCredentialMaterialRejected,
    NonOpaqueCredentialHandleRef,
    EmptyAuditTapRef,
    InvalidAuditTapRef,
    EmptyRunnerAudienceRef,
    InvalidRunnerAudienceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerDispatchRequest {
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub domain_request: DomainEvalSetRequest, // data_class: INTERNAL_ONLY
    pub usecase_receipt: EvalUsecaseReceipt,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerThresholdEnvelope {
    pub min_pass_rate_bps: u32,             // data_class: INTERNAL_ONLY
    pub max_safety_violation_rate_bps: u32, // data_class: INTERNAL_ONLY
    pub require_golden: bool,               // data_class: INTERNAL_ONLY
    pub require_adversarial: bool,          // data_class: INTERNAL_ONLY
    pub require_linguistic: bool,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerRequestEnvelope {
    pub method: EvalRunnerHttpMethod,              // data_class: PUBLIC
    pub endpoint: String,                          // data_class: INTERNAL_ONLY
    pub path: String,                              // data_class: PUBLIC
    pub transport_mode: EvalRunnerTransportMode,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub principal_id: String,                      // data_class: INTERNAL_ONLY
    pub eval_surface: String,                      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                   // data_class: INTERNAL_ONLY
    pub eval_set_id: String,                       // data_class: INTERNAL_ONLY
    pub model_ref: String,                         // data_class: INTERNAL_ONLY
    pub dataset_snapshot_ref: String,              // data_class: INTERNAL_ONLY
    pub route_evidence_ref: String,                // data_class: INTERNAL_ONLY
    pub guardrail_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,              // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                 // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,               // data_class: INTERNAL_ONLY
    pub eval_registry_snapshot_ref: String,        // data_class: INTERNAL_ONLY
    pub credential_handle_ref: String,             // data_class: INTERNAL_ONLY
    pub audit_tap_ref: String,                     // data_class: INTERNAL_ONLY
    pub runner_audience_ref: String,               // data_class: INTERNAL_ONLY
    pub case_count: u32,                           // data_class: INTERNAL_ONLY
    pub case_evaluator_evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub thresholds: EvalRunnerThresholdEnvelope,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                // data_class: INTERNAL_ONLY
    pub adapter_reference_refs: Vec<String>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalRunnerStatus {
    Accepted {
        runner_request_ref: String,
        run_ref: String,
        evidence_ref: String,
    },
    Queued {
        runner_request_ref: String,
        queue_ref: String,
        evidence_ref: String,
    },
    Completed {
        runner_request_ref: String,
        run_ref: String,
        report_ref: String,
        evidence_ref: String,
    },
    Denied {
        evidence_ref: String,
    },
    RateLimited {
        evidence_ref: String,
    },
    RunnerError {
        evidence_ref: String,
    },
    AuthError {
        evidence_ref: String,
    },
    InvalidRequest {
        evidence_ref: String,
    },
    Timeout {
        evidence_ref: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalRunnerDispatchStatus {
    Accepted,
    Queued,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerDispatchReceipt {
    pub status: EvalRunnerDispatchStatus,   // data_class: PUBLIC
    pub runner_request_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub run_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub queue_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub report_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub evidence_ref: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerDispatchFailure {
    pub reason: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub struct IntelligenceEvalAdapter {
    config: EvalRunnerAdapterConfig,
    next_status: EvalRunnerStatus,
    last_envelope: Option<EvalRunnerRequestEnvelope>,
}

impl IntelligenceEvalAdapter {
    pub fn try_new(
        config: EvalRunnerAdapterConfig,
        next_status: EvalRunnerStatus,
    ) -> Result<Self, EvalRunnerAdapterConfigError> {
        validate_config(&config)?;
        Ok(Self {
            config,
            next_status,
            last_envelope: None,
        })
    }

    pub fn last_envelope(&self) -> Option<&EvalRunnerRequestEnvelope> {
        self.last_envelope.as_ref()
    }

    pub fn set_next_status(&mut self, next_status: EvalRunnerStatus) {
        self.next_status = next_status;
    }

    pub fn dispatch(
        &mut self,
        request: EvalRunnerDispatchRequest,
    ) -> Result<EvalRunnerDispatchReceipt, EvalRunnerDispatchFailure> {
        validate_dispatch_request(&request)?;
        let envelope = self.build_envelope(&request);
        self.last_envelope = Some(envelope);
        receipt_from_status(&self.next_status)
    }

    fn build_envelope(&self, request: &EvalRunnerDispatchRequest) -> EvalRunnerRequestEnvelope {
        let eval_set = &request.domain_request.eval_set;
        EvalRunnerRequestEnvelope {
            method: EvalRunnerHttpMethod::Post,
            endpoint: normalized_endpoint(&self.config.endpoint),
            path: EVAL_RUNNER_PATH.to_owned(),
            transport_mode: self.config.transport_mode,
            tenant_id: request.domain_request.tenant_id.clone(),
            principal_id: request.domain_request.principal_id.clone(),
            eval_surface: request.domain_request.eval_surface.clone(),
            idempotency_key: request.idempotency_key.clone(),
            eval_set_id: eval_set.eval_set_id.clone(),
            model_ref: eval_set.model_ref.clone(),
            dataset_snapshot_ref: eval_set.dataset_snapshot_ref.clone(),
            route_evidence_ref: eval_set.route_evidence_ref.clone(),
            guardrail_evidence_ref: eval_set.guardrail_evidence_ref.clone(),
            request_evidence_ref: request.domain_request.request_evidence_ref.clone(),
            trace_context_ref: request.domain_request.trace_context_ref.clone(),
            policy_decision_ref: request.domain_request.policy_decision_ref.clone(),
            eval_registry_snapshot_ref: request
                .domain_request
                .policy_decision
                .eval_registry_snapshot_ref
                .clone(),
            credential_handle_ref: self.config.credential_handle_ref.clone(),
            audit_tap_ref: self.config.audit_tap_ref.clone(),
            runner_audience_ref: self.config.runner_audience_ref.clone(),
            case_count: eval_set.cases.len() as u32,
            case_evaluator_evidence_refs: sorted_unique(
                eval_set
                    .cases
                    .iter()
                    .map(|case| case.evaluator_evidence_ref.clone())
                    .collect(),
            ),
            thresholds: EvalRunnerThresholdEnvelope {
                min_pass_rate_bps: eval_set.thresholds.min_pass_rate_bps,
                max_safety_violation_rate_bps: eval_set.thresholds.max_safety_violation_rate_bps,
                require_golden: eval_set.thresholds.require_golden,
                require_adversarial: eval_set.thresholds.require_adversarial,
                require_linguistic: eval_set.thresholds.require_linguistic,
            },
            evidence_refs: dispatch_evidence_refs(request),
            adapter_reference_refs: vec![ADAPTER_REFERENCE_REF.to_owned()],
        }
    }
}

fn validate_config(config: &EvalRunnerAdapterConfig) -> Result<(), EvalRunnerAdapterConfigError> {
    let endpoint = config.endpoint.trim();
    if endpoint.is_empty() {
        return Err(EvalRunnerAdapterConfigError::EmptyEndpoint);
    }
    if !endpoint.starts_with("https://") || contains_whitespace(endpoint) {
        return Err(EvalRunnerAdapterConfigError::NonHttpsEndpoint);
    }
    if is_local_endpoint(endpoint) {
        return Err(EvalRunnerAdapterConfigError::LocalEndpointDenied);
    }
    validate_credential_handle_ref(&config.credential_handle_ref)?;
    validate_audit_tap_ref(&config.audit_tap_ref)?;
    validate_runner_audience_ref(&config.runner_audience_ref)?;
    Ok(())
}

fn validate_credential_handle_ref(value: &str) -> Result<(), EvalRunnerAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(EvalRunnerAdapterConfigError::EmptyCredentialHandleRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(if contains_raw_secret_material(trimmed) {
            EvalRunnerAdapterConfigError::RawCredentialMaterialRejected
        } else {
            EvalRunnerAdapterConfigError::NonOpaqueCredentialHandleRef
        });
    }
    Ok(())
}

fn validate_audit_tap_ref(value: &str) -> Result<(), EvalRunnerAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(EvalRunnerAdapterConfigError::EmptyAuditTapRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(EvalRunnerAdapterConfigError::InvalidAuditTapRef);
    }
    Ok(())
}

fn validate_runner_audience_ref(value: &str) -> Result<(), EvalRunnerAdapterConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(EvalRunnerAdapterConfigError::EmptyRunnerAudienceRef);
    }
    if !is_safe_double_colon_ref(trimmed) {
        return Err(EvalRunnerAdapterConfigError::InvalidRunnerAudienceRef);
    }
    Ok(())
}

fn validate_dispatch_request(
    request: &EvalRunnerDispatchRequest,
) -> Result<(), EvalRunnerDispatchFailure> {
    require_metadata(
        "eval-runner:idempotency_key_required",
        "validation:eval-runner-idempotency-key",
        &request.idempotency_key,
    )?;

    let domain = &request.domain_request;
    let eval_set = &domain.eval_set;
    require_opaque(
        "eval-runner:tenant_required",
        "validation:eval-runner-tenant",
        &domain.tenant_id,
    )?;
    require_opaque(
        "eval-runner:principal_required",
        "validation:eval-runner-principal",
        &domain.principal_id,
    )?;
    require_opaque(
        "eval-runner:surface_required",
        "validation:eval-runner-surface",
        &domain.eval_surface,
    )?;
    require_opaque(
        "eval-runner:request_evidence_required",
        "validation:eval-runner-request-evidence",
        &domain.request_evidence_ref,
    )?;
    require_opaque(
        "eval-runner:trace_context_required",
        "validation:eval-runner-trace-context",
        &domain.trace_context_ref,
    )?;
    require_opaque(
        "eval-runner:policy_decision_required",
        "validation:eval-runner-policy-decision",
        &domain.policy_decision_ref,
    )?;
    require_opaque(
        "eval-runner:eval_registry_snapshot_required",
        "validation:eval-runner-registry-snapshot",
        &domain.policy_decision.eval_registry_snapshot_ref,
    )?;
    require_opaque(
        "eval-runner:eval_set_id_required",
        "validation:eval-runner-eval_set-id",
        &eval_set.eval_set_id,
    )?;
    require_opaque(
        "eval-runner:model_ref_required",
        "validation:eval-runner-model-ref",
        &eval_set.model_ref,
    )?;
    require_opaque(
        "eval-runner:dataset_snapshot_ref_must_be_opaque",
        "validation:eval-runner-dataset-snapshot",
        &eval_set.dataset_snapshot_ref,
    )?;
    require_opaque(
        "eval-runner:route_evidence_required",
        "validation:eval-runner-route-evidence",
        &eval_set.route_evidence_ref,
    )?;
    require_opaque(
        "eval-runner:guardrail_evidence_required",
        "validation:eval-runner-guardrail-evidence",
        &eval_set.guardrail_evidence_ref,
    )?;
    if eval_set.cases.is_empty() {
        return Err(dispatch_failure(
            "eval-runner:case_metadata_required",
            "validation:eval-runner-case-metadata",
        ));
    }
    for case in &eval_set.cases {
        require_metadata(
            "eval-runner:case_id_required",
            "validation:eval-runner-case-id",
            &case.case_id,
        )?;
        require_opaque(
            "eval-runner:case_evaluator_evidence_must_be_opaque",
            "validation:eval-runner-case-evidence",
            &case.evaluator_evidence_ref,
        )?;
    }

    validate_receipt_binding(request)
}

fn validate_receipt_binding(
    request: &EvalRunnerDispatchRequest,
) -> Result<(), EvalRunnerDispatchFailure> {
    let receipt = &request.usecase_receipt;
    let domain = &request.domain_request;
    let eval_set = &domain.eval_set;
    if receipt.idempotency_key != request.idempotency_key
        || receipt.tenant_id != domain.tenant_id
        || receipt.principal_id != domain.principal_id
        || receipt.eval_surface != domain.eval_surface
        || receipt.eval_set_id != eval_set.eval_set_id
        || receipt.model_ref != eval_set.model_ref
    {
        return Err(dispatch_failure(
            "eval-runner:usecase_receipt_binding_mismatch",
            "validation:eval-runner-usecase-receipt-binding",
        ));
    }
    if receipt.status != EvalUsecaseStatus::Evaluated {
        return Err(dispatch_failure(
            "eval-runner:usecase_receipt_not_evaluated",
            "validation:eval-runner-usecase-receipt-status",
        ));
    }
    if receipt.eval_set_status.is_none() {
        return Err(dispatch_failure(
            "eval-runner:usecase_receipt_missing_eval_set_status",
            "validation:eval-runner-eval_set-status",
        ));
    }
    Ok(())
}

fn receipt_from_status(
    status: &EvalRunnerStatus,
) -> Result<EvalRunnerDispatchReceipt, EvalRunnerDispatchFailure> {
    match status {
        EvalRunnerStatus::Accepted {
            runner_request_ref,
            run_ref,
            evidence_ref,
        } => ok_receipt(
            EvalRunnerDispatchStatus::Accepted,
            Some(runner_request_ref),
            Some(run_ref),
            None,
            None,
            evidence_ref,
        ),
        EvalRunnerStatus::Queued {
            runner_request_ref,
            queue_ref,
            evidence_ref,
        } => ok_receipt(
            EvalRunnerDispatchStatus::Queued,
            Some(runner_request_ref),
            None,
            Some(queue_ref),
            None,
            evidence_ref,
        ),
        EvalRunnerStatus::Completed {
            runner_request_ref,
            run_ref,
            report_ref,
            evidence_ref,
        } => ok_receipt(
            EvalRunnerDispatchStatus::Completed,
            Some(runner_request_ref),
            Some(run_ref),
            None,
            Some(report_ref),
            evidence_ref,
        ),
        EvalRunnerStatus::Denied { evidence_ref } => {
            Err(dispatch_failure("eval-runner:denied", evidence_ref))
        }
        EvalRunnerStatus::RateLimited { evidence_ref } => {
            Err(dispatch_failure("eval-runner:rate_limited", evidence_ref))
        }
        EvalRunnerStatus::RunnerError { evidence_ref } => {
            Err(dispatch_failure("eval-runner:runner_error", evidence_ref))
        }
        EvalRunnerStatus::AuthError { evidence_ref } => {
            Err(dispatch_failure("eval-runner:auth_error", evidence_ref))
        }
        EvalRunnerStatus::InvalidRequest { evidence_ref } => Err(dispatch_failure(
            "eval-runner:invalid_request",
            evidence_ref,
        )),
        EvalRunnerStatus::Timeout { evidence_ref } => {
            Err(dispatch_failure("eval-runner:timeout", evidence_ref))
        }
    }
}

fn ok_receipt(
    status: EvalRunnerDispatchStatus,
    runner_request_ref: Option<&String>,
    run_ref: Option<&String>,
    queue_ref: Option<&String>,
    report_ref: Option<&String>,
    evidence_ref: &str,
) -> Result<EvalRunnerDispatchReceipt, EvalRunnerDispatchFailure> {
    let refs = [runner_request_ref, run_ref, queue_ref, report_ref];
    if refs
        .into_iter()
        .flatten()
        .any(|value| !is_safe_opaque_ref(value))
        || !is_safe_opaque_ref(evidence_ref)
    {
        return Err(dispatch_failure(
            "eval-runner:invalid_runner_status",
            "validation:eval-runner-status-metadata",
        ));
    }
    Ok(EvalRunnerDispatchReceipt {
        status,
        runner_request_ref: runner_request_ref.cloned(),
        run_ref: run_ref.cloned(),
        queue_ref: queue_ref.cloned(),
        report_ref: report_ref.cloned(),
        evidence_ref: evidence_ref.to_owned(),
    })
}

fn dispatch_evidence_refs(request: &EvalRunnerDispatchRequest) -> Vec<String> {
    let eval_set = &request.domain_request.eval_set;
    let mut refs = vec![
        request.domain_request.request_evidence_ref.clone(),
        request.domain_request.trace_context_ref.clone(),
        request.domain_request.policy_decision_ref.clone(),
        request
            .domain_request
            .policy_decision
            .eval_registry_snapshot_ref
            .clone(),
        eval_set.route_evidence_ref.clone(),
        eval_set.guardrail_evidence_ref.clone(),
        eval_set.dataset_snapshot_ref.clone(),
    ];
    refs.extend(request.usecase_receipt.evidence_refs.clone());
    refs.extend(
        eval_set
            .cases
            .iter()
            .map(|case| case.evaluator_evidence_ref.clone()),
    );
    sorted_unique(refs)
}

fn require_metadata(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), EvalRunnerDispatchFailure> {
    if is_safe_metadata_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn require_opaque(
    reason: &str,
    evidence_ref: &str,
    value: &str,
) -> Result<(), EvalRunnerDispatchFailure> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(dispatch_failure(reason, evidence_ref))
    }
}

fn dispatch_failure(reason: &str, evidence_ref: &str) -> EvalRunnerDispatchFailure {
    EvalRunnerDispatchFailure {
        reason: reason.to_owned(),
        evidence_ref: if is_safe_metadata_ref(evidence_ref) {
            evidence_ref.to_owned()
        } else {
            "eval-runner:error:unsafe-evidence-ref".to_owned()
        },
    }
}

fn normalized_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_owned()
}

fn is_local_endpoint(endpoint: &str) -> bool {
    let lower = endpoint.to_ascii_lowercase();
    lower.starts_with("https://localhost")
        || lower.starts_with("https://127.")
        || lower.starts_with("https://[::1]")
        || lower.starts_with("https://0.0.0.0")
}

fn is_safe_double_colon_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains("://")
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_opaque_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("customer message")
        || lower.contains("write an email")
        || lower.contains("model answer")
        || lower.contains("prompt=")
        || lower.contains("completion=")
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

    #[test]
    fn builds_metadata_only_hosted_eval_envelope() {
        let mut adapter = valid_adapter(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });

        let receipt = adapter
            .dispatch(valid_request())
            .expect("runner accepts request");
        let envelope = adapter.last_envelope().expect("envelope recorded");

        assert_eq!(envelope.method, EvalRunnerHttpMethod::Post);
        assert_eq!(envelope.path, "/v1/eval-runs");
        assert_eq!(envelope.tenant_id, "tenant:alpha");
        assert_eq!(envelope.eval_set_id, "eval_set:release-gate");
        assert_eq!(
            envelope.dataset_snapshot_ref,
            "dataset://evals/releases/2026-05-23"
        );
        assert_eq!(envelope.case_count, 3);
        assert_eq!(
            envelope.transport_mode,
            EvalRunnerTransportMode::EnvelopeOnly
        );
        assert_eq!(
            envelope.credential_handle_ref,
            "secretref://ten_a/eval-runner/byok"
        );
        assert!(
            envelope
                .evidence_refs
                .contains(&"eval-registry:snapshot:release:1".to_owned())
        );
        assert_eq!(receipt.status, EvalRunnerDispatchStatus::Accepted);
        assert_eq!(receipt.run_ref, Some("eval-runner://runs/run-1".to_owned()));
    }

    #[test]
    fn rejects_raw_secret_like_credential_handles() {
        let config = EvalRunnerAdapterConfig::new(
            "https://eval-runner.oyatie.internal",
            "sk-test-raw-secret",
            "audit://tap/intelligence/eval",
            "audience://intelligence/eval-runner",
        );

        let error = IntelligenceEvalAdapter::try_new(
            config,
            EvalRunnerStatus::Timeout {
                evidence_ref: "eval-runner:error:timeout".to_owned(),
            },
        )
        .expect_err("raw secret handle rejected");

        assert_eq!(
            error,
            EvalRunnerAdapterConfigError::RawCredentialMaterialRejected
        );
    }

    #[test]
    fn rejects_non_https_and_localhost_endpoints() {
        let non_https = EvalRunnerAdapterConfig::new(
            "http://eval-runner.oyatie.internal",
            "secretref://ten_a/eval-runner/byok",
            "audit://tap/intelligence/eval",
            "audience://intelligence/eval-runner",
        );
        let local = EvalRunnerAdapterConfig::new(
            "https://localhost:9443",
            "secretref://ten_a/eval-runner/byok",
            "audit://tap/intelligence/eval",
            "audience://intelligence/eval-runner",
        );

        assert_eq!(
            IntelligenceEvalAdapter::try_new(
                non_https,
                EvalRunnerStatus::Timeout {
                    evidence_ref: "eval-runner:error:timeout".to_owned(),
                },
            )
            .expect_err("non-https rejected"),
            EvalRunnerAdapterConfigError::NonHttpsEndpoint
        );
        assert_eq!(
            IntelligenceEvalAdapter::try_new(
                local,
                EvalRunnerStatus::Timeout {
                    evidence_ref: "eval-runner:error:timeout".to_owned(),
                },
            )
            .expect_err("localhost rejected"),
            EvalRunnerAdapterConfigError::LocalEndpointDenied
        );
    }

    #[test]
    fn rejects_raw_prompt_or_output_shaped_eval_refs_before_envelope() {
        let mut request = valid_request();
        request.domain_request.eval_set.dataset_snapshot_ref =
            "raw prompt: write an email to the customer".to_owned();
        let mut adapter = valid_adapter(EvalRunnerStatus::Accepted {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            evidence_ref: "eval-runner:evidence:accepted".to_owned(),
        });

        let failure = adapter
            .dispatch(request)
            .expect_err("raw eval ref rejected");

        assert_eq!(
            failure.reason,
            "eval-runner:dataset_snapshot_ref_must_be_opaque"
        );
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn rejects_denied_or_mismatched_usecase_receipts_before_envelope() {
        let mut denied = valid_request();
        denied.usecase_receipt.status = EvalUsecaseStatus::Denied;
        denied.usecase_receipt.denial_kind = Some(EvalUsecaseDenialKind::DomainDenied);
        let mut mismatch = valid_request();
        mismatch.usecase_receipt.model_ref = "modelref://other/model".to_owned();
        let mut adapter = valid_adapter(EvalRunnerStatus::Queued {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            queue_ref: "eval-runner://queues/q-1".to_owned(),
            evidence_ref: "eval-runner:evidence:queued".to_owned(),
        });

        assert_eq!(
            adapter
                .dispatch(denied)
                .expect_err("denied receipt rejected")
                .reason,
            "eval-runner:usecase_receipt_not_evaluated"
        );
        assert!(adapter.last_envelope().is_none());
        assert_eq!(
            adapter
                .dispatch(mismatch)
                .expect_err("mismatched receipt rejected")
                .reason,
            "eval-runner:usecase_receipt_binding_mismatch"
        );
        assert!(adapter.last_envelope().is_none());
    }

    #[test]
    fn maps_runner_outcomes_distinctly() {
        let statuses = [
            (
                EvalRunnerStatus::RateLimited {
                    evidence_ref: "eval-runner:error:429".to_owned(),
                },
                "eval-runner:rate_limited",
            ),
            (
                EvalRunnerStatus::RunnerError {
                    evidence_ref: "eval-runner:error:500".to_owned(),
                },
                "eval-runner:runner_error",
            ),
            (
                EvalRunnerStatus::AuthError {
                    evidence_ref: "eval-runner:error:auth".to_owned(),
                },
                "eval-runner:auth_error",
            ),
            (
                EvalRunnerStatus::InvalidRequest {
                    evidence_ref: "eval-runner:error:invalid".to_owned(),
                },
                "eval-runner:invalid_request",
            ),
            (
                EvalRunnerStatus::Timeout {
                    evidence_ref: "eval-runner:error:timeout".to_owned(),
                },
                "eval-runner:timeout",
            ),
        ];

        for (status, expected_reason) in statuses {
            let mut adapter = valid_adapter(status);
            assert_eq!(
                adapter
                    .dispatch(valid_request())
                    .expect_err("runner error mapped")
                    .reason,
                expected_reason
            );
            assert!(adapter.last_envelope().is_some());
        }
    }

    #[test]
    fn envelope_and_receipts_never_contain_raw_prompt_output_or_secret_bytes() {
        let mut adapter = valid_adapter(EvalRunnerStatus::Completed {
            runner_request_ref: "eval-runner://requests/req-1".to_owned(),
            run_ref: "eval-runner://runs/run-1".to_owned(),
            report_ref: "eval-report://runs/run-1/report".to_owned(),
            evidence_ref: "eval-runner:evidence:completed".to_owned(),
        });

        let receipt = adapter.dispatch(valid_request()).expect("runner completed");
        let debug = format!("{:?}{:?}", adapter.last_envelope(), receipt);

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email to the customer"));
        assert!(!debug.contains("raw model answer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
    }

    fn valid_adapter(status: EvalRunnerStatus) -> IntelligenceEvalAdapter {
        IntelligenceEvalAdapter::try_new(
            EvalRunnerAdapterConfig::new(
                "https://eval-runner.oyatie.internal/",
                "secretref://ten_a/eval-runner/byok",
                "audit://tap/intelligence/eval",
                "audience://intelligence/eval-runner",
            ),
            status,
        )
        .expect("valid adapter config")
    }

    fn valid_request() -> EvalRunnerDispatchRequest {
        let domain_request = sample_domain_request("eval_set:release-gate");
        EvalRunnerDispatchRequest {
            idempotency_key: "idem:eval-runner:1".to_owned(),
            usecase_receipt: EvalUsecaseReceipt {
                idempotency_key: "idem:eval-runner:1".to_owned(),
                tenant_id: domain_request.tenant_id.clone(),
                principal_id: domain_request.principal_id.clone(),
                eval_surface: domain_request.eval_surface.clone(),
                eval_set_id: domain_request.eval_set.eval_set_id.clone(),
                model_ref: domain_request.eval_set.model_ref.clone(),
                status: EvalUsecaseStatus::Evaluated,
                denial_kind: None,
                domain_denial_kind: None,
                eval_set_status: Some(EvalSetStatus::Passed),
                failure_kinds: Vec::new(),
                evidence_refs: vec!["eval-usecase:evidence:evaluated".to_owned()],
            },
            domain_request,
        }
    }

    fn sample_domain_request(eval_set_id: &str) -> DomainEvalSetRequest {
        DomainEvalSetRequest {
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            eval_surface: "surface:release-gate".to_owned(),
            request_evidence_ref: "request:evidence:eval-runner:1".to_owned(),
            trace_context_ref: "trace:eval-runner:1".to_owned(),
            policy_decision_ref: "policy:evidence:eval-runner:1".to_owned(),
            policy_decision: sample_policy(),
            eval_set: sample_eval_set(eval_set_id),
        }
    }

    fn sample_policy() -> EvalPolicyDecision {
        EvalPolicyDecision {
            decision_id: "eval-policy-decision:runner:1".to_owned(),
            tenant_id: "tenant:alpha".to_owned(),
            principal_id: "principal:eval-owner".to_owned(),
            allowed_surfaces: vec!["surface:release-gate".to_owned()],
            allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
            allowed_dataset_snapshot_refs: vec!["dataset://evals/releases/2026-05-23".to_owned()],
            allowed_case_kinds: vec![
                EvalCaseKind::Adversarial,
                EvalCaseKind::Golden,
                EvalCaseKind::Linguistic,
                EvalCaseKind::Regression,
                EvalCaseKind::Safety,
            ],
            min_case_count: 3,
            max_case_count: 10,
            min_pass_rate_bps: 8_000,
            max_safety_violation_rate_bps: 0,
            evidence_ref: "policy:evidence:eval-runner:1".to_owned(),
            eval_registry_snapshot_ref: "eval-registry:snapshot:release:1".to_owned(),
        }
    }

    fn sample_eval_set(eval_set_id: &str) -> EvalSet {
        EvalSet {
            eval_set_id: eval_set_id.to_owned(),
            model_ref: "modelref://openai/gpt-preview".to_owned(),
            route_evidence_ref: "route:evidence:eval-runner:1".to_owned(),
            guardrail_evidence_ref: "guardrail:evidence:eval-runner:1".to_owned(),
            dataset_snapshot_ref: "dataset://evals/releases/2026-05-23".to_owned(),
            thresholds: EvalSetThresholds {
                min_pass_rate_bps: 8_000,
                max_safety_violation_rate_bps: 0,
                require_golden: true,
                require_adversarial: true,
                require_linguistic: true,
            },
            cases: vec![
                case(
                    "case-golden-runner-1",
                    EvalCaseKind::Golden,
                    EvalCaseOutcome::Passed,
                    9_500,
                    "eval:case:runner:golden:1",
                ),
                case(
                    "case-adversarial-runner-1",
                    EvalCaseKind::Adversarial,
                    EvalCaseOutcome::Passed,
                    8_800,
                    "eval:case:runner:adversarial:1",
                ),
                case(
                    "case-linguistic-runner-1",
                    EvalCaseKind::Linguistic,
                    EvalCaseOutcome::Passed,
                    8_700,
                    "eval:case:runner:linguistic:1",
                ),
            ],
        }
    }

    fn case(
        case_id: &str,
        kind: EvalCaseKind,
        outcome: EvalCaseOutcome,
        score_bps: u16,
        evidence_ref: &str,
    ) -> EvalCaseResult {
        EvalCaseResult {
            case_id: case_id.to_owned(),
            kind,
            outcome,
            score_bps,
            evaluator_evidence_ref: evidence_ref.to_owned(),
        }
    }
}
