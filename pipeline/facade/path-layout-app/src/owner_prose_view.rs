use std::fs;
use std::path::Path;

use pipeline_admission::{
    OwnerProseQualification, OwnerProseRepositoryBinding, OwnerProseRevision,
    OwnerProseRevisionBinding, QualifiedOwnerProseView, qualify_owner_prose,
};
use pipeline_repository_draft::{RepositoryEntryKind, RepositoryRead};

pub(super) fn qualify_view(
    repository: &impl RepositoryRead,
    source_commit: &str,
    candidate_commit: &str,
    path: &Path,
) -> Result<QualifiedOwnerProseView, String> {
    let bytes = external_view_bytes(repository, path)?;
    let observed = OwnerProseRepositoryBinding {
        identity: repository
            .repository_identity()
            .map_err(|error| unknown(format!("repository identity unavailable: {error}")))?,
        source: revision_binding(repository, source_commit)?,
        candidate: revision_binding(repository, candidate_commit)?,
    };
    let qualification = qualify_owner_prose(&bytes, &observed, |revision, blob_path| {
        let commit = match revision {
            OwnerProseRevision::Source => source_commit,
            OwnerProseRevision::Candidate => candidate_commit,
        };
        match repository.entry_kind(commit, blob_path)? {
            None => Ok(None),
            Some(RepositoryEntryKind::Blob | RepositoryEntryKind::ExecutableBlob) => {
                repository.blob_bytes(commit, blob_path).map(Some)
            }
            Some(kind) => Err(format!(
                "{blob_path} at {commit} must be a regular Git blob, got {kind:?}"
            )),
        }
    });
    match qualification {
        OwnerProseQualification::Ready(view) => Ok(*view),
        OwnerProseQualification::Unknown(refusals) => Err(unknown(
            refusals
                .iter()
                .map(|refusal| refusal.message())
                .collect::<Vec<_>>()
                .join("\n"),
        )),
    }
}

fn revision_binding(
    repository: &impl RepositoryRead,
    revision: &str,
) -> Result<OwnerProseRevisionBinding, String> {
    let commit = repository
        .resolve_commit(revision)
        .map_err(|error| unknown(format!("commit resolution failed: {error}")))?;
    let tree = repository
        .tree_id(&commit)
        .map_err(|error| unknown(format!("tree resolution failed: {error}")))?;
    Ok(OwnerProseRevisionBinding { commit, tree })
}

fn external_view_bytes(repository: &impl RepositoryRead, path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err(unknown("view path must be absolute"));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| unknown(format!("view path unavailable: {error}")))?;
    if metadata.file_type().is_symlink() {
        return Err(unknown("view path must not be a symlink"));
    }
    if !metadata.is_file() {
        return Err(unknown("view path must be one regular file"));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| unknown(format!("canonicalize view path: {error}")))?;
    let repository_root = repository
        .working_tree_root()
        .map_err(|error| unknown(format!("working tree root unavailable: {error}")))?;
    if canonical.starts_with(repository_root) {
        return Err(unknown("view path must remain outside the repository"));
    }
    fs::read(&canonical).map_err(|error| unknown(format!("read view path: {error}")))
}

fn unknown(detail: impl AsRef<str>) -> String {
    format!("owner prose view Unknown:\n{}", detail.as_ref())
}
