// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn foundry_eval_gate_accepts_signed_passing_eval_artifacts() {
    let temp = temp_dir("foundry-eval-valid");
    write_capability_eval_fixture(&temp, true, "95", true);

    let output = run_gate(&temp);

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("foundry eval validation passed: 1 capabilities, 7 cases, 1 passing runs")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_missing_adversarial_coverage() {
    let temp = temp_dir("foundry-eval-missing-adversarial");
    write_capability_eval_fixture(&temp, false, "95", true);

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MissingAdversarialCoverage"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_below_threshold_run() {
    let temp = temp_dir("foundry-eval-failing-run");
    write_capability_eval_fixture(&temp, true, "70", true);

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("EvalRunBelowThreshold"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_unsigned_run() {
    let temp = temp_dir("foundry-eval-unsigned-run");
    write_capability_eval_fixture(&temp, true, "95", false);

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("UnsignedEvalRun"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_impossible_percentages() {
    let temp = temp_dir("foundry-eval-impossible-percent");
    write_capability_eval_fixture(&temp, true, "101", true);

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("pass_rate_percent must be 0-100"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_duplicate_scalar_fields() {
    let temp = temp_dir("foundry-eval-duplicate-scalar");
    write_capability_eval_fixture(&temp, true, "95", true);
    fs::write(
        temp.join("eval-runs/cap.demo.readiness.yaml"),
        r#"capability_id: cap.demo.readiness
eval_set_version: eval-v1
pass_rate_percent: 101
pass_rate_percent: 95
p95_score_percent: 90
adversarial_passed: true
linguistic_passed: true
signed: true
"#,
    )
    .expect("eval run with duplicate scalar written");

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate field pass_rate_percent"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_duplicate_capability_status() {
    let temp = temp_dir("foundry-eval-duplicate-capability-status");
    write_capability_eval_fixture(&temp, true, "95", true);
    fs::write(
        temp.join("cap.demo.readiness.yaml"),
        r#"id: cap.demo.readiness
namespace: foundry.demo
status: published
status: draft
version: 0.1.0
eval_set: eval-sets/cap.demo.readiness.yaml
eval_run: eval-runs/cap.demo.readiness.yaml
owner_team: axis-foundry
"#,
    )
    .expect("capability record with duplicate status written");

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate field status"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_rejects_duplicate_capability_eval_run() {
    let temp = temp_dir("foundry-eval-duplicate-capability-eval-run");
    write_capability_eval_fixture(&temp, true, "95", true);
    fs::write(
        temp.join("cap.demo.readiness.yaml"),
        r#"id: cap.demo.readiness
namespace: foundry.demo
status: published
version: 0.1.0
eval_set: eval-sets/cap.demo.readiness.yaml
eval_run: eval-runs/cap.demo.readiness.yaml
eval_run: eval-runs/other.yaml
owner_team: axis-foundry
"#,
    )
    .expect("capability record with duplicate eval_run written");

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate field eval_run"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_eval_gate_allows_nested_schema_keys_matching_root_metadata() {
    let temp = temp_dir("foundry-eval-nested-schema-id");
    write_capability_eval_fixture(&temp, true, "95", true);
    fs::write(
        temp.join("cap.demo.readiness.yaml"),
        r#"id: cap.demo.readiness
namespace: foundry.demo
status: published
version: 0.1.0
eval_set: eval-sets/cap.demo.readiness.yaml
eval_run: eval-runs/cap.demo.readiness.yaml
owner_team: axis-foundry
input_schema:
  properties:
    id:
      type: string
output_schema:
  properties:
    status:
      type: string
"#,
    )
    .expect("capability record with nested schema keys written");

    let output = run_gate(&temp);

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

fn write_capability_eval_fixture(
    root: &Path,
    include_all_adversarial: bool,
    pass_rate_percent: &str,
    run_signed: bool,
) {
    fs::create_dir_all(root.join("eval-sets")).expect("eval sets dir created");
    fs::create_dir_all(root.join("eval-runs")).expect("eval runs dir created");
    fs::write(
        root.join("cap.demo.readiness.yaml"),
        r#"id: cap.demo.readiness
namespace: foundry.demo
status: published
version: 0.1.0
eval_set: eval-sets/cap.demo.readiness.yaml
eval_run: eval-runs/cap.demo.readiness.yaml
owner_team: axis-foundry
"#,
    )
    .expect("capability record written");

    let tool_case = if include_all_adversarial {
        r#"  - case_id: adv-tool
    locale: en-US
    input_ref: eval://tool
    expected_ref: eval://tool/expected
    adversarial_kind: ToolExfiltration
"#
    } else {
        ""
    };
    fs::write(
        root.join("eval-sets/cap.demo.readiness.yaml"),
        format!(
            r#"capability_id: cap.demo.readiness
version: eval-v1
metric: ExactMatch
min_pass_rate_percent: 80
min_p95_score_percent: 80
signed: true
cases:
  - case_id: case-en
    locale: en-US
    input_ref: eval://en
    expected_ref: eval://en/expected
  - case_id: case-ko
    locale: ko-KR
    input_ref: eval://ko
    expected_ref: eval://ko/expected
  - case_id: case-ja
    locale: ja-JP
    input_ref: eval://ja
    expected_ref: eval://ja/expected
  - case_id: adv-prompt
    locale: en-US
    input_ref: eval://prompt
    expected_ref: eval://prompt/expected
    adversarial_kind: PromptInjection
  - case_id: adv-class
    locale: en-US
    input_ref: eval://class
    expected_ref: eval://class/expected
    adversarial_kind: DataClassViolation
  - case_id: adv-autonomy
    locale: en-US
    input_ref: eval://autonomy
    expected_ref: eval://autonomy/expected
    adversarial_kind: AutonomyBypass
{tool_case}"#
        ),
    )
    .expect("eval set written");

    fs::write(
        root.join("eval-runs/cap.demo.readiness.yaml"),
        format!(
            r#"capability_id: cap.demo.readiness
eval_set_version: eval-v1
pass_rate_percent: {pass_rate_percent}
p95_score_percent: 90
adversarial_passed: true
linguistic_passed: true
signed: {run_signed}
"#
        ),
    )
    .expect("eval run written");
}

fn run_gate(capabilities_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundry-eval",
            "--capabilities-dir",
            capabilities_dir.to_str().expect("utf8 capabilities dir"),
        ])
        .output()
        .expect("foundry eval gate command runs")
}

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{name}-{nonce}"))
}
