// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn masterplan_drift_gate_accepts_controller_materialized_output_without_committed_byte_match() {
    let temp = TempDirGuard::new("masterplan-drift-controller");
    let decisions_dir = temp.path().join("docs/decisions");
    fs::create_dir_all(&decisions_dir).expect("decisions dir created");
    fs::write(
        decisions_dir.join("ADR-2000-controller-masterplan.md"),
        r#"---
status: Accepted
planning_impact: true
milestone: M-CONTROLLER
depends_on: []
deliverables:
  - id: ADR-2000-D1
    description: "controller-owned masterplan projection"
    exit_criteria: "source ADR projection regenerates successfully"
    verified_by: "buck2 run //marketplace/facade/dev-cli:oya -- gate validate masterplan-drift"
---
# ADR-2000
"#,
    )
    .expect("ADR fixture written");

    let output_path = temp.path().join("docs/machine-readable");
    fs::create_dir_all(&output_path).expect("machine-readable dir created");
    fs::write(
        output_path.join("masterplan.generated.json"),
        "{\"stale\":\"committed bytes intentionally do not match regenerated projection\"}\n",
    )
    .expect("stale projection fixture written");

    let registry_dir = temp.path().join("registry");
    fs::create_dir_all(&registry_dir).expect("registry dir created");
    fs::write(
        registry_dir.join("generated-artifact-control-plane.json"),
        r#"{
  "artifacts": [
    {
      "path": "docs/machine-readable/masterplan.generated.json",
      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
      "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
      "generator": {
        "runner": "oya-ci-native-controller",
        "generator_target": "oya-ci://generated-artifact-controller/planning/masterplan",
        "output_mode": "controller-materialized"
      }
    }
  ]
}
"#,
    )
    .expect("generated artifact control-plane fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "masterplan-drift"])
        .current_dir(temp.path())
        .output()
        .expect("masterplan drift command runs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(
        stdout.contains("controller-materialized")
            && stdout.contains("source projection regenerated successfully"),
        "stdout={stdout}"
    );
}

#[test]
fn masterplan_drift_gate_rejects_stale_bytes_without_exact_controller_output_mode() {
    let temp = TempDirGuard::new("masterplan-drift-controller-fail-closed");
    let decisions_dir = temp.path().join("docs/decisions");
    fs::create_dir_all(&decisions_dir).expect("decisions dir created");
    fs::write(
        decisions_dir.join("ADR-2001-controller-masterplan.md"),
        r#"---
status: Accepted
planning_impact: true
milestone: M-CONTROLLER
depends_on: []
deliverables:
  - id: ADR-2001-D1
    description: "controller-owned masterplan projection"
    exit_criteria: "source ADR projection regenerates successfully"
    verified_by: "buck2 run //marketplace/facade/dev-cli:oya -- gate validate masterplan-drift"
---
# ADR-2001
"#,
    )
    .expect("ADR fixture written");

    let output_path = temp.path().join("docs/machine-readable");
    fs::create_dir_all(&output_path).expect("machine-readable dir created");
    fs::write(
        output_path.join("masterplan.generated.json"),
        "{\"stale\":\"committed bytes intentionally do not match regenerated projection\"}\n",
    )
    .expect("stale projection fixture written");

    let registry_dir = temp.path().join("registry");
    fs::create_dir_all(&registry_dir).expect("registry dir created");
    fs::write(
        registry_dir.join("generated-artifact-control-plane.json"),
        r#"{
  "artifacts": [
    {
      "path": "docs/machine-readable/masterplan.generated.json",
      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
      "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
      "generator": {
        "runner": "oya-ci-native-controller",
        "generator_target": "oya-ci://generated-artifact-controller/planning/masterplan",
        "output_mode": "branch-committed"
      }
    }
  ]
}
"#,
    )
    .expect("generated artifact control-plane fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "masterplan-drift"])
        .current_dir(temp.path())
        .output()
        .expect("masterplan drift command runs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(
        stderr.contains("gen masterplan --check failed")
            && stderr.contains("drifted from the regenerated projection"),
        "stderr={stderr}"
    );
}

#[test]
fn foundation_bypass_gate_allows_empty_or_fresh_ledgers() {
    let temp = temp_dir("gate-fresh");
    fs::create_dir_all(&temp).expect("ledger dir created");
    fs::write(
        temp.join("byp_0001.yaml"),
        "id: byp_0001\npr_ref: gh:oyatie/oyatie#123\ncrate_ref: oya-intelligence-capability-kernel\ngate_bypassed: architecture\nbypassing_actor: usr_architect\nrationale: temporary foundation sequencing gap\nregression_window_days: 10\ncreated_at_epoch_days: 10\n",
    )
    .expect("bypass record written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            temp.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("foundation gate exception ledger validation passed: 1 records, 1 open")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundation_bypass_gate_rejects_zero_window_and_expired_records() {
    let temp = temp_dir("gate-expired");
    fs::create_dir_all(&temp).expect("ledger dir created");
    fs::write(
        temp.join("byp_0002.yaml"),
        "id: byp_0002\npr_ref: gh:oyatie/oyatie#124\ncrate_ref: oya-intelligence-capability-kernel\ngate_bypassed: architecture\nbypassing_actor: usr_architect\nrationale: temporary foundation sequencing gap\nregression_window_days: 0\ncreated_at_epoch_days: 10\n",
    )
    .expect("bypass record written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            temp.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("foundation gate exception ledger validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundation_bypass_gate_requires_an_explicit_ledger_directory() {
    let temp = temp_dir("gate-missing-ledger");
    let missing_ledger = temp.join("missing-ledger");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            missing_ledger.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("foundation gate exception ledger validation failed")
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("directory unreadable"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundation_bypass_gate_rejects_malformed_or_unknown_record_fields() {
    let malformed = temp_dir("gate-malformed-bypass");
    fs::create_dir_all(&malformed).expect("ledger dir created");
    fs::write(
        malformed.join("byp_malformed.yaml"),
        "id: byp_malformed\nthis line is not yaml-ish\npr_ref: gh:oyatie/oyatie#125\ncrate_ref: oya-intelligence-capability-kernel\ngate_bypassed: architecture\nbypassing_actor: usr_architect\nrationale: strict parse fixture\nregression_window_days: 10\ncreated_at_epoch_days: 10\n",
    )
    .expect("malformed bypass record written");

    let malformed_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            malformed.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(!malformed_output.status.success());
    assert!(String::from_utf8_lossy(&malformed_output.stderr).contains("malformed line"));

    let unknown = temp_dir("gate-unknown-bypass-field");
    fs::create_dir_all(&unknown).expect("ledger dir created");
    fs::write(
        unknown.join("byp_unknown.yaml"),
        "id: byp_unknown\npr_ref: gh:oyatie/oyatie#126\ncrate_ref: oya-intelligence-capability-kernel\ngate_bypassed: architecture\nbypassing_actor: usr_architect\nrationale: strict parse fixture\nregression_window_days: 10\ncreated_at_epoch_days: 10\nsurprise_field: no\n",
    )
    .expect("unknown-field bypass record written");

    let unknown_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            unknown.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(!unknown_output.status.success());
    assert!(String::from_utf8_lossy(&unknown_output.stderr).contains("unknown field"));

    fs::remove_dir_all(malformed).ok();
    fs::remove_dir_all(unknown).ok();
}

#[test]
fn foundation_bypass_gate_rejects_duplicate_record_fields() {
    let temp = temp_dir("gate-duplicate-bypass-field");
    fs::create_dir_all(&temp).expect("ledger dir created");
    fs::write(
        temp.join("byp_duplicate.yaml"),
        "id: byp_duplicate\npr_ref: gh:oyatie/oyatie#127\ncrate_ref: oya-intelligence-capability-kernel\ngate_bypassed: architecture\nbypassing_actor: usr_architect\nrationale: strict parse fixture\nregression_window_days: 0\nregression_window_days: 10\ncreated_at_epoch_days: 10\n",
    )
    .expect("duplicate-field bypass record written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            temp.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "19",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("duplicate field regression_window_days"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundation_bypass_gate_validates_autonomy_break_glass_records_in_same_ledger() {
    let temp = temp_dir("gate-break-glass");
    fs::create_dir_all(&temp).expect("ledger dir created");
    fs::write(
        temp.join("abg_0001.yaml"),
        "entry_class: autonomy-break-glass\nid: abg_0001\ntenant_id: ten_healthcare\ncapability_id: cap.clinical.assist\nrequested_tier: T4AutoExecute\npermitted_tier: T4AutoExecute\nrequesting_actor: usr_operator\napproving_actors: usr_security,usr_privacy\napproval_quorum: two-of-three\nrationale: patient safety emergency with explicit expiry\ncreated_at_epoch_days: 10\nexpires_at_epoch_days: 12\n",
    )
    .expect("break-glass record written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            temp.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "12",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("foundation gate exception ledger validation passed: 1 records, 1 open")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn foundation_bypass_gate_rejects_expired_or_underapproved_break_glass_records() {
    let expired = temp_dir("gate-expired-break-glass");
    fs::create_dir_all(&expired).expect("ledger dir created");
    fs::write(
        expired.join("abg_0002.yaml"),
        "entry_class: autonomy-break-glass\nid: abg_0002\ntenant_id: ten_healthcare\ncapability_id: cap.clinical.assist\nrequested_tier: T4AutoExecute\npermitted_tier: T4AutoExecute\nrequesting_actor: usr_operator\napproving_actors: usr_security,usr_privacy\napproval_quorum: two-of-three\nrationale: patient safety emergency with explicit expiry\ncreated_at_epoch_days: 10\nexpires_at_epoch_days: 12\n",
    )
    .expect("break-glass record written");

    let expired_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            expired.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "13",
        ])
        .output()
        .expect("gate command runs");

    assert!(!expired_output.status.success());
    assert!(
        String::from_utf8_lossy(&expired_output.stderr).contains("ExpiredBypass"),
        "stderr={}",
        String::from_utf8_lossy(&expired_output.stderr)
    );

    let underapproved = temp_dir("gate-underapproved-break-glass");
    fs::create_dir_all(&underapproved).expect("ledger dir created");
    fs::write(
        underapproved.join("abg_0003.yaml"),
        "entry_class: autonomy-break-glass\nid: abg_0003\ntenant_id: ten_healthcare\ncapability_id: cap.clinical.assist\nrequested_tier: T4AutoExecute\npermitted_tier: T4AutoExecute\nrequesting_actor: usr_operator\napproving_actors: usr_security\napproval_quorum: two-of-three\nrationale: patient safety emergency with explicit expiry\ncreated_at_epoch_days: 10\nexpires_at_epoch_days: 12\n",
    )
    .expect("break-glass record written");

    let underapproved_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "foundation-bypass",
            "--ledger",
            underapproved.to_str().expect("utf8 ledger"),
            "--now-epoch-days",
            "11",
        ])
        .output()
        .expect("gate command runs");

    assert!(!underapproved_output.status.success());
    assert!(
        String::from_utf8_lossy(&underapproved_output.stderr)
            .contains("InsufficientBreakGlassApprovals"),
        "stderr={}",
        String::from_utf8_lossy(&underapproved_output.stderr)
    );

    fs::remove_dir_all(expired).ok();
    fs::remove_dir_all(underapproved).ok();
}

#[test]
fn plane_class_gate_accepts_stable_catalog_planes() {
    let temp = temp_dir("plane-stable");
    let baseline = temp.join("baseline");
    let current = temp.join("current");
    write_catalog_record(&baseline, "oya-intelligence-capability-kernel", "control");
    write_catalog_record(&current, "oya-intelligence-capability-kernel", "control");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "plane-class",
            "--registry",
            current.to_str().expect("utf8 current registry"),
            "--baseline",
            baseline.to_str().expect("utf8 baseline registry"),
        ])
        .output()
        .expect("plane gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("plane class validation passed: 1 records, 0 reviewed changes")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plane_class_gate_rejects_unreviewed_plane_changes() {
    let temp = temp_dir("plane-change");
    let baseline = temp.join("baseline");
    let current = temp.join("current");
    write_catalog_record(&baseline, "oya-intelligence-capability-kernel", "control");
    write_catalog_record(&current, "oya-intelligence-capability-kernel", "data");

    let rejected = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "plane-class",
            "--registry",
            current.to_str().expect("utf8 current registry"),
            "--baseline",
            baseline.to_str().expect("utf8 baseline registry"),
        ])
        .output()
        .expect("plane gate command runs");

    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("plane class validation failed"));

    let reviewed = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "plane-class",
            "--registry",
            current.to_str().expect("utf8 current registry"),
            "--baseline",
            baseline.to_str().expect("utf8 baseline registry"),
            "--reviewed-change",
            "oya-intelligence-capability-kernel",
        ])
        .output()
        .expect("plane gate command runs");

    assert!(
        reviewed.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&reviewed.stdout),
        String::from_utf8_lossy(&reviewed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&reviewed.stdout)
            .contains("plane class validation passed: 1 records, 1 reviewed changes")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn claim_ceiling_gate_rejects_catalog_claims_above_foundation() {
    let temp = temp_dir("claim-ceiling");
    write_catalog_record_with_claim(
        &temp,
        "oya-intelligence-capability-kernel",
        "control",
        "stable",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "claim-ceiling",
            "--registry",
            temp.to_str().expect("utf8 registry"),
        ])
        .output()
        .expect("claim gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("claim ceiling validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn license_policy_gate_rejects_forbidden_workspace_crate_license() {
    let temp = temp_dir("license-policy");
    let crate_dir = temp.join("crates/oya-intelligence-capability-kernel");
    fs::create_dir_all(&crate_dir).expect("crate dir created");
    fs::write(
        temp.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/oya-intelligence-capability-kernel"]
"#,
    )
    .expect("workspace manifest written");
    fs::write(
        crate_dir.join("Cargo.toml"),
        r#"[package]
name = "oya-intelligence-capability-kernel"
edition = "2024"
version = "0.1.0"
license = "GPL-3.0"
"#,
    )
    .expect("crate manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "license-policy",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
        ])
        .output()
        .expect("license gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("license policy validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dependency_seam_gate_accepts_report_only_fixture_and_emits_report() {
    let temp = temp_dir("dependency-seam-valid");
    write_dependency_seam_gate_fixture(&temp, false);
    let report = temp.join("dependency-seam-report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(dependency_seam_gate_args(&temp, &report, "report-only"))
        .output()
        .expect("dependency seam gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("dependency-seam validation passed"));
    assert!(report.exists(), "dependency seam report should be emitted");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dependency_seam_gate_rejects_strict_import_violation() {
    let temp = temp_dir("dependency-seam-strict-violation");
    write_dependency_seam_gate_fixture(&temp, true);
    let report = temp.join("dependency-seam-report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(dependency_seam_gate_args(&temp, &report, "error"))
        .output()
        .expect("dependency seam gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("blocking diagnostics"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        report.exists(),
        "strict failure still emits diagnostic report"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dependency_blessed_allowlist_gate_reports_unblessed_dep_per_crate_without_failing() {
    let temp = temp_dir("dependency-blessed-allowlist-report");
    write_dependency_blessed_allowlist_fixture(&temp, true);
    let report = temp.join("blessed-report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(dependency_blessed_allowlist_args(&temp, &report, false))
        .output()
        .expect("dependency-blessed-allowlist gate command runs");

    assert!(
        output.status.success(),
        "report-only must not fail; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("offender-adapter")
            && stdout.contains("sketchy-unblessed-crate")
            && stdout.contains("crates/offender/Cargo.toml"),
        "per-crate finding must name crate + dep + path; stdout={stdout}"
    );
    assert!(stdout.contains("report-only"));
    assert!(report.exists(), "report should be emitted");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dependency_blessed_allowlist_gate_fails_under_enforce_on_unblessed_dep() {
    let temp = temp_dir("dependency-blessed-allowlist-enforce");
    write_dependency_blessed_allowlist_fixture(&temp, true);
    let report = temp.join("blessed-report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(dependency_blessed_allowlist_args(&temp, &report, true))
        .output()
        .expect("dependency-blessed-allowlist gate command runs");

    assert!(
        !output.status.success(),
        "enforce must fail on unblessed dep"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unblessed direct dependencies"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dependency_blessed_allowlist_gate_passes_under_enforce_when_all_blessed() {
    let temp = temp_dir("dependency-blessed-allowlist-clean");
    write_dependency_blessed_allowlist_fixture(&temp, false);
    let report = temp.join("blessed-report.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(dependency_blessed_allowlist_args(&temp, &report, true))
        .output()
        .expect("dependency-blessed-allowlist gate command runs");

    assert!(
        output.status.success(),
        "all-blessed crate must pass even under enforce; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("0 unblessed findings"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn vendor_contract_recency_gate_accepts_explicit_no_signed_contracts_declaration() {
    let temp = temp_dir("vendor-contract-recency-empty");
    let ledger = write_vendor_contract_ledger(
        &temp,
        "| `vcr-no-signed-contracts-2026-05-10` | All listed vendors and partners | no-signed-contracts | n/a | n/a | `gtm-partnerships` + `ops-security` |\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(vendor_contract_recency_args(&ledger, "2026-05-10"))
        .output()
        .expect("vendor contract recency gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("vendor contract recency validation passed: 1 records, 0 contracted, 0 renewal tasks required"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn vendor_contract_recency_gate_accepts_near_expiry_contract_with_task() {
    let temp = temp_dir("vendor-contract-recency-tasked");
    let ledger = write_vendor_contract_ledger(
        &temp,
        "| `ctr-oci-001` | OCI | contracted | 2026-06-01 | gh:oyatie/oyatie#123 | `gtm-partnerships` |\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(vendor_contract_recency_args(&ledger, "2026-05-10"))
        .output()
        .expect("vendor contract recency gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("vendor contract recency validation passed: 1 records, 1 contracted, 1 renewal tasks required"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn vendor_contract_recency_gate_rejects_near_expiry_contract_without_task() {
    let temp = temp_dir("vendor-contract-recency-missing-task");
    let ledger = write_vendor_contract_ledger(
        &temp,
        "| `ctr-oci-001` | OCI | contracted | 2026-06-01 | n/a | `gtm-partnerships` |\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(vendor_contract_recency_args(&ledger, "2026-05-10"))
        .output()
        .expect("vendor contract recency gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("RenewalTaskRequired"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn mobile_native_gate_accepts_explicit_web_only_preview_manifest() {
    let temp = temp_dir("mobile-native-web-only");
    let manifest = write_mobile_native_manifest(&temp, "");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(mobile_native_args(&manifest, &temp))
        .output()
        .expect("mobile native gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "mobile native validation passed: current_wave=W-Foundry-Preview, 0 native products, 0 native project markers, 0 quality records"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn mobile_native_gate_rejects_native_marker_without_product_manifest_row() {
    let temp = temp_dir("mobile-native-marker-without-row");
    let manifest = write_mobile_native_manifest(&temp, "");
    fs::create_dir_all(temp.join("apps/workspace/ios")).expect("native dir created");
    fs::write(temp.join("apps/workspace/ios/App.swift"), "struct App {}\n")
        .expect("native marker written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(mobile_native_args(&manifest, &temp))
        .output()
        .expect("mobile native gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("NativeMarkersWithoutDeclaredProducts"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn mobile_native_gate_accepts_native_product_with_quality_evidence() {
    let temp = temp_dir("mobile-native-product");
    fs::create_dir_all(temp.join("apps/workspace/ios")).expect("native dir created");
    fs::write(temp.join("apps/workspace/ios/App.swift"), "struct App {}\n")
        .expect("native marker written");
    let manifest = write_mobile_native_manifest(
        &temp,
        "workspace-mail-mobile\tworkspace\tnative-in-scope\tdocs/products/workspace/PRD.md#mail\tdocs/products/workspace/mobile.md#target-matrix\tdocs/products/workspace/mobile.md#tech-stack\tpacks/kr/localization/README.md#store-policy\ttrue\tartifact://mobile/workspace-mail/accessibility.json\ttrue\tartifact://mobile/workspace-mail/parity.json\ttrue\tartifact://mobile/workspace-mail/sbom.spdx.json\t0\t9950\t20\t2000\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(mobile_native_args(&manifest, &temp))
        .output()
        .expect("mobile native gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "mobile native validation passed: current_wave=W-Foundry-Preview, 1 native products, 1 native project markers, 1 quality records"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn typescript_workspace_gate_accepts_absent_workspace_for_active_pnpm_lanes() {
    let temp = temp_dir("typescript-workspace-absent");
    fs::create_dir_all(&temp).expect("temp dir created");

    for lane in ["typecheck", "test"] {
        let output = Command::new(env!("CARGO_BIN_EXE_oya"))
            .args(typescript_workspace_args(&temp, lane))
            .output()
            .expect("typescript workspace gate command runs");

        assert!(
            output.status.success(),
            "lane={lane}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("lane={lane}, workspace_present=false"))
        );
    }

    fs::remove_dir_all(temp).ok();
}

#[test]
fn typescript_workspace_gate_rejects_missing_repo_root() {
    let missing = temp_dir("typescript-workspace-missing-root");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(typescript_workspace_args(&missing, "typecheck"))
        .output()
        .expect("typescript workspace gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("repo root is not a directory"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn typescript_workspace_gate_accepts_pnpm_workspace_scripts() {
    let temp = temp_dir("typescript-workspace-valid");
    write_typescript_workspace_fixture(
        &temp,
        r#"{
  "packageManager": "pnpm@9.12.0",
  "scripts": {
    "typecheck": "tsc --noEmit",
    "test": "vitest run"
  }
}
"#,
        true,
        true,
    );

    for lane in ["typecheck", "test"] {
        let output = Command::new(env!("CARGO_BIN_EXE_oya"))
            .args(typescript_workspace_args(&temp, lane))
            .output()
            .expect("typescript workspace gate command runs");

        assert!(
            output.status.success(),
            "lane={lane}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("lane={lane}, workspace_present=true"))
        );
    }

    fs::remove_dir_all(temp).ok();
}

#[test]
fn typescript_workspace_gate_rejects_ts_marker_without_package_json() {
    let temp = temp_dir("typescript-workspace-no-package");
    fs::create_dir_all(temp.join("src")).expect("src dir created");
    fs::write(temp.join("src/index.ts"), "export const ok = true;\n").expect("ts marker written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(typescript_workspace_args(&temp, "typecheck"))
        .output()
        .expect("typescript workspace gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingRootPackageJson"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn typescript_workspace_gate_rejects_missing_required_script() {
    let temp = temp_dir("typescript-workspace-missing-script");
    write_typescript_workspace_fixture(
        &temp,
        r#"{
  "packageManager": "pnpm@9.12.0",
  "scripts": {
    "test": "vitest run"
  }
}
"#,
        true,
        true,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(typescript_workspace_args(&temp, "typecheck"))
        .output()
        .expect("typescript workspace gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("RequiredScriptMissing"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn typescript_workspace_gate_rejects_non_pnpm_package_manager() {
    let temp = temp_dir("typescript-workspace-npm");
    write_typescript_workspace_fixture(
        &temp,
        r#"{
  "packageManager": "npm@10.0.0",
  "scripts": {
    "typecheck": "tsc --noEmit",
    "test": "vitest run"
  }
}
"#,
        true,
        true,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(typescript_workspace_args(&temp, "typecheck"))
        .output()
        .expect("typescript workspace gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PackageManagerNotPnpm"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn adr_citation_gate_accepts_new_pack_refs_and_forensic_mapping_refs() {
    let temp = temp_dir("adr-citation-valid");
    write_adr_citation_fixture(
        &temp,
        "ADR-0051 is the mobile/native source.",
        "Legacy ADR-0201 maps to ADR-0051.",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(adr_citation_args(&temp))
        .output()
        .expect("ADR citation gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("ADR citation validation passed: 4 documents, 3 citations, 2 allowed ADRs")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn adr_citation_gate_accepts_architecture_scorecard_as_forensic_surface() {
    let temp = temp_dir("adr-citation-scorecard");
    write_adr_citation_fixture(&temp, "ADR-0051 is active.", "Legacy ADR-0201 maps.");
    fs::create_dir_all(temp.join("docs/architecture")).expect("architecture dir created");
    fs::write(
        temp.join("docs/architecture/wave-3-final-scorecard-2026-05-20.md"),
        "# Scorecard\n\nHistorical missing slot ADR-0201 remains forensic-only.\n",
    )
    .expect("scorecard written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(adr_citation_args(&temp))
        .output()
        .expect("ADR citation gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn adr_citation_gate_rejects_legacy_ref_in_active_doc() {
    let temp = temp_dir("adr-citation-legacy");
    write_adr_citation_fixture(
        &temp,
        "Legacy ADR-0201 must not appear here.",
        "Legacy ADR-0201 maps to ADR-0051.",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(adr_citation_args(&temp))
        .output()
        .expect("ADR citation gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("DisallowedAdrCitation"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn adr_citation_gate_rejects_future_ref_without_pack_file() {
    let temp = temp_dir("adr-citation-future");
    write_adr_citation_fixture(
        &temp,
        "ADR-0052 has not landed yet.",
        "Legacy ADR-0201 maps to ADR-0051.",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(adr_citation_args(&temp))
        .output()
        .expect("ADR citation gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ADR-0052"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn brand_residue_gate_accepts_canonical_brand_and_real_transitions() {
    let temp = temp_dir("brand-residue-valid");
    write_brand_residue_file(
        &temp,
        "docs/GLOSSARY.md",
        "# Glossary\n\nOyatie is the product brand.\n| `oyatie-*` Cargo prefix | `oya-*` | ADR-0017 |\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(brand_residue_args(&temp))
        .output()
        .expect("brand residue gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("brand residue validation passed: 1 files, 1 transition patterns")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn brand_residue_gate_rejects_tautological_rebrand_statement() {
    let temp = temp_dir("brand-residue-tautology");
    write_brand_residue_file(
        &temp,
        "docs/PRD.md",
        "All product strings rebrand from `Oyatie` → `Oyatie`.\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(brand_residue_args(&temp))
        .output()
        .expect("brand residue gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("TautologicalBrandTransition"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn brand_residue_gate_rejects_tautological_retired_term_rows() {
    let temp = temp_dir("brand-residue-table");
    write_brand_residue_file(
        &temp,
        "docs/GLOSSARY.md",
        "| Old | New | Reason |\n|---|---|---|\n| Oyatie | Oyatie | Brand rename per ADR-0017 |\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(brand_residue_args(&temp))
        .output()
        .expect("brand residue gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("RetiredTermsTable"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn brand_residue_gate_rejects_forbidden_forgejo_token_in_live_doc() {
    let temp = temp_dir("brand-residue-forgejo");
    write_brand_residue_file(
        &temp,
        "docs/ci/forge.md",
        "# CI\n\nSelf-hosted Forgejo PRs gate merges.\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(brand_residue_args(&temp))
        .output()
        .expect("brand residue gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ForbiddenBrandToken"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn brand_residue_gate_excludes_superseded_adr_history() {
    let temp = temp_dir("brand-residue-superseded-adr");
    // Live doc keeps the gate non-empty; the Superseded ADR retains a retired
    // brand token in its immutable history and must be excluded, not rewritten.
    write_brand_residue_file(
        &temp,
        "docs/GLOSSARY.md",
        "# Glossary\n\nOyatie is the product brand.\n",
    );
    write_brand_residue_file(
        &temp,
        "docs/adr-archive/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md",
        "---\nid: ADR-0511\nstatus: Superseded\nsuperseded_by: [ADR-0515]\n---\n\n# ADR-0511\n\nForgejo Commit Status remained the gate-result sink before the cutover.\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(brand_residue_args(&temp))
        .output()
        .expect("brand residue gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn api_semver_gate_accepts_bootstrap_without_contracts() {
    let temp = temp_dir("api-semver-empty");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(api_semver_args(&temp))
        .output()
        .expect("API semver gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("API semver validation passed: 0 contracts, 0 metadata records")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn api_semver_gate_accepts_contract_with_metadata() {
    let temp = temp_dir("api-semver-valid");
    write_api_contract(
        &temp,
        "contracts/openapi/workspace/mail-v1.yaml",
        "openapi: 3.2.0\ninfo:\n  title: Mail\n  version: 1.0.0\n",
    );
    write_api_contract(
        &temp,
        "contracts/openapi/workspace/mail-v1.meta.yaml",
        "tier: preview\nowner_team: platform-api-sdk\nversion: 1.0.0\nsunset: none\nrelated_adrs: [ADR-0037]\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(api_semver_args(&temp))
        .output()
        .expect("API semver gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("API semver validation passed: 1 contracts, 1 metadata records")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn api_semver_gate_rejects_contract_without_metadata() {
    let temp = temp_dir("api-semver-missing-metadata");
    write_api_contract(
        &temp,
        "contracts/openapi/workspace/mail-v1.yaml",
        "openapi: 3.2.0\ninfo:\n  title: Mail\n  version: 1.0.0\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(api_semver_args(&temp))
        .output()
        .expect("API semver gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingMetadata"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn api_semver_gate_rejects_metadata_version_mismatch() {
    let temp = temp_dir("api-semver-version-mismatch");
    write_api_contract(
        &temp,
        "contracts/openapi/workspace/mail-v2.yaml",
        "openapi: 3.2.0\ninfo:\n  title: Mail\n  version: 2.0.0\n",
    );
    write_api_contract(
        &temp,
        "contracts/openapi/workspace/mail-v2.meta.yaml",
        "tier: stable\nowner_team: platform-api-sdk\nversion: 1.0.0\nsunset: none\nrelated_adrs:\n  - ADR-0037\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(api_semver_args(&temp))
        .output()
        .expect("API semver gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("VersionSuffixMismatch"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_accepts_source_only_bootstrap() {
    let temp = temp_dir("supply-chain-valid");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        None,
        false,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "supply chain validation passed: 1 catalog records, 1 source-only attestations"
        )
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_rejects_missing_cargo_audit_wiring() {
    let temp = temp_dir("supply-chain-no-audit");
    write_supply_chain_fixture(&temp, "source-only", "cargo deny check\n", None, false);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingCargoAuditCheck"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_rejects_unpinned_third_party_actions() {
    let temp = temp_dir("supply-chain-unpinned-action");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        Some(
            "name: supply\njobs:\n  guard:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: vendor/tool@v1\n",
        ),
        false,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("UnpinnedThirdPartyAction"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_rejects_release_manifest_without_adr0039_evidence() {
    let temp = temp_dir("supply-chain-release-manifest");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        None,
        true,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ReleaseManifestWithoutAdr0039Evidence"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_accepts_complete_release_attestation_evidence() {
    let temp = temp_dir("release-supply-chain-valid");
    write_release_supply_chain_fixture(&temp, "123", "0", "true");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args(&temp))
        .output()
        .expect("release supply chain gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("release supply chain validation passed: 1 artifacts, 1 evidence records")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn image_promotion_gate_accepts_signed_dev_staging_prod_ladder_with_admission_and_kill_switch() {
    let temp = temp_dir("image-promotion-valid");
    write_image_promotion_fixture(&temp, None);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(image_promotion_args(&temp))
        .output()
        .expect("image promotion gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "image promotion validation passed: 1 artifacts, 3 promotion records, 2 kubewarden verifier records, 1 kyverno verifier records"
        ),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn image_promotion_gate_rejects_missing_staging_promotion_record() {
    let temp = temp_dir("image-promotion-missing-staging");
    write_image_promotion_fixture(&temp, Some("staging"));

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(image_promotion_args(&temp))
        .output()
        .expect("image promotion gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingTierPromotion"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_rejects_missing_rekor_inclusion() {
    let temp = temp_dir("release-supply-chain-missing-rekor");
    write_release_supply_chain_fixture(&temp, "0", "0", "true");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args(&temp))
        .output()
        .expect("release supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingRekorInclusion"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_rejects_open_high_critical_findings() {
    let temp = temp_dir("release-supply-chain-open-findings");
    write_release_supply_chain_fixture(&temp, "123", "1", "true");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args(&temp))
        .output()
        .expect("release supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("OpenHighCriticalFindings"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_accepts_pre_release_empty_scope_declaration() {
    let temp = temp_dir("release-supply-chain-pre-release-empty");
    write_pre_release_image_manifest(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args_with_phase(&temp, "pre-release"))
        .output()
        .expect("release supply chain gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "release supply chain validation passed: 0 artifacts, 0 evidence records, phase=pre-release"
        ),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_rejects_pre_release_empty_scope_without_rationale() {
    let temp = temp_dir("release-supply-chain-pre-release-no-rationale");
    fs::create_dir_all(temp.join("registry/release")).expect("release registry dir created");
    fs::write(
        temp.join("registry/release/images.yaml"),
        "# release_state: pre-release\nimages: []\n",
    )
    .expect("release image manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args_with_phase(&temp, "pre-release"))
        .output()
        .expect("release supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingPreReleaseEmptyScopeRationale"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_supply_chain_gate_rejects_release_phase_without_artifacts() {
    let temp = temp_dir("release-supply-chain-release-empty");
    write_pre_release_image_manifest(&temp);
    fs::create_dir_all(temp.join("registry/release/supply-chain"))
        .expect("release supply-chain evidence dir created");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_supply_chain_args_with_phase(&temp, "release"))
        .output()
        .expect("release supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("NoReleaseArtifacts"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_evidence_pack_gate_accepts_explicit_pre_release_bootstrap() {
    let temp = temp_dir("release-evidence-pack-bootstrap");
    write_release_evidence_pack_fixture(&temp, "");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_evidence_pack_args(&temp))
        .output()
        .expect("release evidence pack gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "release evidence pack validation passed: 3 known regulators, 0 records, 0 published"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_evidence_pack_gate_rejects_bootstrap_when_records_required() {
    let temp = temp_dir("release-evidence-pack-requires-records");
    write_release_evidence_pack_fixture(&temp, "");
    let mut args = release_evidence_pack_args(&temp);
    args.push("--require-records".into());

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args)
        .output()
        .expect("release evidence pack gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("RecordsRequired"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_evidence_pack_gate_accepts_published_regulator_pack() {
    let temp = temp_dir("release-evidence-pack-published");
    write_release_evidence_pack_fixture(
        &temp,
        "GDPR\teu\toya-pack-eu\t0.1.0\tper-release\t2026-05-01\t2026-05-10\tops-compliance\tartifact://release/0.1.0/evidence/gdpr.md\trekor://log/123/evidence-pack\tEVT-EVIDENCE-PACK-PUBLISHED-0001\t1000\t1240\t8\t8\ttrue\ttrue\tpublished\n",
    );
    rewrite_release_evidence_pack_version(&temp, "0.1.0", "n/a");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_evidence_pack_args(&temp))
        .output()
        .expect("release evidence pack gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "release evidence pack validation passed: 3 known regulators, 1 records, 1 published"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_evidence_pack_gate_rejects_unknown_regulator() {
    let temp = temp_dir("release-evidence-pack-unknown-regulator");
    write_release_evidence_pack_fixture(
        &temp,
        "Unknown\teu\toya-pack-eu\t0.1.0\tper-release\t2026-05-01\t2026-05-10\tops-compliance\tartifact://release/0.1.0/evidence/unknown.md\trekor://log/123/evidence-pack\tEVT-EVIDENCE-PACK-PUBLISHED-0001\t1000\t1240\t8\t8\ttrue\ttrue\tpublished\n",
    );
    rewrite_release_evidence_pack_version(&temp, "0.1.0", "n/a");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_evidence_pack_args(&temp))
        .output()
        .expect("release evidence pack gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("UnknownRegulator"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn release_evidence_pack_gate_rejects_sla_over_four_hours() {
    let temp = temp_dir("release-evidence-pack-sla");
    write_release_evidence_pack_fixture(
        &temp,
        "GDPR\teu\toya-pack-eu\t0.1.0\tper-release\t2026-05-01\t2026-05-10\tops-compliance\tartifact://release/0.1.0/evidence/gdpr.md\trekor://log/123/evidence-pack\tEVT-EVIDENCE-PACK-PUBLISHED-0001\t1000\t1241\t8\t8\ttrue\ttrue\tpublished\n",
    );
    rewrite_release_evidence_pack_version(&temp, "0.1.0", "n/a");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(release_evidence_pack_args(&temp))
        .output()
        .expect("release evidence pack gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("RegenerationSlaExceeded"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_accepts_full_adr0039_static_wiring() {
    let temp = temp_dir("supply-chain-full-valid");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        Some(
            "name: supply-chain\njobs:\n  adr0039:\n    steps:\n      - run: scripts/supply-chain-adr0039.sh\n",
        ),
        true,
    );
    write_supply_chain_adr0039_script(&temp);
    write_supply_chain_branch_protection(&temp);
    write_supply_chain_admission_policy(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_full_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "supply chain validation passed: 1 catalog records, 1 source-only attestations"
        )
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_accepts_full_adr0039_static_wiring_with_pre_release_empty_scope() {
    let temp = temp_dir("supply-chain-full-pre-release-empty-scope");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        Some(
            "name: supply-chain\njobs:\n  adr0039:\n    steps:\n      - run: scripts/supply-chain-adr0039.sh\n",
        ),
        false,
    );
    write_pre_release_contract_image_manifest(&temp);
    write_supply_chain_adr0039_script(&temp);
    write_supply_chain_branch_protection(&temp);
    write_supply_chain_admission_policy(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_full_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn supply_chain_gate_rejects_full_adr0039_without_signed_commit_policy() {
    let temp = temp_dir("supply-chain-full-missing-branch-policy");
    write_supply_chain_fixture(
        &temp,
        "source-only",
        "cargo audit\ncargo deny check\n",
        Some(
            "name: supply-chain\njobs:\n  adr0039:\n    steps:\n      - run: scripts/supply-chain-adr0039.sh\n",
        ),
        true,
    );
    write_supply_chain_adr0039_script(&temp);
    write_supply_chain_admission_policy(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(supply_chain_full_args(&temp))
        .output()
        .expect("supply chain gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("signed_commit_policy_wired"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cargo_prefix_gate_accepts_oya_workspace_members() {
    let temp = temp_dir("cargo-prefix-valid");
    write_cargo_prefix_workspace(
        &temp,
        "crates/oya-intelligence-capability-kernel",
        "oya-intelligence-capability-kernel",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(cargo_prefix_args(&temp))
        .output()
        .expect("cargo prefix gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("cargo prefix validation passed: 1 workspace members")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cargo_prefix_gate_rejects_unprefixed_package_name() {
    let temp = temp_dir("cargo-prefix-unprefixed-package");
    write_cargo_prefix_workspace(
        &temp,
        "crates/oya-intelligence-capability-kernel",
        "foundry-capability-kernel",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(cargo_prefix_args(&temp))
        .output()
        .expect("cargo prefix gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PackageNamePrefixViolation"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cargo_prefix_gate_rejects_member_path_package_name_mismatch() {
    let temp = temp_dir("cargo-prefix-mismatch");
    write_cargo_prefix_workspace(
        &temp,
        "crates/oya-intelligence-capability-kernel",
        "oya-intelligence-policy-kernel",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(cargo_prefix_args(&temp))
        .output()
        .expect("cargo prefix gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PackageNamePathMismatch"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn authority_cohesion_gate_rejects_drifted_chain_declarations() {
    let temp = temp_dir("authority-cohesion");
    fs::create_dir_all(&temp).expect("docs dir created");
    write_authority_doc(&temp, "AGENTS.md", "drifted");
    write_authority_doc(&temp, "README.md", "canonical");
    write_authority_doc(&temp, "MASTERPLAN.md", "canonical");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "authority-cohesion",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
        ])
        .output()
        .expect("authority gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("authority cohesion validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn hyperscaler_maturity_claims_gate_accepts_repo_control_surfaces() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "hyperscaler-maturity-claims"])
        .current_dir(repo_root())
        .output()
        .expect("hyperscaler maturity gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hyperscaler maturity claim governance validation passed"));
    assert!(stdout.contains("claim_status=blocked_until_required_evidence_is_green"));
}

#[test]
fn design_spec_maturity_claims_gate_accepts_fixture_and_emits_evidence() {
    let temp = temp_dir("design-spec-maturity-claims");
    let microservices_root = temp.join("microservices");
    write_design_spec_maturity_service_fixture(&microservices_root);
    let evidence_path = temp.join("evidence/design-spec-maturity.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "design-spec-maturity-claims",
            "--standard",
            repo_root()
                .join("specs/design-spec-maturity-claims.json")
                .to_str()
                .expect("utf8 standard path"),
            "--microservices-root",
            microservices_root
                .to_str()
                .expect("utf8 microservices path"),
            "--emit-evidence",
            evidence_path.to_str().expect("utf8 evidence path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("design/spec maturity gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("design/spec maturity claim validation passed"));
    assert!(
        stdout.contains("operational_claim_status=blocked_until_operational_evidence_is_green")
    );
    let evidence = fs::read_to_string(&evidence_path).expect("evidence written");
    assert!(evidence.contains("\"missing_count\": 0"));
    assert!(evidence.contains("hyperscaler-grade design maturity bar"));

    fs::remove_dir_all(temp).ok();
}

#[cfg(any())]
// disabled: write_korea_localization_evidence_fixture helper not yet wired in this branch; restore after Wave 15Z korea-localization-evidence sub-wave authors the fixture writer
#[test]
fn korea_localization_evidence_gate_accepts_fixture_and_emits_bundle() {
    let temp = temp_dir("korea-localization-evidence");
    write_korea_localization_evidence_fixture(&temp);
    let evidence_path = temp.join("evidence/fd001/kr-localization.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "korea-localization-evidence",
            "--repo-root",
            temp.to_str().expect("utf8 repo root"),
            "--emit-evidence",
            evidence_path.to_str().expect("utf8 evidence path"),
        ])
        .output()
        .expect("korea localization evidence gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("korea-localization-evidence validation passed"));
    assert!(stdout.contains("pack_status=planning-closed-foundational"));
    assert!(stdout.contains("activation_claim=not-active"));
    let evidence = fs::read_to_string(&evidence_path).expect("evidence bundle written");
    assert!(evidence.contains("\"schema_version\": \"oyatie.kr-localization-evidence.v1\""));
    assert!(evidence.contains("\"covered_kr_pack_surface_count\": 12"));

    fs::remove_dir_all(temp).ok();
}

#[cfg(any())]
// disabled: write_korea_localization_evidence_fixture helper not yet wired in this branch; restore after Wave 15Z korea-localization-evidence sub-wave authors the fixture writer
#[test]
fn korea_localization_evidence_gate_rejects_missing_surface_evidence() {
    let temp = temp_dir("korea-localization-evidence-missing");
    write_korea_localization_evidence_fixture(&temp);
    fs::remove_file(temp.join("docs/localization-packs/kr/evidence/mail.md"))
        .expect("fixture file removed");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "korea-localization-evidence",
            "--repo-root",
            temp.to_str().expect("utf8 repo root"),
        ])
        .output()
        .expect("korea localization evidence gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("korea-localization-evidence validation failed"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("mail.md"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn design_spec_maturity_claims_gate_rejects_unblocked_operational_claims() {
    let temp = temp_dir("design-spec-maturity-operational-claim");
    let microservices_root = temp.join("microservices");
    write_design_spec_maturity_service_fixture(&microservices_root);
    let standard_path = temp.join("bad-standard.json");
    let fixture = fs::read_to_string(repo_root().join("specs/design-spec-maturity-claims.json"))
        .expect("repo standard read");
    fs::write(
        &standard_path,
        fixture.replace(
            "\"claim_status\": \"blocked_until_operational_evidence_is_green\"",
            "\"claim_status\": \"allowed\"",
        ),
    )
    .expect("bad standard written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "design-spec-maturity-claims",
            "--standard",
            standard_path.to_str().expect("utf8 standard path"),
            "--microservices-root",
            microservices_root
                .to_str()
                .expect("utf8 microservices path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("design/spec maturity gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("operational maturity claim_status must remain"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn design_spec_maturity_claims_gate_rejects_unimplemented_required_surfaces() {
    let temp = temp_dir("design-spec-maturity-unknown-surface");
    let microservices_root = temp.join("microservices");
    write_design_spec_maturity_service_fixture(&microservices_root);
    let standard_path = temp.join("bad-standard.json");
    let fixture = fs::read_to_string(repo_root().join("specs/design-spec-maturity-claims.json"))
        .expect("repo standard read");
    let mut standard: serde_json::Value =
        serde_json::from_str(&fixture).expect("repo standard parses");
    standard
        .get_mut("required_surfaces")
        .and_then(serde_json::Value::as_array_mut)
        .expect("required surfaces array")
        .push(serde_json::json!({
            "id": "new_required_surface",
            "name": "New Required Surface",
            "evidence_policy": "Every service must prove this surface."
        }));
    fs::write(
        &standard_path,
        serde_json::to_string_pretty(&standard).expect("bad standard serializes"),
    )
    .expect("bad standard written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "design-spec-maturity-claims",
            "--standard",
            standard_path.to_str().expect("utf8 standard path"),
            "--microservices-root",
            microservices_root
                .to_str()
                .expect("utf8 microservices path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("design/spec maturity gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unknown required design/spec surface ids: new_required_surface"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn hyperscaler_arch_invariants_gate_accepts_repo_spec() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "hyperscaler-arch-invariants"])
        .current_dir(repo_root())
        .output()
        .expect("hyperscaler architecture invariant gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hyperscaler architecture invariant validation passed"));
    assert!(stdout.contains("35 invariants, 13 services"));
}

#[test]
fn hyperscaler_arch_invariants_gate_rejects_active_enforcement_fields() {
    let temp = temp_dir("hyperscaler-arch-invariants-active-field");
    fs::create_dir_all(&temp).expect("fixture dir created");
    let fixture_path = temp.join("hyperscaler-architecture-invariants.json");
    let mut fixture =
        fs::read_to_string(repo_root().join("specs/hyperscaler-architecture-invariants.json"))
            .expect("repo invariant spec read");
    fixture = fixture.replacen("\"planned_enforced_by\"", "\"enforced_by\"", 1);
    fs::write(&fixture_path, fixture).expect("bad invariant fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "hyperscaler-arch-invariants",
            "--spec",
            fixture_path.to_str().expect("utf8 fixture path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("hyperscaler architecture invariant gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("must not use active field `enforced_by`"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn hyperscaler_arch_invariants_gate_rejects_omitted_applicable_product_ref() {
    let temp = temp_dir("hyperscaler-arch-invariants-omitted-ref");
    fs::create_dir_all(&temp).expect("fixture dir created");
    let fixture_path = temp.join("hyperscaler-architecture-invariants.json");
    let fixture =
        fs::read_to_string(repo_root().join("specs/hyperscaler-architecture-invariants.json"))
            .expect("repo invariant spec read");
    let marker = "\"ads\": [\n      \"INV-AUDIT-CHAIN-EMIT\",";
    let fixture = fixture.replace(marker, "\"ads\": [");
    fs::write(&fixture_path, fixture).expect("bad invariant fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "hyperscaler-arch-invariants",
            "--spec",
            fixture_path.to_str().expect("utf8 fixture path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("hyperscaler architecture invariant gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("product ads omits applicable invariant INV-AUDIT-CHAIN-EMIT"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn hyperscaler_maturity_claims_gate_rejects_unsourced_workflow_studio_claims() {
    let temp = temp_dir("hyperscaler-maturity-claims");
    fs::create_dir_all(temp.join("specs/microservices")).expect("spec dirs created");
    fs::copy(
        repo_root().join("specs/hyperscaler-gates.json"),
        temp.join("specs/hyperscaler-gates.json"),
    )
    .expect("hyperscaler gates fixture copied");
    fs::copy(
        repo_root().join("specs/microservices/workflow.json"),
        temp.join("specs/microservices/workflow.json"),
    )
    .expect("workflow spec fixture copied");
    fs::write(
        temp.join("specs/microservices/workflow-studio.json"),
        r#"{
  "identity": { "product_id": "workflow-studio" },
  "competitive_claim_policy": {
    "status": "binding",
    "forbidden_without_benchmark_evidence": ["numeric latency comparisons"],
    "required_per_competitor_row": [
      "source_evidence_refs",
      "observed_strengths",
      "observed_weaknesses_or_gaps",
      "adopt_from_them",
      "improve_beyond_them",
      "claim_boundary"
    ]
  },
  "user_experience": {
    "accessibility_coverage": "WCAG 2.2 AA",
    "offline_behavior": "buffered",
    "loading_state_coverage": "covered",
    "journey_critical_paths": ["first workflow"],
    "keyboard_navigation_coverage_pct": 100,
    "error_state_coverage": {
      "invalid_spec": "covered",
      "collaboration_conflict": "covered",
      "network_partition": "covered",
      "policy_denied": "covered"
    }
  },
  "competitive": [
    { "competitor": "n8n", "we_beat_on": ["speed"], "measurable": "10x faster" }
  ]
}"#,
    )
    .expect("bad workflow-studio fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "hyperscaler-maturity-claims",
            "--gates",
            temp.join("specs/hyperscaler-gates.json")
                .to_str()
                .expect("utf8 gates path"),
            "--workflow-studio",
            temp.join("specs/microservices/workflow-studio.json")
                .to_str()
                .expect("utf8 workflow-studio path"),
            "--workflow",
            temp.join("specs/microservices/workflow.json")
                .to_str()
                .expect("utf8 workflow path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("hyperscaler maturity gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("retired unsupported benchmark fields"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn workspace_hygiene_gate_accepts_repo_policy_without_scan() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "workspace-hygiene", "--no-scan"])
        .current_dir(repo_root())
        .output()
        .expect("workspace hygiene gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("workspace hygiene validation passed")
    );
}

#[test]
fn workspace_hygiene_gate_rejects_cleanup_flags_without_scan() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "workspace-hygiene",
            "--no-scan",
            "--clean-build-artifacts",
        ])
        .current_dir(repo_root())
        .output()
        .expect("workspace hygiene gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cleanup requires scanning"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn workspace_hygiene_gate_strict_mode_rejects_build_artifact_residue() {
    let temp = temp_dir("workspace-hygiene-build-artifacts");
    let scan_root = temp.join("scan-root");
    fs::create_dir_all(scan_root.join("target")).expect("build artifact fixture created");
    let policy = temp.join("workspace-hygiene.json");
    fs::write(
        &policy,
        workspace_hygiene_fixture(scan_root.to_str().expect("utf8 scan root")),
    )
    .expect("workspace hygiene fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "workspace-hygiene",
            "--policy",
            policy.to_str().expect("utf8 policy"),
            "--strict",
        ])
        .output()
        .expect("workspace hygiene gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("build-artifacts"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("target"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn workspace_hygiene_gate_clean_build_artifacts_removes_configured_residue() {
    let temp = temp_dir("workspace-hygiene-clean-build-artifacts");
    let scan_root = temp.join("scan-root");
    fs::create_dir_all(scan_root.join("target/debug")).expect("build artifact fixture created");
    fs::create_dir_all(scan_root.join("build")).expect("non-cleanable build dir fixture created");
    let policy = temp.join("workspace-hygiene.json");
    fs::write(
        &policy,
        workspace_hygiene_fixture(scan_root.to_str().expect("utf8 scan root")),
    )
    .expect("workspace hygiene fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "workspace-hygiene",
            "--policy",
            policy.to_str().expect("utf8 policy"),
            "--clean-build-artifacts",
        ])
        .output()
        .expect("workspace hygiene cleanup command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!scan_root.join("target").exists());
    assert!(scan_root.join("build").exists());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("cleaned=1"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn workspace_hygiene_gate_clean_temp_artifacts_keeps_exempt_owned_roots() {
    let temp = temp_dir("workspace-hygiene-clean-temp-artifacts");
    let scan_root = temp.join("scan-root");
    fs::create_dir_all(scan_root.join("oyatie")).expect("owned root fixture created");
    fs::write(scan_root.join("unused-temp-pattern"), "temp")
        .expect("temp artifact fixture written");
    let policy = temp.join("workspace-hygiene.json");
    fs::write(
        &policy,
        workspace_hygiene_fixture(scan_root.to_str().expect("utf8 scan root")),
    )
    .expect("workspace hygiene fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "workspace-hygiene",
            "--policy",
            policy.to_str().expect("utf8 policy"),
            "--clean-temp-artifacts",
            "--strict",
        ])
        .output()
        .expect("workspace hygiene temp cleanup command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(scan_root.join("oyatie").exists());
    assert!(!scan_root.join("unused-temp-pattern").exists());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("cleaned=1"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn runbook_index_gate_rejects_indexed_missing_runbook() {
    let temp = temp_dir("runbook-index");
    fs::create_dir_all(temp.join("runbooks/foundry")).expect("runbooks dir created");
    fs::write(
        temp.join("RUNBOOKS-INDEX.md"),
        "# Runbooks\n\n## 2. Critical runbooks\n\n- `foundry/missing.md`\n",
    )
    .expect("runbooks index written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "runbook-index-resolves",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
        ])
        .output()
        .expect("runbook index gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("runbook index validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn runbook_index_gate_checks_prefixed_index_entries() {
    let temp = temp_dir("runbook-index-prefixed");
    fs::create_dir_all(temp.join("runbooks/foundry")).expect("runbooks dir created");
    fs::write(temp.join("runbooks/foundry/valid.md"), "# Valid\n").expect("runbook written");
    fs::write(
        temp.join("RUNBOOKS-INDEX.md"),
        "# Runbooks\n\n## 2. Critical runbooks\n\n- `foundry/valid.md`\n- `runbooks/foundry/missing.md`\n",
    )
    .expect("runbooks index written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "runbook-index-resolves",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
        ])
        .output()
        .expect("runbook index gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("runbook index validation failed"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn data_class_gate_rejects_untracked_unannotated_kernel_field() {
    let temp = temp_dir("data-class-untracked");
    write_kernel_workspace(
        &temp,
        "pub struct Example {\n    pub tenant_id: String,\n}\n",
    );
    fs::write(temp.join("legacy.tsv"), "").expect("legacy ledger written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "data-class",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--legacy",
            temp.join("legacy.tsv").to_str().expect("utf8 legacy"),
        ])
        .output()
        .expect("data-class gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("data class fitness validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn data_class_gate_accepts_annotated_fields_and_tracked_legacy_fields() {
    let temp = temp_dir("data-class-tracked");
    write_kernel_workspace(
        &temp,
        "pub struct Example {\n    pub display_name: String, // data_class: PUBLIC\n    pub tenant_id: String,\n}\n",
    );
    fs::write(
        temp.join("legacy.tsv"),
        "crates/example-kernel/src/lib.rs\tExample\ttenant_id\tMFL-0008 bootstrap ledger; new fields must be annotated\n",
    )
    .expect("legacy ledger written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "data-class",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--legacy",
            temp.join("legacy.tsv").to_str().expect("utf8 legacy"),
        ])
        .output()
        .expect("data-class gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "data class fitness validation passed: 2 fields checked, 1 annotated, 1 legacy unannotated"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn slo_coverage_gate_rejects_catalog_record_without_slo() {
    let temp = temp_dir("slo-coverage-missing");
    write_slo_catalog_record(&temp, "oya-intelligence-capability-kernel", None);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "slo-coverage",
            "--registry",
            temp.to_str().expect("utf8 registry"),
        ])
        .output()
        .expect("slo coverage gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("slo coverage validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn slo_coverage_gate_accepts_catalog_records_with_slo() {
    let temp = temp_dir("slo-coverage-present");
    write_slo_catalog_record(
        &temp,
        "oya-intelligence-capability-kernel",
        Some("preview-control-plane"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "slo-coverage",
            "--registry",
            temp.to_str().expect("utf8 registry"),
        ])
        .output()
        .expect("slo coverage gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("slo coverage validation passed: 1 records")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cohesion_gate_rejects_implemented_contract_source_missing_catalog() {
    let temp = temp_dir("cohesion-missing-catalog");
    write_workspace_manifest(&temp, &["crates/example-kernel"]);
    write_package_manifest(&temp.join("crates/example-kernel"), "example-kernel");
    let registry = temp.join("registry");
    fs::create_dir_all(&registry).expect("registry dir created");
    let contracts = temp.join("contracts.json");
    write_contracts_json(
        &contracts,
        "TENANT_KERNEL",
        "crates/example-kernel",
        "cross-axis",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "cohesion",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            registry.to_str().expect("utf8 registry"),
            "--contracts",
            contracts.to_str().expect("utf8 contracts"),
        ])
        .output()
        .expect("cohesion gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cohesion validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cohesion_gate_accepts_implemented_contract_source_with_catalog() {
    let temp = temp_dir("cohesion-present-catalog");
    write_workspace_manifest(&temp, &["crates/example-kernel"]);
    write_package_manifest(&temp.join("crates/example-kernel"), "example-kernel");
    let registry = temp.join("registry");
    write_slo_catalog_record(&registry, "example-kernel", Some("preview-control-plane"));
    let contracts = temp.join("contracts.json");
    write_contracts_json(
        &contracts,
        "TENANT_KERNEL",
        "crates/example-kernel",
        "cross-axis",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "cohesion",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            registry.to_str().expect("utf8 registry"),
            "--contracts",
            contracts.to_str().expect("utf8 contracts"),
        ])
        .output()
        .expect("cohesion gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("cohesion validation passed: 1 contracts, 1 implemented sources")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn codeowners_mirror_gate_rejects_unknown_team_owner() {
    let temp = temp_dir("codeowners-unknown-team");
    write_codeowners_fixture(&temp, "@teams/missing-team");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "codeowners-mirror",
            "--codeowners",
            temp.join(".github/CODEOWNERS")
                .to_str()
                .expect("utf8 codeowners"),
            "--teams-dir",
            temp.join("teams").to_str().expect("utf8 teams dir"),
        ])
        .output()
        .expect("codeowners mirror gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("codeowners mirror validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cohesion_gate_rejects_non_object_contract_registry_entries() {
    let temp = temp_dir("cohesion-non-object-entry");
    write_workspace_manifest(&temp, &["crates/example-kernel"]);
    write_package_manifest(&temp.join("crates/example-kernel"), "example-kernel");
    let registry = temp.join("registry");
    write_slo_catalog_record(&registry, "example-kernel", Some("preview-control-plane"));
    let contracts = temp.join("contracts.json");
    fs::write(
        &contracts,
        r#"{
  "cross_axis_contracts": [
    {
      "id": "TENANT_KERNEL",
      "owner_axis": "saas",
      "consumer_axes": ["all"],
      "location": "crates/example-kernel",
      "change_review": "cross-axis"
    },
    null
  ]
}"#,
    )
    .expect("contracts json written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "cohesion",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            registry.to_str().expect("utf8 registry"),
            "--contracts",
            contracts.to_str().expect("utf8 contracts"),
        ])
        .output()
        .expect("cohesion gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("cross_axis_contracts contains non-object entry"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn cohesion_gate_rejects_empty_contract_registry_entries() {
    let temp = temp_dir("cohesion-empty-entry");
    write_workspace_manifest(&temp, &["crates/example-kernel", "crates/second-kernel"]);
    write_package_manifest(&temp.join("crates/example-kernel"), "example-kernel");
    write_package_manifest(&temp.join("crates/second-kernel"), "second-kernel");
    let registry = temp.join("registry");
    write_slo_catalog_record(&registry, "example-kernel", Some("preview-control-plane"));
    write_slo_catalog_record(&registry, "second-kernel", Some("preview-control-plane"));
    let contracts = temp.join("contracts.json");
    fs::write(
        &contracts,
        r#"{
  "cross_axis_contracts": [
    {
      "id": "TENANT_KERNEL",
      "owner_axis": "saas",
      "consumer_axes": ["all"],
      "location": "crates/example-kernel",
      "change_review": "cross-axis"
    },,
    {
      "id": "SECOND_KERNEL",
      "owner_axis": "saas",
      "consumer_axes": ["all"],
      "location": "crates/second-kernel",
      "change_review": "cross-axis"
    }
  ]
}"#,
    )
    .expect("contracts json written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "cohesion",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            registry.to_str().expect("utf8 registry"),
            "--contracts",
            contracts.to_str().expect("utf8 contracts"),
        ])
        .output()
        .expect("cohesion gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("cross_axis_contracts contains empty entry"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn codeowners_mirror_gate_accepts_chartered_team_owners() {
    let temp = temp_dir("codeowners-valid");
    write_codeowners_fixture(&temp, "@teams/council-architecture");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "codeowners-mirror",
            "--codeowners",
            temp.join(".github/CODEOWNERS")
                .to_str()
                .expect("utf8 codeowners"),
            "--teams-dir",
            temp.join("teams").to_str().expect("utf8 teams dir"),
        ])
        .output()
        .expect("codeowners mirror gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("codeowners mirror validation passed: 5 entries, 5 owners")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn raci_team_coverage_gate_rejects_team_missing_raci_row() {
    let temp = temp_dir("raci-team-missing-raci");
    write_raci_team_coverage_fixture(&temp, false, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(raci_team_coverage_args(&temp))
        .output()
        .expect("raci team coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("raci team coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn raci_team_coverage_gate_rejects_team_missing_codeowners_owner() {
    let temp = temp_dir("raci-team-missing-codeowners");
    write_raci_team_coverage_fixture(&temp, true, false);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(raci_team_coverage_args(&temp))
        .output()
        .expect("raci team coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("raci team coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn raci_team_coverage_gate_accepts_raci_and_codeowners_covered_teams() {
    let temp = temp_dir("raci-team-covered");
    write_raci_team_coverage_fixture(&temp, true, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(raci_team_coverage_args(&temp))
        .output()
        .expect("raci team coverage gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("raci team coverage validation passed: 2 teams")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn readme_doc_coverage_gate_rejects_root_doc_missing_readme_link() {
    let temp = temp_dir("readme-doc-missing-link");
    write_readme_doc_coverage_fixture(&temp, &["README.md", "CONSTITUTION.md"], &["README.md"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(readme_doc_coverage_args(&temp))
        .output()
        .expect("readme doc coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("readme doc coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn readme_doc_coverage_gate_rejects_catalog_missing_root_doc() {
    let temp = temp_dir("readme-doc-missing-catalog");
    write_readme_doc_coverage_fixture(
        &temp,
        &["README.md", "CONSTITUTION.md"],
        &["README.md", "CONSTITUTION.md"],
    );
    fs::write(
        temp.join("machine-readable/catalog.json"),
        r#"{
  "docs": {
    "doc.readme": {
      "path": "docs/README.md",
      "owner_team": "council-architecture",
      "dependent_docs": [],
      "validation_check": "readme-doc-coverage"
    }
  }
}
"#,
    )
    .expect("machine catalog rewritten");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(readme_doc_coverage_args(&temp))
        .output()
        .expect("readme doc coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("readme doc coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn readme_doc_coverage_gate_accepts_root_docs_with_catalog_and_links() {
    let temp = temp_dir("readme-doc-covered");
    write_readme_doc_coverage_fixture(
        &temp,
        &["README.md", "CONSTITUTION.md"],
        &["README.md", "CONSTITUTION.md"],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(readme_doc_coverage_args(&temp))
        .output()
        .expect("readme doc coverage gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("readme doc coverage validation passed: 2 documents")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_cross_doc_gate_rejects_machine_term_missing_glossary_markdown() {
    let temp = temp_dir("glossary-missing-markdown");
    write_glossary_coverage_fixture(&temp, "Old Alias", "", "Alpha Term");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_coverage_args(&temp))
        .output()
        .expect("glossary coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("glossary cross-doc coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_cross_doc_gate_rejects_active_term_missing_cross_doc_coverage() {
    let temp = temp_dir("glossary-missing-cross-doc");
    write_glossary_coverage_fixture(
        &temp,
        "Alpha Term\nOld Alias",
        "No covered term here.",
        "Alpha Term",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_coverage_args(&temp))
        .output()
        .expect("glossary coverage gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("glossary cross-doc coverage validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_cross_doc_gate_accepts_active_and_retired_term_policy() {
    let temp = temp_dir("glossary-covered");
    write_glossary_coverage_fixture(
        &temp,
        "Alpha Term\nOld Alias",
        "Alpha Term is covered outside the glossary.",
        "Alpha Term",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_coverage_args(&temp))
        .output()
        .expect("glossary coverage gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("glossary cross-doc coverage validation passed: 2 terms, 1 cross-doc terms")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_cross_doc_gate_decodes_machine_unicode_escapes() {
    let temp = temp_dir("glossary-unicode-escapes");
    write_glossary_coverage_fixture(
        &temp,
        "공공정보법\nOld Alias",
        "공공정보법 is covered outside the glossary.",
        r"\uacf5\uacf5\uc815\ubcf4\ubc95",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_coverage_args(&temp))
        .output()
        .expect("glossary coverage gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("glossary cross-doc coverage validation passed: 2 terms, 1 cross-doc terms")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_rejects_forbidden_active_vocabulary() {
    let temp = temp_dir("glossary-vocab-forbidden");
    write_glossary_vocabulary_fixture(&temp, "Trust portal MVP launch.", &["ADR"]);
    write_glossary_vocabulary_baseline(&temp, &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_vocabulary_args(&temp))
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("glossary vocabulary validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_rejects_warning_outside_baseline() {
    let temp = temp_dir("glossary-vocab-new-warning");
    write_glossary_vocabulary_fixture(&temp, "ABC appears.", &["ADR"]);
    write_glossary_vocabulary_baseline(&temp, &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_vocabulary_args(&temp))
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("NewWarningOutsideBaseline"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_accepts_forensic_retired_terms_and_reports_warnings() {
    let temp = temp_dir("glossary-vocab-warnings");
    write_glossary_vocabulary_fixture(
        &temp,
        "oyatie documents ABC support.",
        &["ABC", "MVP", "CUG"],
    );
    write_glossary_vocabulary_baseline(&temp, &["casing-variant\toyatie"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_vocabulary_args(&temp))
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "glossary vocabulary validation passed: 2 documents, 1 casing warnings, 0 uncited acronym warnings"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_uses_rationalized_ignored_uppercase_words() {
    let temp = temp_dir("glossary-vocab-ignored-words");
    write_glossary_vocabulary_fixture(
        &temp,
        "YES for ALL docs; ABC remains.",
        &["ADR", "MVP", "CUG"],
    );
    write_glossary_vocabulary_ignored_words(
        &temp,
        &[
            "YES\tdoc-catalog boolean value",
            "ALL\temphatic ordinary prose word",
        ],
    );
    write_glossary_vocabulary_baseline(&temp, &["uncited-acronym\tABC"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_vocabulary_args(&temp))
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "glossary vocabulary validation passed: 2 documents, 0 casing warnings, 1 uncited acronym warnings"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_rejects_stale_ignored_uppercase_words() {
    let temp = temp_dir("glossary-vocab-stale-ignored-word");
    write_glossary_vocabulary_fixture(&temp, "ABC remains.", &["ADR", "MVP", "CUG"]);
    write_glossary_vocabulary_ignored_words(&temp, &["YES\tdoc-catalog boolean value"]);
    write_glossary_vocabulary_baseline(&temp, &["uncited-acronym\tABC"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(glossary_vocabulary_args(&temp))
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("StaleIgnoredUppercaseWord"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn glossary_vocabulary_gate_writes_warning_source_report() {
    let temp = temp_dir("glossary-vocab-warning-report");
    write_glossary_vocabulary_fixture(
        &temp,
        "oyatie documents ABC support.",
        &["ADR", "MVP", "CUG"],
    );
    write_glossary_vocabulary_baseline(&temp, &["casing-variant\toyatie", "uncited-acronym\tABC"]);
    let report_path = temp.join("warning-report.tsv");
    let mut args = glossary_vocabulary_args(&temp);
    args.push("--write-warning-report".into());
    args.push(report_path.to_str().expect("utf8 warning report").into());

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args)
        .output()
        .expect("glossary vocabulary gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string(report_path).expect("warning report written");
    assert!(report.contains("casing-variant\toyatie\tdocs/README.md"));
    assert!(report.contains("uncited-acronym\tABC\tdocs/README.md"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn placeholder_debt_gate_rejects_unregistered_marker() {
    let temp = temp_dir("placeholder-debt-unregistered");
    write_placeholder_debt_docs(&temp, "TODO: decide the owner.");
    write_placeholder_debt_registry(&temp, &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(placeholder_debt_args(&temp))
        .output()
        .expect("placeholder debt gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("NewPlaceholderOutsideRegistry"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn placeholder_debt_gate_accepts_registry_and_writes_report() {
    let temp = temp_dir("placeholder-debt-tracked");
    write_placeholder_debt_docs(&temp, "TODO: decide the owner. TBD by council.");
    write_placeholder_debt_registry(
        &temp,
        &[
            "TODO\tdocs/README.md\t1\tTODO: decide the owner. TBD by council.\towner=council-architecture; issue=PLACEHOLDER-DEBT-README; captured_at=2026-05-10; action=burn-down",
            "TBD\tdocs/README.md\t1\tTODO: decide the owner. TBD by council.\towner=council-architecture; issue=PLACEHOLDER-DEBT-README; captured_at=2026-05-10; action=burn-down",
        ],
    );
    let report_path = temp.join("placeholder-report.tsv");
    let mut args = placeholder_debt_args(&temp);
    args.push("--write-report".into());
    args.push(report_path.to_str().expect("utf8 report").into());

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args)
        .output()
        .expect("placeholder debt gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "placeholder debt validation passed: 1 documents, 2 open placeholders, 2 registry records"
    ));
    let report = fs::read_to_string(report_path).expect("placeholder report written");
    assert!(report.contains("TODO\tdocs/README.md\t1\tTODO: decide the owner. TBD by council."));
    assert!(report.contains("TBD\tdocs/README.md\t1\tTODO: decide the owner. TBD by council."));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn placeholder_debt_gate_writes_accountable_registry() {
    let temp = temp_dir("placeholder-debt-accountable");
    write_placeholder_debt_docs(&temp, "TODO: decide the owner.");
    let registry_path = temp.join("generated-registry.tsv");
    let mut args = placeholder_debt_args(&temp);
    args.push("--write-registry".into());
    args.push(
        registry_path
            .to_str()
            .expect("utf8 generated registry")
            .into(),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args)
        .output()
        .expect("placeholder debt gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let registry = fs::read_to_string(&registry_path).expect("placeholder registry written");
    assert!(registry.contains("TODO\tdocs/README.md\t1\tTODO: decide the owner."));
    assert!(registry.contains("owner=council-architecture; issue=PLACEHOLDER-DEBT-AUTO-CAPTURE; captured_at=2026-05-19; action=close-or-archive-before-production-claim"));

    let enforcement_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "placeholder-debt",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--registry",
            registry_path.to_str().expect("utf8 generated registry"),
        ])
        .output()
        .expect("placeholder debt enforcement command runs");
    assert!(
        enforcement_output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&enforcement_output.stdout),
        String::from_utf8_lossy(&enforcement_output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn quality_lanes_gate_accepts_registry_doc_and_check_wiring() {
    let temp = temp_dir("quality-lanes-valid");
    write_quality_lanes_fixture(
        &temp,
        "cargo fmt --all -- --check",
        "`cargo fmt --all -- --check`",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(quality_lanes_args(&temp))
        .output()
        .expect("quality lanes gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "quality lane validation passed: 1 registry records, 1 markdown rows, 1 active commands, 1 owner teams"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn quality_lanes_gate_rejects_markdown_purpose_drift() {
    let temp = temp_dir("quality-lanes-purpose-drift");
    write_quality_lanes_fixture(
        &temp,
        "cargo fmt --all -- --check",
        "`cargo check --workspace`",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(quality_lanes_args(&temp))
        .output()
        .expect("quality lanes gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("PurposeDrift"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn quality_lanes_gate_rejects_unwired_active_command() {
    let temp = temp_dir("quality-lanes-unwired");
    write_quality_lanes_fixture(
        &temp,
        "cargo fmt --all -- --check",
        "`cargo fmt --all -- --check`",
    );
    fs::write(temp.join("check.sh"), "#!/usr/bin/env bash\ncargo check\n")
        .expect("check script rewritten");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(quality_lanes_args(&temp))
        .output()
        .expect("quality lanes gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("CheckCommandNotWired"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn quality_lanes_gate_rejects_unknown_owner_team() {
    let temp = temp_dir("quality-lanes-unknown-owner");
    write_quality_lanes_fixture(
        &temp,
        "cargo fmt --all -- --check",
        "`cargo fmt --all -- --check`",
    );
    let registry_path = temp.join("registry/quality/lanes.yaml");
    let registry = fs::read_to_string(&registry_path)
        .expect("quality lane registry readable")
        .replace("owner_team: axis-foundry", "owner_team: missing-team");
    fs::write(registry_path, registry).expect("quality lane registry rewritten");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(quality_lanes_args(&temp))
        .output()
        .expect("quality lanes gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("UnknownOwnerTeam"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_rejects_root_doc_missing_machine_row() {
    let temp = temp_dir("doc-catalog-missing-machine");
    write_doc_catalog_fixture(&temp, &["README.md", "CONSTITUTION.md"], &["README.md"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            temp.join("machine-readable/catalog.json")
                .to_str()
                .expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("doc catalog validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_accepts_root_docs_with_machine_and_markdown_rows() {
    let temp = temp_dir("doc-catalog-complete");
    write_doc_catalog_fixture(
        &temp,
        &["README.md", "CONSTITUTION.md"],
        &["README.md", "CONSTITUTION.md", "DOC-CATALOG.md"],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            temp.join("machine-readable/catalog.json")
                .to_str()
                .expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("doc catalog validation passed: 3 documents")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_accepts_owner_team_arrays_from_machine_catalog() {
    let temp = temp_dir("doc-catalog-owner-array");
    write_doc_catalog_fixture(&temp, &["README.md"], &["README.md", "DOC-CATALOG.md"]);
    let catalog_path = temp.join("machine-readable/catalog.json");
    let catalog = fs::read_to_string(&catalog_path).expect("machine catalog read");
    fs::write(
        &catalog_path,
        catalog.replace(
            r#""owner_team": "council-architecture""#,
            r#""owner_team": ["council-architecture", "axis-foundry"]"#,
        ),
    )
    .expect("machine catalog updated");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            catalog_path.to_str().expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("doc catalog validation passed: 2 documents")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_rejects_unknown_dependent_doc_id() {
    let temp = temp_dir("doc-catalog-unknown-dependency");
    write_doc_catalog_fixture(&temp, &["README.md"], &["README.md", "DOC-CATALOG.md"]);
    let catalog_path = temp.join("machine-readable/catalog.json");
    let catalog = fs::read_to_string(&catalog_path).expect("machine catalog read");
    fs::write(
        &catalog_path,
        catalog.replacen(
            r#""dependent_docs": []"#,
            r#""dependent_docs": ["doc.missing"]"#,
            1,
        ),
    )
    .expect("machine catalog updated");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            catalog_path.to_str().expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("doc catalog validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_rejects_unresolved_path_dependency() {
    let temp = temp_dir("doc-catalog-unresolved-path");
    write_doc_catalog_fixture(&temp, &["README.md"], &["README.md", "DOC-CATALOG.md"]);
    let catalog_path = temp.join("machine-readable/catalog.json");
    let catalog = fs::read_to_string(&catalog_path).expect("machine catalog read");
    fs::write(
        &catalog_path,
        catalog.replacen(
            r#""dependent_docs": []"#,
            r#""dependent_docs": ["products/*/PRD.md"]"#,
            1,
        ),
    )
    .expect("machine catalog updated");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            catalog_path.to_str().expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("doc catalog validation failed"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_accepts_path_glob_adr_and_codeowners_dependencies() {
    let temp = temp_dir("doc-catalog-resolved-dependencies");
    write_doc_catalog_fixture(&temp, &["README.md"], &["README.md", "DOC-CATALOG.md"]);
    fs::create_dir_all(temp.join("products/foundry")).expect("product dir created");
    fs::write(temp.join("products/foundry/PRD.md"), "# Foundry PRD\n").expect("product written");
    fs::create_dir_all(temp.join("decisions")).expect("decisions dir created");
    fs::write(
        temp.join("decisions/ADR-0050-automation-first-pipeline.md"),
        "# ADR-0050\n",
    )
    .expect("adr written");
    fs::create_dir_all(temp.join(".github")).expect("github dir created");
    fs::write(
        temp.join(".github/CODEOWNERS"),
        "* @teams/council-architecture\n",
    )
    .expect("codeowners written");

    let catalog_path = temp.join("machine-readable/catalog.json");
    let catalog = fs::read_to_string(&catalog_path).expect("machine catalog read");
    fs::write(
        &catalog_path,
        catalog.replacen(
            r#""dependent_docs": []"#,
            r#""dependent_docs": ["products/*/PRD.md", "machine-readable/catalog.json (this file)", "ADR-0050", ".github/CODEOWNERS"]"#,
            1,
        ),
    )
    .expect("machine catalog updated");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            catalog_path.to_str().expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("doc catalog validation passed: 2 documents")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_catalog_gate_accepts_workspace_spec_dependencies() {
    let temp = temp_dir("doc-catalog-spec-dependencies");
    write_doc_catalog_fixture(&temp, &["README.md"], &["README.md", "DOC-CATALOG.md"]);
    fs::create_dir_all(temp.join("specs")).expect("spec dir created");
    fs::write(temp.join("specs/decision-principles.json"), "{}\n").expect("spec written");

    let catalog_path = temp.join("machine-readable/catalog.json");
    let catalog = fs::read_to_string(&catalog_path).expect("machine catalog read");
    fs::write(
        &catalog_path,
        catalog.replacen(
            r#""dependent_docs": []"#,
            r#""dependent_docs": ["/specs/decision-principles.json"]"#,
            1,
        ),
    )
    .expect("machine catalog updated");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "doc-catalog",
            "--docs-dir",
            temp.to_str().expect("utf8 docs dir"),
            "--catalog",
            catalog_path.to_str().expect("utf8 catalog"),
        ])
        .output()
        .expect("doc-catalog gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("doc catalog validation passed: 2 documents")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn documentation_system_gate_accepts_pipeline_registry_and_quickref_baseline() {
    let temp = temp_dir("documentation-system-valid");
    write_documentation_system_fixture(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(documentation_system_args(&temp))
        .output()
        .expect("documentation system gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "documentation system validation passed: 6 pipeline records, 2 active, 3 adoption-guard, 1 tracked-deferred"
        ),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn documentation_system_gate_rejects_missing_quickref_baseline() {
    let temp = temp_dir("documentation-system-missing-quickref");
    write_documentation_system_fixture(&temp);
    fs::remove_file(temp.join("docs/wiki/quickref/README.md")).expect("quickref removed");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(documentation_system_args(&temp))
        .output()
        .expect("documentation system gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("WikiQuickrefMissing"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn documentation_system_gate_rejects_unwired_pipeline_guard() {
    let temp = temp_dir("documentation-system-unwired");
    write_documentation_system_fixture(&temp);
    fs::write(
        temp.join("scripts/check.sh"),
        "cargo run -p oya-dev-cli -- gate validate documentation-system\n",
    )
    .expect("check script rewritten");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(documentation_system_args(&temp))
        .output()
        .expect("documentation system gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("UnwiredPipelineCommand"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

fn write_documentation_system_fixture(root: &Path) {
    fs::create_dir_all(root.join("docs/wiki/quickref")).expect("quickref dir created");
    fs::create_dir_all(root.join("docs/decisions")).expect("decisions dir created");
    fs::create_dir_all(root.join("docs/machine-readable")).expect("machine docs dir created");
    fs::create_dir_all(root.join("registry/catalog")).expect("catalog dir created");
    fs::create_dir_all(root.join("registry/docs")).expect("docs registry dir created");
    fs::create_dir_all(root.join("scripts")).expect("scripts dir created");
    fs::write(
        root.join("docs/DOCUMENTATION.md"),
        "# Documentation System\n\nThe `oya-governance-docs` lane covers docs/wiki/quickref/*.\n",
    )
    .expect("documentation system doc written");
    fs::write(
        root.join("docs/wiki/quickref/README.md"),
        "# Wiki quick reference\n\nOwned by `council-architecture`.\n",
    )
    .expect("quickref written");
    fs::write(
        root.join("scripts/check.sh"),
        "cargo run -p oya-dev-cli -- gate validate documentation-system\ncargo run -p oya-dev-cli -- gate validate api-semver\ncargo run -p oya-dev-cli -- gate validate adr-citation\ncargo run -p oya-dev-cli -- catalog validate\ncargo run -p oya-dev-cli -- gate validate doc-catalog\n",
    )
    .expect("check script written");
    fs::write(
        root.join("registry/docs/pipeline.tsv"),
        documentation_pipeline_tsv(),
    )
    .expect("documentation pipeline registry written");
}

fn documentation_pipeline_tsv() -> &'static str {
    "step_id\tdocumented_command\tstate\tcheck_command\tscope_path\trationale\nrustdoc\toya doc rustdoc\ttracked-deferred\t\tcrates\tblocked: full rustdoc artifact publication is not part of the bootstrap lane\nopenapi\toya doc openapi\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate api-semver\tcontracts\tcontracts are absent; api-semver guards first contract adoption\nmdbook\toya doc mdbook\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate documentation-system\tdocs/site\tpublic mdbook source is absent; documentation-system guards the pipeline registry\nadr-index\toya doc adr-index\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate adr-citation\tdocs/decisions\tadr-citation prevents stale ADR references until generator publication ships\ncatalog\toya doc catalog\tactive\tcargo run -p oya-dev-cli -- catalog validate\tregistry/catalog\t\nlint\toya doc lint\tactive\tcargo run -p oya-dev-cli -- gate validate doc-catalog\tdocs\t\n"
}

fn documentation_system_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "documentation-system".into(),
        "--documentation".into(),
        root.join("docs/DOCUMENTATION.md")
            .to_str()
            .expect("utf8 docs")
            .into(),
        "--pipeline".into(),
        root.join("registry/docs/pipeline.tsv")
            .to_str()
            .expect("utf8 pipeline")
            .into(),
        "--check-script".into(),
        root.join("scripts/check.sh")
            .to_str()
            .expect("utf8 check script")
            .into(),
        "--wiki-quickref".into(),
        root.join("docs/wiki/quickref/README.md")
            .to_str()
            .expect("utf8 quickref")
            .into(),
        "--repo-root".into(),
        root.to_str().expect("utf8 root").into(),
    ]
}

fn write_authority_doc(docs_dir: &Path, file_name: &str, second_line: &str) {
    fs::write(
        docs_dir.join(file_name),
        format!(
            "---\nauthority_chain_declaration: |\n  docs/consolidated/CONSTITUTION.md\n  > {second_line}\n---\n"
        ),
    )
    .expect("authority doc written");
}

fn write_slo_catalog_record(registry_dir: &Path, crate_id: &str, slo: Option<&str>) {
    fs::create_dir_all(registry_dir).expect("registry dir created");
    let slo_row = slo.map(|slo| format!("slo: {slo}\n")).unwrap_or_default();
    fs::write(
        registry_dir.join(format!("{crate_id}.yaml")),
        format!(
            "context: foundry\nrole: kernel\ncapability: capability\nplane: control\n{slo_row}data_classes_owned: [INTERNAL_ONLY]\napi_stability: preview\nsecurity_review: unreviewed\nsupply_chain: source-only\n"
        ),
    )
    .expect("catalog record written");
}

fn write_kernel_workspace(root: &Path, lib_rs: &str) {
    let crate_dir = root.join("crates/example-kernel");
    fs::create_dir_all(crate_dir.join("src")).expect("crate source dir created");
    write_workspace_manifest(root, &["crates/example-kernel"]);
    write_package_manifest(&crate_dir, "example-kernel");
    fs::write(crate_dir.join("src/lib.rs"), lib_rs).expect("kernel source written");
}

fn write_workspace_manifest(root: &Path, members: &[&str]) {
    fs::create_dir_all(root).expect("workspace dir created");
    let members = members
        .iter()
        .map(|member| format!("\"{member}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members}]\n"),
    )
    .expect("workspace manifest written");
}

fn write_package_manifest(crate_dir: &Path, package_name: &str) {
    fs::create_dir_all(crate_dir.join("src")).expect("crate source dir created");
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package_name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n"
        ),
    )
    .expect("crate manifest written");
    fs::write(crate_dir.join("src/lib.rs"), "").expect("crate lib written");
}

fn write_contracts_json(path: &Path, contract_id: &str, location: &str, change_review: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "cross_axis_contracts": [
    {{
      "id": "{contract_id}",
      "owner_axis": "saas",
      "consumer_axes": ["all"],
      "location": "{location}",
      "change_review": "{change_review}"
    }}
  ]
}}"#
        ),
    )
    .expect("contracts json written");
}

fn write_codeowners_fixture(root: &Path, fallback_owner: &str) {
    let teams = root.join("teams");
    for team_id in ["council-architecture", "axis-foundry"] {
        fs::create_dir_all(teams.join(team_id)).expect("team dir created");
        fs::write(
            teams.join(team_id).join("CHARTER.md"),
            format!("# {team_id}\n"),
        )
        .expect("team charter written");
    }
    fs::create_dir_all(root.join(".github")).expect("github dir created");
    fs::write(
        root.join(".github/CODEOWNERS"),
        format!(
            "* {fallback_owner}\ncrates/oya-foundry-* @teams/axis-foundry\nregistry/catalog/ @teams/axis-foundry\ndocs/teams/*/CHARTER.md @teams/council-architecture\ndocs/RACI-OWNERSHIP.md @teams/council-architecture\n"
        ),
    )
    .expect("CODEOWNERS written");
}

fn write_raci_team_coverage_fixture(
    root: &Path,
    include_all_raci: bool,
    include_all_codeowners: bool,
) {
    let teams = root.join("teams");
    for team_id in ["axis-foundry", "axis-cloud"] {
        fs::create_dir_all(teams.join(team_id)).expect("team dir created");
        fs::write(
            teams.join(team_id).join("CHARTER.md"),
            format!("# {team_id}\n"),
        )
        .expect("team charter written");
    }
    let raci_rows = if include_all_raci {
        "| `axis-foundry` | `docs/teams/axis-foundry/CHARTER.md` | `@teams/axis-foundry` |\n| `axis-cloud` | `docs/teams/axis-cloud/CHARTER.md` | `@teams/axis-cloud` |\n"
    } else {
        "| `axis-foundry` | `docs/teams/axis-foundry/CHARTER.md` | `@teams/axis-foundry` |\n"
    };
    fs::write(
        root.join("RACI-OWNERSHIP.md"),
        format!(
            "# RACI\n\n## Team charter coverage\n\n| team_id | charter | codeowners |\n|---|---|---|\n{raci_rows}"
        ),
    )
    .expect("RACI written");
    fs::create_dir_all(root.join(".github")).expect("github dir created");
    let cloud_owner = if include_all_codeowners {
        "docs/teams/axis-cloud/CHARTER.md @teams/axis-cloud\n"
    } else {
        ""
    };
    fs::write(
        root.join(".github/CODEOWNERS"),
        format!("docs/teams/axis-foundry/CHARTER.md @teams/axis-foundry\n{cloud_owner}"),
    )
    .expect("CODEOWNERS written");
}

fn raci_team_coverage_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "raci-team-coverage".into(),
        "--teams-dir".into(),
        root.join("teams").to_str().expect("utf8 teams dir").into(),
        "--raci".into(),
        root.join("RACI-OWNERSHIP.md")
            .to_str()
            .expect("utf8 raci")
            .into(),
        "--codeowners".into(),
        root.join(".github/CODEOWNERS")
            .to_str()
            .expect("utf8 codeowners")
            .into(),
    ]
}

fn write_readme_doc_coverage_fixture(docs_dir: &Path, root_docs: &[&str], readme_links: &[&str]) {
    fs::create_dir_all(docs_dir.join("machine-readable")).expect("machine-readable dir created");
    for doc in root_docs {
        fs::write(docs_dir.join(doc), format!("# {doc}\n")).expect("root doc written");
    }
    let links = readme_links
        .iter()
        .map(|doc| format!("- [`{doc}`]({doc})\n"))
        .collect::<String>();
    fs::write(docs_dir.join("README.md"), format!("# Docs\n\n{links}")).expect("README written");
    let entries = root_docs
        .iter()
        .map(|doc| {
            let doc_id = doc
                .trim_end_matches(".md")
                .to_ascii_lowercase()
                .replace('-', "_");
            format!(
                r#""doc.{doc_id}": {{
      "path": "docs/{doc}",
      "owner_team": "council-architecture",
      "dependent_docs": [],
      "validation_check": "readme-doc-coverage"
    }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(
        docs_dir.join("machine-readable/catalog.json"),
        format!("{{\n  \"docs\": {{\n{entries}\n  }}\n}}\n"),
    )
    .expect("machine catalog written");
}

fn readme_doc_coverage_args(docs_dir: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "readme-doc-coverage".into(),
        "--docs-dir".into(),
        docs_dir.to_str().expect("utf8 docs dir").into(),
        "--catalog".into(),
        docs_dir
            .join("machine-readable/catalog.json")
            .to_str()
            .expect("utf8 catalog")
            .into(),
    ]
}

fn write_glossary_coverage_fixture(
    docs_dir: &Path,
    glossary_body: &str,
    cross_doc_body: &str,
    active_term: &str,
) {
    fs::create_dir_all(docs_dir.join("machine-readable")).expect("machine-readable dir created");
    fs::write(
        docs_dir.join("GLOSSARY.md"),
        format!("# Glossary\n\n{glossary_body}\n"),
    )
    .expect("glossary written");
    fs::write(
        docs_dir.join("README.md"),
        format!("# Docs\n\n{cross_doc_body}\n"),
    )
    .expect("cross doc written");
    fs::write(
        docs_dir.join("machine-readable/glossary.json"),
        format!(
            r#"{{
  "term_categories": {{
    "industry_standard": {{
      "architecture": ["{active_term}"],
      "operations": [],
      "cloud": [],
      "auth": [],
      "data_search_ml": [],
      "ads": [],
      "compliance_kr": [],
      "compliance_global": []
    }},
    "oyatie_specific": [],
    "retired_terms": [
      {{"old": "Old Alias", "new": "Current Term", "retirement_date": "2026-05-09"}}
    ]
  }}
}}
"#
        ),
    )
    .expect("machine glossary written");
}

fn glossary_coverage_args(docs_dir: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "glossary-cross-doc-coverage".into(),
        "--docs-dir".into(),
        docs_dir.to_str().expect("utf8 docs dir").into(),
        "--glossary".into(),
        docs_dir
            .join("GLOSSARY.md")
            .to_str()
            .expect("utf8 glossary")
            .into(),
        "--machine".into(),
        docs_dir
            .join("machine-readable/glossary.json")
            .to_str()
            .expect("utf8 machine glossary")
            .into(),
    ]
}

fn write_glossary_vocabulary_fixture(docs_dir: &Path, active_doc_body: &str, acronyms: &[&str]) {
    fs::create_dir_all(docs_dir).expect("docs dir created");
    let acronym_rows = acronyms
        .iter()
        .map(|acronym| format!("| {acronym} | Test expansion | fixture |\n"))
        .collect::<String>();
    fs::write(
        docs_dir.join("GLOSSARY.md"),
        format!(
            "# Glossary\n\n## 10. Acronym index\n\n| Acronym | Expansion | See |\n|---|---|---|\n{acronym_rows}\n## 11. Deprecated / renamed terms\n\nM0 / M1 / M2 / M3 / MVP and CUG are retired.\n"
        ),
    )
    .expect("glossary written");
    fs::write(
        docs_dir.join("README.md"),
        format!("# Docs\n\n{active_doc_body}\n"),
    )
    .expect("active doc written");
    write_glossary_vocabulary_ignored_words(docs_dir, &[]);
}

fn write_glossary_vocabulary_baseline(docs_dir: &Path, warning_rows: &[&str]) {
    let rows = warning_rows
        .iter()
        .map(|row| format!("{row}\n"))
        .collect::<String>();
    fs::write(
        docs_dir.join("warning-baseline.tsv"),
        format!("# test glossary warning baseline\n{rows}"),
    )
    .expect("warning baseline written");
}

fn write_glossary_vocabulary_ignored_words(docs_dir: &Path, ignored_rows: &[&str]) {
    let rows = ignored_rows
        .iter()
        .map(|row| format!("{row}\n"))
        .collect::<String>();
    fs::write(
        docs_dir.join("ignored-uppercase-words.tsv"),
        format!("# test glossary ignored uppercase prose words\n{rows}"),
    )
    .expect("ignored uppercase words written");
}

fn glossary_vocabulary_args(docs_dir: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "glossary-vocabulary".into(),
        "--docs-dir".into(),
        docs_dir.to_str().expect("utf8 docs dir").into(),
        "--glossary".into(),
        docs_dir
            .join("GLOSSARY.md")
            .to_str()
            .expect("utf8 glossary")
            .into(),
        "--baseline".into(),
        docs_dir
            .join("warning-baseline.tsv")
            .to_str()
            .expect("utf8 baseline")
            .into(),
        "--ignored-uppercase-words".into(),
        docs_dir
            .join("ignored-uppercase-words.tsv")
            .to_str()
            .expect("utf8 ignored uppercase words")
            .into(),
    ]
}

fn write_placeholder_debt_docs(docs_dir: &Path, body: &str) {
    fs::create_dir_all(docs_dir).expect("docs dir created");
    fs::write(docs_dir.join("README.md"), format!("# Docs\n\n{body}\n"))
        .expect("placeholder doc written");
}

fn write_placeholder_debt_registry(docs_dir: &Path, rows: &[&str]) {
    let rows = rows
        .iter()
        .map(|row| format!("{row}\n"))
        .collect::<String>();
    fs::write(
        docs_dir.join("placeholder-registry.tsv"),
        format!("# test placeholder debt registry\n{rows}"),
    )
    .expect("placeholder registry written");
}

fn placeholder_debt_args(docs_dir: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "placeholder-debt".into(),
        "--docs-dir".into(),
        docs_dir.to_str().expect("utf8 docs dir").into(),
        "--registry".into(),
        docs_dir
            .join("placeholder-registry.tsv")
            .to_str()
            .expect("utf8 placeholder registry")
            .into(),
    ]
}

fn write_adr_citation_fixture(root: &Path, active_body: &str, forensic_body: &str) {
    fs::create_dir_all(root.join("docs/decisions")).expect("decisions dir created");
    fs::write(
        root.join("docs/README.md"),
        format!("# Docs\n\n{active_body}\n"),
    )
    .expect("active doc written");
    fs::write(
        root.join("docs/ADR-LEGACY-REGRESSION-MAPPING.md"),
        format!("# Legacy mapping\n\n{forensic_body}\n"),
    )
    .expect("forensic mapping written");
    fs::write(
        root.join("docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md"),
        "# Cohesion\n",
    )
    .expect("ADR-0001 written");
    fs::write(root.join("docs/adr-archive/ADR-0051-mobile-and-native-client-strategy.md"), "# Mobile\n")
        .expect("ADR-0051 written");
}

fn adr_citation_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "adr-citation".into(),
        "--docs-dir".into(),
        root.join("docs").to_str().expect("utf8 docs dir").into(),
        "--decisions-dir".into(),
        root.join("docs/decisions")
            .to_str()
            .expect("utf8 decisions dir")
            .into(),
        // Pin to a nonexistent registry so the allowed-ADR count is determined
        // solely by the fixture's two pack ADRs (ADR-0001, ADR-0051), making
        // the count assertion stable regardless of the live inheritance registry.
        "--inheritance-registry".into(),
        root.join("registry/adr/inherited-bominal-adrs.yaml")
            .to_str()
            .expect("utf8 inheritance registry path")
            .into(),
    ]
}

fn write_brand_residue_file(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().expect("brand residue parent"))
        .expect("brand residue parent dir created");
    fs::write(path, contents).expect("brand residue fixture written");
}

fn brand_residue_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "brand-residue".into(),
        "--docs-dir".into(),
        root.join("docs").to_str().expect("utf8 docs dir").into(),
    ]
}

fn write_api_contract(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().expect("API contract parent"))
        .expect("API contract parent dir created");
    fs::write(path, contents).expect("API contract fixture written");
}

fn api_semver_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "api-semver".into(),
        "--contracts-dir".into(),
        root.join("contracts")
            .to_str()
            .expect("utf8 contracts dir")
            .into(),
    ]
}

fn write_supply_chain_fixture(
    root: &Path,
    attestation: &str,
    check_script: &str,
    workflow: Option<&str>,
    release_manifest: bool,
) {
    fs::create_dir_all(root.join("registry/catalog")).expect("supply catalog dir created");
    fs::create_dir_all(root.join("scripts")).expect("scripts dir created");
    fs::write(
        root.join("registry/catalog/oya-intelligence-capability-kernel.yaml"),
        format!(
            "context: foundry\nrole: kernel\ncapability: capability\nplane: control\ndata_classes_owned: [INTERNAL_ONLY]\napi_stability: preview\nsecurity_review: unreviewed\nsupply_chain: {attestation}\n"
        ),
    )
    .expect("supply catalog record written");
    fs::write(root.join("deny.toml"), "[licenses]\nallow = []\n").expect("deny config written");
    fs::write(root.join("scripts/check.sh"), check_script).expect("check script written");
    if let Some(workflow) = workflow {
        fs::create_dir_all(root.join(".github/workflows")).expect("workflows dir created");
        fs::write(root.join(".github/workflows/supply.yml"), workflow).expect("workflow written");
    }
    if release_manifest {
        fs::create_dir_all(root.join("contracts/release")).expect("release contract dir created");
        fs::write(
            root.join("contracts/release/images.yaml"),
            "images:\n  - name: ghcr.io/oyatie/app\n",
        )
        .expect("release images contract written");
    }
}

fn supply_chain_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "supply-chain".into(),
        "--registry".into(),
        root.join("registry/catalog")
            .to_str()
            .expect("utf8 registry")
            .into(),
        "--deny".into(),
        root.join("deny.toml").to_str().expect("utf8 deny").into(),
        "--check-script".into(),
        root.join("scripts/check.sh")
            .to_str()
            .expect("utf8 check script")
            .into(),
        "--adr0039-script".into(),
        root.join("scripts/supply-chain-adr0039.sh")
            .to_str()
            .expect("utf8 adr0039 script")
            .into(),
        // Pin --adr0039-rust to a fixture-local (non-existent) path so tests
        // are hermetic and cannot accidentally inherit trivy evidence from the
        // live repo source file (marketplace/facade/dev-cli/src/commands/supply_chain.rs).
        "--adr0039-rust".into(),
        root.join("src/commands/supply_chain.rs")
            .to_str()
            .expect("utf8 adr0039 rust")
            .into(),
        "--workflows-dir".into(),
        root.join(".github/workflows")
            .to_str()
            .expect("utf8 workflows dir")
            .into(),
        "--release-images".into(),
        root.join("contracts/release/images.yaml")
            .to_str()
            .expect("utf8 release images")
            .into(),
    ]
}

fn supply_chain_full_args(root: &Path) -> Vec<String> {
    let mut args = supply_chain_args(root);
    args.extend([
        "--branch-protection".into(),
        root.join(".github/branch-protection.yaml")
            .to_str()
            .expect("utf8 branch protection")
            .into(),
        "--admission-policy".into(),
        root.join("infra/kyverno/policies/require-signed-images.yaml")
            .to_str()
            .expect("utf8 admission policy")
            .into(),
        "--require-adr0039-evidence".into(),
    ]);
    args
}

fn write_release_supply_chain_fixture(
    root: &Path,
    rekor_log_index: &str,
    high_critical_findings_open: &str,
    signed: &str,
) {
    let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let artifact_ref = format!("ghcr.io/oyatie/oya-dev-cli@{digest}");
    fs::create_dir_all(root.join("registry/release/supply-chain"))
        .expect("release supply-chain evidence dir created");
    fs::write(
        root.join("registry/release/images.yaml"),
        format!("images:\n  - ref: {artifact_ref}\n"),
    )
    .expect("release image manifest written");
    fs::write(
        root.join("registry/release/supply-chain/oya-dev-cli.yaml"),
        format!(
            r#"artifact_ref: {artifact_ref}
artifact_digest: {digest}
release_version: 0.1.0
source_revision: 0123456789abcdef0123456789abcdef01234567
sbom_spdx_ref: artifact://release/0.1.0/oya-dev-cli.spdx.json
sbom_cyclonedx_ref: artifact://release/0.1.0/oya-dev-cli.cyclonedx.json
cosign_signature_ref: rekor://log/{rekor_log_index}/signature
cosign_certificate_ref: rekor://log/{rekor_log_index}/certificate
rekor_log_index: {rekor_log_index}
trivy_filesystem_scan_ref: artifact://release/0.1.0/trivy-fs.sarif
trivy_container_scan_ref: artifact://release/0.1.0/trivy-image.sarif
trivy_iac_scan_ref: artifact://release/0.1.0/trivy-iac.sarif
trivy_dependency_scan_ref: artifact://release/0.1.0/trivy-dep.sarif
provenance_attestation_ref: artifact://release/0.1.0/provenance.intoto.jsonl
audit_event_type: oya.audit.builder_supply_attest
attestor: axis-foundry
high_critical_findings_open: {high_critical_findings_open}
signed: {signed}
"#
        ),
    )
    .expect("release supply-chain evidence written");
}

fn release_supply_chain_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "release-supply-chain".into(),
        "--release-images".into(),
        root.join("registry/release/images.yaml")
            .to_str()
            .expect("utf8 release images")
            .into(),
        "--evidence-dir".into(),
        root.join("registry/release/supply-chain")
            .to_str()
            .expect("utf8 release supply-chain evidence dir")
            .into(),
    ]
}

fn release_supply_chain_args_with_phase(root: &Path, phase: &str) -> Vec<String> {
    let mut args = release_supply_chain_args(root);
    args.extend(["--phase".into(), phase.into()]);
    args
}

fn write_image_promotion_fixture(root: &Path, missing_tier: Option<&str>) {
    let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fs::create_dir_all(root.join("registry/release/image-promotions"))
        .expect("image promotion dir created");
    for (tier, verifier) in [
        ("dev", "kubewarden"),
        ("staging", "kubewarden"),
        ("prod", "kyverno"),
    ] {
        if missing_tier == Some(tier) {
            continue;
        }
        fs::write(
            root.join(format!("registry/release/image-promotions/oya-dev-cli-{tier}.yaml")),
            format!(
                r#"artifact_ref: ghcr.io/oyatie/oya-dev-cli:0123456789abcdef0123456789abcdef01234567-{tier}@{digest}
artifact_digest: {digest}
tier: {tier}
cosign_identity: https://token.actions.githubusercontent.com/oyatie/image-promotion-{tier}-oidc
verifier: {verifier}
verifier_ref: infra/{verifier}/policies/require-signed-images.yaml
provenance_attestation_ref: artifact://release/0.1.0/oya-dev-cli-provenance.intoto.jsonl
runner_kill_switch_ref: artifact://fixtures/bootstrap-runner-kill-switch.cedar
audit_event_type: oya.audit.image_promotion
signed: true
"#
            ),
        )
        .expect("image promotion record written");
    }
}

fn image_promotion_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "image-promotion".into(),
        "--promotion-dir".into(),
        root.join("registry/release/image-promotions")
            .to_str()
            .expect("utf8 image promotion dir")
            .into(),
    ]
}

fn write_pre_release_image_manifest(root: &Path) {
    fs::create_dir_all(root.join("registry/release")).expect("release registry dir created");
    fs::write(
        root.join("registry/release/images.yaml"),
        "# release_state: pre-release\n# empty_scope_rationale: No digest-pinned release artifacts exist before a release candidate.\nimages: []\n",
    )
    .expect("release image manifest written");
}

fn write_pre_release_contract_image_manifest(root: &Path) {
    fs::create_dir_all(root.join("contracts/release")).expect("release contract dir created");
    fs::write(
        root.join("contracts/release/images.yaml"),
        "# release_state: pre-release\n# empty_scope_rationale: No digest-pinned release artifacts exist before a release candidate.\nimages: []\n",
    )
    .expect("release image manifest written");
}

fn write_release_evidence_pack_fixture(root: &Path, rows: &str) {
    fs::create_dir_all(root.join("registry/release")).expect("release registry dir created");
    fs::create_dir_all(root.join("docs/machine-readable"))
        .expect("machine-readable docs dir created");
    fs::write(
        root.join("registry/release/evidence-packs.tsv"),
        format!(
            "# release_version: pre-release\n# empty_scope_rationale: No regulator-facing release evidence packs exist before a release candidate.\nregulator\tregion\tpack_id\trelease_version\taudit_cycle\tcoverage_window_start\tcoverage_window_end\towner_team\tevidence_pack_ref\tcosign_attestation_ref\taudit_event_id\trequested_at_epoch_minutes\tregenerated_at_epoch_minutes\tcontrols_mapped\tevidence_links\ttrust_portal_mirror_regenerated\tregulator_notification_sent\tstatus\n{rows}"
        ),
    )
    .expect("release evidence-pack manifest written");
    fs::write(
        root.join("docs/machine-readable/compliance.json"),
        r#"{
  "regulators_per_region": {
    "kr": ["KR PIPA"],
    "eu": ["GDPR"]
  },
  "cross_regional_standards": ["SOC 2 Type II"]
}
"#,
    )
    .expect("compliance matrix fixture written");
}

fn rewrite_release_evidence_pack_version(root: &Path, version: &str, rationale: &str) {
    let path = root.join("registry/release/evidence-packs.tsv");
    let contents = fs::read_to_string(&path).expect("release evidence manifest readable");
    let mut lines = contents.lines().map(str::to_string).collect::<Vec<_>>();
    lines[0] = format!("# release_version: {version}");
    lines[1] = format!("# empty_scope_rationale: {rationale}");
    fs::write(&path, format!("{}\n", lines.join("\n")))
        .expect("release evidence manifest rewritten");
}

fn release_evidence_pack_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "release-evidence-pack".into(),
        "--manifest".into(),
        root.join("registry/release/evidence-packs.tsv")
            .to_str()
            .expect("utf8 release evidence manifest")
            .into(),
        "--compliance".into(),
        root.join("docs/machine-readable/compliance.json")
            .to_str()
            .expect("utf8 compliance matrix")
            .into(),
    ]
}

fn write_supply_chain_adr0039_script(root: &Path) {
    fs::create_dir_all(root.join("scripts")).expect("scripts dir created");
    fs::write(
        root.join("scripts/supply-chain-adr0039.sh"),
        r#"#!/usr/bin/env bash
trivy fs --severity HIGH,CRITICAL --exit-code 1 .
trivy image --severity HIGH,CRITICAL --exit-code 1 "$image"
trivy config --severity HIGH,CRITICAL --exit-code 1 infra/
trivy fs --scanners vuln,secret,license --format sarif --output artifacts/trivy.sarif .
trivy fs --format spdx-json --output artifacts/sbom/oyatie.spdx.json .
trivy fs --format cyclonedx --output artifacts/sbom/oyatie.cyclonedx.json .
cosign sign --yes "$image"
cosign verify --rekor-url https://rekor.sigstore.dev "$image"
cosign attest --yes --predicate artifacts/provenance.json "$image"
"#,
    )
    .expect("ADR-0039 script written");
}

fn write_supply_chain_branch_protection(root: &Path) {
    fs::create_dir_all(root.join(".github")).expect("github dir created");
    fs::write(
        root.join(".github/branch-protection.yaml"),
        "branches:\n  main:\n    require_signed_commits: true\n    require_signed_tags: true\n",
    )
    .expect("branch protection written");
}

fn write_supply_chain_admission_policy(root: &Path) {
    fs::create_dir_all(root.join("infra/kyverno/policies")).expect("kyverno dir created");
    fs::write(
        root.join("infra/kyverno/policies/require-signed-images.yaml"),
        "apiVersion: kyverno.io/v1\nkind: ClusterPolicy\nspec:\n  rules:\n    - name: verify\n      verifyImages:\n        - imageReferences: ['ghcr.io/oyatie/*']\n          attestors:\n            - entries:\n                - keyless:\n                    rekor:\n                      url: https://rekor.sigstore.dev\n",
    )
    .expect("admission policy written");
}

fn write_cargo_prefix_workspace(root: &Path, member_path: &str, package_name: &str) {
    let crate_dir = root.join(member_path);
    fs::create_dir_all(&crate_dir).expect("cargo prefix crate dir created");
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [\"{member_path}\"]\n"),
    )
    .expect("cargo prefix workspace manifest written");
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package_name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n"
        ),
    )
    .expect("cargo prefix package manifest written");
}

fn cargo_prefix_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "cargo-prefix".into(),
        "--workspace".into(),
        root.join("Cargo.toml")
            .to_str()
            .expect("utf8 workspace")
            .into(),
    ]
}

fn write_dependency_seam_gate_fixture(root: &Path, include_offender: bool) {
    fs::create_dir_all(root.join("crates/adapter/src")).expect("adapter dir created");
    let mut members = vec!["\"crates/adapter\"".to_string()];
    if include_offender {
        fs::create_dir_all(root.join("crates/offender/src")).expect("offender dir created");
        fs::write(
            root.join("crates/offender/Cargo.toml"),
            r#"[package]
name = "offender"
edition = "2024"
version = "0.1.0"
license = "Apache-2.0"
[dependencies]
hyper.workspace = true
"#,
        )
        .expect("offender manifest written");
        fs::write(
            root.join("crates/offender/src/lib.rs"),
            "pub fn offender() { let _ = hyper::Version::HTTP_11; }\n",
        )
        .expect("offender source written");
        members.push("\"crates/offender\"".to_string());
    }
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [{}]\n[workspace.dependencies]\nhyper = \"1\"\n",
            members.join(", ")
        ),
    )
    .expect("workspace manifest written");
    fs::write(
        root.join("crates/adapter/Cargo.toml"),
        r#"[package]
name = "adapter"
edition = "2024"
version = "0.1.0"
license = "Apache-2.0"
[dependencies]
hyper.workspace = true
"#,
    )
    .expect("adapter manifest written");
    fs::write(
        root.join("crates/adapter/src/lib.rs"),
        "pub fn adapter() { let _ = hyper::Version::HTTP_11; }\n",
    )
    .expect("adapter source written");
    fs::create_dir_all(root.join("registry")).expect("registry dir created");
    fs::write(
        root.join("registry/dependency-rationales.json"),
        r#"{"entries":{"hyper":{"isolated_in_crate":"adapter"}}}
"#,
    )
    .expect("dependency registry written");
    fs::create_dir_all(root.join("evidence/multispectrum")).expect("evidence dir created");
    fs::write(
        root.join("evidence/multispectrum/dependency-seam-test.json"),
        dependency_seam_valid_evidence(),
    )
    .expect("dependency evidence written");
}

fn dependency_seam_gate_args(root: &Path, report: &Path, severity: &str) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "dependency-seam".into(),
        "--repo-root".into(),
        root.to_str().expect("utf8 root").into(),
        "--fixture-root".into(),
        repo_root()
            .join("crates/oya-check-dependency-seam/tests/fixtures")
            .to_str()
            .expect("utf8 fixture root")
            .into(),
        "--evidence".into(),
        root.join("evidence/multispectrum/dependency-seam-test.json")
            .to_str()
            .expect("utf8 evidence")
            .into(),
        "--severity".into(),
        severity.into(),
        "--emit-report".into(),
        report.to_str().expect("utf8 report").into(),
    ]
}

fn write_dependency_blessed_allowlist_fixture(root: &Path, include_offender: bool) {
    fs::create_dir_all(root.join("crates/clean/src")).expect("clean dir created");
    let mut members = vec!["\"crates/clean\"".to_string()];
    // A clean crate: blessed workspace dep + an exempt oya-* path dep.
    fs::write(
        root.join("crates/clean/Cargo.toml"),
        r#"[package]
name = "clean-adapter"
edition = "2024"
version = "0.1.0"
license = "Apache-2.0"
[dependencies]
tokio.workspace = true
oya-kernel = { path = "../oya-kernel" }
"#,
    )
    .expect("clean manifest written");
    fs::write(root.join("crates/clean/src/lib.rs"), "pub fn clean() {}\n")
        .expect("clean source written");

    if include_offender {
        fs::create_dir_all(root.join("crates/offender/src")).expect("offender dir created");
        fs::write(
            root.join("crates/offender/Cargo.toml"),
            r#"[package]
name = "offender-adapter"
edition = "2024"
version = "0.1.0"
license = "Apache-2.0"
[dependencies]
tokio.workspace = true
sketchy-unblessed-crate = "1"
"#,
        )
        .expect("offender manifest written");
        fs::write(
            root.join("crates/offender/src/lib.rs"),
            "pub fn offender() {}\n",
        )
        .expect("offender source written");
        members.push("\"crates/offender\"".to_string());
    }

    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [{}]\n[workspace.dependencies]\ntokio = \"1\"\n",
            members.join(", ")
        ),
    )
    .expect("workspace manifest written");

    fs::create_dir_all(root.join("registry")).expect("registry dir created");
    fs::write(
        root.join("registry/dependency-blessed-allowlist.json"),
        r#"{"blessed":{"tokio":{"rationale":"runtime"},"serde":{"rationale":"serialization"}}}
"#,
    )
    .expect("blessed allowlist written");
}

fn dependency_blessed_allowlist_args(root: &Path, report: &Path, enforce: bool) -> Vec<String> {
    let mut args = vec![
        "gate".to_string(),
        "validate".to_string(),
        "dependency-blessed-allowlist".to_string(),
        "--repo-root".to_string(),
        root.to_str().expect("utf8 root").to_string(),
        "--emit-report".to_string(),
        report.to_str().expect("utf8 report").to_string(),
    ];
    if enforce {
        args.push("--enforce".to_string());
    } else {
        args.push("--report-only".to_string());
    }
    args
}

fn dependency_seam_valid_evidence() -> &'static str {
    r#"{
  "change_id": "dependency-seam-test",
  "change_class_id": "CC-7",
  "git_sha": "abcdef1",
  "freshness_unix": 1700000000,
  "facets": {
    "F1_linus": {"considered": true, "rigor": "scan"},
    "F2_hyperscaler": {"considered": true, "rigor": "scan"},
    "F3_adversarial": {"considered": true, "rigor": "deep"},
    "F4_ergonomic": {"considered": true, "rigor": "scan"},
    "F5_quality": {"considered": true, "rigor": "scan"},
    "F6_alternatives": {"considered": true, "rigor": "scan"},
    "F7_security": {"considered": true, "rigor": "scan"},
    "F8_performance": {"considered": true},
    "F9_compliance": {"considered": true}
  }
}"#
}

fn write_quality_lanes_fixture(root: &Path, check_command: &str, markdown_purpose: &str) {
    fs::create_dir_all(root.join("registry/quality")).expect("quality registry dir created");
    fs::create_dir_all(root.join("docs/standards")).expect("standards dir created");
    fs::create_dir_all(root.join("docs/teams/axis-foundry")).expect("team charter dir created");
    fs::write(
        root.join("registry/quality/lanes.yaml"),
        format!(
            "lanes:\n  - id: cargo-fmt\n    stage: per-pr\n    status: active\n    owner_team: axis-foundry\n    purpose: `cargo fmt --all -- --check`\n    source: TOOLCHAIN.md\n    runtime_budget_seconds: 300\n    check_command: {check_command}\n"
        ),
    )
    .expect("quality lane registry written");
    fs::write(
        root.join("docs/teams/axis-foundry/CHARTER.md"),
        "# Team: Axis Foundry\n",
    )
    .expect("team charter written");
    fs::write(
        root.join("docs/standards/ci-lanes.md"),
        format!(
            "# CI\n\n### 1.2 Per-PR gates\n\n| Lane | Purpose |\n|---|---|\n| `cargo-fmt` | {markdown_purpose} |\n"
        ),
    )
    .expect("quality lane doc written");
    fs::write(
        root.join("check.sh"),
        format!("#!/usr/bin/env bash\n{check_command}\n"),
    )
    .expect("check script written");
}

fn quality_lanes_args(root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "quality-lanes".into(),
        "--registry".into(),
        root.join("registry/quality/lanes.yaml")
            .to_str()
            .expect("utf8 quality registry")
            .into(),
        "--ci-lanes".into(),
        root.join("docs/standards/ci-lanes.md")
            .to_str()
            .expect("utf8 quality docs")
            .into(),
        "--check-script".into(),
        root.join("check.sh")
            .to_str()
            .expect("utf8 check script")
            .into(),
        "--teams-dir".into(),
        root.join("docs/teams")
            .to_str()
            .expect("utf8 teams dir")
            .into(),
    ]
}

fn workspace_hygiene_fixture(scan_root: &str) -> String {
    format!(
        r#"{{
  "schema_version": "1.0.0",
  "id": "workspace-hygiene",
  "purpose": "test policy",
  "gate": {{
    "command": "oya gate validate workspace-hygiene",
    "side_effect_policy": "inventory_by_default_cleanup_requires_explicit_flag"
  }},
  "required_scan_surfaces": ["tmp", "home", "repo", "build-artifacts", "oyatie-worktrees"],
  "scan_surfaces": [
    {{
      "id": "tmp",
      "roots": ["{scan_root}"],
      "missing_ok": false,
      "max_depth": 1,
      "match_globs": ["unused-temp-pattern", "*pipeline*"],
      "audit_finding_budget": 10,
      "strict_finding_budget": 0,
      "cleanup_globs": ["unused-temp-pattern", "*pipeline*"],
      "action": "cleanable_temp_artifacts"
    }},
    {{
      "id": "home",
      "roots": ["{scan_root}"],
      "missing_ok": false,
      "max_depth": 1,
      "match_globs": ["unused-home-pattern"],
      "exempt_globs": ["oyatie"],
      "exemption_evidence_refs": ["docs/AGENTS.md#project-doc"],
      "audit_finding_budget": 10,
      "strict_finding_budget": 0,
      "action": "inventory_only"
    }},
    {{
      "id": "repo",
      "roots": ["{scan_root}"],
      "missing_ok": false,
      "max_depth": 1,
      "match_globs": ["unused-repo-pattern"],
      "audit_finding_budget": 10,
      "strict_finding_budget": 0,
      "action": "inventory_only"
    }},
    {{
      "id": "build-artifacts",
      "roots": ["{scan_root}"],
      "missing_ok": false,
      "max_depth": 2,
      "match_globs": ["target", "target-*", "dist", "build", ".next", ".turbo", ".vite", "coverage", "lcov.info", "*.profraw", "*.profdata"],
      "cleanup_globs": ["target", "target-*", "dist", ".next", ".turbo", ".vite", "coverage", "lcov.info", "*.profraw", "*.profdata"],
      "audit_finding_budget": 10,
      "strict_finding_budget": 0,
      "action": "cleanable_build_artifacts"
    }},
    {{
      "id": "oyatie-worktrees",
      "roots": ["{scan_root}"],
      "missing_ok": false,
      "max_depth": 1,
      "match_globs": ["unused-worktree-pattern"],
      "exempt_globs": ["*"],
      "exemption_evidence_refs": ["docs/AGENTS.md#scaffold_protocol"],
      "audit_finding_budget": 10,
      "strict_finding_budget": 0,
      "action": "inventory_only"
    }}
  ],
  "pipeline_contract": {{
    "required_phases": ["session-start", "pre-pr", "post-merge", "session-close"],
    "minimum_actions": [
      "inventory_all_required_scan_surfaces",
      "classify_findings_by_hygiene_class",
      "classify_build_artifacts_by_cleanup_or_exemption",
      "clean_configured_build_artifacts_with_explicit_cleanup_flag",
      "clean_configured_temp_artifacts_with_explicit_cleanup_flag",
      "classify_owned_roots_by_exemption_evidence",
      "link_each_keep_item_to_owner_or_evidence",
      "strict_mode_zero_untriaged_findings_before_release_or_hyperscaler_claim"
    ]
  }}
}}"#
    )
}

fn write_vendor_contract_ledger(root: &Path, rows: &str) -> std::path::PathBuf {
    fs::create_dir_all(root).expect("vendor ledger dir created");
    let ledger = root.join("VENDOR-PARTNER-LEDGER.md");
    fs::write(
        &ledger,
        format!(
            "# Vendor ledger\n\n## Contract recency ledger\n\n| Contract ID | Vendor / partner | Status | Expiry date | Renewal task | Owner |\n|---|---|---|---|---|---|\n{rows}"
        ),
    )
    .expect("vendor ledger written");
    ledger
}

fn vendor_contract_recency_args(ledger: &Path, today: &str) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "vendor-contract-recency".into(),
        "--ledger".into(),
        ledger.to_str().expect("utf8 vendor ledger").into(),
        "--today".into(),
        today.into(),
    ]
}

fn write_mobile_native_manifest(root: &Path, rows: &str) -> std::path::PathBuf {
    fs::create_dir_all(root).expect("mobile native manifest dir created");
    let manifest = root.join("mobile-native-products.tsv");
    fs::write(
        &manifest,
        format!(
            "# current_wave: W-Foundry-Preview\n# empty_scope_rationale: ADR-0051 keeps native out of scope before W-Workspace-Stable.\nproduct_id\taxis\tstatus\tcanonical_web_reference\ttarget_matrix_ref\ttech_stack_rationale_ref\tstore_policy_ref\tstore_policy_validator_passed\taccessibility_audit_ref\taccessibility_audit_passed\tcapability_parity_ref\tcapability_parity_passed\tsbom_ref\tnative_binary_blobs_without_sbom\tcrash_free_sessions_bps\tcrash_free_regression_bps\tcold_start_p99_ms\n{rows}"
        ),
    )
    .expect("mobile native manifest written");
    manifest
}

fn mobile_native_args(manifest: &Path, repo_root: &Path) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "mobile-native".into(),
        "--manifest".into(),
        manifest.to_str().expect("utf8 mobile manifest").into(),
        "--repo-root".into(),
        repo_root.to_str().expect("utf8 repo root").into(),
    ]
}

fn write_typescript_workspace_fixture(
    root: &Path,
    package_json: &str,
    write_lockfile: bool,
    write_tsconfig: bool,
) {
    fs::create_dir_all(root.join("src")).expect("typescript src dir created");
    fs::write(root.join("package.json"), package_json).expect("package.json written");
    fs::write(
        root.join("src/index.ts"),
        "export const ok: boolean = true;\n",
    )
    .expect("typescript marker written");
    if write_lockfile {
        fs::write(root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n")
            .expect("pnpm lockfile written");
    }
    if write_tsconfig {
        fs::write(root.join("tsconfig.json"), "{\"compilerOptions\": {}}\n")
            .expect("tsconfig written");
    }
}

fn typescript_workspace_args(root: &Path, lane: &str) -> Vec<String> {
    vec![
        "gate".into(),
        "validate".into(),
        "typescript-workspace".into(),
        "--repo-root".into(),
        root.to_str().expect("utf8 repo root").into(),
        "--lane".into(),
        lane.into(),
    ]
}

fn write_doc_catalog_fixture(docs_dir: &Path, root_docs: &[&str], catalog_docs: &[&str]) {
    fs::create_dir_all(docs_dir.join("machine-readable")).expect("machine-readable dir created");
    for doc in root_docs {
        fs::write(docs_dir.join(doc), format!("# {doc}\n")).expect("root doc written");
    }
    let mut markdown = "# Doc Catalog\n\n| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |\n|---|---|---|---|---|---|---|---|\n".to_string();
    for doc in catalog_docs {
        let doc_id = doc
            .trim_end_matches(".md")
            .to_ascii_lowercase()
            .replace('-', "_");
        markdown.push_str(&format!(
            "| `doc.{doc_id}` | `{doc}` | `council-architecture` | event | monthly | (none) | `doc-catalog-self-coverage` | YES |\n"
        ));
    }
    fs::write(docs_dir.join("DOC-CATALOG.md"), markdown).expect("doc catalog written");

    let entries = catalog_docs
        .iter()
        .map(|doc| {
            let doc_id = doc
                .trim_end_matches(".md")
                .to_ascii_lowercase()
                .replace('-', "_");
            format!(
                r#""doc.{doc_id}": {{
      "path": "docs/{doc}",
      "owner_team": "council-architecture",
      "dependent_docs": [],
      "validation_check": "doc-catalog-self-coverage"
    }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(
        docs_dir.join("machine-readable/catalog.json"),
        format!("{{\n  \"docs\": {{\n{entries}\n  }}\n}}\n"),
    )
    .expect("machine catalog written");
}

#[test]
fn product_index_gate_is_dispatched() {
    let temp = temp_dir("product-index-dispatch");
    let products_dir = temp.join("docs/products");
    let machine_dir = temp.join("docs/machine-readable");
    fs::create_dir_all(&products_dir).expect("products dir created");
    fs::create_dir_all(&machine_dir).expect("machine dir created");
    let readme = "intro\n\n### Axis products (7)\n\n| Product | PRD |\n|---|---|\n| SaaS Platform | saas-platform/PRD.md |\n| Workspace | workspace/PRD.md |\n| Foundry | foundry/PRD.md |\n| Cloud Provider | cloud/PRD.md |\n| Search | search/PRD.md |\n| Ads + Analytics | ads-analytics/PRD.md |\n| Vertical Industry Cloud | n/a |\n\n### Vertical products\n\nlater\n";
    fs::write(products_dir.join("README.md"), readme).expect("readme written");
    for product in [
        "saas-platform",
        "workspace",
        "foundry",
        "cloud",
        "search",
        "ads-analytics",
    ] {
        let dir = products_dir.join(product);
        fs::create_dir_all(&dir).expect("product dir created");
        fs::write(dir.join("PRD.md"), "# PRD\n").expect("prd written");
    }
    fs::write(
        machine_dir.join("catalog.json"),
        r#"{"products":{"saas-platform":{"prd_path":"docs/products/saas-platform/PRD.md"},"workspace":{"prd_path":"docs/products/workspace/PRD.md"},"foundry":{"prd_path":"docs/products/foundry/PRD.md"},"cloud":{"prd_path":"docs/products/cloud/PRD.md"},"search":{"prd_path":"docs/products/search/PRD.md"},"ads-analytics":{"prd_path":"docs/products/ads-analytics/PRD.md"}}}"#,
    )
    .expect("catalog written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "product-index",
            "--products-readme",
            products_dir
                .join("README.md")
                .to_str()
                .expect("utf8 readme"),
            "--catalog",
            machine_dir
                .join("catalog.json")
                .to_str()
                .expect("utf8 catalog"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("product-index validation passed"));
    fs::remove_dir_all(temp).ok();
}

#[test]
fn master_plan_completion_gate_is_dispatched() {
    let temp = temp_dir("master-plan-completion-dispatch");
    let specs_dir = temp.join("specs");
    let evidence_dir = temp.join("evidence/foundation");
    fs::create_dir_all(&specs_dir).expect("specs dir created");
    fs::create_dir_all(&evidence_dir).expect("evidence dir created");
    fs::write(
        specs_dir.join("masterplan.json"),
        r#"{
  "live_implementation_index": {
    "milestones": [
      {
        "phases": [
          {
            "id": "P-X",
            "status": "complete",
            "implementation_plans": [
              {"id": "IP-001", "status": "complete"}
            ]
          }
        ]
      }
    ]
  }
}"#,
    )
    .expect("masterplan written");
    fs::write(evidence_dir.join("ip-001.json"), r#"{"ip":"IP-001"}"#).expect("evidence written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "master-plan-completion",
            "--master-plan",
            specs_dir
                .join("masterplan.json")
                .to_str()
                .expect("utf8 masterplan"),
            "--evidence-dir",
            evidence_dir.to_str().expect("utf8 evidence"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("master-plan-completion validation passed")
    );
    fs::remove_dir_all(temp).ok();
}

#[test]
fn board_masterplan_consistency_gate_is_dispatched() {
    let temp = temp_dir("board-masterplan-consistency-dispatch");
    let specs_dir = temp.join("specs");
    let evidence_dir = temp.join("evidence/board-sync");
    fs::create_dir_all(&specs_dir).expect("specs dir created");
    fs::create_dir_all(&evidence_dir).expect("board evidence dir created");
    fs::write(
        specs_dir.join("masterplan.json"),
        include_str!("fixtures/board-sync/masterplan-minimal.json"),
    )
    .expect("masterplan written");
    fs::write(
        evidence_dir.join("board-snapshot.json"),
        include_str!("fixtures/board-sync/board-snapshot-matching.json"),
    )
    .expect("board snapshot written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "board-masterplan-consistency",
            "--master-plan",
            specs_dir
                .join("masterplan.json")
                .to_str()
                .expect("utf8 masterplan"),
            "--board-snapshot",
            evidence_dir
                .join("board-snapshot.json")
                .to_str()
                .expect("utf8 snapshot"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("board-masterplan-consistency validation passed: 2 masterplan deliverables, 2 board deliverables")
    );
    fs::remove_dir_all(temp).ok();
}

#[test]
fn board_masterplan_consistency_gate_rejects_forward_orphans() {
    let temp = temp_dir("board-masterplan-consistency-forward-orphan");
    let specs_dir = temp.join("specs");
    let evidence_dir = temp.join("evidence/board-sync");
    fs::create_dir_all(&specs_dir).expect("specs dir created");
    fs::create_dir_all(&evidence_dir).expect("board evidence dir created");
    fs::write(
        specs_dir.join("masterplan.json"),
        include_str!("fixtures/board-sync/masterplan-minimal.json"),
    )
    .expect("masterplan written");
    fs::write(
        evidence_dir.join("board-snapshot.json"),
        include_str!("fixtures/board-sync/board-snapshot-forward-orphan.json"),
    )
    .expect("board snapshot written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "board-masterplan-consistency",
            "--master-plan",
            specs_dir
                .join("masterplan.json")
                .to_str()
                .expect("utf8 masterplan"),
            "--board-snapshot",
            evidence_dir
                .join("board-snapshot.json")
                .to_str()
                .expect("utf8 snapshot"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("masterplan deliverables missing from board snapshot"));
    assert!(stderr.contains("IP-002"));
    fs::remove_dir_all(temp).ok();
}

#[test]
fn board_masterplan_consistency_gate_rejects_reverse_orphans() {
    let temp = temp_dir("board-masterplan-consistency-reverse-orphan");
    let specs_dir = temp.join("specs");
    let evidence_dir = temp.join("evidence/board-sync");
    fs::create_dir_all(&specs_dir).expect("specs dir created");
    fs::create_dir_all(&evidence_dir).expect("board evidence dir created");
    fs::write(
        specs_dir.join("masterplan.json"),
        include_str!("fixtures/board-sync/masterplan-minimal.json"),
    )
    .expect("masterplan written");
    fs::write(
        evidence_dir.join("board-snapshot.json"),
        include_str!("fixtures/board-sync/board-snapshot-reverse-orphan.json"),
    )
    .expect("board snapshot written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "board-masterplan-consistency",
            "--master-plan",
            specs_dir
                .join("masterplan.json")
                .to_str()
                .expect("utf8 masterplan"),
            "--board-snapshot",
            evidence_dir
                .join("board-snapshot.json")
                .to_str()
                .expect("utf8 snapshot"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("board snapshot deliverables missing from masterplan"));
    assert!(stderr.contains("IP-999"));
    fs::remove_dir_all(temp).ok();
}

#[test]
fn stage0_prereqs_gate_is_dispatched() {
    let temp = temp_dir("stage0-prereqs-dispatch");
    fs::create_dir_all(temp.join("crates/oya-application-app/src")).expect("app dir created");
    fs::create_dir_all(temp.join("docs/decisions")).expect("decisions dir created");
    fs::write(
        temp.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/oya-application-app\"]\nresolver = \"2\"\n\n[workspace.package]\nedition = \"2024\"\nversion = \"0.1.0\"\nrust-version = \"1.97.1\"\n",
    )
    .expect("workspace written");
    fs::write(
        temp.join("crates/oya-application-app/Cargo.toml"),
        "[package]\nname = \"oya-application-app\"\nedition.workspace = true\nversion.workspace = true\nrust-version.workspace = true\npublish = false\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .expect("app manifest written");
    fs::write(
        temp.join("crates/oya-application-app/src/lib.rs"),
        "pub fn smoke() -> bool { true }\n",
    )
    .expect("app lib written");
    fs::write(
        temp.join("docs/decisions/ADR-0709-general-live-apex.md"),
        "# ADR-0061\n",
    )
    .expect("adr written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "stage0-prereqs",
            "--repo-root",
            temp.to_str().expect("utf8 temp"),
            "--self-test",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("stage0-prereqs validation passed"));
    fs::remove_dir_all(temp).ok();
}

// ─── statelessness integration tests (M02b/P22 exit-gate lane 1) ────────────

/// Proves the statelessness validator catches a `static mut` in an app-layer
/// crate — the canonical synthetic-violation shape for ADR-0062 §"sharded state".
#[test]
fn statelessness_gate_catches_static_mut_violation_in_app_crate() {
    let temp = temp_dir("stateless-violation");
    let src_dir = temp.join("crates/oya-foo-app/src");
    fs::create_dir_all(&src_dir).expect("app src dir created");
    fs::write(
        src_dir.join("lib.rs"),
        "// intentional violation for test\nstatic mut COUNTER: usize = 0;\npub fn inc() { unsafe { COUNTER += 1; } }\n",
    )
    .expect("violation source written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "statelessness",
            "--workspace-root",
            temp.to_str().expect("utf8 temp"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for static mut in app layer\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("statelessness validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("static mut"),
        "stderr must name the violation kind; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the statelessness validator passes when an app-layer crate contains
/// no mutable globals — clean path smoke-test.
#[test]
fn statelessness_gate_passes_clean_app_crate() {
    let temp = temp_dir("stateless-clean");
    let src_dir = temp.join("crates/oya-bar-app/src");
    fs::create_dir_all(&src_dir).expect("app src dir created");
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn add(a: u32, b: u32) -> u32 { a + b }\n",
    )
    .expect("clean source written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "statelessness",
            "--workspace-root",
            temp.to_str().expect("utf8 temp"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected pass for clean app layer\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("statelessness validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

// ─── shardability integration tests (M02b/P22 exit-gate lane 2) ─────────────

/// Proves the shardability validator catches a `CREATE TABLE` that is missing
/// `tenant_id` — the canonical synthetic-violation shape for ADR-0062 §"sharded
/// state" (non-shard-keyed table, Postgres+Citus row-level-security requirement).
#[test]
fn shardability_gate_catches_missing_tenant_id_in_app_crate() {
    let temp = temp_dir("shard-violation");
    let migrations_dir = temp.join("migrations");
    fs::create_dir_all(&migrations_dir).expect("migrations dir created");
    fs::write(
        migrations_dir.join("001_orders.sql"),
        // intentional violation: no tenant_id column, no global opt-out marker
        "CREATE TABLE orders (\n  id UUID PRIMARY KEY,\n  amount BIGINT NOT NULL\n);\n",
    )
    .expect("violation migration written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "shardability",
            "--migrations-dir",
            migrations_dir.to_str().expect("utf8 migrations dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for missing tenant_id\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("shardability validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("tenant_id"),
        "stderr must name the violation kind; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the shardability validator passes when all tables declare `tenant_id`
/// — clean path smoke-test.
#[test]
fn shardability_gate_passes_clean_app_crate() {
    let temp = temp_dir("shard-clean");
    let migrations_dir = temp.join("migrations");
    fs::create_dir_all(&migrations_dir).expect("migrations dir created");
    fs::write(
        migrations_dir.join("001_events.sql"),
        "CREATE TABLE events (\n  id UUID PRIMARY KEY,\n  tenant_id UUID NOT NULL,\n  payload JSONB\n);\n",
    )
    .expect("clean migration written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "shardability",
            "--migrations-dir",
            migrations_dir.to_str().expect("utf8 migrations dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected pass for tenant_id-keyed table\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("shardability validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the protection-context-match validator catches the canonical
/// silent-bypass class: branch-protection lists a context name that no
/// workflow job posts (data-class label mismatch between the protection
/// config and the actual workflow `name:` fields). Gate must exit 1.
#[test]
fn protection_context_match_gate_catches_data_class_label_mismatch() {
    let temp = temp_dir("pcm-mismatch");
    let workflows_dir = temp.join("workflows");
    fs::create_dir_all(&workflows_dir).expect("workflows dir created");

    // branch-protection lists `oya-governance-protection-context-match`
    // but the workflow posts `pcm-check` (different label) — silent bypass.
    let branch_protection = "branches:\n  dev:\n    require_pull_request: true\n    \
                             required_status_checks:\n      \
                             - oya-governance-protection-context-match\n    \
                             require_signed_commits: true\n";
    let protection_file = temp.join("branch-protection.yaml");
    fs::write(&protection_file, branch_protection).expect("branch-protection written");

    let workflow_yaml = "name: pr-tests\non:\n  pull_request:\njobs:\n  pcm-check:\n    \
                         name: pcm-check\n    runs-on: ubuntu-latest\n    steps:\n      \
                         - run: echo hi\n";
    fs::write(workflows_dir.join("pr-tests.yml"), workflow_yaml).expect("workflow written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "protection-context-match",
            "--branch-protection",
            protection_file.to_str().expect("utf8 protection path"),
            "--workflows-dir",
            workflows_dir.to_str().expect("utf8 workflows dir"),
            "--branch",
            "dev",
            "--skip-applied-branch-protection",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected exit 1 for mismatched label\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("protection-context-match validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("oya-governance-protection-context-match"),
        "stderr must name the missing context; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the protection-context-match validator passes when every required
/// context in branch-protection is posted by a workflow job — clean path.
#[test]
fn protection_context_match_gate_passes_clean_crate() {
    let temp = temp_dir("pcm-clean");
    let workflows_dir = temp.join("workflows");
    fs::create_dir_all(&workflows_dir).expect("workflows dir created");

    let branch_protection = "branches:\n  dev:\n    require_pull_request: true\n    \
                             required_status_checks:\n      - cargo-fmt\n      \
                             - oya-governance-protection-context-match\n    \
                             require_signed_commits: true\n";
    let protection_file = temp.join("branch-protection.yaml");
    fs::write(&protection_file, branch_protection).expect("branch-protection written");

    let workflow_yaml = "name: pr-tests\non:\n  pull_request:\njobs:\n  fmt:\n    \
                         name: cargo-fmt\n    runs-on: ubuntu-latest\n    steps:\n      \
                         - run: cargo fmt --check\n  pcm:\n    \
                         name: oya-governance-protection-context-match\n    \
                         runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n";
    fs::write(workflows_dir.join("pr-tests.yml"), workflow_yaml).expect("workflow written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "protection-context-match",
            "--branch-protection",
            protection_file.to_str().expect("utf8 protection path"),
            "--workflows-dir",
            workflows_dir.to_str().expect("utf8 workflows dir"),
            "--branch",
            "dev",
            "--skip-applied-branch-protection",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected exit 0 for aligned contexts\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("protection-context-match validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the protection-context-match validator also catches the
/// hosted false-green class: local branch-protection + workflow job
/// names agree, but live GitHub branch protection requires fewer
/// contexts than the repo policy.
#[test]
fn protection_context_match_gate_catches_live_branch_protection_drift() {
    let temp = temp_dir("pcm-live-drift");
    let workflows_dir = temp.join("workflows");
    fs::create_dir_all(&workflows_dir).expect("workflows dir created");

    let branch_protection = "branches:\n  dev:\n    require_pull_request: true\n    \
                             required_status_checks:\n      - cargo-fmt\n      \
                             - oya-pr-review\n    require_signed_commits: true\n";
    let protection_file = temp.join("branch-protection.yaml");
    fs::write(&protection_file, branch_protection).expect("branch-protection written");

    let workflow_yaml = "name: pr-tests\non:\n  pull_request:\njobs:\n  fmt:\n    \
                         name: cargo-fmt\n    runs-on: ubuntu-latest\n    steps:\n      \
                         - run: cargo fmt --check\n  review:\n    \
                         name: oya-pr-review\n    runs-on: ubuntu-latest\n    steps:\n      \
                         - run: echo ok\n";
    fs::write(workflows_dir.join("pr-tests.yml"), workflow_yaml).expect("workflow written");
    let live_required_contexts = temp.join("live-required-contexts.json");
    fs::write(&live_required_contexts, r#"{"contexts":["cargo-fmt"]}"#)
        .expect("live contexts written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "protection-context-match",
            "--branch-protection",
            protection_file.to_str().expect("utf8 protection path"),
            "--workflows-dir",
            workflows_dir.to_str().expect("utf8 workflows dir"),
            "--branch",
            "dev",
            "--skip-applied-branch-protection",
            "--live-required-contexts",
            live_required_contexts
                .to_str()
                .expect("utf8 live contexts path"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected exit 1 for live branch-protection drift\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("live branch protection required_status_checks"),
        "stderr must name live drift; got: {stderr}"
    );
    assert!(
        stderr.contains("missing from live branch protection: oya-pr-review"),
        "stderr must name the unenforced context; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the protection-context-match validator catches the local
/// false-green class where the canonical YAML and workflow job names agree,
/// but the JSON config used by branch-protection apply still carries a stale
/// required context.
#[test]
fn protection_context_match_gate_catches_applied_branch_protection_drift() {
    let temp = temp_dir("pcm-applied-drift");
    let workflows_dir = temp.join("workflows");
    fs::create_dir_all(&workflows_dir).expect("workflows dir created");

    let branch_protection = "branches:\n  dev:\n    require_pull_request: true\n    \
                             required_status_checks:\n      - cargo-fmt\n      \
                             - oya-governance-protection-context-match\n    \
                             require_signed_commits: true\n";
    let protection_file = temp.join("branch-protection.yaml");
    fs::write(&protection_file, branch_protection).expect("branch-protection written");

    let workflow_yaml = "name: pr-tests\non:\n  pull_request:\njobs:\n  fmt:\n    \
                         name: cargo-fmt\n    runs-on: ubuntu-latest\n    steps:\n      \
                         - run: cargo fmt --check\n  pcm:\n    \
                         name: oya-governance-protection-context-match\n    \
                         runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n";
    fs::write(workflows_dir.join("pr-tests.yml"), workflow_yaml).expect("workflow written");

    let applied_config = temp.join("dev.json");
    fs::write(
        &applied_config,
        r#"{"required_status_checks":{"contexts":["cargo-fmt","stale-applied-check"]}}"#,
    )
    .expect("applied config written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "protection-context-match",
            "--branch-protection",
            protection_file.to_str().expect("utf8 protection path"),
            "--workflows-dir",
            workflows_dir.to_str().expect("utf8 workflows dir"),
            "--branch",
            "dev",
            "--applied-branch-protection",
            applied_config.to_str().expect("utf8 applied config path"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected exit 1 for applied branch-protection drift\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("applied branch-protection config required_status_checks"),
        "stderr must name applied drift; got: {stderr}"
    );
    assert!(
        stderr.contains(
            "missing from applied branch-protection config: oya-governance-protection-context-match"
        ),
        "stderr must name the missing new context; got: {stderr}"
    );
    assert!(
        stderr.contains("extra in applied branch-protection config: stale-applied-check"),
        "stderr must name the stale old context; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn canonical_base_neutrality_self_test_passes() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "canonical-base-neutrality",
            "--self-test",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected self-test to pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("canonical-base-neutrality validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn canonical_base_neutrality_catches_identifier_and_string_leaks() {
    let temp = temp_dir("canonical-base-neutrality-dirty");
    let src_dir = temp.join("crates/oya-cloud-data-kernel/src");
    fs::create_dir_all(&src_dir).expect("fixture src dir created");
    fs::write(
        src_dir.join("lib.rs"),
        r#"
pub struct FinancialKrCredit;
pub enum ResidencyClass { StrictKr }
pub struct LeastUsed;
pub struct KmsUseReceipt;
pub struct InvalidUserDataUri;
pub const LOCALE: &str = "ko-KR";
"#,
    )
    .expect("fixture source written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "canonical-base-neutrality",
            "--repo-root",
            temp.to_str().expect("utf8 temp path"),
            "--root",
            "crates/oya-cloud-data-kernel/src",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected jurisdiction leaks to fail\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("FinancialKrCredit"),
        "stderr must name identifier leak; got: {stderr}"
    );
    assert!(
        stderr.contains("StrictKr"),
        "stderr must name second identifier leak; got: {stderr}"
    );
    assert!(
        stderr.contains("ko-KR"),
        "stderr must name string leak; got: {stderr}"
    );
    assert!(
        !stderr.contains("LeastUsed")
            && !stderr.contains("KmsUseReceipt")
            && !stderr.contains("InvalidUserDataUri"),
        "stderr must avoid common Us false positives; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

fn write_catalog_record(registry_dir: &Path, crate_id: &str, plane: &str) {
    write_catalog_record_with_claim(registry_dir, crate_id, plane, "preview");
}

fn write_catalog_record_with_claim(
    registry_dir: &Path,
    crate_id: &str,
    plane: &str,
    api_stability: &str,
) {
    fs::create_dir_all(registry_dir).expect("registry dir created");
    fs::write(
        registry_dir.join(format!("{crate_id}.yaml")),
        format!(
            "context: foundry\nrole: kernel\ncapability: capability\nplane: {plane}\ndata_classes_owned: [INTERNAL_ONLY]\napi_stability: {api_stability}\nsecurity_review: unreviewed\nsupply_chain: source-only\n"
        ),
    )
    .expect("catalog record written");
}

// ─── perf-budget integration tests (M02b/P22 exit-gate lane 3) ──────────────

/// Proves the perf-budget validator catches an IP-*.md file that is missing a
/// `## Load test` section — the canonical synthetic-violation shape for the
/// perf-budget gate (ADR-0062 §"performance budgets").
#[test]
fn perf_budget_gate_catches_missing_load_test_section() {
    let temp = TempDirGuard::new("perf-budget-violation");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        plans_dir.join("IP-001-foo.md"),
        // intentional violation: no `## Load test` section
        "# IP-001 Foo\n\n## Summary\n\nImplement foo.\n\n## Acceptance criteria\n\n- Foo works.\n",
    )
    .expect("violation plan written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "perf-budget",
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for IP plan missing ## Load test section\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("perf-budget validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("Load test"),
        "stderr must name the violated section; got: {stderr}"
    );
}

/// Proves the perf-budget validator catches a present but empty `## Load test`
/// section instead of treating the heading alone as sufficient evidence.
#[test]
fn perf_budget_gate_catches_empty_load_test_section() {
    let temp = TempDirGuard::new("perf-budget-empty");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        plans_dir.join("IP-002-empty.md"),
        "# IP-002 Empty\n\n## Summary\n\nImplement empty.\n\n## Load test\n\n## Acceptance criteria\n\n- Done.\n",
    )
    .expect("empty-section plan written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "perf-budget",
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for empty ## Load test section\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty `## Load test` section"),
        "stderr must name empty section violation; got: {stderr}"
    );
}

/// Proves the perf-budget validator rejects digit-bearing filler that is not a
/// concrete load-test measurement.
#[test]
fn perf_budget_gate_catches_load_test_without_concrete_measurements() {
    let temp = TempDirGuard::new("perf-budget-no-measurement");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        plans_dir.join("IP-003-filler.md"),
        "# IP-003 Filler\n\n## Summary\n\nImplement filler.\n\n## Load test\n\n0 things to do before merge.\n",
    )
    .expect("filler plan written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "perf-budget",
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for non-measurement filler\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no concrete performance measurements"),
        "stderr must reject digit-only filler; got: {stderr}"
    );
}

/// Proves the perf-budget validator passes when an IP-*.md file contains a
/// `## Load test` section with concrete performance measurements.
#[test]
fn perf_budget_gate_passes_clean_crate() {
    let temp = TempDirGuard::new("perf-budget-clean");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        plans_dir.join("IP-004-bar.md"),
        // well-formed: ## Load test section present with concrete p99 number
        "# IP-004 Bar\n\n## Summary\n\nImplement bar.\n\n## Load test\n\nTarget: p99 < 50ms at 1000 rps sustained for 60s.\n",
    )
    .expect("clean plan written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "perf-budget",
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected pass for IP plan with ## Load test section and concrete measurements\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("perf-budget validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn benchmark_gate_catches_missing_competitive_benchmark_section() {
    let temp = temp_dir("benchmark-violation");
    let prds_dir = temp.join("prds");
    fs::create_dir_all(&prds_dir).expect("prds dir created");
    fs::write(
        prds_dir.join("PRD-missing-bench.md"),
        "# My Product\n\n## Overview\n\nThis product does things.\n\n## Load test\n\nP99 latency < 10ms at 10k rps.\n",
    )
    .expect("PRD written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "benchmark",
            "--prds-dir",
            prds_dir.to_str().expect("utf8 prds dir"),
            "--products-dir",
            temp.join("products").to_str().expect("utf8 products dir"),
        ])
        .output()
        .expect("benchmark gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("benchmark validation failed"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("SectionMissing")
            || String::from_utf8_lossy(&output.stderr)
                .contains("missing `## Competitive benchmark` section"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn benchmark_gate_passes_clean_crate() {
    let temp = temp_dir("benchmark-clean");
    let prds_dir = temp.join("prds");
    fs::create_dir_all(&prds_dir).expect("prds dir created");
    fs::write(
        prds_dir.join("PRD-workflow.md"),
        "# Workflow Studio\n\n## Overview\n\nDrag-and-drop automation engine.\n\n## Competitive benchmark\n\nStripe Sigma: 200ms P99 query latency at 50k tenants. Linear: sub-100ms issue create at 1M issues. n8n: 10k workflow executions/sec on a single node. Oyatie targets <80ms P99 at 100k tenants.\n",
    )
    .expect("PRD written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "benchmark",
            "--prds-dir",
            prds_dir.to_str().expect("utf8 prds dir"),
            "--products-dir",
            temp.join("products").to_str().expect("utf8 products dir"),
        ])
        .output()
        .expect("benchmark gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("benchmark validation passed: 1 PRDs checked"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

// ── audit-chain-replay gate (lane 9) ─────────────────────────────────────────

#[test]
fn audit_chain_replay_gate_accepts_clean_shard() {
    use audit_chain_domain::{AuditChain, Plane};
    use audit_file_adapter::FileAuditLedger;
    use oya_data_boundary_kernel::{DataClass, Purpose};

    let temp = temp_dir("acr-accept");
    fs::create_dir_all(&temp).expect("shards dir created");
    let shard_path = temp.join("tenant-alpha.log");

    let mut chain = AuditChain::default();
    chain
        .append_classifications(
            "ten_alpha",
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("fixture: append tenant.create");
    chain
        .append_classifications(
            "ten_alpha",
            "identity.user.upsert",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::PiiIdentifying],
            "ALLOW",
        )
        .expect("fixture: append identity.user.upsert");
    FileAuditLedger::new(shard_path)
        .append_chain(&chain)
        .expect("fixture: shard written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "audit-chain-replay",
            "--shards-dir",
            temp.to_str().expect("utf8 shards dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected clean shard to pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("audit chain replay validation passed: 1 shards, 2 events"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn audit_chain_replay_gate_rejects_tampered_hash() {
    use audit_chain_domain::{AuditChain, Plane};
    use audit_file_adapter::FileAuditLedger;
    use oya_data_boundary_kernel::{DataClass, Purpose};

    let temp = temp_dir("acr-tamper");
    fs::create_dir_all(&temp).expect("shards dir created");
    let shard_path = temp.join("tenant-beta.log");

    let mut chain = AuditChain::default();
    chain
        .append_classifications(
            "ten_beta",
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("fixture: append tenant.create");
    FileAuditLedger::new(shard_path.clone())
        .append_chain(&chain)
        .expect("fixture: shard written");

    // tamper: mutate the surface field so the hash no longer matches
    let raw = fs::read_to_string(&shard_path).expect("shard readable");
    let tampered = raw.replace("tenant.create", "tenant.delete");
    fs::write(&shard_path, tampered).expect("tampered shard written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "audit-chain-replay",
            "--shards-dir",
            temp.to_str().expect("utf8 shards dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected tampered shard to fail\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("audit chain replay validation failed"),
        "stderr must name failure; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

// ── lane-7: openapi-rest-route-parity ────────────────────────────────────────

/// Helper that writes a fake REST crate `src/lib.rs` under `crates_dir`.
/// The crate name must match `<prefix>*-rest`.
fn write_rest_crate(crates_dir: &Path, crate_name: &str, lib_rs_content: &str) {
    let src_dir = crates_dir.join(crate_name).join("src");
    fs::create_dir_all(&src_dir).expect("rest crate src dir created");
    fs::write(src_dir.join("lib.rs"), lib_rs_content).expect("rest lib.rs written");
}

/// Helper that writes a fake OpenAPI contract under `contracts_dir`.
fn write_openapi_contract(contracts_dir: &Path, filename: &str, content: &str) {
    fs::create_dir_all(contracts_dir).expect("contracts dir created");
    fs::write(contracts_dir.join(filename), content).expect("openapi contract written");
}

#[test]
fn openapi_rest_route_parity_gate_rejects_route_missing_from_openapi() {
    // Synthetic violation: REST crate declares /ops/api/v1/orders but the
    // OpenAPI contract only declares /ops/api/v1/health.  The gate must exit
    // non-zero and report the missing route.
    let temp = temp_dir("orp-reject-missing-openapi");
    let crates_dir = temp.join("crates");
    let contracts_dir = temp.join("contracts");

    write_rest_crate(
        &crates_dir,
        "oya-ops-orders-rest",
        "pub const HEALTH_ROUTE: &str = \"/ops/api/v1/health\";\npub const ORDERS_ROUTE: &str = \"/ops/api/v1/orders\";\n",
    );
    write_openapi_contract(
        &contracts_dir,
        "ops-orders.openapi.yaml",
        "openapi: 3.2.0\ninfo:\n  title: Orders\n  version: 1.0.0\npaths:\n  /ops/api/v1/health:\n    get: {}\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "openapi-rest-route-parity",
            "--crates-dir",
            crates_dir.to_str().expect("utf8 crates_dir"),
            "--contracts-dir",
            contracts_dir.to_str().expect("utf8 contracts_dir"),
            "--crate-prefix",
            "oya-ops-",
            "--contract-prefix",
            "ops-",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "gate must reject when route missing from openapi; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("openapi-rest-route-parity validation failed"),
        "stderr must contain failure message; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("/ops/api/v1/orders"),
        "stderr must name the offending route; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn openapi_rest_route_parity_gate_accepts_clean_matched_routes() {
    // Clean path: REST crate and OpenAPI contract declare the same single
    // route.  The gate must exit 0 and confirm counts.
    let temp = temp_dir("orp-accept-clean");
    let crates_dir = temp.join("crates");
    let contracts_dir = temp.join("contracts");

    write_rest_crate(
        &crates_dir,
        "oya-ops-health-rest",
        "pub const HEALTH_ROUTE: &str = \"/ops/api/v1/health\";\n",
    );
    write_openapi_contract(
        &contracts_dir,
        "ops-health.openapi.yaml",
        "openapi: 3.2.0\ninfo:\n  title: Health\n  version: 1.0.0\npaths:\n  /ops/api/v1/health:\n    get: {}\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "openapi-rest-route-parity",
            "--crates-dir",
            crates_dir.to_str().expect("utf8 crates_dir"),
            "--contracts-dir",
            contracts_dir.to_str().expect("utf8 contracts_dir"),
            "--crate-prefix",
            "oya-ops-",
            "--contract-prefix",
            "ops-",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "gate must accept clean matched routes; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("openapi-rest-route-parity validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn honest_claims_gate_accepts_clean_fixture_corpus_and_plan_graph() {
    let temp = TempDirGuard::new("honest-claims-clean");
    let docs_dir = temp.path().join("docs");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&docs_dir).expect("docs dir created");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        docs_dir.join("ADR-9000.md"),
        "claim_status: blocked_until_required_evidence_is_green\n\
         Active lane evidence references concrete validation output.\n",
    )
    .expect("doc written");
    write_honest_claims_plan(&plans_dir, "IP-001.md", "M01-P01-IP-001", "");
    write_honest_claims_plan(
        &plans_dir,
        "IP-002.md",
        "M01-P01-IP-002",
        "depends_on_changesets: [\"M01-P01-IP-001\"]\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "honest-claims",
            "--clear-default-corpus",
            "--corpus-root",
            docs_dir.to_str().expect("utf8 docs dir"),
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected honest claims gate to accept clean fixture\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("honest-claims validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn honest_claims_gate_rejects_deferred_required_claims() {
    let temp = TempDirGuard::new("honest-claims-deferred");
    let docs_dir = temp.path().join("docs");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&docs_dir).expect("docs dir created");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(
        docs_dir.join("ADR-9001.md"),
        "The required workflow lands in v2.\n",
    )
    .expect("doc written");
    write_honest_claims_plan(&plans_dir, "IP-001.md", "M01-P01-IP-001", "");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "honest-claims",
            "--clear-default-corpus",
            "--corpus-root",
            docs_dir.to_str().expect("utf8 docs dir"),
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("honest-claims violation"),
        "stderr must include honest-claims violation; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn honest_claims_gate_rejects_changeset_dependency_cycles() {
    let temp = TempDirGuard::new("honest-claims-cycle");
    let docs_dir = temp.path().join("docs");
    let plans_dir = temp.path().join("plans");
    fs::create_dir_all(&docs_dir).expect("docs dir created");
    fs::create_dir_all(&plans_dir).expect("plans dir created");
    fs::write(docs_dir.join("ADR-9002.md"), "Evidence is advisory.\n").expect("doc written");
    write_honest_claims_plan(
        &plans_dir,
        "IP-001.md",
        "M01-P01-IP-001",
        "depends_on_changesets: [\"M01-P01-IP-002\"]\n",
    );
    write_honest_claims_plan(
        &plans_dir,
        "IP-002.md",
        "M01-P01-IP-002",
        "depends_on_changesets: [\"M01-P01-IP-001\"]\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "honest-claims",
            "--clear-default-corpus",
            "--corpus-root",
            docs_dir.to_str().expect("utf8 docs dir"),
            "--plans-dir",
            plans_dir.to_str().expect("utf8 plans dir"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Cycle"),
        "stderr must include cycle violation; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_accepts_real_required_surfaces() {
    let temp = TempDirGuard::new("aspirational-clean");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9000.md"),
        "enforced_by: oya-check-real\nbranch protection required check: oya-governance-real\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected clean aspirational fixture to pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("aspirational-enforcement validation passed")
    );
}

/// The relocation this gate must survive: `libs/oya-check-<topic>` moves to
/// `governance/check/<topic>`, so both the catalog record stem and the corpus
/// spelling drop the `oya-` brand. Before the identity fix the tokenizer keyed
/// on `oya-check-` and matched NOTHING in that family, so a binding claim
/// against an ABSENT gate reported clean. It must still be caught.
#[test]
fn aspirational_enforcement_gate_catches_missing_gate_under_the_relocated_spelling() {
    let temp = TempDirGuard::new("aspirational-relocated");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9000.md"),
        "enforced_by: check-absent-gate\n",
    )
    .expect("doc written");

    let output = run_aspirational_gate(&fixture);

    assert!(
        !output.status.success(),
        "a binding claim on an unregistered check capability must fail even after the relocation\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("check-absent-gate"),
        "expected the relocated identity in the violation; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// STANDING RULE, surface side: zero measured check capabilities is a broken
/// scan, never dormant-and-passing.
#[test]
fn aspirational_enforcement_gate_rejects_an_emptied_capability_scan() {
    let temp = TempDirGuard::new("aspirational-empty-capabilities");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9000.md"),
        "enforced_by: check-real\n",
    )
    .expect("doc written");
    // Simulate the scan going blind: the records are there, none of them
    // resolve to a check capability any more.
    fs::write(
        fixture.catalog.join("governance-check-real.yaml"),
        "context: tooling\ncapability: renamed-away-real\n",
    )
    .expect("catalog record rewritten");

    let output = run_aspirational_gate(&fixture);

    assert!(
        !output.status.success(),
        "a gate observing zero check capabilities must be RED, not clean\nstdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ZERO check capabilities"),
        "expected the emptied-surface-scan diagnosis; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// STANDING RULE, corpus side: the surface scan can be healthy while the
/// document scan is the empty one.
#[test]
fn aspirational_enforcement_gate_rejects_an_emptied_corpus_site_scan() {
    let temp = TempDirGuard::new("aspirational-empty-sites");
    let fixture = write_aspirational_fixture(temp.path());
    fs::remove_file(fixture.docs.join("_baseline.md")).expect("baseline doc removed");
    fs::write(
        fixture.docs.join("ADR-9000.md"),
        "branch protection required check: oya-governance-real\n",
    )
    .expect("doc written");

    let output = run_aspirational_gate(&fixture);

    assert!(
        !output.status.success(),
        "a gate observing zero check sites must be RED, not clean\nstdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ZERO check-capability sites"),
        "expected the emptied-corpus-scan diagnosis; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_aspirational_gate(fixture: &AspirationalFixture) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs")
}

#[test]
fn banned_primitives_gate_rejects_retired_oya_git_surface() {
    let temp = TempDirGuard::new("banned-primitives-retired-oya-git");
    write_banned_primitives_fixture(
        temp.path(),
        "  - oya-git\nretirement_note: `oya git <git-subcommand>` is the retired git wrapper\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected retired oya git surface to fail\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("hard-banned primitive manual-mutation")
    );
}

#[test]
fn banned_primitives_gate_rejects_manual_push_inside_agent_fence() {
    let temp = TempDirGuard::new("banned-primitives-manual-push");
    write_banned_primitives_fixture(temp.path(), "run git push origin dev\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("manual-push"),
        "stderr must include manual-push violation; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_accepts_sanitized_command_log_corpus() {
    let temp = TempDirGuard::new("banned-primitives-command-log-clean");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "command-log.v1.jsonl",
        r#"{"record_id":"clean-001","origin":"session_tool_call_fixture","tool":"bin/oya","args":["git","status","--short"],"redacted":true,"expected":"allow"}
{"record_id":"clean-002","origin":"session_tool_call_fixture","tool":"oya","args":["vcs","verify","--agent","fd001"],"redacted":true,"expected":"allow"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected sanitized command-log corpus to pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("2 command-log records"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn banned_primitives_gate_requires_command_log_records_when_requested() {
    let temp = TempDirGuard::new("banned-primitives-command-log-required");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("command-log corpus required"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_direct_git_in_command_log_corpus() {
    let temp = TempDirGuard::new("banned-primitives-command-log-direct-git");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-direct-git-status.jsonl",
        r#"{"record_id":"reject-001","origin":"session_tool_call_fixture","tool":"git","args":["status","--short"],"redacted":true,"expected":"direct-vcs"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("forbidden command-log primitive direct-vcs"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_nested_shell_command_log_corpus() {
    let temp = TempDirGuard::new("banned-primitives-command-log-shell");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-nested-shell.jsonl",
        r#"{"record_id":"reject-002","origin":"session_tool_call_fixture","tool":"Bash","tool_input":{"command":"git push origin dev"},"redacted":true,"expected":"manual-push"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("forbidden command-log primitive manual-push"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_cmd_command_log_field() {
    let temp = TempDirGuard::new("banned-primitives-command-log-cmd-field");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-cmd-field.jsonl",
        r#"{"record_id":"reject-004","origin":"session_tool_call_fixture","tool":"Bash","cmd":"git push origin dev","redacted":true,"expected":"manual-push"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("forbidden command-log primitive manual-push"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_json_encoded_tool_arguments() {
    let temp = TempDirGuard::new("banned-primitives-command-log-json-arguments");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-json-arguments.jsonl",
        r#"{"record_id":"reject-005","origin":"session_tool_call_fixture","tool":"Bash","arguments":"{\"cmd\":\"gh pr merge 123\"}","redacted":true,"expected":"forge-merge"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("forbidden command-log primitive forge-merge"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_tool_only_command_log_records() {
    let temp = TempDirGuard::new("banned-primitives-command-log-tool-only");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-tool-only.jsonl",
        r#"{"record_id":"reject-006","origin":"session_tool_call_fixture","tool":"Bash","redacted":true,"expected":"reject-missing-command-surface"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("has no command or tool/args surface"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn banned_primitives_gate_rejects_unredacted_command_log_records() {
    let temp = TempDirGuard::new("banned-primitives-command-log-unredacted");
    write_banned_primitives_fixture(temp.path(), "  - oya-git\n");
    let corpus = temp
        .path()
        .join("registry/governance-corpora/banned-primitives");
    write_command_log_fixture(
        &corpus,
        "reject-unredacted.jsonl",
        r#"{"record_id":"reject-003","origin":"session_tool_call_fixture","tool":"oya","args":["git","status"],"redacted":false,"expected":"reject-unredacted"}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "banned-primitives",
            "--repo-root",
            temp.path().to_str().expect("utf8 temp root"),
            "--clear-default-roots",
            "--root",
            "AGENTS.md",
            "--root",
            "CLAUDE.md",
            "--root",
            "docs/AGENTS.md",
            "--require-command-log-corpus",
            "--command-log-root",
            "registry/governance-corpora/banned-primitives",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("redacted=true"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_missing_required_workflow() {
    let temp = TempDirGuard::new("aspirational-missing-workflow");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9001.md"),
        "required check: oya-governance-lane-only\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "stderr must include missing workflow violation; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_negated_advisory_required_claim() {
    let temp = TempDirGuard::new("aspirational-negated-advisory");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9008.md"),
        "required check: oya-governance-lane-only is active, not advisory\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "stderr must include negated-advisory missing workflow; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_multiline_enforced_by_claim() {
    let temp = TempDirGuard::new("aspirational-multiline");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9009.md"),
        "enforced_by:\n  - oya-check-missing\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingCrate"),
        "stderr must include multiline missing crate; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_unindented_multiline_enforced_by_claim() {
    let temp = TempDirGuard::new("aspirational-unindented-multiline");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9010.md"),
        "enforced_by:\n- oya-check-missing\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingCrate"),
        "stderr must include unindented multiline missing crate; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_unindented_multiline_required_check() {
    let temp = TempDirGuard::new("aspirational-unindented-required-check");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9011.md"),
        "required check:\n- oya-governance-lane-only\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "stderr must include unindented required-check missing workflow; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_unindented_multiline_required_status() {
    let temp = TempDirGuard::new("aspirational-unindented-required-status");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9012.md"),
        "required status:\n- oya-governance-real\n",
    )
    .expect("doc written");
    fs::write(
        &fixture.branch_protection,
        "branches:\n  dev:\n    required_status_checks:\n      - oya-governance-other\n",
    )
    .expect("branch protection written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingRequiredContext"),
        "stderr must include unindented required-status missing branch context; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_missing_branch_protection_file() {
    let temp = TempDirGuard::new("aspirational-missing-branch-protection");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9003.md"),
        "branch protection required check: oya-governance-real\n",
    )
    .expect("doc written");
    fs::remove_file(&fixture.branch_protection).expect("branch protection removed");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("branch-protection unreadable"),
        "stderr must include unreadable branch-protection failure; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_accepts_inline_comments_in_control_surfaces() {
    let temp = TempDirGuard::new("aspirational-inline-comments");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9013.md"),
        "branch protection required check: oya-governance-real\n",
    )
    .expect("doc written");
    fs::write(
        fixture.workflows.join("real-with-comments.yml"),
        "name: oya-governance-real # workflow context\njobs:\n  oya-governance-real: # job context\n    name: oya-governance-real # check name\n",
    )
    .expect("workflow written");
    fs::write(
        &fixture.quality_lanes,
        "lanes:\n  - id: oya-governance-real # lane id\n    status: active # current\n",
    )
    .expect("quality lanes written");
    fs::write(
        &fixture.branch_protection,
        "branches:\n  dev: # default branch\n    required_status_checks: # current required contexts\n      - oya-governance-real # required\n",
    )
    .expect("branch protection written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "inline comments in control surfaces should pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_required_context_on_wrong_branch() {
    let temp = TempDirGuard::new("aspirational-wrong-branch");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9006.md"),
        "branch protection required check: oya-governance-real\n",
    )
    .expect("doc written");
    fs::write(
        &fixture.branch_protection,
        "branches:\n  dev:\n    required_status_checks:\n      - oya-governance-other\n  staging:\n    required_status_checks:\n      - oya-governance-real\n",
    )
    .expect("branch protection written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingRequiredContext"),
        "stderr must include wrong-branch required-context failure; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_file_stem_only_workflow_stub() {
    let temp = TempDirGuard::new("aspirational-stem-only-workflow");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9014.md"),
        "enforced_by: oya-governance-stem-only\n",
    )
    .expect("doc written");
    fs::write(
        fixture.workflows.join("oya-governance-stem-only.yml"),
        "name: unrelated-workflow\njobs:\n  unrelated-job:\n    name: unrelated-job\n",
    )
    .expect("workflow written");
    fs::write(
        &fixture.quality_lanes,
        "lanes:\n  - id: oya-governance-stem-only\n    status: active\n",
    )
    .expect("quality lanes written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "workflow filename stem alone must not satisfy the context; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_metadata_key_only_workflow_stub() {
    let temp = TempDirGuard::new("aspirational-metadata-key-workflow");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9015.md"),
        "enforced_by: oya-governance-metadata-only\n",
    )
    .expect("doc written");
    fs::write(
        fixture.workflows.join("metadata-only.yml"),
        "name: unrelated-workflow\nmetadata:\n  oya-governance-metadata-only:\njobs:\n  unrelated-job:\n    name: unrelated-job\n",
    )
    .expect("workflow written");
    fs::write(
        &fixture.quality_lanes,
        "lanes:\n  - id: oya-governance-metadata-only\n    status: active\n",
    )
    .expect("quality lanes written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "metadata keys must not satisfy workflow context; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_step_name_only_workflow_stub() {
    let temp = TempDirGuard::new("aspirational-step-name-workflow");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9016.md"),
        "enforced_by: oya-governance-step-only\n",
    )
    .expect("doc written");
    fs::write(
        fixture.workflows.join("step-only.yml"),
        "name: unrelated-workflow\njobs:\n  unrelated-job:\n    steps:\n      - name: oya-governance-step-only\n        run: echo step-only\n",
    )
    .expect("workflow written");
    fs::write(
        &fixture.quality_lanes,
        "lanes:\n  - id: oya-governance-step-only\n    status: active\n",
    )
    .expect("quality lanes written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "step names must not satisfy workflow context; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_blocks_merge_without_required_context() {
    let temp = TempDirGuard::new("aspirational-blocks-merge");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9007.md"),
        "oya-governance-real blocks merge for active enforcement claims\n",
    )
    .expect("doc written");
    fs::write(
        &fixture.branch_protection,
        "branches:\n  dev:\n    required_status_checks:\n      - oya-governance-other\n",
    )
    .expect("branch protection written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingRequiredContext"),
        "stderr must include blocks-merge required-context failure; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_missing_corpus_root() {
    let temp = TempDirGuard::new("aspirational-missing-corpus");
    let fixture = write_aspirational_fixture(temp.path());
    let missing_root = temp.path().join("missing-docs");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            missing_root.to_str().expect("utf8 missing docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("corpus root unreadable"),
        "stderr must include unreadable corpus failure; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_allows_planned_missing_lanes() {
    let temp = TempDirGuard::new("aspirational-planned");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9002.md"),
        "candidate validator oya-governance-missing remains planned and advisory\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "planned/advisory missing lane must not fail\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_active_quality_lane_without_workflow() {
    let temp = TempDirGuard::new("aspirational-quality-lane");
    let fixture = write_aspirational_fixture(temp.path());
    fs::write(
        fixture.docs.join("ADR-9004.md"),
        "enforced_by: oya-governance-lane-only\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingWorkflow"),
        "active quality-lane without workflow must fail; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn aspirational_enforcement_gate_rejects_workflow_without_quality_lane() {
    let temp = TempDirGuard::new("aspirational-workflow-only");
    let fixture = write_aspirational_fixture(temp.path());
    // Declare oya-governance-workflow-only (so it is validated, not advisory per
    // ADR-0362(a)) but leave it status:proposed so it is NOT an active quality
    // lane -> the workflow-backed binding claim must flag MissingQualityLane.
    fs::write(
        &fixture.quality_lanes,
        "lanes:\n  - id: oya-governance-real\n    status: active\n  - id: oya-governance-workflow-only\n    status: proposed\n",
    )
    .expect("quality lanes rewritten");
    fs::write(
        fixture.workflows.join("oya-governance-workflow-only.yml"),
        "name: oya-governance-workflow-only\njobs:\n  oya-governance-workflow-only:\n    name: oya-governance-workflow-only\n",
    )
    .expect("workflow written");
    fs::write(
        fixture.docs.join("ADR-9005.md"),
        "enforced_by: oya-governance-workflow-only\n",
    )
    .expect("doc written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "aspirational-enforcement",
            "--clear-default-corpus",
            "--corpus-root",
            fixture.docs.to_str().expect("utf8 docs"),
            "--catalog-dir",
            fixture.catalog.to_str().expect("utf8 catalog"),
            "--workflows-dir",
            fixture.workflows.to_str().expect("utf8 workflows"),
            "--quality-lanes",
            fixture.quality_lanes.to_str().expect("utf8 quality lanes"),
            "--branch-protection",
            fixture
                .branch_protection
                .to_str()
                .expect("utf8 branch protection"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingQualityLane"),
        "workflow without active quality lane must fail; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

struct AspirationalFixture {
    docs: PathBuf,
    catalog: PathBuf,
    workflows: PathBuf,
    quality_lanes: PathBuf,
    branch_protection: PathBuf,
}

fn write_aspirational_fixture(root: &Path) -> AspirationalFixture {
    let docs = root.join("docs");
    let catalog = root.join("catalog");
    let workflows = root.join("workflows");
    fs::create_dir_all(&docs).expect("docs dir created");
    fs::create_dir_all(&catalog).expect("catalog dir created");
    // The check-gate identity is the catalog `capability:` facet. The record
    // STEM here is deliberately the RELOCATED path shape while the facet keeps
    // the identity, which is exactly the rename the tokenizer must survive.
    fs::write(
        catalog.join("governance-check-real.yaml"),
        "context: tooling\ncapability: check-real\n",
    )
    .expect("catalog record written");
    // Every aspirational test inherits one resolvable check site, so the
    // zero-site backstop stays armed instead of firing on narrow fixtures.
    fs::write(docs.join("_baseline.md"), "enforced_by: check-real\n")
        .expect("baseline doc written");
    fs::create_dir_all(&workflows).expect("workflows dir created");
    fs::write(
        workflows.join("oya-governance-real.yml"),
        "name: oya-governance-real\njobs:\n  oya-governance-real:\n    name: oya-governance-real\n",
    )
    .expect("workflow written");
    let quality_lanes = root.join("quality-lanes.yaml");
    fs::write(
        &quality_lanes,
        "lanes:\n  - id: oya-governance-real\n    status: active\n  - id: oya-governance-lane-only\n    status: active\n",
    )
    .expect("quality lanes written");
    let branch_protection = root.join("branch-protection.yaml");
    fs::write(
        &branch_protection,
        "branches:\n  dev:\n    required_status_checks:\n      - oya-governance-real\n",
    )
    .expect("branch protection written");
    AspirationalFixture {
        docs,
        catalog,
        workflows,
        quality_lanes,
        branch_protection,
    }
}

fn write_banned_primitives_fixture(root: &Path, root_agent_fence_body: &str) {
    fs::create_dir_all(root.join("docs")).expect("docs dir created");
    fs::write(
        root.join("AGENTS.md"),
        format!(
            "# Agent contract\n\n<!-- agent-instructions:start -->\n{root_agent_fence_body}<!-- agent-instructions:end -->\n"
        ),
    )
    .expect("AGENTS fixture written");
    fs::write(
        root.join("CLAUDE.md"),
        "# Claude contract\n\n<!-- agent-instructions:start -->\ncoordination_surface: governance_pipeline\n<!-- agent-instructions:end -->\n",
    )
    .expect("CLAUDE fixture written");
    fs::write(
        root.join("docs/AGENTS.md"),
        "# Docs agent contract\n\n<!-- agent-instructions:start -->\nsanctioned_primitives:\n  - oya-vcs\n<!-- agent-instructions:end -->\n",
    )
    .expect("docs AGENTS fixture written");
}

fn write_command_log_fixture(root: &Path, file_name: &str, contents: &str) {
    fs::create_dir_all(root).expect("command-log corpus dir created");
    fs::write(root.join(file_name), contents).expect("command-log fixture written");
}

fn write_honest_claims_plan(plans_dir: &Path, file_name: &str, id: &str, extra: &str) {
    fs::write(
        plans_dir.join(file_name),
        format!(
            "---\n\
             doc_class: ImplementationPlan\n\
             id: {id}\n\
             execution_unit: ChangeSet\n\
             changeset_contract: claimable-verifiable-bundleable-promotable\n\
             changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable\n\
             {extra}\
             ---\n\
             # {id}\n"
        ),
    )
    .expect("implementation plan written");
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}

fn write_design_spec_maturity_service_fixture(microservices_root: &Path) {
    let service = microservices_root.join("billing");
    fs::create_dir_all(service.join("contracts")).expect("contracts dir created");
    fs::create_dir_all(service.join("capabilities")).expect("capabilities dir created");
    fs::create_dir_all(service.join("policy")).expect("policy dir created");
    fs::create_dir_all(service.join("slos")).expect("slos dir created");
    fs::create_dir_all(service.join("runbooks")).expect("runbooks dir created");
    fs::write(
        service.join("manifest.json"),
        r#"{
  "service_id": "billing",
  "adrs": ["ADR-0123"],
  "regulatory_packs": ["REGIONAL-US"],
  "audit_chain": {
    "enabled": true,
    "seal_events": ["billing.invoice.issued"]
  }
}
"#,
    )
    .expect("manifest written");
    fs::write(
        service.join("PRD.md"),
        "# Billing PRD\n\nAcceptance criteria: billing design is implementation-ready.\n",
    )
    .expect("PRD written");
    fs::write(
        service.join("IP-001-billing-design.md"),
        "# IP-001 Billing Design\n\nAcceptance criteria: contracts, policy, SLOs, and runbooks are present.\n",
    )
    .expect("IP written");
    fs::write(
        service.join("contracts/billing.openapi.yaml"),
        "openapi: 3.1.0\ninfo:\n  title: Billing\n  version: 0.1.0\npaths: {}\n",
    )
    .expect("OpenAPI written");
    fs::write(
        service.join("contracts/billing.asyncapi.yaml"),
        "asyncapi: 3.0.0\ninfo:\n  title: Billing Events\n  version: 0.1.0\nchannels: {}\noperations: {}\n",
    )
    .expect("AsyncAPI written");
    fs::write(
        service.join("contracts/billing.proto"),
        "syntax = \"proto3\";\npackage oyatie.billing.v1;\nmessage BillingEvent { string id = 1; }\n",
    )
    .expect("proto written");
    fs::write(
        service.join("capabilities/invoice.yaml"),
        "id: cap.billing.invoice\n",
    )
    .expect("capability written");
    fs::write(
        service.join("policy/tenant-isolation.cedar"),
        "permit(principal, action, resource) when { principal.tenant == resource.tenant };\n",
    )
    .expect("policy written");
    fs::write(
        service.join("slos/billing.openslo.yaml"),
        "apiVersion: openslo/v1\nkind: SLO\nmetadata:\n  name: billing\nspec: {}\n",
    )
    .expect("SLO written");
    fs::write(
        service.join("runbooks/billing.md"),
        "# Billing Runbook\n\nIncident boundary: rollback invoice emission.\n",
    )
    .expect("runbook written");
    fs::write(
        service.join("threat-model.md"),
        "# Threat Model\n\nTenant invoice access and replay threats.\n",
    )
    .expect("threat model written");
    fs::write(
        service.join("failure-modes.md"),
        "# Failure Modes\n\nQueue delay, duplicate invoice emission, and policy denial.\n",
    )
    .expect("failure modes written");
    fs::write(
        service.join("cost-budget.md"),
        "# Cost Budget\n\nFinOps guardrail: bound invoice recomputation cost.\n",
    )
    .expect("cost budget written");
    fs::write(
        service.join("operational-boundaries.md"),
        "# Operational Boundaries\n\nIncident and capacity ownership remain design-only here.\n",
    )
    .expect("operational boundaries written");
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(label: &str) -> Self {
        Self {
            path: temp_dir(label),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).ok();
    }
}

fn repo_root() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .expect("repo root")
        .to_path_buf()
}

// --- active-artifact-contract integration tests (M02b/P22 exit-gate lane 5) ---

fn write_aac_registry(dir: &Path, rows: serde_json::Value) -> PathBuf {
    fs::create_dir_all(dir).expect("registry dir created");
    let path = dir.join("artifact-capabilities-registry.json");
    let registry = serde_json::json!({
        "_meta": { "contract_version": "v3.0.0" },
        "rows": rows
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&registry).expect("registry JSON serializes"),
    )
    .expect("registry written");
    path
}

fn aac_capability(
    status: &str,
    evidence_ref: Option<&str>,
    prerequisite_for_operational: &[&str],
    not_applicable_rationale: Option<&str>,
) -> serde_json::Value {
    let mut declaration = serde_json::Map::new();
    declaration.insert("status".to_string(), serde_json::json!(status));
    if let Some(evidence_ref) = evidence_ref {
        declaration.insert("evidence_ref".to_string(), serde_json::json!(evidence_ref));
    }
    if !prerequisite_for_operational.is_empty() {
        declaration.insert(
            "prerequisite_for_operational".to_string(),
            serde_json::json!(prerequisite_for_operational),
        );
    }
    if let Some(not_applicable_rationale) = not_applicable_rationale {
        declaration.insert(
            "not_applicable_rationale".to_string(),
            serde_json::json!(not_applicable_rationale),
        );
    }
    serde_json::Value::Object(declaration)
}

fn aac_capabilities(
    status: &str,
    evidence_ref: Option<&str>,
    prerequisite_for_operational: &[&str],
    not_applicable_rationale: Option<&str>,
) -> serde_json::Value {
    let mut capabilities = serde_json::Map::new();
    for capability in [
        "enforcement",
        "verification",
        "validation",
        "autogen",
        "selfheal",
        "selfupdate",
        "selfmaintain",
        "telemetry",
        "provenance",
    ] {
        capabilities.insert(
            capability.to_string(),
            aac_capability(
                status,
                evidence_ref,
                prerequisite_for_operational,
                not_applicable_rationale,
            ),
        );
    }
    serde_json::Value::Object(capabilities)
}

fn active_artifact_contract_repo_root() -> PathBuf {
    repo_root()
}

fn run_active_artifact_contract_gate(registry: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(active_artifact_contract_repo_root())
        .args([
            "gate",
            "validate",
            "active-artifact-contract",
            "--registry",
            registry.to_str().expect("utf8 registry path"),
        ])
        .output()
        .expect("gate command runs")
}

fn run_active_artifact_contract_gate_with_evidence(
    registry: &Path,
    evidence: &Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(active_artifact_contract_repo_root())
        .args([
            "gate",
            "validate",
            "active-artifact-contract",
            "--registry",
            registry.to_str().expect("utf8 registry path"),
            "--emit-evidence",
            evidence.to_str().expect("utf8 evidence path"),
        ])
        .output()
        .expect("gate command runs")
}

fn run_active_artifact_contract_gate_with_graph_edges(
    registry: &Path,
    graph_edges: &Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(active_artifact_contract_repo_root())
        .args([
            "gate",
            "validate",
            "active-artifact-contract",
            "--registry",
            registry.to_str().expect("utf8 registry path"),
            "--emit-graph-edges",
            graph_edges.to_str().expect("utf8 graph edges path"),
        ])
        .output()
        .expect("gate command runs")
}

#[test]
fn active_artifact_contract_gate_emits_canonical_graph_edge_json_bytes() {
    let temp = TempDirGuard::new("aac-graph-edges-canonical-json");
    let graph_edges = temp
        .path()
        .join("graph/active-artifact-contract-edges.json");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "active-artifact-contract-spec",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "schema"
            }
        ]),
    );

    let output = run_active_artifact_contract_gate_with_graph_edges(&registry, &graph_edges);

    assert!(
        output.status.success(),
        "graph edge emission must succeed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let emitted = fs::read_to_string(&graph_edges).expect("graph edges readable");
    let expected = format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "$schema_ref": "specs/knowledge-graph-schema.json",
            "_artifact_id": "active-artifact-contract-edges",
            "_meta": {
                "emitter": "oya-dev-cli gate validate active-artifact-contract",
                "layer": "semantic",
                "purpose": "Generated graph edges that connect active machine-readable artifacts to their declared schemas, registries, templates, and ledgers."
            },
            "edges": [{
                "source": "active-artifact-contract-spec",
                "target": "schema",
                "edge_type": "declares"
            }]
        }))
        .expect("canonical graph projection serializes")
    );
    assert_eq!(
        emitted, expected,
        "graph edge bytes must be canonical serde JSON"
    );
}

#[test]
fn active_artifact_contract_gate_escapes_vertical_tabs_in_graph_edge_json() {
    let temp = TempDirGuard::new("aac-graph-edges-vertical-tab");
    let graph_edges = temp.path().join("active-artifact-contract-edges.json");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "source\u{000b}",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "target\u{000b}",
                "capabilities": aac_capabilities("planned", None, &["foundation-prerequisite"], None)
            }
        ]),
    );

    let output = run_active_artifact_contract_gate_with_graph_edges(&registry, &graph_edges);

    assert!(
        output.status.success(),
        "graph edge emission must accept vertical tabs\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let emitted = fs::read_to_string(&graph_edges).expect("graph edges readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&emitted).expect("graph edges must parse as JSON");
    assert_eq!(parsed["edges"][0]["source"], "source\u{000b}");
    assert_eq!(parsed["edges"][0]["target"], "target\u{000b}");
    assert!(
        emitted.contains("\\u000b"),
        "vertical tab must use a JSON unicode escape; emitted={emitted:?}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_untracked_artifact_path() {
    let temp = TempDirGuard::new("aac-untracked-path");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "missing-artifact",
                "artifact_path": "specs/does-not-exist-for-aac-test.json",
                "artifact_profile": "schema"
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for untracked artifact_path\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("active-artifact-contract validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("R01"),
        "stderr must cite R01; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_duplicate_artifact_id() {
    let temp = TempDirGuard::new("aac-dup-id");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "dup-artifact",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "schema"
            },
            {
                "artifact_id": "dup-artifact",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "schema"
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for duplicate artifact_id\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("active-artifact-contract validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );
    assert!(
        stderr.contains("R02"),
        "stderr must cite R02; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_unknown_profile_without_capabilities() {
    let temp = TempDirGuard::new("aac-unknown-profile");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "unknown-profile-row",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "definitely-not-a-canonical-profile"
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for unknown artifact_profile without full capabilities\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown artifact_profile"),
        "stderr must cite unknown profile; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_missing_capability() {
    let temp = TempDirGuard::new("aac-missing-capability");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "partial-capability-row",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "capabilities": {
                    "enforcement": aac_capability(
                        "planned",
                        None,
                        &["fixture-prerequisite"],
                        None
                    )
                }
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for missing capabilities\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("R03"),
        "stderr must cite R03; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_operational_without_evidence() {
    let temp = TempDirGuard::new("aac-operational-without-evidence");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "operational-without-evidence",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "capabilities": aac_capabilities("operational", None, &[], None)
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for operational capability without evidence\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("R04"),
        "stderr must cite R04; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_planned_without_prerequisite() {
    let temp = TempDirGuard::new("aac-planned-without-prerequisite");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "planned-without-prerequisite",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "capabilities": aac_capabilities("planned", None, &[], None)
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for planned capability without prerequisite\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("R05"),
        "stderr must cite R05; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_rejects_blocked_without_foundation_prerequisite() {
    let temp = TempDirGuard::new("aac-blocked-without-prerequisite");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "blocked-without-prerequisite",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "capabilities": aac_capabilities("blocked-by-foundation", None, &[], None)
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        !output.status.success(),
        "expected failure for blocked capability without prerequisite\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("R06"),
        "stderr must cite R06; got: {stderr}"
    );
}

#[test]
fn active_artifact_contract_gate_reports_not_applicable_without_rationale() {
    let temp = TempDirGuard::new("aac-not-applicable-without-rationale");
    let evidence = temp.path().join("aac-evidence.json");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "not-applicable-without-rationale",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "capabilities": aac_capabilities("not-applicable", None, &[], None)
            }
        ]),
    );

    let output = run_active_artifact_contract_gate_with_evidence(&registry, &evidence);

    assert!(
        output.status.success(),
        "R07 is warning-only and must not fail the gate\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("9 warnings"),
        "stdout must count warnings; got: {stdout}"
    );
    assert!(
        stdout.contains("R07"),
        "stdout must cite R07; got: {stdout}"
    );

    let evidence_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence).expect("evidence readable"))
            .expect("evidence JSON parses");
    assert_eq!(evidence_json["outcome"], "success");
    assert_eq!(evidence_json["warning_count"], 9);
    assert_eq!(
        evidence_json["violations"][0]["rule_id"],
        "R07-not-applicable-without-rationale"
    );
}

#[test]
fn active_artifact_contract_gate_accepts_clean_registry() {
    let temp = TempDirGuard::new("aac-clean");
    let registry = write_aac_registry(
        temp.path(),
        serde_json::json!([
            {
                "artifact_id": "active-artifact-contract-spec",
                "artifact_path": "specs/active-machine-readable-artifact-contract.json",
                "artifact_profile": "schema"
            }
        ]),
    );

    let output = run_active_artifact_contract_gate(&registry);

    assert!(
        output.status.success(),
        "expected pass for clean single-row registry\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("active-artifact-contract validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

// ─── documentation-system integration tests (M02b/P22 exit-gate lane 3) ─────

/// Proves the documentation-system validator catches an `active` pipeline record
/// whose check_command is absent from the wired-commands catalog — the canonical
/// synthetic-violation shape for the lean-a5-documentation lane (ADR-0063 §"doc
/// coverage enforced").  The check-script override (`--check-script`) replaces
/// the canonical catalog with a fixture file that deliberately omits
/// `cargo run -p oya-dev-cli -- catalog validate`, so the `catalog` active step
/// is flagged as unwired.
#[test]
fn documentation_system_gate_catches_unwired_active_command() {
    let temp = temp_dir("doc-system-violation");
    fs::create_dir_all(&temp).expect("temp dir created");

    // DOCUMENTATION.md declares the fitness lane so the lane-declared check passes.
    let doc_path = temp.join("DOCUMENTATION.md");
    fs::write(
        &doc_path,
        "# Documentation\n\noya-governance-docs is the fitness lane.\n",
    )
    .expect("DOCUMENTATION.md written");

    // wiki quickref not referenced → wiki_quickref_referenced=false, skip presence check.
    let quickref_path = temp.join("quickref-absent.md");

    // check-script fixture: contains commands for all steps EXCEPT catalog validate —
    // this leaves the `catalog` active record unwired.
    let check_script_path = temp.join("check.sh");
    fs::write(
        &check_script_path,
        "cargo run -p oya-dev-cli -- gate validate api-semver\n\
         cargo run -p oya-dev-cli -- gate validate documentation-system\n\
         cargo run -p oya-dev-cli -- gate validate adr-citation\n\
         cargo run -p oya-dev-cli -- gate validate doc-catalog\n",
    )
    .expect("check script written");

    // Pipeline TSV: all 6 required steps; `catalog` is active with a check_command
    // that is NOT present in the check-script above (the violation).
    let pipeline_path = temp.join("pipeline.tsv");
    fs::write(
        &pipeline_path,
        "step_id\tdocumented_command\tstate\tcheck_command\tscope_path\trationale\n\
         rustdoc\toya doc rustdoc\ttracked-deferred\t\tcrates\tblocked: full rustdoc artifact publication is not part of the bootstrap lane\n\
         openapi\toya doc openapi\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate api-semver\tcontracts\tcontracts are absent; api-semver guards first contract adoption\n\
         mdbook\toya doc mdbook\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate documentation-system\tdocs/site\tpublic mdbook source is absent; documentation-system guards the pipeline registry\n\
         adr-index\toya doc adr-index\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate adr-citation\tdocs/decisions\tadr-citation prevents stale ADR references until generator publication ships\n\
         catalog\toya doc catalog\tactive\tcargo run -p oya-dev-cli -- catalog validate\tregistry/catalog\t\n\
         lint\toya doc lint\tactive\tcargo run -p oya-dev-cli -- gate validate doc-catalog\tdocs\t\n",
    )
    .expect("pipeline TSV written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "documentation-system",
            "--documentation",
            doc_path.to_str().expect("utf8 doc path"),
            "--pipeline",
            pipeline_path.to_str().expect("utf8 pipeline path"),
            "--check-script",
            check_script_path.to_str().expect("utf8 check script path"),
            "--wiki-quickref",
            quickref_path.to_str().expect("utf8 quickref path"),
            "--repo-root",
            temp.to_str().expect("utf8 repo root"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        !output.status.success(),
        "expected failure for unwired active check_command\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("documentation system validation failed"),
        "stderr must contain failure message; got: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

/// Proves the documentation-system validator passes when all six required
/// pipeline steps are present and every active/adoption-guard record has its
/// check_command present in the wired-commands catalog — clean-path smoke-test.
#[test]
fn documentation_system_gate_passes_clean_pipeline() {
    let temp = temp_dir("doc-system-clean");
    fs::create_dir_all(&temp).expect("temp dir created");

    // DOCUMENTATION.md with fitness lane declaration; no wiki quickref reference
    // so the wiki-quickref presence check is skipped.
    let doc_path = temp.join("DOCUMENTATION.md");
    fs::write(
        &doc_path,
        "# Documentation\n\noya-governance-docs is the fitness lane.\n",
    )
    .expect("DOCUMENTATION.md written");

    let quickref_path = temp.join("quickref-absent.md");

    // check-script fixture: contains all four check_commands used by
    // active/adoption-guard records below.
    let check_script_path = temp.join("check.sh");
    fs::write(
        &check_script_path,
        "cargo run -p oya-dev-cli -- gate validate api-semver\n\
         cargo run -p oya-dev-cli -- gate validate documentation-system\n\
         cargo run -p oya-dev-cli -- gate validate adr-citation\n\
         cargo run -p oya-dev-cli -- catalog validate\n\
         cargo run -p oya-dev-cli -- gate validate doc-catalog\n",
    )
    .expect("check script written");

    // Pipeline TSV: all 6 required steps; every check_command is present in
    // the check-script above — no violations.
    let pipeline_path = temp.join("pipeline.tsv");
    fs::write(
        &pipeline_path,
        "step_id\tdocumented_command\tstate\tcheck_command\tscope_path\trationale\n\
         rustdoc\toya doc rustdoc\ttracked-deferred\t\tcrates\tblocked: full rustdoc artifact publication is not part of the bootstrap lane\n\
         openapi\toya doc openapi\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate api-semver\tcontracts\tcontracts are absent; api-semver guards first contract adoption\n\
         mdbook\toya doc mdbook\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate documentation-system\tdocs/site\tpublic mdbook source is absent; documentation-system guards the pipeline registry\n\
         adr-index\toya doc adr-index\tadoption-guard\tcargo run -p oya-dev-cli -- gate validate adr-citation\tdocs/decisions\tadr-citation prevents stale ADR references until generator publication ships\n\
         catalog\toya doc catalog\tactive\tcargo run -p oya-dev-cli -- catalog validate\tregistry/catalog\t\n\
         lint\toya doc lint\tactive\tcargo run -p oya-dev-cli -- gate validate doc-catalog\tdocs\t\n",
    )
    .expect("pipeline TSV written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "documentation-system",
            "--documentation",
            doc_path.to_str().expect("utf8 doc path"),
            "--pipeline",
            pipeline_path.to_str().expect("utf8 pipeline path"),
            "--check-script",
            check_script_path.to_str().expect("utf8 check script path"),
            "--wiki-quickref",
            quickref_path.to_str().expect("utf8 quickref path"),
            "--repo-root",
            temp.to_str().expect("utf8 repo root"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "expected pass for fully-wired pipeline\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("documentation system validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

/// ADR-0361 R3b: with GitHub Actions workflows retired (absent dir), the
/// Jenkins-reported status-context manifest is the producer source, and the
/// protection-context-match gate still passes for aligned contexts.
#[test]
fn protection_context_match_gate_passes_from_jenkins_manifest_without_workflows() {
    let temp = temp_dir("pcm-jenkins-manifest");
    fs::create_dir_all(&temp).expect("temp dir created");

    let branch_protection = "branches:\n  dev:\n    require_pull_request: true\n    \
                             required_status_checks:\n      - cargo-fmt\n      \
                             - oya-governance-protection-context-match\n    \
                             require_signed_commits: true\n";
    let protection_file = temp.join("branch-protection.yaml");
    fs::write(&protection_file, branch_protection).expect("branch-protection written");

    let manifest = "{\n  \"reported_status_contexts\": [\n    \"cargo-fmt\",\n    \
                    \"oya-governance-protection-context-match\"\n  ]\n}\n";
    let manifest_file = temp.join("reported-status-contexts.json");
    fs::write(&manifest_file, manifest).expect("manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "protection-context-match",
            "--branch-protection",
            protection_file.to_str().expect("utf8 protection path"),
            // workflows dir intentionally ABSENT (retired per ADR-0361)
            "--workflows-dir",
            temp.join("no-such-workflows-dir").to_str().expect("utf8"),
            "--reported-contexts",
            manifest_file.to_str().expect("utf8 manifest path"),
            "--branch",
            "dev",
            "--skip-applied-branch-protection",
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "Jenkins manifest must satisfy required contexts with no workflows dir\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("protection-context-match validation passed"),
        "stdout must confirm pass; got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}
