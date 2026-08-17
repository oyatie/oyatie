// GATE-3 cloud-ci-staleness-reaper: RED/GREEN fixture corpus + born-blocking live-corpus
// self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_stale_artifact_detection::{Verdict, evaluate};
use serde_json::{Value, json};

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
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn fixture_dir() -> PathBuf {
    repo_root().join("specs/fixtures/staleness-reaper")
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
fn staleness_reaper_fixtures_execute_red_green_cases() {
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
        "staleness-reaper fixture corpus must not be empty"
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
        "staleness-reaper fixtures must include BOTH RED and GREEN cases"
    );
}

/// Born-blocking self-test: GATE-3 must REPORT over-budget AND unreachable archive
/// candidates on TODAY's real corpus. Per the firewall doctrine, "a firewall that doesn't
/// block today is the facade we're killing." This ages the live registry rows from the
/// UNTRACKED scm-volatile-facts snapshot (ADR-0552: history-derived aging data is
/// materialized at evaluation time, never committed) against the ttl-policy budgets and
/// asserts the gate flags real candidates as `stale_over_budget_unreachable` — REPORTED,
/// never reaped in-gate.
///
/// Counts are MEASURED, not hardcoded.
#[test]
fn gate3_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let registry = run_producer_face(&root, "registry");
    let rows = registry["rows"].as_array().expect("registry rows");
    assert!(
        rows.len() > 1000,
        "live registry should account thousands of paths, got {}",
        rows.len()
    );

    let volatile = volatile_facts_value(&root);
    let now_secs = volatile["head_time_secs"].as_u64().unwrap_or(0);
    let commit_ts = volatile_commit_author_timestamps(&volatile);
    let last_touch = volatile["last_touch_commit"]
        .as_object()
        .expect("volatile last_touch_commit");

    // Build the gate input by aging each row from its last-touch commit timestamp (the
    // path -> sha join lives in the volatile snapshot; registry rows are history-free).
    let mut aged_rows: Vec<Value> = Vec::new();
    let mut candidate_count = 0u64;
    for row in rows {
        let path = row["path"].as_str().unwrap_or("");
        let sha = last_touch.get(path).and_then(Value::as_str).unwrap_or("");
        let age_days = commit_ts
            .get(sha)
            .map(|ts| (now_secs.saturating_sub(*ts)) / 86_400)
            .unwrap_or(0);
        let mut aged = row.clone();
        if let Value::Object(map) = &mut aged {
            map.insert("age_days".into(), json!(age_days));
        }
        // Count the real archive candidates (the evidence digest).
        let ttl = &row["ttl"];
        let budget = ttl["budget_days"].as_u64();
        let protected = ttl["protected"].as_bool() == Some(true);
        let unreachable = row["reachable_from"]
            .as_array()
            .map(Vec::is_empty)
            .unwrap_or(true);
        if let Some(b) = budget
            && !protected
            && unreachable
            && age_days > b
        {
            candidate_count += 1;
        }
        aged_rows.push(aged);
    }

    let report = evaluate(&json!({"rows": aged_rows}));

    eprintln!(
        "BORN-BLOCKING live-corpus counts: total_rows={} now_secs={} archive_candidates(over-budget+unreachable+unprotected)={} violations={:?}",
        rows.len(),
        now_secs,
        candidate_count,
        report.violations
    );

    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-3 MUST go RED on today's corpus (the firewall must report stale candidates today)"
    );
    assert!(
        report.violations.contains("stale_over_budget_unreachable"),
        "over-budget + unreachable scratch artifacts -> stale_over_budget_unreachable must fire"
    );
    assert!(
        candidate_count > 0,
        "the live corpus must surface at least one over-budget unreachable archive candidate"
    );
}

/// Run the producer to emit a single face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

/// The committed scm-facts face beside the accounting faces (a declared input under buck2).
fn scm_facts_path(root: &Path) -> PathBuf {
    root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json")
}

/// The commit-sha -> author-timestamp map from the volatile snapshot. Mirrors the aging the
/// emitter froze (`commit_author_ts_secs`), so the gate ages each row deterministically.
fn volatile_commit_author_timestamps(volatile: &Value) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    if let Some(obj) = volatile["commit_author_ts_secs"].as_object() {
        for (sha, ts) in obj {
            if let Some(ts) = ts.as_u64() {
                map.insert(sha.clone(), ts);
            }
        }
    }
    map
}

/// Read + parse the UNTRACKED scm-volatile-facts snapshot (ADR-0552, FRIC-1781234047).
/// FAIL-CLOSED: a missing snapshot is a hard failure naming the exact materialization
/// command — the gate must never silently age rows from nothing.
fn volatile_facts_value(root: &Path) -> Value {
    let path = root.join("ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "FAIL-CLOSED: scm-volatile-facts snapshot missing at {} ({e}). History-derived \
             aging facts are materialized, never committed (ADR-0552). Materialize them: \
             buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . (CI runs this before every \
             gate lane).",
            path.display()
        )
    });
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("parse scm-volatile-facts {}: {e}", path.display()))
}
