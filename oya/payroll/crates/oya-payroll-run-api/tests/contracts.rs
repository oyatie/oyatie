#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_api::{
    ApiErrorEnvelope, HrLeaveImpactIntakeRequest, HrLeaveImpactIntakeResponse,
    HrLeaveImpactKindDto, MoneyAmountDto, PayeeClassDto, PayeeRequest, PayrollJournalDraftRequest,
    PayrollJournalLineRequest, PayrollTrialCloseRequest, WageLedgerEntryRequest, WageLineKindDto,
};
use oya_payroll_run_app::{
    close_trial_run, prepare_accounting_dispatch, prepare_hr_leave_impact_intake,
};
use serde_json::json;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| candidate.join("specs/root-hub-pointers.json").is_file())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn trial_close_request_uses_camel_case_and_stable_enums() {
    let request = trial_close_request();
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(body["runId"], "prun_kr_2026_01");
    assert_eq!(body["payees"][0]["payeeClass"], "EMPLOYEE");
    assert_eq!(
        body["payees"][0]["wageLedger"][0]["lineKind"],
        "GROSS_EARNINGS"
    );

    let outcome = close_trial_run(request.into_domain()).expect("app accepts DTO request");
    assert_eq!(outcome.audit_envelope.run_id.value.value, "prun_kr_2026_01");
}

#[test]
fn hr_leave_impact_request_converts_to_domain_input() {
    let request = hr_leave_impact_request();
    let body = serde_json::to_value(&request).expect("serialize HR leave impact request");

    assert_eq!(body["runId"], "prun_kr_2026_06");
    assert_eq!(body["payrollPeriod"], "2026-06");
    assert_eq!(body["impactKind"], "UNPAID_LEAVE_DEDUCTION");
    assert_eq!(body["sourceTopic"], "integration.hr.payroll.leave-impact");
    assert_eq!(
        body["payrollImpactEvidenceRef"],
        "audit/hr/leave/leave_001/payroll-impact"
    );

    let outcome = prepare_hr_leave_impact_intake(request.into_domain())
        .expect("app accepts HR leave impact intake DTO");
    let response = HrLeaveImpactIntakeResponse::from_intake(&outcome.intake);
    let response_body = serde_json::to_value(response).expect("serialize HR leave response");

    assert_eq!(
        response_body["integrationTopic"],
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(response_body["payrollPeriod"], "2026-06");
    assert_eq!(response_body["impactKind"], "UNPAID_LEAVE_DEDUCTION");
    assert_eq!(response_body["payloadDataClass"], "FINANCIAL");
    assert_eq!(response_body["schemaVersion"], 1);
}

#[test]
fn payroll_journal_request_converts_to_accounting_dispatch_input() {
    let outcome = prepare_accounting_dispatch(journal_request().into_domain()).expect("dispatch");

    assert_eq!(
        outcome.dispatch_envelope.idempotency_key.value,
        "prun_kr_2026_01:jrn_payroll_2026_01:accounting-dispatch"
    );
}

#[test]
fn error_envelope_has_consistent_shape() {
    let envelope =
        ApiErrorEnvelope::validation("Invalid payroll request", Some("payees".to_owned()));

    assert_eq!(
        serde_json::to_value(envelope).expect("serialize error"),
        json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Invalid payroll request",
                "details": "payees"
            }
        })
    );
}

#[test]
fn openapi_contract_declares_auth_failures_for_money_mutations() {
    let contract_text =
        std::fs::read_to_string(repo_root().join("oya/payroll/contracts/openapi-v1.yaml"))
            .expect("read payroll OpenAPI contract");
    let contract: serde_json::Value =
        serde_json::from_str(&contract_text).expect("parse payroll OpenAPI contract");

    assert_eq!(
        contract["security"][0]["tenantBearer"],
        json!([]),
        "payroll contract must keep tenant bearer auth at the API boundary"
    );
    assert_eq!(
        contract["components"]["responses"]["AuthenticationError"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/ApiErrorEnvelope"
    );
    assert_eq!(
        contract["components"]["responses"]["AuthorizationError"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/ApiErrorEnvelope"
    );

    for (path, methods) in contract["paths"].as_object().expect("paths object").iter() {
        for (method, operation) in methods.as_object().expect("path methods").iter() {
            let responses = &operation["responses"];
            assert!(
                responses.get("401").is_some(),
                "{method} {path} must document unauthenticated failure"
            );
            assert!(
                responses.get("403").is_some(),
                "{method} {path} must document unauthorized failure"
            );
        }
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
            wage_ledger: vec![
                WageLedgerEntryRequest {
                    entry_id: "wage_001_gross".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKindDto::GrossEarnings,
                    amount: MoneyAmountDto {
                        amount_minor: 1_000_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/hr/time/001".to_owned(),
                },
                WageLedgerEntryRequest {
                    entry_id: "wage_001_net".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKindDto::NetPay,
                    amount: MoneyAmountDto {
                        amount_minor: -800_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/payroll/net/001".to_owned(),
                },
            ],
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
        received_at_epoch_seconds: 1_779_535_200,
    }
}
