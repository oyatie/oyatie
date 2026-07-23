//! Independently compilable, durable FixupTask v2 admission.
//!
//! This source contains only durable registry admission and its protected facts.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const GATE_ID: &str = "cloud-ci-fixuptask-v2-admission";
pub const CANDIDATE_REGISTRY_PATH: &str = "registry/fixuptasks.jsonl";
pub const PROTECTED_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json";
const POLICY_KEY: &str = "<policy>";
const FIXUPTASK_V2_SCHEMA: &str = include_str!("../fixuptask-v2-schema.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
    Parse { line: usize, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "FixupTask v2 input io: {message}"),
            Self::Parse { line, message } => write!(f, "FixupTask v2 input line {line}: {message}"),
        }
    }
}
impl std::error::Error for CollectError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}
impl Finding {
    pub fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

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

pub fn fixuptask_v2_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn non_blank<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
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
            .filter(|(i, _)| ![4, 7, 10, 13, 16, 19].contains(i))
            .all(|(_, b)| b.is_ascii_digit())
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

#[derive(Debug)]
struct Contract {
    required: BTreeSet<String>,
    properties: BTreeSet<String>,
    statuses: BTreeSet<String>,
    conditionals: BTreeMap<String, BTreeSet<String>>,
    date_fields: BTreeSet<String>,
}
fn string_set(value: Option<&Value>, label: &str) -> Result<BTreeSet<String>, String> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| format!("schema {label} must be array"))?
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("schema {label} contains non-string"))
        })
        .collect()
}
fn contract() -> Result<Contract, String> {
    let schema: Value =
        serde_json::from_str(FIXUPTASK_V2_SCHEMA).map_err(|error| error.to_string())?;
    if schema.get("additionalProperties").and_then(Value::as_bool) != Some(false) {
        return Err("schema must close row properties".to_owned());
    }
    let required = string_set(schema.get("required"), "required")?;
    let definitions = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "schema properties must be object".to_owned())?;
    let properties = definitions.keys().cloned().collect::<BTreeSet<_>>();
    if !required.is_subset(&properties) {
        return Err("schema required field not declared".to_owned());
    }
    let statuses = string_set(
        definitions
            .get("status")
            .and_then(|value| value.get("enum")),
        "status enum",
    )?;
    let date_fields = definitions
        .iter()
        .filter_map(|(field, definition)| {
            (definition.get("format").and_then(Value::as_str) == Some("date-time"))
                .then_some(field.clone())
        })
        .collect();
    let mut conditionals = BTreeMap::new();
    for conditional in schema
        .get("allOf")
        .and_then(Value::as_array)
        .ok_or_else(|| "schema allOf must be array".to_owned())?
    {
        let status = conditional
            .get("if")
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("status"))
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str)
            .ok_or_else(|| "lifecycle condition lacks status".to_owned())?;
        let fields = string_set(
            conditional
                .get("then")
                .and_then(|value| value.get("required")),
            "lifecycle required",
        )?;
        if !fields.is_subset(&properties) {
            return Err("lifecycle field not declared".to_owned());
        }
        conditionals.insert(status.to_owned(), fields);
    }
    Ok(Contract {
        required,
        properties,
        statuses,
        conditionals,
        date_fields,
    })
}
fn rows<'a>(value: &'a Value, scope: &str, findings: &mut BTreeSet<Finding>) -> Vec<&'a Value> {
    match value.get("rows").and_then(Value::as_array) {
        Some(rows) => rows.iter().collect(),
        None => {
            findings.insert(Finding::new(
                "fixuptask_v2_rows_not_array",
                scope,
                "FixupTask document must contain a rows array",
            ));
            Vec::new()
        }
    }
}
fn validate_row(contract: &Contract, row: &Value, id: &str, findings: &mut BTreeSet<Finding>) {
    let Some(object) = row.as_object() else {
        findings.insert(Finding::new(
            "fixuptask_v2_malformed_row",
            id,
            "candidate row must be object",
        ));
        return;
    };
    for field in &contract.required {
        if non_blank(row, field).is_none() {
            findings.insert(Finding::new(
                "fixuptask_v2_schema_required_field",
                id,
                format!("missing `{field}`"),
            ));
        }
    }
    for field in object.keys() {
        if !contract.properties.contains(field) {
            findings.insert(Finding::new(
                "fixuptask_v2_extra_field",
                id,
                format!("forbidden `{field}`"),
            ));
        }
    }
    for field in object
        .keys()
        .filter(|field| contract.properties.contains(*field))
    {
        if non_blank(row, field).is_none() {
            findings.insert(Finding::new(
                "fixuptask_v2_invalid_field",
                id,
                format!("schema field `{field}` must be a non-blank string"),
            ));
        }
    }
    for field in &contract.date_fields {
        if let Some(value) = non_blank(row, field)
            && canonical_timestamp(value).is_none()
        {
            findings.insert(Finding::new(
                "fixuptask_v2_invalid_datetime",
                id,
                format!("invalid `{field}`"),
            ));
        }
    }
    let Some(status) = non_blank(row, "status") else {
        findings.insert(Finding::new(
            "fixuptask_v2_unknown_status",
            id,
            "status not in schema enum",
        ));
        return;
    };
    if !contract.statuses.contains(status) {
        findings.insert(Finding::new(
            "fixuptask_v2_unknown_status",
            id,
            "status not in schema enum",
        ));
        return;
    }
    if let Some(required) = contract.conditionals.get(status) {
        for field in required {
            if non_blank(row, field).is_none() {
                findings.insert(Finding::new(
                    "fixuptask_v2_lifecycle_required_field",
                    id,
                    format!("status requires `{field}`"),
                ));
            }
        }
    }
}

pub fn evaluate_fixuptasks_v2_at(
    merge_base: &Value,
    candidate: &Value,
    evaluation_time: &str,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let contract = match contract() {
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
    let evaluation_time = canonical_timestamp(evaluation_time);
    if evaluation_time.is_none() {
        findings.insert(Finding::new(
            "fixuptask_v2_invalid_evaluation_time",
            POLICY_KEY,
            "evaluation time must be canonical UTC RFC3339",
        ));
    }
    let mut base = BTreeMap::new();
    for row in rows(merge_base, "<protected-merge-base>", &mut findings) {
        let Some(id) = non_blank(row, "id") else {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_malformed_row",
                "<protected-merge-base>",
                "row lacks id",
            ));
            continue;
        };
        if base.insert(id, row).is_some() {
            findings.insert(Finding::new(
                "fixuptask_v2_protected_duplicate_id",
                id,
                "protected rows duplicate id",
            ));
        }
    }
    let mut ids = BTreeSet::new();
    for row in rows(candidate, "<candidate>", &mut findings) {
        let Some(id) = non_blank(row, "id") else {
            findings.insert(Finding::new(
                "fixuptask_v2_missing_id",
                "<row-without-id>",
                "candidate row lacks id",
            ));
            continue;
        };
        if !ids.insert(id) {
            findings.insert(Finding::new(
                "fixuptask_v2_duplicate_id",
                id,
                "candidate ids must be unique",
            ));
        }
        if base.get(id).is_some_and(|old| *old == row) {
            continue;
        }
        validate_row(&contract, row, id, &mut findings);
        if non_blank(row, "status") == Some("accepted-risk")
            && non_blank(row, "accepted_risk_expires_at")
                .and_then(canonical_timestamp)
                .zip(evaluation_time)
                .is_some_and(|(expiry, now)| expiry <= now)
        {
            findings.insert(Finding::new(
                "fixuptask_v2_accepted_risk_expired",
                id,
                "accepted-risk expiry is at or before evaluation time",
            ));
        }
    }
    for id in base.keys() {
        if !ids.contains(id) {
            findings.insert(Finding::new(
                "fixuptask_v2_row_deleted",
                id,
                "candidate deletes protected row",
            ));
        }
    }
    findings
}

/// Evaluates the durable candidate registry using only protected SCM facts.
pub fn evaluate_materialized_gate(root: &Path) -> Result<BTreeSet<Finding>, CollectError> {
    let bytes = fs::read(root.join(CANDIDATE_REGISTRY_PATH))
        .map_err(|error| CollectError::Io(format!("read {CANDIDATE_REGISTRY_PATH}: {error}")))?;
    let candidate = collect_fixuptask_candidate_jsonl(root, CANDIDATE_REGISTRY_PATH)?;
    let text = fs::read_to_string(root.join(PROTECTED_FACTS_PATH))
        .map_err(|error| CollectError::Io(format!("read {PROTECTED_FACTS_PATH}: {error}")))?;
    let facts: Value = serde_json::from_str(&text).map_err(|error| CollectError::Parse {
        line: 1,
        message: error.to_string(),
    })?;
    Ok(evaluate_admission(&facts, &candidate, &bytes))
}
pub fn evaluate_admission(protected: &Value, candidate: &Value, bytes: &[u8]) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(facts) = protected
        .get("fixuptask_v2_durable")
        .and_then(Value::as_object)
    else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_missing",
            POLICY_KEY,
            "protected SCM facts must contain durable facts",
        ));
        return findings;
    };
    let allowed = BTreeSet::from([
        "merge_base",
        "merge_base_tree",
        "merge_base_rows",
        "candidate_registry_digest",
        "evaluation_time",
    ]);
    if facts.len() != allowed.len() || facts.keys().any(|key| !allowed.contains(key.as_str())) {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "durable facts must contain only durable contract",
        ));
        return findings;
    }
    let Some(rows) = facts.get("merge_base_rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "merge_base_rows must be array",
        ));
        return findings;
    };
    let valid_sha = |value: Option<&str>| {
        value.is_some_and(|value| {
            value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    };
    let digest = facts
        .get("candidate_registry_digest")
        .and_then(Value::as_str);
    if !valid_sha(facts.get("merge_base").and_then(Value::as_str))
        || !valid_sha(facts.get("merge_base_tree").and_then(Value::as_str))
        || !digest.is_some_and(|value| {
            value.starts_with("sha256:")
                && value.len() == 71
                && value[7..]
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "durable facts must bind merge-base identity and candidate digest",
        ));
        return findings;
    }
    if digest != Some(fixuptask_v2_digest(bytes).as_str()) {
        findings.insert(Finding::new(
            "fixuptask_v2_candidate_registry_digest_mismatch",
            POLICY_KEY,
            "facts do not bind exact candidate bytes",
        ));
        return findings;
    }
    let Some(time) = facts.get("evaluation_time").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "fixuptask_v2_protected_facts_malformed",
            POLICY_KEY,
            "evaluation_time must be timestamp",
        ));
        return findings;
    };
    findings.extend(evaluate_fixuptasks_v2_at(
        &json!({ "rows": rows }),
        candidate,
        time,
    ));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durable_admission_is_green_without_legacy_inputs() {
        let bytes = br#"{"id":"FT-1","title":"x","priority":"high","status":"open","source_session":"s","source_change_id":"c","named_in":"ADR-0621","created_at":"2026-07-21T00:00:00Z","accountable_owner":"o","accountable_role":"r","acceptance_criteria":"a","verification_path":"v","blocker_for":"none"}"#;
        let candidate =
            collect_fixuptask_candidate_jsonl_from_bytes(bytes).expect("fixture parses");
        let facts = json!({ "fixuptask_v2_durable": { "merge_base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "merge_base_tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "merge_base_rows": [], "candidate_registry_digest": fixuptask_v2_digest(bytes), "evaluation_time": "2026-07-21T00:00:00Z" }});
        assert!(evaluate_admission(&facts, &candidate, bytes).is_empty());
    }

    #[test]
    fn durable_admission_rejects_non_string_optional_schema_fields() {
        let candidate = json!({ "rows": [{
            "id": "FT-1",
            "title": "x",
            "priority": "high",
            "status": "open",
            "source_session": "s",
            "source_change_id": "c",
            "named_in": "ADR-0621",
            "created_at": "2026-07-21T00:00:00Z",
            "accountable_owner": "o",
            "accountable_role": "r",
            "acceptance_criteria": "a",
            "verification_path": "v",
            "blocker_for": "none",
            "accepted_risk_evidence": false
        }] });

        assert!(
            evaluate_fixuptasks_v2_at(&json!({ "rows": [] }), &candidate, "2026-07-21T00:00:00Z",)
                .iter()
                .any(|finding| finding.code == "fixuptask_v2_invalid_field")
        );
    }

    fn collect_fixuptask_candidate_jsonl_from_bytes(bytes: &[u8]) -> Result<Value, CollectError> {
        let line = std::str::from_utf8(bytes).map_err(|error| CollectError::Parse {
            line: 1,
            message: error.to_string(),
        })?;
        let row: Value = serde_json::from_str(line).map_err(|error| CollectError::Parse {
            line: 1,
            message: error.to_string(),
        })?;
        Ok(json!({ "rows": [row] }))
    }
}
