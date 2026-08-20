//! Typed terminal state for a required-context run.
//!
//! WHY THIS EXISTS. Branch protection keys on one context (`oya-ci-required`). Today that context
//! collapses every outcome to exit 0 / exit 1, and the fan-in's own message says the quiet part out
//! loud: "RED — at least one constituent gate lane failed **or was skipped**". Those are not the
//! same event and they do not permit the same next action, but an operator cannot tell them apart
//! without scraping the Actions API by hand. Three measured incidents, all real:
//!
//!   1. job 91306848257 (`gate · affected-set`) reported `conclusion: failure` after 42 minutes
//!      with **zero steps carrying a failure conclusion** — step 9 was still `in_progress`
//!      (`conclusion: null`) and steps 10-14 were `pending`. `--log-failed` was empty and the log
//!      itself 404s (BlobNotFound): the runner died before uploading. The only real evidence was a
//!      check-run ANNOTATION, "The self-hosted runner lost communication with the server". No gate
//!      observed anything about the candidate. That is a NO-VERDICT, not a failure.
//!   2. job 91361747751 was `skipped` with **zero steps** because the lane it `needs:` failed. The
//!      candidate was never evaluated; nothing about it can be fixed from this signal.
//!   3. job 91359470200 (`freshness`) failed with step "Run freshness gate" at
//!      `conclusion: failure` — a gate that ran, observed the candidate, and rejected it.
//!
//! All three presented identically as a red required context. They demand `wait`, `fix-infra`, and
//! `fix-candidate` respectively.
//!
//! DERIVATION IS STRUCTURAL, NEVER LOG-GREPPING. The discriminator is the shape of the typed step
//! array — did any step actually record a failure verdict? — not the text of a log. Log text is
//! exactly what this replaces, and in incident 1 the log did not survive to be read. Check-run
//! annotations are carried as corroborating EVIDENCE and never as the primary discriminator, so
//! classification still holds when annotations are absent.
//!
//! This is a pure kernel: no I/O, no clock, no network. The adapter that fetches the run and jobs
//! from the Actions API lives in `src/bin/oya-cloud-ci-run-terminal-state.rs`.

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{Map, Value, json};

/// Bumped when the classification RULES change, so a recorded state is always attributable to the
/// logic that produced it. Carried in every emitted artifact.
pub const CLASSIFIER_VERSION: &str = "cloud-ci-run-terminal-state/v1";

/// Substring of the check-run annotation GitHub emits when a runner dies mid-job. Corroborating
/// evidence only — `NoVerdict` is derived structurally and does not depend on this matching.
const RUNNER_LOST_ANNOTATION: &str = "lost communication with the server";

/// The terminal state of one lane, or of the required context as a whole.
///
/// Deliberately five states. `retry` is NOT a state — it is one of the permitted next actions for
/// `NoVerdict`, and modelling it as a state would conflate "what happened" with "what to do".
/// `verified-noop` is folded into [`TerminalState::VerifiedEmpty`]: there is one underlying event
/// (the lane ran and observed nothing) and no typed signal distinguishes a further split, so a
/// second state would be a distinction the data cannot support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TerminalState {
    /// A verdict was reached and no gate observed a violation.
    Pass,
    /// A gate ran, observed the candidate, and recorded a violation against it.
    Fail,
    /// The run could not reach a verdict: the runner died, a step never completed, or the log was
    /// never uploaded. NOT a failure — nothing was observed about the candidate.
    NoVerdict,
    /// The lane never ran: still queued, or skipped because an upstream lane it `needs:` failed.
    Blocked,
    /// The lane ran and completed, but observed ZERO subjects.
    ///
    /// This is a CLAIM REQUIRING EVIDENCE, not a pass. A rule whose measured site count is 0 is
    /// enforcing nothing, and this repo's standing rule is that such a lane must be RED rather than
    /// "dormant and passing" — a lane that matched zero artifacts and evaluated clean is a
    /// false green. [`TerminalState::is_green`] therefore returns false here.
    VerifiedEmpty,
}

impl TerminalState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NoVerdict => "no-verdict",
            Self::Blocked => "blocked",
            Self::VerifiedEmpty => "verified-empty",
        }
    }

    /// Whether this state may admit a candidate. Only `Pass` is green: `VerifiedEmpty` is
    /// explicitly NOT green (see its doc comment), and the other three never observed a clean
    /// candidate at all.
    pub const fn is_green(self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Aggregation precedence when rolling lanes up into one required-context verdict, lowest
    /// first. An observed violation outranks everything: it is true about the candidate regardless
    /// of how much infrastructure noise surrounds it, and fixing it is what unblocks the run.
    /// `Blocked` ranks lowest because a lane that never ran tells you nothing, and in a wedged run
    /// (incident 2) it is the loudest and least informative signal — 44 skipped lanes behind one
    /// real failure must not drown out that failure.
    const fn precedence(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Blocked => 1,
            Self::NoVerdict => 2,
            Self::VerifiedEmpty => 3,
            Self::Fail => 4,
        }
    }
}

impl fmt::Display for TerminalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The single action a state permits. This is the whole point of the type: the operator reads the
/// action, not the logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NextAction {
    /// Nothing to do.
    Proceed,
    /// The candidate is at fault. Change the code.
    FixCandidate,
    /// Re-run as-is: a first-attempt infrastructure fault is usually transient.
    Retry,
    /// Retrying already failed, or the fault is systemic. Fix the fleet, not the candidate.
    FixInfra,
    /// Nothing is wrong with the candidate; the lane has not run yet or is behind a wedge.
    Wait,
    /// A judgement call no rule can make.
    NeedsHuman,
}

impl NextAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::FixCandidate => "fix-candidate",
            Self::Retry => "retry",
            Self::FixInfra => "fix-infra",
            Self::Wait => "wait",
            Self::NeedsHuman => "needs-human",
        }
    }
}

impl fmt::Display for NextAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lane's classification, with the evidence that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneVerdict {
    pub job_id: u64,
    pub job_name: String,
    pub state: TerminalState,
    pub next_action: NextAction,
    /// Why this state, in one line, naming the typed fields that decided it. Never a log excerpt.
    pub because: String,
    /// Steps that recorded `conclusion: failure` — the candidate-facing evidence.
    pub failed_steps: Vec<String>,
    /// Steps that never reached a conclusion (`in_progress`/`pending`/null) — the no-verdict
    /// evidence, and the field that separates incident 1 from incident 3.
    pub unfinished_steps: Vec<String>,
    /// Check-run annotations, carried as corroborating evidence.
    pub annotations: Vec<String>,
    /// Seconds the lane spent queued before starting, when both timestamps are present. Surfaces
    /// starvation even on lanes that eventually delivered a verdict.
    pub queue_delay_seconds: Option<i64>,
}

/// The required context's terminal state, plus every lane that contributed to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTerminalState {
    pub state: TerminalState,
    pub next_action: NextAction,
    /// Candidate SHA the verdict is about.
    pub candidate_sha: String,
    pub run_id: String,
    pub attempt: u64,
    /// sha256 over the canonicalized typed inputs, so a recorded verdict is bound to the exact
    /// bytes it was derived from.
    pub input_digest: String,
    pub classifier_version: String,
    /// RFC3339. Supplied by the caller — the kernel has no clock.
    pub observed_at: String,
    pub lanes: Vec<LaneVerdict>,
}

impl RunTerminalState {
    /// Counts per state, for the operator summary line.
    pub fn tally(&self) -> BTreeMap<&'static str, usize> {
        let mut out = BTreeMap::new();
        for lane in &self.lanes {
            *out.entry(lane.state.as_str()).or_insert(0) += 1;
        }
        out
    }

    pub fn to_json(&self) -> Value {
        let lanes: Vec<Value> = self
            .lanes
            .iter()
            .map(|lane| {
                json!({
                    "job_id": lane.job_id,
                    "job_name": lane.job_name,
                    "state": lane.state.as_str(),
                    "next_action": lane.next_action.as_str(),
                    "because": lane.because,
                    "failed_steps": lane.failed_steps,
                    "unfinished_steps": lane.unfinished_steps,
                    "annotations": lane.annotations,
                    "queue_delay_seconds": lane.queue_delay_seconds,
                })
            })
            .collect();
        let tally: Map<String, Value> = self
            .tally()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), json!(v)))
            .collect();
        json!({
            "schema_version": CLASSIFIER_VERSION,
            "state": self.state.as_str(),
            "next_action": self.next_action.as_str(),
            "green": self.state.is_green(),
            "candidate_sha": self.candidate_sha,
            "run_id": self.run_id,
            "attempt": self.attempt,
            "input_digest": self.input_digest,
            "classifier_version": self.classifier_version,
            "observed_at": self.observed_at,
            "tally": tally,
            "lanes": lanes,
        })
    }
}

/// Optional per-lane subject counts, keyed by job name.
///
/// The Actions API cannot report how many subjects a gate examined — only the gate knows. A lane
/// that reports 0 is [`TerminalState::VerifiedEmpty`]. Absent an entry, no such claim is made:
/// silence is not evidence of emptiness.
pub type SubjectCounts = BTreeMap<String, u64>;

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// Parse an RFC3339 timestamp to epoch seconds.
///
/// ponytail: hand-rolled rather than pulling in `chrono`/`time` for one subtraction. Handles the
/// `YYYY-MM-DDTHH:MM:SSZ` shape the Actions API emits; anything else yields `None`, which degrades
/// `queue_delay_seconds` to absent rather than to a wrong number. Upgrade to a date crate if this
/// ever needs timezones or sub-second precision.
fn epoch_seconds(ts: &str) -> Option<i64> {
    let bytes = ts.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' {
        return None;
    }
    let num = |a: usize, b: usize| ts.get(a..b)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, s) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    // Days from civil epoch (Howard Hinnant's algorithm; shifts the year to start in March so the
    // leap day lands last and needs no special case).
    let y = if mo <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (mo + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + h * 3600 + mi * 60 + s)
}

/// Classify ONE lane from its typed job object.
///
/// `run_attempt` decides `Retry` vs `FixInfra` for a no-verdict: a first attempt is presumed
/// transient, a repeat is presumed systemic — "retry harder" is not a remediation.
pub fn classify_lane(
    job: &Value,
    annotations: &[String],
    run_attempt: u64,
    run_created_at: Option<&str>,
    subjects_observed: Option<u64>,
) -> LaneVerdict {
    let job_id = job.get("id").and_then(Value::as_u64).unwrap_or(0);
    let job_name = str_field(job, "name").unwrap_or("<unnamed>").to_owned();
    let status = str_field(job, "status").unwrap_or("");
    let conclusion = str_field(job, "conclusion");

    let steps = job.get("steps").and_then(Value::as_array);
    let step_names = |pred: fn(Option<&str>) -> bool| -> Vec<String> {
        steps
            .map(|steps| {
                steps
                    .iter()
                    .filter(|s| pred(str_field(s, "conclusion")))
                    .filter_map(|s| str_field(s, "name").map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let failed_steps = step_names(|c| c == Some("failure"));
    // A step with no conclusion never reached a verdict: `in_progress` when the runner died, or
    // `pending` because it was queued behind that step.
    let unfinished_steps = step_names(|c| c.is_none());
    let started_any_step = steps.is_some_and(|steps| {
        steps
            .iter()
            .any(|s| str_field(s, "status") != Some("queued"))
    });

    let queue_delay_seconds = match (run_created_at, str_field(job, "started_at")) {
        (Some(created), Some(started)) => epoch_seconds(started)
            .zip(epoch_seconds(created))
            .map(|(a, b)| a - b),
        _ => None,
    };

    let runner_died = annotations
        .iter()
        .any(|a| a.contains(RUNNER_LOST_ANNOTATION));

    let (state, next_action, because) = if status != "completed" {
        (
            TerminalState::Blocked,
            NextAction::Wait,
            format!("lane has not completed (status `{status}`); no verdict exists yet"),
        )
    } else {
        match conclusion {
            Some("success") => match subjects_observed {
                // A lane that examined nothing enforces nothing. RED by standing rule.
                Some(0) => (
                    TerminalState::VerifiedEmpty,
                    NextAction::NeedsHuman,
                    "lane completed but reported 0 subjects observed; a rule with 0 measured sites \
                     enforces nothing and cannot be counted as a pass"
                        .to_owned(),
                ),
                Some(n) => (
                    TerminalState::Pass,
                    NextAction::Proceed,
                    format!("lane succeeded over {n} observed subject(s)"),
                ),
                None => (
                    TerminalState::Pass,
                    NextAction::Proceed,
                    "lane succeeded".to_owned(),
                ),
            },
            // Never ran. In a wedged run this is the upstream `needs:` deadlock: the candidate was
            // not evaluated, so there is nothing in it to fix.
            Some("skipped") => (
                TerminalState::Blocked,
                NextAction::Wait,
                "lane was skipped and ran no steps; an upstream lane it depends on did not \
                 succeed, so this lane never evaluated the candidate"
                    .to_owned(),
            ),
            // Superseded by a newer run, or cancelled by an operator. Not a statement about the
            // candidate either way.
            Some("cancelled") => (
                TerminalState::Blocked,
                NextAction::Wait,
                "lane was cancelled before reaching a verdict".to_owned(),
            ),
            Some("neutral" | "action_required") => (
                TerminalState::NoVerdict,
                NextAction::NeedsHuman,
                format!(
                    "lane concluded `{}`, which asserts nothing about the candidate",
                    conclusion.unwrap_or("")
                ),
            ),
            // THE DISCRIMINATOR. Red, but did any step actually record a verdict?
            Some(other) => {
                if !failed_steps.is_empty() {
                    (
                        TerminalState::Fail,
                        NextAction::FixCandidate,
                        format!(
                            "lane concluded `{other}` and {} step(s) recorded a failure verdict: {}",
                            failed_steps.len(),
                            failed_steps.join(", ")
                        ),
                    )
                } else if !started_any_step && unfinished_steps.is_empty() {
                    (
                        TerminalState::Blocked,
                        NextAction::Wait,
                        format!("lane concluded `{other}` without running any step"),
                    )
                } else {
                    let detail = if runner_died {
                        " and the runner reported losing communication with the server"
                    } else {
                        ""
                    };
                    (
                        TerminalState::NoVerdict,
                        // A repeat is systemic; retrying a second time is not a remediation.
                        if run_attempt > 1 {
                            NextAction::FixInfra
                        } else {
                            NextAction::Retry
                        },
                        format!(
                            "lane concluded `{other}` but ZERO steps recorded a failure verdict \
                             while {} step(s) never completed{detail}; no gate observed the \
                             candidate",
                            unfinished_steps.len()
                        ),
                    )
                }
            }
            None => (
                TerminalState::NoVerdict,
                NextAction::Retry,
                "lane completed with no conclusion at all".to_owned(),
            ),
        }
    };

    LaneVerdict {
        job_id,
        job_name,
        state,
        next_action,
        because,
        failed_steps,
        unfinished_steps,
        annotations: annotations.to_vec(),
        queue_delay_seconds,
    }
}

/// Classify a whole required-context run from the typed `{run, jobs, annotations}` payload the
/// adapter assembles from the Actions API.
///
/// `observed_at` is passed in rather than read from a clock so the kernel stays pure and every
/// classification is reproducible from its inputs.
pub fn classify_run(
    payload: &Value,
    observed_at: &str,
    subjects: &SubjectCounts,
) -> RunTerminalState {
    let run = payload.get("run").cloned().unwrap_or(Value::Null);
    let attempt = run.get("run_attempt").and_then(Value::as_u64).unwrap_or(1);
    let run_created_at = str_field(&run, "created_at").map(str::to_owned);
    let candidate_sha = str_field(&run, "head_sha").unwrap_or("").to_owned();
    let run_id = run
        .get("id")
        .map(|id| match id {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .unwrap_or_default();

    let annotations_map = payload.get("annotations");
    let lanes: Vec<LaneVerdict> = payload
        .get("jobs")
        .and_then(Value::as_array)
        .map(|jobs| {
            jobs.iter()
                .map(|job| {
                    let key = job
                        .get("id")
                        .map(|id| match id {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();
                    let annotations: Vec<String> = annotations_map
                        .and_then(|m| m.get(&key))
                        .and_then(Value::as_array)
                        .map(|list| {
                            list.iter()
                                .filter_map(|a| str_field(a, "message").map(str::to_owned))
                                .collect()
                        })
                        .unwrap_or_default();
                    let subjects_observed =
                        str_field(job, "name").and_then(|n| subjects.get(n).copied());
                    classify_lane(
                        job,
                        &annotations,
                        attempt,
                        run_created_at.as_deref(),
                        subjects_observed,
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    // Roll up by precedence, then adopt the next action of a lane actually holding that state, so
    // the action always points at a real lane rather than at a recomputed guess.
    let state = lanes
        .iter()
        .map(|l| l.state)
        .max_by_key(|s| s.precedence())
        .unwrap_or(TerminalState::Blocked);
    let next_action = lanes
        .iter()
        .filter(|l| l.state == state)
        .map(|l| l.next_action)
        .max()
        .unwrap_or(NextAction::Wait);

    RunTerminalState {
        state,
        next_action,
        candidate_sha,
        run_id,
        attempt,
        input_digest: input_digest(payload),
        classifier_version: CLASSIFIER_VERSION.to_owned(),
        observed_at: observed_at.to_owned(),
        lanes,
    }
}

/// sha256 over the canonicalized typed inputs.
///
/// `serde_json`'s default `Map` is a `BTreeMap`, so re-serializing sorts keys and makes the digest
/// stable against key-order churn from the API.
pub fn input_digest(payload: &Value) -> String {
    use sha2::{Digest, Sha256};
    let canonical = serde_json::to_vec(payload).unwrap_or_default();
    format!("{:x}", Sha256::digest(&canonical))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(conclusion: &str, steps: Value) -> Value {
        json!({
            "id": 1, "name": "lane", "status": "completed",
            "conclusion": conclusion, "steps": steps
        })
    }

    fn step(name: &str, status: &str, conclusion: Value) -> Value {
        json!({"number": 1, "name": name, "status": status, "conclusion": conclusion})
    }

    #[test]
    fn failure_with_a_failed_step_is_fail_and_fix_candidate() {
        let j = job(
            "failure",
            json!([step("Run freshness gate", "completed", json!("failure"))]),
        );
        let v = classify_lane(&j, &[], 1, None, None);
        assert_eq!(v.state, TerminalState::Fail);
        assert_eq!(v.next_action, NextAction::FixCandidate);
    }

    /// The incident-1 shape: red, but nothing observed the candidate.
    #[test]
    fn failure_with_no_failed_step_is_no_verdict_not_fail() {
        let j = job(
            "failure",
            json!([
                step("ok", "completed", json!("success")),
                step("stuck", "in_progress", Value::Null),
                step("never ran", "pending", Value::Null)
            ]),
        );
        let v = classify_lane(&j, &[], 1, None, None);
        assert_eq!(v.state, TerminalState::NoVerdict);
        assert_eq!(v.next_action, NextAction::Retry);
        assert_eq!(v.unfinished_steps.len(), 2);
    }

    #[test]
    fn a_repeat_no_verdict_escalates_from_retry_to_fix_infra() {
        let j = job(
            "failure",
            json!([step("stuck", "in_progress", Value::Null)]),
        );
        assert_eq!(
            classify_lane(&j, &[], 2, None, None).next_action,
            NextAction::FixInfra
        );
    }

    #[test]
    fn skipped_lane_is_blocked_and_wait() {
        let v = classify_lane(&job("skipped", json!([])), &[], 1, None, None);
        assert_eq!(v.state, TerminalState::Blocked);
        assert_eq!(v.next_action, NextAction::Wait);
    }

    #[test]
    fn queued_lane_is_blocked() {
        let j = json!({"id": 1, "name": "lane", "status": "queued", "conclusion": Value::Null});
        assert_eq!(
            classify_lane(&j, &[], 1, None, None).state,
            TerminalState::Blocked
        );
    }

    /// Zero observed subjects is RED, never a quiet pass.
    #[test]
    fn zero_subjects_is_verified_empty_and_not_green() {
        let j = job(
            "success",
            json!([step("ok", "completed", json!("success"))]),
        );
        let v = classify_lane(&j, &[], 1, None, Some(0));
        assert_eq!(v.state, TerminalState::VerifiedEmpty);
        assert_eq!(v.next_action, NextAction::NeedsHuman);
        assert!(!v.state.is_green());
    }

    #[test]
    fn a_success_with_subjects_is_a_plain_pass() {
        let j = job(
            "success",
            json!([step("ok", "completed", json!("success"))]),
        );
        assert_eq!(
            classify_lane(&j, &[], 1, None, Some(12)).state,
            TerminalState::Pass
        );
    }

    /// An observed violation must survive being surrounded by skipped lanes.
    #[test]
    fn fail_outranks_blocked_in_a_wedged_run() {
        let payload = json!({
            "run": {"id": 7, "run_attempt": 1, "head_sha": "a".repeat(40), "created_at": "2026-08-01T00:00:00Z"},
            "jobs": [
                job("skipped", json!([])),
                json!({"id": 2, "name": "real", "status": "completed", "conclusion": "failure",
                       "steps": [step("gate", "completed", json!("failure"))]})
            ],
            "annotations": {}
        });
        let r = classify_run(&payload, "2026-08-01T00:00:00Z", &SubjectCounts::new());
        assert_eq!(r.state, TerminalState::Fail);
        assert_eq!(r.next_action, NextAction::FixCandidate);
    }

    #[test]
    fn epoch_seconds_parses_actions_timestamps_and_rejects_junk() {
        assert_eq!(epoch_seconds("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(epoch_seconds("2026-08-01T01:13:32Z"), Some(1_785_546_812));
        assert_eq!(epoch_seconds("not-a-timestamp"), None);
        assert_eq!(epoch_seconds("2026-13-01T00:00:00Z"), None);
    }

    #[test]
    fn queue_delay_is_measured_from_run_creation() {
        let mut j = job("success", json!([]));
        j["started_at"] = json!("2026-08-01T01:00:00Z");
        let v = classify_lane(&j, &[], 1, Some("2026-08-01T00:20:00Z"), None);
        assert_eq!(v.queue_delay_seconds, Some(2400));
    }
}
