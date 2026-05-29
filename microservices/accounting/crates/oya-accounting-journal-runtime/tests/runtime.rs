#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_accounting_journal_api::{
    JournalLineRequest, JournalPostRequest, JurisdictionDto, PayrollPostingRequest, PeriodStateDto,
    VatDeadlineRequest,
};
use oya_accounting_journal_runtime::{
    ACCOUNTING_HEALTH_PATH, ACCOUNTING_JOURNALS_PATH, ACCOUNTING_PAYROLL_POSTINGS_PATH,
    ACCOUNTING_VAT_WORKFLOW_PLANS_PATH, accounting_runtime_routes, accounting_server_config,
    dispatch_accounting_request,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;

#[test]
fn accounting_runtime_dispatches_journal_payroll_and_vat() {
    let journal = dispatch_accounting_request(mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &journal_request(),
    ));
    let journal_body: serde_json::Value =
        serde_json::from_slice(&journal.body).expect("journal json");
    assert_eq!(journal.status, 202);
    assert_eq!(journal_body["accepted"], true);
    assert_eq!(
        journal_body["auditTopic"],
        "audit.accounting.journal.posted"
    );
    assert_eq!(journal_body["service"], "accounting");

    let payroll = dispatch_accounting_request(mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_PAYROLL_POSTINGS_PATH,
        &payroll_request(),
    ));
    let payroll_body: serde_json::Value =
        serde_json::from_slice(&payroll.body).expect("payroll json");
    assert_eq!(payroll.status, 202);
    assert_eq!(
        payroll_body["auditTopic"],
        "audit.accounting.payroll.posted"
    );
    assert_eq!(
        payroll_body["idempotencyKey"],
        "ten_acme:jrn_payroll_2026_01:payroll-posted"
    );

    let vat = dispatch_accounting_request(mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_VAT_WORKFLOW_PLANS_PATH,
        &vat_request(),
    ));
    let vat_body: serde_json::Value = serde_json::from_slice(&vat.body).expect("vat json");
    assert_eq!(vat.status, 200);
    assert_eq!(vat_body["opened"], true);
    assert_eq!(vat_body["workflowRef"], "workflow/accounting/vat/kr");
    assert!(!vat_body["requiredSteps"].as_array().unwrap().is_empty());
}

#[test]
fn accounting_runtime_rejects_invalid_json_and_domain_errors() {
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: ACCOUNTING_JOURNALS_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_response = dispatch_accounting_request(invalid_json);
    let invalid_body: serde_json::Value =
        serde_json::from_slice(&invalid_response.body).expect("invalid json response");
    assert_eq!(invalid_response.status, 400);
    assert_eq!(invalid_body["error"]["code"], "VALIDATION_ERROR");

    let unbalanced = dispatch_accounting_request(mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &JournalPostRequest {
            lines: vec![JournalLineRequest {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: 1_000_000,
                credit_minor: 0,
            }],
            ..journal_request()
        },
    ));
    let unbalanced_body: serde_json::Value =
        serde_json::from_slice(&unbalanced.body).expect("domain error json");
    assert_eq!(unbalanced.status, 400);
    assert!(
        unbalanced_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("UnbalancedJournal")
    );
}

#[test]
fn accounting_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = accounting_runtime_routes();
    assert_eq!(routes.len(), 4);
    assert!(
        routes
            .iter()
            .any(|route| route.path == ACCOUNTING_VAT_WORKFLOW_PLANS_PATH)
    );

    let config = accounting_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    let health = dispatch_accounting_request(HttpRequest {
        method: HttpMethod::Get,
        path: ACCOUNTING_HEALTH_PATH.to_owned(),
        headers: BTreeMap::new(),
        body: Vec::new(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    });
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "router-ready");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["statutoryFilingRails"], false);
    assert_eq!(body["paymentExecution"], false);
    assert_eq!(body["payrollNetworkCall"], false);
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
    format!("sha256:{}", "b".repeat(64))
}

fn lines() -> Vec<JournalLineRequest> {
    vec![
        JournalLineRequest {
            account_code: "EXP-WAGES".to_owned(),
            debit_minor: 1_000_000,
            credit_minor: 0,
        },
        JournalLineRequest {
            account_code: "LIAB-NETPAY".to_owned(),
            debit_minor: 0,
            credit_minor: 1_000_000,
        },
    ]
}

fn journal_request() -> JournalPostRequest {
    JournalPostRequest {
        journal_id: "jrn_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        period_state: PeriodStateDto::Open,
        source_documents: vec!["src/payroll/run/prun_kr_2026_01".to_owned()],
        approval_evidence_ref: "audit/accounting/journal/approval".to_owned(),
        lines: lines(),
    }
}

fn payroll_request() -> PayrollPostingRequest {
    PayrollPostingRequest {
        journal_id: "jrn_payroll_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        wage_ledger_refs: vec!["audit/payroll/wage-ledger/001".to_owned()],
        approval_evidence_ref: "audit/accounting/payroll/approval".to_owned(),
        reversal_path_ref: "audit/accounting/payroll/reversal".to_owned(),
        lines: lines(),
    }
}

fn vat_request() -> VatDeadlineRequest {
    VatDeadlineRequest {
        return_id: "vat_2026_q1".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        period: "2026-01".to_owned(),
        deadline_epoch_seconds: 1_779_519_600,
        now_epoch_seconds: 1_779_519_601,
        workflow_ref: "workflow/accounting/vat/kr".to_owned(),
        hometax_export_hash: digest(),
        evidence_ref: "audit/accounting/vat/evidence".to_owned(),
    }
}
