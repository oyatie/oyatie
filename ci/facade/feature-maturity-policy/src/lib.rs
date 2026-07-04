//! Planned-feature maturity gate.
//!
//! The gate is intentionally shape-neutral: production evaluation consumes observed rows supplied by
//! a collector/test and applies contract checks. It does not treat product readiness as a function of
//! repository path names, legacy prefixes, or retired planning surfaces.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// Gate id used by the Buck2 target and CI matrix label.
pub const GATE_ID: &str = "planned-maturity";

/// Stable violation-code contract.
pub const VIOLATION_CODES: [&str; 8] = [
    "planned_maturity_no_product_prds",
    "planned_maturity_product_prd_missing_acceptance_contract",
    "planned_maturity_product_prd_missing_verification_contract",
    "planned_maturity_no_capability_records",
    "planned_maturity_capability_record_too_shallow",
    "planned_maturity_no_retired_plan_scan",
    "planned_maturity_retired_plan_live_input",
    "planned_maturity_live_gate_input_missing",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
    pub findings: BTreeSet<Finding>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.to_owned(),
        }
    }
}

/// Bare-code projection used by fixtures and gate output.
pub fn evaluate(input: &Value) -> Report {
    let findings = evaluate_keyed(input);
    let violations = findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect::<BTreeSet<_>>();
    let verdict = if findings.is_empty() {
        Verdict::Green
    } else {
        Verdict::Red
    };
    Report {
        verdict,
        violations,
        findings,
    }
}

/// Single source of truth for planned-maturity findings.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    evaluate_product_prds(input, &mut findings);
    evaluate_capability_records(input, &mut findings);
    evaluate_retired_plan_refs(input, &mut findings);
    findings
}

fn evaluate_product_prds(input: &Value, findings: &mut BTreeSet<Finding>) {
    let rows = array_field(input, "product_prds");
    let minimum = minimum_count(input, "minimum_product_prds", 1);
    let minimum_acceptance_rows = minimum_count(input, "minimum_acceptance_rows_per_prd", 1);
    let minimum_verification_rows = minimum_count(input, "minimum_verification_rows_per_prd", 1);
    if rows.len() < minimum {
        findings.insert(Finding::new(
            "planned_maturity_no_product_prds",
            "<product_prds>",
            "observed product PRD corpus is below the declared floor",
        ));
    }

    for row in rows {
        let key = key_field(row, "path", "<product_prd>");
        let acceptance_rows = count_field(row, "acceptance_row_count");
        let product_acceptance_rows = count_field(row, "product_specific_acceptance_row_count");
        if !bool_field(row, "has_acceptance_heading")
            || acceptance_rows < minimum_acceptance_rows
            || product_acceptance_rows == 0
        {
            findings.insert(Finding::new(
                "planned_maturity_product_prd_missing_acceptance_contract",
                &key,
                "product PRD lacks parsed product-specific AC/Test rows under the acceptance heading",
            ));
        }

        let verification_rows = count_field(row, "verification_command_row_count");
        let product_verification_rows = count_field(row, "product_specific_verification_row_count");
        if !bool_field(row, "has_verification_heading")
            || verification_rows < minimum_verification_rows
            || product_verification_rows == 0
        {
            findings.insert(Finding::new(
                "planned_maturity_product_prd_missing_verification_contract",
                &key,
                "product PRD lacks parsed product-specific verification command rows",
            ));
        }
    }
}

fn evaluate_capability_records(input: &Value, findings: &mut BTreeSet<Finding>) {
    let rows = array_field(input, "capability_records");
    let minimum = minimum_count(input, "minimum_capability_records", 1);
    if rows.len() < minimum {
        findings.insert(Finding::new(
            "planned_maturity_no_capability_records",
            "<capability_records>",
            "observed capability registry is below the declared floor",
        ));
    }

    for row in rows {
        let key = key_field(row, "id", "<capability_record>");
        let missing = capability_missing_fields(row);
        if !missing.is_empty() {
            findings.insert(Finding::new(
                "planned_maturity_capability_record_too_shallow",
                &key,
                &format!(
                    "capability record misses required maturity fields: {}",
                    missing.join(", ")
                ),
            ));
        }
    }
}

fn evaluate_retired_plan_refs(input: &Value, findings: &mut BTreeSet<Finding>) {
    let rows = array_field(input, "retired_plan_refs");
    if !bool_field(input, "retired_plan_scan_executed") {
        findings.insert(Finding::new(
            "planned_maturity_no_retired_plan_scan",
            "<retired_plan_refs>",
            "retired plan reference scan did not run",
        ));
    }

    for row in rows {
        let key = key_field(row, "key", "<retired_plan_ref>");
        if str_field(row, "usage") == Some("live_input")
            || str_field(row, "status") != Some("historical_provenance_only")
        {
            findings.insert(Finding::new(
                "planned_maturity_retired_plan_live_input",
                &key,
                "retired plan path is not explicitly classified as historical provenance only",
            ));
        }
        if !bool_field(row, "live_ref_resolves") {
            findings.insert(Finding::new(
                "planned_maturity_live_gate_input_missing",
                &key,
                "retired plan path lacks a resolving live gate-input reference",
            ));
        }
    }
}

fn capability_missing_fields(row: &Value) -> Vec<&'static str> {
    let required = [
        "id",
        "namespace",
        "name",
        "status",
        "owner_team",
        "autonomy_tier",
        "autonomy_tier_required",
        "data_classes_touched",
        "evidence_emit_required",
        "evidence_emission_topic",
        "prd_ref",
        "task_ref",
        "test_ref",
        "verification_ref",
        "cost_profile.per_invocation_budget_usd",
        "cost_profile.monthly_budget_usd",
        "mcp_contract.agent_readable",
        "mcp_contract.human_readable",
        "mcp_contract.input_schema_ref",
        "mcp_contract.output_schema_ref",
        "failure_modes",
        "maturity.claim_boundary",
        "maturity.admission_ref",
    ];

    required
        .into_iter()
        .filter(|path| !field_present(row, path))
        .collect()
}

fn field_present(row: &Value, dotted: &str) -> bool {
    let Some(value) = value_at(row, dotted) else {
        return false;
    };
    value_has_maturity_content(value)
}
fn value_has_maturity_content(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(_) | Value::Number(_) => true,
        Value::String(value) => string_field_has_maturity_content(value),
        Value::Array(values) => !values.is_empty() && values.iter().all(value_has_maturity_content),
        Value::Object(values) => {
            !values.is_empty() && values.values().all(value_has_maturity_content)
        }
    }
}
fn string_field_has_maturity_content(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let placeholder_tokens = [
        "todo",
        "tbd",
        "placeholder",
        "replace-me",
        "replace_me",
        "unknown",
        "n/a",
    ];
    !placeholder_tokens.iter().any(|token| lower.contains(token))
}

fn value_at<'a>(row: &'a Value, dotted: &str) -> Option<&'a Value> {
    let mut current = row;
    for part in dotted.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn array_field<'a>(input: &'a Value, key: &str) -> &'a [Value] {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn bool_field(input: &Value, key: &str) -> bool {
    input.get(key).and_then(Value::as_bool) == Some(true)
}

fn str_field<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input.get(key).and_then(Value::as_str)
}

fn key_field(input: &Value, key: &str, fallback: &str) -> String {
    str_field(input, key)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_owned()
}
fn count_field(input: &Value, key: &str) -> usize {
    input
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}

fn minimum_count(input: &Value, key: &str, default: usize) -> usize {
    input
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn green_fixture_passes() {
        let fixture = json!({
            "minimum_product_prds": 1,
            "product_prds": [{
                "path": "product-a",
                "has_acceptance_heading": true,
                "acceptance_row_count": 2,
                "product_specific_acceptance_row_count": 1,
                "has_verification_heading": true,
                "verification_command_row_count": 2,
                "product_specific_verification_row_count": 1
            }],
            "minimum_capability_records": 1,
            "capability_records": [rich_capability_record()],
            "retired_plan_scan_executed": true,
            "retired_plan_refs": [{
                "key": "plan-ref",
                "status": "historical_provenance_only",
                "live_ref_resolves": true
            }]
        });

        assert_eq!(evaluate(&fixture).verdict, Verdict::Green);
    }

    #[test]
    fn red_fixture_exposes_all_three_issue_classes() {
        let fixture = json!({
            "minimum_product_prds": 2,
            "product_prds": [{
                "path": "product-a",
                "has_acceptance_heading": true,
                "acceptance_row_count": 0,
                "product_specific_acceptance_row_count": 0,
                "has_verification_heading": true,
                "verification_command_row_count": 0,
                "product_specific_verification_row_count": 0
            }],
            "minimum_capability_records": 1,
            "capability_records": [{ "id": "capability.minimal" }],
            "retired_plan_scan_executed": true,
            "retired_plan_refs": [{
                "key": "retired-live-ref",
                "status": "live_input",
                "live_ref_resolves": false
            }]
        });

        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(
            report
                .violations
                .contains("planned_maturity_no_product_prds")
        );
        assert!(
            report
                .violations
                .contains("planned_maturity_product_prd_missing_acceptance_contract")
        );
        assert!(
            report
                .violations
                .contains("planned_maturity_product_prd_missing_verification_contract")
        );
        assert!(
            report
                .violations
                .contains("planned_maturity_capability_record_too_shallow")
        );
        assert!(
            report
                .violations
                .contains("planned_maturity_retired_plan_live_input")
        );
        assert!(
            report
                .violations
                .contains("planned_maturity_live_gate_input_missing")
        );
    }
    #[test]
    fn all_fields_present_placeholder_capability_is_red() {
        let mut record = rich_capability_record();
        record["prd_ref"] = Value::String("TODO: add real PRD".to_owned());
        let fixture = json!({
            "minimum_product_prds": 0,
            "product_prds": [],
            "minimum_capability_records": 1,
            "capability_records": [record],
            "retired_plan_scan_executed": true,
            "retired_plan_refs": []
        });

        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(
            report
                .violations
                .contains("planned_maturity_capability_record_too_shallow")
        );
    }

    fn rich_capability_record() -> Value {
        json!({
            "id": "capability.rich",
            "namespace": "capability",
            "name": "Rich capability",
            "status": "published",
            "owner_team": "team-a",
            "autonomy_tier": "T1Read",
            "autonomy_tier_required": "T1",
            "data_classes_touched": ["INTERNAL_ONLY"],
            "evidence_emit_required": true,
            "evidence_emission_topic": "capability.rich.invoke",
            "prd_ref": "spec#prd",
            "task_ref": "task#plan",
            "test_ref": "test#case",
            "verification_ref": "verify#case",
            "cost_profile": {
                "per_invocation_budget_usd": "0.01",
                "monthly_budget_usd": "10"
            },
            "mcp_contract": {
                "agent_readable": "Agent-readable contract",
                "human_readable": "Human-readable contract",
                "input_schema_ref": "schema#input",
                "output_schema_ref": "schema#output"
            },
            "failure_modes": ["evidence_emission_failed"],
            "maturity": {
                "claim_boundary": "Registry metadata only.",
                "admission_ref": "ADR-0000"
            }
        })
    }
}
