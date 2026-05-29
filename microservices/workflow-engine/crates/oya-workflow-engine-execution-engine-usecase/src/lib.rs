//! Workflow-engine execution-engine usecase foundation.
//!
//! The usecase composes request/idempotency/trace validation, abstract run-store,
//! step-dispatcher, retry-policy, and SLA-timer ports, plus the policy-bound
//! execution-engine domain. It is source-level only: no concrete storage,
//! network, wall-clock, random, queue, signing, Valkey, Postgres, or cloud
//! runtime work is performed here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use oya_workflow_engine_execution_engine_domain::{
    ExecutionDispatchError, ExecutionDomainCommandKind, ExecutionDomainDecision,
    ExecutionDomainDenialKind, ExecutionDomainOrigin, ExecutionEngineDomainRequest,
    ExecutionEngineKernelError, ExecutionStoreError, RetryAttempt, RetryPolicyEvaluator, SlaTimer,
    SlaTimerStore, StepDispatcher, StepExecution, StepExecutionStatus, WorkflowExecutionStatus,
    WorkflowRun, WorkflowRunStore, evaluate_execution_domain,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineUsecaseInput {
    pub request_id: String,                           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                      // data_class: INTERNAL_ONLY
    pub trace_ref: String,                            // data_class: INTERNAL_ONLY
    pub expected_run_version: u64,                    // data_class: INTERNAL_ONLY
    pub domain_request: ExecutionEngineDomainRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionUsecaseStatus {
    Applied,
    DispatchDenied,
    DispatchUnavailable,
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
    RetryPolicyRejected,
    StoreConflict,
    StoreUnavailable,
    TimerUnavailable,
}

impl ExecutionUsecaseStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::DispatchDenied => "dispatch-denied",
            Self::DispatchUnavailable => "dispatch-unavailable",
            Self::DomainDenied => "domain-denied",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::RetryPolicyRejected => "retry-policy-rejected",
            Self::StoreConflict => "store-conflict",
            Self::StoreUnavailable => "store-unavailable",
            Self::TimerUnavailable => "timer-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionAuditEventKind {
    DispatchDenied,
    DispatchUnavailable,
    DomainDenied,
    ExecutionApplied,
    ExecutionInvalid,
    ExecutionRequested,
    IdempotencyConflict,
    RetryPolicyRejected,
    StoreConflict,
    StoreUnavailable,
}

impl ExecutionAuditEventKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::DispatchDenied => "dispatch-denied",
            Self::DispatchUnavailable => "dispatch-unavailable",
            Self::DomainDenied => "domain-denied",
            Self::ExecutionApplied => "execution-applied",
            Self::ExecutionInvalid => "execution-invalid",
            Self::ExecutionRequested => "execution-requested",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::RetryPolicyRejected => "retry-policy-rejected",
            Self::StoreConflict => "store-conflict",
            Self::StoreUnavailable => "store-unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionAuditEvent {
    pub kind: ExecutionAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub run_id: String,                // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionUsecaseReceipt {
    pub status: ExecutionUsecaseStatus, // data_class: INTERNAL_ONLY
    pub command: ExecutionDomainCommandKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub run_id: String,                 // data_class: INTERNAL_ONLY
    pub run_status: Option<WorkflowExecutionStatus>, // data_class: PUBLIC
    pub step_status: Option<StepExecutionStatus>, // data_class: PUBLIC
    pub retry_delay_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub domain_denial_kind: Option<ExecutionDomainDenialKind>, // data_class: INTERNAL_ONLY
    pub store_expected_version: Option<u64>, // data_class: INTERNAL_ONLY
    pub store_observed_version: Option<u64>, // data_class: INTERNAL_ONLY
    pub audit_events: Vec<ExecutionAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutionUsecaseIntent {
    fingerprint: String,
}

#[derive(Default, Debug)]
pub struct ExecutionEngineUsecase {
    receipts_by_idempotency_key:
        BTreeMap<String, (ExecutionUsecaseIntent, ExecutionUsecaseReceipt)>,
}

impl ExecutionEngineUsecase {
    pub fn apply<S, D, R, T>(
        &mut self,
        store: &mut S,
        dispatcher: &mut D,
        retry_policy: &R,
        timers: &mut T,
        input: ExecutionEngineUsecaseInput,
    ) -> ExecutionUsecaseReceipt
    where
        S: WorkflowRunStore,
        D: StepDispatcher,
        R: RetryPolicyEvaluator,
        T: SlaTimerStore,
    {
        if let Some(receipt_value) = invalid_input_receipt(&input) {
            return receipt_value;
        }

        let intent = ExecutionUsecaseIntent {
            fingerprint: canonical_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.receipts_by_idempotency_key.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return idempotency_conflict_receipt(&input);
        }

        let mut requested = requested_event(&input);
        let mut domain_request = input.domain_request.clone();
        let current_run =
            match store.load_run(&domain_request.run.tenant_id, &domain_request.run.run_id) {
                Ok(run) => run,
                Err(error) => {
                    return store_error_receipt(
                        &input,
                        requested,
                        store_error_refs(error),
                        ExecutionUsecaseStatus::StoreUnavailable,
                    );
                }
            };
        if let Some(current_run) = current_run {
            domain_request.run = current_run;
        }

        let domain_decision = evaluate_execution_domain(domain_request.clone());
        let domain_receipt = match domain_decision {
            ExecutionDomainDecision::Denied(denial) => {
                let mut refs = denial.audit_refs.clone();
                refs.push("workflow-execution-usecase:domain-denied".to_owned());
                let refs = sorted_unique(refs);
                let receipt_value = receipt(
                    ExecutionUsecaseStatus::DomainDenied,
                    input.domain_request.command,
                    &denial.tenant_id,
                    &denial.run_id,
                    None,
                    None,
                    None,
                    Some(denial.kind),
                    None,
                    None,
                    vec![
                        requested,
                        audit_event(
                            ExecutionAuditEventKind::DomainDenied,
                            &denial.tenant_id,
                            &denial.run_id,
                            refs.clone(),
                        ),
                    ],
                    refs,
                );
                self.cache_receipt(input.idempotency_key.clone(), intent, receipt_value.clone());
                return receipt_value;
            }
            ExecutionDomainDecision::Accepted(receipt_value) => receipt_value,
        };

        requested.evidence_refs =
            sorted_unique([requested.evidence_refs, domain_receipt.audit_refs.clone()].concat());

        if input.domain_request.command == ExecutionDomainCommandKind::StartRun
            && let Err(error) = store.create_run(domain_request.run.clone())
        {
            return store_error_receipt(
                &input,
                requested,
                store_error_refs(error),
                ExecutionUsecaseStatus::StoreUnavailable,
            );
        }

        if let Some(step) = step_with_domain_status(&domain_request, domain_receipt.step_status)
            && let Err(error) = store.save_step(step)
        {
            return store_error_receipt(
                &input,
                requested,
                store_error_refs(error),
                ExecutionUsecaseStatus::StoreUnavailable,
            );
        }

        if let Err(error) = store.update_run_status(
            &domain_receipt.tenant_id,
            &domain_receipt.run_id,
            input.expected_run_version,
            domain_receipt.next_run_status,
            "workflow-execution-usecase:run-status-update",
        ) {
            return store_error_receipt(
                &input,
                requested,
                store_error_refs(error),
                ExecutionUsecaseStatus::StoreUnavailable,
            );
        }

        let retry_delay_seconds =
            if input.domain_request.command == ExecutionDomainCommandKind::ScheduleRetry {
                let Some(retry_attempt) = domain_request.retry_attempt.as_ref() else {
                    return invalid_shape_after_domain(&input, requested);
                };
                match retry_policy.next_delay_seconds(retry_attempt) {
                    Ok(delay) => delay,
                    Err(error) => {
                        let refs = sorted_unique(vec![
                            kernel_error_ref(error).to_owned(),
                            "workflow-execution-usecase:retry-policy-rejected".to_owned(),
                        ]);
                        return receipt(
                            ExecutionUsecaseStatus::RetryPolicyRejected,
                            input.domain_request.command,
                            &domain_receipt.tenant_id,
                            &domain_receipt.run_id,
                            Some(domain_receipt.next_run_status),
                            domain_receipt.step_status,
                            None,
                            None,
                            None,
                            None,
                            vec![
                                requested,
                                audit_event(
                                    ExecutionAuditEventKind::RetryPolicyRejected,
                                    &domain_receipt.tenant_id,
                                    &domain_receipt.run_id,
                                    refs.clone(),
                                ),
                            ],
                            refs,
                        );
                    }
                }
            } else {
                None
            };

        if input.domain_request.command == ExecutionDomainCommandKind::ArmSlaTimer {
            let Some(timer) = domain_request.sla_timer.clone() else {
                return invalid_shape_after_domain(&input, requested);
            };
            if let Err(error) = timers.arm_timer(timer) {
                return timer_error_receipt(&input, requested, error);
            }
        }

        if domain_receipt.dispatch_required {
            let step_index = domain_request
                .step
                .as_ref()
                .map_or(0, |step| step.step_index);
            if let Err(error) = dispatcher.dispatch_step(
                &domain_receipt.tenant_id,
                &domain_receipt.run_id,
                step_index,
                "workflow-execution-usecase:dispatch-step",
            ) {
                return dispatch_error_receipt(&input, requested, error);
            }
        }

        let mut refs = domain_receipt.audit_refs.clone();
        refs.push("workflow-execution-usecase:applied".to_owned());
        if retry_delay_seconds.is_some() {
            refs.push("workflow-execution-usecase:retry-delay-computed".to_owned());
        }
        let refs = sorted_unique(refs);
        let receipt_value = receipt(
            ExecutionUsecaseStatus::Applied,
            input.domain_request.command,
            &domain_receipt.tenant_id,
            &domain_receipt.run_id,
            Some(domain_receipt.next_run_status),
            domain_receipt.step_status,
            retry_delay_seconds,
            None,
            None,
            None,
            vec![
                requested,
                audit_event(
                    ExecutionAuditEventKind::ExecutionApplied,
                    &domain_receipt.tenant_id,
                    &domain_receipt.run_id,
                    refs.clone(),
                ),
            ],
            refs,
        );
        self.cache_receipt(input.idempotency_key.clone(), intent, receipt_value.clone());
        receipt_value
    }

    pub fn cached_receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn cache_receipt(
        &mut self,
        idempotency_key: String,
        intent: ExecutionUsecaseIntent,
        receipt_value: ExecutionUsecaseReceipt,
    ) {
        self.receipts_by_idempotency_key
            .insert(idempotency_key, (intent, receipt_value));
    }
}

fn invalid_input_receipt(input: &ExecutionEngineUsecaseInput) -> Option<ExecutionUsecaseReceipt> {
    let mut refs = Vec::new();
    if !is_safe_ref(&input.request_id) {
        refs.push("validation:execution-request-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.idempotency_key) {
        refs.push("validation:execution-idempotency-key-invalid".to_owned());
    }
    if !is_safe_ref(&input.trace_ref) {
        refs.push("validation:execution-trace-ref-invalid".to_owned());
    }
    if input.expected_run_version == 0 {
        refs.push("validation:execution-run-version-invalid".to_owned());
    }
    if has_unsafe_domain_metadata(&input.domain_request) {
        refs.push("validation:execution-domain-metadata-invalid".to_owned());
    }
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-execution-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    Some(receipt(
        ExecutionUsecaseStatus::InvalidInput,
        input.domain_request.command,
        &safe_tenant(&input.domain_request.run.tenant_id),
        &safe_ref(&input.domain_request.run.run_id, "redacted-invalid-run-id"),
        None,
        None,
        None,
        None,
        None,
        None,
        vec![audit_event(
            ExecutionAuditEventKind::ExecutionInvalid,
            &safe_tenant(&input.domain_request.run.tenant_id),
            &safe_ref(&input.domain_request.run.run_id, "redacted-invalid-run-id"),
            refs.clone(),
        )],
        refs,
    ))
}

fn idempotency_conflict_receipt(input: &ExecutionEngineUsecaseInput) -> ExecutionUsecaseReceipt {
    let refs = sorted_unique(vec![
        input.request_id.clone(),
        input.trace_ref.clone(),
        "workflow-execution-usecase:idempotency-conflict".to_owned(),
    ]);
    receipt(
        ExecutionUsecaseStatus::IdempotencyConflict,
        input.domain_request.command,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![audit_event(
            ExecutionAuditEventKind::IdempotencyConflict,
            &input.domain_request.run.tenant_id,
            &input.domain_request.run.run_id,
            refs.clone(),
        )],
        refs,
    )
}

fn requested_event(input: &ExecutionEngineUsecaseInput) -> ExecutionAuditEvent {
    audit_event(
        ExecutionAuditEventKind::ExecutionRequested,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
        ]),
    )
}

fn store_error_refs(error: ExecutionStoreError) -> Vec<String> {
    match error {
        ExecutionStoreError::Conflict {
            evidence_ref,
            expected_version,
            observed_version,
        } => sorted_unique(vec![
            evidence_ref,
            format!("workflow-execution-usecase:expected-version:{expected_version}"),
            format!("workflow-execution-usecase:observed-version:{observed_version}"),
            "workflow-execution-usecase:store-conflict".to_owned(),
        ]),
        ExecutionStoreError::Unavailable { evidence_ref } => sorted_unique(vec![
            evidence_ref,
            "workflow-execution-usecase:store-unavailable".to_owned(),
        ]),
    }
}

fn store_error_receipt(
    input: &ExecutionEngineUsecaseInput,
    requested: ExecutionAuditEvent,
    refs: Vec<String>,
    fallback_status: ExecutionUsecaseStatus,
) -> ExecutionUsecaseReceipt {
    let status = if refs
        .iter()
        .any(|value| value == "workflow-execution-usecase:store-conflict")
    {
        ExecutionUsecaseStatus::StoreConflict
    } else {
        fallback_status
    };
    let event_kind = match status {
        ExecutionUsecaseStatus::StoreConflict => ExecutionAuditEventKind::StoreConflict,
        _ => ExecutionAuditEventKind::StoreUnavailable,
    };
    let expected = parse_version_ref(&refs, "workflow-execution-usecase:expected-version:");
    let observed = parse_version_ref(&refs, "workflow-execution-usecase:observed-version:");
    receipt(
        status,
        input.domain_request.command,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        None,
        None,
        None,
        None,
        expected,
        observed,
        vec![
            requested,
            audit_event(
                event_kind,
                &input.domain_request.run.tenant_id,
                &input.domain_request.run.run_id,
                refs.clone(),
            ),
        ],
        refs,
    )
}

fn dispatch_error_receipt(
    input: &ExecutionEngineUsecaseInput,
    requested: ExecutionAuditEvent,
    error: ExecutionDispatchError,
) -> ExecutionUsecaseReceipt {
    let (status, event_kind, refs) = match error {
        ExecutionDispatchError::Denied { evidence_ref } => (
            ExecutionUsecaseStatus::DispatchDenied,
            ExecutionAuditEventKind::DispatchDenied,
            sorted_unique(vec![
                evidence_ref,
                "workflow-execution-usecase:dispatch-denied".to_owned(),
            ]),
        ),
        ExecutionDispatchError::Unavailable { evidence_ref } => (
            ExecutionUsecaseStatus::DispatchUnavailable,
            ExecutionAuditEventKind::DispatchUnavailable,
            sorted_unique(vec![
                evidence_ref,
                "workflow-execution-usecase:dispatch-unavailable".to_owned(),
            ]),
        ),
    };
    receipt(
        status,
        input.domain_request.command,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![
            requested,
            audit_event(
                event_kind,
                &input.domain_request.run.tenant_id,
                &input.domain_request.run.run_id,
                refs.clone(),
            ),
        ],
        refs,
    )
}

fn timer_error_receipt(
    input: &ExecutionEngineUsecaseInput,
    requested: ExecutionAuditEvent,
    error: ExecutionStoreError,
) -> ExecutionUsecaseReceipt {
    let refs = store_error_refs(error);
    receipt(
        ExecutionUsecaseStatus::TimerUnavailable,
        input.domain_request.command,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![
            requested,
            audit_event(
                ExecutionAuditEventKind::StoreUnavailable,
                &input.domain_request.run.tenant_id,
                &input.domain_request.run.run_id,
                refs.clone(),
            ),
        ],
        refs,
    )
}

fn invalid_shape_after_domain(
    input: &ExecutionEngineUsecaseInput,
    requested: ExecutionAuditEvent,
) -> ExecutionUsecaseReceipt {
    let refs = sorted_unique(vec![
        "workflow-execution-usecase:domain-shape-drift".to_owned(),
        "workflow-execution-usecase:invalid-input".to_owned(),
    ]);
    receipt(
        ExecutionUsecaseStatus::InvalidInput,
        input.domain_request.command,
        &input.domain_request.run.tenant_id,
        &input.domain_request.run.run_id,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![requested],
        refs,
    )
}

fn step_with_domain_status(
    request: &ExecutionEngineDomainRequest,
    status: Option<StepExecutionStatus>,
) -> Option<StepExecution> {
    let mut step = request.step.clone()?;
    if let Some(status) = status {
        step.status = status;
    }
    Some(step)
}

fn kernel_error_ref(error: ExecutionEngineKernelError) -> &'static str {
    match error {
        ExecutionEngineKernelError::InvalidAttempt => "kernel:invalid-attempt",
        ExecutionEngineKernelError::InvalidStepIndex => "kernel:invalid-step-index",
        ExecutionEngineKernelError::InvalidTimerDeadline => "kernel:invalid-timer-deadline",
        ExecutionEngineKernelError::UnsafeMetadata => "kernel:unsafe-metadata",
    }
}

#[allow(clippy::too_many_arguments)]
fn receipt(
    status: ExecutionUsecaseStatus,
    command: ExecutionDomainCommandKind,
    tenant_id: &str,
    run_id: &str,
    run_status: Option<WorkflowExecutionStatus>,
    step_status: Option<StepExecutionStatus>,
    retry_delay_seconds: Option<u64>,
    domain_denial_kind: Option<ExecutionDomainDenialKind>,
    store_expected_version: Option<u64>,
    store_observed_version: Option<u64>,
    audit_events: Vec<ExecutionAuditEvent>,
    evidence_refs: Vec<String>,
) -> ExecutionUsecaseReceipt {
    ExecutionUsecaseReceipt {
        status,
        command,
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
        run_status,
        step_status,
        retry_delay_seconds,
        domain_denial_kind,
        store_expected_version,
        store_observed_version,
        audit_events,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn audit_event(
    kind: ExecutionAuditEventKind,
    tenant_id: &str,
    run_id: &str,
    evidence_refs: Vec<String>,
) -> ExecutionAuditEvent {
    ExecutionAuditEvent {
        kind,
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn parse_version_ref(refs: &[String], prefix: &str) -> Option<u64> {
    refs.iter().find_map(|value| {
        value
            .strip_prefix(prefix)
            .and_then(|number| number.parse().ok())
    })
}

fn canonical_fingerprint(input: &ExecutionEngineUsecaseInput) -> String {
    let mut parts = vec![
        canonical_entry("request_id", &input.request_id),
        canonical_entry("trace_ref", &input.trace_ref),
        canonical_entry(
            "expected_run_version",
            &input.expected_run_version.to_string(),
        ),
        canonical_entry("command", input.domain_request.command.as_wire()),
        canonical_entry("origin", input.domain_request.origin.as_wire()),
        canonical_entry("tenant_id", &input.domain_request.expected_tenant_id),
        canonical_entry("run_id", &input.domain_request.run.run_id),
        canonical_entry("spec_id", &input.domain_request.expected_spec_id),
        canonical_entry("version_sha", &input.domain_request.expected_version_sha),
        canonical_entry("cell_id", &input.domain_request.expected_cell_id),
        canonical_entry("run_status", input.domain_request.run.status.as_wire()),
        canonical_entry("policy", &input.domain_request.policy_evidence_ref),
        canonical_entry("spec_integrity", &input.domain_request.spec_integrity_ref),
        canonical_entry("replay", &input.domain_request.replay_epoch_ref),
        canonical_entry("scheduler", &input.domain_request.scheduler_epoch_ref),
    ];
    if let Some(step) = &input.domain_request.step {
        parts.extend([
            canonical_entry("step_id", &step.step_id),
            canonical_entry("step_index", &step.step_index.to_string()),
            canonical_entry("step_attempt", &step.attempt.to_string()),
            canonical_entry("step_status", step.status.as_wire()),
        ]);
    }
    if let Some(retry) = &input.domain_request.retry_attempt {
        parts.extend([
            canonical_entry("retry_step_id", &retry.step_id),
            canonical_entry("retry_attempt", &retry.attempt.to_string()),
            canonical_entry("retry_policy", &retry.retry_policy_ref),
        ]);
    }
    if let Some(timer) = &input.domain_request.sla_timer {
        parts.extend([
            canonical_entry("timer_id", &timer.timer_id),
            canonical_entry("timer_deadline", &timer.deadline_epoch_seconds.to_string()),
        ]);
    }
    parts.concat()
}

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}{}:{}", label.len(), label, value.len(), value)
}

fn has_unsafe_domain_metadata(request: &ExecutionEngineDomainRequest) -> bool {
    !is_safe_tenant(&request.expected_tenant_id)
        || !is_safe_ref(&request.expected_spec_id)
        || !is_safe_ref(&request.expected_version_sha)
        || !is_safe_ref(&request.expected_cell_id)
        || !is_safe_ref(&request.policy_evidence_ref)
        || !is_safe_ref(&request.spec_integrity_ref)
        || !is_safe_ref(&request.replay_epoch_ref)
        || !is_safe_ref(&request.scheduler_epoch_ref)
        || !is_safe_run(&request.run)
        || request
            .step
            .as_ref()
            .is_some_and(|step| !is_safe_step(step))
        || request
            .retry_attempt
            .as_ref()
            .is_some_and(|retry| !is_safe_retry(retry))
        || request
            .sla_timer
            .as_ref()
            .is_some_and(|timer| !is_safe_timer(timer))
}

fn is_safe_run(run: &WorkflowRun) -> bool {
    is_safe_tenant(&run.tenant_id)
        && is_safe_ref(&run.run_id)
        && is_safe_ref(&run.spec_id)
        && is_safe_ref(&run.version_sha)
        && is_safe_ref(&run.active_cell_id)
        && run.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_step(step: &StepExecution) -> bool {
    is_safe_tenant(&step.tenant_id)
        && is_safe_ref(&step.run_id)
        && is_safe_ref(&step.step_id)
        && is_safe_ref(&step.idempotency_key)
        && step
            .lease_owner_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .side_effect_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .last_error_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_retry(retry: &RetryAttempt) -> bool {
    is_safe_tenant(&retry.tenant_id)
        && is_safe_ref(&retry.run_id)
        && is_safe_ref(&retry.step_id)
        && is_safe_ref(&retry.error_class_ref)
        && is_safe_ref(&retry.retry_policy_ref)
        && retry.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_timer(timer: &SlaTimer) -> bool {
    is_safe_ref(&timer.timer_id)
        && is_safe_tenant(&timer.tenant_id)
        && is_safe_ref(&timer.run_id)
        && timer.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn safe_tenant(value: &str) -> String {
    if is_safe_tenant(value) {
        value.to_owned()
    } else {
        "redacted-invalid-tenant-id".to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("payload")
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
    use std::cell::Cell;

    struct FakeStore {
        current: Option<WorkflowRun>,
        load_calls: Cell<usize>,
        created: Vec<WorkflowRun>,
        saved_steps: Vec<StepExecution>,
        updated_statuses: Vec<(String, WorkflowExecutionStatus)>,
        load_failure: bool,
        update_conflict: bool,
        save_failure: bool,
    }

    impl Default for FakeStore {
        fn default() -> Self {
            Self {
                current: Some(run_with_status(WorkflowExecutionStatus::Running, 7)),
                load_calls: Cell::new(0),
                created: Vec::new(),
                saved_steps: Vec::new(),
                updated_statuses: Vec::new(),
                load_failure: false,
                update_conflict: false,
                save_failure: false,
            }
        }
    }

    impl WorkflowRunStore for FakeStore {
        fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError> {
            self.created.push(run);
            Ok(())
        }

        fn load_run(
            &self,
            tenant_id: &str,
            run_id: &str,
        ) -> Result<Option<WorkflowRun>, ExecutionStoreError> {
            self.load_calls.set(self.load_calls.get() + 1);
            if self.load_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:load-redacted".to_owned(),
                });
            }
            assert_eq!(tenant_id, "ten_a");
            assert!(run_id.starts_with("run:execution-usecase"));
            Ok(self.current.clone())
        }

        fn update_run_status(
            &mut self,
            tenant_id: &str,
            run_id: &str,
            expected_version: u64,
            status: WorkflowExecutionStatus,
            evidence_ref: &str,
        ) -> Result<(), ExecutionStoreError> {
            if self.update_conflict {
                return Err(ExecutionStoreError::Conflict {
                    expected_version,
                    observed_version: expected_version + 1,
                    evidence_ref: evidence_ref.to_owned(),
                });
            }
            assert_eq!(tenant_id, "ten_a");
            self.updated_statuses.push((run_id.to_owned(), status));
            Ok(())
        }

        fn save_step(&mut self, step: StepExecution) -> Result<(), ExecutionStoreError> {
            if self.save_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:save-step-redacted".to_owned(),
                });
            }
            self.saved_steps.push(step);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeDispatcher {
        dispatched: Vec<(String, String, u32)>,
        denied: bool,
        unavailable: bool,
    }

    impl StepDispatcher for FakeDispatcher {
        fn dispatch_step(
            &mut self,
            tenant_id: &str,
            run_id: &str,
            step_index: u32,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionDispatchError> {
            if self.denied {
                return Err(ExecutionDispatchError::Denied {
                    evidence_ref: "dispatcher:denied".to_owned(),
                });
            }
            if self.unavailable {
                return Err(ExecutionDispatchError::Unavailable {
                    evidence_ref: "dispatcher:unavailable".to_owned(),
                });
            }
            self.dispatched
                .push((tenant_id.to_owned(), run_id.to_owned(), step_index));
            Ok(())
        }
    }

    struct FakeRetryPolicy {
        delay: Result<Option<u64>, ExecutionEngineKernelError>,
        calls: Cell<usize>,
    }

    impl Default for FakeRetryPolicy {
        fn default() -> Self {
            Self {
                delay: Ok(Some(30)),
                calls: Cell::new(0),
            }
        }
    }

    impl RetryPolicyEvaluator for FakeRetryPolicy {
        fn next_delay_seconds(
            &self,
            attempt: &RetryAttempt,
        ) -> Result<Option<u64>, ExecutionEngineKernelError> {
            self.calls.set(self.calls.get() + 1);
            assert_eq!(attempt.attempt, 2);
            self.delay
        }
    }

    #[derive(Default)]
    struct FakeTimers {
        armed: Vec<SlaTimer>,
        unavailable: bool,
    }

    impl SlaTimerStore for FakeTimers {
        fn arm_timer(&mut self, timer: SlaTimer) -> Result<(), ExecutionStoreError> {
            if self.unavailable {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "timer-store:unavailable".to_owned(),
                });
            }
            self.armed.push(timer);
            Ok(())
        }

        fn cancel_timer(
            &mut self,
            _tenant_id: &str,
            _timer_id: &str,
        ) -> Result<(), ExecutionStoreError> {
            Ok(())
        }

        fn fire_expired(
            &mut self,
            _tenant_id: &str,
            _now_epoch_seconds: u64,
        ) -> Result<Vec<SlaTimer>, ExecutionStoreError> {
            Ok(Vec::new())
        }
    }

    fn run_with_status(status: WorkflowExecutionStatus, version: u64) -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:execution-usecase:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec!["workflow-execution-usecase:requested".to_owned()],
        )
        .unwrap();
        run.status = status;
        run.version = version;
        run
    }

    fn step_with_status(status: StepExecutionStatus, attempt: u32) -> StepExecution {
        let mut step = StepExecution::new(
            "ten_a",
            "run:execution-usecase:1",
            "step:approve",
            0,
            attempt,
            "idempotency:step:approve:1",
            vec!["workflow-execution-usecase:step".to_owned()],
        )
        .unwrap();
        step.status = status;
        step
    }

    fn retry_attempt(attempt: u32) -> RetryAttempt {
        RetryAttempt::new(
            "ten_a",
            "run:execution-usecase:1",
            "step:approve",
            attempt,
            "error-class:retryable-http-503",
            "retry-policy:workflow-standard",
            vec!["workflow-execution-usecase:retry".to_owned()],
        )
        .unwrap()
    }

    fn timer() -> SlaTimer {
        SlaTimer::new(
            "timer:execution-usecase:1",
            "ten_a",
            "run:execution-usecase:1",
            Some(0),
            100,
            130,
            vec!["workflow-execution-usecase:sla".to_owned()],
        )
        .unwrap()
    }

    fn input(command: ExecutionDomainCommandKind) -> ExecutionEngineUsecaseInput {
        ExecutionEngineUsecaseInput {
            request_id: "req:execution-usecase:1".to_owned(),
            idempotency_key: "idem:execution-usecase:1".to_owned(),
            trace_ref: "trace:execution-usecase:1".to_owned(),
            expected_run_version: 7,
            domain_request: ExecutionEngineDomainRequest {
                run: run_with_status(WorkflowExecutionStatus::Running, 7),
                step: Some(step_with_status(StepExecutionStatus::Pending, 1)),
                retry_attempt: None,
                sla_timer: None,
                expected_tenant_id: "ten_a".to_owned(),
                expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
                expected_version_sha: "sha256:spec-v1".to_owned(),
                expected_cell_id: "cell:use1:a".to_owned(),
                policy_evidence_ref: "cedar://workflow/execution/dispatch".to_owned(),
                spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
                replay_epoch_ref: "replay-epoch:execution-usecase:1".to_owned(),
                scheduler_epoch_ref: "scheduler-epoch:execution-usecase:1".to_owned(),
                sla_reference_epoch_seconds: 0,
                command,
                origin: ExecutionDomainOrigin::WorkerScheduler,
            },
        }
    }

    fn apply(
        usecase: &mut ExecutionEngineUsecase,
        store: &mut FakeStore,
        dispatcher: &mut FakeDispatcher,
        retry_policy: &FakeRetryPolicy,
        timers: &mut FakeTimers,
        input: ExecutionEngineUsecaseInput,
    ) -> ExecutionUsecaseReceipt {
        usecase.apply(store, dispatcher, retry_policy, timers, input)
    }

    #[test]
    fn dispatch_usecase_loads_run_delegates_domain_updates_store_and_dispatches() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            input(ExecutionDomainCommandKind::DispatchStep),
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(receipt.run_status, Some(WorkflowExecutionStatus::Running));
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Leased));
        assert_eq!(store.load_calls.get(), 1);
        assert_eq!(store.saved_steps[0].status, StepExecutionStatus::Leased);
        assert_eq!(store.updated_statuses.len(), 1);
        assert_eq!(
            dispatcher.dispatched,
            vec![("ten_a".to_owned(), "run:execution-usecase:1".to_owned(), 0)]
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"workflow-execution-usecase:applied".to_owned())
        );
    }

    #[test]
    fn invalid_request_or_trace_metadata_denies_before_store_dispatch_or_cache() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();
        let mut invalid = input(ExecutionDomainCommandKind::DispatchStep);
        invalid.trace_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            invalid,
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::InvalidInput);
        assert_eq!(store.load_calls.get(), 0);
        assert!(dispatcher.dispatched.is_empty());
        assert_eq!(usecase.cached_receipt_count(), 0);
        let rendered = format!("{receipt:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn domain_denial_does_not_update_store_or_dispatch() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();
        let mut invalid = input(ExecutionDomainCommandKind::DispatchStep);
        invalid.domain_request.expected_cell_id = "cell:other".to_owned();

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            invalid,
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::DomainDenied);
        assert_eq!(
            receipt.domain_denial_kind,
            Some(ExecutionDomainDenialKind::ScopeMismatch)
        );
        assert!(store.updated_statuses.is_empty());
        assert!(dispatcher.dispatched.is_empty());
    }

    #[test]
    fn idempotent_replay_returns_cached_receipt_and_conflict_does_not_dispatch_again() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();

        let first = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            input(ExecutionDomainCommandKind::DispatchStep),
        );
        let replay = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            input(ExecutionDomainCommandKind::DispatchStep),
        );
        let mut conflict = input(ExecutionDomainCommandKind::DispatchStep);
        conflict.trace_ref = "trace:execution-usecase:other".to_owned();
        let conflict = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            conflict,
        );

        assert_eq!(first, replay);
        assert_eq!(dispatcher.dispatched.len(), 1);
        assert_eq!(conflict.status, ExecutionUsecaseStatus::IdempotencyConflict);
        assert_eq!(dispatcher.dispatched.len(), 1);
    }

    #[test]
    fn store_conflict_is_sanitized_and_skips_dispatch() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore {
            update_conflict: true,
            ..FakeStore::default()
        };
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            input(ExecutionDomainCommandKind::DispatchStep),
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::StoreConflict);
        assert_eq!(receipt.store_expected_version, Some(7));
        assert_eq!(receipt.store_observed_version, Some(8));
        assert!(dispatcher.dispatched.is_empty());
    }

    #[test]
    fn dispatcher_unavailable_maps_to_metadata_only_receipt() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher {
            unavailable: true,
            ..FakeDispatcher::default()
        };
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            input(ExecutionDomainCommandKind::DispatchStep),
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::DispatchUnavailable);
        assert!(
            receipt
                .evidence_refs
                .contains(&"dispatcher:unavailable".to_owned())
        );
    }

    #[test]
    fn retry_schedule_invokes_policy_and_records_delay_without_timer_runtime() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();
        let mut retry = input(ExecutionDomainCommandKind::ScheduleRetry);
        retry.domain_request.step = Some(step_with_status(StepExecutionStatus::Failed, 1));
        retry.domain_request.retry_attempt = Some(retry_attempt(2));

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            retry,
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Retrying));
        assert_eq!(receipt.retry_delay_seconds, Some(30));
        assert_eq!(retry_policy.calls.get(), 1);
        assert!(timers.armed.is_empty());
        assert!(dispatcher.dispatched.is_empty());
    }

    #[test]
    fn sla_timer_command_arms_timer_through_port_only() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy::default();
        let mut timers = FakeTimers::default();
        let mut timer_input = input(ExecutionDomainCommandKind::ArmSlaTimer);
        timer_input.domain_request.step = None;
        timer_input.domain_request.sla_timer = Some(timer());

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            timer_input,
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::Applied);
        assert_eq!(timers.armed.len(), 1);
        assert!(dispatcher.dispatched.is_empty());
    }

    #[test]
    fn retry_policy_error_is_sanitized_without_caching_failure() {
        let mut usecase = ExecutionEngineUsecase::default();
        let mut store = FakeStore::default();
        let mut dispatcher = FakeDispatcher::default();
        let retry_policy = FakeRetryPolicy {
            delay: Err(ExecutionEngineKernelError::InvalidAttempt),
            ..FakeRetryPolicy::default()
        };
        let mut timers = FakeTimers::default();
        let mut retry = input(ExecutionDomainCommandKind::ScheduleRetry);
        retry.domain_request.step = Some(step_with_status(StepExecutionStatus::Failed, 1));
        retry.domain_request.retry_attempt = Some(retry_attempt(2));

        let receipt = apply(
            &mut usecase,
            &mut store,
            &mut dispatcher,
            &retry_policy,
            &mut timers,
            retry,
        );

        assert_eq!(receipt.status, ExecutionUsecaseStatus::RetryPolicyRejected);
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(
            receipt
                .evidence_refs
                .contains(&"kernel:invalid-attempt".to_owned())
        );
    }
}
