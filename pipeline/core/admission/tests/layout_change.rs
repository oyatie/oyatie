//! Change-relative layout and workspace regressions.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pipeline_admission::{
    WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS, cargo_entrypoint, cargo_manifest_for_crate_path,
    cargo_manifest_for_entrypoint, cargo_manifest_violations, changed_layout_violations,
    git_change_paths_from_name_status_z, layout_violations, workspace_membership_violations,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn workspace_manifest(members: &[&str], excludes: &[&str]) -> String {
    let members = members
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    let excludes = excludes
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n")
}

fn workspace_admits(members: &[&str], excludes: &[&str]) -> bool {
    workspace_membership_violations(&workspace_manifest(members, excludes)).is_empty()
}

#[test]
fn changed_layout_checks_only_paths_present_after_the_change() {
    let existing_build_roots: BTreeSet<String> = ["pipeline".to_owned()].into();
    let deletion = git_change_paths_from_name_status_z(b"D\0plan/legacy.md\0").unwrap();
    assert!(deletion.occupied.contains("plan/legacy.md"));
    assert!(changed_layout_violations(&deletion, &existing_build_roots).is_empty());

    let cleanup = git_change_paths_from_name_status_z(
        b"R100\0plan/legacy.md\0pipeline/core/graph/src/lib.rs\0",
    )
    .unwrap();
    assert!(cleanup.occupied.contains("plan/legacy.md"));
    assert!(cleanup.occupied.contains("pipeline/core/graph/src/lib.rs"));
    assert!(changed_layout_violations(&cleanup, &existing_build_roots).is_empty());

    let regression = git_change_paths_from_name_status_z(
        b"R100\0pipeline/core/graph/src/lib.rs\0plan/legacy.md\0",
    )
    .unwrap();
    assert!(
        changed_layout_violations(&regression, &existing_build_roots)
            .iter()
            .any(|violation| violation.contains("plan/legacy.md"))
    );
}

#[test]
fn every_new_capability_requires_core_and_owner_law() {
    let paperwork =
        git_change_paths_from_name_status_z(b"A\0network/OWNERS\0A\0network/README.md\0").unwrap();
    let violations = changed_layout_violations(&paperwork, &BTreeSet::new());
    assert!(violations.iter().any(|item| item.contains("core crate")));
    assert!(
        violations
            .iter()
            .any(|item| item.contains("canonical owner-law"))
    );

    let implementation = git_change_paths_from_name_status_z(
        b"A\0network/OWNERS\0A\0network/ADR.md\0A\0network/PRD.md\0A\0network/SPEC.md\0A\0network/PLAN.md\0A\0network/core/route/Cargo.toml\0A\0network/core/route/src/lib.rs\0",
    )
    .unwrap();
    assert!(changed_layout_violations(&implementation, &BTreeSet::new()).is_empty());
}

#[test]
fn inner_face_grammar_rejects_crate_root_dumps() {
    let violations = layout_violations(&[
        "pipeline/core/admission/plan/note.md".into(),
        "app/foundry/core/grid/tasks/note.md".into(),
        "network/ports/draft/blob/src/lib.rs".into(),
        "network/facade/proto/network/edge/v1/edge_service.proto".into(),
    ]);
    assert!(
        violations
            .iter()
            .any(|item| item.contains("admission/plan"))
    );
    assert!(violations.iter().any(|item| item.contains("grid/tasks")));
    assert!(!violations.iter().any(|item| item.contains("draft/blob")));
    assert!(!violations.iter().any(|item| item.contains("edge_service")));
}

#[test]
fn closed_root_set_rejects_hidden_target_and_backslash_bypasses() {
    let violations = layout_violations(&[
        ".idea/workspace.xml".into(),
        "target/kept.txt".into(),
        r"network\core\dump.rs".into(),
        ".github/workflows/new.yml".into(),
        ".cargo/config.toml".into(),
        ".buckconfig".into(),
    ]);
    assert!(violations.iter().any(|item| item.contains(".idea")));
    assert!(violations.iter().any(|item| item.contains("target")));
    assert!(
        violations
            .iter()
            .any(|item| item.contains("backslash") || item.contains("separator"))
    );
    assert!(!violations.iter().any(|item| item.contains(".github")));
    assert!(!violations.iter().any(|item| item.contains(".cargo/config")));
    assert!(!violations.iter().any(|item| item.contains(".buckconfig")));
}

#[test]
fn cargo_files_belong_below_faces_not_on_owner_roots() {
    let violations = layout_violations(&[
        "network/Cargo.toml".into(),
        "app/foundry/Cargo.lock".into(),
        "network/core/domain/Cargo.toml".into(),
    ]);
    assert!(violations.iter().any(|item| item.contains("network/Cargo")));
    assert!(violations.iter().any(|item| item.contains("foundry/Cargo")));
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("core/domain/Cargo"))
    );
}

#[test]
fn new_build_root_requires_core_source_not_owner_paperwork() {
    let paperwork = git_change_paths_from_name_status_z(
        b"A\0policy/OWNERS\0A\0policy/README.md\0A\0.github/CODEOWNERS\0",
    )
    .unwrap();
    let violations = changed_layout_violations(&paperwork, &BTreeSet::new());
    assert!(violations.iter().any(|item| item.contains("core crate")));

    let implementation = git_change_paths_from_name_status_z(
        b"A\0policy/OWNERS\0A\0policy/ADR.md\0A\0policy/PRD.md\0A\0policy/SPEC.md\0A\0policy/PLAN.md\0A\0policy/core/evaluate/Cargo.toml\0A\0policy/core/evaluate/src/lib.rs\0",
    )
    .unwrap();
    assert!(changed_layout_violations(&implementation, &BTreeSet::new()).is_empty());

    let existing: BTreeSet<String> = ["policy".to_owned()].into();
    assert!(changed_layout_violations(&paperwork, &existing).is_empty());
}

#[test]
fn workspace_globs_bound_direct_and_draft_crate_depths() {
    let workspace = std::fs::read_to_string(repo_root().join("Cargo.toml")).expect("Cargo.toml");
    assert!(
        pipeline_admission::workspace_membership_violations(&workspace).is_empty(),
        "the live root workspace must equal the closed admission policy"
    );
    for member in [
        "*/ports/*/src/..",
        "*/adapters/*/src/..",
        "*/facade/*/src/..",
        "*/ports/draft/*/src/..",
        "*/adapters/draft/*/src/..",
        "app/*/ports/*/src/..",
        "app/*/adapters/*/src/..",
        "app/*/ports/draft/*/src/..",
        "app/*/adapters/draft/*/src/..",
        "app/*/facade/*/src/..",
    ] {
        assert!(workspace.contains(&format!("\"{member}\"")), "{member}");
    }
    for recursive in [
        "*/ports/**/src/..",
        "*/adapters/**/src/..",
        "app/*/ports/**/src/..",
        "app/*/adapters/**/src/..",
    ] {
        assert!(
            !workspace.contains(&format!("\"{recursive}\"")),
            "{recursive}"
        );
    }
    for forbidden_parent in [
        "\"*/ports/*\"",
        "\"*/adapters/*\"",
        "\"*/facade/*\"",
        "\"app/*/ports/*\"",
        "\"app/*/adapters/*\"",
        "\"app/*/facade/*\"",
    ] {
        assert!(!workspace.contains(forbidden_parent), "{forbidden_parent}");
    }
}

#[test]
fn dependency_declarations_bind_exact_names_and_entrypoints() {
    for (path, package, entrypoint) in [
        (
            "core/reconcile",
            "dependency-declarations-reconcile",
            "src/lib.rs",
        ),
        (
            "ports/generation",
            "dependency-declarations-generation",
            "src/lib.rs",
        ),
        (
            "ports/publication",
            "dependency-declarations-publication",
            "src/lib.rs",
        ),
        (
            "adapters/generation-reindeer",
            "dependency-declarations-generation-reindeer",
            "src/lib.rs",
        ),
        (
            "adapters/publication-filesystem",
            "dependency-declarations-publication-filesystem",
            "src/lib.rs",
        ),
        (
            "facade/reconciler-app",
            "dependency-declarations-reconciler-app",
            "src/main.rs",
        ),
    ] {
        let root = format!("build/dependency-declarations/{path}");
        let manifest = format!("{root}/Cargo.toml");
        let entrypoint = format!("{root}/{entrypoint}");
        assert!(!cargo_manifest_violations(&manifest, "[package]\nname='wrong'\n").is_empty());
        assert_eq!(
            cargo_entrypoint(&manifest).as_deref(),
            Some(entrypoint.as_str())
        );
        assert_eq!(
            cargo_manifest_for_entrypoint(&entrypoint).as_deref(),
            Some(manifest.as_str())
        );
        assert_eq!(
            cargo_manifest_for_crate_path(&format!("{root}/tests/contract.rs")).as_deref(),
            Some(manifest.as_str())
        );
        assert!(
            cargo_manifest_violations(&manifest, &format!("[package]\nname='{package}'\n"))
                .is_empty()
        );
    }
}

#[test]
fn dependency_declarations_workspace_pair_is_atomic_and_final() {
    const MEMBER: &str = "build/dependency-declarations/*/*/src/..";
    const EXCLUDE: &str = "build/dependency-declarations/*/*";
    let base_members = WORKSPACE_MEMBER_GLOBS.to_vec();
    let base_excludes = WORKSPACE_EXCLUDES.to_vec();
    assert!(workspace_admits(&base_members, &base_excludes));

    let mut paired_members = base_members.clone();
    paired_members.push(MEMBER);
    let mut paired_excludes = base_excludes.clone();
    paired_excludes.push(EXCLUDE);
    assert!(workspace_admits(&paired_members, &paired_excludes));

    for (members, excludes) in [
        (paired_members.clone(), base_excludes.clone()),
        (base_members.clone(), paired_excludes.clone()),
    ] {
        assert!(!workspace_admits(&members, &excludes));
    }
    let mut reordered = paired_members.clone();
    let optional = reordered.pop().expect("optional member");
    reordered.insert(0, optional);
    assert!(!workspace_admits(&reordered, &paired_excludes));

    for entry in [
        MEMBER,
        "build/dependency-declarations/*/*",
        "build/dependency-declarations/**/src/..",
    ] {
        let mut members = paired_members.clone();
        members.push(entry);
        assert!(!workspace_admits(&members, &paired_excludes));
    }
    for entry in [
        EXCLUDE,
        "build/dependency-declarations/*",
        "build/dependency-declarations/**",
    ] {
        let mut excludes = paired_excludes.clone();
        excludes.push(entry);
        assert!(!workspace_admits(&paired_members, &excludes));
    }
}
