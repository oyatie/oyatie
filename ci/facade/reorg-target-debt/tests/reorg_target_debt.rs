// ci-reorg-target-debt gate test (Global Binding Rule 1; North-Star Completion bootstrap
// step T3b). Two halves, both required:
//
//   (a) LIVE-CORPUS, born-blocking zero-NEW ratchet: Arms A–D run over the real tree
//       against the committed shrink-only baseline and must be GREEN — the existing
//       target-prefix estate is migration inventory, anything NEW fails closed. The run
//       must carry the liveness signal (evaluated_path_count, evaluated_arms); a missing
//       run is a gap, never a pass.
//
//   (b) FIXTURE CORPUS: specs/fixtures/reorg-target-debt/tc-*.json (parse-verbatim gate
//       inputs, ADR-0555 convention) prove every refusal class RED and the proven-claim
//       case GREEN without touching the live tree, through the SAME engine the binary
//       runs — including the fail-closed interval-audit mode.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use ci_reorg_target_debt::{
    Baseline, Policy, Report, Verdict, audit_interval, check_live_tree,
    collect_target_prefix_paths, evaluate_masterplan, evaluate_reduction_claims, evaluate_tree,
    evaluate_workspace_manifest, load_baseline, load_json, load_policy, POLICY_PATH,
};

/// Walk up from the test's working directory to the repo root (the dir holding the
/// canonical root-hub pointer file). Mirrors the helper in the baseline-ratchet
/// registration meta-test so both gates resolve the root identically.
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

fn live_policy(root: &Path) -> Policy {
    let (policy, _) = load_policy(root, POLICY_PATH).expect("load live policy");
    policy
}

fn fixtures_dir(root: &Path) -> PathBuf {
    root.join("specs/fixtures/reorg-target-debt")
}

fn finding_codes(report: &Report) -> Vec<String> {
    report.findings.iter().map(|f| f.code.clone()).collect()
}

// ─── (a) live corpus ────────────────────────────────────────────────────────

#[test]
fn live_tree_is_green_at_the_committed_baseline_with_liveness_signal() {
    let root = repo_root();
    let policy = live_policy(&root);
    let baseline = load_baseline(&root, &policy).expect("load committed baseline");
    let report = check_live_tree(&root, &policy, &baseline).expect("evaluate live tree");

    assert_eq!(
        report.verdict(),
        Verdict::Green,
        "born-blocking zero-NEW ratchet: the live tree must be green at the committed \
         shrink-only baseline. Findings: {:#?}",
        report.findings
    );
    // Liveness signal: every run reports what it evaluated. A run that examined nothing
    // is itself a gap.
    assert!(
        report.evaluated_path_count > 0,
        "the gate must report a non-zero evaluated-path count"
    );
    let rendered = report.to_json();
    assert!(rendered.get("evaluated_path_count").is_some());
    assert!(
        rendered
            .get("evaluated_arms")
            .and_then(Value::as_array)
            .is_some_and(|arms| arms.len() == 4),
        "all four blocking arms must be evaluated on every run"
    );
}

#[test]
fn committed_baseline_matches_the_live_target_prefix_estate_exactly() {
    let root = repo_root();
    let policy = live_policy(&root);
    let baseline = load_baseline(&root, &policy).expect("load committed baseline");
    let live = collect_target_prefix_paths(&root, &policy).expect("collect target-prefix paths");

    let new: Vec<&String> = live.difference(&baseline.paths).collect();
    let stale: Vec<&String> = baseline.paths.difference(&live).collect();
    assert!(
        new.is_empty() && stale.is_empty(),
        "baseline drift — NEW target-prefix file(s) {new:?} / stale baseline row(s) {stale:?}. \
         New files under a target prefix are refused (Global Binding Rule 1); admissible \
         removals require regenerating the baseline with: {}",
        policy.regeneration_command
    );
}

// ─── (b) fixture corpus ─────────────────────────────────────────────────────

const REQUIRED_CASES: [&str; 6] = [
    "tc-RTD-bad-new-file-under-target-prefix.json",
    "tc-RTD-bad-new-workspace-path-dep.json",
    "tc-RTD-bad-work-item-target-anchor.json",
    "tc-RTD-bad-unproven-net-reduction-claim.json",
    "tc-RTD-good-proven-net-reduction-claim.json",
    "tc-RTD-audit-bad-planted-target-debt-commit.json",
];

fn baseline_from_fixture(input: &Value) -> Baseline {
    let set = |key: &str| -> BTreeSet<String> {
        input
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|v| v.as_str().expect("baseline entry is a string").to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    Baseline {
        paths: set("baseline_paths"),
        workspace_path_deps: set("baseline_workspace_path_deps"),
        dep_names: set("baseline_dep_names"),
        anchors: set("baseline_anchors"),
    }
}

fn run_fixture(policy: &Policy, fixture: &Value, name: &str) {
    let kind = fixture
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{name}: fixture must declare kind"));
    let input = fixture
        .get("input")
        .unwrap_or_else(|| panic!("{name}: fixture must declare input"));
    let expected_codes: Vec<String> = fixture
        .get("expected_codes")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{name}: fixture must declare expected_codes"))
        .iter()
        .map(|v| v.as_str().expect("expected code is a string").to_owned())
        .collect();
    let baseline = baseline_from_fixture(input);

    let report = match kind {
        "tree" => {
            let paths: BTreeSet<String> = input
                .get("paths")
                .and_then(Value::as_array)
                .expect("tree fixture declares paths")
                .iter()
                .map(|v| v.as_str().expect("path is a string").to_owned())
                .collect();
            evaluate_tree(policy, &baseline, &paths)
        }
        "workspace-manifest" => {
            let manifest = input
                .get("manifest_lines")
                .and_then(Value::as_array)
                .expect("workspace-manifest fixture declares manifest_lines")
                .iter()
                .map(|v| v.as_str().expect("manifest line is a string"))
                .collect::<Vec<_>>()
                .join("\n");
            evaluate_workspace_manifest(policy, &baseline, &manifest, "workspace.dependencies")
        }
        "masterplan" => {
            let plan = input.get("plan").expect("masterplan fixture declares plan");
            evaluate_masterplan(policy, &baseline, plan)
        }
        "reduction-claims" => {
            let artifact = input
                .get("artifact")
                .expect("reduction-claims fixture declares artifact");
            evaluate_reduction_claims(policy, artifact)
        }
        "interval-audit" => {
            run_audit_fixture(policy, fixture, input, name);
            return;
        }
        other => panic!("{name}: unknown fixture kind {other:?}"),
    };

    assert_eq!(
        finding_codes(&report),
        expected_codes,
        "{name}: finding codes must equal the declared expectation exactly"
    );
    assert!(
        !report.evaluated_arms.is_empty(),
        "{name}: every evaluation carries the liveness arm list"
    );
}

fn run_audit_fixture(policy: &Policy, fixture: &Value, input: &Value, name: &str) {
    let audit_input = input
        .get("audit_input")
        .unwrap_or_else(|| panic!("{name}: interval-audit fixture must declare audit_input"));
    let expected_verdict = fixture
        .get("expected_verdict")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{name}: interval-audit fixture must declare expected_verdict"));

    match audit_interval(policy, audit_input) {
        Ok(report) => {
            assert_ne!(
                expected_verdict, "invalid",
                "{name}: expected fail-closed input rejection but the audit ran"
            );
            assert_eq!(
                report.verdict().to_string(),
                expected_verdict,
                "{name}: audit verdict mismatch; findings: {:#?}",
                report.findings
            );
            let expected_commits: BTreeSet<String> = fixture
                .get("expected_finding_commits")
                .and_then(Value::as_array)
                .unwrap_or_else(|| {
                    panic!("{name}: interval-audit fixture must declare expected_finding_commits")
                })
                .iter()
                .map(|v| v.as_str().expect("commit sha is a string").to_owned())
                .collect();
            let reported: BTreeSet<String> = report
                .findings
                .iter()
                .map(|f| f.subject.split(':').next().unwrap_or("").to_owned())
                .collect();
            assert_eq!(
                reported, expected_commits,
                "{name}: the audit must report exactly the planted debt commit(s)"
            );
            // Liveness signal on the audit surface too.
            let rendered = report.to_json();
            assert!(rendered.get("evaluated_path_count").is_some());
            assert!(rendered.get("evaluated_arms").is_some());
        }
        Err(error) => {
            assert_eq!(
                expected_verdict, "invalid",
                "{name}: unexpected audit input rejection: {error}"
            );
            assert!(
                error.to_string().contains("RTD_AUDIT_INPUT_INVALID"),
                "{name}: fail-closed rejection must carry the explicit finding code: {error}"
            );
        }
    }
}

#[test]
fn fixture_corpus_proves_every_arm_through_the_live_engine() {
    let root = repo_root();
    let policy = live_policy(&root);
    let dir = fixtures_dir(&root);

    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read fixtures dir {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("tc-") && name.ends_with(".json"))
        .collect();
    names.sort();

    for required in REQUIRED_CASES {
        assert!(
            names.iter().any(|name| name == required),
            "required fixture case {required} is missing from {}",
            dir.display()
        );
    }

    for name in &names {
        let fixture = load_json(&dir.join(name)).unwrap_or_else(|e| panic!("load {name}: {e}"));
        run_fixture(&policy, &fixture, name);
    }
}

#[test]
fn audit_remediation_lifecycle_planted_commit_stays_red_until_remediated() {
    let root = repo_root();
    let policy = live_policy(&root);
    let dir = fixtures_dir(&root);
    let planted = load_json(&dir.join("tc-RTD-audit-bad-planted-target-debt-commit.json"))
        .expect("load planted-commit audit fixture");
    let mut audit_input = planted["input"]["audit_input"].clone();

    let report = audit_interval(&policy, &audit_input).expect("planted-range audit runs");
    assert_eq!(report.verdict(), Verdict::Red, "unremediated planted debt stays red");

    // The SAME capture with a remediation record present goes green while the finding
    // stays reported as durable evidence.
    let planted_sha = report.findings[0]
        .subject
        .split(':')
        .next()
        .expect("finding subject carries the commit sha")
        .to_owned();
    audit_input["remediation_records"] = serde_json::json!([
        { "commit": planted_sha, "resolution": "reverted; target-surface census re-measured" }
    ]);
    let remediated = audit_interval(&policy, &audit_input).expect("remediated audit runs");
    assert_eq!(remediated.verdict(), Verdict::Green);
    assert_eq!(
        remediated.findings.len(),
        report.findings.len(),
        "remediation resolves the verdict, never erases the evidence"
    );
}
