#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const FIXTURE: &[u8] = include_bytes!("fixtures/read-access.json");
const BASELINE_REPORT: &str = r#"{
  "policy_version": "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2",
  "qualification_digest": "sha256:c17c6d383103439f45a8d6133d3ca6901f729363bb6ee5da593e31d3aa2aef92",
  "passed_cases": 2
}"#;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/read-access.json")
}

#[test]
fn process_check_and_prepare_preserve_the_stable_fixture_identity() {
    let checked = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("check")
        .arg(fixture_path())
        .output()
        .unwrap();
    assert!(checked.status.success(), "{:?}", checked.stderr);
    assert_eq!(
        std::str::from_utf8(&checked.stdout).unwrap().trim(),
        BASELINE_REPORT
    );

    let prepared = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("prepare")
        .arg(fixture_path())
        .output()
        .unwrap();
    assert!(prepared.status.success(), "{:?}", prepared.stderr);
    let bundle: policy_pdp_kernel::PolicyBundle = serde_json::from_slice(&prepared.stdout).unwrap();
    assert_eq!(
        bundle.version.as_str(),
        "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2"
    );
}

#[test]
fn failed_authored_cases_exit_nonzero_without_stdout() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "policy-cli-failed-case-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut project: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    project["cases"][0]["expected"]["obligations"] = serde_json::json!([]);
    std::fs::write(&path, serde_json::to_vec(&project).unwrap()).unwrap();

    let refused = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("prepare")
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(path);
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
}

#[test]
fn usage_names_the_canonical_policy_cli_executable() {
    let refused = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(
        std::str::from_utf8(&refused.stderr)
            .unwrap()
            .contains("usage: policy-cli <check|prepare> <project.json>")
    );
}
