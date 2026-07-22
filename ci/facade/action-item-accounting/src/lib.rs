//! # cloud-ci-friction-accounting (ADR-0544)
//!
//! The FRIC-total-accounting meta-gate. Founder decision 2026-06-10: every friction-ledger row
//! must terminate in a gate, an automation, or an explicit accepted-risk entry, enforced by a gate
//! so unconverted frictions block merges like code debt. This is the closed-loop accounting tracker
//! for the friction ledger — the Google SRE postmortem **action-item** model reimplemented
//! Rust-native: every action item (here, every friction) must have a declared disposition and, once
//! terminal or accepted-risk, verifiable closure (evidence).
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the ledger path, the free-text status
//! taxonomy (`status -> {open|terminal|accepted-risk}`), the required-field set, the evidence policy
//! — are DATA in `friction-accounting-policy.json`. Nothing oyatie-specific is hardcoded in Rust; a
//! different repo adopts the gate by repointing the policy at its own ledger.
//!
//! ## Kernel contract
//! - [`collect_observed_frictions`] `(root, policy) -> {rows:[..]}` reads the ledger file (the only
//!   I/O; read-only, no temp files).
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it folds the event-sourced append rows onto their friction id and applies the
//!   closed-loop invariants.
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Ratchet semantics (never discourage logging)
//! Appending a friction row never fails the gate by itself. The blocking codes police schema
//! validity, declared disposition, and closure integrity. Closure-integrity codes
//! (`friction_closed_without_evidence`, `friction_accepted_risk_without_evidence`,
//! `friction_duplicate_primary_row`) are born-blocking frozen-empty: the live ledger satisfies them
//! today, so any NEW occurrence fails closed. Schema/disposition/taxonomy codes baseline today's
//! legacy debt behind a reviewed shrink-only ceiling (the live-repo test owns it) so the gate is not
//! launderable by same-PR baseline regeneration (FRIC-1781112000).
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `friction_policy_gate_id_mismatch`        — policy `gate_id` != [`GATE_ID`].
//! - `friction_missing_required_field`         — a PRIMARY row omits/blanks a required field.
//! - `friction_unknown_status`                 — a friction's effective status maps to no taxonomy class.
//! - `friction_no_disposition` — a friction declares no non-blank `enforcement_fix` and is not in
//!   the accepted-risk class.
//! - `friction_closed_without_evidence`        — a terminal-class friction cites no evidence.
//! - `friction_accepted_risk_without_evidence` — an accepted-risk friction cites no evidence.
//! - `friction_duplicate_primary_row`          — two PRIMARY rows share one `id` (appends are legitimate).
//! - `friction_orphan_update_row` — a friction id has ONLY update-shaped rows and no anchoring
//!   PRIMARY record. Without a primary the schema/disposition checks cannot bind, so an update-only
//!   row would otherwise evade every check and be silently unaccounted.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the oya-ci registry id.
pub const GATE_ID: &str = "cloud-ci-friction-accounting";

/// Candidate and protected inputs for the v2 admission extension. The protected-facts path is an
/// untracked SCM-materializer output: CI owns its merge-base contents, never the candidate PR.
pub const FIXUPTASK_V2_CANDIDATE_JSONL_PATH: &str = "registry/fixuptasks.jsonl";
pub const FIXUPTASK_V2_MAPPING_PATH: &str = "registry/fixuptask-v2-predecessor-mapping.json";
pub const FIXUPTASK_V2_PROTECTED_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json";

/// The eight blocking violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 8] = [
    "friction_policy_gate_id_mismatch",
    "friction_missing_required_field",
    "friction_unknown_status",
    "friction_no_disposition",
    "friction_closed_without_evidence",
    "friction_accepted_risk_without_evidence",
    "friction_duplicate_primary_row",
    "friction_orphan_update_row",
];

/// The sentinel key for codes that are policy-level rather than per-friction.
const POLICY_KEY: &str = "<policy>";

/// The taxonomy class a status maps to. `Unknown` is a fail-closed sentinel for NEW statuses that
/// match no taxonomy entry (legacy unknowns are baselined by the live-repo ceiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Open,
    Terminal,
    AcceptedRisk,
    Unknown,
}

/// Errors collecting the observed ledger. The kernel returns these instead of panicking so the
/// caller (CI / a controller) decides how to surface them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    MissingLedgerPath,
    Io(String),
    Parse { line: usize, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::MissingLedgerPath => {
                write!(f, "policy `ledger_path` must be a non-empty string")
            }
            CollectError::Io(message) => write!(f, "friction ledger io: {message}"),
            CollectError::Parse { line, message } => {
                write!(
                    f,
                    "friction ledger line {line} is not valid JSON: {message}"
                )
            }
        }
    }
}

impl std::error::Error for CollectError {}

/// Collect the observed friction ledger described by the policy's `ledger_path`.
///
/// Reads the JSONL ledger relative to `root`, parsing each non-blank line into a row. The output is
/// `{ "rows": [ <row>, .. ] }` mirroring the on-disk physical order (folding happens in the pure
/// evaluator). Read-only: writes no temporary files, so each run cleans up after itself by
/// construction. A blank ledger (no rows) is valid and yields `{ "rows": [] }`.
pub fn collect_observed_frictions(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let ledger_path = policy
        .get("ledger_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(CollectError::MissingLedgerPath)?;
    let absolute = root.join(ledger_path);
    let text = fs::read_to_string(&absolute)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", absolute.display())))?;
    let mut rows = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line).map_err(|e| CollectError::Parse {
            line: index + 1,
            message: e.to_string(),
        })?;
        rows.push(row);
    }
    Ok(json!({ "rows": rows }))
}

/// Read the candidate JSONL registry without treating its required descriptive header as a row.
/// The caller supplies the path so tests and the materializer can use the same adapter.
pub fn collect_fixuptask_candidate_jsonl(root: &Path, path: &str) -> Result<Value, CollectError> {
    let text = fs::read_to_string(root.join(path))
        .map_err(|error| CollectError::Io(format!("read {path}: {error}")))?;
    let mut rows = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).map_err(|error| CollectError::Parse {
            line: index + 1,
            message: error.to_string(),
        })?;
        if value.get("_meta").is_some() && value.get("id").is_none() {
            continue;
        }
        rows.push(value);
    }
    Ok(json!({ "rows": rows }))
}

fn collect_json_input(root: &Path, path: &str) -> Result<Value, CollectError> {
    let text = fs::read_to_string(root.join(path))
        .map_err(|error| CollectError::Io(format!("read {path}: {error}")))?;
    serde_json::from_str(&text).map_err(|error| CollectError::Parse {
        line: 1,
        message: format!("parse {path}: {error}"),
    })
}

fn collect_optional_json_input(root: &Path, path: &str) -> Result<Option<Value>, CollectError> {
    match fs::read_to_string(root.join(path)) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| CollectError::Parse {
                line: 1,
                message: format!("parse {path}: {error}"),
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CollectError::Io(format!("read {path}: {error}"))),
    }
}

fn collect_optional_bytes(root: &Path, path: &str) -> Result<Option<Vec<u8>>, CollectError> {
    match fs::read(root.join(path)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CollectError::Io(format!("read {path}: {error}"))),
    }
}

/// Runs the v2 admission extension from its real gate inputs. The protected facts MUST be the
/// SCM-materializer sidecar; this adapter has no candidate parameter for a merge-base baseline.
pub fn evaluate_fixuptask_v2_materialized_gate(
    root: &Path,
) -> Result<BTreeSet<Finding>, CollectError> {
    let candidate = collect_fixuptask_candidate_jsonl(root, FIXUPTASK_V2_CANDIDATE_JSONL_PATH)?;
    let protected = collect_json_input(root, FIXUPTASK_V2_PROTECTED_FACTS_PATH)?;
    let legacy = collect_optional_bytes(root, ".omc/ultragoal/friction-ledger.jsonl")?;
    let mapping = collect_optional_json_input(root, FIXUPTASK_V2_MAPPING_PATH)?;
    Ok(evaluate_fixuptask_v2_admission(
        &protected,
        &candidate,
        legacy.as_deref(),
        mapping.as_ref(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

fn non_blank_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

/// Classify a free-text status against the policy taxonomy. With `status_match == "prefix"` (the
/// default), the LONGEST matching taxonomy key wins, so `RESOLVED-fully` resolves via `RESOLVED`
/// while a more specific key (e.g. `escalated-to-leader`) overrides a broader one. Exact match is
/// also supported (`status_match == "exact"`).
pub fn classify_status(policy: &Value, status: &str) -> StatusClass {
    let taxonomy = policy.get("status_taxonomy").and_then(Value::as_object);
    let Some(taxonomy) = taxonomy else {
        return StatusClass::Unknown;
    };
    let prefix_mode = policy
        .get("status_match")
        .and_then(Value::as_str)
        .map(|mode| mode != "exact")
        .unwrap_or(true);
    let status = status.trim();
    let mut best: Option<(usize, StatusClass)> = None;
    for (key, class) in taxonomy {
        let matches = if prefix_mode {
            status == key || status.starts_with(key)
        } else {
            status == key
        };
        if !matches {
            continue;
        }
        let class = match class.as_str() {
            Some("open") => StatusClass::Open,
            Some("terminal") => StatusClass::Terminal,
            Some("accepted-risk") => StatusClass::AcceptedRisk,
            _ => StatusClass::Unknown,
        };
        let weight = key.len();
        if best.map(|(w, _)| weight > w).unwrap_or(true) {
            best = Some((weight, class));
        }
    }
    best.map(|(_, class)| class).unwrap_or(StatusClass::Unknown)
}

/// The folded, effective state of one friction id across its physical rows.
#[derive(Debug, Default, Clone)]
struct FrictionState {
    /// Count of PRIMARY rows (rows carrying both `status` and `friction`): >1 is a duplicate defect.
    primary_count: usize,
    /// The effective status: the latest `status_update` if any update rows exist, else the primary
    /// `status`. Empty if neither is present.
    effective_status: String,
    /// True if any row for this id carries a non-blank `enforcement_fix`.
    has_disposition: bool,
    /// True if any row for this id carries a non-blank `evidence`.
    has_evidence: bool,
}

fn is_primary(row: &Value) -> bool {
    row.get("status").and_then(Value::as_str).is_some()
        && row.get("friction").and_then(Value::as_str).is_some()
}

/// Pure evaluator. `policy` is DATA (`friction-accounting-policy.json`); `observed` is the collected
/// ledger shaped as `{ "rows": [ <row>, .. ] }` in physical order.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if non_blank_str(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "friction_policy_gate_id_mismatch",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let required_fields: Vec<String> = policy
        .get("required_primary_fields")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let terminal_requires_evidence = policy
        .get("terminal_requires_evidence")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let accepted_risk_requires_evidence = policy
        .get("accepted_risk_requires_evidence")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let rows = observed
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Fold physical rows onto their friction id (the event-sourced append model). `states` is a
    // BTreeMap (key-sorted, deterministic) so per-friction findings are order-independent; the
    // BTreeSet output is sorted anyway.
    let mut states: BTreeMap<String, FrictionState> = BTreeMap::new();
    for row in &rows {
        let id = non_blank_str(row, "id").unwrap_or("");
        if id.is_empty() {
            // A row with no usable id cannot be accounted: schema violation keyed to a sentinel.
            findings.insert(Finding::new(
                "friction_missing_required_field",
                "<row-without-id>",
                "ledger row carries no non-blank `id`",
            ));
            continue;
        }
        let state = states.entry(id.to_owned()).or_default();

        if is_primary(row) {
            state.primary_count += 1;
            // Required-field check applies to PRIMARY rows (they carry the full record).
            for field in &required_fields {
                if non_blank_str(row, field).is_none() {
                    findings.insert(Finding::new(
                        "friction_missing_required_field",
                        id,
                        format!("primary row missing non-blank `{field}`"),
                    ));
                }
            }
            if let Some(status) = non_blank_str(row, "status") {
                // The primary status is the baseline effective status; an update overrides it below.
                if state.effective_status.is_empty() {
                    state.effective_status = status.to_owned();
                }
            }
        }
        // Update rows carry the latest disposition transition; they always win the effective status.
        if let Some(update) = non_blank_str(row, "status_update") {
            state.effective_status = update.to_owned();
        }
        if non_blank_str(row, "enforcement_fix").is_some() {
            state.has_disposition = true;
        }
        if non_blank_str(row, "evidence").is_some() {
            state.has_evidence = true;
        }
    }

    for (id, state) in &states {
        if state.primary_count == 0 {
            // ONLY update-shaped rows exist for this id: there is no anchoring PRIMARY record, so the
            // required-field/disposition/duplicate/class checks cannot bind. An update-only row is the
            // cheapest way to evade the born-blocking schema check (status_update=RESOLVED + evidence
            // would otherwise fold to a clean terminal state and pass), so the missing primary is
            // itself the violation — and the ONLY one we emit for this id, so a fixed orphan drops a
            // single baseline key rather than churning several. This is baseline-block-on-new: the
            // live ledger's pre-existing orphan ids are frozen as shrinkable legacy debt, and any NEW
            // orphan-update id fails closed.
            findings.insert(Finding::new(
                "friction_orphan_update_row",
                id,
                "friction has only update-shaped rows and no anchoring primary record; \
                 log a primary row (id/seen_at/friction/enforcement_fix/status) for this id",
            ));
            continue;
        }
        if state.primary_count > 1 {
            findings.insert(Finding::new(
                "friction_duplicate_primary_row",
                id,
                format!(
                    "{} primary rows share id `{id}`; updates append, primaries do not",
                    state.primary_count
                ),
            ));
        }

        let class = classify_status(policy, &state.effective_status);
        match class {
            StatusClass::Unknown => {
                findings.insert(Finding::new(
                    "friction_unknown_status",
                    id,
                    format!(
                        "effective status `{}` maps to no taxonomy class",
                        state.effective_status
                    ),
                ));
            }
            StatusClass::Open => {
                if !state.has_disposition {
                    findings.insert(Finding::new(
                        "friction_no_disposition",
                        id,
                        "open friction declares no non-blank `enforcement_fix` (disposition)",
                    ));
                }
            }
            StatusClass::Terminal => {
                if !state.has_disposition {
                    findings.insert(Finding::new(
                        "friction_no_disposition",
                        id,
                        "terminal friction declares no non-blank `enforcement_fix` (disposition)",
                    ));
                }
                if terminal_requires_evidence && !state.has_evidence {
                    findings.insert(Finding::new(
                        "friction_closed_without_evidence",
                        id,
                        "terminal-status friction cites no `evidence` (gate id / ADR / tool path)",
                    ));
                }
            }
            StatusClass::AcceptedRisk => {
                // Accepted-risk frictions (founder-held / escalated) are not agent-closeable and are
                // not open debt; their disposition IS the recorded acceptance, proven by evidence.
                if accepted_risk_requires_evidence && !state.has_evidence {
                    findings.insert(Finding::new(
                        "friction_accepted_risk_without_evidence",
                        id,
                        "accepted-risk friction cites no `evidence` for the holder/decision",
                    ));
                }
            }
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// The canonical v2 row contract. The evaluator parses this embedded JSON Schema rather than
/// duplicating its required fields or lifecycle conditionals in Rust. Keep the schema and gate in
/// the same crate so a changed schema cannot silently weaken admission.
const FIXUPTASK_V2_SCHEMA: &str = include_str!("../fixuptask-v2-schema.json");

#[derive(Debug)]
struct FixupTaskV2Contract {
    required: BTreeSet<String>,
    properties: BTreeSet<String>,
    statuses: BTreeSet<String>,
    conditionals: BTreeMap<String, BTreeSet<String>>,
    date_time_fields: BTreeSet<String>,
}

fn fixuptask_v2_contract() -> Result<FixupTaskV2Contract, String> {
    let schema: Value = serde_json::from_str(FIXUPTASK_V2_SCHEMA)
        .map_err(|error| format!("embedded FixupTask v2 schema is invalid JSON: {error}"))?;
    if schema.get("additionalProperties").and_then(Value::as_bool) != Some(false) {
        return Err("FixupTask v2 schema must close candidate row properties".to_owned());
    }
    let strings = |value: Option<&Value>, label: &str| -> Result<BTreeSet<String>, String> {
        value
            .and_then(Value::as_array)
            .ok_or_else(|| format!("FixupTask v2 schema {label} must be an array"))?
            .iter()
            .map(|item| {
                item.as_str()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| format!("FixupTask v2 schema {label} contains non-string"))
            })
            .collect()
    };
    let required = strings(schema.get("required"), "required")?;
    let properties_object = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "FixupTask v2 schema properties must be an object".to_owned())?;
    let properties = properties_object.keys().cloned().collect();
    if !required.is_subset(&properties) {
        return Err("FixupTask v2 schema required field is not a declared property".to_owned());
    }
    let date_time_fields = properties_object
        .iter()
        .filter_map(|(field, definition)| {
            (definition.get("format").and_then(Value::as_str) == Some("date-time"))
                .then_some(field.clone())
        })
        .collect();
    let statuses = strings(
        schema
            .get("properties")
            .and_then(|value| value.get("status"))
            .and_then(|value| value.get("enum")),
        "properties.status.enum",
    )?;
    let mut conditionals = BTreeMap::new();
    for conditional in schema
        .get("allOf")
        .and_then(Value::as_array)
        .ok_or_else(|| "FixupTask v2 schema allOf must be an array".to_owned())?
    {
        let status = conditional
            .get("if")
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("status"))
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str)
            .ok_or_else(|| "FixupTask v2 lifecycle condition must select status".to_owned())?;
        let fields = strings(
            conditional
                .get("then")
                .and_then(|value| value.get("required")),
            "lifecycle then.required",
        )?;
        if !fields.is_subset(&properties) {
            return Err("FixupTask v2 lifecycle field is not a declared property".to_owned());
        }
        conditionals.insert(status.to_owned(), fields);
    }
    Ok(FixupTaskV2Contract {
        required,
        properties,
        statuses,
        conditionals,
        date_time_fields,
    })
}

fn object_rows(value: &Value) -> Vec<&Value> {
    value
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter(|row| row.is_object()).collect())
        .unwrap_or_default()
}

fn rows<'a>(
    value: &'a Value,
    scope: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<Vec<&'a Value>> {
    let Some(rows) = value.get("rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "fixuptask_v2_rows_not_array",
            scope,
            "FixupTask document must contain a `rows` array",
        ));
        return None;
    };
    Some(rows.iter().collect())
}

fn canonical_timestamp(value: &str) -> Option<(i32, u32, u32, u32, u32, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || !bytes
            .iter()
            .enumerate()
            .filter(|(index, _)| ![4, 7, 10, 13, 16, 19].contains(index))
            .all(|(_, byte)| byte.is_ascii_digit())
    {
        return None;
    }
    let parse = |start, end| value[start..end].parse::<u32>().ok();
    let (year, month, day, hour, minute, second) = (
        i32::try_from(parse(0, 4)?).ok()?,
        parse(5, 7)?,
        parse(8, 10)?,
        parse(11, 13)?,
        parse(14, 16)?,
        parse(17, 19)?,
    );
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    (day >= 1 && day <= days && hour < 24 && minute < 60 && second < 60)
        .then_some((year, month, day, hour, minute, second))
}

fn validate_v2_row(
    contract: &FixupTaskV2Contract,
    row: &Value,
    id: &str,
    evaluation_time: Option<(i32, u32, u32, u32, u32, u32)>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(object) = row.as_object() else {
        findings.insert(Finding::new(
            "fixuptask_v2_malformed_row",
            id,
            "candidate row must be an object",
        ));
        return;
    };
    for field in &contract.required {
        if non_blank_str(row, field).is_none() {
            findings.insert(Finding::new(
                "fixuptask_v2_schema_required_field",
                id,
                format!("candidate row missing non-blank `{field}` required by schema"),
            ));
        }
    }
    for field in object.keys() {
        if !contract.properties.contains(field) {
            findings.insert(Finding::new(
                "fixuptask_v2_extra_field",
                id,
                format!("candidate row contains schema-forbidden field `{field}`"),
            ));
        }
    }
    for (field, value) in object {
        if contract.properties.contains(field)
            && non_blank_str(&json!({ field: value }), field).is_none()
        {
            findings.insert(Finding::new(
                "fixuptask_v2_invalid_field",
                id,
                format!("schema field `{field}` must be a non-blank string"),
            ));
        }
    }
    for field in &contract.date_time_fields {
        if let Some(value) = non_blank_str(row, field)
            && canonical_timestamp(value).is_none()
        {
            findings.insert(Finding::new(
                "fixuptask_v2_invalid_datetime",
                id,
                format!("schema date-time field `{field}` must be canonical UTC RFC3339 `YYYY-MM-DDTHH:MM:SSZ`"),
            ));
        }
    }
    let status = non_blank_str(row, "status");
    if !status.is_some_and(|value| contract.statuses.contains(value)) {
        findings.insert(Finding::new(
            "fixuptask_v2_unknown_status",
            id,
            "new or modified FixupTask status is not in the schema enum",
        ));
        return;
    }
    let Some(status) = status else { return };
    if let Some(required) = contract.conditionals.get(status) {
        for field in required {
            if non_blank_str(row, field).is_none() {
                findings.insert(Finding::new(
                    "fixuptask_v2_lifecycle_required_field",
                    id,
                    format!("status requires non-blank `{field}` by schema"),
                ));
            }
        }
    }
    if status == "accepted-risk" {
        let expires = non_blank_str(row, "accepted_risk_expires_at");
        let parsed = expires.and_then(canonical_timestamp);
        if parsed.is_some() && evaluation_time.is_some_and(|now| parsed <= Some(now)) {
            findings.insert(Finding::new(
                "fixuptask_v2_accepted_risk_expired",
                id,
                "accepted-risk expiry is at or before the explicitly supplied evaluation time",
            ));
        }
    }
}

/// Validates the strict FixupTask v2 contract for rows introduced or changed relative to a
/// protected merge-base snapshot. Exact byte-for-byte JSON equality is the sole grandfathering
/// condition; the evaluator never accepts a candidate-supplied legacy baseline.
///
/// Qualified-human decision values are opaque accountability references. This pure kernel proves
/// their presence and expiry shape only; it cannot, and does not, claim that a person approved one.
pub fn evaluate_fixuptasks_v2_at(
    merge_base: &Value,
    candidate: &Value,
    evaluation_time: &str,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let contract = match fixuptask_v2_contract() {
        Ok(contract) => contract,
        Err(error) => {
            findings.insert(Finding::new(
                "fixuptask_v2_contract_invalid",
                POLICY_KEY,
                error,
            ));
            return findings;
        }
    };
    let evaluation_time = match canonical_timestamp(evaluation_time) {
        Some(timestamp) => Some(timestamp),
        None => {
            findings.insert(Finding::new(
                "fixuptask_v2_invalid_evaluation_time",
                POLICY_KEY,
                "evaluation time must be canonical UTC RFC3339 `YYYY-MM-DDTHH:MM:SSZ`",
            ));
            None
        }
    };
    let mut base_by_id: BTreeMap<&str, &Value> = BTreeMap::new();
    let mut duplicate_base_ids = BTreeSet::new();
    for row in rows(merge_base, "<protected-merge-base>", &mut findings).unwrap_or_default() {
        if !row.is_object() {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_malformed_row",
                "<protected-merge-base>",
                "protected merge-base row must be an object",
            ));
            continue;
        }
        let Some(id) = non_blank_str(row, "id") else {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_malformed_row",
                "<protected-merge-base>",
                "protected merge-base row carries no non-blank id",
            ));
            continue;
        };
        if base_by_id.insert(id, row).is_some() {
            duplicate_base_ids.insert(id);
            findings.insert(Finding::new(
                "fixuptask_v2_protected_duplicate_id",
                id,
                "protected merge-base must not contain duplicate FixupTask identities",
            ));
        }
    }

    let mut candidate_ids = BTreeSet::new();
    for row in rows(candidate, "<candidate>", &mut findings).unwrap_or_default() {
        if !row.is_object() {
            findings.insert(Finding::new(
                "fixuptask_v2_malformed_row",
                "<row-without-id>",
                "candidate row must be an object",
            ));
            continue;
        }
        let Some(id) = non_blank_str(row, "id") else {
            findings.insert(Finding::new(
                "fixuptask_v2_missing_id",
                "<row-without-id>",
                "candidate FixupTask row carries no non-blank `id`",
            ));
            continue;
        };
        if !candidate_ids.insert(id) {
            findings.insert(Finding::new(
                "fixuptask_v2_duplicate_id",
                id,
                "candidate FixupTask ids must be unique",
            ));
        }
        let unchanged_legacy = !duplicate_base_ids.contains(id)
            && base_by_id.get(id).is_some_and(|base_row| *base_row == row);
        if unchanged_legacy {
            continue;
        }
        validate_v2_row(&contract, row, id, evaluation_time, &mut findings);
    }
    for id in base_by_id.keys() {
        if !candidate_ids.contains(*id) {
            findings.insert(Finding::new(
                "fixuptask_v2_row_deleted",
                id,
                "candidate deletes a protected merge-base row; FixupTask v2 is append-only",
            ));
        }
    }
    findings
}

/// Compatibility projection for callers that only need static contract validation. Admission
/// callers MUST use [`evaluate_fixuptask_v2_admission`] so expiry is evaluated against protected
/// CI facts rather than ambient wall-clock time.
pub fn evaluate_fixuptasks_v2(merge_base: &Value, candidate: &Value) -> BTreeSet<Finding> {
    evaluate_fixuptasks_v2_at(merge_base, candidate, "9999-12-31T23:59:59Z")
}

/// Admission adapter. The candidate contributes only its proposed rows and mapping. Merge-base
/// rows, predecessor source/ids, and evaluation time are derived from the protected SCM-facts
/// envelope materialized by CI; malformed protected facts fail closed.
pub fn evaluate_fixuptask_v2_admission(
    protected_scm_facts: &Value,
    candidate_fixuptasks: &Value,
    candidate_legacy_ledger: Option<&[u8]>,
    candidate_mapping: Option<&Value>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(facts) = protected_scm_facts
        .get("fixuptask_v2_admission")
        .and_then(Value::as_object)
    else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_missing",
            POLICY_KEY,
            "protected SCM facts must contain fixuptask_v2_admission",
        ));
        return findings;
    };
    let allowed = BTreeSet::from([
        "merge_base_rows",
        "merge_base",
        "merge_base_tree",
        "legacy_ledger",
        "predecessor_source",
        "predecessor_ids",
        "evaluation_time",
    ]);
    if facts.keys().any(|key| !allowed.contains(key.as_str())) {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM FixupTask facts contain an unknown field",
        ));
        return findings;
    }
    let Some(merge_base_rows) = facts.get("merge_base_rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts merge_base_rows must be an array",
        ));
        return findings;
    };
    let Some(source) = facts
        .get("predecessor_source")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts predecessor_source must be non-blank",
        ));
        return findings;
    };
    let Some(evaluation_time) = facts.get("evaluation_time").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts evaluation_time must be a timestamp",
        ));
        return findings;
    };
    let Some(predecessor_values) = facts.get("predecessor_ids").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts predecessor_ids must be an array",
        ));
        return findings;
    };
    let mut predecessor_ids = BTreeSet::new();
    for predecessor in predecessor_values {
        let Some(id) = predecessor
            .as_str()
            .filter(|value| !value.trim().is_empty())
        else {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_facts_malformed",
                POLICY_KEY,
                "protected SCM facts predecessor_ids must contain non-blank strings",
            ));
            continue;
        };
        if !predecessor_ids.insert(id.to_owned()) {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_facts_malformed",
                POLICY_KEY,
                "protected SCM facts predecessor_ids must be unique",
            ));
        }
    }
    if !findings.is_empty() {
        return findings;
    }
    let Some(legacy) = facts.get("legacy_ledger").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts must bind the legacy ledger",
        ));
        return findings;
    };
    let Some(expected_digest) = legacy.get("candidate_digest").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "protected SCM facts legacy ledger lacks candidate_digest",
        ));
        return findings;
    };
    let actual_digest = candidate_legacy_ledger.map(fixuptask_v2_digest);
    let actual_digest = actual_digest.as_deref().unwrap_or("absent");
    if actual_digest != expected_digest {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_stale",
            POLICY_KEY,
            "protected SCM facts do not describe the candidate legacy-ledger bytes",
        ));
        return findings;
    }
    let merge_base = json!({ "rows": merge_base_rows });
    findings.extend(evaluate_fixuptasks_v2_at(
        &merge_base,
        candidate_fixuptasks,
        evaluation_time,
    ));
    let merge_base_digest = legacy
        .get("merge_base_digest")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let cutover_proposed = actual_digest != merge_base_digest;
    if cutover_proposed {
        let Some(mapping) = candidate_mapping else {
            findings.insert(Finding::new(
                "friction_mapping_required_for_legacy_cutover",
                POLICY_KEY,
                "changing or deleting the legacy ledger requires a complete identity-only mapping",
            ));
            return findings;
        };
        findings.extend(evaluate_friction_predecessor_mapping(
            source,
            &predecessor_ids,
            mapping,
            candidate_fixuptasks,
        ));
    }
    findings
}

pub fn fixuptask_v2_digest(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// Checks an identity-only predecessor mapping. It deliberately accepts predecessor identifiers,
/// never predecessor text, and therefore cannot recreate a readable retirement archive.
pub fn evaluate_friction_predecessor_mapping(
    expected_source: &str,
    predecessor_ids: &BTreeSet<String>,
    mapping: &Value,
    fixuptasks: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(mapping_object) = mapping.as_object() else {
        findings.insert(Finding::new(
            "friction_mapping_malformed",
            "<mapping>",
            "identity-only mapping must be an object",
        ));
        return findings;
    };
    let allowed = BTreeSet::from(["source", "entries"]);
    for field in mapping_object.keys() {
        if !allowed.contains(field.as_str()) {
            findings.insert(Finding::new(
                "friction_mapping_extra_field",
                "<mapping>",
                format!("identity-only mapping forbids field `{field}`"),
            ));
        }
    }
    if non_blank_str(mapping, "source") != Some(expected_source) {
        findings.insert(Finding::new(
            "friction_mapping_source_mismatch",
            "<mapping>",
            "mapping source must match the protected predecessor source identifier",
        ));
    }
    let target_ids: BTreeSet<&str> = object_rows(fixuptasks)
        .into_iter()
        .filter_map(|row| non_blank_str(row, "id"))
        .collect();
    let Some(entries) = mapping.get("entries").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "friction_mapping_malformed",
            "<mapping>",
            "identity-only mapping entries must be an array",
        ));
        return findings;
    };
    let mut mapped = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let entry_key = format!("<mapping-entry-{index}>");
        let Some(entry_object) = entry.as_object() else {
            findings.insert(Finding::new(
                "friction_mapping_malformed",
                &entry_key,
                "identity-only mapping entry must be an object",
            ));
            continue;
        };
        for field in entry_object.keys() {
            if !BTreeSet::from(["predecessor_id", "target_fixuptask_id"]).contains(field.as_str()) {
                findings.insert(Finding::new(
                    "friction_mapping_extra_field",
                    &entry_key,
                    format!("identity-only mapping entry forbids field `{field}`"),
                ));
            }
        }
        let predecessor_id =
            non_blank_str(entry, "predecessor_id").unwrap_or("<entry-without-predecessor-id>");
        if predecessor_id == "<entry-without-predecessor-id>" {
            findings.insert(Finding::new(
                "friction_mapping_malformed",
                &entry_key,
                "identity-only mapping entry requires non-blank predecessor_id",
            ));
        }
        if !mapped.insert(predecessor_id) {
            findings.insert(Finding::new(
                "friction_mapping_duplicate_predecessor_id",
                predecessor_id,
                "each predecessor friction id maps to exactly one FixupTask id",
            ));
        }
        if !predecessor_ids.contains(predecessor_id) {
            findings.insert(Finding::new(
                "friction_mapping_unknown_predecessor_id",
                predecessor_id,
                "mapping entry names no protected predecessor friction id",
            ));
        }
        let target_id = non_blank_str(entry, "target_fixuptask_id");
        if !target_id.is_some_and(|id| target_ids.contains(id)) {
            findings.insert(Finding::new(
                "friction_mapping_missing_target_fixuptask",
                predecessor_id,
                "mapping target_fixuptask_id does not name a candidate FixupTask",
            ));
        }
        if let Some(target_id) = target_id
            && !targets.insert(target_id)
        {
            findings.insert(Finding::new(
                "friction_mapping_duplicate_target_fixuptask_id",
                target_id,
                "each target FixupTask id must receive exactly one predecessor mapping",
            ));
        }
    }
    for predecessor_id in predecessor_ids {
        if !mapped.contains(predecessor_id.as_str()) {
            findings.insert(Finding::new(
                "friction_mapping_omitted_predecessor_id",
                predecessor_id,
                "every protected predecessor friction id requires one identity-only mapping",
            ));
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "required_primary_fields": ["id", "seen_at", "friction", "enforcement_fix", "status"],
            "status_match": "prefix",
            "terminal_requires_evidence": true,
            "accepted_risk_requires_evidence": true,
            "status_taxonomy": {
                "open": "open",
                "queued-G11": "open",
                "escalated-to-leader": "accepted-risk",
                "interim-accepted": "accepted-risk",
                "RESOLVED": "terminal",
                "ADR-": "terminal"
            }
        })
    }

    fn primary(id: &str, status: &str) -> Value {
        json!({
            "id": id,
            "seen_at": "2026-06-10",
            "friction": "something went wrong",
            "pipeline_defect": "a defect",
            "enforcement_fix": "wire a gate",
            "status": status
        })
    }

    fn observed(rows: Vec<Value>) -> Value {
        json!({ "rows": rows })
    }

    #[test]
    fn fully_accounted_open_friction_is_green() {
        let report = evaluate(&policy(), &observed(vec![primary("FRIC-1", "open")]));
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn empty_ledger_is_green() {
        assert_eq!(
            evaluate(&policy(), &observed(vec![])).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn appending_a_well_formed_row_never_fails_the_gate() {
        // The ratchet must never punish logging: a brand-new valid open friction is green.
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-NEW", "queued-G11")]),
        );
        assert!(
            findings.is_empty(),
            "logging a valid friction must not block: {findings:#?}"
        );
    }

    #[test]
    fn unknown_status_on_new_row_fails_closed() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-X", "totally-made-up-status")]),
        );
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "friction_unknown_status" && f.key == "FRIC-X" })
        );
    }

    #[test]
    fn blank_enforcement_fix_on_open_row_is_no_disposition() {
        let mut row = primary("FRIC-ND", "open");
        row["enforcement_fix"] = json!("   ");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "friction_no_disposition" && f.key == "FRIC-ND" })
        );
        // The blank required field is ALSO a schema violation.
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "friction_missing_required_field" && f.key == "FRIC-ND" })
        );
    }

    #[test]
    fn terminal_status_without_evidence_fails_closed() {
        let findings = evaluate_keyed(&policy(), &observed(vec![primary("FRIC-T", "RESOLVED")]));
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "friction_closed_without_evidence" && f.key == "FRIC-T" })
        );
    }

    #[test]
    fn terminal_status_with_evidence_is_green() {
        let mut row = primary("FRIC-T2", "RESOLVED-fully");
        row["evidence"] = json!("PR #669 merged @ 16f2e3b54: enforcement-liveness gate");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(
            findings.is_empty(),
            "terminal+evidence must be green: {findings:#?}"
        );
    }

    #[test]
    fn accepted_risk_without_evidence_fails_closed() {
        let mut row = primary("FRIC-AR", "escalated-to-leader-for-force-complete");
        // accepted-risk does not require enforcement_fix, but DOES require evidence.
        row["enforcement_fix"] = json!("escalated");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(findings.iter().any(|f| {
            f.code == "friction_accepted_risk_without_evidence" && f.key == "FRIC-AR"
        }));
    }

    #[test]
    fn accepted_risk_with_evidence_is_green() {
        let mut row = primary("FRIC-AR2", "interim-accepted");
        row["evidence"] = json!("founder-held 2026-06-10; leader-side transition pending");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(
            findings.is_empty(),
            "accepted-risk+evidence must be green: {findings:#?}"
        );
    }

    #[test]
    fn duplicate_primary_row_fails_closed_but_appends_do_not() {
        // Two PRIMARY rows sharing an id is a defect.
        let dup = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-D", "open"), primary("FRIC-D", "open")]),
        );
        assert!(
            dup.iter()
                .any(|f| { f.code == "friction_duplicate_primary_row" && f.key == "FRIC-D" })
        );

        // A primary + an append (update row) sharing an id is LEGITIMATE event-sourcing.
        let append = json!({
            "id": "FRIC-D2",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "evidence": "PR #700 merged @ deadbeef: gate landed"
        });
        let folded = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-D2", "open"), append]),
        );
        assert!(
            !folded
                .iter()
                .any(|f| f.code == "friction_duplicate_primary_row"),
            "an append must not count as a duplicate primary: {folded:#?}"
        );
    }

    #[test]
    fn orphan_update_only_friction_fails_closed_as_sole_finding() {
        // A friction with ONLY update rows (no primary) must fail closed: it would otherwise fold to
        // a clean terminal-with-evidence state and evade every schema/disposition/closure check.
        let orphan = json!({
            "id": "FRIC-ORPHAN",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "enforcement_fix": "looks disposed",
            "evidence": "looks closed"
        });
        let findings = evaluate_keyed(&policy(), &observed(vec![orphan]));
        let mine: Vec<_> = findings.iter().filter(|f| f.key == "FRIC-ORPHAN").collect();
        assert_eq!(
            mine.len(),
            1,
            "orphan emits exactly one finding: {findings:#?}"
        );
        assert_eq!(mine[0].code, "friction_orphan_update_row");
    }

    #[test]
    fn primary_plus_updates_is_not_an_orphan() {
        // A real friction (primary + later update rows) must NOT be flagged as an orphan.
        let append = json!({
            "id": "FRIC-REAL",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "evidence": "PR #700 merged @ deadbeef"
        });
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-REAL", "open"), append]),
        );
        assert!(
            !findings
                .iter()
                .any(|f| f.code == "friction_orphan_update_row"),
            "primary+updates must not be an orphan: {findings:#?}"
        );
    }

    #[test]
    fn update_row_overrides_primary_status_for_effective_state() {
        // Primary is open (no evidence needed); update closes it (now needs evidence).
        let append_no_evidence = json!({
            "id": "FRIC-U",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED-structurally"
        });
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-U", "open"), append_no_evidence]),
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == "friction_closed_without_evidence" && f.key == "FRIC-U"),
            "update closing a friction must require evidence: {findings:#?}"
        );

        // With evidence on the closing update, it is green.
        let append_with_evidence = json!({
            "id": "FRIC-U2",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED-structurally",
            "evidence": "PR #661 merged @ 28154faa7"
        });
        let green = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-U2", "open"), append_with_evidence]),
        );
        assert!(
            green.is_empty(),
            "closed-with-evidence via update must be green: {green:#?}"
        );
    }

    #[test]
    fn gate_id_mismatch_in_policy_fails_closed() {
        let mut bad = policy();
        bad["gate_id"] = json!("cloud-ci-wrong");
        let findings = evaluate_keyed(&bad, &observed(vec![]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "friction_policy_gate_id_mismatch")
        );
    }

    #[test]
    fn prefix_match_prefers_longest_taxonomy_key() {
        // `escalated-to-leader` (accepted-risk) must win over a hypothetical broader `escalated` key.
        let mut p = policy();
        p["status_taxonomy"]["escalated"] = json!("open");
        assert_eq!(
            classify_status(&p, "escalated-to-leader-for-force-complete"),
            StatusClass::AcceptedRisk
        );
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let rows = vec![
            primary("FRIC-A", "totally-unknown"),
            primary("FRIC-B", "RESOLVED"),
        ];
        let input = observed(rows);
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &input)
            .into_iter()
            .map(|finding| finding.code)
            .collect();
        assert_eq!(evaluate(&policy(), &input).violations, projected);
    }

    #[test]
    fn fixuptask_v2_rejects_new_rows_without_accountability_and_closed_state_evidence() {
        let base = json!({ "rows": [] });
        let candidate = json!({
            "rows": [{
                "id": "F-V2-1",
                "title": "make the successor durable",
                "priority": "high",
                "status": "resolved",
                "source_session": "session",
                "source_change_id": "change",
                "named_in": "ADR-0621",
                "created_at": "2026-07-21T00:00:00Z"
            }]
        });

        let findings = evaluate_fixuptasks_v2(&base, &candidate);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "fixuptask_v2_schema_required_field")
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == "fixuptask_v2_lifecycle_required_field")
        );
    }

    #[test]
    fn fixuptask_v2_grandfathers_only_exact_legacy_rows_from_merge_base() {
        let legacy = json!({
            "id": "F-LEGACY",
            "title": "historical row",
            "status": "free-text-legacy"
        });
        let base = json!({ "rows": [legacy.clone()] });
        let unchanged = json!({ "rows": [legacy] });
        assert!(evaluate_fixuptasks_v2(&base, &unchanged).is_empty());

        let changed = json!({
            "rows": [{
                "id": "F-LEGACY",
                "title": "historical row revised",
                "status": "free-text-legacy"
            }]
        });
        assert!(
            evaluate_fixuptasks_v2(&base, &changed)
                .iter()
                .any(|f| f.code == "fixuptask_v2_unknown_status")
        );
    }

    #[test]
    fn fixuptask_v2_requires_decision_expiry_for_accepted_risk_and_decision_for_blocked() {
        let base = json!({ "rows": [] });
        let common = json!({
            "id": "F-V2-2",
            "title": "strict accountability",
            "priority": "high",
            "source_session": "session",
            "source_change_id": "change",
            "named_in": "ADR-0621",
            "created_at": "2026-07-21T00:00:00Z",
            "accountable_owner": "owner-id",
            "accountable_role": "role-id",
            "acceptance_criteria": "criterion",
            "verification_path": "cargo test",
            "blocker_for": "roadmap"
        });
        let mut accepted_risk = common.clone();
        accepted_risk["status"] = json!("accepted-risk");
        let accepted = evaluate_fixuptasks_v2(&base, &json!({ "rows": [accepted_risk] }));
        assert!(
            accepted
                .iter()
                .any(|f| { f.code == "fixuptask_v2_lifecycle_required_field" })
        );

        let mut blocked = common;
        blocked["status"] = json!("blocked");
        let blocked_findings = evaluate_fixuptasks_v2(&base, &json!({ "rows": [blocked] }));
        assert!(
            blocked_findings
                .iter()
                .any(|f| { f.code == "fixuptask_v2_lifecycle_required_field" })
        );
    }

    #[test]
    fn friction_mapping_is_identity_only_and_fails_closed_on_all_join_gaps() {
        let predecessor_ids = BTreeSet::from(["FRIC-1".to_owned(), "FRIC-2".to_owned()]);
        let fixuptasks = json!({ "rows": [{ "id": "F-1" }] });
        let mapping = json!({
            "source": "git-history:predecessor-ledger",
            "entries": [
                { "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-1" },
                { "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-MISSING" },
                { "predecessor_id": "FRIC-UNKNOWN", "target_fixuptask_id": "F-1" }
            ]
        });

        let findings = evaluate_friction_predecessor_mapping(
            "git-history:expected-predecessor",
            &predecessor_ids,
            &mapping,
            &fixuptasks,
        );
        let codes: BTreeSet<_> = findings.iter().map(|f| f.code.as_str()).collect();
        assert!(codes.contains("friction_mapping_source_mismatch"));
        assert!(codes.contains("friction_mapping_duplicate_predecessor_id"));
        assert!(codes.contains("friction_mapping_omitted_predecessor_id"));
        assert!(codes.contains("friction_mapping_missing_target_fixuptask"));
        assert!(codes.contains("friction_mapping_unknown_predecessor_id"));
    }

    #[test]
    fn friction_mapping_green_fixture_contains_only_ids_and_existing_targets() {
        let predecessor_ids = BTreeSet::from(["FRIC-1".to_owned(), "FRIC-2".to_owned()]);
        let fixuptasks = json!({ "rows": [{ "id": "F-1" }, { "id": "F-2" }] });
        let mapping = json!({
            "source": "git-history:expected-predecessor",
            "entries": [
                { "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-1" },
                { "predecessor_id": "FRIC-2", "target_fixuptask_id": "F-2" }
            ]
        });
        assert!(
            evaluate_friction_predecessor_mapping(
                "git-history:expected-predecessor",
                &predecessor_ids,
                &mapping,
                &fixuptasks,
            )
            .is_empty()
        );
    }

    #[test]
    fn fixuptask_v2_schema_stays_aligned_with_the_pure_kernel_contract() {
        let contract = fixuptask_v2_contract().expect("embedded schema is a complete contract");
        assert!(contract.required.contains("accountable_owner"));
        assert!(contract.required.contains("blocker_for"));
        assert_eq!(contract.statuses.len(), 5);
        assert_eq!(
            contract.date_time_fields,
            BTreeSet::from([
                "accepted_risk_expires_at".to_owned(),
                "created_at".to_owned(),
                "resolved_at".to_owned(),
            ])
        );
        assert_eq!(contract.conditionals["resolved"].len(), 3);
        assert_eq!(contract.conditionals["accepted-risk"].len(), 3);
        assert_eq!(contract.conditionals["blocked"].len(), 1);
    }

    #[test]
    fn fixuptask_v2_rejects_deletion_malformed_rows_and_extra_fields() {
        let base = json!({ "rows": [{ "id": "F-KEEP", "status": "legacy" }] });
        let candidate = json!({ "rows": [null, { "id": "F-NEW", "title": "x", "priority": "p", "status": "open", "source_session": "s", "source_change_id": "c", "named_in": "n", "created_at": "2026-07-21T00:00:00Z", "accountable_owner": "o", "accountable_role": "r", "acceptance_criteria": "a", "verification_path": "v", "blocker_for": "b", "forged": "no" }] });
        let findings = evaluate_fixuptasks_v2_at(&base, &candidate, "2026-07-21T00:00:00Z");
        let codes: BTreeSet<_> = findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect();
        assert!(codes.contains("fixuptask_v2_row_deleted"));
        assert!(codes.contains("fixuptask_v2_malformed_row"));
        assert!(codes.contains("fixuptask_v2_extra_field"));
    }

    #[test]
    fn fixuptask_v2_rejects_expired_or_nondeterministic_accepted_risk() {
        let base = json!({ "rows": [] });
        let row = json!({ "id": "F-RISK", "title": "x", "priority": "p", "status": "accepted-risk", "source_session": "s", "source_change_id": "c", "named_in": "n", "created_at": "2026-07-21T00:00:00Z", "accountable_owner": "o", "accountable_role": "r", "acceptance_criteria": "a", "verification_path": "v", "blocker_for": "b", "qualified_human_decision_ref": "opaque-ref", "accepted_risk_expires_at": "2026-07-20T00:00:00Z", "accepted_risk_evidence": "e" });
        let findings =
            evaluate_fixuptasks_v2_at(&base, &json!({ "rows": [row] }), "2026-07-21T00:00:00Z");
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "fixuptask_v2_accepted_risk_expired")
        );
    }

    #[test]
    fn fixuptask_v2_validates_all_schema_datetime_fields() {
        let base = json!({ "rows": [] });
        let mut resolved = json!({ "id": "F-RESOLVED", "title": "x", "priority": "p", "status": "resolved", "source_session": "s", "source_change_id": "c", "named_in": "n", "created_at": "not-a-time", "accountable_owner": "o", "accountable_role": "r", "acceptance_criteria": "a", "verification_path": "v", "blocker_for": "b", "resolved_at": "also-not-a-time", "resolved_in_change_id": "change", "resolved_evidence": "e" });
        let invalid = evaluate_fixuptasks_v2_at(
            &base,
            &json!({ "rows": [resolved.clone()] }),
            "2026-07-21T00:00:00Z",
        );
        assert_eq!(
            invalid
                .iter()
                .filter(|finding| finding.code == "fixuptask_v2_invalid_datetime")
                .count(),
            2
        );
        resolved["created_at"] = json!("2026-07-20T00:00:00Z");
        resolved["resolved_at"] = json!("2026-07-21T00:00:00Z");
        assert!(
            evaluate_fixuptasks_v2_at(
                &base,
                &json!({ "rows": [resolved] }),
                "2026-07-21T00:00:00Z"
            )
            .is_empty()
        );
    }

    #[test]
    fn admission_adapter_uses_only_protected_scm_facts() {
        let digest = fixuptask_v2_digest(b"legacy");
        let facts = json!({ "fixuptask_v2_admission": { "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "merge_base_rows": [{ "id": "F-LEGACY", "status": "legacy" }], "predecessor_source": "scm:merge-base", "predecessor_ids": ["FRIC-1"], "evaluation_time": "2026-07-21T00:00:00Z", "legacy_ledger": { "merge_base_blob": "cccccccccccccccccccccccccccccccccccccccc", "merge_base_digest": digest, "candidate_digest": digest } } });
        let candidate = json!({ "rows": [{ "id": "F-LEGACY", "status": "legacy" }] });
        let mapping = json!({ "source": "scm:merge-base", "entries": [{ "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-LEGACY" }] });
        assert!(
            evaluate_fixuptask_v2_admission(&facts, &candidate, Some(b"legacy"), Some(&mapping))
                .is_empty()
        );
        let malformed = json!({ "fixuptask_v2_admission": { "merge_base_rows": [], "predecessor_source": "scm:merge-base", "predecessor_ids": ["FRIC-1", "FRIC-1"], "evaluation_time": "2026-07-21T00:00:00Z" } });
        assert!(
            evaluate_fixuptask_v2_admission(
                &malformed,
                &candidate,
                Some(b"legacy"),
                Some(&mapping)
            )
            .iter()
            .any(|finding| finding.code == "fixuptask_v2_protected_facts_malformed")
        );
    }

    #[test]
    fn unchanged_legacy_ledger_needs_no_mapping_but_cutover_needs_a_bijection() {
        let digest = fixuptask_v2_digest(b"legacy");
        let facts = json!({ "fixuptask_v2_admission": { "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "merge_base_rows": [], "predecessor_source": "scm:merge-base", "predecessor_ids": ["FRIC-1"], "evaluation_time": "2026-07-21T00:00:00Z", "legacy_ledger": { "merge_base_blob": "cccccccccccccccccccccccccccccccccccccccc", "merge_base_digest": digest, "candidate_digest": digest } } });
        let candidate = json!({ "rows": [] });
        assert!(
            evaluate_fixuptask_v2_admission(&facts, &candidate, Some(b"legacy"), None).is_empty()
        );
        let mut cutover_facts = facts.clone();
        cutover_facts["fixuptask_v2_admission"]["legacy_ledger"]["candidate_digest"] =
            json!(fixuptask_v2_digest(b"changed"));
        assert!(
            evaluate_fixuptask_v2_admission(&cutover_facts, &candidate, Some(b"changed"), None)
                .iter()
                .any(|finding| finding.code == "friction_mapping_required_for_legacy_cutover")
        );
    }

    #[test]
    fn predecessor_mapping_rejects_duplicate_targets() {
        let predecessors = BTreeSet::from(["FRIC-1".to_owned(), "FRIC-2".to_owned()]);
        let fixuptasks = json!({ "rows": [{ "id": "F-1" }] });
        let mapping = json!({ "source": "scm", "entries": [
            { "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-1" },
            { "predecessor_id": "FRIC-2", "target_fixuptask_id": "F-1" }
        ] });
        assert!(
            evaluate_friction_predecessor_mapping("scm", &predecessors, &mapping, &fixuptasks)
                .iter()
                .any(|finding| finding.code == "friction_mapping_duplicate_target_fixuptask_id")
        );
    }

    #[test]
    fn predecessor_mapping_rejects_non_object_and_extra_identity_fields() {
        let predecessors = BTreeSet::from(["FRIC-1".to_owned()]);
        let fixuptasks = json!({ "rows": [{ "id": "F-1" }] });
        let mapping = json!({ "source": "scm", "entries": [null, { "predecessor_id": "FRIC-1", "target_fixuptask_id": "F-1", "predecessor_text": "forbidden" }], "forged": true });
        let findings =
            evaluate_friction_predecessor_mapping("scm", &predecessors, &mapping, &fixuptasks);
        let codes: BTreeSet<_> = findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect();
        assert!(codes.contains("friction_mapping_malformed"));
        assert!(codes.contains("friction_mapping_extra_field"));
    }
}
