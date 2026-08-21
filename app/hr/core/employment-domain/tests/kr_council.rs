#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hr_employment_domain::{
    Jurisdiction, LaborComplianceObligationKind, LaborComplianceWorkflowStep,
    LegalEntityWorkforceSnapshot, evaluate_labor_compliance,
};

#[test]
fn test_labor_management_council_threshold() {
    let obligations = evaluate_labor_compliance(snapshot_with_count(30)).expect("obligations");

    assert_eq!(obligations.len(), 2);
    let council = obligations
        .iter()
        .find(|obligation| {
            obligation.kind.value == LaborComplianceObligationKind::KoreaLaborManagementCouncil
        })
        .expect("council obligation");
    assert_eq!(council.threshold_employee_count.value, 30);
    assert!(
        council
            .workflow_steps
            .value
            .contains(&LaborComplianceWorkflowStep::CouncilRosterRequired)
    );
    assert!(
        council
            .workflow_steps
            .value
            .contains(&LaborComplianceWorkflowStep::MeetingCadenceRequired)
    );
    assert!(
        council
            .workflow_steps
            .value
            .contains(&LaborComplianceWorkflowStep::MinutesEvidenceRequired)
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
