#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_domain::{
    HrDomainError, Jurisdiction, LaborComplianceObligationKind, LaborComplianceObligationState,
    LaborComplianceWorkflowStep, LegalEntityWorkforceSnapshot, evaluate_labor_compliance,
};

#[test]
fn test_rules_of_employment_threshold_opens_workflow() {
    let obligations = evaluate_labor_compliance(snapshot_with_count(10)).expect("obligations");

    assert_eq!(obligations.len(), 1);
    assert_eq!(
        obligations[0].kind.value,
        LaborComplianceObligationKind::KoreaRulesOfEmployment
    );
    assert_eq!(
        obligations[0].obligation_id.value.value,
        "hrobl_le_kr_001_korea_rules_of_employment_2026-01-01"
    );
    assert_eq!(
        obligations[0].state.value,
        LaborComplianceObligationState::Open
    );
    assert_eq!(obligations[0].threshold_employee_count.value, 10);
    assert_eq!(
        obligations[0].rulepack_effective_date.value.value,
        "2026-01-01"
    );
    assert_eq!(
        obligations[0].idempotency_key.value,
        "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01"
    );
    assert_eq!(
        obligations[0].workflow_steps.value,
        vec![
            LaborComplianceWorkflowStep::Drafted,
            LaborComplianceWorkflowStep::EmployeeReviewSent,
            LaborComplianceWorkflowStep::MajorityConsentObtained,
            LaborComplianceWorkflowStep::MoelFiled,
            LaborComplianceWorkflowStep::Active,
        ]
    );

    let below_threshold = evaluate_labor_compliance(snapshot_with_count(9)).expect("none");
    assert!(below_threshold.is_empty());
}

#[test]
fn test_snapshot_rejects_prefix_only_and_unsafe_refs() {
    let mut prefix_only_rulepack = snapshot_with_count(10);
    prefix_only_rulepack.rulepack_ref = "rulepack/".to_owned();
    assert_eq!(
        evaluate_labor_compliance(prefix_only_rulepack),
        Err(HrDomainError::InvalidRulepackRef)
    );

    let mut unsafe_workflow = snapshot_with_count(10);
    unsafe_workflow.workflow_ref = "workflow/hr/../compliance".to_owned();
    assert_eq!(
        evaluate_labor_compliance(unsafe_workflow),
        Err(HrDomainError::InvalidWorkflowRef)
    );

    let mut bad_rulepack_date = snapshot_with_count(10);
    bad_rulepack_date.rulepack_effective_date = "2026-99-99".to_owned();
    assert_eq!(
        evaluate_labor_compliance(bad_rulepack_date),
        Err(HrDomainError::InvalidRulepackEffectiveDate)
    );
}

#[test]
fn test_non_korea_snapshot_does_not_smuggle_korea_obligations() {
    let mut snapshot = snapshot_with_count(100);
    snapshot.jurisdiction = Jurisdiction::UnitedStates;

    assert!(
        evaluate_labor_compliance(snapshot)
            .expect("valid")
            .is_empty()
    );
}

fn snapshot_with_count(active_employee_count: u32) -> LegalEntityWorkforceSnapshot {
    LegalEntityWorkforceSnapshot {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        active_employee_count,
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
        evaluated_at_epoch_seconds: 1_779_519_600,
    }
}
