use oya_hr_employment_domain::{
    HrDomainError, HrRulepackSourceInput, HrRulepackSourceKind, HrStatutoryRulepackManifestInput,
    Jurisdiction, build_hr_statutory_rulepack_manifest,
};

fn digest(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}

fn korea_source(
    kind: HrRulepackSourceKind,
    slug: &str,
    official_url: &str,
) -> HrRulepackSourceInput {
    HrRulepackSourceInput {
        source_kind: kind,
        source_ref: format!("hr-rulepack-source/korea/{slug}"),
        official_url: official_url.to_owned(),
        version_label: format!("korea-hr-2026-{slug}"),
        effective_date: "2026-01-01".to_owned(),
        retrieved_at_epoch_seconds: 1_779_544_200,
        evidence_ref: format!("audit/hr-rulepack/korea/{slug}"),
        digest: digest('a'),
    }
}

fn korea_manifest() -> HrStatutoryRulepackManifestInput {
    HrStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/hr/korea/2026".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        source_version: "korea-hr-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/hr-rulepack/korea/approval".to_owned(),
        sources: vec![
            korea_source(
                HrRulepackSourceKind::LaborStandards,
                "labor-standards-act",
                "https://law.go.kr/LSW/lsInfoP.do",
            ),
            korea_source(
                HrRulepackSourceKind::RulesOfEmployment,
                "rules-of-employment",
                "https://www.moel.go.kr/english/policy/laborStandards.do",
            ),
            korea_source(
                HrRulepackSourceKind::LaborManagementCouncil,
                "labor-management-council",
                "https://www.moel.go.kr/english/policy/laborRelations.do",
            ),
        ],
        labor_workflow_engine_attached: false,
        payroll_calculation_attached: false,
        filing_rail_attached: false,
        cloud_deployment_attached: false,
    }
}

#[test]
fn hr_statutory_rulepack_manifest_requires_official_sources_without_runtime_claims() {
    let manifest = build_hr_statutory_rulepack_manifest(korea_manifest()).unwrap();

    assert_eq!(manifest.rulepack_ref.value.value, "rulepack/hr/korea/2026");
    assert_eq!(manifest.jurisdiction.value, Jurisdiction::Korea);
    assert_eq!(manifest.source_version.value, "korea-hr-rulepack-2026.1");
    assert_eq!(manifest.source_count.value, 3);
    assert!(!manifest.labor_workflow_engine_attached.value);
    assert!(!manifest.payroll_calculation_attached.value);
    assert!(!manifest.filing_rail_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
    assert_eq!(
        manifest.sources.value[0].source_ref.value,
        "hr-rulepack-source/korea/labor-standards-act"
    );
    assert_eq!(
        manifest.sources.value[1].official_url.value,
        "https://www.moel.go.kr/english/policy/laborStandards.do"
    );
}

#[test]
fn korea_manifest_requires_all_production_evidence_sources() {
    let mut missing_moel_rules = korea_manifest();
    missing_moel_rules
        .sources
        .retain(|source| source.source_kind != HrRulepackSourceKind::RulesOfEmployment);

    assert_eq!(
        build_hr_statutory_rulepack_manifest(missing_moel_rules),
        Err(HrDomainError::RulepackSourcesRequired),
        "KR production-evidence manifests must include the Law.go.kr labor source plus MOEL rules-of-employment and council sources"
    );
}

#[test]
fn hr_statutory_rulepack_manifest_rejects_empty_sources_and_overclaims() {
    let mut missing_sources = korea_manifest();
    missing_sources.sources.clear();
    assert_eq!(
        build_hr_statutory_rulepack_manifest(missing_sources),
        Err(HrDomainError::RulepackSourcesRequired)
    );

    let mut workflow_claim = korea_manifest();
    workflow_claim.labor_workflow_engine_attached = true;
    assert_eq!(
        build_hr_statutory_rulepack_manifest(workflow_claim),
        Err(HrDomainError::UnsupportedRulepackCapabilityClaim)
    );

    let mut cloud_claim = korea_manifest();
    cloud_claim.cloud_deployment_attached = true;
    assert_eq!(
        build_hr_statutory_rulepack_manifest(cloud_claim),
        Err(HrDomainError::UnsupportedRulepackCapabilityClaim)
    );
}

#[test]
fn hr_statutory_rulepack_manifest_rejects_unofficial_or_unversioned_sources() {
    let mut unofficial = korea_manifest();
    unofficial.sources[0].official_url = "https://example.com/hr/law".to_owned();
    assert_eq!(
        build_hr_statutory_rulepack_manifest(unofficial),
        Err(HrDomainError::InvalidRulepackSourceUrl)
    );

    let mut unversioned = korea_manifest();
    unversioned.sources[0].version_label.clear();
    assert_eq!(
        build_hr_statutory_rulepack_manifest(unversioned),
        Err(HrDomainError::InvalidRulepackSourceVersion)
    );

    let mut bad_digest = korea_manifest();
    bad_digest.sources[0].digest = "sha256:not-a-digest".to_owned();
    assert_eq!(
        build_hr_statutory_rulepack_manifest(bad_digest),
        Err(HrDomainError::InvalidRulepackSourceDigest)
    );
}

#[test]
fn united_states_manifest_requires_dol_and_eeoc_expansion_sources() {
    let missing_eeoc = HrStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/hr/us-federal/2026".to_owned(),
        jurisdiction: Jurisdiction::UnitedStates,
        source_version: "us-federal-hr-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/hr-rulepack/us-federal/approval".to_owned(),
        sources: vec![HrRulepackSourceInput {
            source_kind: HrRulepackSourceKind::WageHourRecordkeeping,
            source_ref: "hr-rulepack-source/us-federal/flsa".to_owned(),
            official_url: "https://www.dol.gov/agencies/whd/flsa".to_owned(),
            version_label: "us-dol-flsa-2026".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_544_201,
            evidence_ref: "audit/hr-rulepack/us-federal/flsa".to_owned(),
            digest: digest('b'),
        }],
        labor_workflow_engine_attached: false,
        payroll_calculation_attached: false,
        filing_rail_attached: false,
        cloud_deployment_attached: false,
    };

    assert_eq!(
        build_hr_statutory_rulepack_manifest(missing_eeoc),
        Err(HrDomainError::RulepackSourcesRequired),
        "US expansion evidence must include both DOL wage/hour and EEOC equal-employment official sources"
    );
}

#[test]
fn european_union_manifest_accepts_eur_lex_inventory_without_runtime_claims() {
    let manifest = build_hr_statutory_rulepack_manifest(HrStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/hr/eu/2026".to_owned(),
        jurisdiction: Jurisdiction::EuropeanUnion,
        source_version: "eu-hr-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/hr-rulepack/eu/approval".to_owned(),
        sources: vec![HrRulepackSourceInput {
            source_kind: HrRulepackSourceKind::LeaveAndHolidayStandards,
            source_ref: "hr-rulepack-source/eu/working-time-directive".to_owned(),
            official_url: "https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32003L0088"
                .to_owned(),
            version_label: "eu-working-time-directive-2003-88-ec".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            retrieved_at_epoch_seconds: 1_779_544_203,
            evidence_ref: "audit/hr-rulepack/eu/working-time-directive".to_owned(),
            digest: digest('d'),
        }],
        labor_workflow_engine_attached: false,
        payroll_calculation_attached: false,
        filing_rail_attached: false,
        cloud_deployment_attached: false,
    })
    .unwrap();

    assert_eq!(manifest.jurisdiction.value, Jurisdiction::EuropeanUnion);
    assert_eq!(manifest.source_count.value, 1);
    assert!(!manifest.labor_workflow_engine_attached.value);
    assert!(!manifest.payroll_calculation_attached.value);
    assert!(!manifest.filing_rail_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
}

#[test]
fn european_union_manifest_rejects_non_eur_lex_sources_even_with_eur_lex_inventory() {
    let non_eur_lex_inventory = HrStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/hr/eu/2026".to_owned(),
        jurisdiction: Jurisdiction::EuropeanUnion,
        source_version: "eu-hr-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/hr-rulepack/eu/approval".to_owned(),
        sources: vec![
            HrRulepackSourceInput {
                source_kind: HrRulepackSourceKind::LeaveAndHolidayStandards,
                source_ref: "hr-rulepack-source/eu/working-time-directive".to_owned(),
                official_url:
                    "https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32003L0088"
                        .to_owned(),
                version_label: "eu-working-time-directive-2003-88-ec".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_544_203,
                evidence_ref: "audit/hr-rulepack/eu/working-time-directive".to_owned(),
                digest: digest('d'),
            },
            HrRulepackSourceInput {
                source_kind: HrRulepackSourceKind::WageHourRecordkeeping,
                source_ref: "hr-rulepack-source/eu/us-dol-leak".to_owned(),
                official_url: "https://www.dol.gov/agencies/whd/flsa".to_owned(),
                version_label: "us-dol-flsa-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_544_204,
                evidence_ref: "audit/hr-rulepack/eu/us-dol-leak".to_owned(),
                digest: digest('e'),
            },
        ],
        labor_workflow_engine_attached: false,
        payroll_calculation_attached: false,
        filing_rail_attached: false,
        cloud_deployment_attached: false,
    };

    assert_eq!(
        build_hr_statutory_rulepack_manifest(non_eur_lex_inventory),
        Err(HrDomainError::InvalidRulepackSourceUrl),
        "EU source inventories must stay bounded to official EUR-Lex evidence only"
    );
}

#[test]
fn united_states_hr_manifest_can_reference_dol_and_eeoc_sources_without_payroll_claim() {
    let manifest = build_hr_statutory_rulepack_manifest(HrStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/hr/us-federal/2026".to_owned(),
        jurisdiction: Jurisdiction::UnitedStates,
        source_version: "us-federal-hr-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/hr-rulepack/us-federal/approval".to_owned(),
        sources: vec![
            HrRulepackSourceInput {
                source_kind: HrRulepackSourceKind::WageHourRecordkeeping,
                source_ref: "hr-rulepack-source/us-federal/flsa".to_owned(),
                official_url: "https://www.dol.gov/agencies/whd/flsa".to_owned(),
                version_label: "us-dol-flsa-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_544_201,
                evidence_ref: "audit/hr-rulepack/us-federal/flsa".to_owned(),
                digest: digest('b'),
            },
            HrRulepackSourceInput {
                source_kind: HrRulepackSourceKind::EqualEmployment,
                source_ref: "hr-rulepack-source/us-federal/eeoc".to_owned(),
                official_url: "https://www.eeoc.gov/employers/small-business".to_owned(),
                version_label: "us-eeoc-small-business-2026".to_owned(),
                effective_date: "2026-01-01".to_owned(),
                retrieved_at_epoch_seconds: 1_779_544_202,
                evidence_ref: "audit/hr-rulepack/us-federal/eeoc".to_owned(),
                digest: digest('c'),
            },
        ],
        labor_workflow_engine_attached: false,
        payroll_calculation_attached: false,
        filing_rail_attached: false,
        cloud_deployment_attached: false,
    })
    .unwrap();

    assert_eq!(manifest.jurisdiction.value, Jurisdiction::UnitedStates);
    assert_eq!(manifest.source_count.value, 2);
    assert!(!manifest.payroll_calculation_attached.value);
    assert_eq!(
        manifest.sources.value[0].official_url.value,
        "https://www.dol.gov/agencies/whd/flsa"
    );
}
