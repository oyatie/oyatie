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
    PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH, PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH,
    PAYROLL_TRIAL_CLOSE_PATH, PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH, dispatch_payroll_request,
    payroll_runtime_routes, payroll_server_config,
};

#[test]
fn payroll_runtime_dispatches_hr_leave_intake() {
    let request = mock_json_request(
        HttpMethod::Post,
        PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH,
        &hr_leave_impact_request(),
    );

    let response = dispatch_payroll_request(request);
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json response");

    assert_eq!(response.status, 202);
    assert_eq!(
        body["integrationTopic"],
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(body["payrollPeriod"], "2026-06");
    assert_eq!(body["impactKind"], "UNPAID_LEAVE_DEDUCTION");
    assert_eq!(body["payloadDataClass"], "FINANCIAL");
}

#[test]
fn payroll_runtime_dispatches_trial_close_and_journal_metadata() {
    let trial = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_TRIAL_CLOSE_PATH,
        &trial_close_request(),
    ));
    let trial_body: serde_json::Value = serde_json::from_slice(&trial.body).expect("trial json");
    assert_eq!(trial.status, 202);
    assert_eq!(trial_body["accepted"], true);
    assert_eq!(trial_body["auditTopic"], "audit.payroll.run.close");
    assert_eq!(trial_body["service"], "payroll");

    let journal = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_ACCOUNTING_JOURNAL_DRAFT_PATH,
        &journal_request(),
    ));
    let journal_body: serde_json::Value =
        serde_json::from_slice(&journal.body).expect("journal json");
    assert_eq!(journal.status, 202);
    assert_eq!(
        journal_body["auditTopic"],
        "tenant_rbac.payroll.accounting.journal_draft"
    );
    assert_eq!(
        journal_body["idempotencyKey"],
        "prun_kr_2026_01:jrn_payroll_2026_01:accounting-dispatch"
    );
}

#[test]
fn payroll_runtime_rejects_invalid_json_and_domain_errors_without_panicking() {
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_json_response = dispatch_payroll_request(invalid_json);
    let body: serde_json::Value =
        serde_json::from_slice(&invalid_json_response.body).expect("error json");
    assert_eq!(invalid_json_response.status, 400);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    let domain_error = dispatch_payroll_request(mock_json_request(
        HttpMethod::Post,
        PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH,
        &HrLeaveImpactIntakeRequest {
            source_topic: "integration.hr.payroll.raw-leave".to_owned(),
            ..hr_leave_impact_request()
        },
    ));
    let domain_body: serde_json::Value =
        serde_json::from_slice(&domain_error.body).expect("domain error json");
    assert_eq!(domain_error.status, 400);
    assert_eq!(domain_body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        domain_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("InvalidHrLeaveImpactTopic")
    );
}

#[test]
fn payroll_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = payroll_runtime_routes();
    assert!(
        routes
            .iter()
            .any(|route| route.path == PAYROLL_HR_LEAVE_IMPACT_INTAKE_PATH)
    );
    assert!(
        routes
            .iter()
            .any(|route| route.path == PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH)
    );
    assert!(
        routes
            .iter()
            .any(|route| route.path == PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH)
    );
    assert!(routes.iter().any(|route| route.path == PAYROLL_HEALTH_PATH));

    let config = payroll_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    let health = dispatch_payroll_request(HttpRequest {
        method: HttpMethod::Get,
        path: PAYROLL_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "router-ready");
    assert_eq!(body["closeHealthGate"], "domain-local-only");
    assert_eq!(body["rollbackObservability"], "metadata-only");
    assert_eq!(body["productionCloseController"], false);
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowDispatch"], false);
    assert_eq!(body["opentofuOpsConvergence"], false);
    assert_eq!(body["statutoryFilingRails"], false);
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
