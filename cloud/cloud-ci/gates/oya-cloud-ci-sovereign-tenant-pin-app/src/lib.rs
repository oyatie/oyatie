//! # sovereign-tenant-pin
//!
//! Pure cloud-ci readiness gate packet for `/specs/multi-region-disposition-canonical.json`.
//! The future producer owns repository/OpenAPI scanning and route-table inventory. This crate
//! evaluates DATA rows shaped as tenant-registry fields plus the gateway routing decision and
//! proves the non-live contract: a request for a sovereign tenant may be admitted only in an
//! allowed/home region, while a mismatched cell fails closed with `421 Misdirected Request` and a
//! `Location` header. It does not mutate live API gateway routes, provision regions/cells, move
//! tenants, or claim production routing readiness.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// Accepted spec validator name: `cloud-ci/Rust gate packet: sovereign-tenant-pin`.
pub const GATE_ID: &str = "sovereign-tenant-pin";

pub const TENANT_PIN_GATE_ID_MISMATCH: &str = "tenant_pin_gate_id_mismatch";
pub const TENANT_PIN_NO_SCENARIOS: &str = "tenant_pin_no_scenarios";
pub const TENANT_PIN_NO_ADMITTED_SCENARIO: &str = "tenant_pin_no_admitted_scenario";
pub const TENANT_PIN_NO_MISDIRECTED_SCENARIO: &str = "tenant_pin_no_misdirected_scenario";
pub const TENANT_PIN_ROW_MISSING_FIELD: &str = "tenant_pin_row_missing_field";
pub const TENANT_PIN_INVALID_TENANT_ID: &str = "tenant_pin_invalid_tenant_id";
pub const TENANT_PIN_HOME_REGION_NOT_ALLOWED: &str = "tenant_pin_home_region_not_allowed";
pub const TENANT_PIN_STRICT_RESIDENCY_NOT_SINGLE_HOME_REGION: &str =
    "tenant_pin_strict_residency_not_single_home_region";
pub const TENANT_PIN_ALLOWED_CELL_NOT_ADMITTED: &str = "tenant_pin_allowed_cell_not_admitted";
pub const TENANT_PIN_ADMITTED_STATUS_NOT_202: &str = "tenant_pin_admitted_status_not_202";
pub const TENANT_PIN_CURRENT_CELL_NOT_ALLOWED: &str = "tenant_pin_current_cell_not_allowed";
pub const TENANT_PIN_MISDIRECTED_STATUS_NOT_421: &str = "tenant_pin_misdirected_status_not_421";
pub const TENANT_PIN_LOCATION_HEADER_MISSING: &str = "tenant_pin_location_header_missing";

/// Stable blocking violation codes emitted by this readiness gate packet.
pub const VIOLATION_CODES: [&str; 13] = [
    TENANT_PIN_GATE_ID_MISMATCH,
    TENANT_PIN_NO_SCENARIOS,
    TENANT_PIN_NO_ADMITTED_SCENARIO,
    TENANT_PIN_NO_MISDIRECTED_SCENARIO,
    TENANT_PIN_ROW_MISSING_FIELD,
    TENANT_PIN_INVALID_TENANT_ID,
    TENANT_PIN_HOME_REGION_NOT_ALLOWED,
    TENANT_PIN_STRICT_RESIDENCY_NOT_SINGLE_HOME_REGION,
    TENANT_PIN_ALLOWED_CELL_NOT_ADMITTED,
    TENANT_PIN_ADMITTED_STATUS_NOT_202,
    TENANT_PIN_CURRENT_CELL_NOT_ALLOWED,
    TENANT_PIN_MISDIRECTED_STATUS_NOT_421,
    TENANT_PIN_LOCATION_HEADER_MISSING,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioKind {
    AllowedCurrentCell,
    MismatchedCurrentCell,
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
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
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
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

fn string_field<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row.get(field).and_then(Value::as_str).map(str::trim)
}

fn scenario_key(row: &Value, index: usize) -> String {
    string_field(row, "scenario_id")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("<scenario-{index}>"))
}

fn required_nonblank(
    row: &Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<String> {
    match string_field(row, field).filter(|value| !value.is_empty()) {
        Some(value) => Some(value.to_owned()),
        None => {
            findings.insert(Finding::new(
                TENANT_PIN_ROW_MISSING_FIELD,
                format!("{key}:{field}"),
                format!("scenario is missing accepted tenant-registry field `{field}`"),
            ));
            None
        }
    }
}

fn allowed_regions(row: &Value, key: &str, findings: &mut BTreeSet<Finding>) -> Vec<String> {
    let regions = row
        .get("allowed_regions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|region| !region.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if regions.is_empty() {
        findings.insert(Finding::new(
            TENANT_PIN_ROW_MISSING_FIELD,
            format!("{key}:allowed_regions"),
            "scenario is missing non-empty accepted tenant-registry field `allowed_regions`",
        ));
    }

    regions
}

fn contains_region(regions: &[String], region: &str) -> bool {
    regions
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(region.trim()))
}

fn nonblank_location(row: &Value) -> bool {
    string_field(row, "location").is_some_and(|value| !value.is_empty())
}

fn status(row: &Value) -> Option<u64> {
    row.get("status").and_then(Value::as_u64)
}

fn evaluate_scenario(
    index: usize,
    row: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Option<ScenarioKind> {
    let key = scenario_key(row, index);
    let tenant_id = required_nonblank(row, "tenant_id", &key, findings);
    let home_region = required_nonblank(row, "home_region", &key, findings);
    let residency_class = required_nonblank(row, "residency_class", &key, findings);
    let pack_id = required_nonblank(row, "pack_id", &key, findings);
    let current_cell_region = required_nonblank(row, "current_cell_region", &key, findings);
    let decision = required_nonblank(row, "decision", &key, findings);
    let allowed_regions = allowed_regions(row, &key, findings);

    let tenant_id = tenant_id?;
    let home_region = home_region?;
    let residency_class = residency_class?;
    let current_cell_region = current_cell_region?;
    let decision = decision?;
    let _pack_id = pack_id?;
    if allowed_regions.is_empty() {
        return None;
    }

    if !tenant_id.starts_with("ten_") || tenant_id.len() <= "ten_".len() {
        findings.insert(Finding::new(
            TENANT_PIN_INVALID_TENANT_ID,
            key.clone(),
            "tenant_id must use the accepted tenant registry `ten_` prefix",
        ));
    }

    if !contains_region(&allowed_regions, &home_region) {
        findings.insert(Finding::new(
            TENANT_PIN_HOME_REGION_NOT_ALLOWED,
            key.clone(),
            "home_region must be listed in allowed_regions",
        ));
    }

    if residency_class.starts_with("strict_")
        && (allowed_regions.len() != 1 || !contains_region(&allowed_regions, &home_region))
    {
        findings.insert(Finding::new(
            TENANT_PIN_STRICT_RESIDENCY_NOT_SINGLE_HOME_REGION,
            key.clone(),
            "strict sovereign residency must pin to exactly one allowed home region",
        ));
    }

    if contains_region(&allowed_regions, &current_cell_region) {
        if decision != "admit" {
            findings.insert(Finding::new(
                TENANT_PIN_ALLOWED_CELL_NOT_ADMITTED,
                key.clone(),
                "allowed-region requests must be admitted by the readiness scenario",
            ));
        }
        if status(row) != Some(202) {
            findings.insert(Finding::new(
                TENANT_PIN_ADMITTED_STATUS_NOT_202,
                key.clone(),
                "allowed-region requests must return the admitted 202 readiness status",
            ));
        }
        return Some(ScenarioKind::AllowedCurrentCell);
    }

    if decision != "misdirect" {
        findings.insert(Finding::new(
            TENANT_PIN_CURRENT_CELL_NOT_ALLOWED,
            key.clone(),
            "mismatched-cell requests must fail closed as a misdirect decision",
        ));
    }
    if status(row) != Some(421) {
        findings.insert(Finding::new(
            TENANT_PIN_MISDIRECTED_STATUS_NOT_421,
            key.clone(),
            "mismatched-cell requests must return 421 Misdirected Request",
        ));
    }
    if !nonblank_location(row) {
        findings.insert(Finding::new(
            TENANT_PIN_LOCATION_HEADER_MISSING,
            key,
            "mismatched-cell 421 responses must include a Location header for the home cell",
        ));
    }
    Some(ScenarioKind::MismatchedCurrentCell)
}

/// Pure evaluator for producer-emitted tenant pin routing scenarios.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if string_field(input, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            TENANT_PIN_GATE_ID_MISMATCH,
            "<gate_id>",
            format!("gate_id must be `{GATE_ID}`"),
        ));
    }

    let scenarios = input
        .get("scenarios")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if scenarios.is_empty() {
        findings.insert(Finding::new(
            TENANT_PIN_NO_SCENARIOS,
            "<empty-scenarios>",
            "at least one accepted and one denied routing scenario must be emitted",
        ));
        return findings;
    }

    let mut saw_allowed_current_cell = false;
    let mut saw_mismatched_current_cell = false;
    for (index, scenario) in scenarios.iter().enumerate() {
        match evaluate_scenario(index, scenario, &mut findings) {
            Some(ScenarioKind::AllowedCurrentCell) => saw_allowed_current_cell = true,
            Some(ScenarioKind::MismatchedCurrentCell) => saw_mismatched_current_cell = true,
            None => {}
        }
    }

    if !saw_allowed_current_cell {
        findings.insert(Finding::new(
            TENANT_PIN_NO_ADMITTED_SCENARIO,
            "<missing-admitted-scenario>",
            "scenario corpus must include an allowed-current-cell admit case",
        ));
    }
    if !saw_mismatched_current_cell {
        findings.insert(Finding::new(
            TENANT_PIN_NO_MISDIRECTED_SCENARIO,
            "<missing-misdirected-scenario>",
            "scenario corpus must include a mismatched-current-cell 421+Location case",
        ));
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(input))
}
