// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// CLI invariants against disposable fixture trees.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn capacity_model_gate_accepts_tenant_class_delta_manifest() {
    let temp = temp_dir("capacity-model-valid-deltas");
    let manifest = temp.join("workflow/workflow-engine/manifest.json");
    write_manifest(&manifest, valid_capacity_manifest());

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "capacity-model-manifest",
            "--microservices-root",
            temp.join("oya").to_str().expect("utf8 root"),
            "--require-tenant-class-deltas",
        ])
        .output()
        .expect("capacity-model gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "capacity-model-manifest passed: 1 manifests, 1 capacity_model blocks, 1 tenant_class_deltas"
        ),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn capacity_model_gate_rejects_invalid_units_and_enums() {
    let temp = temp_dir("capacity-model-invalid-units");
    let manifest = temp.join("workflow/workflow-engine/manifest.json");
    write_manifest(
        &manifest,
        r#"{
  "microservice": "workflow-engine",
  "capacity_model": {
    "baseline_cpu_per_tenant": "0.1",
    "baseline_ram_per_tenant": 768.5,
    "storage_per_tenant": -1,
    "connections_per_tenant": {
      "valkey": 2,
      "postgres": 4,
      "outbound_http": "12"
    },
    "scaling_dimension": "per_token",
    "cell_placement_class": "Tier-9"
  }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "capacity-model-manifest",
            "--microservices-root",
            temp.join("oya").to_str().expect("utf8 root"),
        ])
        .output()
        .expect("capacity-model gate command runs");

    assert!(
        !output.status.success(),
        "invalid units/enums must fail; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("capacity-model-manifest FAILED")
            && stderr.contains("baseline_cpu_per_tenant must be a JSON number")
            && stderr.contains("scaling_dimension must be one of"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn capacity_model_gate_requires_tenant_class_deltas_when_requested() {
    let temp = temp_dir("capacity-model-require-deltas");
    let manifest = temp.join("workflow/workflow-engine/manifest.json");
    write_manifest(
        &manifest,
        r#"{
  "microservice": "workflow-engine",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.6,
    "baseline_ram_per_tenant": 768,
    "storage_per_tenant": 12,
    "connections_per_tenant": {
      "valkey": 4,
      "postgres": 6,
      "outbound_http": 12
    },
    "scaling_dimension": "per_workflow_run",
    "cell_placement_class": "Tier-2"
  }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "capacity-model-manifest",
            "--microservices-root",
            temp.join("oya").to_str().expect("utf8 root"),
            "--require-tenant-class-deltas",
        ])
        .output()
        .expect("capacity-model gate command runs");

    assert!(
        !output.status.success(),
        "missing tenant_class_deltas must fail when required; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("capacity_model.tenant_class_deltas is required by this gate invocation"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn capacity_model_gate_ignores_non_runtime_placeholders_during_root_scan() {
    let temp = temp_dir("capacity-model-skip-placeholder");
    let runtime_manifest = temp.join("workflow/workflow-engine/manifest.json");
    write_manifest(&runtime_manifest, valid_capacity_manifest());
    let placeholder_manifest = temp.join("cloud/cloud-iac/manifest.json");
    write_manifest(
        &placeholder_manifest,
        r#"{
  "microservice": "cloud-iac",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.0,
    "baseline_ram_per_tenant": 0,
    "storage_per_tenant": 0,
    "connections_per_tenant": {},
    "scaling_dimension": "not_claimed_runtime",
    "cell_placement_class": "Tier-1",
    "notes": "Pure domain/metadata foundation; runtime OpenTofu execution, measured capacity, autoscaling, and SLOs remain explicit non-claims."
  }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "capacity-model-manifest",
            "--microservices-root",
            temp.join("oya").to_str().expect("utf8 oya root"),
            "--microservices-root",
            temp.join("cloud").to_str().expect("utf8 cloud root"),
        ])
        .output()
        .expect("capacity-model gate command runs");

    assert!(
        output.status.success(),
        "non-runtime placeholders should not block the workload-producing manifest slice; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("capacity-model-manifest passed: 2 manifests, 1 capacity_model blocks"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

fn valid_capacity_manifest() -> &'static str {
    r#"{
  "microservice": "workflow-engine",
  "capacity_model": {
    "baseline_cpu_per_tenant": 0.6,
    "baseline_ram_per_tenant": 768,
    "storage_per_tenant": 12,
    "connections_per_tenant": {
      "valkey": 4,
      "postgres": 6,
      "outbound_http": 12
    },
    "scaling_dimension": "per_workflow_run",
    "cell_placement_class": "Tier-2",
    "tenant_class_deltas": {
      "demo_trial": {
        "baseline_cpu_per_tenant": 0.12,
        "baseline_ram_per_tenant": 192,
        "storage_per_tenant": 2,
        "connections_per_tenant": {
          "valkey": 1,
          "postgres": 2,
          "outbound_http": 4
        },
        "scaling_dimension": "per_workflow_run",
        "cell_placement_class": "Tier-2"
      },
      "paid": {
        "baseline_cpu_per_tenant": 0.6,
        "baseline_ram_per_tenant": 768,
        "storage_per_tenant": 12,
        "connections_per_tenant": {
          "valkey": 4,
          "postgres": 6,
          "outbound_http": 12
        },
        "scaling_dimension": "per_workflow_run",
        "cell_placement_class": "Tier-2"
      }
    }
  }
}"#
}

fn write_manifest(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("manifest parent")).expect("manifest dir created");
    fs::write(path, contents).expect("manifest written");
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oyatie-{prefix}-{nanos}"))
}
