#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    CalculationBoundary, PayrollDomainError, PayrollRulepackJurisdiction,
    StatutoryCalculationInput, StatutoryDeductionKind, StatutoryRateLineInput,
    calculate_statutory_deductions,
};

const SYNTHETIC_FIXTURE_NOTE: &str =
    "synthetic/non-authoritative fixture: no official KR/US/EU rate correctness claim";

#[test]
fn statutory_calculation_requires_source_versioned_rulepack_before_any_deduction() {
    let input = synthetic_kr_calculation_input_without_rulepack_manifest();

    assert_eq!(input.fixture_note, SYNTHETIC_FIXTURE_NOTE);
    assert_eq!(
        calculate_statutory_deductions(input),
        Err(PayrollDomainError::StatutoryRulepackManifestRequired),
        "statutory calculation must fail closed until a source-versioned official rulepack manifest is attached"
    );
}

#[test]
fn statutory_calculation_rejects_unofficial_or_unversioned_source_evidence() {
    let mut input = synthetic_us_calculation_input_with_rulepack_manifest();
    input.rulepack_source_version = Some("synthetic-only".to_owned());
    input.official_source_evidence_refs.clear();
    input.unofficial_source_fixture = true;

    assert_eq!(input.fixture_note, SYNTHETIC_FIXTURE_NOTE);
    assert_eq!(
        calculate_statutory_deductions(input),
        Err(PayrollDomainError::OfficialRulepackSourceEvidenceRequired),
        "synthetic fixtures are allowed only as RED fixtures, never as official tax-rate evidence"
    );
}

#[test]
fn statutory_calculation_result_schema_preserves_non_filing_boundaries() {
    let result =
        calculate_statutory_deductions(synthetic_kr_calculation_input_with_rulepack_manifest())
            .expect("future build should calculate against the synthetic schema fixture");

    assert_eq!(result.run_id.value, "prun_kr_2026_01");
    assert_eq!(result.payee_id.value, "payee_001");
    assert_eq!(
        result.rulepack_ref.value,
        "rulepack/kr-payroll-synthetic-2026"
    );
    assert_eq!(
        result.rulepack_source_version.value,
        "synthetic-kr-stat-2026-v0"
    );
    assert_eq!(result.deductions.value.len(), 2);
    assert!(result.deductions.value.iter().all(|line| {
        matches!(
            line.kind.value,
            StatutoryDeductionKind::IncomeTax | StatutoryDeductionKind::SocialInsurance
        ) && line.amount.value.currency == "KRW"
            && line
                .source_evidence_ref
                .value
                .value
                .starts_with("audit/payroll/rulepack/")
    }));
    assert_eq!(result.gross_pay.value.amount_minor, 3_000_000);
    assert!(
        result.net_pay.value.amount_minor < result.gross_pay.value.amount_minor,
        "future schema should expose a synthetic net-pay result without asserting official rate correctness"
    );
    assert_eq!(
        result.boundary.value,
        CalculationBoundary::PureDomainNoFilingTransport
    );
    assert!(!result.direct_agency_submission_attached.value);
    assert!(!result.filing_rail_attached.value);
    assert!(!result.disbursement_rail_attached.value);
    assert!(!result.production_close_attached.value);
    assert!(!result.cloud_deployment_attached.value);
}

#[test]
fn statutory_calculation_requires_regional_pack_dependency_for_requested_jurisdiction() {
    let mut input = synthetic_kr_calculation_input_with_rulepack_manifest();
    input.required_regional_pack = "EU".to_owned();
    input.jurisdiction = PayrollRulepackJurisdiction::Korea;

    assert_eq!(
        calculate_statutory_deductions(input),
        Err(PayrollDomainError::StatutoryRegionalPackRequired),
        "requested jurisdiction and regional pack dependency must match before calculation runs"
    );
}

fn synthetic_kr_calculation_input_without_rulepack_manifest() -> StatutoryCalculationInput {
    let mut input = synthetic_kr_calculation_input_with_rulepack_manifest();
    input.rulepack_manifest_ref = None;
    input.rulepack_source_version = None;
    input.official_source_evidence_refs.clear();
    input
}

fn synthetic_kr_calculation_input_with_rulepack_manifest() -> StatutoryCalculationInput {
    StatutoryCalculationInput {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payee_id: "payee_001".to_owned(),
        payroll_period: "2026-01".to_owned(),
        jurisdiction: PayrollRulepackJurisdiction::Korea,
        required_regional_pack: "KR".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-synthetic-2026".to_owned(),
        rulepack_manifest_ref: Some("rulepack-source/kr/synthetic/manifest/2026".to_owned()),
        rulepack_source_version: Some("synthetic-kr-stat-2026-v0".to_owned()),
        official_source_evidence_refs: vec![
            "audit/payroll/rulepack/kr/synthetic-source-version".to_owned(),
        ],
        unofficial_source_fixture: false,
        gross_pay_minor: 3_000_000,
        currency: "KRW".to_owned(),
        rate_lines: vec![
            StatutoryRateLineInput {
                kind: StatutoryDeductionKind::IncomeTax,
                synthetic_rate_basis_points: 300,
                source_evidence_ref: "audit/payroll/rulepack/kr/synthetic-income-tax".to_owned(),
            },
            StatutoryRateLineInput {
                kind: StatutoryDeductionKind::SocialInsurance,
                synthetic_rate_basis_points: 450,
                source_evidence_ref: "audit/payroll/rulepack/kr/synthetic-social-insurance"
                    .to_owned(),
            },
        ],
        fixture_note: SYNTHETIC_FIXTURE_NOTE,
        filing_rail_requested: false,
        disbursement_rail_requested: false,
        production_close_requested: false,
        cloud_deployment_requested: false,
    }
}

fn synthetic_us_calculation_input_with_rulepack_manifest() -> StatutoryCalculationInput {
    StatutoryCalculationInput {
        jurisdiction: PayrollRulepackJurisdiction::UnitedStatesFederal,
        required_regional_pack: "US".to_owned(),
        rulepack_ref: "rulepack/us-payroll-synthetic-2026".to_owned(),
        rulepack_manifest_ref: Some("rulepack-source/us/synthetic/manifest/2026".to_owned()),
        rulepack_source_version: Some("synthetic-us-stat-2026-v0".to_owned()),
        official_source_evidence_refs: vec![
            "audit/payroll/rulepack/us/synthetic-source-version".to_owned(),
        ],
        currency: "USD".to_owned(),
        rate_lines: vec![StatutoryRateLineInput {
            kind: StatutoryDeductionKind::IncomeTax,
            synthetic_rate_basis_points: 500,
            source_evidence_ref: "audit/payroll/rulepack/us/synthetic-income-tax".to_owned(),
        }],
        ..synthetic_kr_calculation_input_with_rulepack_manifest()
    }
}
