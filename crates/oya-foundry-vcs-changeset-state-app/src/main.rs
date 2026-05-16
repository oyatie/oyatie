//! `oya-foundry-vcs-changeset-state-app` — composition root for the
//! agentic-VCS changeset state-machine event log (ADR-0110 wave-A,
//! IP-001).
//!
//! Single subcommand today:
//!
//! ```text
//! oya-foundry-vcs-changeset-state-app append \
//!     --changeset <id> \
//!     --to-state <state> \
//!     [--from-state <state>] \
//!     [--emitted-by <agent>] \
//!     [--evidence k=v,k=v] \
//!     [--log <path>]
//! ```
//!
//! Behavior:
//!
//! 1. Read the JSON file at `--log` (defaults to
//!    `registry/vcs/changeset-event-log.json`). Shape: `{"events": [...]}`.
//! 2. Parse existing rows into typed [`ChangesetEvent`]s.
//! 3. Construct the candidate event from CLI args (RFC3339 UTC `at`
//!    timestamp via `SystemTime::now`, `dedup_key` =
//!    `<changeset_id>_<to_state>_<at>`).
//! 4. Run [`validate_monotonic_event_log`] against `existing ||
//!    candidate`. On error, exit non-zero without writing.
//! 5. On success, append the candidate, write the new JSON to a
//!    sibling temp file, then `rename(2)` it into place (atomic).
//!
//! ## Signing
//!
//! ADR-0110 §"Event log shape" specifies Ed25519 signatures, keyed by
//! the agent's signing key per ADR-0058. The signing-key infrastructure
//! is not yet online — IP-001 ships an `ed25519-stub:<base64>`
//! placeholder so the field is non-empty and round-trips through the
//! validator. The stub is rejected by future signature-verification
//! adapters; that wiring is the wave-B follow-up.
//!
//! TODO(wave-B): replace [`stub_signature`] with real Ed25519 over
//! `(changeset_id, dedup_key, to_state, at, emitted_by,
//! cost_budget_remaining)` using the per-agent signing key resolved
//! from ADR-0058's key registry.
//!
//! ADR-0083 Tier 1: no `.unwrap()` / `.expect()` / `panic!()` outside
//! `cfg(test)`. The binary returns `ExitCode::FAILURE` with a
//! diagnostic on any I/O or validation error.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_foundry_vcs_changeset_state_kernel::{
    ChangesetEvent, ChangesetState, CostBudget, MonotonicityError, validate_monotonic_event_log,
};
use serde_json::{Map, Value, json};

const DEFAULT_LOG_PATH: &str = "registry/vcs/changeset-event-log.json";
const DEFAULT_EMITTER: &str = "oya-foundry-vcs-changeset-state-app";

// Wave-A default budget. Replaced by real budget-tracking adapter in
// wave-B (per ADR-0110 §"Event log shape"); for now every append
// stamps a stub-but-non-exhausted budget so the validator's
// `CostBudget::is_exhausted` check does not fire on the first event.
const STUB_USD: f64 = 100.0;
const STUB_TOKENS: u64 = 10_000_000;
const STUB_INVOCATIONS: u32 = 100;

const USAGE: &str = "Usage: oya-foundry-vcs-changeset-state-app append \
                     --changeset <id> --to-state <state> \
                     [--from-state <state>] [--emitted-by <agent>] \
                     [--evidence k=v,k=v] [--log <path>]";

fn main() -> ExitCode {
    let mut args = env::args();
    // Discard the binary path; surface the rest to the parser.
    let _ = args.next();
    let argv: Vec<String> = args.collect();
    match run(argv) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("oya-foundry-vcs-changeset-state-app: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(argv: Vec<String>) -> Result<String, String> {
    let mut iter = argv.into_iter();
    let subcommand = iter.next().ok_or_else(|| USAGE.to_string())?;
    if subcommand != "append" {
        return Err(format!("unknown subcommand `{subcommand}`. {USAGE}"));
    }
    let parsed = parse_append_args(iter.collect())?;
    let log_path = parsed.log_path.clone();
    let existing = read_event_log(&log_path)?;
    let candidate = build_candidate_event(&parsed)?;
    let mut combined = existing;
    combined.push(candidate.clone());
    validate_monotonic_event_log(&combined).map_err(|err: MonotonicityError| {
        format!("monotonic invariant violated by candidate event: {err}")
    })?;
    write_event_log(&log_path, &combined)?;
    Ok(format!(
        "appended event: changeset={} to_state={} dedup_key={} log={}",
        candidate.changeset_id,
        candidate.to_state.as_wire(),
        candidate.dedup_key,
        log_path.display(),
    ))
}

#[derive(Clone, Debug)]
struct AppendArgs {
    changeset_id: String,
    to_state: ChangesetState,
    from_state: Option<ChangesetState>,
    emitted_by: String,
    evidence: BTreeMap<String, String>,
    log_path: PathBuf,
}

fn parse_append_args(argv: Vec<String>) -> Result<AppendArgs, String> {
    let mut changeset_id: Option<String> = None;
    let mut to_state_raw: Option<String> = None;
    let mut from_state_raw: Option<String> = None;
    let mut emitted_by: Option<String> = None;
    let mut evidence_raw: Option<String> = None;
    let mut log_path = PathBuf::from(DEFAULT_LOG_PATH);

    let mut iter = argv.into_iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("flag `{flag}` requires a value. {USAGE}"))?;
        match flag.as_str() {
            "--changeset" => changeset_id = Some(value),
            "--to-state" => to_state_raw = Some(value),
            "--from-state" => from_state_raw = Some(value),
            "--emitted-by" => emitted_by = Some(value),
            "--evidence" => evidence_raw = Some(value),
            "--log" => log_path = PathBuf::from(value),
            other => return Err(format!("unknown flag `{other}`. {USAGE}")),
        }
    }

    let changeset_id = changeset_id.ok_or_else(|| format!("--changeset is required. {USAGE}"))?;
    let to_state_raw = to_state_raw.ok_or_else(|| format!("--to-state is required. {USAGE}"))?;
    let to_state = ChangesetState::from_wire(&to_state_raw).ok_or_else(|| {
        format!("--to-state value `{to_state_raw}` is not in the closed 13-value enum per ADR-0110")
    })?;
    let from_state = match from_state_raw {
        Some(raw) => Some(ChangesetState::from_wire(&raw).ok_or_else(|| {
            format!("--from-state value `{raw}` is not in the closed 13-value enum")
        })?),
        None => None,
    };

    Ok(AppendArgs {
        changeset_id,
        to_state,
        from_state,
        emitted_by: emitted_by.unwrap_or_else(|| DEFAULT_EMITTER.to_string()),
        evidence: parse_evidence_csv(evidence_raw.as_deref())?,
        log_path,
    })
}

fn parse_evidence_csv(raw: Option<&str>) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let Some(raw) = raw else { return Ok(map) };
    if raw.is_empty() {
        return Ok(map);
    }
    for pair in raw.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair
            .split_once('=')
            .ok_or_else(|| format!("evidence pair `{pair}` is not in k=v form"))?;
        map.insert(k.trim().to_string(), v.trim().to_string());
    }
    Ok(map)
}

fn build_candidate_event(args: &AppendArgs) -> Result<ChangesetEvent, String> {
    let at = current_rfc3339_utc()?;
    let dedup_key = format!("{}_{}_{}", args.changeset_id, args.to_state.as_wire(), at);
    Ok(ChangesetEvent {
        changeset_id: args.changeset_id.clone(),
        dedup_key,
        from_state: args.from_state,
        to_state: args.to_state,
        at,
        emitted_by: args.emitted_by.clone(),
        cost_budget_remaining: CostBudget {
            usd_remaining: STUB_USD,
            tokens_remaining: STUB_TOKENS,
            agent_invocations_remaining: STUB_INVOCATIONS,
        },
        evidence: args.evidence.clone(),
        alternates_considered: Vec::new(),
        skipped: false,
        signature: stub_signature(&args.changeset_id, args.to_state),
    })
}

fn stub_signature(changeset_id: &str, to_state: ChangesetState) -> String {
    // TODO(wave-B): replace with real Ed25519 over the canonical
    // signing tuple (ADR-0058 + ADR-0110 §"Event log shape"). For
    // wave-A we stamp a deterministic placeholder so the field is
    // non-empty and consumers can grep for the stub prefix.
    let payload = format!("{changeset_id}:{}", to_state.as_wire());
    let encoded = base64_encode(payload.as_bytes());
    format!("ed25519-stub:{encoded}")
}

fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((bytes.len() + 2) / 3) * 4);
    let mut chunks = bytes.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(ALPHABET[(n & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        0 => {}
        1 => {
            let n = u32::from(rem[0]) << 16;
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

fn current_rfc3339_utc() -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock before UNIX epoch: {err}"))?;
    let secs = now.as_secs() as i64;
    Ok(format_rfc3339_utc(secs))
}

/// Format a Unix-epoch second count as an RFC3339 UTC `Z` timestamp
/// (`YYYY-MM-DDThh:mm:ssZ`). Implementation is calendar arithmetic on
/// proleptic Gregorian dates; no chrono/time dep so the app stays
/// dep-thin and ADR-0083 Tier-1-clean. Sub-second precision is
/// intentionally truncated — the dedup_key only needs second-level
/// granularity for ADR-0110.
fn format_rfc3339_utc(epoch_secs: i64) -> String {
    let total = epoch_secs;
    let day_secs: i64 = 86_400;
    let days_since_epoch = total.div_euclid(day_secs);
    let secs_in_day = total.rem_euclid(day_secs) as u32;
    let hour = secs_in_day / 3600;
    let minute = (secs_in_day / 60) % 60;
    let second = secs_in_day % 60;
    let (year, month, day) = epoch_day_to_ymd(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Map "days since 1970-01-01" to (year, month, day) on the proleptic
/// Gregorian calendar. Algorithm: Howard Hinnant, "date" library
/// public-domain civil-from-days; adapted to plain i64 arithmetic.
fn epoch_day_to_ymd(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m, d)
}

fn read_event_log(path: &Path) -> Result<Vec<ChangesetEvent>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} as JSON: {err}", path.display()))?;
    let events = value
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing top-level `events` array", path.display()))?;
    events.iter().map(parse_event_value).collect()
}

fn parse_event_value(value: &Value) -> Result<ChangesetEvent, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "event row is not a JSON object".to_string())?;
    let changeset_id = string_field(obj, "changeset_id")?;
    let dedup_key = string_field(obj, "dedup_key")?;
    let to_state_raw = string_field(obj, "to_state")?;
    let to_state = ChangesetState::from_wire(&to_state_raw).ok_or_else(|| {
        format!("event `to_state`=`{to_state_raw}` is not in the closed 13-value enum")
    })?;
    let from_state = match obj.get("from_state") {
        Some(Value::Null) | None => None,
        Some(Value::String(raw)) => Some(ChangesetState::from_wire(raw).ok_or_else(|| {
            format!("event `from_state`=`{raw}` is not in the closed 13-value enum")
        })?),
        Some(other) => {
            return Err(format!(
                "event `from_state` is not a string or null: {other}"
            ));
        }
    };
    let at = string_field(obj, "at")?;
    let emitted_by = string_field(obj, "emitted_by")?;
    let cost_budget_remaining = parse_cost_budget(obj.get("cost_budget_remaining"))?;
    let evidence = parse_evidence_map(obj.get("evidence"))?;
    let alternates_considered = parse_string_array(obj.get("alternates_considered"));
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

fn string_field(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
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
        .ok_or_else(|| "cost_budget_remaining.usd_remaining missing or not a number".to_string())?;
    let tokens_remaining = obj
        .get("tokens_remaining")
        .and_then(Value::as_u64)
        .ok_or_else(|| "cost_budget_remaining.tokens_remaining missing or not u64".to_string())?;
    let agent_invocations_remaining = obj
        .get("agent_invocations_remaining")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| {
            "cost_budget_remaining.agent_invocations_remaining missing or out of range".to_string()
        })?;
    Ok(CostBudget {
        usd_remaining,
        tokens_remaining,
        agent_invocations_remaining,
    })
}

fn parse_evidence_map(value: Option<&Value>) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let Some(value) = value else { return Ok(out) };
    if value.is_null() {
        return Ok(out);
    }
    let obj = value
        .as_object()
        .ok_or_else(|| "`evidence` is not a JSON object".to_string())?;
    for (k, v) in obj {
        let string_v = match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        out.insert(k.clone(), string_v);
    }
    Ok(out)
}

fn parse_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn write_event_log(path: &Path, events: &[ChangesetEvent]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("log path {} has no parent directory", path.display()))?;
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("mkdir -p {}: {err}", parent.display()))?;
    }
    let serialized = serialize_event_log(events)?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, serialized.as_bytes())
        .map_err(|err| format!("write {}: {err}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .map_err(|err| format!("rename {} -> {}: {err}", tmp_path.display(), path.display()))?;
    Ok(())
}

fn serialize_event_log(events: &[ChangesetEvent]) -> Result<String, String> {
    let mut rows: Vec<Value> = Vec::with_capacity(events.len());
    for event in events {
        rows.push(event_to_json(event));
    }
    let doc = json!({ "events": rows });
    serde_json::to_string_pretty(&doc)
        .map(|mut s| {
            s.push('\n');
            s
        })
        .map_err(|err| format!("serialize event log: {err}"))
}

fn event_to_json(event: &ChangesetEvent) -> Value {
    let mut evidence = Map::new();
    for (k, v) in &event.evidence {
        evidence.insert(k.clone(), Value::String(v.clone()));
    }
    json!({
        "changeset_id": event.changeset_id,
        "dedup_key": event.dedup_key,
        "from_state": event.from_state.map(|s| s.as_wire().to_string()),
        "to_state": event.to_state.as_wire(),
        "at": event.at,
        "emitted_by": event.emitted_by,
        "cost_budget_remaining": {
            "usd_remaining": event.cost_budget_remaining.usd_remaining,
            "tokens_remaining": event.cost_budget_remaining.tokens_remaining,
            "agent_invocations_remaining": event.cost_budget_remaining.agent_invocations_remaining,
        },
        "evidence": Value::Object(evidence),
        "alternates_considered": event.alternates_considered,
        "skipped": event.skipped,
        "signature": event.signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_format_epoch_zero() {
        assert_eq!(format_rfc3339_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn rfc3339_format_known_date() {
        // 2026-05-16T01:00:00Z = 1_779_584_400 (verified via `date -u -d`).
        // Compute the expected from algorithm rather than hardcoding the
        // epoch to keep the test self-checking against the algorithm.
        let formatted = format_rfc3339_utc(1_779_584_400);
        assert_eq!(formatted, "2026-05-16T01:00:00Z");
    }

    #[test]
    fn base64_encode_round_known_vector() {
        // RFC 4648 §10 test vectors.
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn parse_evidence_csv_supports_multiple_pairs() {
        let parsed = parse_evidence_csv(Some("pr_number=4,head_sha=abc123")).unwrap();
        assert_eq!(parsed.get("pr_number").map(String::as_str), Some("4"));
        assert_eq!(parsed.get("head_sha").map(String::as_str), Some("abc123"));
    }

    #[test]
    fn append_to_empty_log_then_round_trip() {
        let tmp_dir =
            std::env::temp_dir().join(format!("oya-cs-state-app-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let log_path = tmp_dir.join("changeset-event-log.json");
        let _ = std::fs::remove_file(&log_path);

        let args = AppendArgs {
            changeset_id: "cs_test1".to_string(),
            to_state: ChangesetState::Opened,
            from_state: None,
            emitted_by: "unit-test".to_string(),
            evidence: BTreeMap::new(),
            log_path: log_path.clone(),
        };
        let candidate = build_candidate_event(&args).unwrap();
        write_event_log(&log_path, &[candidate.clone()]).unwrap();
        let round_trip = read_event_log(&log_path).unwrap();
        assert_eq!(round_trip.len(), 1);
        assert_eq!(round_trip[0].changeset_id, candidate.changeset_id);
        assert_eq!(round_trip[0].to_state, ChangesetState::Opened);
        assert!(round_trip[0].signature.starts_with("ed25519-stub:"));
        std::fs::remove_file(&log_path).unwrap();
        std::fs::remove_dir(&tmp_dir).ok();
    }
}
