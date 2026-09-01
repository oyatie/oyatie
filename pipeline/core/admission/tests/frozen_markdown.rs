//! Default freeze and qualified owner-prose deletion boundary.

#[path = "support/owner_prose.rs"]
mod support;

use std::collections::BTreeSet;

use pipeline_admission::{
    OwnerProseQualification, changed_layout_violations,
    changed_layout_violations_with_qualified_owner_prose, git_change_paths_from_name_status_z,
    layout_violations,
};
use support::Fixture;

const LAWS: [&str; 4] = ["ADR.md", "PLAN.md", "PRD.md", "SPEC.md"];

#[test]
fn every_changed_non_root_markdown_shape_refuses_by_default() {
    let existing: BTreeSet<String> = ["pipeline".to_owned()].into();
    for input in [
        b"A\0pipeline/new.md\0".as_slice(),
        b"M\0pipeline/current.md\0".as_slice(),
        b"D\0pipeline/legacy.md\0".as_slice(),
        b"R100\0pipeline/legacy.md\0pipeline/core/graph/src/lib.rs\0".as_slice(),
    ] {
        let changes = git_change_paths_from_name_status_z(input).expect("exact Git change");
        assert!(
            changed_layout_violations(&changes, &existing)
                .iter()
                .any(|violation| violation.contains("frozen non-root Markdown"))
        );
    }
}

#[test]
fn only_three_exact_root_markdown_destinations_remain_mutable() {
    assert!(
        layout_violations(&["README.md", "AGENTS.md", "CLAUDE.md"].map(str::to_owned)).is_empty()
    );
    for path in [
        "policy/README.md",
        "docs/decisions/ADR-0720-example.md",
        "app/README.md",
        "policy/NOTES.MD",
        "policy/notes.markdown",
    ] {
        assert!(
            layout_violations(&[path.to_owned()])
                .iter()
                .any(|violation| violation.contains("frozen non-root Markdown")),
            "{path}"
        );
    }
}

#[test]
fn only_one_complete_qualified_owner_prose_deletion_is_admitted() {
    let existing: BTreeSet<String> = ["policy".to_owned()].into();
    let OwnerProseQualification::Ready(qualified) = Fixture::complete().qualify() else {
        panic!("complete fixture must qualify");
    };
    let complete = git_change_paths_from_name_status_z(
        b"D\0policy/ADR.md\0D\0policy/PLAN.md\0D\0policy/PRD.md\0D\0policy/SPEC.md\0",
    )
    .expect("complete deletion");
    assert!(!changed_layout_violations(&complete, &existing).is_empty());
    assert!(
        changed_layout_violations_with_qualified_owner_prose(
            &complete,
            &existing,
            Some(&qualified)
        )
        .is_empty()
    );

    for omitted in LAWS {
        let input = LAWS
            .into_iter()
            .filter(|name| *name != omitted)
            .map(|name| format!("D\0policy/{name}\0"))
            .collect::<String>();
        let partial = git_change_paths_from_name_status_z(input.as_bytes()).expect("partial");
        let violations = changed_layout_violations_with_qualified_owner_prose(
            &partial,
            &existing,
            Some(&qualified),
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains(omitted) && violation.contains("incomplete")),
            "missing failure injection for {omitted}: {violations:#?}"
        );
    }
}

#[test]
fn new_owner_is_proved_by_native_source_and_cannot_add_owner_prose() {
    let source_only = git_change_paths_from_name_status_z(
        b"A\0policy/OWNERS\0A\0policy/core/evaluate/Cargo.toml\0A\0policy/core/evaluate/src/lib.rs\0",
    )
    .expect("source-only owner");
    assert!(changed_layout_violations(&source_only, &BTreeSet::new()).is_empty());

    let with_prose = git_change_paths_from_name_status_z(
        b"A\0policy/OWNERS\0A\0policy/ADR.md\0A\0policy/PRD.md\0A\0policy/SPEC.md\0A\0policy/PLAN.md\0A\0policy/core/evaluate/Cargo.toml\0A\0policy/core/evaluate/src/lib.rs\0",
    )
    .expect("owner with prose");
    assert!(
        changed_layout_violations(&with_prose, &BTreeSet::new())
            .iter()
            .any(|violation| violation.contains("frozen non-root Markdown"))
    );
}

#[test]
fn owner_docs_and_app_meta_do_not_reintroduce_tracked_prose() {
    for path in [
        "network/docs/design/routing.md",
        "network/docs/runbooks/tasks/todo.md",
        "app/ADR.md",
        "app/README.md",
    ] {
        assert!(!layout_violations(&[path.to_owned()]).is_empty(), "{path}");
    }
    assert!(layout_violations(&["app/OWNERS".to_owned()]).is_empty());
}
