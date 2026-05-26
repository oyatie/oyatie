//! `oya gate validate changeset-state-enum-closed` runner.
//!
//! Walks every row in `registry/vcs/changeset-event-log.json` and
//! asserts that the `to_state` (and `from_state` when present) value
//! belongs to the closed 13-value `ChangesetState` enum per ADR-0110.
//! Lane id: `oya-governance-changeset-state-enum-closed`.
//!
//! The validator in
//! `changeset_state_monotonicity_gate::validate_changeset_state_monotonicity_gate`
//! also implicitly enforces this via `ChangesetState::from_wire`, but
//! this lane makes it explicit and emits per-event evidence so a drift
//! to a typo like `pr_oepn` surfaces under this lane name (and not as
//! a confusing "missing field" parse error).

use std::fs;
use std::path::PathBuf;

use oya_vcs_changeset_state_kernel::ChangesetState;
use serde_json::Value;

const USAGE: &str =
    "oya gate validate changeset-state-enum-closed [--log <registry/vcs/changeset-event-log.json>]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangesetStateEnumClosedValidateArgs {
    pub log_path: PathBuf,
}

impl Default for ChangesetStateEnumClosedValidateArgs {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("registry/vcs/changeset-event-log.json"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangesetStateEnumClosedReport {
    pub events_checked: usize,
    pub distinct_states_seen: usize,
}

pub(crate) fn parse_changeset_state_enum_closed_validate_args(
    args: Vec<String>,
) -> Result<ChangesetStateEnumClosedValidateArgs, String> {
    let mut parsed = ChangesetStateEnumClosedValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(USAGE.to_string());
        };
        match flag.as_str() {
            "--log" => parsed.log_path = PathBuf::from(value),
            _ => return Err(USAGE.to_string()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_changeset_state_enum_closed_gate(
    args: ChangesetStateEnumClosedValidateArgs,
) -> Result<ChangesetStateEnumClosedReport, String> {
    if !args.log_path.exists() {
        return Ok(ChangesetStateEnumClosedReport {
            events_checked: 0,
            distinct_states_seen: 0,
        });
    }
    let text = fs::read_to_string(&args.log_path)
        .map_err(|err| format!("read {}: {err}", args.log_path.display()))?;
    if text.trim().is_empty() {
        return Ok(ChangesetStateEnumClosedReport {
            events_checked: 0,
            distinct_states_seen: 0,
        });
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} as JSON: {err}", args.log_path.display()))?;
    let array = value
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing `events` array", args.log_path.display()))?;
    let mut events_checked = 0usize;
    let mut seen: std::collections::BTreeSet<&'static str> = std::collections::BTreeSet::new();
    for (idx, row) in array.iter().enumerate() {
        let obj = row
            .as_object()
            .ok_or_else(|| format!("event[{idx}] is not a JSON object"))?;
        let to_state_raw = obj
            .get("to_state")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("event[{idx}] missing string `to_state`"))?;
        let typed = ChangesetState::from_wire(to_state_raw).ok_or_else(|| {
            format!(
                "event[{idx}] `to_state`=`{to_state_raw}` is not in the closed 13-value ChangesetState enum (ADR-0110)"
            )
        })?;
        seen.insert(typed.as_wire());
        if let Some(Value::String(raw)) = obj.get("from_state") {
            let typed_from = ChangesetState::from_wire(raw).ok_or_else(|| {
                format!(
                    "event[{idx}] `from_state`=`{raw}` is not in the closed 13-value ChangesetState enum"
                )
            })?;
            seen.insert(typed_from.as_wire());
        }
        events_checked += 1;
    }
    Ok(ChangesetStateEnumClosedReport {
        events_checked,
        distinct_states_seen: seen.len(),
    })
}
