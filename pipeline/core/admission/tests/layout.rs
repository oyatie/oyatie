//! Tree shape against the public layout engine. Lives in `tests/` because it
//! reads the repo, not because it needs a private API.

use pipeline_admission::{
    ALLOWED_ROOT_DIRS, BUILD_ROOT_DIRS, cap_root_file_ok, changed_layout_violations,
    git_change_paths_from_name_status_z, layout_violations,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn unknown_root_dir_is_red() {
    let allowed: BTreeSet<&str> = ALLOWED_ROOT_DIRS
        .iter()
        .chain(BUILD_ROOT_DIRS)
        .copied()
        .collect();
    let mut unknown = Vec::new();
    for entry in std::fs::read_dir(repo_root()).expect("read root") {
        let entry = entry.expect("entry");
        if !entry.file_type().expect("ft").is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if !allowed.contains(name.as_ref()) {
            unknown.push(name.into_owned());
        }
    }
    assert!(
        unknown.is_empty(),
        "unknown root names (not admitted by D-8): {unknown:?}"
    );
}

#[test]
fn layout_engine_rejects_dump_and_accepts_faces() {
    let violations = layout_violations(&[
        "plan/foo.md".into(),
        "libs/x.rs".into(),
        "storage/src/lib.rs".into(),
        "storage/core/journal/src/lib.rs".into(),
        "app/foundry/ports/blob/src/lib.rs".into(),
        "docs/decisions/ADR.md".into(),
    ]);
    assert!(violations.iter().any(|item| item.contains("plan")));
    assert!(violations.iter().any(|item| item.contains("libs")));
    assert!(violations.iter().any(|item| item.contains("storage/src")));
    assert!(!violations.iter().any(|item| item.contains("storage/core")));
    assert!(!violations.iter().any(|item| item.contains("foundry/ports")));
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("docs/decisions"))
    );
}

#[test]
fn owner_law_files_are_the_four() {
    for name in ["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"] {
        assert!(cap_root_file_ok(name), "{name}");
    }
    assert!(!cap_root_file_ok("ADR-2.md"));
    let violations = layout_violations(&[
        "network/ADR.md".into(),
        "app/foundry/PLAN.md".into(),
        "network/ADR-2.md".into(),
    ]);
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("network/ADR.md"))
    );
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("foundry/PLAN.md"))
    );
    assert!(violations.iter().any(|item| item.contains("ADR-2.md")));
}

#[test]
fn do_have_not_capability_roots_are_admitted_when_built() {
    let violations = layout_violations(&[
        "policy/core/evaluate/src/lib.rs".into(),
        "workflow/core/saga/src/lib.rs".into(),
        "notify/core/send/src/lib.rs".into(),
    ]);
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn directory_names_cannot_be_added_as_files() {
    let violations = layout_violations(&[
        "policy".into(),
        "policy/core".into(),
        "app/new-product/core".into(),
    ]);
    assert_eq!(violations.len(), 3, "{violations:#?}");
}

#[test]
fn tests_live_in_the_crate_not_an_owner_tests_root() {
    let violations = layout_violations(&[
        "tests/foo.rs".into(),
        "e2e/foo.rs".into(),
        "network/tests/proxy.rs".into(),
        "app/foundry/tests/e2e.rs".into(),
        "network/facade/edge/tests/proxy.rs".into(),
        "network/facade/edge/tests/e2e/main.rs".into(),
        "iam/adapters/identity-scim-store-postgres/tests/live_rls.rs".into(),
    ]);
    assert!(violations.iter().any(|item| item.contains("tests/foo.rs")));
    assert!(violations.iter().any(|item| item.contains("e2e/foo.rs")));
    assert!(violations.iter().any(|item| item.contains("network/tests")));
    assert!(violations.iter().any(|item| item.contains("foundry/tests")));
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("facade/edge/tests/proxy"))
    );
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("tests/e2e/main.rs"))
    );
    assert!(!violations.iter().any(|item| item.contains("live_rls.rs")));
}

#[test]
fn iac_and_observability_are_capabilities_not_meta_roots() {
    let violations = layout_violations(&[
        "iac/src/lib.rs".into(),
        "iac/core/domain/src/lib.rs".into(),
        "observability/adapters/tracing/src/lib.rs".into(),
        "docs/foo.md".into(),
    ]);
    assert!(violations.iter().any(|item| item.contains("iac/src")));
    assert!(!violations.iter().any(|item| item.contains("iac/core")));
    assert!(
        !violations
            .iter()
            .any(|item| item.contains("observability/adapters"))
    );
    assert!(!violations.iter().any(|item| item.contains("docs/foo")));
}

#[test]
fn changed_layout_checks_only_paths_present_after_the_change() {
    let existing_build_roots = BTreeSet::new();
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
    assert!(!violations.iter().any(|item| item.contains(".cargo")));
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
    assert!(violations.iter().any(|item| item.contains("core source")));

    let implementation = git_change_paths_from_name_status_z(
        b"A\0policy/OWNERS\0A\0policy/core/evaluate/Cargo.toml\0A\0policy/core/evaluate/src/lib.rs\0",
    )
    .unwrap();
    assert!(changed_layout_violations(&implementation, &BTreeSet::new()).is_empty());

    let existing: BTreeSet<String> = ["policy".to_owned()].into();
    assert!(changed_layout_violations(&paperwork, &existing).is_empty());
}
