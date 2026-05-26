//! `oya gate validate changeset-state-monotonicity` and
//! `oya gate validate changeset-state-enum-closed` runners.
//!
//! Both gates read `registry/vcs/changeset-event-log.json`, which is the
//! canonical event-sourced log defined in ADR-0110. The log has shape:
//!
//! ```json
//! { "events": [ { "changeset_id": "...", "from_state": "...", "to_state": "...", ... } ] }
//! ```
//!
//! ## `changeset-state-monotonicity`
//!
//! For every distinct `changeset_id` the sequence of `to_state` values in log
//! order MUST be a non-decreasing subsequence of the canonical advancing-state
//! ordering defined by ADR-0110:
//!
//!   opened → working → verified → pr_open → ci_running → ci_passed →
//!   reviewed → merged_dev → staged → produced
//!
//! A changeset MAY end at one of the three terminal-fail states (`abandoned`,
//! `rejected`, `cost_exhausted`) after any advancing state. Once a terminal
//! state is recorded, no further transitions may follow. A backwards move in
//! advancing state, or any transition after a terminal state, is a violation.
//!
//! An empty event log (no changesets) is a valid vacuous-green state per
//! ADR-0221 §M-06 — the gate reports 0 changesets, 0 violations and exits
//! SUCCESS.
//!
//! ## `changeset-state-enum-closed`
//!
//! Every `to_state` value in the log MUST be a member of the 12-value closed
//! enum defined in ADR-0110 (9 advancing + 3 terminal-fail). Any unrecognised
//! `to_state` is a violation.
//!
//! The same vacuous-green semantics apply when the log is empty.
//!
//! Lane IDs: `oya-governance-changeset-state-monotonicity` and
//! `oya-governance-changeset-state-enum-closed`. Both are Tier 1 kernel-tier
//! per ADR-0083: pure-domain parse + validate, no subprocess, no network, no
//! panics outside `cfg(test)`.

use std::fs;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Closed enum (ADR-0110 §The 9 states + §Plus 3 terminal-fail states)
// ─────────────────────────────────────────────────────────────────────────────

/// The 9 advancing states in monotonic order. Index position encodes ordinal
/// rank: a valid transition satisfies `rank(to) >= rank(from)`.
const ADVANCING_STATES: &[&str] = &[
    "opened",
    "working",
    "verified",
    "pr_open",
    "ci_running",
    "ci_passed",
    "reviewed",
    "merged_dev",
    "staged",
    "produced",
];

/// The 3 terminal-fail states. Once reached, no further transitions are valid.
const TERMINAL_FAIL_STATES: &[&str] = &["abandoned", "rejected", "cost_exhausted"];

fn advancing_rank(state: &str) -> Option<usize> {
    ADVANCING_STATES.iter().position(|&s| s == state)
}

fn is_terminal_fail(state: &str) -> bool {
    TERMINAL_FAIL_STATES.contains(&state)
}

fn is_known_state(state: &str) -> bool {
    advancing_rank(state).is_some() || is_terminal_fail(state)
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared report structs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct ChangesetStateMonotonicityReport {
    pub events_checked: usize,
    pub changesets_checked: usize,
    pub violations: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ChangesetStateEnumClosedReport {
    pub events_checked: usize,
    pub violations: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared args
// ─────────────────────────────────────────────────────────────────────────────

const DEFAULT_EVENT_LOG: &str = "registry/vcs/changeset-event-log.json";

const MONOTONICITY_USAGE: &str = "oya gate validate changeset-state-monotonicity \
                                   [--event-log <registry/vcs/changeset-event-log.json>]";

const ENUM_CLOSED_USAGE: &str = "oya gate validate changeset-state-enum-closed \
                                  [--event-log <registry/vcs/changeset-event-log.json>]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangesetStateValidateArgs {
    pub event_log_path: PathBuf,
}

impl Default for ChangesetStateValidateArgs {
    fn default() -> Self {
        Self {
            event_log_path: PathBuf::from(DEFAULT_EVENT_LOG),
        }
    }
}

pub(crate) fn parse_changeset_state_monotonicity_args(
    args: Vec<String>,
) -> Result<ChangesetStateValidateArgs, String> {
    parse_changeset_state_args(args, MONOTONICITY_USAGE)
}

pub(crate) fn parse_changeset_state_enum_closed_args(
    args: Vec<String>,
) -> Result<ChangesetStateValidateArgs, String> {
    parse_changeset_state_args(args, ENUM_CLOSED_USAGE)
}

fn parse_changeset_state_args(
    args: Vec<String>,
    usage: &str,
) -> Result<ChangesetStateValidateArgs, String> {
    let mut parsed = ChangesetStateValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--event-log" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.event_log_path = PathBuf::from(value);
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON parsing helpers
// ─────────────────────────────────────────────────────────────────────────────

/// A single event row extracted from the log.
struct EventRow {
    changeset_id: String,
    to_state: String,
}

fn read_event_log(path: &std::path::Path) -> Result<Vec<EventRow>, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "could not read changeset event log at {}: {error}",
            path.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "changeset event log at {} is invalid JSON: {error}",
            path.display()
        )
    })?;
    let events = value
        .get("events")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            format!(
                "changeset event log at {} must have a top-level `events` array",
                path.display()
            )
        })?;

    let mut rows = Vec::with_capacity(events.len());
    for (i, event) in events.iter().enumerate() {
        let changeset_id = event
            .get("changeset_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "event[{i}] in {} is missing `changeset_id` string field",
                    path.display()
                )
            })?
            .to_owned();
        let to_state = event
            .get("to_state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "event[{i}] in {} is missing `to_state` string field",
                    path.display()
                )
            })?
            .to_owned();
        rows.push(EventRow {
            changeset_id,
            to_state,
        });
    }
    Ok(rows)
}

// ─────────────────────────────────────────────────────────────────────────────
// Gate: changeset-state-monotonicity
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn validate_changeset_state_monotonicity(
    args: ChangesetStateValidateArgs,
) -> Result<ChangesetStateMonotonicityReport, String> {
    let rows = read_event_log(&args.event_log_path)?;
    let events_checked = rows.len();

    // Group events per changeset in log order (stable: Vec preserves insertion
    // order; BTreeMap gives deterministic violation messages).
    let mut per_changeset: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in rows {
        per_changeset
            .entry(row.changeset_id)
            .or_default()
            .push(row.to_state);
    }

    let changesets_checked = per_changeset.len();
    let mut violations = Vec::new();

    for (id, states) in &per_changeset {
        let mut last_advancing_rank: Option<usize> = None;
        let mut terminal_seen = false;

        for state in states {
            if terminal_seen {
                // Any event after a terminal state is a violation.
                violations.push(format!(
                    "changeset {id}: state `{state}` follows a terminal-fail state (no \
                     transitions are valid after a terminal state)"
                ));
                continue;
            }

            if is_terminal_fail(state) {
                terminal_seen = true;
                continue;
            }

            match advancing_rank(state) {
                Some(rank) => {
                    if let Some(prev) = last_advancing_rank
                        && rank < prev
                    {
                        violations.push(format!(
                            "changeset {id}: state `{state}` (rank {rank}) is a backwards \
                             move from rank {prev} — monotonicity violated"
                        ));
                    }
                    last_advancing_rank = Some(rank);
                }
                None => {
                    // Unknown state: the enum-closed gate catches this; the
                    // monotonicity gate skips unknown states rather than
                    // double-reporting.
                }
            }
        }
    }

    Ok(ChangesetStateMonotonicityReport {
        events_checked,
        changesets_checked,
        violations,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Gate: changeset-state-enum-closed
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn validate_changeset_state_enum_closed(
    args: ChangesetStateValidateArgs,
) -> Result<ChangesetStateEnumClosedReport, String> {
    let rows = read_event_log(&args.event_log_path)?;
    let events_checked = rows.len();
    let mut violations = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        if !is_known_state(&row.to_state) {
            violations.push(format!(
                "event[{i}] changeset {}: `to_state` = `{}` is not a member of the \
                 ADR-0110 closed state enum (known advancing: {}; known terminal-fail: {})",
                row.changeset_id,
                row.to_state,
                ADVANCING_STATES.join(", "),
                TERMINAL_FAIL_STATES.join(", "),
            ));
        }
    }

    Ok(ChangesetStateEnumClosedReport {
        events_checked,
        violations,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests (ADR-0083 Tier 3 exemption)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn write_event_log(dir: &Path, json: &str) -> PathBuf {
        let path = dir.join("changeset-event-log.json");
        fs::write(&path, json).expect("write event log");
        path
    }

    fn tmp_dir(suffix: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("oya-changeset-state-{suffix}"));
        fs::create_dir_all(&base).expect("tmp dir");
        base
    }

    // ── Monotonicity ──────────────────────────────────────────────────────────

    #[test]
    fn monotonicity_passes_on_empty_log() {
        let dir = tmp_dir("mono-empty");
        let path = write_event_log(&dir, r#"{"events":[]}"#);
        let args = ChangesetStateValidateArgs {
            event_log_path: path,
        };
        let report = validate_changeset_state_monotonicity(args).expect("ok");
        assert_eq!(report.events_checked, 0);
        assert_eq!(report.changesets_checked, 0);
        assert!(report.violations.is_empty());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn monotonicity_passes_for_valid_progression() {
        let dir = tmp_dir("mono-valid");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_1","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_1","from_state":"opened","to_state":"working"},
            {"changeset_id":"cs_1","from_state":"working","to_state":"verified"},
            {"changeset_id":"cs_1","from_state":"verified","to_state":"pr_open"},
            {"changeset_id":"cs_1","from_state":"pr_open","to_state":"ci_passed"},
            {"changeset_id":"cs_1","from_state":"ci_passed","to_state":"reviewed"},
            {"changeset_id":"cs_1","from_state":"reviewed","to_state":"merged_dev"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_monotonicity(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert_eq!(report.events_checked, 7);
        assert_eq!(report.changesets_checked, 1);
        assert!(report.violations.is_empty(), "{:?}", report.violations);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn monotonicity_passes_when_changeset_ends_at_terminal_fail() {
        let dir = tmp_dir("mono-terminal");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_2","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_2","from_state":"opened","to_state":"pr_open"},
            {"changeset_id":"cs_2","from_state":"pr_open","to_state":"rejected"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_monotonicity(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert!(report.violations.is_empty(), "{:?}", report.violations);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn monotonicity_rejects_backwards_move() {
        let dir = tmp_dir("mono-backwards");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_3","from_state":"","to_state":"reviewed"},
            {"changeset_id":"cs_3","from_state":"reviewed","to_state":"pr_open"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_monotonicity(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert!(
            report.violations[0].contains("backwards"),
            "{}",
            report.violations[0]
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn monotonicity_rejects_transition_after_terminal() {
        let dir = tmp_dir("mono-post-terminal");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_4","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_4","from_state":"opened","to_state":"abandoned"},
            {"changeset_id":"cs_4","from_state":"abandoned","to_state":"working"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_monotonicity(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert!(
            report.violations[0].contains("follows a terminal-fail state"),
            "{}",
            report.violations[0]
        );
        fs::remove_dir_all(dir).ok();
    }

    // ── Enum-closed ───────────────────────────────────────────────────────────

    #[test]
    fn enum_closed_passes_on_empty_log() {
        let dir = tmp_dir("enum-empty");
        let path = write_event_log(&dir, r#"{"events":[]}"#);
        let report = validate_changeset_state_enum_closed(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert_eq!(report.events_checked, 0);
        assert!(report.violations.is_empty());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn enum_closed_passes_for_all_known_states() {
        let dir = tmp_dir("enum-all-known");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_5","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_5","from_state":"opened","to_state":"working"},
            {"changeset_id":"cs_5","from_state":"working","to_state":"verified"},
            {"changeset_id":"cs_5","from_state":"verified","to_state":"pr_open"},
            {"changeset_id":"cs_5","from_state":"pr_open","to_state":"ci_running"},
            {"changeset_id":"cs_5","from_state":"ci_running","to_state":"ci_passed"},
            {"changeset_id":"cs_5","from_state":"ci_passed","to_state":"reviewed"},
            {"changeset_id":"cs_5","from_state":"reviewed","to_state":"merged_dev"},
            {"changeset_id":"cs_5","from_state":"merged_dev","to_state":"staged"},
            {"changeset_id":"cs_5","from_state":"staged","to_state":"produced"},
            {"changeset_id":"cs_6","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_6","from_state":"opened","to_state":"abandoned"},
            {"changeset_id":"cs_7","from_state":"","to_state":"pr_open"},
            {"changeset_id":"cs_7","from_state":"pr_open","to_state":"rejected"},
            {"changeset_id":"cs_8","from_state":"","to_state":"ci_running"},
            {"changeset_id":"cs_8","from_state":"ci_running","to_state":"cost_exhausted"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_enum_closed(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert!(report.violations.is_empty(), "{:?}", report.violations);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn enum_closed_rejects_unknown_state() {
        let dir = tmp_dir("enum-unknown");
        let log = r#"{
          "events": [
            {"changeset_id":"cs_9","from_state":"","to_state":"opened"},
            {"changeset_id":"cs_9","from_state":"opened","to_state":"BOGUS_STATE"}
          ]
        }"#;
        let path = write_event_log(&dir, log);
        let report = validate_changeset_state_enum_closed(ChangesetStateValidateArgs {
            event_log_path: path,
        })
        .expect("ok");
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert!(
            report.violations[0].contains("BOGUS_STATE"),
            "{}",
            report.violations[0]
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn parse_accepts_custom_event_log_path() {
        let args = parse_changeset_state_monotonicity_args(vec![
            "--event-log".to_string(),
            "custom/path.json".to_string(),
        ])
        .expect("parse ok");
        assert_eq!(
            args.event_log_path,
            std::path::PathBuf::from("custom/path.json")
        );
    }

    #[test]
    fn parse_uses_canonical_default_path() {
        let args = parse_changeset_state_monotonicity_args(Vec::new()).expect("parse ok");
        assert_eq!(
            args.event_log_path,
            std::path::PathBuf::from(DEFAULT_EVENT_LOG)
        );
    }
}
