#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const FIXTURE: &[u8] = include_bytes!("fixtures/read-access.json");
const BASELINE_REPORT: &str = r#"{
  "policy_version": "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2",
  "qualification_digest": "sha256:c17c6d383103439f45a8d6133d3ca6901f729363bb6ee5da593e31d3aa2aef92",
  "passed_cases": 2
}"#;
static NEXT_TEMPORARY_PROJECT: AtomicUsize = AtomicUsize::new(0);

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn materialize(label: &str, contents: &[u8]) -> Self {
        loop {
            let sequence = NEXT_TEMPORARY_PROJECT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "policy-cli-{label}-{}-{sequence}.json",
                std::process::id()
            ));
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("temporary project {} failed: {error}", path.display()),
            };
            let project = Self { path };
            file.write_all(contents).unwrap_or_else(|error| {
                panic!(
                    "temporary project {} write failed: {error}",
                    project.path.display()
                )
            });
            return project;
        }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn process_check_and_prepare_preserve_the_stable_fixture_identity() {
    let fixture = TemporaryProject::materialize("stable-fixture", FIXTURE);
    let checked = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("check")
        .arg(fixture.path())
        .output()
        .unwrap();
    assert!(checked.status.success(), "{:?}", checked.stderr);
    assert_eq!(
        std::str::from_utf8(&checked.stdout).unwrap().trim(),
        BASELINE_REPORT
    );

    let prepared = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("prepare")
        .arg(fixture.path())
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
    let mut project: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    project["cases"][0]["expected"]["obligations"] = serde_json::json!([]);
    let fixture =
        TemporaryProject::materialize("failed-case", &serde_json::to_vec(&project).unwrap());

    let refused = Command::new(env!("CARGO_BIN_EXE_policy-cli"))
        .arg("prepare")
        .arg(fixture.path())
        .output()
        .unwrap();
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
