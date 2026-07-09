//! # cloud-ci-contract-slice-conformance
//!
//! Paved-road Rust/Buck2 gate that replaces the fleet-wide
//! `scripts/tests/*_check.py` "contract slice" validators with a single owned,
//! declarative, owned-Rust gate.
//!
//! A worker declares a slice as one entry in `contract-slice-policy.json`
//! (the committed spec path, its required fields, enum constraints, forbidden
//! content markers, and — for a migration — the retired Python source) and
//! ships the slice's committed spec JSON. No new Python, no shell, no CLI, no
//! new crate: the gate reads the declared slices and validates the live
//! committed specs.
//!
//! The surface is API/config shaped: callers pass the policy plus the typed
//! JSON corpus to [`evaluate_configured`]. The gate is pure — it never shells
//! out, spawns an interpreter, mutates files, or reads ambient repository
//! state. Repository-specific paths and per-slice rules live in
//! `contract-slice-policy.json`.
//!
//! Mirrors the `resource-contract-conformance` gate (ADR-0515 WS-D pure gate
//! shape; the `source_migration_slice` Python→Rust retirement pattern).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Cloud-ci gate id for the contract-slice conformance gate.
pub const GATE_ID: &str = "cloud-ci-contract-slice-conformance";

/// The one legitimate `primary_execution_path` for a contract-slice policy.
pub const REQUIRED_PRIMARY_EXECUTION_PATH: &str = "rust_buck2_cloud_ci_gate";

/// Content markers that must never appear inside a contract-slice *spec*:
/// retired CLI authority and non-Rust interpreter invocations. This enforces
/// the no-shell / no-interpreter / no-retired-CLI doctrine inside the contract
/// content itself (e.g. a `python3 …` verification command baked into a spec).
///
/// Matched case-insensitively. Retired Python sources are declared in the
/// policy's `source_migration_slice`, which is validated separately and never
/// scanned here, so naming a retired `*.py` in the ledger is not a violation.
const FORBIDDEN_SPEC_MARKERS: &[&str] = &[
    "oya gate",
    "oya-dev-cli",
    "python3",
    "cargo run",
    "terraform apply",
    "opentofu apply",
    "kubectl apply",
    "aws cli",
    "gcloud cli",
];

/// Bare-code verdict of a gate run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed gate violation: a stable `code` plus the offending unit `key`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String, // data_class: INTERNAL_ONLY
    pub key: String,  // data_class: INTERNAL_ONLY
}

impl Finding {
    fn new(code: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            key: key.into(),
        }
    }
}

/// The outcome of a gate run: verdict, the keyed findings, and the bare set of
/// violation codes (for terse assertions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,             // data_class: INTERNAL_ONLY
    pub findings: BTreeSet<Finding>,  // data_class: INTERNAL_ONLY
    pub violations: BTreeSet<String>, // data_class: INTERNAL_ONLY
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let violations = findings.iter().map(|f| f.code.clone()).collect();
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            findings,
            violations,
        }
    }
}

/// Evaluate a contract-slice policy against a corpus of committed slice specs.
///
/// `corpus` maps each slice's declared `spec_path` to its parsed JSON. The gate
/// is pure and total: any structural gap becomes a keyed [`Finding`] rather than
/// a panic, so a malformed policy is RED, never a crash.
#[must_use]
pub fn evaluate_configured(policy: &Value, corpus: &BTreeMap<String, Value>) -> Report {
    let mut findings = BTreeSet::new();

    if policy.get("primary_execution_path").and_then(Value::as_str)
        != Some(REQUIRED_PRIMARY_EXECUTION_PATH)
    {
        findings.insert(Finding::new(
            "contract_slice_primary_path_not_rust",
            "primary_execution_path",
        ));
    }

    let slices = policy.get("slices").and_then(Value::as_array);
    match slices {
        Some(slices) if !slices.is_empty() => {
            for slice in slices {
                evaluate_slice(slice, corpus, &mut findings);
            }
        }
        _ => {
            findings.insert(Finding::new("contract_slice_policy_has_no_slices", "slices"));
        }
    }

    Report::from_findings(findings)
}

fn evaluate_slice(slice: &Value, corpus: &BTreeMap<String, Value>, findings: &mut BTreeSet<Finding>) {
    let slice_id = slice
        .get("slice_id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown-slice>");

    let Some(spec_path) = slice.get("spec_path").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "contract_slice_missing_spec_path",
            slice_id.to_owned(),
        ));
        return;
    };

    let Some(spec) = corpus.get(spec_path) else {
        findings.insert(Finding::new(
            "contract_slice_spec_absent",
            format!("{slice_id}:{spec_path}"),
        ));
        return;
    };

    // 1. Forbidden content markers (universal doctrine set + per-slice extras).
    for marker in FORBIDDEN_SPEC_MARKERS {
        if recursively_contains(spec, marker) {
            findings.insert(Finding::new(
                "contract_slice_forbidden_marker",
                format!("{slice_id}:{marker}"),
            ));
        }
    }
    for marker in string_array(slice, "forbidden_markers") {
        if recursively_contains(spec, &marker) {
            findings.insert(Finding::new(
                "contract_slice_forbidden_marker",
                format!("{slice_id}:{marker}"),
            ));
        }
    }

    // 2. Required fields (dotted paths) must resolve to a non-null value.
    for field in string_array(slice, "required_fields") {
        if get_dotted(spec, &field).is_none_or(Value::is_null) {
            findings.insert(Finding::new(
                "contract_slice_missing_required_field",
                format!("{slice_id}:{field}"),
            ));
        }
    }

    // 3. Enum constraints: the dotted field's string value must be allowed.
    if let Some(constraints) = slice.get("enum_constraints").and_then(Value::as_array) {
        for constraint in constraints {
            let field = constraint.get("field").and_then(Value::as_str).unwrap_or("");
            let allowed: Vec<&str> = constraint
                .get("allowed")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let actual = get_dotted(spec, field).and_then(Value::as_str);
            if !actual.is_some_and(|value| allowed.contains(&value)) {
                findings.insert(Finding::new(
                    "contract_slice_enum_violation",
                    format!("{slice_id}:{field}"),
                ));
            }
        }
    }

    // 4. Required array members: a dotted string-array field must contain (be a
    //    superset of) every declared member. Covers "this contract must enumerate
    //    exactly these source ADRs / nonclaims / filters" without hardcoding them
    //    in Rust — they stay data in the policy.
    if let Some(requirements) = slice.get("required_array_members").and_then(Value::as_array) {
        for requirement in requirements {
            let field = requirement.get("field").and_then(Value::as_str).unwrap_or("");
            let present: Vec<&str> = get_dotted(spec, field)
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            for member in requirement
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if !present.contains(&member) {
                    findings.insert(Finding::new(
                        "contract_slice_missing_array_member",
                        format!("{slice_id}:{field}:{member}"),
                    ));
                }
            }
        }
    }

    // 5. Migration declarations (optional): each retired source must declare a
    //    retired_primary_path disposition, a Buck2 gate replacement target, and
    //    an interpreter-script legacy path. This proves a Python validator is
    //    being retired onto this gate rather than run in parallel.
    if let Some(rows) = slice.get("source_migration_slice").and_then(Value::as_array) {
        for (index, row) in rows.iter().enumerate() {
            let key = format!("{slice_id}:migration[{index}]");
            if row.get("disposition").and_then(Value::as_str) != Some("retired_primary_path") {
                findings.insert(Finding::new(
                    "contract_slice_migration_not_retired",
                    key.clone(),
                ));
            }
            let target = row
                .get("replacement_target")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !(target.starts_with("//ci/facade/") && target.ends_with("-gate")) {
                findings.insert(Finding::new(
                    "contract_slice_migration_bad_target",
                    key.clone(),
                ));
            }
            let legacy = row
                .get("legacy_path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !(legacy.ends_with(".py") || legacy.ends_with(".sh")) {
                findings.insert(Finding::new("contract_slice_migration_bad_legacy", key));
            }
        }
    }
}

/// Collect a slice field that is an array-of-strings into owned strings.
fn string_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a dotted path (`a.b.c`) to a nested value.
fn get_dotted<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// True when `needle` (matched case-insensitively) appears in any string leaf
/// of `value` — including object keys.
fn recursively_contains(value: &Value, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    contains_lowered(value, &needle)
}

fn contains_lowered(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text.to_ascii_lowercase().contains(needle),
        Value::Array(items) => items.iter().any(|item| contains_lowered(item, needle)),
        Value::Object(map) => map
            .iter()
            .any(|(key, val)| key.to_ascii_lowercase().contains(needle) || contains_lowered(val, needle)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_slice_spec() -> Value {
        json!({
            "slice_id": "exemplar",
            "spec_kind": "contract-slice",
            "cloud_ci_gate": GATE_ID,
            "required_contract_fields": ["field_a", "field_b"],
            "non_claims": ["fixture only; not live evidence"]
        })
    }

    fn policy_with(slice: Value) -> Value {
        json!({
            "gate_id": GATE_ID,
            "primary_execution_path": REQUIRED_PRIMARY_EXECUTION_PATH,
            "slices": [slice]
        })
    }

    fn exemplar_slice() -> Value {
        json!({
            "slice_id": "exemplar",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": ["slice_id", "spec_kind", "cloud_ci_gate", "required_contract_fields", "non_claims"],
            "enum_constraints": [{ "field": "spec_kind", "allowed": ["contract-slice"] }]
        })
    }

    fn corpus_with(spec: Value) -> BTreeMap<String, Value> {
        BTreeMap::from([("fixtures/exemplar-slice.json".to_owned(), spec)])
    }

    #[test]
    fn get_dotted_walks_nested_objects() {
        let v = json!({ "a": { "b": { "c": 1 } } });
        assert_eq!(get_dotted(&v, "a.b.c"), Some(&json!(1)));
        assert_eq!(get_dotted(&v, "a.b.missing"), None);
    }

    #[test]
    fn recursively_contains_is_case_insensitive_and_scans_keys() {
        let v = json!({ "verification": "Python3 check.py" });
        assert!(recursively_contains(&v, "python3"));
        let keyed = json!({ "kubectl apply": true });
        assert!(recursively_contains(&keyed, "kubectl apply"));
        assert!(!recursively_contains(&json!({ "ok": "rust gate" }), "python3"));
    }

    #[test]
    fn valid_slice_is_green() {
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(valid_slice_spec()));
        assert_eq!(report.verdict, Verdict::Green, "{:#?}", report.findings);
    }

    #[test]
    fn missing_required_field_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut().unwrap().remove("non_claims");
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_missing_required_field"));
    }

    #[test]
    fn baked_in_interpreter_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut()
            .unwrap()
            .insert("verification".to_owned(), json!("python3 scripts/tests/x_check.py"));
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_forbidden_marker"));
    }

    #[test]
    fn enum_violation_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut()
            .unwrap()
            .insert("spec_kind".to_owned(), json!("not-a-contract-slice"));
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_enum_violation"));
    }

    #[test]
    fn wrong_primary_path_is_red() {
        let mut policy = policy_with(exemplar_slice());
        policy
            .as_object_mut()
            .unwrap()
            .insert("primary_execution_path".to_owned(), json!("python_script"));
        let report = evaluate_configured(&policy, &corpus_with(valid_slice_spec()));
        assert!(report.violations.contains("contract_slice_primary_path_not_rust"));
    }

    #[test]
    fn absent_spec_is_red_not_panic() {
        let report = evaluate_configured(&policy_with(exemplar_slice()), &BTreeMap::new());
        assert!(report.violations.contains("contract_slice_spec_absent"));
    }

    #[test]
    fn bad_migration_declaration_is_red() {
        let slice = json!({
            "slice_id": "mig",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "source_migration_slice": [
                { "legacy_path": "scripts/tests/x.py", "replacement_target": "//somewhere:else", "disposition": "kept" }
            ]
        });
        let report = evaluate_configured(&policy_with(slice), &corpus_with(valid_slice_spec()));
        assert!(report.violations.contains("contract_slice_migration_not_retired"));
        assert!(report.violations.contains("contract_slice_migration_bad_target"));
    }

    #[test]
    fn required_array_members_superset_is_green_missing_is_red() {
        let slice = json!({
            "slice_id": "arr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_array_members": [
                { "field": "required_contract_fields", "members": ["field_a", "field_b"] }
            ]
        });
        // valid_slice_spec has required_contract_fields = [field_a, field_b] -> green
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(valid_slice_spec())).verdict,
            Verdict::Green
        );
        // drop field_b -> missing member is red
        let mut spec = valid_slice_spec();
        spec["required_contract_fields"] = json!(["field_a"]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_missing_array_member"));
    }
}
