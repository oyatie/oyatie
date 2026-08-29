//! Settlement regression coverage.

use super::test_support::*;
use crate::*;

#[test]
fn settlement_records_run_completion_failure_without_masking_primary_error() {
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
                "res_already_final".into(),
                request.started_at_epoch_seconds,
            )
            .unwrap(),
        )
        .unwrap();
    foundation
        .foundry_runs
        .complete(
            &run.run_id.value,
            RunDisposition::Success,
            request.started_at_epoch_seconds.saturating_add(1),
        )
        .unwrap();

    let error = foundation
        .settle_failed_invocation(
            &request,
            None,
            Some(&run.run_id.value),
            RunDisposition::FailureProvider,
            FoundationError::CostBudgetExceeded,
        )
        .unwrap();

    assert_eq!(error, FoundationError::CostBudgetExceeded);
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| { event.surface == "foundry.run.complete" && event.decision == "DENY" })
    );
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
        Some("not_applicable")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("run_completion")
            .map(String::as_str),
        Some("failed")
    );
}

#[test]
fn settlement_audits_no_run_budget_release_without_masking_primary_error() {
    let mut foundation = settlement_foundation();
    let request = settlement_request();
    let scope = settlement_scope();
    let reservation = foundation.cost_budgets.reserve(&scope, 10).unwrap();

    let error = foundation
        .settle_failed_invocation(
            &request,
            Some(&reservation.reservation_id.value),
            None,
            RunDisposition::FailureProvider,
            FoundationError::InvalidInput,
        )
        .unwrap();

    assert_eq!(error, FoundationError::InvalidInput);
    assert_eq!(
        foundation
            .cost_budgets
            .release(&reservation.reservation_id.value),
        Err(BudgetError::ReservationNotPending)
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.cost-budget.release" && event.decision == "ALLOW"
    }));
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.invocation.compensate" && event.decision == "ALLOW"
    }));
    assert!(
        foundation.foundry_evidence_chain().records().is_empty(),
        "no-run settlement cannot append run-scoped evidence"
    );
    assert!(
        foundation.outbox_records().is_empty(),
        "no-run settlement has no evidence record to publish"
    );
}
