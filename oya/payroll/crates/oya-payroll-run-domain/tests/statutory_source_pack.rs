#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    PayrollDomainError, PayrollRulepackJurisdiction, PayrollRulepackSourceKind,
    PayrollStatutorySourceApplicability, PayrollStatutorySourceCadence,
    PayrollStatutorySourcePackInput, PayrollStatutorySourceRetrievalStatus,
    PayrollStatutorySourceRowInput, build_payroll_statutory_source_pack,
};

const SOURCE_PACK_FIXTURE_NOTE: &str =
    "source-pack plan/red synthetic fixture: no official KR/US/EU tax-rate correctness claim";

#[test]
fn statutory_source_pack_inventories_kr_us_eu_rows_without_official_rate_claim() {
    let source_pack = build_payroll_statutory_source_pack(source_pack_with_blocked_eu_row())
        .expect("future build should produce a source-pack inventory without rate claims");

    assert_eq!(
        source_pack.source_pack_ref.value,
        "rulepack-source-pack/payroll/kr-us-eu/2026"
    );
    assert_eq!(source_pack.rows.value.len(), 3);
    assert_eq!(source_pack.region_count.value, 3);
    assert!(source_pack.rows.value.iter().any(|row| {
        row.region.value == PayrollRulepackJurisdiction::Korea
            && row.publisher.value == "Ministry of Employment and Labor"
    }));
    assert!(source_pack.rows.value.iter().any(|row| {
        row.region.value == PayrollRulepackJurisdiction::UnitedStatesFederal
            && row.publisher.value == "Internal Revenue Service"
    }));
    assert!(source_pack.rows.value.iter().any(|row| {
        row.region.value == PayrollRulepackJurisdiction::EuropeanUnion
            && row.retrieval_status.value == PayrollStatutorySourceRetrievalStatus::Blocked
            && row.applicability.value
                == PayrollStatutorySourceApplicability::RegionalPackInventoryOnly
            && row.cadence.value == PayrollStatutorySourceCadence::Unresolved
            && row.unresolved_blocker_reason.value.is_some()
    }));
    assert!(source_pack.has_unresolved_blockers.value);
    assert!(!source_pack.official_tax_rate_correctness_attached.value);
    assert!(!source_pack.calculation_engine_attached.value);
    assert!(!source_pack.filing_rail_attached.value);
    assert!(!source_pack.disbursement_rail_attached.value);
    assert!(!source_pack.cloud_deployment_attached.value);
}

#[test]
fn statutory_source_pack_requires_digest_approval_owner_and_cadence_for_retrieved_rows() {
    let mut missing_digest = source_pack_with_blocked_eu_row();
    missing_digest.rows[0].source_digest = None;
    assert_eq!(
        build_payroll_statutory_source_pack(missing_digest),
        Err(PayrollDomainError::OfficialSourceDigestRequired)
    );

    let mut missing_approval = source_pack_with_blocked_eu_row();
    missing_approval.rows[1].approval_evidence_ref = None;
    assert_eq!(
        build_payroll_statutory_source_pack(missing_approval),
        Err(PayrollDomainError::OfficialSourceApprovalEvidenceRequired)
    );

    let mut missing_owner = source_pack_with_blocked_eu_row();
    missing_owner.rows[1].owner = " ".to_owned();
    assert_eq!(
        build_payroll_statutory_source_pack(missing_owner),
        Err(PayrollDomainError::OfficialSourceOwnerRequired)
    );

    let mut invalid_digest = source_pack_with_blocked_eu_row();
    invalid_digest.rows[0].source_digest = Some(digest('z'));
    assert_eq!(
        build_payroll_statutory_source_pack(invalid_digest),
        Err(PayrollDomainError::InvalidEvidenceDigest),
        "source digests must remain sha256 hex fingerprints, not arbitrary safe-looking text"
    );

    let mut unresolved_cadence = source_pack_with_blocked_eu_row();
    unresolved_cadence.rows[0].cadence = PayrollStatutorySourceCadence::Unresolved;
    assert_eq!(
        build_payroll_statutory_source_pack(unresolved_cadence),
        Err(PayrollDomainError::OfficialSourceCadenceRequired),
        "retrieved rows need a concrete cadence so cadence cannot be silently unvalidated"
    );
}

#[test]
fn statutory_source_pack_requires_unresolved_reason_for_blocked_or_missing_rows() {
    let mut blocked_without_reason = source_pack_with_blocked_eu_row();
    blocked_without_reason.rows[2].unresolved_blocker_reason = None;

    assert_eq!(
        build_payroll_statutory_source_pack(blocked_without_reason),
        Err(PayrollDomainError::OfficialSourceBlockerReasonRequired),
        "blocked EU source rows must explain the source-access/authority blocker instead of silently satisfying official-rate provenance"
    );
}

#[test]
fn statutory_source_pack_rejects_blocked_rows_as_calculation_authority() {
    let mut calculation_applicability = source_pack_with_blocked_eu_row();
    calculation_applicability.rows[2].applicability =
        PayrollStatutorySourceApplicability::Calculation;
    assert_eq!(
        build_payroll_statutory_source_pack(calculation_applicability),
        Err(PayrollDomainError::OfficialSourceApplicabilityRequired),
        "blocked EU source rows must stay inventory-only and cannot satisfy a calculation authority claim"
    );

    let mut annual_blocked_source = source_pack_with_blocked_eu_row();
    annual_blocked_source.rows[2].cadence = PayrollStatutorySourceCadence::Annual;
    assert_eq!(
        build_payroll_statutory_source_pack(annual_blocked_source),
        Err(PayrollDomainError::OfficialSourceCadenceRequired),
        "blocked EU source rows must keep an unresolved cadence instead of looking like current official cadence"
    );
}

fn source_pack_with_blocked_eu_row() -> PayrollStatutorySourcePackInput {
    PayrollStatutorySourcePackInput {
        source_pack_ref: "rulepack-source-pack/payroll/kr-us-eu/2026".to_owned(),
        payroll_year: 2026,
        rows: vec![
            kr_labor_source_row(),
            us_employer_tax_guide_row(),
            blocked_eu_source_row(),
        ],
        fixture_note: SOURCE_PACK_FIXTURE_NOTE,
        official_tax_rate_correctness_requested: false,
        calculation_engine_requested: false,
        filing_rail_requested: false,
        disbursement_rail_requested: false,
        cloud_deployment_requested: false,
    }
}

fn kr_labor_source_row() -> PayrollStatutorySourceRowInput {
    PayrollStatutorySourceRowInput {
        region: PayrollRulepackJurisdiction::Korea,
        source_kind: PayrollRulepackSourceKind::LaborStandards,
        publisher: "Ministry of Employment and Labor".to_owned(),
        official_url_or_path: Some(
            "https://www.moel.go.kr/english/policy/laborStandards.do".to_owned(),
        ),
        version_label: "moel-labor-standards-2026".to_owned(),
        effective_date: Some("2026-01-01".to_owned()),
        retrieval_status: PayrollStatutorySourceRetrievalStatus::Retrieved,
        source_digest: Some(digest('a')),
        approval_evidence_ref: Some("audit/payroll/source-pack/kr/moel-approval".to_owned()),
        applicability: PayrollStatutorySourceApplicability::CalculationAndYearEndSettlement,
        cadence: PayrollStatutorySourceCadence::Annual,
        owner: "axis-enterprise/payroll".to_owned(),
        supersedes_source_ref: None,
        expires_on: Some("2026-12-31".to_owned()),
        unresolved_blocker_reason: None,
        fixture_note: SOURCE_PACK_FIXTURE_NOTE,
    }
}

fn us_employer_tax_guide_row() -> PayrollStatutorySourceRowInput {
    PayrollStatutorySourceRowInput {
        region: PayrollRulepackJurisdiction::UnitedStatesFederal,
        source_kind: PayrollRulepackSourceKind::EmployerTaxGuide,
        publisher: "Internal Revenue Service".to_owned(),
        official_url_or_path: Some("https://www.irs.gov/publications/p15".to_owned()),
        version_label: "irs-publication-15-2026".to_owned(),
        effective_date: Some("2026-01-01".to_owned()),
        retrieval_status: PayrollStatutorySourceRetrievalStatus::Retrieved,
        source_digest: Some(digest('b')),
        approval_evidence_ref: Some("audit/payroll/source-pack/us/irs-p15-approval".to_owned()),
        applicability: PayrollStatutorySourceApplicability::CalculationAndYearEndSettlement,
        cadence: PayrollStatutorySourceCadence::Annual,
        owner: "axis-enterprise/payroll".to_owned(),
        supersedes_source_ref: None,
        expires_on: Some("2026-12-31".to_owned()),
        unresolved_blocker_reason: None,
        fixture_note: SOURCE_PACK_FIXTURE_NOTE,
    }
}

fn blocked_eu_source_row() -> PayrollStatutorySourceRowInput {
    PayrollStatutorySourceRowInput {
        region: PayrollRulepackJurisdiction::EuropeanUnion,
        source_kind: PayrollRulepackSourceKind::YearEndSettlement,
        publisher: "EU payroll statutory source authority unresolved".to_owned(),
        official_url_or_path: None,
        version_label: "blocked-eu-payroll-source-2026".to_owned(),
        effective_date: None,
        retrieval_status: PayrollStatutorySourceRetrievalStatus::Blocked,
        source_digest: None,
        approval_evidence_ref: None,
        applicability: PayrollStatutorySourceApplicability::RegionalPackInventoryOnly,
        cadence: PayrollStatutorySourceCadence::Unresolved,
        owner: "axis-enterprise/payroll".to_owned(),
        supersedes_source_ref: None,
        expires_on: None,
        unresolved_blocker_reason: Some(
            "no accepted official EU payroll statutory source URL/path is cited by PRD-PAYROLL or accepted payroll source-pack tests"
                .to_owned(),
        ),
        fixture_note: SOURCE_PACK_FIXTURE_NOTE,
    }
}

fn digest(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}
