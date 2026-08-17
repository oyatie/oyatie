//! Workflow-engine execution-engine usecase foundation.
//!
//! The usecase composes request/idempotency/trace validation, abstract run-store,
//! step-dispatcher, retry-policy, and SLA-timer ports, plus the policy-bound
//! execution-engine domain. It is source-level only: no concrete storage,
//! network, wall-clock, random, queue, signing, Valkey, Postgres, or cloud
//! runtime work is performed here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_execution_engine_domain::{
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
        ExecutionEngineKernelError::InvalidObservationWindow => "kernel:invalid-observation-window",
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

// ── Signal/await durable-wait slice ─────────────────────────────────────────
//
// Three command paths — await_signal, deliver_signal, timeout_signal — each on
// SignalAwaitUsecase, composing over SignalAwaitStore / SignalDeliverStore /
// SignalTimeoutStore ports and the existing SlaTimerStore port.
// Source-level only: no DB, clock, network, filesystem, queue, or randomness.

/// A record stored by the SignalAwaitStore port to track the suspend/resume state
/// for a `(tenant_id, run_id, signal_name)` correlation key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAwaitRecord {
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub run_id: String,      // data_class: INTERNAL_ONLY
    pub step_id: String,     // data_class: INTERNAL_ONLY
    pub signal_name: String, // data_class: INTERNAL_ONLY
    pub delivered: bool,     // data_class: INTERNAL_ONLY
}

/// Port: suspend a step awaiting a named signal and load the await record.
pub trait SignalAwaitStore {
    fn suspend_step_awaiting_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_id: &str,
        signal_name: &str,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError>;

    fn load_await_record(
        &self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
    ) -> Result<Option<SignalAwaitRecord>, ExecutionStoreError>;
}

/// Port: resume a previously-suspended step on signal delivery.
pub trait SignalDeliverStore: SignalAwaitStore {
    fn resume_step_on_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError>;
}

/// Port: mark a suspended step as timed out when the armed timer fires.
pub trait SignalTimeoutStore: SignalDeliverStore {
    fn timeout_step_awaiting_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError>;
}

// ── Audit event types ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SignalAuditEventKind {
    AwaitRequested,
    AwaitSuspended,
    AwaitInvalid,
    AwaitIdempotencyConflict,
    SignalDelivered,
    SignalUnmatched,
    SignalDeliverInvalid,
    SignalDeliverIdempotencyConflict,
    SignalTimedOut,
    SignalAlreadyDelivered,
    SignalTimeoutInvalid,
}

impl SignalAuditEventKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::AwaitRequested => "await-requested",
            Self::AwaitSuspended => "await-suspended",
            Self::AwaitInvalid => "await-invalid",
            Self::AwaitIdempotencyConflict => "await-idempotency-conflict",
            Self::SignalDelivered => "signal-delivered",
            Self::SignalUnmatched => "signal-unmatched",
            Self::SignalDeliverInvalid => "signal-deliver-invalid",
            Self::SignalDeliverIdempotencyConflict => "signal-deliver-idempotency-conflict",
            Self::SignalTimedOut => "signal-timed-out",
            Self::SignalAlreadyDelivered => "signal-already-delivered",
            Self::SignalTimeoutInvalid => "signal-timeout-invalid",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAuditEvent {
    pub kind: SignalAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub run_id: String,             // data_class: INTERNAL_ONLY
    pub signal_name: String,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

// ── WF-ENG-1: AwaitSignal ────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAwaitInput {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub trace_ref: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub step_id: String,                 // data_class: INTERNAL_ONLY
    pub signal_name: String,             // data_class: INTERNAL_ONLY
    pub timeout_timer: Option<SlaTimer>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SignalAwaitStatus {
    Awaiting,
    IdempotencyConflict,
    InvalidInput,
    StoreUnavailable,
    TimerUnavailable,
}

impl SignalAwaitStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Awaiting => "awaiting",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::StoreUnavailable => "store-unavailable",
            Self::TimerUnavailable => "timer-unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAwaitReceipt {
    pub status: SignalAwaitStatus,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub step_id: String,                     // data_class: INTERNAL_ONLY
    pub signal_name: String,                 // data_class: INTERNAL_ONLY
    pub timer_armed: bool,                   // data_class: INTERNAL_ONLY
    pub audit_events: Vec<SignalAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

// ── WF-ENG-2: SignalDeliver ──────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalDeliverInput {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub trace_ref: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub run_id: String,          // data_class: INTERNAL_ONLY
    pub signal_name: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SignalDeliverStatus {
    Delivered,
    Unmatched,
    IdempotencyConflict,
    InvalidInput,
    StoreUnavailable,
}

impl SignalDeliverStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Unmatched => "unmatched",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::StoreUnavailable => "store-unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalDeliverReceipt {
    pub status: SignalDeliverStatus,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub signal_name: String,                 // data_class: INTERNAL_ONLY
    pub audit_events: Vec<SignalAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

// ── WF-ENG-3: SignalTimeout ──────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalTimeoutInput {
    pub request_id: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,      // data_class: INTERNAL_ONLY
    pub trace_ref: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub run_id: String,               // data_class: INTERNAL_ONLY
    pub signal_name: String,          // data_class: INTERNAL_ONLY
    pub reference_epoch_seconds: u64, // caller-supplied, no wall-clock
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SignalTimeoutStatus {
    TimedOut,
    AlreadyDelivered,
    InvalidInput,
    StoreUnavailable,
}

impl SignalTimeoutStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::TimedOut => "timed-out",
            Self::AlreadyDelivered => "already-delivered",
            Self::InvalidInput => "invalid-input",
            Self::StoreUnavailable => "store-unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalTimeoutReceipt {
    pub status: SignalTimeoutStatus,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub signal_name: String,                 // data_class: INTERNAL_ONLY
    pub audit_events: Vec<SignalAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

// ── Intent fingerprints (private) ───────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
struct SignalAwaitIntent {
    fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SignalDeliverIntent {
    fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SignalTimeoutIntent {
    fingerprint: String,
}

// ── SignalAwaitUsecase ───────────────────────────────────────────────────────

#[derive(Default, Debug)]
pub struct SignalAwaitUsecase {
    await_receipts: BTreeMap<String, (SignalAwaitIntent, SignalAwaitReceipt)>,
    deliver_receipts: BTreeMap<String, (SignalDeliverIntent, SignalDeliverReceipt)>,
    timeout_receipts: BTreeMap<String, (SignalTimeoutIntent, SignalTimeoutReceipt)>,
}

impl SignalAwaitUsecase {
    // ── WF-ENG-1 ──────────────────────────────────────────────────────────

    pub fn await_signal<S, T>(
        &mut self,
        store: &mut S,
        timers: &mut T,
        input: SignalAwaitInput,
    ) -> SignalAwaitReceipt
    where
        S: SignalAwaitStore,
        T: SlaTimerStore,
    {
        // 1. Input validation
        if let Some(r) = signal_await_invalid_input(&input) {
            return r;
        }

        // 2. Idempotency
        let intent = SignalAwaitIntent {
            fingerprint: await_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.await_receipts.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return signal_await_conflict_receipt(&input);
        }

        // 3. Suspend via store
        let evidence_ref = "workflow-signal-usecase:suspend-step";
        if let Err(error) = store.suspend_step_awaiting_signal(
            &input.tenant_id,
            &input.run_id,
            &input.step_id,
            &input.signal_name,
            evidence_ref,
        ) {
            return signal_await_store_error_receipt(&input, error);
        }

        // 4. Optional timeout timer
        let timer_armed = if let Some(timer) = input.timeout_timer.clone() {
            match timers.arm_timer(timer) {
                Ok(()) => true,
                Err(error) => {
                    return signal_await_timer_error_receipt(&input, error);
                }
            }
        } else {
            false
        };

        // 5. Build and cache receipt
        let mut refs = sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
            "workflow-signal-usecase:awaiting".to_owned(),
        ]);
        if timer_armed {
            refs.push("workflow-signal-usecase:timer-armed".to_owned());
            refs = sorted_unique(refs);
        }
        let r = SignalAwaitReceipt {
            status: SignalAwaitStatus::Awaiting,
            tenant_id: input.tenant_id.clone(),
            run_id: input.run_id.clone(),
            step_id: input.step_id.clone(),
            signal_name: input.signal_name.clone(),
            timer_armed,
            audit_events: vec![
                signal_audit(
                    SignalAuditEventKind::AwaitRequested,
                    &input.tenant_id,
                    &input.run_id,
                    &input.signal_name,
                    sorted_unique(vec![
                        input.request_id.clone(),
                        input.idempotency_key.clone(),
                        input.trace_ref.clone(),
                    ]),
                ),
                signal_audit(
                    SignalAuditEventKind::AwaitSuspended,
                    &input.tenant_id,
                    &input.run_id,
                    &input.signal_name,
                    refs.clone(),
                ),
            ],
            evidence_refs: refs,
        };
        self.await_receipts
            .insert(input.idempotency_key.clone(), (intent, r.clone()));
        r
    }

    // ── WF-ENG-2 ──────────────────────────────────────────────────────────

    pub fn deliver_signal<S>(
        &mut self,
        store: &mut S,
        input: SignalDeliverInput,
    ) -> SignalDeliverReceipt
    where
        S: SignalDeliverStore,
    {
        // 1. Input validation
        if let Some(r) = signal_deliver_invalid_input(&input) {
            return r;
        }

        // 2. Idempotency
        let intent = SignalDeliverIntent {
            fingerprint: deliver_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.deliver_receipts.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return signal_deliver_conflict_receipt(&input);
        }

        // 3. Load await record
        let record =
            match store.load_await_record(&input.tenant_id, &input.run_id, &input.signal_name) {
                Ok(r) => r,
                Err(error) => {
                    return signal_deliver_store_error_receipt(&input, error);
                }
            };

        // 4. No active await → Unmatched (typed receipt, no store write)
        if record.as_ref().is_none_or(|r| r.delivered) {
            let refs = sorted_unique(vec![
                input.request_id.clone(),
                input.trace_ref.clone(),
                "workflow-signal-usecase:unmatched".to_owned(),
            ]);
            let r = SignalDeliverReceipt {
                status: SignalDeliverStatus::Unmatched,
                tenant_id: input.tenant_id.clone(),
                run_id: input.run_id.clone(),
                signal_name: input.signal_name.clone(),
                audit_events: vec![signal_audit(
                    SignalAuditEventKind::SignalUnmatched,
                    &input.tenant_id,
                    &input.run_id,
                    &input.signal_name,
                    refs.clone(),
                )],
                evidence_refs: refs,
            };
            self.deliver_receipts
                .insert(input.idempotency_key.clone(), (intent, r.clone()));
            return r;
        }

        // 5. Resume via store
        let evidence_ref = "workflow-signal-usecase:resume-step";
        if let Err(error) = store.resume_step_on_signal(
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            evidence_ref,
        ) {
            return signal_deliver_store_error_receipt(&input, error);
        }

        // 6. Build and cache receipt
        let refs = sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
            "workflow-signal-usecase:delivered".to_owned(),
        ]);
        let r = SignalDeliverReceipt {
            status: SignalDeliverStatus::Delivered,
            tenant_id: input.tenant_id.clone(),
            run_id: input.run_id.clone(),
            signal_name: input.signal_name.clone(),
            audit_events: vec![signal_audit(
                SignalAuditEventKind::SignalDelivered,
                &input.tenant_id,
                &input.run_id,
                &input.signal_name,
                refs.clone(),
            )],
            evidence_refs: refs,
        };
        self.deliver_receipts
            .insert(input.idempotency_key.clone(), (intent, r.clone()));
        r
    }

    // ── WF-ENG-3 ──────────────────────────────────────────────────────────

    pub fn timeout_signal<S>(
        &mut self,
        store: &mut S,
        input: SignalTimeoutInput,
    ) -> SignalTimeoutReceipt
    where
        S: SignalTimeoutStore,
    {
        // 1. Input validation
        if let Some(r) = signal_timeout_invalid_input(&input) {
            return r;
        }

        // 2. Idempotency
        let intent = SignalTimeoutIntent {
            fingerprint: timeout_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.timeout_receipts.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return signal_timeout_conflict_receipt(&input);
        }

        // 3. Load await record
        let record =
            match store.load_await_record(&input.tenant_id, &input.run_id, &input.signal_name) {
                Ok(r) => r,
                Err(error) => {
                    return signal_timeout_store_error_receipt(&input, error);
                }
            };

        // 4. Already delivered → AlreadyDelivered (deterministic no-op, cached)
        if record.as_ref().is_some_and(|r| r.delivered) || record.is_none() {
            let refs = sorted_unique(vec![
                input.request_id.clone(),
                input.trace_ref.clone(),
                format!(
                    "workflow-signal-usecase:reference-epoch:{}",
                    input.reference_epoch_seconds
                ),
                "workflow-signal-usecase:already-delivered".to_owned(),
            ]);
            let r = SignalTimeoutReceipt {
                status: SignalTimeoutStatus::AlreadyDelivered,
                tenant_id: input.tenant_id.clone(),
                run_id: input.run_id.clone(),
                signal_name: input.signal_name.clone(),
                audit_events: vec![signal_audit(
                    SignalAuditEventKind::SignalAlreadyDelivered,
                    &input.tenant_id,
                    &input.run_id,
                    &input.signal_name,
                    refs.clone(),
                )],
                evidence_refs: refs,
            };
            self.timeout_receipts
                .insert(input.idempotency_key.clone(), (intent, r.clone()));
            return r;
        }

        // 5. Timeout via store
        let evidence_ref = "workflow-signal-usecase:timeout-step";
        if let Err(error) = store.timeout_step_awaiting_signal(
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            evidence_ref,
        ) {
            return signal_timeout_store_error_receipt(&input, error);
        }

        // 6. Build and cache receipt with reference epoch in evidence refs
        let refs = sorted_unique(vec![
            input.request_id.clone(),
            input.idempotency_key.clone(),
            input.trace_ref.clone(),
            format!(
                "workflow-signal-usecase:reference-epoch:{}",
                input.reference_epoch_seconds
            ),
            "workflow-signal-usecase:timed-out".to_owned(),
        ]);
        let r = SignalTimeoutReceipt {
            status: SignalTimeoutStatus::TimedOut,
            tenant_id: input.tenant_id.clone(),
            run_id: input.run_id.clone(),
            signal_name: input.signal_name.clone(),
            audit_events: vec![signal_audit(
                SignalAuditEventKind::SignalTimedOut,
                &input.tenant_id,
                &input.run_id,
                &input.signal_name,
                refs.clone(),
            )],
            evidence_refs: refs,
        };
        self.timeout_receipts
            .insert(input.idempotency_key.clone(), (intent, r.clone()));
        r
    }

    pub fn cached_await_count(&self) -> usize {
        self.await_receipts.len()
    }

    pub fn cached_deliver_count(&self) -> usize {
        self.deliver_receipts.len()
    }

    pub fn cached_timeout_count(&self) -> usize {
        self.timeout_receipts.len()
    }
}

// ── Private signal helpers ───────────────────────────────────────────────────

fn signal_audit(
    kind: SignalAuditEventKind,
    tenant_id: &str,
    run_id: &str,
    signal_name: &str,
    evidence_refs: Vec<String>,
) -> SignalAuditEvent {
    SignalAuditEvent {
        kind,
        tenant_id: tenant_id.to_owned(),
        run_id: run_id.to_owned(),
        signal_name: signal_name.to_owned(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn signal_await_invalid_input(input: &SignalAwaitInput) -> Option<SignalAwaitReceipt> {
    let mut refs = Vec::new();
    if !is_safe_ref(&input.request_id) {
        refs.push("validation:signal-request-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.idempotency_key) {
        refs.push("validation:signal-idempotency-key-invalid".to_owned());
    }
    if !is_safe_ref(&input.trace_ref) {
        refs.push("validation:signal-trace-ref-invalid".to_owned());
    }
    if !is_safe_tenant(&input.tenant_id) {
        refs.push("validation:signal-tenant-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.run_id) {
        refs.push("validation:signal-run-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.step_id) {
        refs.push("validation:signal-step-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.signal_name) {
        refs.push("validation:signal-name-invalid".to_owned());
    }
    if input
        .timeout_timer
        .as_ref()
        .is_some_and(|timer| !is_safe_timer(timer))
    {
        refs.push("validation:signal-timeout-timer-invalid".to_owned());
    }
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-signal-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    let tenant = safe_tenant(&input.tenant_id);
    let run = safe_ref(&input.run_id, "redacted-invalid-run-id");
    let sig = safe_ref(&input.signal_name, "redacted-invalid-signal-name");
    Some(SignalAwaitReceipt {
        status: SignalAwaitStatus::InvalidInput,
        tenant_id: tenant.clone(),
        run_id: run.clone(),
        step_id: safe_ref(&input.step_id, "redacted-invalid-step-id"),
        signal_name: sig.clone(),
        timer_armed: false,
        audit_events: vec![signal_audit(
            SignalAuditEventKind::AwaitInvalid,
            &tenant,
            &run,
            &sig,
            refs.clone(),
        )],
        evidence_refs: refs,
    })
}

fn signal_await_conflict_receipt(input: &SignalAwaitInput) -> SignalAwaitReceipt {
    let refs = sorted_unique(vec![
        input.request_id.clone(),
        input.trace_ref.clone(),
        "workflow-signal-usecase:idempotency-conflict".to_owned(),
    ]);
    SignalAwaitReceipt {
        status: SignalAwaitStatus::IdempotencyConflict,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        step_id: input.step_id.clone(),
        signal_name: input.signal_name.clone(),
        timer_armed: false,
        audit_events: vec![signal_audit(
            SignalAuditEventKind::AwaitIdempotencyConflict,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_await_store_error_receipt(
    input: &SignalAwaitInput,
    error: ExecutionStoreError,
) -> SignalAwaitReceipt {
    let error_ref = store_error_ref(&error);
    let refs = sorted_unique(vec![
        error_ref,
        "workflow-signal-usecase:store-unavailable".to_owned(),
    ]);
    SignalAwaitReceipt {
        status: SignalAwaitStatus::StoreUnavailable,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        step_id: input.step_id.clone(),
        signal_name: input.signal_name.clone(),
        timer_armed: false,
        audit_events: vec![signal_audit(
            SignalAuditEventKind::AwaitInvalid,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_await_timer_error_receipt(
    input: &SignalAwaitInput,
    error: ExecutionStoreError,
) -> SignalAwaitReceipt {
    let error_ref = store_error_ref(&error);
    let refs = sorted_unique(vec![
        error_ref,
        "workflow-signal-usecase:timer-unavailable".to_owned(),
    ]);
    SignalAwaitReceipt {
        status: SignalAwaitStatus::TimerUnavailable,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        step_id: input.step_id.clone(),
        signal_name: input.signal_name.clone(),
        timer_armed: false,
        audit_events: vec![signal_audit(
            SignalAuditEventKind::AwaitInvalid,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_deliver_invalid_input(input: &SignalDeliverInput) -> Option<SignalDeliverReceipt> {
    let mut refs = Vec::new();
    if !is_safe_ref(&input.request_id) {
        refs.push("validation:signal-request-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.idempotency_key) {
        refs.push("validation:signal-idempotency-key-invalid".to_owned());
    }
    if !is_safe_ref(&input.trace_ref) {
        refs.push("validation:signal-trace-ref-invalid".to_owned());
    }
    if !is_safe_tenant(&input.tenant_id) {
        refs.push("validation:signal-tenant-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.run_id) {
        refs.push("validation:signal-run-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.signal_name) {
        refs.push("validation:signal-name-invalid".to_owned());
    }
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-signal-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    let tenant = safe_tenant(&input.tenant_id);
    let run = safe_ref(&input.run_id, "redacted-invalid-run-id");
    let sig = safe_ref(&input.signal_name, "redacted-invalid-signal-name");
    Some(SignalDeliverReceipt {
        status: SignalDeliverStatus::InvalidInput,
        tenant_id: tenant.clone(),
        run_id: run.clone(),
        signal_name: sig.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalDeliverInvalid,
            &tenant,
            &run,
            &sig,
            refs.clone(),
        )],
        evidence_refs: refs,
    })
}

fn signal_deliver_conflict_receipt(input: &SignalDeliverInput) -> SignalDeliverReceipt {
    let refs = sorted_unique(vec![
        input.request_id.clone(),
        input.trace_ref.clone(),
        "workflow-signal-usecase:idempotency-conflict".to_owned(),
    ]);
    SignalDeliverReceipt {
        status: SignalDeliverStatus::IdempotencyConflict,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        signal_name: input.signal_name.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalDeliverIdempotencyConflict,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_deliver_store_error_receipt(
    input: &SignalDeliverInput,
    error: ExecutionStoreError,
) -> SignalDeliverReceipt {
    let error_ref = store_error_ref(&error);
    let refs = sorted_unique(vec![
        error_ref,
        "workflow-signal-usecase:store-unavailable".to_owned(),
    ]);
    SignalDeliverReceipt {
        status: SignalDeliverStatus::StoreUnavailable,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        signal_name: input.signal_name.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalDeliverInvalid,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_timeout_invalid_input(input: &SignalTimeoutInput) -> Option<SignalTimeoutReceipt> {
    let mut refs = Vec::new();
    if !is_safe_ref(&input.request_id) {
        refs.push("validation:signal-request-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.idempotency_key) {
        refs.push("validation:signal-idempotency-key-invalid".to_owned());
    }
    if !is_safe_ref(&input.trace_ref) {
        refs.push("validation:signal-trace-ref-invalid".to_owned());
    }
    if !is_safe_tenant(&input.tenant_id) {
        refs.push("validation:signal-tenant-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.run_id) {
        refs.push("validation:signal-run-id-invalid".to_owned());
    }
    if !is_safe_ref(&input.signal_name) {
        refs.push("validation:signal-name-invalid".to_owned());
    }
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-signal-usecase:invalid-input".to_owned());
    let refs = sorted_unique(refs);
    let tenant = safe_tenant(&input.tenant_id);
    let run = safe_ref(&input.run_id, "redacted-invalid-run-id");
    let sig = safe_ref(&input.signal_name, "redacted-invalid-signal-name");
    Some(SignalTimeoutReceipt {
        status: SignalTimeoutStatus::InvalidInput,
        tenant_id: tenant.clone(),
        run_id: run.clone(),
        signal_name: sig.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalTimeoutInvalid,
            &tenant,
            &run,
            &sig,
            refs.clone(),
        )],
        evidence_refs: refs,
    })
}

fn signal_timeout_conflict_receipt(input: &SignalTimeoutInput) -> SignalTimeoutReceipt {
    let refs = sorted_unique(vec![
        input.request_id.clone(),
        input.trace_ref.clone(),
        "workflow-signal-usecase:idempotency-conflict".to_owned(),
    ]);
    SignalTimeoutReceipt {
        status: SignalTimeoutStatus::InvalidInput, // reuse InvalidInput for conflict path per spec; no separate variant
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        signal_name: input.signal_name.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalTimeoutInvalid,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

fn signal_timeout_store_error_receipt(
    input: &SignalTimeoutInput,
    error: ExecutionStoreError,
) -> SignalTimeoutReceipt {
    let error_ref = store_error_ref(&error);
    let refs = sorted_unique(vec![
        error_ref,
        "workflow-signal-usecase:store-unavailable".to_owned(),
    ]);
    SignalTimeoutReceipt {
        status: SignalTimeoutStatus::StoreUnavailable,
        tenant_id: input.tenant_id.clone(),
        run_id: input.run_id.clone(),
        signal_name: input.signal_name.clone(),
        audit_events: vec![signal_audit(
            SignalAuditEventKind::SignalTimeoutInvalid,
            &input.tenant_id,
            &input.run_id,
            &input.signal_name,
            refs.clone(),
        )],
        evidence_refs: refs,
    }
}

// ── Shared store-error helper (private) ──────────────────────────────────────

/// Extract the evidence_ref string from any ExecutionStoreError variant.
/// Used by all three signal *_store_error_receipt helpers to avoid duplicate match arms.
fn store_error_ref(error: &ExecutionStoreError) -> String {
    match error {
        ExecutionStoreError::Unavailable { evidence_ref } => evidence_ref.clone(),
        ExecutionStoreError::Conflict { evidence_ref, .. } => evidence_ref.clone(),
    }
}

// ── Fingerprint helpers (private) ────────────────────────────────────────────

fn await_fingerprint(input: &SignalAwaitInput) -> String {
    let mut parts = vec![
        canonical_entry("request_id", &input.request_id),
        canonical_entry("trace_ref", &input.trace_ref),
        canonical_entry("tenant_id", &input.tenant_id),
        canonical_entry("run_id", &input.run_id),
        canonical_entry("step_id", &input.step_id),
        canonical_entry("signal_name", &input.signal_name),
    ];
    if let Some(timer) = &input.timeout_timer {
        parts.extend([
            canonical_entry("timer_id", &timer.timer_id),
            canonical_entry("timer_deadline", &timer.deadline_epoch_seconds.to_string()),
        ]);
    }
    parts.concat()
}

fn deliver_fingerprint(input: &SignalDeliverInput) -> String {
    [
        canonical_entry("request_id", &input.request_id),
        canonical_entry("trace_ref", &input.trace_ref),
        canonical_entry("tenant_id", &input.tenant_id),
        canonical_entry("run_id", &input.run_id),
        canonical_entry("signal_name", &input.signal_name),
    ]
    .concat()
}

fn timeout_fingerprint(input: &SignalTimeoutInput) -> String {
    [
        canonical_entry("request_id", &input.request_id),
        canonical_entry("trace_ref", &input.trace_ref),
        canonical_entry("tenant_id", &input.tenant_id),
        canonical_entry("run_id", &input.run_id),
        canonical_entry("signal_name", &input.signal_name),
        canonical_entry(
            "reference_epoch",
            &input.reference_epoch_seconds.to_string(),
        ),
    ]
    .concat()
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

    // ── Signal/await tests ────────────────────────────────────────────────

    /// Fake signal store used by all signal tests.
    #[derive(Default)]
    struct FakeSignalStore {
        suspended: Vec<(String, String, String, String)>, // (tenant, run, step, signal)
        records: std::collections::BTreeMap<String, SignalAwaitRecord>, // key = "tenant:run:signal"
        resumed: Vec<(String, String, String)>,           // (tenant, run, signal)
        timed_out: Vec<(String, String, String)>,         // (tenant, run, signal)
        load_failure: bool,
        suspend_failure: bool,
        resume_failure: bool,
        timeout_failure: bool,
    }

    impl FakeSignalStore {
        fn record_key(tenant_id: &str, run_id: &str, signal_name: &str) -> String {
            format!("{tenant_id}:{run_id}:{signal_name}")
        }
    }

    impl SignalAwaitStore for FakeSignalStore {
        fn suspend_step_awaiting_signal(
            &mut self,
            tenant_id: &str,
            run_id: &str,
            step_id: &str,
            signal_name: &str,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionStoreError> {
            if self.suspend_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:suspend-unavailable".to_owned(),
                });
            }
            self.suspended.push((
                tenant_id.to_owned(),
                run_id.to_owned(),
                step_id.to_owned(),
                signal_name.to_owned(),
            ));
            self.records.insert(
                Self::record_key(tenant_id, run_id, signal_name),
                SignalAwaitRecord {
                    tenant_id: tenant_id.to_owned(),
                    run_id: run_id.to_owned(),
                    step_id: step_id.to_owned(),
                    signal_name: signal_name.to_owned(),
                    delivered: false,
                },
            );
            Ok(())
        }

        fn load_await_record(
            &self,
            tenant_id: &str,
            run_id: &str,
            signal_name: &str,
        ) -> Result<Option<SignalAwaitRecord>, ExecutionStoreError> {
            if self.load_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:load-unavailable".to_owned(),
                });
            }
            Ok(self
                .records
                .get(&Self::record_key(tenant_id, run_id, signal_name))
                .cloned())
        }
    }

    impl SignalDeliverStore for FakeSignalStore {
        fn resume_step_on_signal(
            &mut self,
            tenant_id: &str,
            run_id: &str,
            signal_name: &str,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionStoreError> {
            if self.resume_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:resume-unavailable".to_owned(),
                });
            }
            self.resumed.push((
                tenant_id.to_owned(),
                run_id.to_owned(),
                signal_name.to_owned(),
            ));
            if let Some(record) =
                self.records
                    .get_mut(&Self::record_key(tenant_id, run_id, signal_name))
            {
                record.delivered = true;
            }
            Ok(())
        }
    }

    impl SignalTimeoutStore for FakeSignalStore {
        fn timeout_step_awaiting_signal(
            &mut self,
            tenant_id: &str,
            run_id: &str,
            signal_name: &str,
            _evidence_ref: &str,
        ) -> Result<(), ExecutionStoreError> {
            if self.timeout_failure {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "store:timeout-unavailable".to_owned(),
                });
            }
            self.timed_out.push((
                tenant_id.to_owned(),
                run_id.to_owned(),
                signal_name.to_owned(),
            ));
            Ok(())
        }
    }

    fn signal_await_input() -> SignalAwaitInput {
        SignalAwaitInput {
            request_id: "req:signal-await:1".to_owned(),
            idempotency_key: "idem:signal-await:1".to_owned(),
            trace_ref: "trace:signal-await:1".to_owned(),
            tenant_id: "ten_a".to_owned(),
            run_id: "run:signal-await:1".to_owned(),
            step_id: "step:signal-await:1".to_owned(),
            signal_name: "signal:human-approval:1".to_owned(),
            timeout_timer: None,
        }
    }

    fn signal_deliver_input() -> SignalDeliverInput {
        SignalDeliverInput {
            request_id: "req:signal-deliver:1".to_owned(),
            idempotency_key: "idem:signal-deliver:1".to_owned(),
            trace_ref: "trace:signal-deliver:1".to_owned(),
            tenant_id: "ten_a".to_owned(),
            run_id: "run:signal-await:1".to_owned(),
            signal_name: "signal:human-approval:1".to_owned(),
        }
    }

    fn signal_timeout_input() -> SignalTimeoutInput {
        SignalTimeoutInput {
            request_id: "req:signal-timeout:1".to_owned(),
            idempotency_key: "idem:signal-timeout:1".to_owned(),
            trace_ref: "trace:signal-timeout:1".to_owned(),
            tenant_id: "ten_a".to_owned(),
            run_id: "run:signal-await:1".to_owned(),
            signal_name: "signal:human-approval:1".to_owned(),
            reference_epoch_seconds: 500,
        }
    }

    // WF-ENG-1: fresh await suspends and returns Awaiting
    #[test]
    fn await_signal_fresh_suspends_and_returns_awaiting() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        let r = uc.await_signal(&mut store, &mut timers, signal_await_input());

        assert_eq!(r.status, SignalAwaitStatus::Awaiting);
        assert!(!r.timer_armed);
        assert_eq!(store.suspended.len(), 1);
        assert_eq!(store.suspended[0].3, "signal:human-approval:1");
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:awaiting".to_owned())
        );
        assert_eq!(uc.cached_await_count(), 1);
    }

    // WF-ENG-1: duplicate idempotency key replays identical receipt
    #[test]
    fn await_signal_duplicate_key_replays_identical_receipt() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        let first = uc.await_signal(&mut store, &mut timers, signal_await_input());
        let replay = uc.await_signal(&mut store, &mut timers, signal_await_input());

        assert_eq!(first, replay);
        assert_eq!(store.suspended.len(), 1, "suspend called only once");
    }

    // WF-ENG-1: mismatched fingerprint on same key yields IdempotencyConflict
    #[test]
    fn await_signal_mismatched_key_yields_idempotency_conflict() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        uc.await_signal(&mut store, &mut timers, signal_await_input());

        let mut conflict = signal_await_input();
        conflict.trace_ref = "trace:signal-await:other".to_owned();
        let r = uc.await_signal(&mut store, &mut timers, conflict);

        assert_eq!(r.status, SignalAwaitStatus::IdempotencyConflict);
        assert_eq!(store.suspended.len(), 1, "no second suspend on conflict");
    }

    // WF-ENG-1: invalid input (empty/bad signal_name) returns InvalidInput without store call
    #[test]
    fn await_signal_invalid_input_returns_invalid_no_store_call() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        let mut bad = signal_await_input();
        bad.signal_name = "not-a-valid-ref-no-colon".to_owned();
        let r = uc.await_signal(&mut store, &mut timers, bad);

        assert_eq!(r.status, SignalAwaitStatus::InvalidInput);
        assert!(store.suspended.is_empty());
        assert_eq!(
            uc.cached_await_count(),
            0,
            "invalid input must not be cached"
        );
    }

    // WF-ENG-1: await with timeout timer arms the timer
    #[test]
    fn await_signal_with_timeout_timer_arms_timer() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        let mut input = signal_await_input();
        input.timeout_timer = Some(
            SlaTimer::new(
                "timer:signal-await:1",
                "ten_a",
                "run:signal-await:1",
                None,
                100,
                200,
                vec!["workflow-signal-usecase:timer".to_owned()],
            )
            .unwrap(),
        );
        let r = uc.await_signal(&mut store, &mut timers, input);

        assert_eq!(r.status, SignalAwaitStatus::Awaiting);
        assert!(r.timer_armed);
        assert_eq!(timers.armed.len(), 1);
    }

    // WF-ENG-2: deliver after await resumes exactly once
    #[test]
    fn deliver_signal_after_await_resumes_exactly_once() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        uc.await_signal(&mut store, &mut timers, signal_await_input());
        let r = uc.deliver_signal(&mut store, signal_deliver_input());

        assert_eq!(r.status, SignalDeliverStatus::Delivered);
        assert_eq!(store.resumed.len(), 1);
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:delivered".to_owned())
        );
    }

    // WF-ENG-2: re-deliver same signal (same idempotency key) is idempotent
    #[test]
    fn deliver_signal_redelivery_is_idempotent() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        uc.await_signal(&mut store, &mut timers, signal_await_input());
        let first = uc.deliver_signal(&mut store, signal_deliver_input());
        let replay = uc.deliver_signal(&mut store, signal_deliver_input());

        assert_eq!(first, replay);
        assert_eq!(store.resumed.len(), 1, "resume called only once total");
    }

    // WF-ENG-2: signal with no prior await yields Unmatched (no panic, no store write)
    #[test]
    fn deliver_signal_no_prior_await_yields_unmatched() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();

        let r = uc.deliver_signal(&mut store, signal_deliver_input());

        assert_eq!(r.status, SignalDeliverStatus::Unmatched);
        assert!(store.resumed.is_empty());
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:unmatched".to_owned())
        );
    }

    // WF-ENG-3 table: timeout-before-delivery → TimedOut + audit event
    #[test]
    fn timeout_signal_before_delivery_yields_timed_out() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        // First suspend the step so a record exists (not yet delivered)
        uc.await_signal(&mut store, &mut timers, signal_await_input());

        let r = uc.timeout_signal(&mut store, signal_timeout_input());

        assert_eq!(r.status, SignalTimeoutStatus::TimedOut);
        assert_eq!(store.timed_out.len(), 1);
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:timed-out".to_owned())
        );
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:reference-epoch:500".to_owned())
        );
        assert!(
            r.audit_events
                .iter()
                .any(|e| e.kind == SignalAuditEventKind::SignalTimedOut)
        );
    }

    // WF-ENG-3 table: delivery-before-timeout → AlreadyDelivered
    #[test]
    fn timeout_signal_after_delivery_yields_already_delivered() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        uc.await_signal(&mut store, &mut timers, signal_await_input());
        uc.deliver_signal(&mut store, signal_deliver_input());

        let r = uc.timeout_signal(&mut store, signal_timeout_input());

        assert_eq!(r.status, SignalTimeoutStatus::AlreadyDelivered);
        assert!(
            store.timed_out.is_empty(),
            "timeout store must not be called after delivery"
        );
        assert!(
            r.evidence_refs
                .contains(&"workflow-signal-usecase:already-delivered".to_owned())
        );
    }

    // WF-ENG-3: crate performs zero IO — all three paths pass with in-memory fakes
    #[test]
    fn signal_slice_zero_io_source_level_only() {
        let mut uc = SignalAwaitUsecase::default();
        let mut store = FakeSignalStore::default();
        let mut timers = FakeTimers::default();

        // Full await → deliver → (second timeout attempt on a separate key)
        let await_r = uc.await_signal(&mut store, &mut timers, signal_await_input());
        assert_eq!(await_r.status, SignalAwaitStatus::Awaiting);

        let deliver_r = uc.deliver_signal(&mut store, signal_deliver_input());
        assert_eq!(deliver_r.status, SignalDeliverStatus::Delivered);

        // Timeout on an unrelated signal (no prior await → AlreadyDelivered path via None record)
        let mut t_input = signal_timeout_input();
        t_input.signal_name = "signal:other:1".to_owned();
        t_input.idempotency_key = "idem:signal-timeout:other".to_owned();
        let timeout_r = uc.timeout_signal(&mut store, t_input);
        assert_eq!(timeout_r.status, SignalTimeoutStatus::AlreadyDelivered);

        // No real IO was performed (store is an in-memory BTreeMap fake)
        assert_eq!(store.suspended.len(), 1);
        assert_eq!(store.resumed.len(), 1);
        assert_eq!(store.timed_out.len(), 0);
    }
}
