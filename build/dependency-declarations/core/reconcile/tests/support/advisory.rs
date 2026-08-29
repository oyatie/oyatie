use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

pub(super) fn identifier(namespace: AdvisoryNamespaceV1, value: &str) -> AdvisoryIdentifierV1 {
    AdvisoryIdentifierV1::try_new(namespace, value).unwrap()
}

pub(super) fn record_source(
    component: LifecycleComponentV1,
    authority: AdvisoryAuthorityV1,
    revision: &str,
    qualification: AdvisoryRecordQualificationV1,
) -> AdvisoryRecordSourceV1 {
    let source = LifecycleSourceV1::try_new(
        LifecycleSourceDescriptorV1::try_new(
            "advisory-provider",
            component,
            LifecycleChannelV1::Advisory,
            revision,
            "advisory-feed",
            LifecycleSourceScopeV1::Global,
            SourceMaturityV1::Released,
        )
        .unwrap(),
        4096,
        digest(&format!("{revision}-object")),
        digest("advisory-schema-v1"),
    )
    .unwrap();
    AdvisoryRecordSourceV1::try_new(source, authority, qualification).unwrap()
}

pub(super) fn qualified() -> AdvisoryRecordQualificationV1 {
    AdvisoryRecordQualificationV1::Qualified {
        qualification_receipt_sha256: digest("qualified-advisory-extractor"),
    }
}

pub(super) fn candidate() -> AdvisoryRecordQualificationV1 {
    AdvisoryRecordQualificationV1::Candidate {
        observation_receipt_sha256: digest("candidate-advisory-extractor"),
    }
}

pub(super) fn h2_package() -> CargoPackageIdentityV1 {
    CargoPackageIdentityV1::try_new("https://github.com/rust-lang/crates.io-index", "h2").unwrap()
}

pub(super) fn complete_h2(fixed: &str) -> AdvisoryAffectedSetV1 {
    let range = CargoAffectedRangeV1::try_new(
        AdvisoryRangeStartV1::Beginning,
        AdvisoryRangeEndV1::Fixed(CargoVersionV1::try_new(fixed).unwrap()),
    )
    .unwrap();
    AdvisoryAffectedSetV1::try_complete(vec![
        CargoAdvisoryClaimV1::try_new(h2_package(), vec![range]).unwrap(),
    ])
    .unwrap()
}

pub(super) fn active_record(
    source: AdvisoryRecordSourceV1,
    primary: AdvisoryIdentifierV1,
    aliases: Vec<AdvisoryIdentifierV1>,
    affected: AdvisoryAffectedSetV1,
    modified_at: u64,
) -> AdvisoryRecordV1 {
    let content_sha256 = digest(&format!("{}-{modified_at}", primary.value()));
    AdvisoryRecordV1::try_new(
        source,
        primary,
        aliases,
        AdvisoryLifecycleV1::try_active(
            AdvisoryTimestampV1::from_unix_seconds(100),
            AdvisoryTimestampV1::from_unix_seconds(modified_at),
        )
        .unwrap(),
        affected,
        content_sha256,
    )
    .unwrap()
}

#[test]
fn rustsec_and_ghsa_aliases_form_one_normalized_vulnerability() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let ghsa = identifier(AdvisoryNamespaceV1::Ghsa, "GHSA-q83h-524g-xf6h");
    let affected = complete_h2("0.4.16");
    let rustsec_record = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-revision",
            qualified(),
        ),
        rustsec.clone(),
        vec![ghsa.clone()],
        affected.clone(),
        200,
    );
    let ghsa_record = active_record(
        record_source(
            LifecycleComponentV1::GitHubAdvisory,
            AdvisoryAuthorityV1::GitHubAdvisory,
            "ghsa-revision",
            qualified(),
        ),
        ghsa.clone(),
        vec![rustsec],
        affected.clone(),
        210,
    );

    let ledger =
        AdvisoryLedgerV1::try_normalize(vec![rustsec_record.clone(), ghsa_record.clone()]).unwrap();
    let reversed = AdvisoryLedgerV1::try_normalize(vec![ghsa_record, rustsec_record]).unwrap();
    assert_eq!(ledger.identity_sha256(), reversed.identity_sha256());
    assert_eq!(ledger.facts().len(), 1);
    assert_eq!(ledger.record_count(), 2);
    assert_eq!(ledger.identifier_occurrence_count(), 4);
    assert_eq!(ledger.facts()[0].identifiers().len(), 2);
    assert_eq!(ledger.facts()[0].canonical(), &ghsa);
    assert!(ledger.facts()[0].identifiers().binary_search(&ghsa).is_ok());
    assert_eq!(
        ledger.facts()[0].affected_set_qualification(),
        NormalizedAdvisoryAffectedSetQualificationV1::Qualified
    );

    let affected_version = CargoVersionV1::try_new("0.4.15+local").unwrap();
    let fixed_version = CargoVersionV1::try_new("0.4.16").unwrap();
    let ranges = affected.claims().unwrap()[0].ranges();
    assert!(ranges[0].contains(&affected_version));
    assert!(!ranges[0].contains(&fixed_version));
}

#[test]
fn later_withdrawal_supersedes_active_history_without_erasing_it() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let active = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-before-withdrawal",
            qualified(),
        ),
        rustsec.clone(),
        Vec::new(),
        complete_h2("0.4.16"),
        200,
    );
    let withdrawn = AdvisoryRecordV1::try_new(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-after-withdrawal",
            qualified(),
        ),
        rustsec,
        Vec::new(),
        AdvisoryLifecycleV1::try_withdrawn(
            AdvisoryTimestampV1::from_unix_seconds(100),
            AdvisoryTimestampV1::from_unix_seconds(300),
            AdvisoryTimestampV1::from_unix_seconds(250),
        )
        .unwrap(),
        complete_h2("0.4.16"),
        digest("withdrawn-record"),
    )
    .unwrap();

    let ledger = AdvisoryLedgerV1::try_normalize(vec![withdrawn, active]).unwrap();
    assert_eq!(ledger.facts()[0].records().len(), 2);
    assert_eq!(
        ledger.facts()[0].lifecycle(),
        NormalizedAdvisoryLifecycleV1::Withdrawn
    );
}

#[test]
fn cna_and_upstream_aliases_retain_cna_identity_without_assigning_ids() {
    let cve = identifier(AdvisoryNamespaceV1::Cve, "CVE-2026-0001");
    let upstream = identifier(AdvisoryNamespaceV1::Upstream, "UPSTREAM-42");
    let cna = active_record(
        record_source(
            LifecycleComponentV1::Cna,
            AdvisoryAuthorityV1::Cna(AdvisoryAuthorityNameV1::try_new("example-cna").unwrap()),
            "cna-revision",
            qualified(),
        ),
        cve.clone(),
        vec![upstream.clone()],
        AdvisoryAffectedSetV1::reference_only(digest("cna-reference")),
        200,
    );
    let upstream_record = active_record(
        record_source(
            LifecycleComponentV1::UpstreamAdvisory,
            AdvisoryAuthorityV1::Upstream(
                AdvisoryAuthorityNameV1::try_new("upstream-project").unwrap(),
            ),
            "upstream-revision",
            qualified(),
        ),
        upstream,
        vec![cve.clone()],
        AdvisoryAffectedSetV1::reference_only(digest("upstream-reference")),
        210,
    );
    let ledger = AdvisoryLedgerV1::try_normalize(vec![upstream_record, cna]).unwrap();
    assert_eq!(ledger.facts()[0].canonical(), &cve);
    assert_eq!(
        ledger.facts()[0].affected_set_qualification(),
        NormalizedAdvisoryAffectedSetQualificationV1::ReferenceOnly
    );
}
