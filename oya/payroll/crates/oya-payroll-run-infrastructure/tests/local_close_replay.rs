#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;
use oya_payroll_run_api::{
    HrLeaveImpactIntakeRequest, HrLeaveImpactKindDto, MoneyAmountDto, PayeeClassDto, PayeeRequest,
    PayrollJournalDraftRequest, PayrollJournalLineRequest, PayrollTrialCloseRequest,
    WageLedgerEntryRequest, WageLineKindDto,
};
use oya_payroll_run_infrastructure::{
    PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH, PAYROLL_HEALTH_PATH,
    PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH, PAYROLL_TRIAL_CLOSE_PATH, dispatch_payroll_request,
};
use serde_json::json;

const LIVE_DEPLOYMENT_NA_RATIONALE: &str = "N/A: no deployable listener or cloud target exists for this Payroll runtime adapter card; evidence is local in-process router replay only.";
const ACCESSIBILITY_NA_RATIONALE: &str =
    "N/A: backend/router-only replay has no Payroll UI surface or browser workflow in this card.";

#[test]
fn local_close_replay_returns_user_story_evidence_and_explicit_na_rationale() {
    let health = dispatch_payroll_request(HttpRequest {
        method: HttpMethod::Get,
        path: PAYROLL_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let health_body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(health_body["localReplayOnly"], true);
    assert_eq!(
        health_body["liveDeploymentStatus"],
        "not-deployed-local-router-only"
    );
    assert_eq!(
        health_body["liveDeploymentRationale"],
        LIVE_DEPLOYMENT_NA_RATIONALE
    );
    assert_eq!(
        health_body["accessibilityEvidenceStatus"],
        ACCESSIBILITY_NA_RATIONALE
    );
    assert_eq!(health_body["productionCloseController"], false);
    assert_eq!(health_body["deployedListener"], false);
    assert_eq!(health_body["storageAttached"], false);
    assert_eq!(health_body["workflowDispatch"], false);
    assert_eq!(health_body["statutoryFilingRails"], false);

    let trial = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_TRIAL_CLOSE_PATH,
        &trial_close_request(),
    ));
    let trial_body: serde_json::Value = serde_json::from_slice(&trial.body).expect("trial json");
    assert_eq!(trial.status, 202);
    assert_eq!(trial_body["runId"], "prun_kr_2026_01");
    assert_eq!(trial_body["storyStage"], "payroll_trial_close");
    assert_eq!(
        trial_body["evidenceRefs"],
        json!(["audit/payroll/trial-close/approval"])
    );
    assert_eq!(trial_body["sourceDigest"], digest());
    assert_common_non_claims(&trial_body);

    let hr = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH,
        &hr_leave_impact_request(),
    ));
    let hr_body: serde_json::Value = serde_json::from_slice(&hr.body).expect("hr json");
    assert_eq!(hr.status, 202);
    assert_eq!(hr_body["storyStage"], "hr_leave_impact_intake");
    assert_eq!(
        hr_body["sourceHrIdempotencyKey"],
        "ten_acme:leave_001:Approved:2026-06"
    );
    assert_eq!(
        hr_body["evidenceRefs"],
        json!([
            "audit/hr/leave/leave_001/decision",
            "audit/hr/leave/leave_001/escalation",
            "audit/hr/leave/leave_001/payroll-impact",
            "audit/payroll/hr-leave/leave_001/intake"
        ])
    );
    assert_common_non_claims(&hr_body);

    let journal = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH,
        &journal_request(),
    ));
    let journal_body: serde_json::Value =
        serde_json::from_slice(&journal.body).expect("journal json");
    assert_eq!(journal.status, 202);
    assert_eq!(journal_body["storyStage"], "accounting_journal_draft");
    assert_eq!(journal_body["runId"], "prun_kr_2026_01");
    assert_eq!(
        journal_body["evidenceRefs"],
        json!([
            "audit/payroll/approval/cfo",
            "audit/le_kr_001/payroll/prun_kr_2026_01/reversal"
        ])
    );
    assert_eq!(journal_body["sourceDigest"], digest());
    assert_common_non_claims(&journal_body);
}

#[test]
fn local_close_replay_rejects_rollback_error_path_without_overclaiming_runtime_side_effects() {
    let mut request = trial_close_request();
    request.approval_evidence_ref = "payroll/approval/missing-audit-prefix".to_owned();

    let response = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_TRIAL_CLOSE_PATH,
        &request,
    ));
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("error json");

    assert_eq!(response.status, 400);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        body["error"]["details"]
            .as_str()
            .expect("error details")
            .contains("InvalidEvidenceRef"),
        "invalid close evidence must fail closed before any local replay can imply rollback side effects: {body}"
    );

    let health = dispatch_payroll_request(HttpRequest {
        method: HttpMethod::Get,
        path: PAYROLL_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let health_body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health_body["rollbackObservability"], "metadata-only");
    assert_eq!(health_body["productionCloseController"], false);
    assert_eq!(health_body["workflowDispatch"], false);
    assert_eq!(health_body["storageAttached"], false);
    assert_eq!(
        health_body["liveDeploymentRationale"],
        LIVE_DEPLOYMENT_NA_RATIONALE
    );
}

fn assert_common_non_claims(body: &serde_json::Value) {
    assert_eq!(body["localReplayOnly"], true);
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowDispatch"], false);
    assert_eq!(body["runtimeAuditEmission"], false);
    assert_eq!(body["externalHrCall"], false);
    assert_eq!(body["externalAccountingCall"], false);
    assert_eq!(body["statutoryFilingRails"], false);
    assert_eq!(body["disbursementRails"], false);
    assert_eq!(body["productionClose"], false);
    assert_eq!(
        body["liveDeploymentRationale"],
        LIVE_DEPLOYMENT_NA_RATIONALE
    );
}

fn mock_json_request<T: serde::Serialize>(
    method: HttpMethod,
    path: &str,
    payload: &T,
) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_owned(),
        headers: BTreeMap::from([("content-type".to_owned(), "application/json".to_owned())]),
        body: serde_json::to_vec(payload).expect("serialize request"),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

fn digest() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn trial_close_request() -> PayrollTrialCloseRequest {
    PayrollTrialCloseRequest {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        evidence_digest: digest(),
        approval_evidence_ref: "audit/payroll/trial-close/approval".to_owned(),
        payees: vec![PayeeRequest {
            payee_id: "payee_001".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            payee_class: PayeeClassDto::Employee,
            person_or_vendor_ref: "person/acme/001".to_owned(),
            tax_profile_ref: "tax/kr/employee/001".to_owned(),
            wage_ledger: vec![WageLedgerEntryRequest {
                entry_id: "wage_001_gross".to_owned(),
                payee_id: "payee_001".to_owned(),
                line_kind: WageLineKindDto::GrossEarnings,
                amount: MoneyAmountDto {
                    amount_minor: 1_000_000,
                    currency: "KRW".to_owned(),
                },
                source_ref: "audit/hr/time/001".to_owned(),
            }],
        }],
    }
}

fn hr_leave_impact_request() -> HrLeaveImpactIntakeRequest {
    HrLeaveImpactIntakeRequest {
        run_id: "prun_kr_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payee_id: "payee_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        leave_request_id: "leave_001".to_owned(),
        impact_kind: HrLeaveImpactKindDto::UnpaidLeaveDeduction,
        source_topic: "integration.hr.payroll.leave-impact".to_owned(),
        source_hr_idempotency_key: "ten_acme:leave_001:Approved:2026-06".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        received_at_epoch_seconds: 1_779_535_800,
    }
}

fn journal_request() -> PayrollJournalDraftRequest {
    PayrollJournalDraftRequest {
        journal_id: "jrn_payroll_2026_01".to_owned(),
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        approval_evidence_ref: "audit/payroll/approval/cfo".to_owned(),
        lines: vec![
            PayrollJournalLineRequest {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: 1_000_000,
                credit_minor: 0,
            },
            PayrollJournalLineRequest {
                account_code: "LIAB-NETPAY".to_owned(),
                debit_minor: 0,
                credit_minor: 1_000_000,
            },
        ],
    }
}
