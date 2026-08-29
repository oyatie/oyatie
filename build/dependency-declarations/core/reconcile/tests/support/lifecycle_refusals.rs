use super::lifecycle_support::*;
use dependency_declarations_reconcile::*;

fn qualified_extraction(source: &LifecycleSourceV1) -> ReleaseExtractionProfileV1 {
    extraction(
        source,
        ReleaseExtractionQualificationV1::Qualified {
            qualification_receipt_sha256: digest("qualified-extraction"),
        },
    )
}

#[test]
fn source_batch_refuses_items_from_another_source() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let cargo = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "cargo-1.98.0",
    );
    let cargo_item = release_item(
        &cargo,
        "cargo#credential-token-crlf",
        ReleaseItemKindV1::Cargo,
    );
    let extraction = qualified_extraction(&rust);
    let failure = ReleaseSourceBatchV1::try_from_items(
        rust,
        extraction,
        std::slice::from_ref(&cargo_item),
        digest("mixed-source-observation"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::SourceCoverageMismatch
    );
}

#[test]
fn ledger_refuses_duplicate_batches_and_dispositions() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let batch = qualified_batch(rust, std::slice::from_ref(&item));
    let owner_disposition = disposition(&item, ReleaseDecisionV1::Benchmark);

    let duplicate_batch = ReleaseLedgerV1::try_new(
        vec![batch.clone(), batch.clone()],
        vec![item.clone()],
        vec![owner_disposition.clone()],
    )
    .unwrap_err();
    assert_eq!(
        duplicate_batch.class(),
        LifecycleFailureClassV1::DuplicateIdentity
    );

    let duplicate_disposition = ReleaseLedgerV1::try_new(
        vec![batch],
        vec![item],
        vec![owner_disposition.clone(), owner_disposition],
    )
    .unwrap_err();
    assert_eq!(
        duplicate_disposition.class(),
        LifecycleFailureClassV1::DuplicateDisposition
    );
}

#[test]
fn ledger_refuses_an_item_without_its_source_batch() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let cargo = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "cargo-1.98.0",
    );
    let rust_item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let cargo_item = release_item(
        &cargo,
        "cargo#credential-token-crlf",
        ReleaseItemKindV1::Cargo,
    );
    let rust_batch = qualified_batch(rust, std::slice::from_ref(&rust_item));
    let failure = ReleaseLedgerV1::try_new(
        vec![rust_batch],
        vec![rust_item.clone(), cargo_item.clone()],
        vec![
            disposition(&rust_item, ReleaseDecisionV1::Benchmark),
            disposition(&cargo_item, ReleaseDecisionV1::Benchmark),
        ],
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::MissingSource);
}

#[test]
fn matrix_refuses_an_msrv_newer_than_stable() {
    let msrv = profile(
        ToolchainRoleV1::DeclaredMsrvCompatibility,
        RustVersionV1::try_new(1, 99, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "msrv-rustc",
        "msrv-cargo",
    );
    let stable = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        RustVersionV1::try_new(1, 98, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "797e8a9bc",
    );
    let beta = profile(
        ToolchainRoleV1::BetaShadow,
        RustVersionV1::try_new(1, 99, 0).unwrap(),
        LifecycleChannelV1::Beta,
        SourceMaturityV1::Provisional,
        "f47d5bb13",
        "eb98b54bc",
    );
    let nightly = profile(
        ToolchainRoleV1::NightlyShadow,
        RustVersionV1::try_new(1, 100, 0).unwrap(),
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "bff8e12ff",
        "e8cb624d5",
    );
    let failure = ToolchainMatrixV1::try_new(msrv, stable, beta, nightly).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnsupportedVersionRelation
    );
}

#[test]
fn extraction_observation_rekeys_the_batch_receipt() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let first = ReleaseSourceBatchV1::try_from_items(
        rust.clone(),
        qualified_extraction(&rust),
        std::slice::from_ref(&item),
        digest("first-observation"),
    )
    .unwrap();
    let second = ReleaseSourceBatchV1::try_from_items(
        rust.clone(),
        qualified_extraction(&rust),
        std::slice::from_ref(&item),
        digest("second-observation"),
    )
    .unwrap();
    assert_ne!(
        first.receipt().identity_sha256(),
        second.receipt().identity_sha256()
    );
}

#[test]
fn target_scope_rekeys_an_otherwise_equal_source() {
    let global = LifecycleSourceDescriptorV1::try_new(
        "rust-lang",
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        "rust-1.98.0",
        "release-object",
        LifecycleSourceScopeV1::Global,
        SourceMaturityV1::Released,
    )
    .unwrap();
    let targeted = LifecycleSourceDescriptorV1::try_new(
        "rust-lang",
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        "rust-1.98.0",
        "release-object",
        LifecycleSourceScopeV1::Target(
            LifecycleTargetTripleV1::try_new("aarch64-apple-darwin").unwrap(),
        ),
        SourceMaturityV1::Released,
    )
    .unwrap();
    let global =
        LifecycleSourceV1::try_new(global, 1024, digest("release-object"), digest("schema-v1"))
            .unwrap();
    let targeted = LifecycleSourceV1::try_new(
        targeted,
        1024,
        digest("release-object"),
        digest("schema-v1"),
    )
    .unwrap();
    assert_ne!(global.identity_sha256(), targeted.identity_sha256());
}

#[test]
fn affected_unit_summary_refuses_inconsistent_or_excessive_bounds() {
    let inconsistent = ReleaseAffectedUnitsV1::try_new(1, 0, digest("affected-units")).unwrap_err();
    assert_eq!(
        inconsistent.class(),
        LifecycleFailureClassV1::BoundsExceeded
    );

    let excessive = ReleaseAffectedUnitsV1::try_new(
        LifecycleBoundsV1::MAX_AFFECTED_UNITS + 1,
        32,
        digest("affected-units"),
    )
    .unwrap_err();
    assert_eq!(excessive.class(), LifecycleFailureClassV1::BoundsExceeded);
}

#[test]
fn extraction_profile_bound_to_another_source_is_refused() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let cargo = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "cargo-1.98.0",
    );
    let item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let cargo_extraction = extraction(
        &cargo,
        ReleaseExtractionQualificationV1::Qualified {
            qualification_receipt_sha256: digest("qualified-extraction"),
        },
    );
    let failure = ReleaseSourceBatchV1::try_from_items(
        rust,
        cargo_extraction,
        std::slice::from_ref(&item),
        digest("mismatched-extraction"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ExtractionProfileMismatch
    );
}
