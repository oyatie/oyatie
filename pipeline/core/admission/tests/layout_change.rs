//! Change-relative layout and workspace regressions.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pipeline_admission::{
    changed_layout_violations, git_change_paths_from_name_status_z, layout_violations,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
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
    assert!(violations.iter().any(|item| item.contains("D-36")));

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
