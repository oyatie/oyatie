#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    PayrollDomainError, PayrollRulepackJurisdiction, YearEndEvidenceRefInput,
    YearEndRegionalDependency, YearEndSettlementInput, YearEndSettlementSourceKind,
    prepare_year_end_settlement_inputs,
};

const SYNTHETIC_FIXTURE_NOTE: &str =
    "synthetic/non-authoritative fixture: no production year-end settlement or filing claim";

#[test]
fn year_end_settlement_requires_rulepack_manifest_and_source_evidence_refs() {
    let mut input = synthetic_year_end_settlement_input();
    input.rulepack_manifest_ref = None;
    input.source_version = None;
    input.evidence_refs.clear();

    assert_eq!(input.fixture_note, SYNTHETIC_FIXTURE_NOTE);
    assert_eq!(
        prepare_year_end_settlement_inputs(input),
        Err(PayrollDomainError::YearEndSettlementSourceEvidenceRequired),
        "year-end settlement input preparation must fail closed without source-versioned rulepack evidence refs"
    );
}

#[test]
fn year_end_settlement_prepares_input_schema_without_filing_transport_side_effect() {
    let prepared = prepare_year_end_settlement_inputs(synthetic_year_end_settlement_input())
        .expect("future build should prepare synthetic year-end input schema");

    assert_eq!(prepared.run_id.value, "prun_kr_2026_12");
    assert_eq!(prepared.tenant_id.value, "ten_acme");
    assert_eq!(prepared.legal_entity_id.value, "le_kr_001");
    assert_eq!(prepared.payroll_year.value, 2026);
    assert_eq!(
        prepared.jurisdiction.value,
        PayrollRulepackJurisdiction::Korea
    );
    assert_eq!(prepared.regional_dependencies.value.len(), 1);
    assert_eq!(
        prepared.regional_dependencies.value[0].pack_code.value,
        "KR"
    );
    assert_eq!(prepared.evidence_refs.value.len(), 3);
    assert!(prepared.evidence_refs.value.iter().all(|evidence| {
        evidence
            .ref_value
            .value
            .starts_with("audit/payroll/year-end/")
            && evidence.source_version.value == "synthetic-kr-year-end-2026-v0"
    }));
    assert_eq!(prepared.employee_inputs.value.len(), 1);
    assert_eq!(
        prepared.employee_inputs.value[0].payee_id.value,
        "payee_001"
    );
    assert_eq!(
        prepared.employee_inputs.value[0].gross_pay.value.currency,
        "KRW"
    );
    assert!(!prepared.direct_agency_submission_attached.value);
    assert!(!prepared.filing_rail_attached.value);
    assert!(!prepared.disbursement_rail_attached.value);
    assert!(!prepared.production_close_attached.value);
    assert!(!prepared.cloud_deployment_attached.value);
}

#[test]
fn year_end_settlement_rejects_unofficial_fixture_as_source_authority() {
    let mut input = synthetic_year_end_settlement_input();
    input.source_version = Some("synthetic-only".to_owned());
    input.unofficial_source_fixture = true;

    assert_eq!(
        prepare_year_end_settlement_inputs(input),
        Err(PayrollDomainError::OfficialYearEndSourceEvidenceRequired),
        "synthetic/non-authoritative fixtures cannot satisfy official source provenance"
    );
}

#[test]
fn year_end_settlement_requires_regional_pack_dependency_before_preparation() {
    let mut input = synthetic_year_end_settlement_input();
    input.regional_dependencies.clear();

    assert_eq!(
        prepare_year_end_settlement_inputs(input),
        Err(PayrollDomainError::YearEndRegionalPackRequired),
        "KR/US/EU regional pack dependencies must be explicit before preparing year-end inputs"
    );
}

fn synthetic_year_end_settlement_input() -> YearEndSettlementInput {
    YearEndSettlementInput {
        run_id: "prun_kr_2026_12".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_year: 2026,
        jurisdiction: PayrollRulepackJurisdiction::Korea,
        rulepack_ref: "rulepack/kr-year-end-synthetic-2026".to_owned(),
        rulepack_manifest_ref: Some("rulepack-source/kr/year-end/synthetic/2026".to_owned()),
        source_version: Some("synthetic-kr-year-end-2026-v0".to_owned()),
        evidence_refs: vec![
            YearEndEvidenceRefInput {
                source_kind: YearEndSettlementSourceKind::WageLedgerDigest,
                ref_value: "audit/payroll/year-end/wage-ledger-digest".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
            YearEndEvidenceRefInput {
                source_kind: YearEndSettlementSourceKind::WithholdingEvidence,
                ref_value: "audit/payroll/year-end/withholding-evidence".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
            YearEndEvidenceRefInput {
                source_kind: YearEndSettlementSourceKind::EmployeeDeclaration,
                ref_value: "audit/payroll/year-end/employee-declaration".to_owned(),
                source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            },
        ],
        regional_dependencies: vec![YearEndRegionalDependency {
            pack_code: "KR".to_owned(),
            source_version: "synthetic-kr-year-end-2026-v0".to_owned(),
            evidence_ref: "audit/payroll/year-end/regional-pack/kr".to_owned(),
        }],
        employee_inputs: vec![synthetic_employee_year_end_input()],
        fixture_note: SYNTHETIC_FIXTURE_NOTE,
        unofficial_source_fixture: false,
        direct_agency_submission_requested: false,
        filing_rail_requested: false,
        disbursement_rail_requested: false,
        production_close_requested: false,
        cloud_deployment_requested: false,
    }
}

fn synthetic_employee_year_end_input() -> oya_payroll_run_domain::YearEndEmployeeInput {
    oya_payroll_run_domain::YearEndEmployeeInput {
        payee_id: "payee_001".to_owned(),
        employee_ref: "person/acme/001".to_owned(),
        gross_pay_minor: 36_000_000,
        withholding_minor: 2_700_000,
        currency: "KRW".to_owned(),
        wage_ledger_evidence_ref: "audit/payroll/year-end/payee-001/wage-ledger".to_owned(),
        declaration_evidence_ref: "audit/payroll/year-end/payee-001/declaration".to_owned(),
    }
}
