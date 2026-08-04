#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;
use oya_payroll_run_api::{
    MoneyAmountDto, PayrollRulepackJurisdictionDto, STATUTORY_PREVIEW_FIXTURE_NOTE,
    StatutoryCalculationPreviewRequest, StatutoryDeductionKindDto, StatutoryRateLineRequest,
    YEAR_END_PREVIEW_FIXTURE_NOTE, YearEndEmployeeInputRequest, YearEndEvidenceRefRequest,
    YearEndRegionalDependencyRequest, YearEndSettlementPreviewRequest,
    YearEndSettlementSourceKindDto,
};
use oya_payroll_run_infrastructure::{
    PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH, PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH,
    dispatch_payroll_request,
};

#[test]
fn payroll_runtime_replays_statutory_calculation_preview_without_external_rails() {
    let response = dispatch_payroll_request(mock_json_request(
        PAYROLL_STATUTORY_CALCULATION_PREVIEW_PATH,
        &statutory_calculation_preview_request(),
    ));
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json response");

    assert_eq!(response.status, 202);
    assert_eq!(body["runId"], "prun_kr_2026_01");
    assert_eq!(body["boundary"], "PURE_DOMAIN_NO_FILING_TRANSPORT");
    assert_eq!(body["payloadDataClass"], "FINANCIAL");
    assert_eq!(body["deductions"].as_array().expect("deductions").len(), 2);
    assert_eq!(body["directAgencySubmissionAttached"], false);
    assert_eq!(body["filingRailAttached"], false);
    assert_eq!(body["disbursementRailAttached"], false);
    assert_eq!(body["productionCloseAttached"], false);
    assert_eq!(body["cloudDeploymentAttached"], false);
}

#[test]
fn payroll_runtime_replays_year_end_settlement_preview_without_external_rails() {
    let response = dispatch_payroll_request(mock_json_request(
        PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH,
        &year_end_settlement_preview_request(),
    ));
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json response");

    assert_eq!(response.status, 202);
    assert_eq!(body["runId"], "prun_kr_2026_12");
    assert_eq!(body["evidenceRefCount"], 3);
    assert_eq!(body["regionalDependencyCount"], 1);
    assert_eq!(body["employeeInputCount"], 1);
    assert_eq!(body["payloadDataClass"], "PII_IDENTIFYING+FINANCIAL");
    assert_eq!(body["directAgencySubmissionAttached"], false);
    assert_eq!(body["filingRailAttached"], false);
    assert_eq!(body["disbursementRailAttached"], false);
    assert_eq!(body["productionCloseAttached"], false);
    assert_eq!(body["cloudDeploymentAttached"], false);
}

#[test]
fn payroll_runtime_rejects_year_end_employee_money_currency_mismatch() {
    let mut request = year_end_settlement_preview_request();
    request.employee_inputs[0].withholding.currency = "USD".to_owned();

    let response = dispatch_payroll_request(mock_json_request(
        PAYROLL_YEAR_END_SETTLEMENT_PREVIEW_PATH,
        &request,
    ));
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json response");

    assert_eq!(response.status, 400);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        body["error"]["details"]
            .as_str()
            .expect("error details")
            .contains("InvalidMoney"),
        "expected InvalidMoney for mismatched gross/withholding currencies, got {body}"
    );
}

fn mock_json_request<T: serde::Serialize>(path: &str, payload: &T) -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Post,
        path: path.to_owned(),
        headers: BTreeMap::from([("content-type".to_owned(), "application/json".to_owned())]),
        body: serde_json::to_vec(payload).expect("serialize request"),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

fn statutory_calculation_preview_request() -> StatutoryCalculationPreviewRequest {
    StatutoryCalculationPreviewRequest {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payee_id: "payee_001".to_owned(),
        payroll_period: "2026-01".to_owned(),
        jurisdiction: PayrollRulepackJurisdictionDto::Korea,
        required_regional_pack: "KR".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-synthetic-2026".to_owned(),
        rulepack_manifest_ref: Some("rulepack-source/kr/synthetic/manifest/2026".to_owned()),
        rulepack_source_version: Some("synthetic-kr-stat-2026-v0".to_owned()),
        official_source_evidence_refs: vec![
            "audit/payroll/rulepack/kr/synthetic-source-version".to_owned(),
        ],
        unofficial_source_fixture: false,
        gross_pay: MoneyAmountDto {
            amount_minor: 3_000_000,
            currency: "KRW".to_owned(),
        },
        rate_lines: vec![
            StatutoryRateLineRequest {
                kind: StatutoryDeductionKindDto::IncomeTax,
                synthetic_rate_basis_points: 300,
                source_evidence_ref: "audit/payroll/rulepack/kr/synthetic-income-tax".to_owned(),
            },
            StatutoryRateLineRequest {
                kind: StatutoryDeductionKindDto::SocialInsurance,
                synthetic_rate_basis_points: 450,
                source_evidence_ref: "audit/payroll/rulepack/kr/synthetic-social-insurance"
                    .to_owned(),
            },
        ],
        fixture_note: STATUTORY_PREVIEW_FIXTURE_NOTE.to_owned(),
        filing_rail_requested: false,
        disbursement_rail_requested: false,
        production_close_requested: false,
        cloud_deployment_requested: false,
    }
}

fn year_end_settlement_preview_request() -> YearEndSettlementPreviewRequest {
    YearEndSettlementPreviewRequest {
        run_id: "prun_kr_2026_12".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_year: 2026,
        jurisdiction: PayrollRulepackJurisdictionDto::Korea,
        rulepack_ref: "rulepack/kr-year-end-synthetic-2026".to_owned(),
        rulepack_manifest_ref: Some("rulepack-source/kr/year-end/synthetic/2026".to_owned()),
        source_version: Some("synthetic-kr-year-end-2026-v0".to_owned()),
        evidence_refs: vec![
            YearEndEvidenceRefRequest {
                source_kind: YearEndSettlementSourceKindDto::WageLedgerDigest,
                ref_value: "audit/payroll/year-end/wage-ledger-digest".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
            YearEndEvidenceRefRequest {
                source_kind: YearEndSettlementSourceKindDto::WithholdingEvidence,
                ref_value: "audit/payroll/year-end/withholding-evidence".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
            YearEndEvidenceRefRequest {
                source_kind: YearEndSettlementSourceKindDto::EmployeeDeclaration,
                ref_value: "audit/payroll/year-end/employee-declaration".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
        ],
        regional_dependencies: vec![YearEndRegionalDependencyRequest {
            pack_code: "KR".to_owned(),
            source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            evidence_ref: "audit/payroll/year-end/regional-pack/kr".to_owned(),
        }],
        employee_inputs: vec![YearEndEmployeeInputRequest {
            payee_id: "payee_001".to_owned(),
            employee_ref: "person/acme/001".to_owned(),
            gross_pay: MoneyAmountDto {
                amount_minor: 36_000_000,
                currency: "KRW".to_owned(),
            },
            withholding: MoneyAmountDto {
                amount_minor: 2_700_000,
                currency: "KRW".to_owned(),
            },
            wage_ledger_evidence_ref: "audit/payroll/year-end/payee-001/wage-ledger".to_owned(),
            declaration_evidence_ref: "audit/payroll/year-end/payee-001/declaration".to_owned(),
        }],
        fixture_note: YEAR_END_PREVIEW_FIXTURE_NOTE.to_owned(),
        unofficial_source_fixture: false,
        direct_agency_submission_requested: false,
        filing_rail_requested: false,
        disbursement_rail_requested: false,
        production_close_requested: false,
        cloud_deployment_requested: false,
    }
}
