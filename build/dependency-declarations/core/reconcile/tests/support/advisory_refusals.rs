use super::advisory::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

#[test]
fn conflicting_complete_ranges_refuse_a_normalized_fact() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let ghsa = identifier(AdvisoryNamespaceV1::Ghsa, "GHSA-q83h-524g-xf6h");
    let rustsec_record = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-conflict",
            qualified(),
        ),
        rustsec.clone(),
        vec![ghsa.clone()],
        complete_h2("0.4.16"),
        200,
    );
    let ghsa_record = active_record(
        record_source(
            LifecycleComponentV1::GitHubAdvisory,
            AdvisoryAuthorityV1::GitHubAdvisory,
            "ghsa-conflict",
            qualified(),
        ),
        ghsa,
        vec![rustsec],
        complete_h2("0.4.17"),
        210,
    );

    let failure = AdvisoryLedgerV1::try_normalize(
        vec![rustsec_record, ghsa_record],
        continue_advisory_normalization,
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ConflictingAdvisoryRange
    );
}

#[test]
fn candidate_and_reference_only_records_remain_unqualified() {
    let candidate_record = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-candidate",
            candidate(),
        ),
        identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258"),
        Vec::new(),
        complete_h2("0.4.16"),
        200,
    );
    let candidate_ledger =
        AdvisoryLedgerV1::try_normalize(vec![candidate_record], continue_advisory_normalization)
            .unwrap();
    assert_eq!(
        candidate_ledger.facts()[0].affected_set_qualification(),
        NormalizedAdvisoryAffectedSetQualificationV1::Candidate
    );

    let reference_record = active_record(
        record_source(
            LifecycleComponentV1::Osv,
            AdvisoryAuthorityV1::Osv,
            "osv-reference",
            qualified(),
        ),
        identifier(AdvisoryNamespaceV1::Osv, "OSV-2026-1"),
        Vec::new(),
        AdvisoryAffectedSetV1::reference_only(digest("reference-only")),
        200,
    );
    let reference_ledger =
        AdvisoryLedgerV1::try_normalize(vec![reference_record], continue_advisory_normalization)
            .unwrap();
    assert_eq!(
        reference_ledger.facts()[0].affected_set_qualification(),
        NormalizedAdvisoryAffectedSetQualificationV1::ReferenceOnly
    );
}

#[test]
fn candidate_aliases_cannot_merge_a_qualified_identity_graph() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let ghsa = identifier(AdvisoryNamespaceV1::Ghsa, "GHSA-q83h-524g-xf6h");
    let candidate_record = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "candidate-alias",
            candidate(),
        ),
        rustsec.clone(),
        vec![ghsa.clone()],
        complete_h2("0.4.16"),
        200,
    );
    let qualified_record = active_record(
        record_source(
            LifecycleComponentV1::GitHubAdvisory,
            AdvisoryAuthorityV1::GitHubAdvisory,
            "qualified-alias",
            qualified(),
        ),
        ghsa,
        vec![rustsec],
        complete_h2("0.4.16"),
        210,
    );

    let failure = AdvisoryLedgerV1::try_normalize(
        vec![candidate_record, qualified_record],
        continue_advisory_normalization,
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::MixedAdvisoryQualification
    );
}

#[test]
fn ambiguous_same_timestamp_history_is_refused() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let first = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-first",
            qualified(),
        ),
        rustsec.clone(),
        Vec::new(),
        complete_h2("0.4.16"),
        200,
    );
    let second = active_record(
        record_source(
            LifecycleComponentV1::RustSec,
            AdvisoryAuthorityV1::RustSec,
            "rustsec-second",
            qualified(),
        ),
        rustsec,
        Vec::new(),
        complete_h2("0.4.17"),
        200,
    );

    let failure =
        AdvisoryLedgerV1::try_normalize(vec![first, second], continue_advisory_normalization)
            .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ConflictingAdvisoryHistory
    );
}

#[test]
fn advisory_authority_must_match_the_exact_source_component() {
    let source = LifecycleSourceV1::try_new(
        LifecycleSourceDescriptorV1::try_new(
            "advisory-provider",
            LifecycleComponentV1::Osv,
            LifecycleChannelV1::Advisory,
            "osv-revision",
            "osv-feed",
            LifecycleSourceScopeV1::Global,
            SourceMaturityV1::Released,
        )
        .unwrap(),
        4096,
        digest("osv-object"),
        digest("osv-schema"),
    )
    .unwrap();

    let failure =
        AdvisoryRecordSourceV1::try_new(source, AdvisoryAuthorityV1::RustSec, qualified())
            .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::AdvisorySourceMismatch
    );

    let mismatched_primary = AdvisoryRecordV1::try_new(
        record_source(
            LifecycleComponentV1::Osv,
            AdvisoryAuthorityV1::Osv,
            "osv-primary-mismatch",
            qualified(),
        ),
        identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258"),
        Vec::new(),
        AdvisoryLifecycleV1::try_active(
            LifecycleTimestampV1::from_unix_seconds(100),
            LifecycleTimestampV1::from_unix_seconds(200),
        )
        .unwrap(),
        AdvisoryAffectedSetV1::reference_only(digest("primary-mismatch")),
        digest("primary-mismatch-content"),
    )
    .unwrap_err();
    assert_eq!(
        mismatched_primary.class(),
        LifecycleFailureClassV1::AdvisorySourceMismatch
    );
}

#[test]
fn structured_advisory_namespaces_refuse_malformed_identifiers() {
    for (namespace, value) in [
        (AdvisoryNamespaceV1::Cve, "CVE-2026-1"),
        (AdvisoryNamespaceV1::RustSec, "RUSTSEC-26-0258"),
        (AdvisoryNamespaceV1::Ghsa, "GHSA-q83h-524g"),
        (AdvisoryNamespaceV1::Osv, "OSV 2026 1"),
    ] {
        assert_eq!(
            AdvisoryIdentifierV1::try_new(namespace, value)
                .unwrap_err()
                .class(),
            LifecycleFailureClassV1::InvalidFact
        );
    }
}

#[test]
fn cargo_ranges_refuse_noncanonical_endpoints_and_overlap() {
    assert_eq!(
        CargoVersionV1::try_new("1").unwrap_err().class(),
        LifecycleFailureClassV1::InvalidPackageVersion
    );
    let metadata_endpoint = CargoAffectedRangeV1::try_new(
        AdvisoryRangeStartV1::Introduced(CargoVersionV1::try_new("1.0.0+local").unwrap()),
        AdvisoryRangeEndV1::Unbounded,
    )
    .unwrap_err();
    assert_eq!(
        metadata_endpoint.class(),
        LifecycleFailureClassV1::InvalidPackageVersion
    );

    let first = CargoAffectedRangeV1::try_new(
        AdvisoryRangeStartV1::Beginning,
        AdvisoryRangeEndV1::Fixed(CargoVersionV1::try_new("2.0.0").unwrap()),
    )
    .unwrap();
    let overlapping = CargoAffectedRangeV1::try_new(
        AdvisoryRangeStartV1::Introduced(CargoVersionV1::try_new("1.5.0").unwrap()),
        AdvisoryRangeEndV1::Unbounded,
    )
    .unwrap();
    let failure =
        CargoAdvisoryClaimV1::try_new(h2_package(), vec![first, overlapping]).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ConflictingAdvisoryRange
    );
}

#[test]
fn duplicate_aliases_and_source_records_are_refused() {
    let rustsec = identifier(AdvisoryNamespaceV1::RustSec, "RUSTSEC-2026-0258");
    let source = record_source(
        LifecycleComponentV1::RustSec,
        AdvisoryAuthorityV1::RustSec,
        "rustsec-duplicate",
        qualified(),
    );
    let duplicate_alias = AdvisoryRecordV1::try_new(
        source.clone(),
        rustsec.clone(),
        vec![rustsec.clone()],
        active_lifecycle(200),
        complete_h2("0.4.16"),
        digest("duplicate-alias"),
    )
    .unwrap_err();
    assert_eq!(
        duplicate_alias.class(),
        LifecycleFailureClassV1::DuplicateIdentity
    );

    let record = active_record(source, rustsec, Vec::new(), complete_h2("0.4.16"), 200);
    let duplicate_record = AdvisoryLedgerV1::try_normalize(
        vec![record.clone(), record],
        continue_advisory_normalization,
    )
    .unwrap_err();
    assert_eq!(
        duplicate_record.class(),
        LifecycleFailureClassV1::DuplicateIdentity
    );
}
