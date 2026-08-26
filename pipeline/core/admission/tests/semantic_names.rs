//! Semantic-name regressions for repository admission and workflow labels.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pipeline_admission::{
    cargo_manifest_violations, changed_layout_violations, file_budget_violations,
    git_change_paths_from_name_status_z, layout_violations,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn has_decision_identifier(value: &str) -> bool {
    ["ADR-", "D-"]
        .into_iter()
        .any(|marker| decision_marker_has_number(value, marker))
}

fn decision_marker_has_number(mut value: &str, marker: &str) -> bool {
    while let Some((_, tail)) = value.split_once(marker) {
        if tail
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            return true;
        }
        value = tail;
    }
    false
}

fn assert_semantic(surface: &str, value: &str) {
    assert!(
        !has_decision_identifier(value),
        "{surface} exposes a decision identifier as an operational name: {value}"
    );
}

#[test]
fn presubmit_display_names_are_semantic_and_required_fan_in_is_stable() {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/presubmit.yml"))
        .expect("presubmit workflow");
    let display_names: Vec<&str> = workflow
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("name: ")
                .or_else(|| line.strip_prefix("- name: "))
        })
        .collect();
    for name in display_names {
        assert_semantic("workflow display name", name);
    }
    assert!(workflow.contains("\n  layout:\n    name: repository layout\n"));
    assert!(workflow.contains("\n  presubmit:\n    name: presubmit\n"));
    assert!(
        workflow.contains(
            "needs: [layout, occupancy, lint, clippy, test, deny, pg-gate, live-postgres]"
        )
    );
}

#[test]
fn emitted_admission_diagnostics_are_semantic() {
    let budget = file_budget_violations(
        "network/core/route/src/lib.rs",
        "line\n".repeat(301).as_bytes(),
    )
    .join("\n");
    assert!(budget.contains("repository 300-line file budget"));
    assert_semantic("file-budget diagnostic", &budget);

    let manifest = cargo_manifest_violations(
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nautotests=false\nbuild='other.rs'\n",
    )
    .join("\n");
    assert!(manifest.contains("required integration-test discovery"));
    assert!(manifest.contains("stable item-scanner `build.rs`"));
    assert_semantic("manifest diagnostic", &manifest);

    let change = git_change_paths_from_name_status_z(
        b"A\0app/ledger/OWNERS\0A\0app/ledger/core/posting/Cargo.toml\0A\0app/ledger/core/posting/src/lib.rs\0",
    )
    .expect("change paths");
    let owner_law = changed_layout_violations(&change, &BTreeSet::new()).join("\n");
    assert!(owner_law.contains("canonical owner-law files"));
    assert_semantic("owner-law diagnostic", &owner_law);
}

#[test]
fn provenance_citations_and_decision_files_remain_valid() {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/presubmit.yml"))
        .expect("presubmit workflow");
    assert!(workflow.contains("ADR-0719"));
    assert!(layout_violations(&["docs/decisions/ADR-0720-example.md".to_owned()]).is_empty());
    assert!(
        file_budget_violations(
            "docs/decisions/ADR-0719-example.md",
            "line\n".repeat(301).as_bytes(),
        )
        .is_empty()
    );
}
