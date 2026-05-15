// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn foundry_capability_schema_gate_projects_yaml_into_typed_capability_contract() {
    let temp = temp_dir("foundry-capability-schema-valid");
    write_valid_capability_record(&temp);

    let output = run_gate(&temp);

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "foundry capability schema validation passed: 1 capabilities, 1 mcp contracts, 2 schemas"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_capability_schema_gate_rejects_missing_agent_description() {
    let temp = temp_dir("foundry-capability-schema-missing-agent-description");
    write_valid_capability_record(&temp);
    let mut record = fs::read_to_string(temp.join("cap.demo.readiness.yaml")).unwrap();
    record = record.replace("  agent_readable: Run demo readiness.\n", "");
    fs::write(temp.join("cap.demo.readiness.yaml"), record).unwrap();

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("missing required field description.agent_readable")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundry_capability_schema_gate_rejects_non_object_input_schema() {
    let temp = temp_dir("foundry-capability-schema-non-object-schema");
    write_valid_capability_record(&temp);
    let mut record = fs::read_to_string(temp.join("cap.demo.readiness.yaml")).unwrap();
    record = record.replace(
        "  type: object\n  properties:\n",
        "  type: array\n  properties:\n",
    );
    fs::write(temp.join("cap.demo.readiness.yaml"), record).unwrap();

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("input_schema must declare type: object")
    );

    fs::remove_dir_all(temp).ok();
}

fn write_valid_capability_record(root: &Path) {
    fs::create_dir_all(root).expect("capabilities dir created");
    fs::write(
        root.join("cap.demo.readiness.yaml"),
        r#"id: cap.demo.readiness
namespace: foundry.demo
status: published
version: 0.1.0
description:
  agent_readable: Run demo readiness.
  human_readable: Human demo readiness guide.
provider:
  preferred: foundation-local
  fallback: [openai-api]
autonomy_tier_required: T1
data_classes_touched: [INTERNAL_ONLY]
evidence_emission_topic: foundry.capability.invoke:cap.demo.readiness
cost_profile:
  per_invocation_budget_usd: 0.05
  monthly_budget_usd: 100
input_schema:
  type: object
  properties:
    release_id:
      type: string
  required: [release_id]
output_schema:
  type: object
  properties:
    verdict:
      type: string
  required: [verdict]
eval_set: eval-sets/cap.demo.readiness.yaml
eval_run: eval-runs/cap.demo.readiness.yaml
owner_team: axis-foundry
"#,
    )
    .expect("capability record written");
}

fn run_gate(capabilities_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundry-capability-schema",
            "--capabilities-dir",
            capabilities_dir.to_str().expect("utf8 capabilities dir"),
        ])
        .output()
        .expect("foundry capability schema gate command runs")
}

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{name}-{nonce}"))
}
