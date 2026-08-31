use crate::{
    Entry, EntryKind, EntryState, ObjectId, RepositoryDelta, RepositoryManifest, RepositoryPath,
    ResolvedRevision, RevisionId, SnapshotFailure, SnapshotLimitSpec, SnapshotLimits, TreeId,
};

fn object(hex: &str) -> ObjectId {
    ObjectId::from_hex(hex).unwrap()
}

fn revision(tree: &str) -> ResolvedRevision {
    let commit = RevisionId::from_hex(&"1".repeat(40)).unwrap();
    ResolvedRevision::new(commit, commit, TreeId::from_hex(tree).unwrap()).unwrap()
}

fn limits() -> SnapshotLimits {
    SnapshotLimits::new(SnapshotLimitSpec {
        max_entries: 100,
        max_path_bytes: 100,
        max_manifest_bytes: 10_000,
        max_selected_contents: 100,
        max_content_bytes: 1_000,
        max_total_content_bytes: 10_000,
        max_stdout_bytes: 20_000,
        max_stderr_bytes: 1_000,
    })
    .unwrap()
}

fn entry(path: &str, kind: EntryKind, hex: &str) -> Entry {
    Entry::new(
        RepositoryPath::from_utf8(path).unwrap(),
        EntryState::new(kind, object(hex)),
    )
}

#[test]
fn manifest_sorts_raw_paths_and_requires_parent_trees() {
    let manifest = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![
            entry("a/z.rs", EntryKind::Blob, &"3".repeat(40)),
            entry("a", EntryKind::Tree, &"4".repeat(40)),
        ],
        limits(),
    )
    .unwrap();

    assert_eq!(manifest.entries()[0].path().as_bytes(), b"a");
    assert_eq!(manifest.entries()[1].path().as_bytes(), b"a/z.rs");
}

#[test]
fn files_under_uses_an_exact_directory_prefix() {
    let manifest = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![
            entry("a", EntryKind::Tree, &"3".repeat(40)),
            entry("a/file", EntryKind::Blob, &"4".repeat(40)),
            entry("aa", EntryKind::Tree, &"5".repeat(40)),
            entry("aa/file", EntryKind::Blob, &"6".repeat(40)),
        ],
        limits(),
    )
    .unwrap();
    let directory = RepositoryPath::from_utf8("a").unwrap();
    let paths: Vec<&[u8]> = manifest
        .files_under(&directory)
        .map(|entry| entry.path().as_bytes())
        .collect();

    assert_eq!(paths, [b"a/file".as_slice()]);
}

#[test]
fn delta_ignores_parent_tree_identity_changes() {
    let base = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![
            entry("a", EntryKind::Tree, &"3".repeat(40)),
            entry("a/file", EntryKind::Blob, &"4".repeat(40)),
        ],
        limits(),
    )
    .unwrap();
    let head = RepositoryManifest::new(
        revision(&"5".repeat(40)),
        vec![
            entry("a", EntryKind::Tree, &"6".repeat(40)),
            entry("a/file", EntryKind::Blob, &"4".repeat(40)),
        ],
        limits(),
    )
    .unwrap();

    assert!(RepositoryDelta::between(&base, &head).entries().is_empty());
}

#[test]
fn only_unambiguous_identical_moves_are_exposed() {
    let base = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![entry("old", EntryKind::Blob, &"4".repeat(40))],
        limits(),
    )
    .unwrap();
    let head = RepositoryManifest::new(
        revision(&"5".repeat(40)),
        vec![entry("new", EntryKind::Blob, &"4".repeat(40))],
        limits(),
    )
    .unwrap();
    let delta = RepositoryDelta::between(&base, &head);

    assert_eq!(
        delta
            .exact_moves()
            .get(&RepositoryPath::from_utf8("new").unwrap()),
        Some(&RepositoryPath::from_utf8("old").unwrap())
    );
}

#[test]
fn duplicate_paths_and_entry_limits_refuse_closed() {
    let duplicate = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![
            entry("same", EntryKind::Blob, &"3".repeat(40)),
            entry("same", EntryKind::ExecutableBlob, &"4".repeat(40)),
        ],
        limits(),
    );
    assert!(matches!(duplicate, Err(SnapshotFailure::DuplicatePath(_))));

    let one_entry = SnapshotLimits::new(SnapshotLimitSpec {
        max_entries: 1,
        ..limit_spec()
    })
    .unwrap();
    let too_many = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![
            entry("one", EntryKind::Blob, &"3".repeat(40)),
            entry("two", EntryKind::Blob, &"4".repeat(40)),
        ],
        one_entry,
    );
    assert!(matches!(
        too_many,
        Err(SnapshotFailure::LimitExceeded {
            limit: "entry count",
            ..
        })
    ));
}

#[test]
fn equal_tree_revisions_share_validated_entries_only() {
    let source = RepositoryManifest::new(
        revision(&"2".repeat(40)),
        vec![entry("one", EntryKind::Blob, &"3".repeat(40))],
        limits(),
    )
    .unwrap();
    let commit = RevisionId::from_hex(&"4".repeat(40)).unwrap();
    let same_tree =
        ResolvedRevision::new(commit, commit, TreeId::from_hex(&"2".repeat(40)).unwrap()).unwrap();
    let shared = RepositoryManifest::at_revision(same_tree, &source).unwrap();

    assert!(std::ptr::eq(
        source.entries().as_ptr(),
        shared.entries().as_ptr()
    ));
    assert_ne!(source.digest(), shared.digest());

    let other_tree =
        ResolvedRevision::new(commit, commit, TreeId::from_hex(&"5".repeat(40)).unwrap()).unwrap();
    assert!(matches!(
        RepositoryManifest::at_revision(other_tree, &source),
        Err(SnapshotFailure::ObjectMismatch(_))
    ));
}

#[test]
fn path_and_manifest_byte_limits_refuse_closed() {
    let short_path = SnapshotLimits::new(SnapshotLimitSpec {
        max_path_bytes: 2,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        RepositoryManifest::new(
            revision(&"2".repeat(40)),
            vec![entry("long", EntryKind::Blob, &"3".repeat(40))],
            short_path,
        ),
        Err(SnapshotFailure::LimitExceeded {
            limit: "path bytes",
            ..
        })
    ));

    let short_manifest = SnapshotLimits::new(SnapshotLimitSpec {
        max_manifest_bytes: 1,
        ..limit_spec()
    })
    .unwrap();
    assert!(matches!(
        RepositoryManifest::new(
            revision(&"2".repeat(40)),
            vec![entry("a", EntryKind::Blob, &"3".repeat(40))],
            short_manifest,
        ),
        Err(SnapshotFailure::LimitExceeded {
            limit: "manifest bytes",
            ..
        })
    ));
}

fn limit_spec() -> SnapshotLimitSpec {
    SnapshotLimitSpec {
        max_entries: 100,
        max_path_bytes: 100,
        max_manifest_bytes: 10_000,
        max_selected_contents: 100,
        max_content_bytes: 1_000,
        max_total_content_bytes: 10_000,
        max_stdout_bytes: 20_000,
        max_stderr_bytes: 1_000,
    }
}
