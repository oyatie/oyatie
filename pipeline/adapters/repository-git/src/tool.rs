use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use pipeline_repository::{
    EvidenceDigest, ProducerId, RepositoryId, SnapshotFailure, ToolId, WorkControl,
};

use crate::command::{GitCommandRunner, require_success};
use crate::repository::{GitRepository, git_arguments};

const TOOL_FILE_LIMIT: u64 = 256 * 1024 * 1024;
const DISCOVERY_OUTPUT_LIMIT: u64 = 16 * 1024;

impl GitRepository {
    pub fn current(
        repository: RepositoryId,
        control: &dyn WorkControl,
    ) -> Result<Self, SnapshotFailure> {
        let root = env::current_dir()
            .map_err(|error| SnapshotFailure::io("resolve repository directory", error))?;
        let executable = resolve_git_executable()?;
        Self::discover(root, executable, repository, control)
    }

    pub fn discover(
        root: impl AsRef<Path>,
        executable: impl AsRef<Path>,
        repository: RepositoryId,
        control: &dyn WorkControl,
    ) -> Result<Self, SnapshotFailure> {
        control.checkpoint()?;
        let root = fs::canonicalize(root.as_ref())
            .map_err(|error| SnapshotFailure::io("canonicalize repository directory", error))?;
        let executable = fs::canonicalize(executable.as_ref())
            .map_err(|error| SnapshotFailure::io("canonicalize Git executable", error))?;
        let runner = GitCommandRunner::new(executable, root);
        let binary_digest = digest_file(runner.executable(), control)?;
        let output = runner.run(
            "identify Git",
            &git_arguments(["--version"]),
            Vec::new(),
            DISCOVERY_OUTPUT_LIMIT,
            DISCOVERY_OUTPUT_LIMIT,
            control,
        )?;
        let version = require_success("identify Git", output)?;
        if version.is_empty() || version.last() != Some(&b'\n') || !version.is_ascii() {
            return Err(SnapshotFailure::MalformedOutput(
                "Git version output is not one complete ASCII line".to_owned(),
            ));
        }
        let verified_digest = digest_file(runner.executable(), control)?;
        if binary_digest != verified_digest {
            return Err(SnapshotFailure::ObjectMismatch(
                "Git executable changed while its identity was observed".to_owned(),
            ));
        }
        let version_digest =
            EvidenceDigest::of_bytes(b"pipeline-repository-git-version-v1", &version);
        let tool = ToolId::new(format!(
            "git-cli-v1:version-{}:binary-{}",
            version_digest.to_hex(),
            binary_digest.to_hex()
        ))?;
        let runner = runner.pin_executable(binary_digest);
        Ok(Self {
            repository,
            producer: ProducerId::new("pipeline-repository-git/v1")?,
            tool,
            runner,
        })
    }

    pub(crate) fn qualified(
        root: impl AsRef<Path>,
        executable: impl AsRef<Path>,
        repository: RepositoryId,
        producer: ProducerId,
        tool: ToolId,
    ) -> Result<Self, SnapshotFailure> {
        let root = fs::canonicalize(root.as_ref())
            .map_err(|error| SnapshotFailure::io("canonicalize repository directory", error))?;
        let executable = fs::canonicalize(executable.as_ref())
            .map_err(|error| SnapshotFailure::io("canonicalize Git executable", error))?;
        if !executable.is_file() {
            return Err(SnapshotFailure::ToolUnavailable(format!(
                "{} is not a file",
                executable.display()
            )));
        }
        Ok(Self {
            repository,
            producer,
            tool,
            runner: GitCommandRunner::new(executable, root),
        })
    }
}

pub(crate) fn resolve_git_executable() -> Result<PathBuf, SnapshotFailure> {
    let path = env::var_os("PATH")
        .ok_or_else(|| SnapshotFailure::ToolUnavailable("PATH is not set".to_owned()))?;
    for directory in env::split_paths(&path) {
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join("git");
        if candidate.is_file() {
            return fs::canonicalize(&candidate)
                .map_err(|error| SnapshotFailure::io("canonicalize Git executable", error));
        }
    }
    Err(SnapshotFailure::ToolUnavailable(
        "no absolute PATH entry contains Git".to_owned(),
    ))
}

pub(crate) fn digest_file(
    path: &Path,
    control: &dyn WorkControl,
) -> Result<EvidenceDigest, SnapshotFailure> {
    let metadata =
        fs::metadata(path).map_err(|error| SnapshotFailure::io("inspect Git executable", error))?;
    if metadata.len() > TOOL_FILE_LIMIT {
        return Err(SnapshotFailure::LimitExceeded {
            limit: "Git executable bytes",
            maximum: TOOL_FILE_LIMIT,
            observed: metadata.len(),
        });
    }
    let mut file =
        File::open(path).map_err(|error| SnapshotFailure::io("open Git executable", error))?;
    let capacity = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    let mut contents = Vec::with_capacity(capacity.min(TOOL_FILE_LIMIT as usize));
    let mut observed = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        control.checkpoint()?;
        let read = file
            .read(&mut buffer)
            .map_err(|error| SnapshotFailure::io("read Git executable", error))?;
        if read == 0 {
            break;
        }
        observed += read as u64;
        if observed > TOOL_FILE_LIMIT {
            return Err(SnapshotFailure::LimitExceeded {
                limit: "Git executable bytes",
                maximum: TOOL_FILE_LIMIT,
                observed,
            });
        }
        contents.extend_from_slice(&buffer[..read]);
    }
    if observed != metadata.len() {
        return Err(SnapshotFailure::ObjectMismatch(format!(
            "Git executable length changed while reading: expected {}, observed {observed}",
            metadata.len()
        )));
    }
    Ok(EvidenceDigest::of_bytes(
        b"pipeline-repository-git-executable-v1",
        &contents,
    ))
}
