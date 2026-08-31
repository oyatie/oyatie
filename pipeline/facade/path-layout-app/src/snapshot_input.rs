use std::collections::BTreeSet;

use pipeline_admission::GitChangePaths;
use pipeline_repository::{
    ContentId, PreparedSnapshot, ProfileId, RepositoryDelta, RepositoryPath, SchemaId,
    SnapshotLimitSpec, SnapshotLimits, SnapshotProfile,
};

use crate::repository_checks::snapshot_error;

pub(crate) fn layout_profile() -> Result<SnapshotProfile, String> {
    let limits = SnapshotLimits::new(SnapshotLimitSpec {
        max_entries: 500_000,
        max_path_bytes: 4_096,
        max_manifest_bytes: 128 * 1024 * 1024,
        max_selected_contents: 50_000,
        max_content_bytes: 8 * 1024 * 1024,
        max_total_content_bytes: 128 * 1024 * 1024,
        max_stdout_bytes: 160 * 1024 * 1024,
        max_stderr_bytes: 1024 * 1024,
    })
    .map_err(snapshot_error)?;
    Ok(SnapshotProfile::new(
        ProfileId::new("path-layout-admission-v1").map_err(|error| error.to_string())?,
        SchemaId::new("repository-snapshot-v1").map_err(|error| error.to_string())?,
        limits,
    ))
}

pub(crate) fn admission_changes(delta: &RepositoryDelta) -> Result<GitChangePaths, String> {
    let mut changes = GitChangePaths::default();
    for entry in delta.entries() {
        let path = entry
            .path()
            .as_utf8()
            .map_err(|error| error.to_string())?
            .to_owned();
        changes.occupied.insert(path.clone());
        if entry.after().is_some() {
            changes.layout_candidates.insert(path);
        }
    }
    for (destination, source) in delta.exact_moves() {
        changes.exact_rename_sources.insert(
            destination
                .as_utf8()
                .map_err(|error| error.to_string())?
                .to_owned(),
            source
                .as_utf8()
                .map_err(|error| error.to_string())?
                .to_owned(),
        );
    }
    Ok(changes)
}

pub(crate) fn selected_content(
    prepared: &PreparedSnapshot,
    changes: &GitChangePaths,
) -> Result<BTreeSet<ContentId>, String> {
    let mut selected = BTreeSet::new();
    for path in changes.layout_candidates.iter().map(String::as_str).chain([
        "Cargo.toml",
        ".cargo/config.toml",
        ".cargo/config",
    ]) {
        let path = RepositoryPath::from_utf8(path).map_err(|error| error.to_string())?;
        if let Some(content) = prepared
            .head()
            .entry(&path)
            .filter(|entry| entry.kind().is_regular_blob())
            .and_then(|entry| entry.content_id())
        {
            selected.insert(content);
        }
    }
    Ok(selected)
}
