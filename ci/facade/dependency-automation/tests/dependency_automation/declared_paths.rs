use std::fs;
use std::path::Path;

use ci_dependency_automation::{Verdict, evaluate_repo};

use crate::helpers::{temp_root, write_minimal_candidate};

// --- failure injection for validate_declared_paths -------------------------------------------
//
// The live-tree test only ever exercises the GREEN path, so deleting the call to
// `validate_declared_paths` would have left the whole suite green. Each of these stales exactly
// one declared artifact and asserts the specific code it must produce.

/// Rewrite one `key = "value"` line inside the fixture's `oya-deps.toml`.
fn set_declared_path(root: &Path, key: &str, replacement: &str) {
    let path = root.join("oya-deps.toml");
    let text = fs::read_to_string(&path).unwrap();
    let rewritten: String = text
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&format!("{key} = ")) {
                replacement.to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(rewritten, text, "fixture must contain a `{key} = ` line");
    fs::write(&path, rewritten + "\n").unwrap();
}

#[test]
fn a_declared_path_that_names_a_deleted_file_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    // Exactly the failure that went unnoticed for months: a drift_guard naming a file deleted
    // with the `cloud/` tree.
    set_declared_path(
        &root,
        "drift_guard",
        "drift_guard = \"cloud/cloud-ci/gates/oya-cloud-ci-freshness-app/src/rust_toolchain_drift.rs\"",
    );
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "DEP-AUTO-DECLARED-PATH-MISSING"
                && f.path.ends_with("rust.drift_guard")),
        "a declared path naming a deleted file must be reported"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_declared_path_outside_the_candidate_tree_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    // `root.join("/etc/passwd")` discards `root` entirely, so an existence check alone passed
    // using state outside the tree under review.
    set_declared_path(&root, "license_policy", "license_policy = \"/etc/passwd\"");
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "DEP-AUTO-BAD-DECLARED-PATH"),
        "an absolute declared path must be rejected rather than satisfied by the runner"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_declared_path_that_escapes_upward_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    set_declared_path(
        &root,
        "stewardship_registry",
        "stewardship_registry = \"../oya-deps.toml\"",
    );
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "DEP-AUTO-BAD-DECLARED-PATH"),
        "a parent-relative declared path must be rejected"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn removing_a_declared_path_key_is_red_rather_than_skipped() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    let path = root.join("oya-deps.toml");
    let text = fs::read_to_string(&path).unwrap();
    let stripped: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("drift_guard = "))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, stripped + "\n").unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "DEP-AUTO-MISSING-KEY" && f.path.ends_with("rust.drift_guard")),
        "deleting the key must not silently disable the check"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_freshness_kernel_declaration_is_validated() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    let path = root.join("oya-deps.toml");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        format!(
            "{text}\n[freshness]\nmirror = \"ci/facade/dep-freshness/mirror/freshness.json\"\n\
             manifest = \"ci/facade/dep-freshness/mirror/freshness-manifest.json\"\n\
             kernel = \"ci/facade/dep-freshness/src/kernel.rs\"\n"
        ),
    )
    .unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "DEP-AUTO-DECLARED-PATH-MISSING"
                && f.path.ends_with("freshness.kernel")),
        "freshness.kernel is a path-valued declaration and must be existence-checked like the rest"
    );
    fs::remove_dir_all(root).unwrap();
}
