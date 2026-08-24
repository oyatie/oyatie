//! Event-independent ADR-0719 D-8 changed-path admission facade.

use std::collections::BTreeSet;
use std::process::ExitCode;

use pipeline_admission::{
    base_admission_violations, cargo_entrypoint, cargo_manifest_for_crate_path,
    cargo_manifest_violations, changed_layout_violations, draft_dependency_violations,
    git_change_paths_from_name_status_z, owner_core_regression_violations,
    owner_law_regression_violations, proto_package_violations,
    workspace_draft_dependency_violations, workspace_membership_violations,
};
use pipeline_repository_draft::RepositoryRead;
use pipeline_repository_git_draft::GitRepository;

mod repository_checks;

use repository_checks::{
    live_candidate_violations, owner_tree_state, regular_blob,
    reject_indirect_dependency_components, repository_cargo_config_violations,
};

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
    let (base, head) = required_shas()?;
    let repository = GitRepository;
    let merge_base = repository.merge_base(&base, &head)?;
    let name_status = repository.changed_name_status(&merge_base, &head)?;
    let changes =
        git_change_paths_from_name_status_z(&name_status).map_err(|error| error.message())?;
    let owners_before = owner_tree_state(&repository, &merge_base)?;
    let owners_after = owner_tree_state(&repository, &head)?;
    let workspace_contents = repository.blob_text(&head, "Cargo.toml")?;
    let mut violations = changed_layout_violations(&changes, &owners_before.live);
    violations.extend(owner_core_regression_violations(
        &changes,
        &owners_before.complete,
        &owners_after.live,
        &owners_after.complete,
    ));
    violations.extend(owner_law_regression_violations(
        &changes,
        &owners_before.complete,
        &owners_after.live,
        &owners_after.complete,
        &owners_after.lawful,
    ));
    violations.extend(workspace_membership_violations(&workspace_contents));
    violations.extend(workspace_draft_dependency_violations(
        &workspace_contents,
        |visited| reject_indirect_dependency_components(&repository, &head, visited),
    ));
    violations.extend(repository_cargo_config_violations(&repository, &head)?);
    violations.extend(live_candidate_violations(
        &repository,
        &head,
        &changes.layout_candidates,
    )?);
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
        let manifest_kind = repository.entry_kind(&head, &manifest)?;
        let manifest_exists = manifest_kind.is_some();
        let manifest_is_blob = regular_blob(manifest_kind);
        match manifest_kind {
            None => violations.push(format!(
                "{directory}: touched face leaf must contain `Cargo.toml` at the head commit"
            )),
            Some(kind) if !regular_blob(Some(kind)) => violations.push(format!(
                "{manifest}: canonical manifest must be a regular Git blob, got {kind:?}"
            )),
            Some(_) => {}
        }
        let entrypoint = cargo_entrypoint(&manifest).expect("canonical crate manifest");
        let entrypoint_kind = repository.entry_kind(&head, &entrypoint)?;
        let entrypoint_exists = entrypoint_kind.is_some();
        let entrypoint_is_blob = regular_blob(entrypoint_kind);
        match entrypoint_kind {
            None => violations.push(format!(
                "{manifest}: canonical entry point `{entrypoint}` is absent at the head commit"
            )),
            Some(kind) if !regular_blob(Some(kind)) => violations.push(format!(
                "{manifest}: canonical entry point `{entrypoint}` must be a regular Git blob, got {kind:?}"
            )),
            Some(_) => {}
        }
        if manifest.starts_with("base/core/")
            && manifest_exists
            && manifest_is_blob
            && entrypoint_exists
            && entrypoint_is_blob
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

fn required_shas() -> Result<(String, String), String> {
    let mut arguments = std::env::args().skip(1);
    let base = arguments
        .next()
        .ok_or_else(|| "usage: pipeline-path-layout-app <base-sha> <head-sha>".to_owned())?;
    let head = arguments
        .next()
        .ok_or_else(|| "usage: pipeline-path-layout-app <base-sha> <head-sha>".to_owned())?;
    if arguments.next().is_some() {
        return Err("usage: pipeline-path-layout-app <base-sha> <head-sha>".to_owned());
    }
    Ok((
        validated_sha("base-sha", base)?,
        validated_sha("head-sha", head)?,
    ))
}

fn validated_sha(name: &str, value: String) -> Result<String, String> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a 40-digit Git object id"));
    }
    Ok(value)
}
