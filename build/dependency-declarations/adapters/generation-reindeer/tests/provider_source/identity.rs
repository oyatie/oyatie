use std::path::PathBuf;

use dependency_declarations_generation_reindeer::{
    ReindeerProviderAdaptationErrorV1, ReindeerProviderSourceSnapshotV1,
    adapt_reindeer_provider_source_v1,
};

use crate::support::{PINNED_REVISION, SOURCE_PATHS, pinned_source_root, source_batch};

#[test]
fn recipe_identity_matches_the_workspace_lock() {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("build/dependency-declarations/adapters/generation-reindeer")
        });
    let lock = std::fs::read_to_string(manifest_dir.join("../../../../Cargo.lock")).unwrap();
    for package in [
        concat!(
            "name = \"syn\"\n",
            "version = \"2.0.119\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "checksum = \"872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297\"",
        ),
        concat!(
            "name = \"prettyplease\"\n",
            "version = \"0.2.37\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "checksum = \"479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b\"",
        ),
        concat!(
            "name = \"proc-macro2\"\n",
            "version = \"1.0.107\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "checksum = \"985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9\"",
        ),
        concat!(
            "name = \"quote\"\n",
            "version = \"1.0.47\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "checksum = \"1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001\"",
        ),
        concat!(
            "name = \"sha2\"\n",
            "version = \"0.10.9\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "checksum = \"a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283\"",
        ),
    ] {
        assert!(
            lock.contains(package),
            "missing exact recipe package:\n{package}"
        );
    }
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn exact_source_batch_produces_one_deterministic_adaptation() {
    let files = source_batch(&pinned_source_root());
    let snapshot = ReindeerProviderSourceSnapshotV1::new(PINNED_REVISION, files);

    let first = adapt_reindeer_provider_source_v1(&snapshot).unwrap();
    let second = adapt_reindeer_provider_source_v1(&snapshot).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.parsed_source_files(), 3);
    assert_eq!(first.files().len(), SOURCE_PATHS.len());
    assert_eq!(
        first
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>(),
        SOURCE_PATHS
    );
    let artifact = first
        .files()
        .iter()
        .find(|file| file.path() == "src/artifact.rs")
        .unwrap();
    let artifact = std::str::from_utf8(artifact.postimage()).unwrap();
    assert!(artifact.contains("ReindeerGeneratedArtifactV1"));
    assert!(artifact.contains("ReindeerRuleGraphV1"));

    let buck = first
        .files()
        .iter()
        .find(|file| file.path() == "src/buck.rs")
        .unwrap();
    assert_eq!(
        buck.preimage().unwrap(),
        std::fs::read(pinned_source_root().join("src/buck.rs")).unwrap()
    );
    assert!(buck.preimage_sha256().is_some());
    assert_ne!(buck.preimage_sha256(), Some(buck.postimage_sha256()));
    assert_eq!(first.schema().rule_variants().len(), 13);
    assert_eq!(
        first.profile().source_repository(),
        "https://github.com/facebookincubator/reindeer"
    );
    assert_eq!(first.profile().source_tag(), "v2026.08.10.00");
    assert_eq!(first.profile().source_revision(), PINNED_REVISION);
    assert!(first.profile().recipe_identity().contains("syn=2.0.119@"));
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn source_discovery_order_does_not_change_the_adaptation() {
    let root = pinned_source_root();
    let forward = source_batch(&root);
    let mut reverse = forward.clone();
    reverse.reverse();
    let adapt = |files| {
        adapt_reindeer_provider_source_v1(&ReindeerProviderSourceSnapshotV1::new(
            PINNED_REVISION,
            files,
        ))
        .unwrap()
    };

    assert_eq!(adapt(forward), adapt(reverse));
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn exact_revision_with_changed_source_bytes_refuses() {
    let mut files = source_batch(&pinned_source_root());
    files
        .iter_mut()
        .find(|file| file.path() == "src/buck.rs")
        .unwrap()
        .replace_present_bytes(b"changed source".to_vec());
    let snapshot = ReindeerProviderSourceSnapshotV1::new(PINNED_REVISION, files);

    assert_eq!(
        adapt_reindeer_provider_source_v1(&snapshot),
        Err(ReindeerProviderAdaptationErrorV1::SourceDigestMismatch)
    );
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn unsupported_revision_batch_and_presence_refuse() {
    let files = source_batch(&pinned_source_root());
    let refusal = |revision, files| {
        adapt_reindeer_provider_source_v1(&ReindeerProviderSourceSnapshotV1::new(revision, files))
    };
    assert_eq!(
        refusal("different", files.clone()),
        Err(ReindeerProviderAdaptationErrorV1::UnsupportedSourceRevision)
    );

    let mut missing = files.clone();
    missing.pop();
    assert_eq!(
        refusal(PINNED_REVISION, missing),
        Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
    );
    let mut duplicate = files.clone();
    duplicate.pop();
    duplicate.push(files[0].clone());
    assert_eq!(
        refusal(PINNED_REVISION, duplicate),
        Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
    );
    let mut present_generated = files;
    present_generated[0].replace_present_bytes(b"mod unexpected;\n".to_vec());
    assert_eq!(
        refusal(PINNED_REVISION, present_generated),
        Err(ReindeerProviderAdaptationErrorV1::SourcePresenceMismatch)
    );
}

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn oversized_source_refuses_before_digest_comparison() {
    let mut files = source_batch(&pinned_source_root());
    files
        .iter_mut()
        .find(|file| file.path() == "src/buck.rs")
        .unwrap()
        .replace_present_bytes(vec![b' '; 2 * 1024 * 1024 + 1]);
    let snapshot = ReindeerProviderSourceSnapshotV1::new(PINNED_REVISION, files);

    assert_eq!(
        adapt_reindeer_provider_source_v1(&snapshot),
        Err(ReindeerProviderAdaptationErrorV1::SourceTooLarge)
    );
}
