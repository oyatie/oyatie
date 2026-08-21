#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use payroll_run_domain::{
    PayrollDomainError, PayrollRulepackJurisdiction, PayrollRulepackSourceInput,
    PayrollRulepackSourceKind, PayrollStatutoryRulepackManifestInput,
    build_statutory_rulepack_manifest,
};

#[test]
fn statutory_rulepack_manifest_requires_source_version_and_evidence_without_calculation_claim() {
    let manifest = build_statutory_rulepack_manifest(valid_manifest()).expect("manifest");

    assert_eq!(
        manifest.rulepack_ref.value.value,
        "rulepack/us-federal-payroll-2026"
    );
    assert_eq!(
        manifest.jurisdiction.value,
        PayrollRulepackJurisdiction::UnitedStatesFederal
    );
    assert_eq!(manifest.payroll_period.value, "2026-01");
    assert_eq!(
        manifest.source_version.value,
        "2026.irs-p15-p15t.dol-flsa-recordkeeping"
    );
    assert_eq!(manifest.sources.value.len(), 3);
    assert_eq!(manifest.source_count.value, 3);
    assert_eq!(
        manifest.sources.value[0].source_ref.value,
        "rulepack-source/irs/pub15/2026"
    );
    assert_eq!(
        manifest.sources.value[1].official_url.value,
        "https://www.irs.gov/publications/p15t"
    );
    assert!(!manifest.calculation_engine_attached.value);
    assert!(!manifest.filing_rail_attached.value);
    assert!(!manifest.disbursement_rail_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
    assert_eq!(manifest.schema_version.value, 1);
}

#[test]
fn statutory_rulepack_manifest_rejects_empty_sources_and_overclaims() {
    let mut no_sources = valid_manifest();
    no_sources.sources.clear();
    assert_eq!(
        build_statutory_rulepack_manifest(no_sources),
        Err(PayrollDomainError::RulepackSourcesRequired)
    );

    let mut calculation_claim = valid_manifest();
    calculation_claim.calculation_engine_attached = true;
    assert_eq!(
        build_statutory_rulepack_manifest(calculation_claim),
        Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim)
    );

    let mut filing_claim = valid_manifest();
    filing_claim.filing_rail_attached = true;
    assert_eq!(
        build_statutory_rulepack_manifest(filing_claim),
        Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim)
    );

    let mut disbursement_claim = valid_manifest();
    disbursement_claim.disbursement_rail_attached = true;
    assert_eq!(
        build_statutory_rulepack_manifest(disbursement_claim),
        Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim)
    );

    let mut cloud_claim = valid_manifest();
    cloud_claim.cloud_deployment_attached = true;
    assert_eq!(
        build_statutory_rulepack_manifest(cloud_claim),
        Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim)
    );
}

#[test]
fn statutory_rulepack_manifest_rejects_unofficial_or_unversioned_sources() {
    let mut unofficial = valid_manifest();
    unofficial.sources[0].official_url = "https://example.com/payroll".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(unofficial),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );

    let mut missing_version = valid_manifest();
    missing_version.source_version = " ".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(missing_version),
        Err(PayrollDomainError::InvalidRulepackSourceVersion)
    );

    let mut unsafe_source_ref = valid_manifest();
    unsafe_source_ref.sources[0].source_ref = "rulepack-source/irs/../pub15".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(unsafe_source_ref),
        Err(PayrollDomainError::InvalidRulepackSourceRef)
    );
}

#[test]
fn statutory_rulepack_manifest_rejects_invalid_source_row_provenance() {
    let mut missing_source_version = valid_manifest();
    missing_source_version.sources[0].version_label = " ".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(missing_source_version),
        Err(PayrollDomainError::InvalidRulepackSourceVersion)
    );

    let mut missing_source_evidence = valid_manifest();
    missing_source_evidence.sources[0].evidence_ref = "audit/".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(missing_source_evidence),
        Err(PayrollDomainError::InvalidEvidenceRef)
    );

    let mut secret_source_evidence = valid_manifest();
    secret_source_evidence.sources[0].evidence_ref =
        "audit/payroll/rulepack/us/bearer-token".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(secret_source_evidence),
        Err(PayrollDomainError::InvalidEvidenceRef)
    );

    let mut invalid_source_digest = valid_manifest();
    invalid_source_digest.sources[0].digest = "sha256:abc".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(invalid_source_digest),
        Err(PayrollDomainError::InvalidEvidenceDigest)
    );

    let mut missing_retrieved_at = valid_manifest();
    missing_retrieved_at.sources[0].retrieved_at_epoch_seconds = 0;
    assert_eq!(
        build_statutory_rulepack_manifest(missing_retrieved_at),
        Err(PayrollDomainError::InvalidReceivedAt)
    );

    let mut invalid_effective_date = valid_manifest();
    invalid_effective_date.sources[0].effective_date = "2026-1-1".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(invalid_effective_date),
        Err(PayrollDomainError::InvalidRulepackEffectiveDate)
    );

    let mut http_source = valid_manifest();
    http_source.sources[0].official_url = "http://www.irs.gov/publications/p15".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(http_source),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );

    let mut traversal_source = valid_manifest();
    traversal_source.sources[0].official_url = "https://www.irs.gov/../publications/p15".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(traversal_source),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );

    let mut backslash_source = valid_manifest();
    backslash_source.sources[0].official_url = "https://www.irs.gov/publications\\p15".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(backslash_source),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );

    let mut whitespace_source = valid_manifest();
    whitespace_source.sources[0].official_url =
        "https://www.irs.gov/publications/p15 bad".to_owned();
    assert_eq!(
        build_statutory_rulepack_manifest(whitespace_source),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );
}

#[test]
fn statutory_rulepack_manifest_rejects_source_rows_from_another_jurisdiction() {
    let mut manifest_input = valid_manifest();
    manifest_input.jurisdiction = PayrollRulepackJurisdiction::Korea;

    assert_eq!(
        build_statutory_rulepack_manifest(manifest_input),
        Err(PayrollDomainError::InvalidRulepackSourceUrl)
    );
}

#[test]
fn korean_statutory_manifest_can_reference_moel_and_law_sources_without_filing_claim() {
    let mut manifest_input = valid_manifest();
    manifest_input.rulepack_ref = "rulepack/kr-payroll-2026".to_owned();
    manifest_input.jurisdiction = PayrollRulepackJurisdiction::Korea;
    manifest_input.source_version = "2026.kr-labor-standards.moel".to_owned();
    manifest_input.sources = vec![
        PayrollRulepackSourceInput {
            source_kind: PayrollRulepackSourceKind::LaborStandards,
            source_ref: "rulepack-source/moel/labor-standards/2026".to_owned(),
            official_url: "https://www.moel.go.kr/english/policy/laborStandards.do".to_owned(),
            version_label: "moel-labor-standards-2026".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_543_600,
            evidence_ref: "audit/payroll/rulepack/kr/moel-labor-standards".to_owned(),
            digest: digest('d'),
        },
        PayrollRulepackSourceInput {
            source_kind: PayrollRulepackSourceKind::StatutoryFilingSchema,
            source_ref: "rulepack-source/law-go-kr/labor-standards-act/2026".to_owned(),
            official_url: "https://law.go.kr/LSW/lsInfoP.do".to_owned(),
            version_label: "law-go-kr-lsa-2026".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_543_600,
            evidence_ref: "audit/payroll/rulepack/kr/labor-standards-act".to_owned(),
            digest: digest('e'),
        },
    ];

    let manifest = build_statutory_rulepack_manifest(manifest_input).expect("KR manifest");
    assert_eq!(
        manifest.jurisdiction.value,
        PayrollRulepackJurisdiction::Korea
    );
    assert_eq!(manifest.source_count.value, 2);
    assert!(!manifest.filing_rail_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
}

#[test]
fn eu_statutory_manifest_can_reference_eur_lex_sources_without_capability_claims() {
    let mut manifest_input = valid_manifest();
    manifest_input.rulepack_ref = "rulepack/eu-payroll-2026".to_owned();
    manifest_input.jurisdiction = PayrollRulepackJurisdiction::EuropeanUnion;
    manifest_input.source_version = "2026.eu-working-time.social-security".to_owned();
    manifest_input.approval_evidence_ref = "audit/payroll/rulepack/eu/approval".to_owned();
    manifest_input.sources = vec![
        PayrollRulepackSourceInput {
            source_kind: PayrollRulepackSourceKind::LaborStandards,
            source_ref: "rulepack-source/eu/working-time-directive/2026".to_owned(),
            official_url: "https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32003L0088"
                .to_owned(),
            version_label: "eur-lex-working-time-directive-2026".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_543_600,
            evidence_ref: "audit/payroll/rulepack/eu/working-time-directive".to_owned(),
            digest: digest('f'),
        },
        PayrollRulepackSourceInput {
            source_kind: PayrollRulepackSourceKind::SocialInsurance,
            source_ref: "rulepack-source/eu/social-security-coordination/2026".to_owned(),
            official_url: "https://ec.europa.eu/social/main.jsp?catId=849".to_owned(),
            version_label: "ec-social-security-coordination-2026".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_543_600,
            evidence_ref: "audit/payroll/rulepack/eu/social-security-coordination".to_owned(),
            digest: digest('a'),
        },
    ];

    let manifest = build_statutory_rulepack_manifest(manifest_input).expect("EU manifest");
    assert_eq!(
        manifest.jurisdiction.value,
        PayrollRulepackJurisdiction::EuropeanUnion
    );
    assert_eq!(manifest.source_count.value, 2);
    assert!(
        manifest.sources.value[0]
            .official_url
            .value
            .starts_with("https://eur-lex.europa.eu/")
    );
    assert!(!manifest.calculation_engine_attached.value);
    assert!(!manifest.filing_rail_attached.value);
    assert!(!manifest.disbursement_rail_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
}

fn valid_manifest() -> PayrollStatutoryRulepackManifestInput {
    PayrollStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/us-federal-payroll-2026".to_owned(),
        jurisdiction: PayrollRulepackJurisdiction::UnitedStatesFederal,
        payroll_period: "2026-01".to_owned(),
        source_version: "2026.irs-p15-p15t.dol-flsa-recordkeeping".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/payroll/rulepack/us/approval".to_owned(),
        sources: vec![
            PayrollRulepackSourceInput {
                source_kind: PayrollRulepackSourceKind::EmployerTaxGuide,
                source_ref: "rulepack-source/irs/pub15/2026".to_owned(),
                official_url: "https://www.irs.gov/publications/p15".to_owned(),
                version_label: "irs-publication-15-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_543_600,
                evidence_ref: "audit/payroll/rulepack/us/irs-pub15".to_owned(),
                digest: digest('a'),
            },
            PayrollRulepackSourceInput {
                source_kind: PayrollRulepackSourceKind::WithholdingMethod,
                source_ref: "rulepack-source/irs/pub15t/2026".to_owned(),
                official_url: "https://www.irs.gov/publications/p15t".to_owned(),
                version_label: "irs-publication-15t-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_543_600,
                evidence_ref: "audit/payroll/rulepack/us/irs-pub15t".to_owned(),
                digest: digest('b'),
            },
            PayrollRulepackSourceInput {
                source_kind: PayrollRulepackSourceKind::WageRecordkeeping,
                source_ref: "rulepack-source/dol/flsa-recordkeeping/2026".to_owned(),
                official_url: "https://www.dol.gov/general/topic/wages/wagesrecordkeeping"
                    .to_owned(),
                version_label: "dol-flsa-recordkeeping-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_543_600,
                evidence_ref: "audit/payroll/rulepack/us/dol-flsa-recordkeeping".to_owned(),
                digest: digest('c'),
            },
        ],
        calculation_engine_attached: false,
        filing_rail_attached: false,
        disbursement_rail_attached: false,
        cloud_deployment_attached: false,
    }
}

fn digest(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}
