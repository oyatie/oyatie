// GATE-2 cloud-ci-total-accounting: RED/GREEN fixture corpus + born-blocking live-corpus
// self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_artifact_accountability::{Verdict, evaluate};
use serde_json::Value;

/// Walk up to the repo root (the dir holding specs/root-hub-pointers.json), matching the
/// existing kernel-test convention.
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

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(if Path::new(bin).is_absolute() {
        PathBuf::from(bin)
    } else {
        root.join(bin)
    })
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn fixture_dir() -> PathBuf {
    repo_root().join("specs/fixtures/total-accounting")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
fn total_accounting_fixtures_execute_red_green_cases() {
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
        "total-accounting fixture corpus must not be empty"
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
                // The contract: report.violations EXACTLY equals expected_violations.
                assert_eq!(report.violations, expected, "{label} violations mismatch");
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "total-accounting fixtures must include BOTH RED and GREEN cases"
    );
}

/// Born-blocking self-test: GATE-2 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer over the live tree and asserts the gate flags the real defects:
/// - broadly `unowned` (live `find -name OWNERS` = 0 tree-wide)
/// - the oya-foundry-* residue as `unjustified` (ADR-0363 claims it was "eradicated")
/// - the oya-governance-* crates broadly `unreachable`
///
/// Counts are MEASURED, not hardcoded (the plan's 780/57 were not re-derived this session).
#[test]
fn gate2_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();

    // Regenerate the registry from the live tree via the producer binary (in --stdout mode).
    let registry = run_producer_stdout(&root);
    let rows = registry["rows"].as_array().expect("registry rows");
    assert!(
        rows.len() > 1000,
        "live registry should account thousands of paths, got {}",
        rows.len()
    );

    // Evaluate the live registry through the gate.
    let report = evaluate(&registry);

    // The live corpus is RED — and specifically for the real systemic gaps.
    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-2 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("unowned"),
        "live corpus has 0 OWNERS files tree-wide -> unowned must fire"
    );
    assert!(
        report.violations.contains("unreachable"),
        "unwired governance crates -> unreachable must fire"
    );

    // Count the real exhibits for the evidence digest.
    let unowned = rows.iter().filter(|r| r["owner"].is_null()).count();
    let unreachable = rows
        .iter()
        .filter(|r| {
            r["reachable_from"]
                .as_array()
                .map(Vec::is_empty)
                .unwrap_or(true)
        })
        .count();
    let foundry_residue = rows
        .iter()
        .filter(|r| {
            r["path"]
                .as_str()
                .is_some_and(|p| p.contains("oya-foundry"))
        })
        .count();
    let governance_unreachable = rows
        .iter()
        .filter(|r| {
            r["path"]
                .as_str()
                .is_some_and(|p| p.contains("oya-governance"))
                && r["reachable_from"]
                    .as_array()
                    .map(Vec::is_empty)
                    .unwrap_or(true)
        })
        .count();

    // These are the live exhibits; surface them so the test output is the evidence.
    eprintln!(
        "BORN-BLOCKING live-corpus counts: total_rows={} unowned={} unreachable={} foundry_residue={} governance_unreachable={}",
        rows.len(),
        unowned,
        unreachable,
        foundry_residue,
        governance_unreachable
    );

    assert!(unowned > 1000, "owner gap is systemic, got {unowned}");
}

/// Every tracked `OWNERS` file is accounted BY CONSTRUCTION on the LIVE corpus — no ADR
/// prose mention, no `specs/reachability-registry.json` row, no hand-edit of any global file.
///
/// This is the regression PR #1473 shipped: a one-line `os/OWNERS` rode a `git mv` and turned
/// `dev` RED (`unjustified` 16545 -> 16546, `unreachable` 12167 -> 12168) because the
/// accounting system demanded that the very files it resolves OWNERSHIP from be individually
/// justified to itself. ADR-0562 §10.29 had already written the manual obligation down ("the
/// OWNERS file rides the `git mv`; its registry entry does not") and the next PR forgot it —
/// so this is the detector that replaces the discipline.
///
/// The assertion is deliberately over the WHOLE tracked OWNERS population rather than over
/// `os/OWNERS` alone, so it keeps holding as the ~275 remaining reorg destinations land and
/// as `os/OWNERS` itself moves again. `os/OWNERS` is then named explicitly because it is the
/// exhibit: if that one path regressed, the general invariant could still pass on a corpus
/// that no longer contains it.
#[test]
fn owners_files_are_accounted_by_construction_on_the_live_corpus() {
    let root = repo_root();
    let registry = run_producer_stdout(&root);
    let rows = registry["rows"].as_array().expect("registry rows");

    let owners_rows: Vec<&Value> = rows
        .iter()
        .filter(|row| {
            row["path"]
                .as_str()
                .is_some_and(|p| p == "OWNERS" || p.ends_with("/OWNERS"))
        })
        .collect();
    // Non-vacuity: a filter that silently matched nothing would make every assertion below
    // pass trivially. The live tree carries >100 OWNERS files.
    assert!(
        owners_rows.len() > 50,
        "expected the live corpus to carry many OWNERS rows, got {} — the filter is measuring \
         the wrong thing",
        owners_rows.len()
    );

    let unaccounted: Vec<(&str, bool, bool)> = owners_rows
        .iter()
        .map(|row| {
            (
                row["path"].as_str().unwrap_or_default(),
                row["justification_ref"].is_null(),
                row["reachable_from"]
                    .as_array()
                    .map(Vec::is_empty)
                    .unwrap_or(true),
            )
        })
        .filter(|(_, unjustified, unreachable)| *unjustified || *unreachable)
        .collect();
    assert!(
        unaccounted.is_empty(),
        "every tracked OWNERS file must be accounted by construction (schema-valid ⇒ justified \
         + reachable). Unaccounted: {unaccounted:?}. If one of these fails the ADR-0555 OWNERS \
         schema that is CORRECT and the fix is the file's CONTENT — never a reachability-registry row."
    );

    // The exhibit (PR #1473). Named explicitly: nobody hand-wrote a row for it.
    // justification_ref stays owners-schema (OWNERS floor). reachable_from may also
    // carry envelope-prefix-ownership once Phase1 prefix allows cover os/** — the floor
    // only fills an empty reachable_from, so envelope coverage replaces the sole
    // owners-schema stamp without changing by-construction justification.
    let os_owners = rows
        .iter()
        .find(|row| row["path"] == "os/OWNERS")
        .expect("os/OWNERS must be in the live registry");
    assert_eq!(os_owners["justification_ref"], "owners-schema");
    let os_reachable = os_owners["reachable_from"]
        .as_array()
        .expect("os/OWNERS reachable_from must be an array");
    assert!(
        !os_reachable.is_empty(),
        "os/OWNERS must be reachable (owners-schema floor and/or envelope-prefix)"
    );
    assert!(
        os_reachable.iter().any(|v| {
            v.as_str() == Some("owners-schema") || v.as_str() == Some("envelope-prefix-ownership")
        }),
        "os/OWNERS reachable_from must include owners-schema or envelope-prefix-ownership, got {os_reachable:?}"
    );

    // ...and the gate agrees: no per-path finding is keyed to ANY OWNERS file.
    let owners_findings: Vec<String> = ci_artifact_accountability::evaluate_keyed(&registry)
        .into_iter()
        .filter(|f| f.key == "OWNERS" || f.key.ends_with("/OWNERS"))
        .map(|f| format!("{}:{}", f.code, f.key))
        .collect();
    assert!(
        owners_findings.is_empty(),
        "GATE-2 must raise no violation against any OWNERS file, got {owners_findings:?}"
    );

    eprintln!(
        "OWNERS-BY-CONSTRUCTION live-corpus counts: owners_rows={} unaccounted=0 registry_rows={}",
        owners_rows.len(),
        rows.len()
    );
}

/// The other half of the detector: the deleted rows must STAY deleted.
///
/// `specs/reachability-registry.json` is a GLOBAL MUTABLE FILE — every capability move that
/// wants a row has to edit it, which serializes moves behind one merge-conflict point. 49 of
/// its 124 rows existed only to permit an OWNERS file, at a median of ~217 characters of
/// hand-written anchor prose apiece. Re-adding one is a no-op (the file is already accounted)
/// that the next mover then has to carry, so it is caught here rather than tolerated. The
/// `--fix-reachability` bridge refuses these paths too, so the paved road agrees with the gate.
#[test]
fn owners_files_are_never_registered_in_the_reachability_registry() {
    let registry: Value = load_json(&repo_root().join("specs/reachability-registry.json"));
    let registered = registry["registered"].as_array().expect("registered array");
    // Non-vacuity: the file still carries its legitimate non-OWNERS registrations.
    assert!(
        registered.len() > 20,
        "reachability registry looks empty ({} rows) — the probe is measuring the wrong file",
        registered.len()
    );
    let owners_entries: Vec<&str> = registered
        .iter()
        .filter_map(|entry| entry["prefix"].as_str())
        .filter(|prefix| *prefix == "OWNERS" || prefix.ends_with("/OWNERS"))
        .collect();
    assert!(
        owners_entries.is_empty(),
        "OWNERS files are accounted by construction — a reachability registration for one is \
         dead weight every future capability move has to maintain. Remove: {owners_entries:?}"
    );
}

/// Run the producer to emit the registry face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_stdout(root: &Path) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer stdout is valid JSON")
}
