use std::collections::{BTreeMap, BTreeSet};

use pipeline_admission::{
    ALLOWED_ROOT_DIRS, APP_PRODUCT_DIRS, BUILD_ROOT_DIRS, cargo_config_violations,
    file_budget_violations, is_capability_root,
};
use pipeline_repository::{
    EntryKind, HydratedSnapshot, RepositoryManifest, RepositoryPath, SnapshotFailure,
};

pub(super) struct RepositoryView<'a> {
    manifest: &'a RepositoryManifest,
    snapshot: &'a HydratedSnapshot,
}

impl<'a> RepositoryView<'a> {
    pub(super) const fn new(
        manifest: &'a RepositoryManifest,
        snapshot: &'a HydratedSnapshot,
    ) -> Self {
        Self { manifest, snapshot }
    }

    pub(super) fn blob_text(&self, path: &str) -> Result<String, String> {
        std::str::from_utf8(self.blob_bytes(path)?)
            .map(str::to_owned)
            .map_err(|_| format!("repository content is non-UTF-8 for {path}"))
    }

    pub(super) fn blob_bytes(&self, path: &str) -> Result<&[u8], String> {
        let path = canonical_path(path)?;
        let entry = self
            .manifest
            .entry(&path)
            .ok_or_else(|| format!("repository path {path} is absent"))?;
        if !entry.kind().is_regular_blob() {
            return Err(format!(
                "repository path {path} must be a regular Git blob, got {:?}",
                entry.kind()
            ));
        }
        let content = entry
            .content_id()
            .ok_or_else(|| format!("repository path {path} has no blob content"))?;
        self.snapshot
            .content(content)
            .ok_or_else(|| format!("repository content {content} for {path} was not selected"))
    }

    pub(super) fn entry_kind(&self, path: &str) -> Result<Option<EntryKind>, String> {
        let path = canonical_path(path)?;
        Ok(self.manifest.entry(&path).map(|entry| entry.kind()))
    }
}

#[derive(Default)]
pub(super) struct OwnerTreeState {
    pub(super) live: BTreeSet<String>,
    pub(super) complete: BTreeSet<String>,
}

pub(super) fn owner_tree_state(manifest: &RepositoryManifest) -> Result<OwnerTreeState, String> {
    let mut state = OwnerTreeState::default();
    let capability_owners = ALLOWED_ROOT_DIRS
        .iter()
        .chain(BUILD_ROOT_DIRS)
        .copied()
        .filter(|owner| *owner == "base" || is_capability_root(owner))
        .map(str::to_owned);
    let app_owners = APP_PRODUCT_DIRS
        .iter()
        .map(|product| format!("app/{product}"));
    for owner in capability_owners.chain(app_owners) {
        let owner_path = canonical_path(&owner)?;
        if !manifest
            .entry(&owner_path)
            .is_some_and(|entry| entry.kind() == EntryKind::Tree)
        {
            continue;
        }
        state.live.insert(owner.clone());
        if owner_has_complete_core(manifest, &owner)? {
            state.complete.insert(owner.clone());
        }
    }
    Ok(state)
}

fn owner_has_complete_core(
    repository_manifest: &RepositoryManifest,
    owner: &str,
) -> Result<bool, String> {
    let prefix = format!("{owner}/core/");
    let directory = canonical_path(&format!("{owner}/core"))?;
    let files: BTreeSet<String> = repository_manifest
        .files_under(&directory)
        .map(|entry| {
            entry
                .path()
                .as_utf8()
                .map(str::to_owned)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<_, _>>()?;
    for manifest_path in files.iter().filter(|path| {
        path.strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix("/Cargo.toml"))
            .is_some_and(|leaf| !leaf.is_empty() && !leaf.contains('/'))
    }) {
        let directory = manifest_path
            .strip_suffix("/Cargo.toml")
            .expect("filtered manifest suffix");
        let entrypoint = format!("{directory}/src/lib.rs");
        if files.contains(&entrypoint)
            && regular_blob(entry_kind(repository_manifest, manifest_path)?)
            && regular_blob(entry_kind(repository_manifest, &entrypoint)?)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn repository_cargo_config_violations(
    repository: &RepositoryView<'_>,
) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    for path in [".cargo/config.toml", ".cargo/config"] {
        match repository.entry_kind(path)? {
            None => {}
            Some(kind) if regular_blob(Some(kind)) => {
                let contents = repository.blob_text(path)?;
                violations.extend(cargo_config_violations(path, &contents));
            }
            Some(kind) => violations.push(format!(
                "{path}: Cargo configuration must be a regular Git blob, got {kind:?}"
            )),
        }
    }
    Ok(violations)
}

pub(super) fn live_candidate_violations(
    repository: &RepositoryView<'_>,
    candidates: &BTreeSet<String>,
    exact_rename_sources: &BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    for path in candidates {
        match repository.entry_kind(path)? {
            Some(kind) if regular_blob(Some(kind)) => {
                let contents = repository.blob_bytes(path)?;
                if !inherits_existing_budget_debt(path, contents, exact_rename_sources) {
                    violations.extend(file_budget_violations(path, contents));
                }
            }
            Some(kind) => violations.push(format!(
                "{path}: live changed content must be a regular Git blob, got {kind:?}"
            )),
            None => violations.push(format!(
                "{path}: live changed path is absent at the head commit"
            )),
        }
    }
    Ok(violations)
}

fn inherits_existing_budget_debt(
    destination: &str,
    contents: &[u8],
    exact_rename_sources: &BTreeMap<String, String>,
) -> bool {
    exact_rename_sources
        .get(destination)
        .is_some_and(|source| !file_budget_violations(source, contents).is_empty())
}

pub(super) fn regular_blob(kind: Option<EntryKind>) -> bool {
    matches!(kind, Some(EntryKind::Blob | EntryKind::ExecutableBlob))
}

pub(super) fn reject_indirect_dependency_components(
    manifest: &RepositoryManifest,
    visited: &[String],
) -> Result<(), String> {
    for path in visited {
        match entry_kind(manifest, path)? {
            Some(EntryKind::Symlink) => {
                return Err(format!("tracked symlink component `{path}` is forbidden"));
            }
            Some(EntryKind::Gitlink) => {
                return Err(format!("tracked gitlink component `{path}` is forbidden"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn entry_kind(manifest: &RepositoryManifest, path: &str) -> Result<Option<EntryKind>, String> {
    let path = canonical_path(path)?;
    Ok(manifest.entry(&path).map(|entry| entry.kind()))
}

fn canonical_path(path: &str) -> Result<RepositoryPath, String> {
    RepositoryPath::from_utf8(path).map_err(|error| error.to_string())
}

pub(super) fn snapshot_error(error: SnapshotFailure) -> String {
    error.to_string()
}
