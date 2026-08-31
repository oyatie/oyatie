use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_repository::{
    ProducerId, ProfileId, RepositoryId, RevisionId, SchemaId, SnapshotLimitSpec, SnapshotLimits,
    SnapshotProfile, ToolId,
};

use crate::GitRepository;
use crate::tool::resolve_git_executable;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TestRepository(PathBuf);

impl TestRepository {
    pub(crate) fn create() -> Self {
        Self::create_with_init(["init", "--quiet"])
    }

    pub(crate) fn create_sha256() -> Self {
        Self::create_with_init(["init", "--quiet", "--object-format=sha256"])
    }

    fn create_with_init<const N: usize>(arguments: [&str; N]) -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "oyatie-repository-snapshot-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        let repository = Self(path);
        repository.git(arguments);
        repository.git(["config", "user.name", "Snapshot Test"]);
        repository.git(["config", "user.email", "snapshot@example.invalid"]);
        repository
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn git<const N: usize>(&self, arguments: [&str; N]) -> Vec<u8> {
        let output = Command::new("git")
            .current_dir(&self.0)
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    pub(crate) fn git_with_input(&self, arguments: &[&str], input: &[u8]) -> Vec<u8> {
        let mut child = Command::new("git")
            .current_dir(&self.0)
            .args(arguments)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    pub(crate) fn commit(&self, message: &str) -> RevisionId {
        self.git(["add", "."]);
        self.git(["commit", "--quiet", "-m", message]);
        let output = self.git(["rev-parse", "HEAD"]);
        RevisionId::from_hex(String::from_utf8(output).unwrap().trim()).unwrap()
    }

    pub(crate) fn adapter(&self, repository: &RepositoryId) -> GitRepository {
        GitRepository::qualified(
            &self.0,
            resolve_git_executable().unwrap(),
            repository.clone(),
            ProducerId::new("pipeline-repository-git/test").unwrap(),
            ToolId::new("git-test-qualified").unwrap(),
        )
        .unwrap()
    }
}

impl Drop for TestRepository {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub(crate) fn profile() -> SnapshotProfile {
    SnapshotProfile::new(
        ProfileId::new("path-layout-v1").unwrap(),
        SchemaId::new("repository-snapshot-v1").unwrap(),
        SnapshotLimits::new(SnapshotLimitSpec {
            max_entries: 10_000,
            max_path_bytes: 4_096,
            max_manifest_bytes: 16 * 1024 * 1024,
            max_selected_contents: 10_000,
            max_content_bytes: 8 * 1024 * 1024,
            max_total_content_bytes: 32 * 1024 * 1024,
            max_stdout_bytes: 64 * 1024 * 1024,
            max_stderr_bytes: 64 * 1024,
        })
        .unwrap(),
    )
}
