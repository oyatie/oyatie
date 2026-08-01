// Typed terminal state acceptance proof, driven by CAPTURED REAL INCIDENTS.
//
// The fixtures under specs/fixtures/cloud-ci-run-terminal-state/ are unmodified projections of the
// live GitHub Actions API (`/actions/runs/<id>` + `/actions/runs/<id>/jobs` +
// `/check-runs/<job>/annotations`) for two runs that actually happened in this repository. Nothing
// in them is synthesized: if the classifier can separate these, it can separate the incidents an
// operator is currently forced to separate by hand.
//
//   incident 1 — run 30677213867, job 91306848257 `gate · affected-set`
//                42 minutes, conclusion `failure`, ZERO steps with a failure conclusion, step 9
//                still `in_progress`, steps 10-14 `pending`, log 404 (BlobNotFound), and a
//                check-run annotation "The self-hosted runner lost communication with the server".
//                => no-verdict / retry
//   incident 2 — run 30692655896, job 91361747751 `matrix.label`
//                `skipped` with zero steps because `needs: producer-regen` failed. The entire gate
//                matrix never evaluated the candidate.
//                => blocked / wait
//   incident 3 — run 30692655896, job 91359470200 `freshness`
//                conclusion `failure` with step "Run freshness gate" at conclusion `failure`.
//                A gate ran, observed the candidate, and rejected it.
//                => fail / fix-candidate
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_baseline_ratchet::run_terminal_state::{
    NextAction, SubjectCounts, TerminalState, classify_lane, classify_run,
};
use serde_json::Value;

const OBSERVED_AT: &str = "2026-08-01T16:00:00Z";

const RUNNER_LOST: u64 = 91306848257;
const SKIPPED_MATRIX: u64 = 91361747751;
const FRESHNESS_FAIL: u64 = 91359470200;

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

fn fixture(name: &str) -> Value {
    let path: PathBuf = repo_root()
        .join("specs/fixtures/cloud-ci-run-terminal-state")
        .join(name);
    read_json(&path)
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn runner_lost_fixture() -> Value {
    fixture("incident-runner-lost-communication.json")
}

fn wedge_fixture() -> Value {
    fixture("incident-producer-regen-wedge.json")
}

/// Pull one real job out of a fixture and classify it exactly as the adapter would.
fn classify_job_from(payload: &Value, job_id: u64) -> ci_baseline_ratchet::run_terminal_state::LaneVerdict {
    let run = payload.get("run").expect("run");
    let attempt = run.get("run_attempt").and_then(Value::as_u64).unwrap_or(1);
    let created = run.get("created_at").and_then(Value::as_str);
    let job = payload
        .get("jobs")
        .and_then(Value::as_array)
        .expect("jobs")
        .iter()
        .find(|j| j.get("id").and_then(Value::as_u64) == Some(job_id))
        .unwrap_or_else(|| panic!("job {job_id} missing from fixture"));
    let annotations: Vec<String> = payload
        .get("annotations")
        .and_then(|m| m.get(job_id.to_string()))
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .filter_map(|a| a.get("message").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    classify_lane(job, &annotations, attempt, created, None)
}

// ── THE HEADLINE PROOF ────────────────────────────────────────────────────────────────────────
// Three real incidents, three different states, three different permitted actions.

#[test]
fn the_three_real_incidents_classify_differently() {
    let no_verdict = classify_job_from(&runner_lost_fixture(), RUNNER_LOST);
    let wedge = wedge_fixture();
    let blocked = classify_job_from(&wedge, SKIPPED_MATRIX);
    let failed = classify_job_from(&wedge, FRESHNESS_FAIL);

    assert_eq!(no_verdict.state, TerminalState::NoVerdict, "{no_verdict:?}");
    assert_eq!(blocked.state, TerminalState::Blocked, "{blocked:?}");
    assert_eq!(failed.state, TerminalState::Fail, "{failed:?}");

    let states = [no_verdict.state, blocked.state, failed.state];
    let mut unique = states.to_vec();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 3, "the three incidents must not collapse");

    let actions = [
        no_verdict.next_action,
        blocked.next_action,
        failed.next_action,
    ];
    let mut unique_actions = actions.to_vec();
    unique_actions.sort();
    unique_actions.dedup();
    assert_eq!(
        unique_actions.len(),
        3,
        "each incident must permit a different next action, got {actions:?}"
    );
    assert_eq!(no_verdict.next_action, NextAction::Retry);
    assert_eq!(blocked.next_action, NextAction::Wait);
    assert_eq!(failed.next_action, NextAction::FixCandidate);
}

// ── RED: today's fan-in logic cannot tell them apart ──────────────────────────────────────────

/// Exactly the predicate the `oya-ci-required` fan-in uses today: green IFF every lane's result is
/// `success`, and everything else is one undifferentiated RED — its own message says "failed **or
/// was skipped**". Reproduced here so the defect is a runnable exhibit rather than an assertion
/// about code that lives in YAML.
fn todays_fan_in_verdict(job: &Value) -> &'static str {
    match job.get("conclusion").and_then(Value::as_str) {
        Some("success") => "GREEN",
        _ => "RED",
    }
}

#[test]
fn todays_fan_in_collapses_all_three_incidents_into_one_red() {
    let wedge = wedge_fixture();
    let runner_lost = runner_lost_fixture();
    let pick = |payload: &Value, id: u64| -> &'static str {
        let job = payload
            .get("jobs")
            .and_then(Value::as_array)
            .expect("jobs")
            .iter()
            .find(|j| j.get("id").and_then(Value::as_u64) == Some(id))
            .expect("job");
        todays_fan_in_verdict(job)
    };

    // The defect, proven on real data: one verdict for three unrelated events.
    assert_eq!(pick(&runner_lost, RUNNER_LOST), "RED");
    assert_eq!(pick(&wedge, SKIPPED_MATRIX), "RED");
    assert_eq!(pick(&wedge, FRESHNESS_FAIL), "RED");
}

// ── The discriminator itself ──────────────────────────────────────────────────────────────────

#[test]
fn no_verdict_is_derived_without_reading_any_log() {
    let v = classify_job_from(&runner_lost_fixture(), RUNNER_LOST);
    // The whole separation rests on these two typed facts, both from the jobs API.
    assert!(
        v.failed_steps.is_empty(),
        "incident 1 had zero failed steps, got {:?}",
        v.failed_steps
    );
    assert_eq!(
        v.unfinished_steps.len(),
        7,
        "incident 1 left 7 steps without a conclusion, got {:?}",
        v.unfinished_steps
    );
    assert!(
        v.because.contains("ZERO steps recorded a failure verdict"),
        "{}",
        v.because
    );
}

#[test]
fn the_runner_death_annotation_is_carried_as_evidence() {
    let v = classify_job_from(&runner_lost_fixture(), RUNNER_LOST);
    assert!(
        v.annotations
            .iter()
            .any(|a| a.contains("lost communication with the server")),
        "the only human-readable evidence for incident 1 must be carried: {:?}",
        v.annotations
    );
}

/// Annotations corroborate; they must never be load-bearing. In incident 1 the log 404'd, and an
/// annotation can be absent for the same reason — the state must still be no-verdict.
#[test]
fn classification_holds_when_annotations_are_absent() {
    let payload = runner_lost_fixture();
    let job = payload
        .get("jobs")
        .and_then(Value::as_array)
        .expect("jobs")
        .iter()
        .find(|j| j.get("id").and_then(Value::as_u64) == Some(RUNNER_LOST))
        .expect("job");
    let v = classify_lane(job, &[], 1, None, None);
    assert_eq!(v.state, TerminalState::NoVerdict);
}

#[test]
fn the_genuine_failure_names_the_step_that_rejected_the_candidate() {
    let v = classify_job_from(&wedge_fixture(), FRESHNESS_FAIL);
    assert_eq!(v.failed_steps, vec!["Run freshness gate".to_owned()]);
}

// ── Run-level rollup on the real runs ─────────────────────────────────────────────────────────

/// The sharpest real exhibit: ONE run, 56 lanes, carrying BOTH failure modes at once.
///
/// Run 30677213867 had five lanes that genuinely failed (each with a step at
/// `conclusion: failure`) and one lane — `gate · affected-set` — that was red having observed
/// nothing at all. Today both look identical in the UI. An operator reading the affected-set red
/// would go hunting for a defect in their candidate that this run never found.
#[test]
fn one_real_run_separates_a_no_verdict_lane_from_five_genuine_failures() {
    let r = classify_run(&runner_lost_fixture(), OBSERVED_AT, &SubjectCounts::new());
    assert_eq!(r.lanes.len(), 56, "fixture is the full real run");

    let tally = r.tally();
    assert_eq!(tally.get("fail"), Some(&5), "tally {tally:?}");
    assert_eq!(tally.get("no-verdict"), Some(&1), "tally {tally:?}");
    assert_eq!(tally.get("pass"), Some(&50), "tally {tally:?}");

    // The no-verdict lane is exactly the one whose runner died — and it is NOT among the failures.
    let no_verdict: Vec<u64> = r
        .lanes
        .iter()
        .filter(|l| l.state == TerminalState::NoVerdict)
        .map(|l| l.job_id)
        .collect();
    assert_eq!(no_verdict, vec![RUNNER_LOST]);

    // A real violation was observed elsewhere in this run, so the run's own action is to fix the
    // candidate. The per-lane detail is what stops the operator chasing the affected-set red.
    assert_eq!(r.state, TerminalState::Fail, "tally {tally:?}");
    assert_eq!(r.next_action, NextAction::FixCandidate);
}

#[test]
fn the_wedged_run_rolls_up_to_fail_not_blocked() {
    let r = classify_run(&wedge_fixture(), OBSERVED_AT, &SubjectCounts::new());
    // The run contains a skipped matrix lane, but a real observed violation outranks it: telling
    // the operator to "wait" here would be wrong.
    assert_eq!(r.state, TerminalState::Fail, "tally {:?}", r.tally());
    assert_eq!(r.next_action, NextAction::FixCandidate);
}

// ── Required evidence fields ──────────────────────────────────────────────────────────────────

#[test]
fn every_terminal_state_carries_its_required_evidence() {
    let r = classify_run(&runner_lost_fixture(), OBSERVED_AT, &SubjectCounts::new());
    assert_eq!(r.candidate_sha.len(), 40, "candidate SHA");
    assert_eq!(
        r.candidate_sha, "faee428489bd16e5a79d00eeb8ec15d14c0e18df",
        "the real head_sha of run 30677213867"
    );
    assert_eq!(r.run_id, "30677213867");
    assert_eq!(r.attempt, 1);
    assert_eq!(r.input_digest.len(), 64, "sha256 input digest");
    assert_eq!(
        r.classifier_version,
        ci_baseline_ratchet::run_terminal_state::CLASSIFIER_VERSION
    );
    assert_eq!(r.observed_at, OBSERVED_AT);

    let json = r.to_json();
    for key in [
        "state",
        "next_action",
        "green",
        "candidate_sha",
        "run_id",
        "attempt",
        "input_digest",
        "classifier_version",
        "observed_at",
        "lanes",
    ] {
        assert!(json.get(key).is_some(), "emitted artifact is missing {key}");
    }
    assert_eq!(json["green"], Value::Bool(false));
}

/// The digest binds a verdict to the exact bytes it came from.
#[test]
fn the_input_digest_changes_when_the_inputs_change() {
    let a = classify_run(&runner_lost_fixture(), OBSERVED_AT, &SubjectCounts::new());
    let mut mutated = runner_lost_fixture();
    mutated["run"]["head_sha"] = Value::String("0".repeat(40));
    let b = classify_run(&mutated, OBSERVED_AT, &SubjectCounts::new());
    assert_ne!(a.input_digest, b.input_digest);
}

// ── verified-empty keeps its RED semantics ────────────────────────────────────────────────────

#[test]
fn a_real_passing_lane_reporting_zero_subjects_goes_red() {
    let payload = wedge_fixture();
    // A lane that genuinely succeeded in the real run.
    let green_lane = payload
        .get("jobs")
        .and_then(Value::as_array)
        .expect("jobs")
        .iter()
        .find(|j| j.get("conclusion").and_then(Value::as_str) == Some("success"))
        .expect("at least one real successful lane");
    let name = green_lane
        .get("name")
        .and_then(Value::as_str)
        .expect("name")
        .to_owned();

    let clean = classify_lane(green_lane, &[], 1, None, None);
    assert_eq!(clean.state, TerminalState::Pass);
    assert!(clean.state.is_green());

    let mut subjects = SubjectCounts::new();
    subjects.insert(name, 0);
    let r = classify_run(&payload, OBSERVED_AT, &subjects);
    let empty = r
        .lanes
        .iter()
        .find(|l| l.state == TerminalState::VerifiedEmpty)
        .expect("the zero-subject lane must be reported verified-empty");
    assert_eq!(empty.next_action, NextAction::NeedsHuman);
    assert!(
        !empty.state.is_green(),
        "a lane that observed zero subjects must never be green"
    );
}
