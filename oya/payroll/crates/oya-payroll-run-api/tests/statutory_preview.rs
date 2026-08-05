#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_api::{
    MoneyAmountDto, PayrollRulepackJurisdictionDto, STATUTORY_PREVIEW_FIXTURE_NOTE,
    StatutoryCalculationPreviewRequest, StatutoryCalculationPreviewResponse,
    StatutoryDeductionKindDto, StatutoryRateLineRequest, YEAR_END_PREVIEW_FIXTURE_NOTE,
    YearEndEmployeeInputRequest, YearEndEvidenceRefRequest, YearEndRegionalDependencyRequest,
    YearEndSettlementPreviewRequest, YearEndSettlementPreviewResponse,
    YearEndSettlementSourceKindDto,
};
use oya_payroll_run_app::{
    PayrollAppError, prepare_statutory_calculation_preview, prepare_year_end_settlement_preview,
};
use oya_payroll_run_domain::PayrollDomainError;

#[test]
fn statutory_calculation_preview_request_converts_to_local_domain_preview_response() {
    let request = statutory_calculation_preview_request();
    let body = serde_json::to_value(&request).expect("serialize statutory preview request");

    assert_eq!(body["jurisdiction"], "KOREA");
    assert_eq!(body["grossPay"]["amountMinor"], 3_000_000);
    assert_eq!(body["rateLines"][0]["kind"], "INCOME_TAX");
    assert_eq!(body["fixtureNote"], STATUTORY_PREVIEW_FIXTURE_NOTE);

    let outcome = prepare_statutory_calculation_preview(request.into_domain())
        .expect("app accepts statutory preview DTO");
    let response = StatutoryCalculationPreviewResponse::from_draft(&outcome.draft);
    let response_body = serde_json::to_value(response).expect("serialize statutory response");

    assert_eq!(response_body["runId"], "prun_kr_2026_01");
    assert_eq!(response_body["payloadDataClass"], "FINANCIAL");
    assert_eq!(response_body["boundary"], "PURE_DOMAIN_NO_FILING_TRANSPORT");
    assert_eq!(response_body["filingRailAttached"], false);
    assert_eq!(response_body["disbursementRailAttached"], false);
    assert_eq!(response_body["productionCloseAttached"], false);
    assert_eq!(response_body["cloudDeploymentAttached"], false);
}

#[test]
fn year_end_settlement_preview_request_converts_to_local_preparation_response() {
    let request = year_end_settlement_preview_request();
    let body = serde_json::to_value(&request).expect("serialize year-end preview request");

    assert_eq!(body["jurisdiction"], "KOREA");
    assert_eq!(body["payrollYear"], 2026);
    assert_eq!(body["evidenceRefs"][0]["sourceKind"], "WAGE_LEDGER_DIGEST");
    assert_eq!(body["fixtureNote"], YEAR_END_PREVIEW_FIXTURE_NOTE);

    let outcome = prepare_year_end_settlement_preview(request.into_domain())
        .expect("app accepts year-end preview DTO");
    let response = YearEndSettlementPreviewResponse::from_prepared(&outcome.prepared);
    let response_body = serde_json::to_value(response).expect("serialize year-end response");

    assert_eq!(response_body["runId"], "prun_kr_2026_12");
    assert_eq!(response_body["evidenceRefCount"], 3);
    assert_eq!(response_body["regionalDependencyCount"], 1);
    assert_eq!(response_body["employeeInputCount"], 1);
    assert_eq!(
        response_body["payloadDataClass"],
        "PII_IDENTIFYING+FINANCIAL"
    );
    assert_eq!(response_body["directAgencySubmissionAttached"], false);
    assert_eq!(response_body["filingRailAttached"], false);
    assert_eq!(response_body["disbursementRailAttached"], false);
    assert_eq!(response_body["productionCloseAttached"], false);
    assert_eq!(response_body["cloudDeploymentAttached"], false);
}

#[test]
fn year_end_settlement_preview_rejects_mismatched_employee_money_currencies() {
    let mut request = year_end_settlement_preview_request();
    request.employee_inputs[0].withholding.currency = "USD".to_owned();

    assert_eq!(
        prepare_year_end_settlement_preview(request.into_domain()),
        Err(PayrollAppError::Domain(PayrollDomainError::InvalidMoney)),
        "year-end DTO conversion must not silently discard withholding.currency when it differs from grossPay.currency"
    );
}

pub fn statutory_calculation_preview_request() -> StatutoryCalculationPreviewRequest {
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

pub fn year_end_settlement_preview_request() -> YearEndSettlementPreviewRequest {
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
