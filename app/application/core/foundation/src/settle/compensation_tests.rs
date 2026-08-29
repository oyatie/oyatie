//! Settlement regression coverage.

use super::test_support::*;
use crate::*;

#[test]
fn settlement_preserves_primary_error_and_records_compensation_evidence() {
    let mut foundation = settlement_foundation();
    let request = settlement_request();
    let scope = settlement_scope();
    let reservation = foundation.cost_budgets.reserve(&scope, 10).unwrap();
    let run = foundation
        .foundry_runs
        .start(
            RunStart::new(
                request.tenant_id.clone(),
                request.capability_id.clone(),
                request.user_id.clone(),
                AutonomyTier::T2Advisory,
                privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
                "failover-region".into(),
                reservation.reservation_id.value.clone(),
                request.started_at_epoch_seconds,
            )
            .unwrap(),
        )
        .unwrap();

    let error = foundation
        .settle_failed_invocation(
            &request,
            Some(&reservation.reservation_id.value),
            Some(&run.run_id.value),
            RunDisposition::FailureProvider,
            FoundationError::CapabilityInvocationUnauthorized,
        )
        .unwrap();

    assert_eq!(error, FoundationError::CapabilityInvocationUnauthorized);
    let settled_run = foundation.foundry_runs().last().unwrap();
    assert_eq!(settled_run.state.value, RunState::Failed);
    assert_eq!(
        settled_run.disposition.value,
        Some(RunDisposition::FailureProvider)
    );
    assert_eq!(
        foundation
            .cost_budgets
            .release(&reservation.reservation_id.value),
        Err(BudgetError::ReservationNotPending)
    );
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .unwrap();
    assert_eq!(evidence.kind.value, EvidenceKind::CapabilityInvocation);
    assert_eq!(evidence.step_id.value, None);
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("invocation_compensation")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("primary_error")
            .map(String::as_str),
        Some("CapabilityInvocationUnauthorized")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("budget_release")
            .map(String::as_str),
        Some("released")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("run_completion")
            .map(String::as_str),
        Some("completed")
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.invocation.compensate" && event.decision == "ALLOW"
    }));
    let outbox = foundation.outbox_records().last().unwrap();
    assert_eq!(outbox.topic.value, "oya.foundry.capability.invoked");
    assert_eq!(outbox.idempotency_key.value, evidence.evidence_id.value);
}

#[test]
fn settlement_continues_after_budget_release_failure_and_preserves_primary_error() {
    let mut foundation = settlement_foundation();
    let request = settlement_request();
    let run = foundation
        .foundry_runs
        .start(
            RunStart::new(
                request.tenant_id.clone(),
                request.capability_id.clone(),
                request.user_id.clone(),
                AutonomyTier::T2Advisory,
                privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap(),
                "failover-region".into(),
                "res_missing".into(),
                request.started_at_epoch_seconds,
            )
            .unwrap(),
        )
        .unwrap();

    let error = foundation
        .settle_failed_invocation(
            &request,
            Some("res_missing"),
            Some(&run.run_id.value),
            RunDisposition::FailureProvider,
            FoundationError::InvalidInput,
        )
        .unwrap();

    assert_eq!(error, FoundationError::InvalidInput);
    let settled_run = foundation.foundry_runs().last().unwrap();
    assert_eq!(settled_run.state.value, RunState::Failed);
    assert_eq!(
        settled_run.disposition.value,
        Some(RunDisposition::FailureProvider)
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.cost-budget.release" && event.decision == "DENY"
    }));
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .unwrap();
    assert_eq!(
        evidence
            .fields
            .value
            .get("budget_release")
            .map(String::as_str),
        Some("failed")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("run_completion")
            .map(String::as_str),
        Some("completed")
    );
}
