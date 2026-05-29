//! Workflow-engine execution-engine domain foundation.
//!
//! The domain layer binds source-level execution commands to policy, spec
//! integrity, replay, and scheduler evidence before later usecase/storage/worker
//! integration. It performs no database, filesystem, network, wall-clock,
//! random, queue, signing, Valkey, Postgres, or cloud-runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_execution_engine_kernel::{
    ExecutionDispatchError, ExecutionEngineKernelError, ExecutionStoreError, RetryAttempt,
    RetryPolicyEvaluator, SlaTimer, SlaTimerStore, StepDispatcher, StepExecution,
    StepExecutionStatus, WorkflowExecutionStatus, WorkflowRun, WorkflowRunStore,
};

/// Classification of an SLA timer's deadline relative to a caller-supplied reference instant.
/// Pure value object — no wall-clock, no I/O.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlaDeadlineClass {
    OnTrack,
    AtRisk,
    Breached,
}

impl SlaDeadlineClass {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::OnTrack => "on-track",
            Self::AtRisk => "at-risk",
            Self::Breached => "breached",
        }
    }
}

/// Classify an SLA timer's deadline against a caller-supplied reference epoch (seconds).
///
/// Thresholds (pure integer arithmetic):
/// - `reference >= deadline`                             → `Breached`
/// - `reference >= armed_at + (deadline - armed_at) * 80 / 100` → `AtRisk`
/// - otherwise                                           → `OnTrack`
///
/// No `std::time::SystemTime::now()`, no randomness, no I/O.
pub fn classify_sla_deadline(timer: &SlaTimer, reference_epoch_seconds: u64) -> SlaDeadlineClass {
    if reference_epoch_seconds >= timer.deadline_epoch_seconds {
        return SlaDeadlineClass::Breached;
    }
    let window = timer.deadline_epoch_seconds - timer.armed_at_epoch_seconds;
    let at_risk_at = timer.armed_at_epoch_seconds + window * 80 / 100;
    if reference_epoch_seconds >= at_risk_at {
        SlaDeadlineClass::AtRisk
    } else {
        SlaDeadlineClass::OnTrack
    }
}

/// Escalation bucket for an SLA timer relative to a caller-supplied reference instant.
/// Pure value object — no wall-clock, no I/O.
///
/// Ordering: `None < Notify < Page < AutoAbort`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SlaEscalationLevel {
    None,
    Notify,
    Page,
    AutoAbort,
}

impl SlaEscalationLevel {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Notify => "notify",
            Self::Page => "page",
            Self::AutoAbort => "auto-abort",
        }
    }
}

/// Project the SLA escalation bucket for a timer given a reference epoch (seconds) and a
/// breach-grace window (seconds after the deadline before auto-abort fires).
///
/// Thresholds (pure integer arithmetic, saturating):
/// - `reference >= deadline + breach_grace_seconds` → `AutoAbort`
/// - `reference >= deadline`                         → `Page`
/// - `reference >= armed_at + window*80/100`         → `Notify`
/// - otherwise                                       → `None`
///
/// No `std::time::SystemTime::now()`, no randomness, no I/O.
pub fn project_sla_escalation(
    timer: &SlaTimer,
    reference_epoch_seconds: u64,
    breach_grace_seconds: u64,
) -> SlaEscalationLevel {
    let armed_at = timer.armed_at_epoch_seconds;
    let deadline = timer.deadline_epoch_seconds;
    let window = deadline.saturating_sub(armed_at);
    let at_risk_at = armed_at.saturating_add(window.saturating_mul(80) / 100);
    let grace_end = deadline.saturating_add(breach_grace_seconds);

    if reference_epoch_seconds >= grace_end {
        SlaEscalationLevel::AutoAbort
    } else if reference_epoch_seconds >= deadline {
        SlaEscalationLevel::Page
    } else if reference_epoch_seconds >= at_risk_at {
        SlaEscalationLevel::Notify
    } else {
        SlaEscalationLevel::None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionDomainOrigin {
    ApiCommand,
    WorkerScheduler,
    ReplayRecovery,
    SlaTimer,
}

impl ExecutionDomainOrigin {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::ApiCommand => "api-command",
            Self::WorkerScheduler => "worker-scheduler",
            Self::ReplayRecovery => "replay-recovery",
            Self::SlaTimer => "sla-timer",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionDomainCommandKind {
    StartRun,
    DispatchStep,
    ScheduleRetry,
    ArmSlaTimer,
}

impl ExecutionDomainCommandKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::StartRun => "start-run",
            Self::DispatchStep => "dispatch-step",
            Self::ScheduleRetry => "schedule-retry",
            Self::ArmSlaTimer => "arm-sla-timer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineDomainRequest {
    pub run: WorkflowRun,                    // data_class: INTERNAL_ONLY
    pub step: Option<StepExecution>,         // data_class: INTERNAL_ONLY
    pub retry_attempt: Option<RetryAttempt>, // data_class: INTERNAL_ONLY
    pub sla_timer: Option<SlaTimer>,         // data_class: INTERNAL_ONLY
    pub expected_tenant_id: String,          // data_class: INTERNAL_ONLY
    pub expected_spec_id: String,            // data_class: INTERNAL_ONLY
    pub expected_version_sha: String,        // data_class: INTERNAL_ONLY
    pub expected_cell_id: String,            // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,          // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,            // data_class: INTERNAL_ONLY
    pub scheduler_epoch_ref: String,         // data_class: INTERNAL_ONLY
    pub sla_reference_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub command: ExecutionDomainCommandKind, // data_class: INTERNAL_ONLY
    pub origin: ExecutionDomainOrigin,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionDomainDenialKind {
    MissingEvidence,
    UnsafeMetadata,
    ScopeMismatch,
    InvalidCommandShape,
    InvalidState,
}

impl ExecutionDomainDenialKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::MissingEvidence => "missing-evidence",
            Self::UnsafeMetadata => "unsafe-metadata",
            Self::ScopeMismatch => "scope-mismatch",
            Self::InvalidCommandShape => "invalid-command-shape",
            Self::InvalidState => "invalid-state",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionDomainReceipt {
    pub command: ExecutionDomainCommandKind, // data_class: INTERNAL_ONLY
    pub origin: ExecutionDomainOrigin,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub next_run_status: WorkflowExecutionStatus, // data_class: PUBLIC
    pub step_status: Option<StepExecutionStatus>, // data_class: PUBLIC
    pub dispatch_required: bool,             // data_class: INTERNAL_ONLY
    pub retry_scheduled: bool,               // data_class: INTERNAL_ONLY
    pub sla_timer_armed: bool,               // data_class: INTERNAL_ONLY
    pub audit_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionDomainDenial {
    pub kind: ExecutionDomainDenialKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub audit_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionDomainDecision {
    Accepted(ExecutionDomainReceipt),
    Denied(ExecutionDomainDenial),
}

impl ExecutionDomainDecision {
    pub fn expect_accepted(self) -> ExecutionDomainReceipt {
        match self {
            Self::Accepted(receipt) => receipt,
            Self::Denied(denial) => {
                panic!("expected accepted execution-domain decision, got {denial:?}")
            }
        }
    }

    pub fn expect_denied(self) -> ExecutionDomainDenial {
        match self {
            Self::Accepted(receipt) => {
                panic!("expected denied execution-domain decision, got {receipt:?}")
            }
            Self::Denied(denial) => denial,
        }
    }
}

pub fn evaluate_execution_domain(request: ExecutionEngineDomainRequest) -> ExecutionDomainDecision {
    if let Some(denial) = preflight_denial(&request) {
        return ExecutionDomainDecision::Denied(denial);
    }

    ExecutionDomainDecision::Accepted(ExecutionDomainReceipt {
        command: request.command,
        origin: request.origin,
        tenant_id: request.run.tenant_id.clone(),
        run_id: request.run.run_id.clone(),
        next_run_status: next_run_status(&request),
        step_status: next_step_status(&request),
        dispatch_required: matches!(
            request.command,
            ExecutionDomainCommandKind::StartRun | ExecutionDomainCommandKind::DispatchStep
        ),
        retry_scheduled: request.command == ExecutionDomainCommandKind::ScheduleRetry,
        sla_timer_armed: request.command == ExecutionDomainCommandKind::ArmSlaTimer,
        audit_refs: domain_audit_refs(&request, "accepted"),
    })
}

fn preflight_denial(request: &ExecutionEngineDomainRequest) -> Option<ExecutionDomainDenial> {
    let missing = missing_evidence_refs(request);
    if !missing.is_empty() {
        return Some(domain_denial(
            request,
            ExecutionDomainDenialKind::MissingEvidence,
            missing,
        ));
    }

    if has_unsafe_metadata(request) {
        return Some(domain_denial(
            request,
            ExecutionDomainDenialKind::UnsafeMetadata,
            vec!["workflow-execution-domain:unsafe-metadata".to_owned()],
        ));
    }

    if has_scope_mismatch(request) {
        return Some(domain_denial(
            request,
            ExecutionDomainDenialKind::ScopeMismatch,
            vec!["workflow-execution-domain:scope-mismatch".to_owned()],
        ));
    }

    if has_invalid_command_shape(request) {
        return Some(domain_denial(
            request,
            ExecutionDomainDenialKind::InvalidCommandShape,
            vec!["workflow-execution-domain:invalid-command-shape".to_owned()],
        ));
    }

    if has_invalid_state(request) {
        return Some(domain_denial(
            request,
            ExecutionDomainDenialKind::InvalidState,
            vec!["workflow-execution-domain:invalid-state".to_owned()],
        ));
    }

    None
}

fn missing_evidence_refs(request: &ExecutionEngineDomainRequest) -> Vec<String> {
    let mut missing = Vec::new();
    if request.policy_evidence_ref.trim().is_empty() {
        missing.push("validation:policy-evidence-required".to_owned());
    }
    if request.spec_integrity_ref.trim().is_empty() {
        missing.push("validation:spec-integrity-required".to_owned());
    }
    if request.replay_epoch_ref.trim().is_empty() {
        missing.push("validation:replay-epoch-required".to_owned());
    }
    if request.scheduler_epoch_ref.trim().is_empty() {
        missing.push("validation:scheduler-epoch-required".to_owned());
    }
    missing
}

fn has_scope_mismatch(request: &ExecutionEngineDomainRequest) -> bool {
    request.run.tenant_id != request.expected_tenant_id
        || request.run.spec_id != request.expected_spec_id
        || request.run.version_sha != request.expected_version_sha
        || request.run.active_cell_id != request.expected_cell_id
        || request.step.as_ref().is_some_and(|step| {
            step.tenant_id != request.expected_tenant_id || step.run_id != request.run.run_id
        })
        || request.retry_attempt.as_ref().is_some_and(|retry| {
            retry.tenant_id != request.expected_tenant_id
                || retry.run_id != request.run.run_id
                || request
                    .step
                    .as_ref()
                    .is_some_and(|step| retry.step_id != step.step_id)
        })
        || request.sla_timer.as_ref().is_some_and(|timer| {
            timer.tenant_id != request.expected_tenant_id || timer.run_id != request.run.run_id
        })
}

fn has_invalid_command_shape(request: &ExecutionEngineDomainRequest) -> bool {
    match request.command {
        ExecutionDomainCommandKind::StartRun | ExecutionDomainCommandKind::DispatchStep => {
            request.step.is_none() || request.retry_attempt.is_some()
        }
        ExecutionDomainCommandKind::ScheduleRetry => {
            request.step.is_none() || request.retry_attempt.is_none()
        }
        ExecutionDomainCommandKind::ArmSlaTimer => request.sla_timer.is_none(),
    }
}

fn has_invalid_state(request: &ExecutionEngineDomainRequest) -> bool {
    if request.run.status.is_terminal() {
        return true;
    }

    match request.command {
        ExecutionDomainCommandKind::StartRun => {
            request.run.status != WorkflowExecutionStatus::Pending
                || request
                    .step
                    .as_ref()
                    .is_none_or(|step| step.status != StepExecutionStatus::Pending)
        }
        ExecutionDomainCommandKind::DispatchStep => {
            request.run.status != WorkflowExecutionStatus::Running
                || request.step.as_ref().is_none_or(|step| {
                    !matches!(
                        step.status,
                        StepExecutionStatus::Pending | StepExecutionStatus::Retrying
                    )
                })
        }
        ExecutionDomainCommandKind::ScheduleRetry => request.step.as_ref().is_none_or(|step| {
            !step.status.is_terminal_failure()
                || request
                    .retry_attempt
                    .as_ref()
                    .is_none_or(|retry| retry.attempt <= step.attempt)
        }),
        ExecutionDomainCommandKind::ArmSlaTimer => false,
    }
}

fn next_run_status(request: &ExecutionEngineDomainRequest) -> WorkflowExecutionStatus {
    match request.command {
        ExecutionDomainCommandKind::StartRun
        | ExecutionDomainCommandKind::DispatchStep
        | ExecutionDomainCommandKind::ScheduleRetry => WorkflowExecutionStatus::Running,
        ExecutionDomainCommandKind::ArmSlaTimer => request.run.status,
    }
}

fn next_step_status(request: &ExecutionEngineDomainRequest) -> Option<StepExecutionStatus> {
    match request.command {
        ExecutionDomainCommandKind::StartRun => Some(StepExecutionStatus::Pending),
        ExecutionDomainCommandKind::DispatchStep => Some(StepExecutionStatus::Leased),
        ExecutionDomainCommandKind::ScheduleRetry => Some(StepExecutionStatus::Retrying),
        ExecutionDomainCommandKind::ArmSlaTimer => request.step.as_ref().map(|step| step.status),
    }
}

fn domain_denial(
    request: &ExecutionEngineDomainRequest,
    kind: ExecutionDomainDenialKind,
    mut audit_refs: Vec<String>,
) -> ExecutionDomainDenial {
    audit_refs.push(format!("workflow-execution-domain:{}", kind.as_wire()));
    ExecutionDomainDenial {
        kind,
        tenant_id: request.run.tenant_id.clone(),
        run_id: request.run.run_id.clone(),
        audit_refs: sorted_unique(audit_refs),
    }
}

fn domain_audit_refs(request: &ExecutionEngineDomainRequest, outcome: &str) -> Vec<String> {
    let mut refs = domain_refs(request);
    refs.extend(request.run.evidence_refs.clone());
    if let Some(step) = &request.step {
        refs.extend(step.evidence_refs.clone());
    }
    if let Some(retry) = &request.retry_attempt {
        refs.extend(retry.evidence_refs.clone());
    }
    if let Some(timer) = &request.sla_timer {
        refs.extend(timer.evidence_refs.clone());
        if request.command == ExecutionDomainCommandKind::ArmSlaTimer {
            let class = classify_sla_deadline(timer, request.sla_reference_epoch_seconds);
            refs.push(format!(
                "workflow-execution-domain:sla-class:{}",
                class.as_wire()
            ));
        }
    }
    refs.push(format!("workflow-execution-domain:{outcome}"));
    refs.push(format!(
        "workflow-execution-domain:command:{}",
        request.command.as_wire()
    ));
    refs.push(format!(
        "workflow-execution-domain:origin:{}",
        request.origin.as_wire()
    ));
    sorted_unique(refs)
}

fn domain_refs(request: &ExecutionEngineDomainRequest) -> Vec<String> {
    vec![
        request.policy_evidence_ref.clone(),
        request.spec_integrity_ref.clone(),
        request.replay_epoch_ref.clone(),
        request.scheduler_epoch_ref.clone(),
    ]
}

fn has_unsafe_metadata(request: &ExecutionEngineDomainRequest) -> bool {
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

    fn run_with_status(status: WorkflowExecutionStatus) -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:execution-domain:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec![
                "workflow-execution-domain:requested".to_owned(),
                "workflow-execution-domain:requested".to_owned(),
            ],
        )
        .unwrap();
        run.status = status;
        run
    }

    fn step_with_status(status: StepExecutionStatus, attempt: u32) -> StepExecution {
        let mut step = StepExecution::new(
            "ten_a",
            "run:execution-domain:1",
            "step:approve",
            0,
            attempt,
            "idempotency:step:approve:1",
            vec!["workflow-execution-domain:step".to_owned()],
        )
        .unwrap();
        step.status = status;
        step
    }

    fn request(command: ExecutionDomainCommandKind) -> ExecutionEngineDomainRequest {
        ExecutionEngineDomainRequest {
            run: run_with_status(WorkflowExecutionStatus::Running),
            step: Some(step_with_status(StepExecutionStatus::Pending, 1)),
            retry_attempt: None,
            sla_timer: None,
            expected_tenant_id: "ten_a".to_owned(),
            expected_spec_id: "workflow-spec:invoice-approval".to_owned(),
            expected_version_sha: "sha256:spec-v1".to_owned(),
            expected_cell_id: "cell:use1:a".to_owned(),
            policy_evidence_ref: "cedar://workflow/execution/dispatch".to_owned(),
            spec_integrity_ref: "spec-integrity:workflow:v1".to_owned(),
            replay_epoch_ref: "replay-epoch:execution-domain:1".to_owned(),
            scheduler_epoch_ref: "scheduler-epoch:execution-domain:1".to_owned(),
            sla_reference_epoch_seconds: 0,
            command,
            origin: ExecutionDomainOrigin::WorkerScheduler,
        }
    }

    fn sla_timer(armed_at: u64, deadline: u64) -> SlaTimer {
        SlaTimer::new(
            "timer:execution-domain:1",
            "ten_a",
            "run:execution-domain:1",
            Some(0),
            armed_at,
            deadline,
            vec!["workflow-execution-domain:sla".to_owned()],
        )
        .unwrap()
    }

    fn arm_sla_timer_request(
        armed_at: u64,
        deadline: u64,
        reference: u64,
    ) -> ExecutionEngineDomainRequest {
        let mut req = request(ExecutionDomainCommandKind::ArmSlaTimer);
        req.step = None;
        req.sla_timer = Some(sla_timer(armed_at, deadline));
        req.sla_reference_epoch_seconds = reference;
        req
    }

    #[test]
    fn execution_domain_accepts_dispatch_and_attaches_evidence_refs() {
        let receipt = evaluate_execution_domain(request(ExecutionDomainCommandKind::DispatchStep))
            .expect_accepted();

        assert_eq!(receipt.next_run_status, WorkflowExecutionStatus::Running);
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Leased));
        assert!(receipt.dispatch_required);
        assert!(
            receipt
                .audit_refs
                .contains(&"cedar://workflow/execution/dispatch".to_owned())
        );
        assert!(
            receipt
                .audit_refs
                .contains(&"spec-integrity:workflow:v1".to_owned())
        );
        assert!(
            receipt
                .audit_refs
                .contains(&"replay-epoch:execution-domain:1".to_owned())
        );
        assert!(
            receipt
                .audit_refs
                .contains(&"scheduler-epoch:execution-domain:1".to_owned())
        );
    }

    #[test]
    fn start_run_requires_pending_run_and_pending_first_step() {
        let mut start = request(ExecutionDomainCommandKind::StartRun);
        start.run = run_with_status(WorkflowExecutionStatus::Pending);

        let receipt = evaluate_execution_domain(start).expect_accepted();

        assert_eq!(receipt.next_run_status, WorkflowExecutionStatus::Running);
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Pending));
        assert!(receipt.dispatch_required);
    }

    #[test]
    fn scope_mismatch_denies_before_dispatch_or_store_side_effects() {
        let mut invalid = request(ExecutionDomainCommandKind::DispatchStep);
        invalid.expected_cell_id = "cell:euw1:b".to_owned();
        invalid.step.as_mut().unwrap().tenant_id = "ten_other".to_owned();

        let denial = evaluate_execution_domain(invalid).expect_denied();

        assert_eq!(denial.kind, ExecutionDomainDenialKind::ScopeMismatch);
        assert!(
            denial
                .audit_refs
                .contains(&"workflow-execution-domain:scope-mismatch".to_owned())
        );
    }

    #[test]
    fn missing_policy_spec_replay_or_scheduler_evidence_denies_before_kernel_values() {
        let mut invalid = request(ExecutionDomainCommandKind::DispatchStep);
        invalid.policy_evidence_ref.clear();
        invalid.spec_integrity_ref = " ".to_owned();
        invalid.replay_epoch_ref.clear();
        invalid.scheduler_epoch_ref.clear();

        let denial = evaluate_execution_domain(invalid).expect_denied();

        assert_eq!(denial.kind, ExecutionDomainDenialKind::MissingEvidence);
        assert!(
            denial
                .audit_refs
                .contains(&"validation:policy-evidence-required".to_owned())
        );
        assert!(
            denial
                .audit_refs
                .contains(&"validation:spec-integrity-required".to_owned())
        );
        assert!(
            denial
                .audit_refs
                .contains(&"validation:replay-epoch-required".to_owned())
        );
        assert!(
            denial
                .audit_refs
                .contains(&"validation:scheduler-epoch-required".to_owned())
        );
    }

    #[test]
    fn terminal_run_denies_step_dispatch_without_runtime_side_effects() {
        let mut invalid = request(ExecutionDomainCommandKind::DispatchStep);
        invalid.run = run_with_status(WorkflowExecutionStatus::Completed);

        let denial = evaluate_execution_domain(invalid).expect_denied();

        assert_eq!(denial.kind, ExecutionDomainDenialKind::InvalidState);
        assert!(
            denial
                .audit_refs
                .contains(&"workflow-execution-domain:invalid-state".to_owned())
        );
    }

    #[test]
    fn retry_schedule_requires_failed_step_and_later_attempt_binding() {
        let mut retry = request(ExecutionDomainCommandKind::ScheduleRetry);
        retry.step = Some(step_with_status(StepExecutionStatus::Failed, 1));
        retry.retry_attempt = Some(
            RetryAttempt::new(
                "ten_a",
                "run:execution-domain:1",
                "step:approve",
                2,
                "error-class:retryable-http-503",
                "retry-policy:workflow-standard",
                vec!["workflow-execution-domain:retry".to_owned()],
            )
            .unwrap(),
        );

        let receipt = evaluate_execution_domain(retry).expect_accepted();

        assert!(receipt.retry_scheduled);
        assert_eq!(receipt.step_status, Some(StepExecutionStatus::Retrying));
    }

    #[test]
    fn sla_timer_arm_validates_scope_and_keeps_timer_metadata_only() {
        let mut timer_request = request(ExecutionDomainCommandKind::ArmSlaTimer);
        timer_request.step = None;
        timer_request.sla_timer = Some(
            SlaTimer::new(
                "timer:execution-domain:1",
                "ten_a",
                "run:execution-domain:1",
                Some(0),
                100,
                160,
                vec!["workflow-execution-domain:sla".to_owned()],
            )
            .unwrap(),
        );

        let receipt = evaluate_execution_domain(timer_request).expect_accepted();

        assert!(receipt.sla_timer_armed);
        assert!(!receipt.dispatch_required);
        assert!(
            receipt
                .audit_refs
                .contains(&"workflow-execution-domain:sla".to_owned())
        );
    }

    #[test]
    fn raw_prompt_output_or_secret_shaped_domain_metadata_is_rejected_without_echo() {
        let mut invalid = request(ExecutionDomainCommandKind::DispatchStep);
        invalid.policy_evidence_ref = "Authorization: Bearer sk-test raw prompt".to_owned();

        let denial = evaluate_execution_domain(invalid).expect_denied();

        assert_eq!(denial.kind, ExecutionDomainDenialKind::UnsafeMetadata);
        let rendered = format!("{denial:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn deterministic_audit_refs_are_sorted_and_deduplicated() {
        let mut duplicate = request(ExecutionDomainCommandKind::DispatchStep);
        duplicate.spec_integrity_ref = duplicate.policy_evidence_ref.clone();

        let receipt = evaluate_execution_domain(duplicate).expect_accepted();
        let mut sorted = receipt.audit_refs.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(receipt.audit_refs, sorted);
    }

    // --- SLA deadline classification tests (subtask 1 + 2 + 3) ---

    #[test]
    fn sla_deadline_class_on_track_when_reference_well_before_at_risk_threshold() {
        // window = 200-100 = 100; at_risk_at = 100 + 100*80/100 = 180
        // reference = 120 < 180 → OnTrack
        let timer = sla_timer(100, 200);
        let class = classify_sla_deadline(&timer, 120);
        assert_eq!(class, SlaDeadlineClass::OnTrack);
        assert_eq!(class.as_wire(), "on-track");
    }

    #[test]
    fn sla_deadline_class_at_risk_when_reference_past_eighty_percent_of_window() {
        // window = 200-100 = 100; at_risk_at = 100 + 80 = 180
        // reference = 185 >= 180 and < 200 → AtRisk
        let timer = sla_timer(100, 200);
        let class = classify_sla_deadline(&timer, 185);
        assert_eq!(class, SlaDeadlineClass::AtRisk);
        assert_eq!(class.as_wire(), "at-risk");
    }

    #[test]
    fn sla_deadline_class_breached_when_reference_at_or_past_deadline() {
        // reference = 200 >= deadline 200 → Breached
        let timer = sla_timer(100, 200);
        let class_at = classify_sla_deadline(&timer, 200);
        assert_eq!(class_at, SlaDeadlineClass::Breached);
        assert_eq!(class_at.as_wire(), "breached");

        // reference past deadline also Breached
        let class_past = classify_sla_deadline(&timer, 999);
        assert_eq!(class_past, SlaDeadlineClass::Breached);
    }

    #[test]
    fn arm_sla_timer_accepts_with_on_track_classification_audit_ref() {
        // reference 120 in window [100, 200) → OnTrack
        let req = arm_sla_timer_request(100, 200, 120);
        let receipt = evaluate_execution_domain(req).expect_accepted();

        assert!(receipt.sla_timer_armed);
        assert!(!receipt.dispatch_required);
        assert!(
            receipt
                .audit_refs
                .contains(&"workflow-execution-domain:sla-class:on-track".to_owned()),
            "expected sla-class:on-track in audit_refs, got {:?}",
            receipt.audit_refs
        );
    }

    #[test]
    fn arm_sla_timer_accepts_with_breached_classification_audit_ref() {
        // reference 200 >= deadline 200 → Breached
        let req = arm_sla_timer_request(100, 200, 200);
        let receipt = evaluate_execution_domain(req).expect_accepted();

        assert!(receipt.sla_timer_armed);
        assert!(
            receipt
                .audit_refs
                .contains(&"workflow-execution-domain:sla-class:breached".to_owned()),
            "expected sla-class:breached in audit_refs, got {:?}",
            receipt.audit_refs
        );
    }

    #[test]
    fn arm_sla_timer_missing_sla_timer_denies_with_invalid_command_shape() {
        let mut req = request(ExecutionDomainCommandKind::ArmSlaTimer);
        req.step = None;
        // sla_timer deliberately left as None

        let denial = evaluate_execution_domain(req).expect_denied();

        assert_eq!(denial.kind, ExecutionDomainDenialKind::InvalidCommandShape);
        assert!(
            denial
                .audit_refs
                .contains(&"workflow-execution-domain:invalid-command-shape".to_owned()),
            "expected invalid-command-shape in audit_refs, got {:?}",
            denial.audit_refs
        );
    }

    // --- SLA escalation bucket projection tests ---

    fn escalation_timer(armed_at: u64, deadline: u64) -> SlaTimer {
        SlaTimer::new(
            "timer:escalation:1",
            "ten_a",
            "run:escalation:1",
            Some(0),
            armed_at,
            deadline,
            vec!["workflow-execution-domain:sla-escalation".to_owned()],
        )
        .unwrap()
    }

    #[test]
    fn escalation_none_when_reference_well_before_at_risk_threshold() {
        // window=100, at_risk_at=180, reference=120 → None
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 120, 30);
        assert_eq!(level, SlaEscalationLevel::None);
        assert_eq!(level.as_wire(), "none");
    }

    #[test]
    fn escalation_notify_when_reference_exactly_at_eighty_percent_threshold() {
        // window=100, at_risk_at=180, reference=180 → Notify
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 180, 30);
        assert_eq!(level, SlaEscalationLevel::Notify);
        assert_eq!(level.as_wire(), "notify");
    }

    #[test]
    fn escalation_notify_when_reference_between_threshold_and_deadline() {
        // at_risk_at=180, deadline=200, reference=195 → Notify
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 195, 30);
        assert_eq!(level, SlaEscalationLevel::Notify);
    }

    #[test]
    fn escalation_page_when_reference_exactly_at_deadline_within_grace() {
        // deadline=200, grace=30, grace_end=230, reference=200 → Page
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 200, 30);
        assert_eq!(level, SlaEscalationLevel::Page);
        assert_eq!(level.as_wire(), "page");
    }

    #[test]
    fn escalation_page_when_reference_inside_grace_window_past_deadline() {
        // deadline=200, grace=30, grace_end=230, reference=215 → Page
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 215, 30);
        assert_eq!(level, SlaEscalationLevel::Page);
    }

    #[test]
    fn escalation_auto_abort_when_reference_exactly_at_grace_end() {
        // deadline=200, grace=30, grace_end=230, reference=230 → AutoAbort
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 230, 30);
        assert_eq!(level, SlaEscalationLevel::AutoAbort);
        assert_eq!(level.as_wire(), "auto-abort");
    }

    #[test]
    fn escalation_auto_abort_when_reference_exceeds_grace_end() {
        // reference=999 → AutoAbort
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 999, 30);
        assert_eq!(level, SlaEscalationLevel::AutoAbort);
    }

    #[test]
    fn escalation_is_monotonic_across_increasing_reference_epochs() {
        let timer = escalation_timer(100, 200);
        let grace = 30u64;
        // sample reference points spanning all four buckets
        let levels: Vec<SlaEscalationLevel> = [120u64, 180, 195, 200, 215, 230, 999]
            .iter()
            .map(|&r| project_sla_escalation(&timer, r, grace))
            .collect();

        for window in levels.windows(2) {
            assert!(
                window[0] <= window[1],
                "escalation must be monotone: {:?} > {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn escalation_saturating_arithmetic_on_max_breach_grace_seconds() {
        // deadline + u64::MAX must not overflow (saturates to u64::MAX)
        let timer = escalation_timer(100, 200);
        // reference = 200 → below grace_end (saturated u64::MAX) → Page
        let level = project_sla_escalation(&timer, 200, u64::MAX);
        assert_eq!(level, SlaEscalationLevel::Page);
        // reference = u64::MAX → equals saturated grace_end → AutoAbort
        let level_max = project_sla_escalation(&timer, u64::MAX, u64::MAX);
        assert_eq!(level_max, SlaEscalationLevel::AutoAbort);
    }

    #[test]
    fn escalation_ord_ordering_none_lt_notify_lt_page_lt_auto_abort() {
        assert!(SlaEscalationLevel::None < SlaEscalationLevel::Notify);
        assert!(SlaEscalationLevel::Notify < SlaEscalationLevel::Page);
        assert!(SlaEscalationLevel::Page < SlaEscalationLevel::AutoAbort);
    }

    #[test]
    fn escalation_zero_grace_auto_aborts_at_deadline() {
        // breach_grace_seconds=0 → grace_end=deadline, reference=200 → AutoAbort immediately
        let timer = escalation_timer(100, 200);
        let level = project_sla_escalation(&timer, 200, 0);
        assert_eq!(level, SlaEscalationLevel::AutoAbort);
    }

    #[test]
    fn sla_deadline_class_deterministic_same_input_yields_byte_stable_audit_refs() {
        let req1 = arm_sla_timer_request(100, 200, 150);
        let req2 = arm_sla_timer_request(100, 200, 150);

        let receipt1 = evaluate_execution_domain(req1).expect_accepted();
        let receipt2 = evaluate_execution_domain(req2).expect_accepted();

        // byte-stable: identical input → identical audit_refs vector
        assert_eq!(
            receipt1.audit_refs, receipt2.audit_refs,
            "audit_refs must be byte-stable across identical evaluations"
        );
    }
}
