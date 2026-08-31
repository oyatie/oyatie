use std::collections::BTreeSet;
use std::fs;
use std::time::Instant;

use pipeline_repository::{
    NoCancellation, ObjectAlgorithm, RepositoryId, RepositoryPath, RepositorySnapshot, RevisionId,
    SnapshotFailure, SnapshotRequest, SnapshotSession, WorkControl,
};

use crate::GitRepository;
use crate::repository_test_support::{TestRepository, profile};
use crate::tool::resolve_git_executable;

#[test]
fn one_capture_and_content_batch_use_constant_processes() {
    let fixture = TestRepository::create();
    fs::write(fixture.path().join("Cargo.toml"), b"[workspace]\n").unwrap();
    let base = fixture.commit("base");
    fs::create_dir(fixture.path().join("src")).unwrap();
    fs::write(fixture.path().join("src/lib.rs"), b"pub fn value() {}\n").unwrap();
    for index in 0..64 {
        fs::write(
            fixture.path().join(format!("src/value_{index:02}.rs")),
            format!("pub const VALUE_{index}: usize = {index};\n"),
        )
        .unwrap();
    }
    let head = fixture.commit("head");
    let repository_id = RepositoryId::new("test/repository").unwrap();
    let control = NoCancellation::without_deadline();
    let adapter = GitRepository::discover(
        fixture.path(),
        resolve_git_executable().unwrap(),
        repository_id.clone(),
        &control,
    )
    .unwrap();
    let session = adapter
        .capture(
            SnapshotRequest::new(repository_id, base, head, profile()).unwrap(),
            &control,
        )
        .unwrap();
    let content = session
        .prepared()
        .head()
        .entry(&RepositoryPath::from_utf8("src/lib.rs").unwrap())
        .unwrap()
        .content_id()
        .unwrap();
    let cargo = session
        .prepared()
        .head()
        .entry(&RepositoryPath::from_utf8("Cargo.toml").unwrap())
        .unwrap()
        .content_id()
        .unwrap();
    let source_directory = RepositoryPath::from_utf8("src").unwrap();
    let selected = session
        .prepared()
        .head()
        .files_under(&source_directory)
        .filter_map(|entry| entry.content_id())
        .chain([cargo])
        .collect();
    let selection = session.prepared().select_content(selected).unwrap();
    assert_eq!(selection.ids().len(), 66);
    let hydrated = session.hydrate(selection, &control).unwrap();

    assert_eq!(
        hydrated.content(content),
        Some(b"pub fn value() {}\n".as_slice())
    );
    assert_eq!(adapter.process_count(), 6);
}

#[test]
fn identical_inputs_produce_identical_receipts() {
    let fixture = TestRepository::create();
    fs::write(fixture.path().join("one"), b"same").unwrap();
    let revision = fixture.commit("one");
    let repository_id = RepositoryId::new("test/repository").unwrap();
    let adapter = fixture.adapter(&repository_id);
    let control = NoCancellation::without_deadline();
    let request =
        || SnapshotRequest::new(repository_id.clone(), revision, revision, profile()).unwrap();
    let first = adapter.capture(request(), &control).unwrap();
    let first_selection = first.prepared().select_content(BTreeSet::new()).unwrap();
    let first = first.hydrate(first_selection, &control).unwrap();
    let second = adapter.capture(request(), &control).unwrap();
    let second_selection = second.prepared().select_content(BTreeSet::new()).unwrap();
    let second = second.hydrate(second_selection, &control).unwrap();

    assert_eq!(first.receipt(), second.receipt());
    assert_eq!(adapter.process_count(), 6);
}

#[test]
fn cancellation_before_capture_spawns_no_process() {
    struct Cancelled;

    impl WorkControl for Cancelled {
        fn is_cancelled(&self) -> bool {
            true
        }

        fn deadline(&self) -> Option<Instant> {
            None
        }
    }

    let fixture = TestRepository::create();
    fs::write(fixture.path().join("one"), b"same").unwrap();
    let revision = fixture.commit("one");
    let repository_id = RepositoryId::new("test/repository").unwrap();
    let adapter = fixture.adapter(&repository_id);
    let result = adapter.capture(
        SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
        &Cancelled,
    );

    assert!(matches!(result, Err(SnapshotFailure::Cancelled)));
    assert_eq!(adapter.process_count(), 0);
}

#[test]
fn sha256_repository_identity_survives_capture_and_hydration() {
    let fixture = TestRepository::create_sha256();
    fs::write(fixture.path().join("one"), b"sha256 content").unwrap();
    let revision = fixture.commit("one");
    assert_eq!(revision.algorithm(), ObjectAlgorithm::Sha256);
    let repository_id = RepositoryId::new("test/repository-sha256").unwrap();
    let adapter = fixture.adapter(&repository_id);
    let control = NoCancellation::without_deadline();
    let session = adapter
        .capture(
            SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
            &control,
        )
        .unwrap();
    let content = session
        .prepared()
        .head()
        .entry(&RepositoryPath::from_utf8("one").unwrap())
        .unwrap()
        .content_id()
        .unwrap();
    let selection = session
        .prepared()
        .select_content(BTreeSet::from([content]))
        .unwrap();
    let hydrated = session.hydrate(selection, &control).unwrap();

    assert_eq!(
        hydrated.content(content),
        Some(b"sha256 content".as_slice())
    );
    assert_eq!(
        hydrated.receipt().head().algorithm(),
        ObjectAlgorithm::Sha256
    );
    assert_eq!(adapter.process_count(), 4);
}

#[cfg(unix)]
#[test]
fn non_utf8_path_is_preserved_without_global_projection() {
    let fixture = TestRepository::create();
    let raw_path = b"raw-\xff".to_vec();
    let blob = fixture.git_with_input(&["hash-object", "-w", "--stdin"], b"raw");
    let blob = String::from_utf8(blob).unwrap().trim().to_owned();
    let mut tree_record = format!("100644 blob {blob}\t").into_bytes();
    tree_record.extend_from_slice(&raw_path);
    tree_record.push(0);
    let tree = fixture.git_with_input(&["mktree", "-z"], &tree_record);
    let tree = String::from_utf8(tree).unwrap().trim().to_owned();
    let commit = fixture.git(["commit-tree", &tree, "-m", "raw path"]);
    let revision = RevisionId::from_hex(String::from_utf8(commit).unwrap().trim()).unwrap();
    let repository_id = RepositoryId::new("test/repository-raw-path").unwrap();
    let adapter = fixture.adapter(&repository_id);
    let control = NoCancellation::without_deadline();
    let session = adapter
        .capture(
            SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
            &control,
        )
        .unwrap();

    assert!(
        session
            .prepared()
            .head()
            .entries()
            .iter()
            .any(|entry| entry.path().as_bytes() == raw_path)
    );
}
