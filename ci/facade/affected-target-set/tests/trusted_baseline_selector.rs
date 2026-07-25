//! Process-boundary tests for the GH #1323 trusted baseline selector.
//!
//! Pure selection behavior is unit-tested in the library. These tests prove the
//! Buck-built binary's argument, file-I/O, exit-code, and JSON-output contract.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const RUN_ID: u64 = 30_144_110_793;
const REPOSITORY_ID: u64 = 1_236_575_706;

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-trusted-baseline-selector-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create fixture directory");
        Self { root }
    }

    fn write_json(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.root.join(name);
        fs::write(
            &path,
            serde_json::to_vec_pretty(value).expect("serialize fixture"),
        )
        .expect("write fixture");
        path
    }

    fn run(&self, runs: &Value, artifacts: &Value) -> Output {
        let runs_path = self.write_json("runs.json", runs);
        let artifacts_path = self.write_json("artifacts.json", artifacts);
        Command::new(selector_binary())
            .arg("--merge-base-sha")
            .arg(SHA)
            .arg("--workflow-runs-json")
            .arg(runs_path)
            .arg("--workflow-artifacts-json")
            .arg(artifacts_path)
            .output()
            .expect("run selector")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn selector_binary() -> PathBuf {
    let value = std::env::var_os("OYA_TRUSTED_BASELINE_SELECTOR_BIN")
        .or_else(|| {
            option_env!("CARGO_BIN_EXE_oya-cloud-ci-trusted-baseline-selector")
                .map(std::ffi::OsString::from)
        })
        .expect(
            "Buck must provide OYA_TRUSTED_BASELINE_SELECTOR_BIN or Cargo must provide \
             CARGO_BIN_EXE_oya-cloud-ci-trusted-baseline-selector",
        );
    let path = Path::new(&value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().expect("current_dir").join(path)
    }
}

fn trusted_run() -> Value {
    json!({
        "id": RUN_ID,
        "head_sha": SHA,
        "event": "push",
        "head_branch": "dev",
        "status": "completed",
        "conclusion": "success",
        "path": ".github/workflows/oya-ci-required.yml",
        "repository": {"id": REPOSITORY_ID},
        "head_repository": {"id": REPOSITORY_ID}
    })
}

fn workflow_runs() -> Value {
    json!({"workflow_runs": [trusted_run()]})
}

fn artifact(id: u64, name: String) -> Value {
    json!({
        "id": id,
        "name": name,
        "expired": false,
        "workflow_run": {
            "id": RUN_ID,
            "head_sha": SHA,
            "head_branch": "dev",
            "repository_id": REPOSITORY_ID,
            "head_repository_id": REPOSITORY_ID
        }
    })
}

fn artifact_pair() -> Value {
    json!({
        "artifacts": [
            artifact(41, format!("build-health-baseline-{SHA}")),
            artifact(42, format!("test-health-baseline-{SHA}"))
        ]
    })
}

#[test]
fn exact_trusted_pair_is_selected() {
    let fixture = Fixture::new("selected");
    let output = fixture.run(&workflow_runs(), &artifact_pair());
    assert!(
        output.status.success(),
        "selector failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("selector JSON");
    assert_eq!(receipt["schema_version"], 1);
    assert_eq!(receipt["decision"], "SELECTED");
    assert_eq!(receipt["merge_base_sha"], SHA);
    assert_eq!(receipt["run_id"], RUN_ID);
    assert_eq!(receipt["repository_id"], REPOSITORY_ID);
    assert_eq!(receipt["build_artifact"]["id"], 41);
    assert_eq!(receipt["test_artifact"]["id"], 42);
}

#[test]
fn incomplete_pair_falls_back_without_becoming_an_error() {
    let fixture = Fixture::new("fallback");
    let artifacts = json!({
        "artifacts": [artifact(41, format!("build-health-baseline-{SHA}"))]
    });
    let output = fixture.run(&workflow_runs(), &artifacts);
    assert!(
        output.status.success(),
        "fallback must be an expected success path: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("selector JSON");
    assert_eq!(receipt["decision"], "FALLBACK");
    assert_eq!(receipt["merge_base_sha"], SHA);
    assert!(
        receipt["reason"]
            .as_str()
            .expect("fallback reason")
            .contains("atomic trusted BUILD + TEST baseline pair")
    );
}

#[test]
fn mismatched_embedded_run_provenance_is_an_error() {
    let fixture = Fixture::new("provenance-error");
    let mut artifacts = artifact_pair();
    artifacts["artifacts"][1]["workflow_run"]["id"] = json!(RUN_ID + 1);
    let output = fixture.run(&workflow_runs(), &artifacts);
    assert!(
        !output.status.success(),
        "provenance mismatch must fail closed"
    );
    let receipt: Value = serde_json::from_slice(&output.stderr).expect("error JSON");
    assert_eq!(receipt["decision"], "ERROR");
    assert!(
        receipt["error"]
            .as_str()
            .expect("error message")
            .contains("workflow run")
    );
}

#[test]
fn provenance_boundaries_have_explicit_fallback_or_error_dispositions() {
    struct Case {
        name: &'static str,
        runs: Value,
        artifacts: Value,
        expected_decision: &'static str,
        expected_message: &'static str,
    }

    let mut wrong_workflow = workflow_runs();
    wrong_workflow["workflow_runs"][0]["path"] = json!(".github/workflows/untrusted.yml");

    let mut incomplete_run = workflow_runs();
    incomplete_run["workflow_runs"][0]["status"] = json!("in_progress");

    let mut fork_head = workflow_runs();
    fork_head["workflow_runs"][0]["head_repository"]["id"] = json!(REPOSITORY_ID + 1);

    let mut wrong_artifact_sha = artifact_pair();
    wrong_artifact_sha["artifacts"][0]["workflow_run"]["head_sha"] =
        json!("fedcba9876543210fedcba9876543210fedcba98");

    let mut wrong_artifact_branch = artifact_pair();
    wrong_artifact_branch["artifacts"][0]["workflow_run"]["head_branch"] = json!("feature");

    let mut duplicate_artifact = artifact_pair();
    let duplicate = duplicate_artifact["artifacts"][0].clone();
    duplicate_artifact["artifacts"]
        .as_array_mut()
        .expect("artifact array")
        .push(duplicate);

    let cases = [
        Case {
            name: "wrong-workflow",
            runs: wrong_workflow,
            artifacts: artifact_pair(),
            expected_decision: "FALLBACK",
            expected_message: "atomic trusted BUILD + TEST baseline pair",
        },
        Case {
            name: "incomplete-run",
            runs: incomplete_run,
            artifacts: artifact_pair(),
            expected_decision: "FALLBACK",
            expected_message: "atomic trusted BUILD + TEST baseline pair",
        },
        Case {
            name: "fork-head",
            runs: fork_head,
            artifacts: artifact_pair(),
            expected_decision: "ERROR",
            expected_message: "does not match trusted repository",
        },
        Case {
            name: "wrong-artifact-sha",
            runs: workflow_runs(),
            artifacts: wrong_artifact_sha,
            expected_decision: "ERROR",
            expected_message: "head SHA",
        },
        Case {
            name: "wrong-artifact-branch",
            runs: workflow_runs(),
            artifacts: wrong_artifact_branch,
            expected_decision: "ERROR",
            expected_message: "trusted `dev` branch",
        },
        Case {
            name: "duplicate-artifact",
            runs: workflow_runs(),
            artifacts: duplicate_artifact,
            expected_decision: "ERROR",
            expected_message: "duplicate exact-name",
        },
        Case {
            name: "missing-artifacts-array",
            runs: workflow_runs(),
            artifacts: json!({"not_artifacts": []}),
            expected_decision: "ERROR",
            expected_message: "`artifacts` array",
        },
    ];

    for case in cases {
        let fixture = Fixture::new(case.name);
        let output = fixture.run(&case.runs, &case.artifacts);
        let bytes = if case.expected_decision == "ERROR" {
            assert!(!output.status.success(), "{} must fail closed", case.name);
            &output.stderr
        } else {
            assert!(
                output.status.success(),
                "{} must take the cold fallback: {}",
                case.name,
                String::from_utf8_lossy(&output.stderr)
            );
            &output.stdout
        };
        let receipt: Value = serde_json::from_slice(bytes).expect("decision JSON");
        assert_eq!(
            receipt["decision"], case.expected_decision,
            "{} disposition",
            case.name
        );
        let message_field = if case.expected_decision == "ERROR" {
            "error"
        } else {
            "reason"
        };
        assert!(
            receipt[message_field]
                .as_str()
                .expect("decision message")
                .contains(case.expected_message),
            "{} receipt: {receipt}",
            case.name
        );
    }
}
