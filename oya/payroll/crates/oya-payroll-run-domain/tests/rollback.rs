#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

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

#[test]
fn failed_close_stops_promotion_and_emits_local_rollback_observability_metadata() {
    let decision = evaluate_close_promotion(CloseHealthInput {
        run_id: "prun_kr_2026_01".to_owned(),
        canary_passed: false,
        evidence_gate_passed: true,
        rollback_evidence_ref: Some("audit/payroll/rollback/prun_kr_2026_01".to_owned()),
        quarantine_evidence_ref: Some("audit/payroll/quarantine/prun_kr_2026_01".to_owned()),
        repair_route: Some(RepairRoute::HotfixPullRequest),
    })
    .expect("rollback metadata decision");

    let ClosePromotionDecision::RollbackFirst {
        rollback_evidence_ref,
        quarantine_evidence_ref,
        repair_route,
        promotion_stopped,
        route_metadata,
        observability_attributes,
    } = decision
    else {
        panic!("failed health must refuse promotion with rollback metadata");
    };

    assert!(
        promotion_stopped,
        "failed close must explicitly stop promotion"
    );
    assert_eq!(
        rollback_evidence_ref.value,
        "audit/payroll/rollback/prun_kr_2026_01"
    );
    assert_eq!(
        quarantine_evidence_ref.value,
        "audit/payroll/quarantine/prun_kr_2026_01"
    );
    assert_eq!(repair_route, RepairRoute::HotfixPullRequest);
    assert_eq!(route_metadata.route_label, "hotfix_pr");
    assert!(route_metadata.hotfix_pr_required);
    assert!(!route_metadata.opentofu_ops_convergence_required);
    assert!(!route_metadata.production_deploy_attached);
    assert!(!route_metadata.workflow_execution_attached);
    assert!(!route_metadata.opentofu_execution_attached);

    let attributes = attributes_by_key(observability_attributes);
    assert_eq!(attributes["service.name"], "payroll");
    assert_eq!(attributes["payroll.run_id"], "prun_kr_2026_01");
    assert_eq!(attributes["payroll.close.promotion_allowed"], "false");
    assert_eq!(
        attributes["payroll.close.stop_reason"],
        "close_health_gate_failed"
    );
    assert_eq!(
        attributes["payroll.close.rollback_evidence_ref"],
        "audit/payroll/rollback/prun_kr_2026_01"
    );
    assert_eq!(
        attributes["payroll.close.quarantine_evidence_ref"],
        "audit/payroll/quarantine/prun_kr_2026_01"
    );
    assert_eq!(attributes["payroll.close.repair_route"], "hotfix_pr");
    assert_eq!(
        attributes["payroll.close.production_deploy_attached"],
        "false"
    );
    assert_eq!(
        attributes["payroll.close.workflow_execution_attached"],
        "false"
    );
}

#[test]
fn evidence_gate_failure_routes_through_opentofu_without_claiming_execution() {
    let decision = evaluate_close_promotion(CloseHealthInput {
        run_id: "prun_kr_2026_02".to_owned(),
        canary_passed: true,
        evidence_gate_passed: false,
        rollback_evidence_ref: Some("audit/payroll/rollback/prun_kr_2026_02".to_owned()),
        quarantine_evidence_ref: Some("audit/payroll/quarantine/prun_kr_2026_02".to_owned()),
        repair_route: Some(RepairRoute::OpenTofuOpsConvergence),
    })
    .expect("opentofu rollback route metadata");

    let ClosePromotionDecision::RollbackFirst {
        promotion_stopped,
        route_metadata,
        observability_attributes,
        ..
    } = decision
    else {
        panic!("evidence gate failure must refuse promotion");
    };

    assert!(promotion_stopped);
    assert_eq!(route_metadata.route_label, "opentofu_ops_convergence");
    assert!(!route_metadata.hotfix_pr_required);
    assert!(route_metadata.opentofu_ops_convergence_required);
    assert!(!route_metadata.production_deploy_attached);
    assert!(!route_metadata.workflow_execution_attached);
    assert!(!route_metadata.opentofu_execution_attached);

    let attributes = attributes_by_key(observability_attributes);
    assert_eq!(
        attributes["payroll.close.repair_route"],
        "opentofu_ops_convergence"
    );
    assert_eq!(
        attributes["payroll.close.opentofu_execution_attached"],
        "false"
    );
}

#[test]
fn failed_close_requires_quarantine_evidence_before_refusing_promotion() {
    assert_eq!(
        evaluate_close_promotion(CloseHealthInput {
            run_id: "prun_kr_2026_03".to_owned(),
            canary_passed: false,
            evidence_gate_passed: true,
            rollback_evidence_ref: Some("audit/payroll/rollback/prun_kr_2026_03".to_owned()),
            quarantine_evidence_ref: None,
            repair_route: Some(RepairRoute::HotfixPullRequest),
        }),
        Err(PayrollDomainError::RollbackEvidenceRequired)
    );
}

fn attributes_by_key(
    attributes: Vec<oya_payroll_run_domain::CloseHealthObservabilityAttribute>,
) -> BTreeMap<String, String> {
    attributes
        .into_iter()
        .map(|attribute| (attribute.key, attribute.value))
        .collect()
}
