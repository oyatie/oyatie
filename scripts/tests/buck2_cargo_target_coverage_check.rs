#![allow(dead_code)]

#[path = "../ci/assert-buck2-cargo-target-coverage.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-buck2-cargo-target-coverage-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap()
}

fn fixture_spec_with_checker(checker: &str, p0_green: bool) -> String {
    format!(
        r#"{{
  "claim_boundary": {{
    "target_coverage_measured": true,
    "source_line_coverage_generated": false,
    "mutation_lane_implemented": false,
    "status_mutation_performed": false,
    "protected_branch_authority_proven": false,
    "live_required_context_execution_proven": false,
    "p0_0_green": {p0_green},
    "phase0_complete": false,
    "production_ready": false,
    "hyperscaler_grade": false
  }},
  "measurement_contract": {{
    "buck2_target": "//:buck2-cargo-target-coverage-check",
    "checker": "{checker}",
    "workspace_manifest": "Cargo.toml",
    "parent_buck_allowed": true,
    "buck2_mapping_rule": "crate_root mapping",
    "cargo_autodiscovery_rule": "autobins src/bin",
    "cargo_target_roots": ["src/bin/*.rs", "src/bin/*/main.rs"],
    "known_divergences": [],
    "forbidden_authority": ["source-line coverage claims", "protected branch authority"]
  }},
  "automated_chain": ["buck2 build //:buck2-cargo-target-coverage-check"],
  "_meta": {{
    "official_references": [
      {{"url": "https://doc.rust-lang.org/cargo/reference/workspaces.html"}},
      {{"url": "https://doc.rust-lang.org/cargo/reference/cargo-targets.html"}},
      {{"url": "https://buck2.build/docs/users/commands/"}},
      {{"url": "https://buck2.build/docs/about/bootstrapping/"}},
      {{"url": "https://github.com/facebookincubator/reindeer"}}
    ]
  }}
}}"#
    )
}

fn make_fixture_repo(name: &str) -> PathBuf {
    let root = temp_root(name);
    write(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/app"]
"#,
    );
    write(
        &root.join("crates/app/Cargo.toml"),
        r#"[package]
name = "fixture-app"
version = "0.1.0"
edition = "2024"
"#,
    );
    write(&root.join("crates/app/src/main.rs"), "fn main() {}\n");
    write(&root.join("crates/app/src/bin/tool.rs"), "fn main() {}\n");
    write(
        &root.join("spec.json"),
        &fixture_spec_with_checker("scripts/ci/assert-buck2-cargo-target-coverage.rs", false),
    );
    root
}

#[test]
fn checked_in_contract_passes() {
    let root = repo_root();
    let evaluation = gate::evaluate(
        &root,
        "Cargo.toml",
        "specs/buck2-cargo-target-coverage.json",
    );
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.workspace_member_count > 700);
    assert!(evaluation.cargo_target_root_count >= evaluation.workspace_member_count);
    assert_eq!(
        evaluation.buck2_mapped_target_root_count,
        evaluation.cargo_target_root_count
    );
    assert!(evaluation.missing_mappings.is_empty());
    assert_eq!(evaluation.known_divergence_count, 0);
}

#[test]
fn missing_buck_mapping_fails() {
    let root = make_fixture_repo("missing");
    let evaluation = gate::evaluate(&root, "Cargo.toml", "spec.json");
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .contains(&"missing_buck2_target_root_mapping".to_owned())
    );
    assert_eq!(evaluation.missing_mappings.len(), 2);
}

#[test]
fn parent_buck_mapping_passes() {
    // good-parent-repo: parent BUCK files may own crate_root mappings for
    // package-local Cargo targets when the mapping resolves to the same source.
    let root = make_fixture_repo("good-parent-repo");
    write(
        &root.join("BUCK"),
        r#"rust_binary(
    name = "fixture-app",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/main.rs",
    visibility = ["PUBLIC"],
)

rust_binary(
    name = "fixture-tool",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/bin/tool.rs",
    visibility = ["PUBLIC"],
)
"#,
    );
    let evaluation = gate::evaluate(&root, "Cargo.toml", "spec.json");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.buck2_mapped_target_root_count, 2);
}

#[test]
fn autobins_false_respects_explicit_bin_only() {
    let root = make_fixture_repo("autobins-false");
    write(
        &root.join("crates/app/Cargo.toml"),
        r#"[package]
name = "fixture-app"
version = "0.1.0"
edition = "2024"
autobins = false

[[bin]]
name = "fixture-app"
path = "src/main.rs"
"#,
    );
    write(
        &root.join("BUCK"),
        r#"rust_binary(
    name = "fixture-app",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/main.rs",
    visibility = ["PUBLIC"],
)
"#,
    );
    let evaluation = gate::evaluate(&root, "Cargo.toml", "spec.json");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.cargo_target_root_count, 1);
}

#[test]
fn p0_green_claim_fails() {
    let root = make_fixture_repo("p0-green");
    write(
        &root.join("spec.json"),
        &fixture_spec_with_checker("scripts/ci/assert-buck2-cargo-target-coverage.rs", true),
    );
    let evaluation = gate::evaluate(&root, "Cargo.toml", "spec.json");
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .contains(&"forbidden_true_or_missing_claim_p0_0_green".to_owned())
    );
}

#[test]
fn retired_python_checker_path_fails() {
    let root = make_fixture_repo("retired-checker");
    let retired_checker_path =
        ["scripts/ci/", "assert-buck2-cargo-target-coverage", ".py"].concat();
    write(
        &root.join("spec.json"),
        &fixture_spec_with_checker(&retired_checker_path, false),
    );
    let evaluation = gate::evaluate(&root, "Cargo.toml", "spec.json");
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .contains(&"wrong_checker_path".to_owned())
    );
    assert!(
        evaluation
            .failures
            .contains(&"retired_python_checker_path_present".to_owned())
    );
}

#[test]
fn checked_in_matrix_keeps_buck2_native_command() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    assert!(matrix.contains("AC-0.13-buck2-cargo-coverage"));
    assert!(matrix.contains("//:buck2-cargo-target-coverage-check"));
    assert!(matrix.contains("buck2 build //:buck2-cargo-target-coverage-check"));
    assert!(matrix.contains("source-line coverage"));
    assert!(matrix.contains("\"no_new_oya_cli_surface\": true"));
}
