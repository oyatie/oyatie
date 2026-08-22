use std::fs;

use ci_dependency_automation::{Verdict, evaluate_repo};

use crate::helpers::{repo_root, temp_root, write_minimal_candidate};

#[test]
fn live_tree_has_valid_owned_dependency_automation_contract() {
    let root = repo_root();
    let report = evaluate_repo(&root).expect("evaluate live repo");
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live oya-deps contract should be green: {:#?}",
        report.findings
    );
}

#[test]
fn missing_oya_deps_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::remove_file(root.join("deps.toml")).unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-MISSING-CONFIG")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_bot_config_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::write(root.join("renovate.json"), "{}\n").unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-EXTERNAL-BOT-CONFIG")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dependabot_yaml_variant_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::create_dir_all(root.join(".github")).unwrap();
    fs::write(root.join(".github/dependabot.yaml"), "version: 2\n").unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-EXTERNAL-BOT-CONFIG"
                && finding.path == ".github/dependabot.yaml")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_config_key_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    let mut text = fs::read_to_string(root.join("deps.toml")).unwrap();
    text.push_str("\nunknown = true\n");
    fs::write(root.join("deps.toml"), text).unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-UNKNOWN-KEY")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rust_pin_split_brain_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace.package]\nrust-version = \"1.95.0\"\n",
    )
    .unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-RUST-PIN-DRIFT")
    );
    fs::remove_dir_all(root).unwrap();
}
