//! Event-independent ADR-0719 D-8 changed-path admission facade.

use std::collections::BTreeSet;
use std::env;
use std::process::ExitCode;

use pipeline_admission::{
    APP_PRODUCT_DIRS, BUILD_ROOT_DIRS, base_admission_violations, cargo_entrypoint,
    cargo_manifest_for_crate_path, cargo_manifest_violations, changed_layout_violations,
    draft_dependency_violations, git_change_paths_from_name_status_z, proto_package_violations,
    workspace_draft_dependency_violations,
};
use pipeline_repository_draft::{RepositoryEntryKind, RepositoryRead};
use pipeline_repository_git_draft::GitRepository;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let base = required_sha("OYATIE_LAYOUT_BASE")?;
    let head = required_sha("OYATIE_LAYOUT_HEAD")?;
    let repository = GitRepository;
    let merge_base = repository.merge_base(&base, &head)?;
    let name_status = repository.changed_name_status(&merge_base, &head)?;
    let changes =
        git_change_paths_from_name_status_z(&name_status).map_err(|error| error.message())?;
    let existing_owner_dirs = existing_owner_dirs(&repository, &merge_base)?;
    let workspace_contents = repository.blob_text(&head, "Cargo.toml")?;
    let mut violations = changed_layout_violations(&changes, &existing_owner_dirs);
    violations.extend(workspace_draft_dependency_violations(
        &workspace_contents,
        |visited| reject_indirect_dependency_components(&repository, &head, visited),
    ));
    let mut manifests = Vec::new();
    for path in changes
        .layout_candidates
        .iter()
        .filter(|path| path.ends_with("/Cargo.toml"))
    {
        let contents = repository.blob_text(&head, path)?;
        violations.extend(cargo_manifest_violations(path, &contents));
        violations.extend(draft_dependency_violations(
            path,
            &contents,
            &workspace_contents,
            |visited| reject_indirect_dependency_components(&repository, &head, visited),
        ));
        manifests.push((path.clone(), contents));
    }
    let touched_manifests: BTreeSet<String> = changes
        .occupied
        .iter()
        .filter_map(|path| cargo_manifest_for_crate_path(path))
        .collect();
    let mut added_base_manifests = Vec::new();
    for manifest in touched_manifests {
        let directory = manifest
            .strip_suffix("/Cargo.toml")
            .expect("canonical crate manifest suffix");
        if !repository.directory_exists(&head, directory)? {
            continue;
        }
        let manifest_exists = repository.path_exists(&head, &manifest)?;
        if !manifest_exists {
            violations.push(format!(
                "{directory}: touched face leaf must contain `Cargo.toml` at the head commit"
            ));
        }
        let entrypoint = cargo_entrypoint(&manifest).expect("canonical crate manifest");
        let entrypoint_exists = repository.path_exists(&head, &entrypoint)?;
        if !entrypoint_exists {
            violations.push(format!(
                "{manifest}: canonical entry point `{entrypoint}` is absent at the head commit"
            ));
        }
        if manifest.starts_with("base/core/")
            && manifest_exists
            && entrypoint_exists
            && (!repository.path_exists(&merge_base, &manifest)?
                || !repository.path_exists(&merge_base, &entrypoint)?)
        {
            added_base_manifests.push(manifest);
        }
    }
    for path in changes
        .layout_candidates
        .iter()
        .filter(|path| path.ends_with(".proto"))
    {
        let contents = repository.blob_text(&head, path)?;
        violations.extend(proto_package_violations(path, &contents));
    }
    for base_manifest in added_base_manifests {
        violations.extend(base_admission_violations(
            &base_manifest,
            &manifests,
            &workspace_contents,
        ));
    }
    violations.sort();
    violations.dedup();
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "ADR-0719 D-8 layout refused:\n{}",
            violations.join("\n")
        ))
    }
}

fn reject_indirect_dependency_components(
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

fn required_sha(name: &str) -> Result<String, String> {
    let value = env::var(name).map_err(|_| format!("{name} is required"))?;
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a 40-digit Git object id"));
    }
    Ok(value)
}

fn existing_owner_dirs(
    repository: &impl RepositoryRead,
    merge_base: &str,
) -> Result<BTreeSet<String>, String> {
    let mut owners = BTreeSet::new();
    for root in BUILD_ROOT_DIRS {
        record_existing_dir(repository, merge_base, root, &mut owners)?;
    }
    for product in APP_PRODUCT_DIRS {
        record_existing_dir(
            repository,
            merge_base,
            &format!("app/{product}"),
            &mut owners,
        )?;
    }
    Ok(owners)
}

fn record_existing_dir(
    repository: &impl RepositoryRead,
    merge_base: &str,
    owner: &str,
    existing: &mut BTreeSet<String>,
) -> Result<(), String> {
    if repository.directory_exists(merge_base, owner)? {
        existing.insert(owner.to_owned());
    }
    Ok(())
}
