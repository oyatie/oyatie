use std::collections::BTreeSet;
use std::io::Write as _;
use std::time::{Duration, Instant};

use pipeline_repository::{
    ContentId, NoCancellation, ObjectId, ProfileId, RepositoryId, RepositorySnapshot, RevisionId,
    SchemaId, SnapshotLimitSpec, SnapshotLimits, SnapshotProfile, SnapshotRequest, SnapshotSession,
};

use crate::GitRepository;
use crate::repository_test_support::TestRepository;
use crate::tool::resolve_git_executable;

const ENTRY_COUNT: usize = 250_000;
const SAMPLE_COUNT: usize = 20;

#[test]
#[ignore = "manual quarter-million-entry corpus qualification"]
fn quarter_million_entries_keep_external_work_constant() {
    let fixture = TestRepository::create();
    let blob = text(fixture.git_with_input(&["hash-object", "-w", "--stdin"], b"x"));
    let mut tree_input = Vec::with_capacity(ENTRY_COUNT * 80);
    for index in 0..ENTRY_COUNT {
        write!(tree_input, "100644 blob {blob}\tentry-{index:06}\0").unwrap();
    }
    let tree = text(fixture.git_with_input(&["mktree", "-z"], &tree_input));
    drop(tree_input);
    let commit = text(fixture.git(["commit-tree", tree.as_str(), "-m", "scale"]));
    let revision = RevisionId::from_hex(&commit).unwrap();
    let repository_id = RepositoryId::new("test/quarter-million").unwrap();
    let control = NoCancellation::without_deadline();
    let adapter = GitRepository::discover(
        fixture.path(),
        resolve_git_executable().unwrap(),
        repository_id.clone(),
        &control,
    )
    .unwrap();
    let request = SnapshotRequest::new(repository_id, revision, revision, scale_profile()).unwrap();
    let content = ContentId::from_object_id(ObjectId::from_hex(&blob).unwrap());
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let session = adapter.capture(request.clone(), &control).unwrap();
        assert_eq!(session.prepared().head().entries().len(), ENTRY_COUNT);
        let selection = session
            .prepared()
            .select_content(BTreeSet::from([content]))
            .unwrap();
        let snapshot = session.hydrate(selection, &control).unwrap();
        assert_eq!(snapshot.content(content), Some(b"x".as_slice()));
        samples.push(started.elapsed());
    }
    samples.sort_unstable();

    eprintln!(
        "quarter-million capture+hydrate samples={SAMPLE_COUNT} p50={:?} p95={:?}",
        percentile(&samples, 50),
        percentile(&samples, 95)
    );
    assert_eq!(adapter.process_count(), (SAMPLE_COUNT * 4 + 1) as u64);
}

fn scale_profile() -> SnapshotProfile {
    SnapshotProfile::new(
        ProfileId::new("quarter-million-tree-v1").unwrap(),
        SchemaId::new("repository-snapshot-v1").unwrap(),
        SnapshotLimits::new(SnapshotLimitSpec {
            max_entries: ENTRY_COUNT as u64,
            max_path_bytes: 64,
            max_manifest_bytes: 32 * 1024 * 1024,
            max_selected_contents: 1,
            max_content_bytes: 1024,
            max_total_content_bytes: 1024,
            max_stdout_bytes: 64 * 1024 * 1024,
            max_stderr_bytes: 64 * 1024,
        })
        .unwrap(),
    )
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap().trim().to_owned()
}

fn percentile(samples: &[Duration], percent: usize) -> Duration {
    let rank = (samples.len() * percent).div_ceil(100);
    samples[rank.saturating_sub(1)]
}
