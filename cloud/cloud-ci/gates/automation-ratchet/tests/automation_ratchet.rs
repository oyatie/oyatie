// GATE-4 cloud-ci-automation-ratchet: RED/GREEN fixture corpus (the EXISTING
// specs/fixtures/phase0-automation-ratchet/ seed, hardened) + born-blocking live-corpus
// self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use automation_ratchet::{evaluate, Verdict};
use serde_json::{json, Value};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn fixture_dir() -> PathBuf {
    // The EXISTING seed fixture directory (the gate hardens it; it does not relocate it).
    repo_root().join("specs/fixtures/phase0-automation-ratchet")
}

fn load_json(path: &PathBuf) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn expected_violations(fixture: &Value) -> BTreeSet<String> {
    fixture["expected_violations"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn automation_ratchet_fixtures_execute_red_green_cases() {
    let dir = fixture_dir();
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "automation-ratchet fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let report = evaluate(&fixture);
        let expected = expected_violations(&fixture);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        match fixture["expected_verdict"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert_eq!(
                    report.verdict,
                    Verdict::Green,
                    "{label} should be GREEN, got violations {:?}",
                    report.violations
                );
                assert!(
                    report.violations.is_empty(),
                    "{label} GREEN must have zero violations, got {:?}",
                    report.violations
                );
            }
            Some("RED") => {
                seen_red = true;
                assert_eq!(report.verdict, Verdict::Red, "{label} should be RED");
                assert_eq!(report.violations, expected, "{label} violations mismatch");
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "automation-ratchet fixtures must include BOTH RED and GREEN cases"
    );
}

/// Born-blocking self-test: GATE-4 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer's enforcement-inventory face over the live tree and asserts the gate flags
/// the real defects:
/// - `advisory_claiming_enforced` — the oya-governance-* crates + diataxis-doc-class +
///   prd-axis-coverage lanes (claim enforcement, no wired buck2 gate target)
/// - `blocking_invariant_mapped_to_oya_cli` — ADR-0365's `oya gate`/`oya gen` verified_by lines
///
/// Counts are MEASURED, not hardcoded.
#[test]
fn gate4_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let inventory = run_producer_face(&root, "enforcement-inventory");
    let surfaces = inventory["rows"].as_array().expect("enforcement rows");
    assert!(
        !surfaces.is_empty(),
        "live enforcement inventory must not be empty"
    );

    // Adapt the enforcement-inventory face rows into automation-matrix rows so the same
    // pure evaluator decides them. A surface that maps a blocking invariant to oya CLI is
    // classified blocking with that target; an unwired claim carries claims_enforced.
    let mut matrix_rows: Vec<Value> = Vec::new();
    let mut advisory_count = 0u64;
    let mut oya_cli_count = 0u64;
    for surface in surfaces {
        let id = surface["id"].as_str().unwrap_or("");
        let src = surface["source_artifact"].as_str().unwrap_or("");
        let claims = surface["claims_enforced"].as_bool() == Some(true);
        let wired = surface["has_wired_buck2_target"].as_bool() == Some(true);
        let maps_oya = surface["maps_to_oya_cli"].as_bool() == Some(true);
        if claims && !wired {
            advisory_count += 1;
        }
        if maps_oya {
            oya_cli_count += 1;
        }
        matrix_rows.push(json!({
            "id": id,
            "source_artifact": src,
            "requirement": "Live enforcement surface inventoried by the producer.",
            "classification": if maps_oya { "automated_blocking_now" } else { "automated_advisory_until_p0_0" },
            "owner": "platform-governance",
            "target_gate_or_controller": if maps_oya { "oya gate / oya gen verified_by authority" } else { src },
            "blocking_fixture": "specs/fixtures/phase0-automation-ratchet/",
            "retirement_phase": "P0.0",
            "evidence_path": src,
            "no_new_oya_cli_surface": !maps_oya,
            "claims_enforced": claims,
            "has_wired_buck2_target": wired
        }));
    }

    let report = evaluate(&json!({"rows": matrix_rows}));

    eprintln!(
        "BORN-BLOCKING live-corpus counts: enforcement_surfaces={} advisory_claiming_enforced={} blocking_invariant_mapped_to_oya_cli={} violations={:?}",
        surfaces.len(),
        advisory_count,
        oya_cli_count,
        report.violations
    );

    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-4 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("advisory_claiming_enforced"),
        "oya-governance crates + governance lanes claim enforcement with no wired buck2 target -> advisory_claiming_enforced must fire"
    );
    assert!(
        report.violations.contains("blocking_invariant_mapped_to_oya_cli"),
        "ADR-0365 oya gate/oya gen verified_by lines -> blocking_invariant_mapped_to_oya_cli must fire"
    );
    assert!(advisory_count > 0, "expected unwired enforcement claims");
    assert!(oya_cli_count > 0, "expected oya-cli-mapped invariants");
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--quiet")
        .arg("-p")
        .arg("accounting-registry-producer")
        .arg("--")
        .arg("--repo-root")
        .arg(root)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run accounting-registry-producer");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}
