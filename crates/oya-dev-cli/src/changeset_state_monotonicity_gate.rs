//! `oya gate validate changeset-state-monotonicity` runner.
//!
//! Reads `registry/vcs/changeset-event-log.json`, parses each row
//! into a typed [`ChangesetEvent`], and invokes
//! [`validate_monotonic_event_log`] from the changeset-state kernel.
//! Lane id: `oya-foundry-fitness-changeset-state-monotonicity`.
//!
//! ADR-0110 wave-A IP-001 acceptance: empty log is treated as
//! vacuously valid (the kernel rejects empty logs by design; the lane
//! distinguishes "no events yet" from "log explicitly malformed").

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oya_vcs_changeset_state_kernel::{
    ChangesetEvent, ChangesetState, CostBudget, MonotonicityReport, validate_monotonic_event_log,
};
use serde_json::Value;

const USAGE: &str = "oya gate validate changeset-state-monotonicity [--log <registry/vcs/changeset-event-log.json>]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangesetStateMonotonicityValidateArgs {
    pub log_path: PathBuf,
}

impl Default for ChangesetStateMonotonicityValidateArgs {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("registry/vcs/changeset-event-log.json"),
        }
    }
}

pub(crate) fn parse_changeset_state_monotonicity_validate_args(
    args: Vec<String>,
) -> Result<ChangesetStateMonotonicityValidateArgs, String> {
    let mut parsed = ChangesetStateMonotonicityValidateArgs::default();
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

pub(crate) fn validate_changeset_state_monotonicity_gate(
    args: ChangesetStateMonotonicityValidateArgs,
) -> Result<MonotonicityReport, String> {
    let events = read_event_log(&args.log_path)?;
    if events.is_empty() {
        return Ok(MonotonicityReport {
            events_checked: 0,
            terminal_state: None,
        });
    }
    validate_monotonic_event_log(&events).map_err(|err| err.to_string())
}

fn read_event_log(path: &PathBuf) -> Result<Vec<ChangesetEvent>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} as JSON: {err}", path.display()))?;
    let array = value
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing `events` array", path.display()))?;
    let mut events = Vec::with_capacity(array.len());
    for row in array {
        events.push(parse_event(row)?);
    }
    Ok(events)
}

fn parse_event(value: &Value) -> Result<ChangesetEvent, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "event row is not a JSON object".to_string())?;
    let changeset_id = string_field(obj, "changeset_id")?;
    let dedup_key = string_field(obj, "dedup_key")?;
    let to_state_raw = string_field(obj, "to_state")?;
    let to_state = ChangesetState::from_wire(&to_state_raw).ok_or_else(|| {
        format!("event `to_state`=`{to_state_raw}` not in closed 13-value enum per ADR-0110")
    })?;
    let from_state = match obj.get("from_state") {
        Some(Value::Null) | None => None,
        Some(Value::String(raw)) => Some(
            ChangesetState::from_wire(raw)
                .ok_or_else(|| format!("event `from_state`=`{raw}` not in closed 13-value enum"))?,
        ),
        Some(other) => return Err(format!("`from_state` not string|null: {other}")),
    };
    let at = string_field(obj, "at")?;
    let emitted_by = string_field(obj, "emitted_by")?;
    let cost_budget_remaining = parse_cost_budget(obj.get("cost_budget_remaining"))?;
    let evidence = parse_evidence(obj.get("evidence"))?;
    let alternates_considered = obj
        .get("alternates_considered")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let skipped = obj.get("skipped").and_then(Value::as_bool).unwrap_or(false);
    let signature = string_field(obj, "signature")?;
    Ok(ChangesetEvent {
        changeset_id,
        dedup_key,
        from_state,
        to_state,
        at,
        emitted_by,
        cost_budget_remaining,
        evidence,
        alternates_considered,
        skipped,
        signature,
    })
}

fn string_field(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("event row missing string field `{key}`"))
}

fn parse_cost_budget(value: Option<&Value>) -> Result<CostBudget, String> {
    let Some(value) = value else {
        return Err("event row missing `cost_budget_remaining`".to_string());
    };
    let obj = value
        .as_object()
        .ok_or_else(|| "`cost_budget_remaining` is not a JSON object".to_string())?;
    let usd_remaining = obj
        .get("usd_remaining")
        .and_then(Value::as_f64)
        .ok_or_else(|| "cost_budget_remaining.usd_remaining missing/not a number".to_string())?;
    let tokens_remaining = obj
        .get("tokens_remaining")
        .and_then(Value::as_u64)
        .ok_or_else(|| "cost_budget_remaining.tokens_remaining missing/not u64".to_string())?;
    let agent_invocations_remaining = obj
        .get("agent_invocations_remaining")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| {
            "cost_budget_remaining.agent_invocations_remaining missing/out of range".to_string()
        })?;
    Ok(CostBudget {
        usd_remaining,
        tokens_remaining,
        agent_invocations_remaining,
    })
}

fn parse_evidence(value: Option<&Value>) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let Some(value) = value else { return Ok(out) };
    if value.is_null() {
        return Ok(out);
    }
    let obj = value
        .as_object()
        .ok_or_else(|| "`evidence` is not a JSON object".to_string())?;
    for (k, v) in obj {
        let s = match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        out.insert(k.clone(), s);
    }
    Ok(out)
}
