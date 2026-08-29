use dependency_declarations_generation_reindeer::{
    ReindeerProviderAdaptationErrorV1, ReindeerProviderSourceFileV1, ReindeerProviderSourceModeV1,
    ReindeerProviderSourceSnapshotV1,
};

fn file(
    path: &str,
    mode: ReindeerProviderSourceModeV1,
    bytes: &[u8],
) -> ReindeerProviderSourceFileV1 {
    ReindeerProviderSourceFileV1::new(path, mode, bytes.to_vec())
}

#[test]
fn snapshot_identity_is_order_independent_and_mode_sensitive() {
    let regular = file(
        "Cargo.toml",
        ReindeerProviderSourceModeV1::Regular,
        b"manifest",
    );
    let executable = file(
        "scripts/setup.sh",
        ReindeerProviderSourceModeV1::Executable,
        b"script",
    );
    let forward = ReindeerProviderSourceSnapshotV1::try_new(
        "revision",
        vec![regular.clone(), executable.clone()],
    )
    .unwrap();
    let reverse =
        ReindeerProviderSourceSnapshotV1::try_new("revision", vec![executable, regular]).unwrap();
    let mode_changed = ReindeerProviderSourceSnapshotV1::try_new(
        "revision",
        vec![
            file(
                "Cargo.toml",
                ReindeerProviderSourceModeV1::Regular,
                b"manifest",
            ),
            file(
                "scripts/setup.sh",
                ReindeerProviderSourceModeV1::Regular,
                b"script",
            ),
        ],
    )
    .unwrap();

    assert_eq!(forward, reverse);
    assert_ne!(
        forward.source_tree_sha256(),
        mode_changed.source_tree_sha256()
    );
    assert_eq!(forward.total_bytes(), 14);
    assert_eq!(forward.files()[0].path(), "Cargo.toml");
}

#[test]
fn snapshot_refuses_ambiguous_paths_and_duplicate_entries() {
    for path in ["", "/root", "trailing/", "a//b", "a/./b", "a/../b", "a\\b"] {
        assert_eq!(
            ReindeerProviderSourceSnapshotV1::try_new(
                "revision",
                vec![file(path, ReindeerProviderSourceModeV1::Regular, b"bytes")],
            ),
            Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
        );
    }
    let duplicate = file(
        "Cargo.toml",
        ReindeerProviderSourceModeV1::Regular,
        b"manifest",
    );
    assert_eq!(
        ReindeerProviderSourceSnapshotV1::try_new("revision", vec![duplicate.clone(), duplicate],),
        Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
    );
}

#[test]
fn snapshot_refuses_empty_or_oversized_input() {
    assert_eq!(
        ReindeerProviderSourceSnapshotV1::try_new("revision", Vec::new()),
        Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
    );
    assert_eq!(
        ReindeerProviderSourceSnapshotV1::try_new(
            "revision",
            vec![file(
                "large",
                ReindeerProviderSourceModeV1::Regular,
                &vec![0; 8 * 1024 * 1024 + 1],
            )],
        ),
        Err(ReindeerProviderAdaptationErrorV1::SourceTooLarge)
    );
    assert_eq!(
        ReindeerProviderSourceSnapshotV1::try_new(
            "r".repeat(129),
            vec![file(
                "Cargo.toml",
                ReindeerProviderSourceModeV1::Regular,
                b"manifest",
            )],
        ),
        Err(ReindeerProviderAdaptationErrorV1::SourceBatchMismatch)
    );
}
