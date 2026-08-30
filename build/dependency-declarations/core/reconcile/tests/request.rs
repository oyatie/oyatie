#[path = "request/input_reads.rs"]
mod input_reads;
mod support;

use dependency_declarations_reconcile::*;

use support::{
    BuckConsumerVariation, digest, generation_request_with_buck_consumer_variation,
    generation_request_with_manifest, generation_request_with_provider_profile,
    valid_generation_request,
};

#[test]
fn request_identity_is_order_independent_but_content_bound() {
    let ordered = valid_generation_request(false);
    let reversed = valid_generation_request(true);
    assert_eq!(ordered.request_id(), reversed.request_id());

    let changed = generation_request_with_manifest(b"[workspace]\nmembers = []\n");
    assert_ne!(ordered.request_id(), changed.request_id());
}

#[test]
fn request_identity_binds_every_provider_graph_profile_field() {
    let baseline = valid_generation_request(false).request_id();
    let variants = [
        generation_request_with_provider_profile(
            "oyatie.reindeer.source-adaptation.v2",
            b"provider source",
            b"graph schema",
        ),
        generation_request_with_provider_profile(
            "oyatie.reindeer.source-adaptation.v1",
            b"changed provider source",
            b"graph schema",
        ),
        generation_request_with_provider_profile(
            "oyatie.reindeer.source-adaptation.v1",
            b"provider source",
            b"changed graph schema",
        ),
    ];
    assert!(
        variants
            .iter()
            .all(|request| request.request_id() != baseline)
    );
}

#[test]
fn request_identity_binds_every_buck_consumer_profile_field() {
    let baseline = valid_generation_request(false).request_id();
    let variations = [
        BuckConsumerVariation::Buck2,
        BuckConsumerVariation::Prelude,
        BuckConsumerVariation::Rules,
        BuckConsumerVariation::Toolchain,
        BuckConsumerVariation::CellConfig,
        BuckConsumerVariation::BuckConfig,
        BuckConsumerVariation::QualificationReceipt,
    ];
    assert!(variations.into_iter().all(|variation| {
        generation_request_with_buck_consumer_variation(variation).request_id() != baseline
    }));
}

#[test]
fn canonical_paths_refuse_ambiguous_or_platform_specific_spellings() {
    for path in [
        "",
        "/absolute",
        "a//b",
        "a/./b",
        "a/../b",
        "a\\b",
        "C:/absolute",
        "C:relative",
    ] {
        let failure = CanonicalPathV1::try_new(path).unwrap_err();
        assert_eq!(failure.class(), FailureClassV1::InvalidRequest, "{path}");
    }
    assert!(CanonicalPathV1::try_new("déclarations/规则.rs").is_ok());
    assert_eq!(
        CanonicalPathV1::try_new("a".repeat(ValidationBoundsV1::MAX_PATH_BYTES + 1))
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidRequest
    );
}

#[test]
fn declared_file_digest_and_length_are_verified() {
    let path = CanonicalPathV1::try_new("Cargo.toml").unwrap();
    let bytes = b"[workspace]\n".to_vec();
    assert!(
        InputFileV1::try_from_declared(
            InputFileRoleV1::Manifest,
            path.clone(),
            bytes.len() as u64,
            digest(&bytes),
            bytes.clone(),
        )
        .is_ok()
    );
    assert_eq!(
        InputFileV1::try_from_declared(
            InputFileRoleV1::Manifest,
            path,
            1,
            digest(b"different"),
            bytes,
        )
        .unwrap_err()
        .class(),
        FailureClassV1::InvalidRequest
    );
}

#[test]
fn duplicate_platform_and_tree_keys_refuse_before_hashing() {
    let platform = PlatformIdentityV1::try_new(
        "linux",
        "x86_64-unknown-linux-gnu",
        "//platform:linux-select",
        "//platform:linux",
        true,
    )
    .unwrap();
    assert_eq!(
        PlatformSetV1::try_new(vec![platform.clone(), platform])
            .unwrap_err()
            .class(),
        FailureClassV1::InvalidRequest
    );

    let entry = TreeEntryV1::new(
        CanonicalPathV1::try_new("crate/src/lib.rs").unwrap(),
        TreeFileModeV1::Regular,
        1,
        digest(b"x"),
    );
    assert_eq!(
        InputTreeV1::try_from_entries(
            TreeRoleV1::CargoHomeRead,
            CanonicalPathV1::try_new("sources.manifest").unwrap(),
            vec![entry.clone(), entry],
        )
        .unwrap_err()
        .class(),
        FailureClassV1::InvalidRequest
    );
}

#[test]
fn tree_manifests_are_order_independent_and_enforce_aggregate_bounds() {
    let first = TreeEntryV1::new(
        CanonicalPathV1::try_new("crate/a.rs").unwrap(),
        TreeFileModeV1::Regular,
        1,
        digest(b"a"),
    );
    let second = TreeEntryV1::new(
        CanonicalPathV1::try_new("crate/b.rs").unwrap(),
        TreeFileModeV1::Regular,
        1,
        digest(b"b"),
    );
    let ordered = InputTreeV1::try_from_entries(
        TreeRoleV1::CargoHomeRead,
        CanonicalPathV1::try_new("sources.manifest").unwrap(),
        vec![first.clone(), second.clone()],
    )
    .unwrap();
    let reversed = InputTreeV1::try_from_entries(
        TreeRoleV1::CargoHomeRead,
        CanonicalPathV1::try_new("sources.manifest").unwrap(),
        vec![second, first],
    )
    .unwrap();
    assert_eq!(ordered, reversed);

    let oversized = TreeEntryV1::new(
        CanonicalPathV1::try_new("crate/fixup.toml").unwrap(),
        TreeFileModeV1::Regular,
        ValidationBoundsV1::MAX_FIXUP_BYTES + 1,
        digest(b"oversized"),
    );
    assert_eq!(
        InputTreeV1::try_from_entries(
            TreeRoleV1::Fixups,
            CanonicalPathV1::try_new("fixups.manifest").unwrap(),
            vec![oversized],
        )
        .unwrap_err()
        .class(),
        FailureClassV1::InvalidRequest
    );

    let too_many_fixups = (0..=ValidationBoundsV1::MAX_FIXUP_FILES)
        .map(|index| {
            TreeEntryV1::new(
                CanonicalPathV1::try_new(format!("fixups/{index}.toml")).unwrap(),
                TreeFileModeV1::Regular,
                0,
                digest(b""),
            )
        })
        .collect();
    assert_eq!(
        InputTreeV1::try_from_entries(
            TreeRoleV1::Fixups,
            CanonicalPathV1::try_new("fixups.manifest").unwrap(),
            too_many_fixups,
        )
        .unwrap_err()
        .class(),
        FailureClassV1::InvalidRequest
    );
}
