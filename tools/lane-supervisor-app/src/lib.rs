#![forbid(unsafe_code)]

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneSupervisorErrorKind {
    Parse,
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneSupervisorError {
    kind: LaneSupervisorErrorKind,
    message: String,
}

impl LaneSupervisorError {
    pub fn new(kind: LaneSupervisorErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> LaneSupervisorErrorKind {
        self.kind
    }
}

impl Display for LaneSupervisorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for LaneSupervisorError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRow {
    fields: Map<String, Value>,
}

impl LedgerRow {
    pub fn from_fields(fields: Map<String, Value>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(Value::as_str)
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.fields.get(key).and_then(Value::as_i64)
    }

    pub fn lane_id(&self) -> Option<&str> {
        self.get_str("lane_id")
    }

    pub fn status(&self) -> Option<&str> {
        self.get_str("status")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneSummary {
    pub lane_id: String,
    pub status: String,
    pub branch: Option<String>,
    pub brief: Option<String>,
    pub worktree: Option<String>,
    pub log: Option<String>,
    pub wait_file: Option<String>,
    pub pid: Option<i64>,
    pub at: Option<String>,
    pub latest_row: LedgerRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrPresence {
    pub number: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneObservation {
    pub process_alive: Option<bool>,
    pub log_mtime_unix_seconds: Option<i64>,
    pub wait_exit_status: Option<i64>,
    pub pr: Option<PrPresence>,
    pub pr_lookup_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReapOptions {
    pub stall_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReapDecision {
    None,
    PrOpen { number: i64 },
    Exited { exit_status: Option<i64> },
    Stalled { stale_seconds: i64 },
    Dead,
    Indeterminate { reason: String },
}

pub trait Clock {
    fn now_unix_seconds(&self) -> i64;
}

pub fn parse_jsonl(input: &str) -> Result<Vec<LedgerRow>, LaneSupervisorError> {
    let mut rows = Vec::new();
    for (index, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(trimmed).map_err(|err| {
            LaneSupervisorError::new(
                LaneSupervisorErrorKind::Parse,
                format!("ledger line {} is not valid JSON: {err}", index + 1),
            )
        })?;
        let Value::Object(fields) = value else {
            return Err(LaneSupervisorError::new(
                LaneSupervisorErrorKind::Parse,
                format!("ledger line {} is not a JSON object", index + 1),
            ));
        };
        rows.push(LedgerRow::from_fields(fields));
    }
    Ok(rows)
}

pub fn render_jsonl_row(row: &LedgerRow) -> Result<String, LaneSupervisorError> {
    serde_json::to_string(row.fields()).map_err(|err| {
        LaneSupervisorError::new(
            LaneSupervisorErrorKind::Parse,
            format!("failed to render ledger row: {err}"),
        )
    })
}

pub fn summarize_lanes(rows: &[LedgerRow]) -> BTreeMap<String, LaneSummary> {
    let mut summaries = BTreeMap::new();
    for row in rows {
        let Some(lane_id) = row.lane_id() else {
            continue;
        };
        let lane_id_owned = lane_id.to_owned();
        let previous = summaries.remove(&lane_id_owned);
        let summary = merge_summary(lane_id_owned.clone(), previous, row.clone());
        summaries.insert(lane_id_owned, summary);
    }
    summaries
}

pub fn is_terminal_status(status: &str) -> bool {
    status.starts_with("pr-open:")
        || matches!(
            status,
            "reviewed"
                | "merged"
                | "escalated"
                | "cancelled"
                | "failed"
                | "exited"
                | "dead"
                | "stalled"
        )
}

pub fn terminal_status_requires_failed_reap(status: &str) -> bool {
    matches!(status, "exited" | "dead" | "stalled" | "failed")
}

pub fn evaluate_reap<C: Clock>(
    lane: &LaneSummary,
    observation: &LaneObservation,
    options: ReapOptions,
    clock: &C,
) -> ReapDecision {
    if is_terminal_status(&lane.status) {
        return ReapDecision::None;
    }

    if let Some(reason) = &observation.pr_lookup_error {
        return ReapDecision::Indeterminate {
            reason: reason.clone(),
        };
    }

    if let Some(pr) = &observation.pr {
        return ReapDecision::PrOpen { number: pr.number };
    }

    if observation.wait_exit_status.is_some() {
        return ReapDecision::Exited {
            exit_status: observation.wait_exit_status,
        };
    }

    if lane.pid.is_none() {
        if lane.status == "dispatching" {
            let Some(at) = lane.at.as_deref() else {
                return ReapDecision::Dead;
            };
            let Some(started_at) = parse_rfc3339_unix_seconds(at) else {
                return ReapDecision::Dead;
            };
            let stale_seconds = clock.now_unix_seconds().saturating_sub(started_at);
            if stale_seconds >= options.stall_seconds {
                return ReapDecision::Stalled { stale_seconds };
            }
            return ReapDecision::None;
        }
        if is_dispatched_status(&lane.status) {
            return ReapDecision::Dead;
        }
    }

    if observation.process_alive == Some(true) {
        if let Some(log_mtime) = observation.log_mtime_unix_seconds {
            let stale_seconds = clock.now_unix_seconds().saturating_sub(log_mtime);
            if stale_seconds >= options.stall_seconds {
                return ReapDecision::Stalled { stale_seconds };
            }
        }
        return ReapDecision::None;
    }

    if observation.process_alive == Some(false) || lane.pid.is_some() {
        return ReapDecision::Dead;
    }

    ReapDecision::None
}

fn is_dispatched_status(status: &str) -> bool {
    status == "dispatched" || status.ends_with("-dispatched")
}

pub fn is_unhealthy_reap_decision(decision: &ReapDecision) -> bool {
    matches!(
        decision,
        ReapDecision::Exited { .. }
            | ReapDecision::Stalled { .. }
            | ReapDecision::Dead
            | ReapDecision::Indeterminate { .. }
    )
}

pub fn event_row_for_decision(
    lane: &LaneSummary,
    decision: &ReapDecision,
    at: String,
) -> Option<LedgerRow> {
    let mut fields = Map::new();
    fields.insert("lane_id".to_owned(), Value::String(lane.lane_id.clone()));
    fields.insert("at".to_owned(), Value::String(at));

    match decision {
        ReapDecision::None => None,
        ReapDecision::PrOpen { number } => {
            fields.insert(
                "status".to_owned(),
                Value::String(format!("pr-open:{number}")),
            );
            fields.insert("pr_number".to_owned(), Value::Number((*number).into()));
            Some(LedgerRow::from_fields(fields))
        }
        ReapDecision::Exited { exit_status } => {
            fields.insert("status".to_owned(), Value::String("exited".to_owned()));
            if let Some(status) = exit_status {
                fields.insert("exit_status".to_owned(), Value::Number((*status).into()));
            }
            Some(LedgerRow::from_fields(fields))
        }
        ReapDecision::Stalled { stale_seconds } => {
            fields.insert("status".to_owned(), Value::String("stalled".to_owned()));
            fields.insert(
                "stale_seconds".to_owned(),
                Value::Number((*stale_seconds).into()),
            );
            Some(LedgerRow::from_fields(fields))
        }
        ReapDecision::Dead => {
            fields.insert("status".to_owned(), Value::String("dead".to_owned()));
            Some(LedgerRow::from_fields(fields))
        }
        ReapDecision::Indeterminate { reason } => {
            fields.insert(
                "status".to_owned(),
                Value::String("indeterminate".to_owned()),
            );
            fields.insert("reason".to_owned(), Value::String(reason.clone()));
            Some(LedgerRow::from_fields(fields))
        }
    }
}

/// Named inputs shared by dispatch-registration and dispatched ledger rows.
pub struct DispatchRowInput<'a> {
    pub lane_id: &'a str,
    pub brief: &'a str,
    pub worktree: &'a str,
    pub branch: &'a str,
    pub base: &'a str,
    pub expected_hard_surfaces: &'a [String],
    pub expected_soft_surfaces: &'a [String],
    pub log: &'a str,
    pub wait_file: &'a str,
    pub start_file: &'a str,
    pub run_id: &'a str,
    pub at: String,
}

pub fn dispatch_row(input: DispatchRowInput<'_>, pid: u32) -> LedgerRow {
    let mut fields = dispatch_common_fields(input);
    fields.insert("status".to_owned(), Value::String("dispatched".to_owned()));
    fields.insert("pid".to_owned(), Value::Number(i64::from(pid).into()));
    fields.insert(
        "pid_role".to_owned(),
        Value::String("supervisor-wrapper".to_owned()),
    );
    fields.insert("detached_process_group".to_owned(), Value::Bool(true));
    LedgerRow::from_fields(fields)
}

pub fn dispatch_registration_row(input: DispatchRowInput<'_>) -> LedgerRow {
    let mut fields = dispatch_common_fields(input);
    fields.insert("status".to_owned(), Value::String("dispatching".to_owned()));
    LedgerRow::from_fields(fields)
}

fn dispatch_common_fields(input: DispatchRowInput<'_>) -> Map<String, Value> {
    let DispatchRowInput {
        lane_id,
        brief,
        worktree,
        branch,
        base,
        expected_hard_surfaces,
        expected_soft_surfaces,
        log,
        wait_file,
        start_file,
        run_id,
        at,
    } = input;
    let mut fields = Map::new();
    fields.insert("lane_id".to_owned(), Value::String(lane_id.to_owned()));
    fields.insert("brief".to_owned(), Value::String(brief.to_owned()));
    fields.insert("worktree".to_owned(), Value::String(worktree.to_owned()));
    fields.insert("branch".to_owned(), Value::String(branch.to_owned()));
    fields.insert("base".to_owned(), Value::String(base.to_owned()));
    fields.insert(
        "expected_surfaces".to_owned(),
        serde_json::json!({
            "hard": expected_hard_surfaces,
            "soft": expected_soft_surfaces,
        }),
    );
    fields.insert("at".to_owned(), Value::String(at.clone()));
    fields.insert("started_at".to_owned(), Value::String(at));
    fields.insert("log".to_owned(), Value::String(log.to_owned()));
    fields.insert("wait_file".to_owned(), Value::String(wait_file.to_owned()));
    fields.insert(
        "start_file".to_owned(),
        Value::String(start_file.to_owned()),
    );
    fields.insert("run_id".to_owned(), Value::String(run_id.to_owned()));
    fields
}

pub fn derive_lane_id(branch: &str, brief: &str) -> String {
    if let Some(branch_stem) = branch.strip_prefix("agent/")
        && !branch_stem.is_empty()
    {
        return branch_stem.replace('/', "-");
    }
    if !branch.is_empty() {
        return branch.replace('/', "-");
    }
    brief
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_prefix("BRIEF-"))
        .and_then(|name| name.strip_suffix(".md"))
        .map_or_else(|| "lane".to_owned(), ToOwned::to_owned)
}

pub fn prompt_from_brief_pointer(brief: &str) -> String {
    format!(
        "You are a lane worker already running inside the isolated worktree assigned by the dispatcher. Treat the brief's worktree-creation instruction as already satisfied; verify the current worktree path and branch before editing, and never modify the main checkout working tree. Read and execute {brief} exactly otherwise. Final output line must be PR_OPENED: <number> or BLOCKED: <reason>."
    )
}

pub fn iso8601_from_unix_seconds(seconds: i64) -> Result<String, LaneSupervisorError> {
    let Some(dt) = Utc.timestamp_opt(seconds, 0).single() else {
        return Err(LaneSupervisorError::new(
            LaneSupervisorErrorKind::InvalidInput,
            format!("invalid unix timestamp: {seconds}"),
        ));
    };
    Ok(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

pub fn unix_seconds_from_datetime(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

fn parse_rfc3339_unix_seconds(input: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(input)
        .ok()
        .map(|dt| dt.timestamp())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitFile {
    #[serde(default)]
    pub run_id: Option<String>,
    pub exit_status: i64,
    pub exited_at: String,
}

fn merge_summary(lane_id: String, previous: Option<LaneSummary>, row: LedgerRow) -> LaneSummary {
    let status = row
        .status()
        .map_or_else(|| previous_status(&previous), ToOwned::to_owned);
    LaneSummary {
        lane_id,
        status,
        branch: row
            .get_str("branch")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.branch.clone())),
        brief: row
            .get_str("brief")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.brief.clone())),
        worktree: row
            .get_str("worktree")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.worktree.clone())),
        log: row
            .get_str("log")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.log.clone())),
        wait_file: row
            .get_str("wait_file")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.wait_file.clone())),
        pid: row
            .get_i64("pid")
            .or_else(|| previous.as_ref().and_then(|item| item.pid)),
        at: row
            .get_str("at")
            .map(ToOwned::to_owned)
            .or_else(|| previous.as_ref().and_then(|item| item.at.clone())),
        latest_row: row,
    }
}

fn previous_status(previous: &Option<LaneSummary>) -> String {
    previous
        .as_ref()
        .map_or_else(|| "unknown".to_owned(), |item| item.status.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedClock {
        now: i64,
    }

    impl Clock for FixedClock {
        fn now_unix_seconds(&self) -> i64 {
            self.now
        }
    }

    fn lane(status: &str) -> LaneSummary {
        let row = LedgerRow::from_fields(Map::from_iter([
            ("lane_id".to_owned(), Value::String("L-1".to_owned())),
            ("status".to_owned(), Value::String(status.to_owned())),
            (
                "branch".to_owned(),
                Value::String("agent/example".to_owned()),
            ),
            ("pid".to_owned(), Value::Number(42.into())),
        ]));
        summarize_lanes(&[row])
            .remove("L-1")
            .expect("test lane should summarize")
    }

    #[test]
    fn parses_jsonl_and_preserves_unknown_fields() {
        let rows =
            parse_jsonl(r#"{"lane_id":"L-1","status":"dispatched","custom":{"nested":true}}"#)
                .expect("valid ledger row should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lane_id(), Some("L-1"));
        assert_eq!(
            rows[0].fields().get("custom"),
            Some(&serde_json::json!({"nested": true}))
        );
        assert_eq!(
            render_jsonl_row(&rows[0]).expect("row should render"),
            r#"{"lane_id":"L-1","status":"dispatched","custom":{"nested":true}}"#
        );
    }

    #[test]
    fn summarizes_latest_state_without_losing_registration_metadata() {
        let rows = parse_jsonl(
            r#"{"lane_id":"L-1","brief":"brief.md","worktree":"/w","branch":"agent/a","log":"/tmp/a.log","wait_file":"/tmp/a.wait","pid":10,"status":"dispatched","at":"t1"}
{"lane_id":"L-1","status":"pr-open:7","at":"t2"}"#,
        )
        .expect("valid rows should parse");

        let summary = summarize_lanes(&rows)
            .remove("L-1")
            .expect("lane should exist");

        assert_eq!(summary.status, "pr-open:7");
        assert_eq!(summary.branch.as_deref(), Some("agent/a"));
        assert_eq!(summary.brief.as_deref(), Some("brief.md"));
        assert_eq!(summary.worktree.as_deref(), Some("/w"));
        assert_eq!(summary.log.as_deref(), Some("/tmp/a.log"));
        assert_eq!(summary.wait_file.as_deref(), Some("/tmp/a.wait"));
        assert_eq!(summary.pid, Some(10));
        assert_eq!(summary.at.as_deref(), Some("t2"));
    }

    #[test]
    fn terminal_state_lattice_covers_pr_and_supervisor_outcomes() {
        for status in [
            "pr-open:670",
            "reviewed",
            "merged",
            "escalated",
            "cancelled",
            "failed",
            "exited",
            "dead",
            "stalled",
        ] {
            assert!(is_terminal_status(status), "{status} should be terminal");
        }
        for status in ["dispatched", "recovery-dispatched", "intervention"] {
            assert!(
                !is_terminal_status(status),
                "{status} should not be terminal"
            );
        }
    }

    #[test]
    fn unhealthy_terminal_states_keep_reap_gate_red() {
        for status in ["exited", "dead", "stalled", "failed"] {
            assert!(
                terminal_status_requires_failed_reap(status),
                "{status} should keep reap red"
            );
        }
        for status in [
            "pr-open:670",
            "reviewed",
            "merged",
            "escalated",
            "cancelled",
        ] {
            assert!(
                !terminal_status_requires_failed_reap(status),
                "{status} should not keep reap red"
            );
        }
    }

    #[test]
    fn pr_presence_wins_before_liveness_failures() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: Some(PrPresence { number: 9 }),
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(decision, ReapDecision::PrOpen { number: 9 });
    }

    #[test]
    fn exited_wait_file_beats_dead_process_state() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: Some(2),
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(
            decision,
            ReapDecision::Exited {
                exit_status: Some(2)
            }
        );
    }

    #[test]
    fn exited_without_pr_fails_closed_even_when_worker_exit_was_zero() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: Some(0),
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(
            decision,
            ReapDecision::Exited {
                exit_status: Some(0)
            }
        );
        assert!(is_unhealthy_reap_decision(&decision));
    }

    #[test]
    fn alive_lane_stalls_when_log_mtime_exceeds_threshold() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(true),
                log_mtime_unix_seconds: Some(60),
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(decision, ReapDecision::Stalled { stale_seconds: 40 });
    }

    #[test]
    fn alive_lane_with_recent_log_has_no_reap_event() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(true),
                log_mtime_unix_seconds: Some(80),
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(decision, ReapDecision::None);
    }

    #[test]
    fn absent_process_without_pr_is_dead() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(decision, ReapDecision::Dead);
    }

    #[test]
    fn dispatching_without_pid_waits_until_stall_threshold() {
        let mut lane = lane("dispatching");
        lane.pid = None;
        lane.at = Some("2026-06-10T17:00:00Z".to_owned());

        let decision = evaluate_reap(
            &lane,
            &LaneObservation {
                process_alive: None,
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 1781110810 },
        );

        assert_eq!(decision, ReapDecision::None);
    }

    #[test]
    fn dispatching_without_pid_stalls_after_threshold() {
        let mut lane = lane("dispatching");
        lane.pid = None;
        lane.at = Some("2026-06-10T17:00:00Z".to_owned());

        let decision = evaluate_reap(
            &lane,
            &LaneObservation {
                process_alive: None,
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 1781110830 },
        );

        assert_eq!(decision, ReapDecision::Stalled { stale_seconds: 30 });
    }

    #[test]
    fn dispatched_without_pid_fails_closed_before_stall_threshold() {
        let mut lane = lane("dispatched");
        lane.pid = None;
        lane.at = Some("2026-06-10T17:00:00Z".to_owned());

        let decision = evaluate_reap(
            &lane,
            &LaneObservation {
                process_alive: None,
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 1781110810 },
        );

        assert_eq!(decision, ReapDecision::Dead);
    }

    #[test]
    fn legacy_dispatched_without_pid_fails_closed_immediately() {
        let mut lane = lane("recovery-dispatched");
        lane.pid = None;
        lane.at = Some("2026-06-10T17:00:00Z".to_owned());

        let decision = evaluate_reap(
            &lane,
            &LaneObservation {
                process_alive: None,
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 1781110810 },
        );

        assert_eq!(decision, ReapDecision::Dead);
    }

    #[test]
    fn dispatching_without_parseable_timestamp_fails_closed() {
        let mut lane = lane("dispatching");
        lane.pid = None;
        lane.at = Some("not-a-timestamp".to_owned());

        let decision = evaluate_reap(
            &lane,
            &LaneObservation {
                process_alive: None,
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 1781110830 },
        );

        assert_eq!(decision, ReapDecision::Dead);
    }

    #[test]
    fn terminal_lanes_are_not_reaped_again() {
        let decision = evaluate_reap(
            &lane("dead"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: None,
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(decision, ReapDecision::None);
    }

    #[test]
    fn dispatch_rows_include_registration_surface_contract() {
        let row = dispatch_row(
            DispatchRowInput {
                lane_id: "L-1",
                brief: "brief.md",
                worktree: "/w",
                branch: "agent/a",
                base: "16f2e3b54",
                expected_hard_surfaces: &["tools/lane-supervisor-app/".to_owned()],
                expected_soft_surfaces: &["Cargo.lock".to_owned(), "generated-faces".to_owned()],
                log: "/tmp/lane.log",
                wait_file: "/tmp/lane.run-1.wait.json",
                start_file: "/tmp/lane.run-1.start.json",
                run_id: "run-1",
                at: "2026-06-10T17:30:00Z".to_owned(),
            },
            42,
        );

        assert_eq!(row.lane_id(), Some("L-1"));
        assert_eq!(row.status(), Some("dispatched"));
        assert_eq!(row.get_str("base"), Some("16f2e3b54"));
        assert_eq!(row.get_str("run_id"), Some("run-1"));
        assert_eq!(row.get_str("pid_role"), Some("supervisor-wrapper"));
        assert_eq!(
            row.fields().get("detached_process_group"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            row.fields().get("expected_surfaces"),
            Some(&serde_json::json!({
                "hard": ["tools/lane-supervisor-app/"],
                "soft": ["Cargo.lock", "generated-faces"],
            }))
        );
    }

    #[test]
    fn dispatch_registration_row_precedes_spawn_without_pid() {
        let row = dispatch_registration_row(DispatchRowInput {
            lane_id: "L-1",
            brief: "brief.md",
            worktree: "/w",
            branch: "agent/a",
            base: "16f2e3b54",
            expected_hard_surfaces: &["tools/lane-supervisor-app/".to_owned()],
            expected_soft_surfaces: &["Cargo.lock".to_owned(), "generated-faces".to_owned()],
            log: "/tmp/lane.log",
            wait_file: "/tmp/lane.run-1.wait.json",
            start_file: "/tmp/lane.run-1.start.json",
            run_id: "run-1",
            at: "2026-06-10T17:30:00Z".to_owned(),
        });

        assert_eq!(row.status(), Some("dispatching"));
        assert_eq!(row.get_str("run_id"), Some("run-1"));
        assert_eq!(
            row.get_str("start_file"),
            Some("/tmp/lane.run-1.start.json")
        );
        assert_eq!(row.get_i64("pid"), None);
    }

    #[test]
    fn worker_prompt_does_not_recreate_dispatcher_worktree() {
        let prompt = prompt_from_brief_pointer("BRIEF-g011-lane-supervisor.md");

        assert!(prompt.contains("already running inside the isolated worktree"));
        assert!(prompt.contains("worktree-creation instruction as already satisfied"));
        assert!(!prompt.contains("create your isolated worktree"));
    }

    #[test]
    fn decision_rows_are_appendable_schema_compatible_objects() {
        let lane = lane("dispatched");
        let row = event_row_for_decision(
            &lane,
            &ReapDecision::Stalled { stale_seconds: 77 },
            "2026-06-10T17:30:00Z".to_owned(),
        )
        .expect("stalled decision should produce row");

        assert_eq!(row.lane_id(), Some("L-1"));
        assert_eq!(row.status(), Some("stalled"));
        assert_eq!(row.get_i64("stale_seconds"), Some(77));
        assert_eq!(row.get_str("at"), Some("2026-06-10T17:30:00Z"));
    }

    #[test]
    fn pr_lookup_error_is_non_terminal_but_unhealthy() {
        let decision = evaluate_reap(
            &lane("dispatched"),
            &LaneObservation {
                process_alive: Some(false),
                log_mtime_unix_seconds: None,
                wait_exit_status: None,
                pr: None,
                pr_lookup_error: Some("gh pr list exited 1".to_owned()),
            },
            ReapOptions { stall_seconds: 30 },
            &FixedClock { now: 100 },
        );

        assert_eq!(
            decision,
            ReapDecision::Indeterminate {
                reason: "gh pr list exited 1".to_owned()
            }
        );
        assert!(is_unhealthy_reap_decision(&decision));
        let row = event_row_for_decision(&lane("dispatched"), &decision, "t".to_owned())
            .expect("indeterminate decision should produce row");
        assert_eq!(row.status(), Some("indeterminate"));
        assert!(!is_terminal_status("indeterminate"));
    }

    #[test]
    fn derives_lane_id_from_agent_branch() {
        assert_eq!(
            derive_lane_id(
                "agent/g011-lane-supervisor",
                "BRIEF-g011-lane-supervisor.md"
            ),
            "g011-lane-supervisor"
        );
    }
}
