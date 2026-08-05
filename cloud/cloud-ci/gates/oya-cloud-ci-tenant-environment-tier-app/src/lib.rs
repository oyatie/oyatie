//! # cloud-ci-tenant-environment-tier
//!
//! Pure cloud-ci readiness gate packet for tenant environment-tier isolation evidence. The
//! producer owns tenancy/Cedar/workflow/intelligence/outbound hook inventory and feeds rows shaped
//! as env-tier fixture facts. This crate makes no live tenant environment, API-key issuance,
//! outbound-delivery, or production-readiness claim.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-tenant-environment-tier";

pub const VIOLATION_CODES: [&str; 9] = [
    "env_tier_fixture_missing",
    "api_key_prefix_unmapped",
    "test_key_routes_to_prod",
    "outbound_mode_unenforced",
    "prod_destructive_ack_missing",
    "cedar_key_grant_missing",
    "audit_chain_env_tier_missing",
    "workflow_default_tier_not_test",
    "tier_model_budget_hook_missing",
];

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

fn string_field<'a>(row: &'a Value, field: &str) -> Option<&'a str> {
    row.get(field).and_then(Value::as_str).map(str::trim)
}

fn bool_field(row: &Value, field: &str) -> bool {
    row.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn row_key(row: &Value, index: usize) -> String {
    string_field(row, "fixture_id")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("<tenant-env-tier-row-{index}>"))
}

fn expected_prefix(tier: &str) -> Option<&'static str> {
    match tier {
        "test" => Some("_test_"),
        "staging" => Some("_stage_"),
        "prod" => Some("_live_"),
        _ => None,
    }
}

fn prefix_matches_tier(api_key_prefix: &str, tier: &str) -> bool {
    expected_prefix(tier).is_some_and(|expected| api_key_prefix.contains(expected))
}

fn is_prod_destination(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("prod") || lower.contains("live")
}

fn evaluate_row(index: usize, row: &Value, findings: &mut BTreeSet<Finding>) {
    let key = row_key(row, index);
    let Some(tier) = string_field(row, "tier").filter(|value| !value.is_empty()) else {
        findings.insert(Finding::new(
            "env_tier_fixture_missing",
            key,
            "tenant environment-tier fixture missing non-empty tier",
        ));
        return;
    };
    let Some(api_key_prefix) =
        string_field(row, "api_key_prefix").filter(|value| !value.is_empty())
    else {
        findings.insert(Finding::new(
            "api_key_prefix_unmapped",
            key,
            "tenant environment-tier fixture missing API-key prefix mapping",
        ));
        return;
    };
    if !prefix_matches_tier(api_key_prefix, tier) {
        findings.insert(Finding::new(
            "api_key_prefix_unmapped",
            key.clone(),
            "API-key prefix does not map to the declared environment tier",
        ));
    }

    let observed_pool = string_field(row, "observed_workload_pool").unwrap_or_default();
    let observed_schema = string_field(row, "observed_schema_or_database").unwrap_or_default();
    if tier == "test"
        && (is_prod_destination(observed_pool) || is_prod_destination(observed_schema))
    {
        findings.insert(Finding::new(
            "test_key_routes_to_prod",
            key.clone(),
            "test-tier API key must not route to prod workload pool or schema/database",
        ));
    }

    let expected_outbound = string_field(row, "outbound_mode_expected").unwrap_or_default();
    let observed_outbound = string_field(row, "outbound_mode_observed").unwrap_or_default();
    if !expected_outbound.is_empty() && expected_outbound != observed_outbound {
        findings.insert(Finding::new(
            "outbound_mode_unenforced",
            key.clone(),
            "observed outbound side-effect mode does not match tier policy",
        ));
    }

    if bool_field(row, "prod_destructive_ack_required")
        && !bool_field(row, "prod_destructive_ack_observed")
    {
        findings.insert(Finding::new(
            "prod_destructive_ack_missing",
            key.clone(),
            "prod destructive operation requires explicit prod_destructive_ack evidence",
        ));
    }

    if api_key_prefix.contains("_live_")
        && string_field(row, "cedar_key_issuance_role") != Some("admin")
        && bool_field(row, "cedar_key_issuance_allowed")
    {
        findings.insert(Finding::new(
            "cedar_key_grant_missing",
            key.clone(),
            "live-key issuance may not be granted to non-admin Cedar roles",
        ));
    }

    if !bool_field(row, "audit_chain_tag_present") {
        findings.insert(Finding::new(
            "audit_chain_env_tier_missing",
            key.clone(),
            "tiered requests must emit an env_tier audit-chain tag",
        ));
    }

    if let Some(workflow_default) = string_field(row, "workflow_default_new_flow_tier")
        && !workflow_default.is_empty()
        && workflow_default != "test"
    {
        findings.insert(Finding::new(
            "workflow_default_tier_not_test",
            key.clone(),
            "Workflow Studio must default new flows to the test environment tier",
        ));
    }

    if row.get("model_or_budget_tier_hook_present").is_some()
        && !bool_field(row, "model_or_budget_tier_hook_present")
    {
        findings.insert(Finding::new(
            "tier_model_budget_hook_missing",
            key,
            "intelligence/model or cost-budget hook must carry tier separation evidence",
        ));
    }
}

pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.is_empty() {
        findings.insert(Finding::new(
            "env_tier_fixture_missing",
            "<tenant-environment-tier-fixtures>",
            "producer emitted no tenant environment-tier fixture rows",
        ));
        return findings;
    }
    for (index, row) in rows.iter().enumerate() {
        evaluate_row(index, row, &mut findings);
    }
    findings
}

pub fn evaluate(input: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(input))
}
