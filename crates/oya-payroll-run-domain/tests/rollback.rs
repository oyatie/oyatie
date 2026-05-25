#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    CloseHealthInput, ClosePromotionDecision, PayrollDomainError, RepairRoute,
    evaluate_close_promotion,
};

#[test]
fn test_failed_close_uses_rollback_first() {
    let decision = evaluate_close_promotion(CloseHealthInput {
        run_id: "prun_kr_2026_01".to_owned(),
        canary_passed: false,
        evidence_gate_passed: true,
        rollback_evidence_ref: Some("audit/payroll/rollback/evidence".to_owned()),
        quarantine_evidence_ref: Some("audit/payroll/quarantine/evidence".to_owned()),
        repair_route: Some(RepairRoute::HotfixPullRequest),
    })
    .expect("rollback decision");

    assert!(matches!(
        decision,
        ClosePromotionDecision::RollbackFirst {
            repair_route: RepairRoute::HotfixPullRequest,
            ..
        }
    ));

    assert_eq!(
        evaluate_close_promotion(CloseHealthInput {
            run_id: "prun_kr_2026_01".to_owned(),
            canary_passed: false,
            evidence_gate_passed: false,
            rollback_evidence_ref: None,
            quarantine_evidence_ref: None,
            repair_route: None,
        }),
        Err(PayrollDomainError::RollbackEvidenceRequired)
    );

    assert_eq!(
        evaluate_close_promotion(CloseHealthInput {
            run_id: "prun_kr_2026_01".to_owned(),
            canary_passed: true,
            evidence_gate_passed: true,
            rollback_evidence_ref: None,
            quarantine_evidence_ref: None,
            repair_route: None,
        }),
        Ok(ClosePromotionDecision::Promote)
    );
}
