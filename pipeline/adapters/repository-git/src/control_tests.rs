#![cfg(unix)]

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, Instant};

use pipeline_repository::{
    NoCancellation, ProducerId, RepositoryId, RepositorySnapshot, RevisionId, SnapshotFailure,
    SnapshotRequest, SnapshotSession, ToolId, WorkControl,
};

use crate::GitRepository;
use crate::command::GitCommandRunner;
use crate::repository_test_support::{TestRepository, profile};
use crate::tool::resolve_git_executable;

const BLOCKING_CHILD_SECONDS: u64 = 30;
const TERMINATION_PROOF_SECONDS: u64 = 20;

struct InvocationCancellation(GitCommandRunner);

impl WorkControl for InvocationCancellation {
    fn is_cancelled(&self) -> bool {
        self.0.invocations() >= 3
    }

    fn deadline(&self) -> Option<Instant> {
        None
    }
}

#[test]
fn cancellation_during_tree_enumeration_terminates_the_process() {
    let fixture = TestRepository::create();
    let executable = fixture.path().join("enumeration-git");
    let commit = "1".repeat(40);
    let tree = "2".repeat(40);
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$2\" = \"merge-base\" ]; then\n\
           printf '%s\\n' '{commit}'\n\
           exit 0\n\
         fi\n\
         if [ \"$2\" = \"cat-file\" ]; then\n\
           index=0\n\
           while IFS= read -r line; do\n\
             index=$((index + 1))\n\
             if [ $((index % 2)) -eq 1 ]; then\n\
               printf '%s commit\\n' '{commit}'\n\
             else\n\
               printf '%s tree\\n' '{tree}'\n\
             fi\n\
           done\n\
           exit 0\n\
         fi\n\
         if [ \"$2\" = \"ls-tree\" ]; then\n\
           exec /bin/sleep {BLOCKING_CHILD_SECONDS}\n\
         fi\n\
         exit 2\n"
    );
    write_executable(&executable, &script);
    let repository_id = RepositoryId::new("test/cancel-enumeration").unwrap();
    let adapter = qualified(&fixture, &executable, &repository_id);
    let revision = RevisionId::from_hex(&commit).unwrap();
    let control = InvocationCancellation(adapter.runner.clone());
    let started = Instant::now();

    let result = adapter.capture(
        SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
        &control,
    );

    assert!(matches!(result, Err(SnapshotFailure::Cancelled)));
    assert!(started.elapsed() < Duration::from_secs(TERMINATION_PROOF_SECONDS));
    assert_eq!(adapter.process_count(), 3);
}

#[test]
fn deadline_during_blob_batch_terminates_the_process() {
    let fixture = TestRepository::create();
    fs::write(fixture.path().join("one"), b"content").unwrap();
    let revision = fixture.commit("one");
    let real_git = resolve_git_executable().unwrap();
    let executable = fixture.path().join("batch-git");
    let script = format!(
        "#!/bin/sh\n\
         for argument in \"$@\"; do\n\
           if [ \"$argument\" = \"--batch-command\" ]; then\n\
             exec /bin/sleep {BLOCKING_CHILD_SECONDS}\n\
           fi\n\
         done\n\
         exec {} \"$@\"\n",
        shell_literal(&real_git)
    );
    write_executable(&executable, &script);
    let repository_id = RepositoryId::new("test/deadline-batch").unwrap();
    let adapter = qualified(&fixture, &executable, &repository_id);
    let session = adapter
        .capture(
            SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
            &NoCancellation::without_deadline(),
        )
        .unwrap();
    let content = session.prepared().head().entries()[0].content_id().unwrap();
    let selection = session
        .prepared()
        .select_content(BTreeSet::from([content]))
        .unwrap();
    let deadline = NoCancellation::until(Instant::now() + Duration::from_millis(50));
    let started = Instant::now();

    let result = session.hydrate(selection, &deadline);

    assert!(matches!(result, Err(SnapshotFailure::DeadlineExceeded)));
    assert!(started.elapsed() < Duration::from_secs(TERMINATION_PROOF_SECONDS));
    assert_eq!(adapter.process_count(), 4);
}

#[test]
fn executable_change_after_discovery_refuses_before_repository_work() {
    let fixture = TestRepository::create();
    let executable = fixture.path().join("mutable-git");
    write_executable(
        &executable,
        "#!/bin/sh\nprintf 'git version fixture-1\\n'\n",
    );
    let repository_id = RepositoryId::new("test/tool-change").unwrap();
    let control = NoCancellation::without_deadline();
    let adapter =
        GitRepository::discover(fixture.path(), &executable, repository_id.clone(), &control)
            .unwrap();
    write_executable(
        &executable,
        "#!/bin/sh\nprintf 'git version fixture-2\\n'\n",
    );
    let revision = RevisionId::from_hex(&"1".repeat(40)).unwrap();

    let result = adapter.capture(
        SnapshotRequest::new(repository_id, revision, revision, profile()).unwrap(),
        &control,
    );

    assert!(matches!(result, Err(SnapshotFailure::ObjectMismatch(_))));
    assert_eq!(adapter.process_count(), 1);
}

fn qualified(
    fixture: &TestRepository,
    executable: &Path,
    repository: &RepositoryId,
) -> GitRepository {
    GitRepository::qualified(
        fixture.path(),
        executable,
        repository.clone(),
        ProducerId::new("pipeline-repository-git/control-test").unwrap(),
        ToolId::new("git-control-test").unwrap(),
    )
    .unwrap()
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn shell_literal(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
