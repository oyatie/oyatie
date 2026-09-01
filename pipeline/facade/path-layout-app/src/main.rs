//! Event-independent changed-path repository-layout admission facade.
//! Provenance: ADR-0719 repository-layout decision (D-8).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use pipeline_admission::{
    base_admission_violations, cargo_entrypoints, cargo_manifest_for_crate_path,
    cargo_manifest_violations, changed_layout_violations_with_qualified_owner_prose,
    draft_dependency_violations, git_change_paths_from_name_status_z,
    owner_core_regression_violations, proto_package_violations,
    workspace_draft_dependency_violations, workspace_membership_violations,
};
use pipeline_repository_draft::RepositoryRead;
use pipeline_repository_git_draft::GitRepository;

mod owner_prose_view;
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
    let repository = GitRepository;
    match invocation()? {
        Invocation::Admit { base, head, view } => admit(repository, &base, &head, view),
        Invocation::Qualify { base, head, view } => {
            let (merge_base, head) = exact_candidate(&repository, &base, &head)?;
            let qualified = owner_prose_view::qualify_view(&repository, &merge_base, &head, &view)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&qualified)
                    .map_err(|error| format!("serialize qualified owner-prose view: {error}"))?
            );
            Ok(())
        }
    }
}

fn admit(
    repository: GitRepository,
    base: &str,
    head: &str,
    view: Option<PathBuf>,
) -> Result<(), String> {
    let (merge_base, head) = exact_candidate(&repository, base, head)?;
    let name_status = repository.changed_name_status(&merge_base, &head)?;
    let changes =
        git_change_paths_from_name_status_z(&name_status).map_err(|error| error.message())?;
    let owners_before = owner_tree_state(&repository, &merge_base)?;
    let owners_after = owner_tree_state(&repository, &head)?;
    let workspace_contents = repository.blob_text(&head, "Cargo.toml")?;
    let qualified_view = match view {
        Some(path) => Some(owner_prose_view::qualify_view(
            &repository,
            &merge_base,
            &head,
            &path,
        )?),
        None => None,
    };
    let mut violations = changed_layout_violations_with_qualified_owner_prose(
        &changes,
        &owners_before.live,
        qualified_view.as_ref(),
    );
    violations.extend(owner_core_regression_violations(
        &changes,
        &owners_before.complete,
        &owners_after.live,
        &owners_after.complete,
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
        &changes.exact_rename_sources,
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
        let candidates = cargo_entrypoints(&manifest);
        let canonical = candidates
            .first()
            .cloned()
            .expect("canonical crate manifest");
        let mut present = None;
        for candidate in &candidates {
            if let Some(kind) = repository.entry_kind(&head, candidate)? {
                present = Some((candidate.clone(), kind));
                break;
            }
        }
        // Ratchet. A facade admits either root, but that relaxation is about
        // a surface whose listener has not been attached YET - not about
        // removing a binary that already runs. Without this, deleting
        // `src/main.rs` from a facade that has one passes clean, because
        // `src/lib.rs` answers in its place.
        if let Some(binary) = candidates.first()
            && binary.ends_with("/src/main.rs")
            && repository.path_exists(&merge_base, binary)?
            && repository.entry_kind(&head, binary)?.is_none()
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
            && (!repository.path_exists(&merge_base, &manifest)?
                || !repository.path_exists(&merge_base, &canonical)?)
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
            "repository layout refused:\n{}",
            violations.join("\n")
        ))
    }
}

fn exact_candidate(
    repository: &impl RepositoryRead,
    base: &str,
    head: &str,
) -> Result<(String, String), String> {
    let base = repository.resolve_commit(base)?;
    let head = repository.resolve_commit(head)?;
    let merge_base = repository.merge_base(&base, &head)?;
    Ok((repository.resolve_commit(&merge_base)?, head))
}

enum Invocation {
    Admit {
        base: String,
        head: String,
        view: Option<PathBuf>,
    },
    Qualify {
        base: String,
        head: String,
        view: PathBuf,
    },
}

const USAGE: &str = "usage: pipeline-path-layout-app <base-sha> <head-sha> [--owner-prose-view <absolute-json-path>]\n       pipeline-path-layout-app qualify-owner-prose <base-sha> <head-sha> <absolute-json-path>";

fn invocation() -> Result<Invocation, String> {
    let mut arguments = std::env::args().skip(1);
    let first = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let qualify = first == "qualify-owner-prose";
    let base = if qualify {
        arguments.next().ok_or_else(|| USAGE.to_owned())?
    } else {
        first
    };
    let head = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    let base = validated_sha("base-sha", base)?;
    let head = validated_sha("head-sha", head)?;
    if qualify {
        let view = PathBuf::from(arguments.next().ok_or_else(|| USAGE.to_owned())?);
        if arguments.next().is_some() {
            return Err(USAGE.to_owned());
        }
        return Ok(Invocation::Qualify { base, head, view });
    }
    let view = match arguments.next() {
        None => None,
        Some(flag) if flag == "--owner-prose-view" => Some(PathBuf::from(
            arguments.next().ok_or_else(|| USAGE.to_owned())?,
        )),
        Some(_) => return Err(USAGE.to_owned()),
    };
    if arguments.next().is_some() {
        return Err(USAGE.to_owned());
    }
    Ok(Invocation::Admit { base, head, view })
}

fn validated_sha(name: &str, value: String) -> Result<String, String> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a full hexadecimal Git object id"));
    }
    Ok(value)
}
