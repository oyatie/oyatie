use oya_accounting_journal_domain::{
    AccountingDomainError, AccountingRulepackSourceInput, AccountingRulepackSourceKind,
    AccountingStatutoryRulepackManifestInput, Jurisdiction,
    build_accounting_statutory_rulepack_manifest,
};

fn digest(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}

fn korea_source(
    kind: AccountingRulepackSourceKind,
    slug: &str,
    official_url: &str,
) -> AccountingRulepackSourceInput {
    AccountingRulepackSourceInput {
        source_kind: kind,
        source_ref: format!("accounting-rulepack-source/korea/{slug}"),
        official_url: official_url.to_owned(),
        version_label: format!("korea-accounting-2026-{slug}"),
        effective_date: "2026-01-01".to_owned(),
        retrieved_at_epoch_seconds: 1_779_544_800,
        evidence_ref: format!("audit/accounting-rulepack/korea/{slug}"),
        digest: digest('d'),
    }
}

fn korea_manifest() -> AccountingStatutoryRulepackManifestInput {
    AccountingStatutoryRulepackManifestInput {
        rulepack_ref: "rulepack/accounting/korea/2026".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        accounting_period: "2026-01".to_owned(),
        source_version: "korea-accounting-rulepack-2026.1".to_owned(),
        effective_date: "2026-01-01".to_owned(),
        approval_evidence_ref: "audit/accounting-rulepack/korea/approval".to_owned(),
        sources: vec![
            korea_source(
                AccountingRulepackSourceKind::VatFilingDeadline,
                "nts-vat-deadlines",
                "https://www.nts.go.kr/english/cm/cntnts/cntntsView.do?cntntsId=238886&mi=40008",
            ),
            korea_source(
                AccountingRulepackSourceKind::ElectronicTaxFiling,
                "hometax-vat-filing",
                "https://www.hometax.go.kr/",
            ),
            korea_source(
                AccountingRulepackSourceKind::CorporateIncomeTax,
                "corporate-tax-law",
                "https://law.go.kr/LSW/lsInfoP.do",
            ),
        ],
        ledger_persistence_attached: false,
        workflow_engine_attached: false,
        statutory_filing_rail_attached: false,
        payment_execution_attached: false,
        cloud_deployment_attached: false,
    }
}

#[test]
fn accounting_statutory_rulepack_manifest_requires_official_sources_without_runtime_claims() {
    let manifest = build_accounting_statutory_rulepack_manifest(korea_manifest()).unwrap();

    assert_eq!(
        manifest.rulepack_ref.value.value,
        "rulepack/accounting/korea/2026"
    );
    assert_eq!(manifest.jurisdiction.value, Jurisdiction::Korea);
    assert_eq!(manifest.accounting_period.value, "2026-01");
    assert_eq!(
        manifest.source_version.value,
        "korea-accounting-rulepack-2026.1"
    );
    assert_eq!(manifest.source_count.value, 3);
    assert!(!manifest.ledger_persistence_attached.value);
    assert!(!manifest.workflow_engine_attached.value);
    assert!(!manifest.statutory_filing_rail_attached.value);
    assert!(!manifest.payment_execution_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
    assert_eq!(
        manifest.sources.value[0].source_ref.value,
        "accounting-rulepack-source/korea/nts-vat-deadlines"
    );
}

#[test]
fn accounting_statutory_rulepack_manifest_rejects_empty_sources_and_overclaims() {
    let mut missing_sources = korea_manifest();
    missing_sources.sources.clear();
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(missing_sources),
        Err(AccountingDomainError::RulepackSourcesRequired)
    );

    let mut filing_claim = korea_manifest();
    filing_claim.statutory_filing_rail_attached = true;
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(filing_claim),
        Err(AccountingDomainError::UnsupportedRulepackCapabilityClaim)
    );

    let mut cloud_claim = korea_manifest();
    cloud_claim.cloud_deployment_attached = true;
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(cloud_claim),
        Err(AccountingDomainError::UnsupportedRulepackCapabilityClaim)
    );
}

#[test]
fn accounting_statutory_rulepack_manifest_rejects_unofficial_or_unversioned_sources() {
    let mut unofficial = korea_manifest();
    unofficial.sources[0].official_url = "https://example.com/tax/vat".to_owned();
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(unofficial),
        Err(AccountingDomainError::InvalidRulepackSourceUrl)
    );

    let mut unversioned = korea_manifest();
    unversioned.sources[0].version_label.clear();
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(unversioned),
        Err(AccountingDomainError::InvalidRulepackSourceVersion)
    );

    let mut bad_digest = korea_manifest();
    bad_digest.sources[0].digest = "sha256:not-a-digest".to_owned();
    assert_eq!(
        build_accounting_statutory_rulepack_manifest(bad_digest),
        Err(AccountingDomainError::InvalidEvidenceDigest)
    );
}

#[test]
fn united_states_accounting_manifest_can_reference_irs_sources_without_filing_claim() {
    let manifest = build_accounting_statutory_rulepack_manifest(
        AccountingStatutoryRulepackManifestInput {
            rulepack_ref: "rulepack/accounting/us-federal/2026".to_owned(),
            jurisdiction: Jurisdiction::UnitedStates,
            accounting_period: "2026-01".to_owned(),
            source_version: "us-federal-accounting-rulepack-2026.1".to_owned(),
            effective_date: "2026-01-01".to_owned(),
            approval_evidence_ref: "audit/accounting-rulepack/us-federal/approval".to_owned(),
            sources: vec![
                AccountingRulepackSourceInput {
                    source_kind: AccountingRulepackSourceKind::BusinessTaxReturn,
                    source_ref: "accounting-rulepack-source/us-federal/business-taxes".to_owned(),
                    official_url: "https://www.irs.gov/businesses/small-businesses-self-employed/business-taxes".to_owned(),
                    version_label: "us-irs-business-taxes-2026".to_owned(),
                    effective_date: "2026-01-01".to_owned(),
                    retrieved_at_epoch_seconds: 1_779_544_801,
                    evidence_ref: "audit/accounting-rulepack/us-federal/business-taxes".to_owned(),
                    digest: digest('e'),
                },
                AccountingRulepackSourceInput {
                    source_kind: AccountingRulepackSourceKind::TaxRecordkeeping,
                    source_ref: "accounting-rulepack-source/us-federal/recordkeeping".to_owned(),
                    official_url: "https://www.irs.gov/businesses/small-businesses-self-employed/recordkeeping".to_owned(),
                    version_label: "us-irs-recordkeeping-2026".to_owned(),
                    effective_date: "2026-01-01".to_owned(),
                    retrieved_at_epoch_seconds: 1_779_544_802,
                    evidence_ref: "audit/accounting-rulepack/us-federal/recordkeeping".to_owned(),
                    digest: digest('f'),
                },
            ],
            ledger_persistence_attached: false,
            workflow_engine_attached: false,
            statutory_filing_rail_attached: false,
            payment_execution_attached: false,
            cloud_deployment_attached: false,
        },
    )
    .unwrap();

    assert_eq!(manifest.jurisdiction.value, Jurisdiction::UnitedStates);
    assert_eq!(manifest.source_count.value, 2);
    assert!(!manifest.statutory_filing_rail_attached.value);
    assert_eq!(
        manifest.sources.value[1].official_url.value,
        "https://www.irs.gov/businesses/small-businesses-self-employed/recordkeeping"
    );
}
