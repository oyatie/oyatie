use std::collections::BTreeSet;

use pipeline_admission::{
    ALLOWED_ROOT_DIRS, APP_PRODUCT_DIRS, BUILD_ROOT_DIRS, cargo_config_violations,
    file_budget_violations, is_capability_root,
};
use pipeline_repository_draft::{RepositoryEntryKind, RepositoryRead};

#[derive(Default)]
pub(super) struct OwnerTreeState {
    pub(super) live: BTreeSet<String>,
    pub(super) complete: BTreeSet<String>,
    pub(super) lawful: BTreeSet<String>,
}

pub(super) fn owner_tree_state(
    repository: &impl RepositoryRead,
    commit: &str,
) -> Result<OwnerTreeState, String> {
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
        if !repository.directory_exists(commit, &owner)? {
            continue;
        }
        state.live.insert(owner.clone());
        if owner_has_complete_core(repository, commit, &owner)? {
            state.complete.insert(owner.clone());
        }
        if owner_has_complete_law(repository, commit, &owner)? {
            state.lawful.insert(owner);
        }
    }
    Ok(state)
}

fn owner_has_complete_law(
    repository: &impl RepositoryRead,
    commit: &str,
    owner: &str,
) -> Result<bool, String> {
    for law in ["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"] {
        if !regular_blob(repository.entry_kind(commit, &format!("{owner}/{law}"))?) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn owner_has_complete_core(
    repository: &impl RepositoryRead,
    commit: &str,
    owner: &str,
) -> Result<bool, String> {
    let prefix = format!("{owner}/core/");
    let files: BTreeSet<String> = repository
        .files_under(commit, &format!("{owner}/core"))?
        .into_iter()
        .collect();
    for manifest in files.iter().filter(|path| {
        path.strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix("/Cargo.toml"))
            .is_some_and(|leaf| !leaf.is_empty() && !leaf.contains('/'))
    }) {
        let directory = manifest
            .strip_suffix("/Cargo.toml")
            .expect("filtered manifest suffix");
        let entrypoint = format!("{directory}/src/lib.rs");
        if files.contains(&entrypoint)
            && regular_blob(repository.entry_kind(commit, manifest)?)
            && regular_blob(repository.entry_kind(commit, &entrypoint)?)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn repository_cargo_config_violations(
    repository: &impl RepositoryRead,
    head: &str,
) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    for path in [".cargo/config.toml", ".cargo/config"] {
        match repository.entry_kind(head, path)? {
            None => {}
            Some(kind) if regular_blob(Some(kind)) => {
                let contents = repository.blob_text(head, path)?;
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
    repository: &impl RepositoryRead,
    head: &str,
    candidates: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    for path in candidates {
        match repository.entry_kind(head, path)? {
            Some(kind) if regular_blob(Some(kind)) => {
                let contents = repository.blob_bytes(head, path)?;
                violations.extend(file_budget_violations(path, &contents));
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

pub(super) fn regular_blob(kind: Option<RepositoryEntryKind>) -> bool {
    matches!(
        kind,
        Some(RepositoryEntryKind::Blob | RepositoryEntryKind::ExecutableBlob)
    )
}

pub(super) fn reject_indirect_dependency_components(
    repository: &impl RepositoryRead,
    head: &str,
    visited: &[String],
) -> Result<(), String> {
    for path in visited {
        match repository.entry_kind(head, path)? {
            Some(RepositoryEntryKind::Symlink) => {
                return Err(format!("tracked symlink component `{path}` is forbidden"));
            }
            Some(RepositoryEntryKind::Gitlink) => {
                return Err(format!("tracked gitlink component `{path}` is forbidden"));
            }
            _ => {}
        }
    }
    Ok(())
}
