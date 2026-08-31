//! Event-independent changed-path repository-layout admission facade.
//! Provenance: ADR-0719 repository-layout decision (D-8).

use std::collections::BTreeSet;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use pipeline_admission::{
    GitChangePaths, base_admission_violations, cargo_entrypoints, cargo_manifest_for_crate_path,
    cargo_manifest_violations, changed_layout_violations, draft_dependency_violations,
    owner_core_regression_violations, proto_package_violations,
    workspace_draft_dependency_violations, workspace_membership_violations,
};
use pipeline_repository::{
    EntryKind, NoCancellation, RepositoryId, RepositoryManifest, RepositoryPath,
    RepositorySnapshot, RevisionId, SnapshotRequest, SnapshotSession,
};
use pipeline_repository_git::GitRepository;

mod repository_checks;
mod snapshot_input;

const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(5 * 60);

use repository_checks::{
    RepositoryView, live_candidate_violations, owner_tree_state, regular_blob,
    reject_indirect_dependency_components, repository_cargo_config_violations, snapshot_error,
};
use snapshot_input::{admission_changes, layout_profile, selected_content};

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
    let control = NoCancellation::until(Instant::now() + SNAPSHOT_TIMEOUT);
    let repository_id = RepositoryId::new("oyatie/oyatie").map_err(|error| error.to_string())?;
    let repository =
        GitRepository::current(repository_id.clone(), &control).map_err(snapshot_error)?;
    let session = repository
        .capture(
            SnapshotRequest::new(repository_id, base, head, layout_profile()?)
                .map_err(snapshot_error)?,
            &control,
        )
        .map_err(snapshot_error)?;
    let changes = admission_changes(session.prepared().delta())?;
    let selected = selected_content(session.prepared(), &changes)?;
    let selection = session
        .prepared()
        .select_content(selected)
        .map_err(snapshot_error)?;
    let snapshot = session
        .hydrate(selection, &control)
        .map_err(snapshot_error)?;
    let merge_manifest = snapshot.prepared().merge_base();
    let head_manifest = snapshot.prepared().head();
    let head_repository = RepositoryView::new(head_manifest, &snapshot);
    let owners_before = owner_tree_state(merge_manifest)?;
    let owners_after = owner_tree_state(head_manifest)?;
    let workspace_contents = head_repository.blob_text("Cargo.toml")?;
    let mut violations = changed_layout_violations(&changes, &owners_before.live);
    violations.extend(owner_core_regression_violations(
        &changes,
        &owners_before.complete,
        &owners_after.live,
        &owners_after.complete,
    ));
    violations.extend(workspace_membership_violations(&workspace_contents));
    violations.extend(workspace_draft_dependency_violations(
        &workspace_contents,
        |visited| reject_indirect_dependency_components(head_manifest, visited),
    ));
    violations.extend(repository_cargo_config_violations(&head_repository)?);
    violations.extend(live_candidate_violations(
        &head_repository,
        &changes.layout_candidates,
        &changes.exact_rename_sources,
    )?);
    let mut manifests = Vec::new();
    for path in changes
        .layout_candidates
        .iter()
        .filter(|path| path.ends_with("/Cargo.toml"))
    {
        let contents = head_repository.blob_text(path)?;
        violations.extend(cargo_manifest_violations(path, &contents));
        violations.extend(draft_dependency_violations(
            path,
            &contents,
            &workspace_contents,
            |visited| reject_indirect_dependency_components(head_manifest, visited),
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
        if !directory_exists(head_manifest, directory)? {
            continue;
        }
        let manifest_kind = entry_kind(head_manifest, &manifest)?;
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
        let candidates = cargo_entrypoints(&manifest);
        let canonical = candidates
            .first()
            .cloned()
            .expect("canonical crate manifest");
        let mut present = None;
        for candidate in &candidates {
            if let Some(kind) = entry_kind(head_manifest, candidate)? {
                present = Some((candidate.clone(), kind));
                break;
            }
        }
        if let Some(binary) = candidates.first()
            && binary.ends_with("/src/main.rs")
            && removed_existing_entrypoint(merge_manifest, head_manifest, binary)?
        {
            violations.push(format!(
                "{manifest}: `{binary}` existed at the merge base and is absent at the head \
                 commit; a facade may land before its listener is attached, but a running \
                 one does not become a library by deleting its entry point"
            ));
        }
        let (entrypoint_exists, entrypoint_is_blob) = match present {
            None => {
                violations.push(format!(
                    "{manifest}: no canonical entry point at the head commit; expected one of {}",
                    candidates.join(", ")
                ));
                (false, false)
            }
            Some((path, kind)) if !regular_blob(Some(kind)) => {
                violations.push(format!(
                    "{manifest}: canonical entry point `{path}` must be a regular Git blob, got {kind:?}"
                ));
                (true, false)
            }
            Some(_) => (true, true),
        };
        if manifest.starts_with("base/core/")
            && manifest_exists
            && manifest_is_blob
            && entrypoint_exists
            && entrypoint_is_blob
            && (!path_exists(merge_manifest, &manifest)?
                || !path_exists(merge_manifest, &canonical)?)
        {
            added_base_manifests.push(manifest);
        }
    }
    for path in changes
        .layout_candidates
        .iter()
        .filter(|path| path.ends_with(".proto"))
    {
        let contents = head_repository.blob_text(path)?;
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
            "repository layout refused:\n{}",
            violations.join("\n")
        ))
    }
}

fn required_shas() -> Result<(RevisionId, RevisionId), String> {
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

fn validated_sha(name: &str, value: String) -> Result<RevisionId, String> {
    RevisionId::from_hex(&value)
        .map_err(|_| format!("{name} must be a complete 40- or 64-digit Git object id"))
}

fn entry_kind(manifest: &RepositoryManifest, path: &str) -> Result<Option<EntryKind>, String> {
    let path = RepositoryPath::from_utf8(path).map_err(|error| error.to_string())?;
    Ok(manifest.entry(&path).map(|entry| entry.kind()))
}

fn path_exists(manifest: &RepositoryManifest, path: &str) -> Result<bool, String> {
    Ok(entry_kind(manifest, path)?.is_some())
}

fn directory_exists(manifest: &RepositoryManifest, path: &str) -> Result<bool, String> {
    Ok(entry_kind(manifest, path)? == Some(EntryKind::Tree))
}

fn removed_existing_entrypoint(
    merge_manifest: &RepositoryManifest,
    head_manifest: &RepositoryManifest,
    path: &str,
) -> Result<bool, String> {
    Ok(path_exists(merge_manifest, path)? && !path_exists(head_manifest, path)?)
}
