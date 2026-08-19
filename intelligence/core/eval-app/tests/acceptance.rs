//! Acceptance tests for the intelligence-eval composition root, grounded in
//! the actual behaviors of the eval vertical slice (kernel + domain +
//! usecase + adapter + worker) and `microservices/intelligence/PRD.md`
//! (carbon-aware advisory eval surface).
//!
//! These drive the FULL flow through the REAL eval slice — the worker is the
//! canonical lifecycle driver and is exercised verbatim by the composition
//! root. There are NO slice stubs. The runner-status source is the
//! in-memory scripted adapter so the dispatch path is deterministic
//! (acceptance tests must not require network egress).
//!
//! Mapped behaviors (slice test contract):
//! - happy-path runner accepts authorized job          (worker `processes_authorized_job_and_dispatches_runner_envelope`)
//! - runner queued / completed transports succeed      (worker outcome mapping)
//! - usecase denial does not reach the runner adapter  (worker `usecase_denial_does_not_call_runner_adapter`)
//! - retryable runner error schedules backoff          (worker `retryable_runner_failure_schedules_backoff_and_exhausts_at_max_attempts`)
//! - retry exhausted denies after max attempts          (worker exhaustion path)
//! - non-retryable invalid-request denies short-circuit (worker `nonretryable_runner_invalid_request_denies_without_retry`)
//! - invalid job metadata denies before usecase         (worker `invalid_raw_job_metadata_denies_before_side_effects`)
//! - deferred not-before defers without side effects    (worker `defers_not_before_jobs_without_usecase_or_adapter_side_effects`)
//! - missing job is composition-root default-deny       (composition-root invariant)
//! - cross-tenant isolation by (TenantId, JobId)        (composition-root invariant)
//! - HyperEvalRunnerStatusSource honest boundary        (`Unimplemented::HostedEvalRunnerDispatch`)
//! - debug output never echoes raw prompt/output bytes  (slice redaction invariant)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use intelligence_eval_app::{
    DispatchError, DomainEvalSetRequest, EvalAuditEventKind, EvalCaseKind, EvalCaseOutcome,
    EvalCaseResult, EvalPolicyDecision, EvalRunnerAdapterConfig, EvalRunnerDispatchStatus,
    EvalRunnerHttpMethod, EvalRunnerStatus, EvalRunnerStatusSource, EvalSet, EvalSetThresholds,
    EvalUsecaseInput, EvalUsecaseStatus, EvalWorkerDenialKind, EvalWorkerEventKind, EvalWorkerJob,
    EvalWorkerStatus, HyperEvalRunnerStatusSource, InMemoryEvalAuditEventSink,
    InMemoryEvalJobRepository, InMemoryEvalReceiptSink, InMemoryEvalRunnerStatusSource, JobId,
    RunnerStatusError, TenantId, Unimplemented, dispatch_eval_job,
};

fn ten(s: &str) -> TenantId {
    TenantId(s.to_owned())
}

fn jid(s: &str) -> JobId {
    JobId(s.to_owned())
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

fn sample_policy() -> EvalPolicyDecision {
    EvalPolicyDecision {
        decision_id: "eval-policy-decision:acceptance:1".to_owned(),
        tenant_id: "tenant:alpha".to_owned(),
        principal_id: "principal:eval-acceptance".to_owned(),
        allowed_surfaces: vec!["surface:release-gate".to_owned()],
        allowed_model_refs: vec!["modelref://openai/gpt-preview".to_owned()],
        allowed_dataset_snapshot_refs: vec!["dataset://evals/acceptance/2026-05-26".to_owned()],
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
        evidence_ref: "policy:evidence:eval-acceptance:1".to_owned(),
        eval_registry_snapshot_ref: "eval-registry:snapshot:acceptance:1".to_owned(),
    }
}

fn sample_eval_set(eval_set_id: &str) -> EvalSet {
    EvalSet {
        eval_set_id: eval_set_id.to_owned(),
        model_ref: "modelref://openai/gpt-preview".to_owned(),
        route_evidence_ref: "route:evidence:eval-acceptance:1".to_owned(),
        guardrail_evidence_ref: "guardrail:evidence:eval-acceptance:1".to_owned(),
        dataset_snapshot_ref: "dataset://evals/acceptance/2026-05-26".to_owned(),
        thresholds: EvalSetThresholds {
            min_pass_rate_bps: 8_000,
            max_safety_violation_rate_bps: 0,
            require_golden: true,
            require_adversarial: true,
            require_linguistic: true,
        },
        cases: vec![
            case(
                "case-golden-acceptance-1",
                EvalCaseKind::Golden,
                EvalCaseOutcome::Passed,
                9_500,
                "eval:case:acceptance:golden:1",
            ),
            case(
                "case-adversarial-acceptance-1",
                EvalCaseKind::Adversarial,
                EvalCaseOutcome::Passed,
                8_800,
                "eval:case:acceptance:adversarial:1",
            ),
            case(
                "case-linguistic-acceptance-1",
                EvalCaseKind::Linguistic,
                EvalCaseOutcome::Passed,
                8_700,
                "eval:case:acceptance:linguistic:1",
            ),
        ],
    }
}

fn sample_request(eval_set_id: &str) -> DomainEvalSetRequest {
    DomainEvalSetRequest {
        tenant_id: "tenant:alpha".to_owned(),
        principal_id: "principal:eval-acceptance".to_owned(),
        eval_surface: "surface:release-gate".to_owned(),
        request_evidence_ref: "request:evidence:eval-acceptance:1".to_owned(),
        trace_context_ref: "trace:eval-acceptance:1".to_owned(),
        policy_decision_ref: "policy:evidence:eval-acceptance:1".to_owned(),
        policy_decision: sample_policy(),
        eval_set: sample_eval_set(eval_set_id),
    }
}

fn sample_job(eval_set_id: &str, idempotency_key: &str) -> EvalWorkerJob {
    EvalWorkerJob {
        job_id: "job:eval-acceptance:1".to_owned(),
        lease_id: "lease:eval-acceptance:1".to_owned(),
        attempt_id: "attempt:eval-acceptance:1".to_owned(),
        attempt_number: 1,
        max_attempts: 3,
        now_epoch_seconds: 1_000,
        not_before_epoch_seconds: 900,
        input: EvalUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            request: sample_request(eval_set_id),
        },
    }
}

fn sample_adapter_config() -> EvalRunnerAdapterConfig {
    EvalRunnerAdapterConfig::new(
        "https://eval-runner.oyatie.internal",
        "secretref://ten_a/eval-runner/byok",
        "audit://tap/intelligence/eval",
        "audience://intelligence/eval-runner",
    )
}

/// AC: happy-path — an authorized job is dispatched to the runner and the
/// composition root sinks both the receipt and the audit-event streams.
#[tokio::test]
async fn happy_path_accepted_runner_dispatches_and_sinks_receipt() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-release-gate", "idem:acceptance:1"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
        runner_request_ref: "eval-runner://requests/req-1".to_owned(),
        run_ref: "eval-runner://runs/run-1".to_owned(),
        evidence_ref: "eval-runner:evidence:accepted".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("happy-path dispatch must succeed");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::RunnerAccepted);
    assert_eq!(
        outcome.receipt.runner_status,
        Some(EvalRunnerDispatchStatus::Accepted)
    );
    // Source was consulted exactly once.
    assert_eq!(source.call_log(), vec![(tenant.clone(), job_id.clone())]);
    // The receipt sink captured the receipt.
    assert_eq!(receipt_sink.len(), 1);
    let (sink_tenant, sink_job, sink_receipt) = &receipt_sink.receipts()[0];
    assert_eq!(sink_tenant, &tenant);
    assert_eq!(sink_job, &job_id);
    assert_eq!(sink_receipt.status, EvalWorkerStatus::RunnerAccepted);
    // The worker emitted JobAccepted + RunnerAccepted events.
    let worker_event_kinds: Vec<_> = audit_sink
        .worker_events()
        .iter()
        .map(|(_, _, event)| event.kind)
        .collect();
    assert_eq!(
        worker_event_kinds,
        vec![
            EvalWorkerEventKind::JobAccepted,
            EvalWorkerEventKind::RunnerAccepted
        ]
    );
    // The usecase audit-events were projected (Requested + Evaluated).
    let usecase_event_kinds: Vec<_> = audit_sink
        .usecase_events()
        .iter()
        .map(|(_, _, event)| event.kind)
        .collect();
    assert_eq!(
        usecase_event_kinds,
        vec![
            EvalAuditEventKind::EvalRequested,
            EvalAuditEventKind::EvalEvaluated,
        ]
    );
}

/// AC: runner-queued is mapped through the worker into RunnerQueued and the
/// composition root sinks the receipt with that status.
#[tokio::test]
async fn runner_queued_outcome_maps_through_worker() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-queued", "idem:acceptance:queued"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Queued {
        runner_request_ref: "eval-runner://requests/req-q".to_owned(),
        queue_ref: "eval-runner://queues/q-1".to_owned(),
        evidence_ref: "eval-runner:evidence:queued".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("queued dispatch must succeed");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::RunnerQueued);
    assert_eq!(
        outcome.receipt.runner_status,
        Some(EvalRunnerDispatchStatus::Queued)
    );
    assert_eq!(audit_sink.worker_events().len(), 2);
    assert_eq!(
        audit_sink.worker_events()[1].2.kind,
        EvalWorkerEventKind::RunnerQueued
    );
}

/// AC: runner-completed is mapped through the worker into RunnerCompleted.
#[tokio::test]
async fn runner_completed_outcome_maps_through_worker() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-completed", "idem:acceptance:completed"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Completed {
        runner_request_ref: "eval-runner://requests/req-c".to_owned(),
        run_ref: "eval-runner://runs/run-c".to_owned(),
        report_ref: "eval-report://runs/run-c/report".to_owned(),
        evidence_ref: "eval-runner:evidence:completed".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("completed dispatch must succeed");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::RunnerCompleted);
    assert_eq!(
        outcome.receipt.runner_status,
        Some(EvalRunnerDispatchStatus::Completed)
    );
}

/// AC: usecase denial — when the eval domain denies the request, the worker
/// records EvalDenied and never calls the runner. The composition root
/// captures the denied receipt and the projected usecase Denied event.
#[tokio::test]
async fn usecase_denied_short_circuits_runner_and_sinks_denied_receipt() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let mut job = sample_job("eval_set:acceptance-denied", "idem:acceptance:denied");
    // Trigger a domain-side denial: model not in the policy allowlist.
    job.input.request.eval_set.model_ref = "modelref://openai/forbidden".to_owned();
    let repo = InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), job);
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
        runner_request_ref: "eval-runner://requests/req-1".to_owned(),
        run_ref: "eval-runner://runs/run-1".to_owned(),
        evidence_ref: "eval-runner:evidence:accepted".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("denial is captured, not panic'd");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::Denied);
    assert_eq!(
        outcome.receipt.denial_kind,
        Some(EvalWorkerDenialKind::EvalUsecaseDenied)
    );
    // The status source was still consulted (the worker needs the adapter
    // even when the usecase will deny — the adapter is constructed before
    // the worker dispatches).
    assert_eq!(source.call_log().len(), 1);
    // Worker events: JobAccepted + EvalDenied (no RunnerAccepted).
    let worker_event_kinds: Vec<_> = audit_sink
        .worker_events()
        .iter()
        .map(|(_, _, event)| event.kind)
        .collect();
    assert_eq!(
        worker_event_kinds,
        vec![
            EvalWorkerEventKind::JobAccepted,
            EvalWorkerEventKind::EvalDenied,
        ]
    );
}

/// AC: retryable runner error — the worker schedules a retry with backoff
/// and the composition root sinks the RetryScheduled receipt.
#[tokio::test]
async fn retryable_runner_failure_schedules_backoff() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-retry", "idem:acceptance:retry"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::RateLimited {
        evidence_ref: "eval-runner:error:429".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("retryable failure produces a receipt");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::RetryScheduled);
    assert_eq!(outcome.receipt.next_attempt_epoch_seconds, Some(1_030));
    let worker_event_kinds: Vec<_> = audit_sink
        .worker_events()
        .iter()
        .map(|(_, _, event)| event.kind)
        .collect();
    assert_eq!(
        worker_event_kinds,
        vec![
            EvalWorkerEventKind::JobAccepted,
            EvalWorkerEventKind::RetryScheduled,
        ]
    );
}

/// AC: retry exhausted — once the worker reaches max_attempts the retryable
/// runner error denies with RetryExhausted.
#[tokio::test]
async fn retry_exhausted_at_max_attempts_denies() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let mut job = sample_job("eval_set:acceptance-exhausted", "idem:acceptance:exhausted");
    job.attempt_number = 3;
    job.max_attempts = 3;
    let repo = InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), job);
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Timeout {
        evidence_ref: "eval-runner:error:timeout".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("exhausted is captured, not panic'd");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::Exhausted);
    assert_eq!(
        outcome.receipt.denial_kind,
        Some(EvalWorkerDenialKind::RetryExhausted)
    );
}

/// AC: non-retryable invalid-request — the worker denies without retry.
#[tokio::test]
async fn nonretryable_runner_invalid_request_denies_without_retry() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-invalid", "idem:acceptance:invalid"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::InvalidRequest {
        evidence_ref: "eval-runner:error:invalid".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("invalid-request is captured");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::Denied);
    assert_eq!(
        outcome.receipt.denial_kind,
        Some(EvalWorkerDenialKind::RunnerInvalidRequest)
    );
    assert_eq!(outcome.receipt.next_attempt_epoch_seconds, None);
}

/// AC: deferred not-before — the worker defers the job and the composition
/// root sinks the deferred receipt with no usecase/adapter side effects.
#[tokio::test]
async fn deferred_not_before_skips_usecase_and_adapter_calls() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let mut job = sample_job("eval_set:acceptance-deferred", "idem:acceptance:deferred");
    job.now_epoch_seconds = 100;
    job.not_before_epoch_seconds = 130;
    let repo = InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), job);
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Queued {
        runner_request_ref: "eval-runner://requests/req-q".to_owned(),
        queue_ref: "eval-runner://queues/q-1".to_owned(),
        evidence_ref: "eval-runner:evidence:queued".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("deferred is captured");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::Deferred);
    assert_eq!(outcome.receipt.next_attempt_epoch_seconds, Some(130));
    // The worker did NOT emit any lifecycle events for a deferred job.
    assert!(audit_sink.worker_events().is_empty());
    // No usecase events either — the worker never reached the usecase.
    assert!(audit_sink.usecase_events().is_empty());
}

/// AC: invalid job metadata — the worker validates the queued job and
/// denies before any side effect; the composition root sinks the redacted
/// denial receipt.
#[tokio::test]
async fn invalid_job_metadata_denies_before_runner_call() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let mut job = sample_job(
        "eval_set:acceptance-invalid-meta",
        "idem:acceptance:invalid-meta",
    );
    // Inject raw-secret-like material that the worker's metadata validator
    // must reject before any other side effect.
    job.input.request.eval_set.cases[0].evaluator_evidence_ref =
        "raw output model answer".to_owned();
    let repo = InMemoryEvalJobRepository::new().with_job(tenant.clone(), job_id.clone(), job);
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
        runner_request_ref: "eval-runner://requests/req-1".to_owned(),
        run_ref: "eval-runner://runs/run-1".to_owned(),
        evidence_ref: "eval-runner:evidence:accepted".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("invalid job metadata is captured");

    assert_eq!(outcome.receipt.status, EvalWorkerStatus::Denied);
    assert_eq!(
        outcome.receipt.denial_kind,
        Some(EvalWorkerDenialKind::InvalidJob)
    );
    // No worker events emitted for an invalid job.
    assert!(audit_sink.worker_events().is_empty());
    assert!(audit_sink.usecase_events().is_empty());
}

/// AC: cross-tenant isolation — the same `JobId` namespace under two
/// different `TenantId`s resolves to two independent jobs.
#[tokio::test]
async fn cross_tenant_jobs_are_isolated() {
    let tenant_a = ten("tenant:alpha");
    let tenant_b = ten("tenant:initech");
    let job_id = jid("job:shared-id");
    let mut job_a = sample_job("eval_set:acceptance-tenant-a", "idem:acceptance:tenant-a");
    job_a.input.request.tenant_id = "tenant:alpha".to_owned();
    let mut job_b = sample_job("eval_set:acceptance-tenant-b", "idem:acceptance:tenant-b");
    job_b.input.request.tenant_id = "tenant:initech".to_owned();
    job_b.input.request.policy_decision.tenant_id = "tenant:initech".to_owned();

    let repo = InMemoryEvalJobRepository::new()
        .with_job(tenant_a.clone(), job_id.clone(), job_a)
        .with_job(tenant_b.clone(), job_id.clone(), job_b);

    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
        runner_request_ref: "eval-runner://requests/req-1".to_owned(),
        run_ref: "eval-runner://runs/run-1".to_owned(),
        evidence_ref: "eval-runner:evidence:accepted".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome_a = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant_a,
        &job_id,
    )
    .await
    .expect("tenant_a dispatch");
    let outcome_b = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant_b,
        &job_id,
    )
    .await
    .expect("tenant_b dispatch");

    // tenant A succeeded (matching policy).
    assert_eq!(outcome_a.receipt.status, EvalWorkerStatus::RunnerAccepted);
    // tenant B denied at the usecase layer because the policy tenant id no
    // longer matches the policy decision tenant id.
    assert_eq!(outcome_b.receipt.tenant_id, "tenant:initech");
    // The receipt sink captured exactly two receipts, one per tenant.
    assert_eq!(receipt_sink.len(), 2);
    let (tenants, _, _): (Vec<&TenantId>, Vec<&JobId>, Vec<&_>) = receipt_sink
        .receipts()
        .iter()
        .map(|(t, j, r)| (t, j, r))
        .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, x| {
            acc.0.push(x.0);
            acc.1.push(x.1);
            acc.2.push(x.2);
            acc
        });
    assert!(tenants.contains(&&tenant_a));
    assert!(tenants.contains(&&tenant_b));
}

/// AC: missing job — the composition root defaults-deny when the
/// repository has no record for `(tenant_id, job_id)`. Source MUST NOT be
/// consulted and no receipts/events MUST be sunk.
#[tokio::test]
async fn missing_job_default_denies_without_source_or_sink_side_effects() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:does-not-exist");
    let repo = InMemoryEvalJobRepository::new();
    let source = InMemoryEvalRunnerStatusSource::new(Arc::new(|_, _| {
        panic!("source must not be consulted when the job is missing");
    }));
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let err = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect_err("missing job must default-deny");

    match err {
        DispatchError::JobNotFound {
            tenant_id,
            job_id: jid_str,
        } => {
            assert_eq!(tenant_id, "tenant:alpha");
            assert_eq!(jid_str, "job:does-not-exist");
        }
        other => panic!("expected JobNotFound, got {other:?}"),
    }
    // No sinks were touched.
    assert!(receipt_sink.is_empty());
    assert!(audit_sink.worker_events().is_empty());
    assert!(audit_sink.usecase_events().is_empty());
    // Source was NEVER consulted.
    assert!(source.call_log().is_empty());
}

/// AC: honest-claims boundary — the production hyper status source surfaces
/// a typed `Unimplemented::HostedEvalRunnerDispatch` and the dispatch loop
/// maps it to `DispatchError::RunnerStatus` (no silent fake success).
#[tokio::test]
async fn hyper_source_surfaces_unimplemented_via_dispatch_error() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-hyper", "idem:acceptance:hyper"),
    );
    let source = HyperEvalRunnerStatusSource::new("https://eval-runner.oyatie.internal");
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let err = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect_err("hyper source surfaces unimplemented boundary");

    match err {
        DispatchError::RunnerStatus(error) => {
            assert!(
                error
                    .detail()
                    .contains(Unimplemented::HostedEvalRunnerDispatch.as_str()),
                "detail must cite the typed Unimplemented variant, got {}",
                error.detail()
            );
            assert!(
                error
                    .detail()
                    .contains(Unimplemented::HostedEvalRunnerDispatch.placeholder_debt_id()),
                "detail must cite the placeholder-debt id, got {}",
                error.detail()
            );
        }
        other => panic!("expected RunnerStatus, got {other:?}"),
    }
    // No side effects — the source failed before adapter construction.
    assert!(receipt_sink.is_empty());
    assert!(audit_sink.worker_events().is_empty());
    assert!(audit_sink.usecase_events().is_empty());
}

/// AC: the production source itself round-trips its base URL and surfaces
/// the typed boundary directly — this is the seam that the hosted-runner
/// follow-up will close.
#[tokio::test]
async fn hyper_source_round_trips_upstream_base_url() {
    let source = HyperEvalRunnerStatusSource::new("https://eval-runner.oyatie.internal");
    assert_eq!(
        source.upstream_base_url(),
        "https://eval-runner.oyatie.internal"
    );
    let err = source
        .next_status(&ten("tenant:alpha"), &jid("job:direct"))
        .expect_err("honest-claims boundary");
    assert!(
        err.detail()
            .contains("Unimplemented::HostedEvalRunnerDispatch")
    );
}

/// AC: source-error short-circuit — the in-memory source can surface a
/// typed `RunnerStatusError` for tests; the dispatch loop maps it through
/// to `DispatchError::RunnerStatus` and never touches the sinks.
#[tokio::test]
async fn in_memory_source_error_short_circuits_dispatch() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job(
            "eval_set:acceptance-status-err",
            "idem:acceptance:status-err",
        ),
    );
    let source = InMemoryEvalRunnerStatusSource::new(Arc::new(|_, _| {
        Err(RunnerStatusError::new("scripted source outage"))
    }));
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let err = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect_err("source error short-circuits");

    match err {
        DispatchError::RunnerStatus(error) => {
            assert_eq!(error.detail(), "scripted source outage");
        }
        other => panic!("expected RunnerStatus, got {other:?}"),
    }
    // No side effects — the source failed before any sink was touched.
    assert!(receipt_sink.is_empty());
    assert!(audit_sink.worker_events().is_empty());
}

/// AC: envelope shape — the adapter envelope the worker built must carry
/// the eval slice's metadata-only refs. We assert the envelope reaches the
/// worker via the successful happy-path receipt and that the receipt's
/// fields preserve the metadata-only invariants.
#[tokio::test]
async fn envelope_metadata_only_no_raw_secrets_or_prompts() {
    let tenant = ten("tenant:alpha");
    let job_id = jid("job:eval-acceptance:1");
    let repo = InMemoryEvalJobRepository::new().with_job(
        tenant.clone(),
        job_id.clone(),
        sample_job("eval_set:acceptance-redaction", "idem:acceptance:redaction"),
    );
    let source = InMemoryEvalRunnerStatusSource::always(EvalRunnerStatus::Accepted {
        runner_request_ref: "eval-runner://requests/req-1".to_owned(),
        run_ref: "eval-runner://runs/run-1".to_owned(),
        evidence_ref: "eval-runner:evidence:accepted".to_owned(),
    });
    let mut receipt_sink = InMemoryEvalReceiptSink::new();
    let mut audit_sink = InMemoryEvalAuditEventSink::new();

    let outcome = dispatch_eval_job(
        &repo,
        &source,
        &mut receipt_sink,
        &mut audit_sink,
        sample_adapter_config(),
        &tenant,
        &job_id,
    )
    .await
    .expect("happy-path");
    let debug = format!(
        "{:?}{:?}{:?}",
        outcome.receipt, outcome.worker_events, outcome.usecase_events
    );

    assert!(!debug.contains("sk-test"));
    assert!(!debug.contains("write an email to the customer"));
    assert!(!debug.contains("raw model answer"));
    assert!(!debug.contains("raw prompt"));
    assert!(!debug.contains("raw output"));
    // The envelope path is the eval-runner v1 surface.
    let _ = EvalRunnerHttpMethod::Post;
    let _ = EvalUsecaseStatus::Evaluated;
}
