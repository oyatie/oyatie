//! # cloud-ci-multi-region-disposition
//!
//! Pure cloud-ci readiness gate packet for the accepted multi-region disposition contract. The
//! producer owns repository/manifest/doc inventory and feeds this evaluator rows shaped as
//! service disposition facts. This crate does not provision regions, mutate manifests, or claim
//! production failover readiness.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-multi-region-disposition";

pub const VIOLATION_CODES: [&str; 7] = [
    "manifest_missing_multi_region_disposition",
    "manifest_invalid_multi_region_disposition",
    "multi_region_doc_missing",
    "multi_region_doc_missing_required_section",
    "disposition_doc_mismatch",
    "active_passive_missing_rpo_rto",
    "deployment_shape_mismatch",
];

const VALID_DISPOSITIONS: [&str; 3] = ["active_active", "active_passive", "single_region"];

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
    string_field(row, "service_id")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            string_field(row, "manifest_path")
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("<multi-region-row-{index}>"))
}

fn valid_disposition(value: &str) -> bool {
    VALID_DISPOSITIONS.contains(&value)
}

fn required_section(row: &Value, section: &str) -> bool {
    row.get("doc_required_sections")
        .and_then(|sections| sections.get(section))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn evaluate_row(index: usize, row: &Value, findings: &mut BTreeSet<Finding>) {
    let key = row_key(row, index);
    let manifest_disposition = string_field(row, "manifest_disposition").filter(|v| !v.is_empty());
    let manifest_valid = row
        .get("manifest_disposition_valid")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| manifest_disposition.is_some_and(valid_disposition));

    if !bool_field(row, "manifest_present") || manifest_disposition.is_none() {
        findings.insert(Finding::new(
            "manifest_missing_multi_region_disposition",
            key.clone(),
            "service manifest must declare multi_region_disposition",
        ));
    } else if !manifest_valid {
        findings.insert(Finding::new(
            "manifest_invalid_multi_region_disposition",
            key.clone(),
            "multi_region_disposition must be active_active, active_passive, or single_region",
        ));
    }

    if !bool_field(row, "doc_present") {
        findings.insert(Finding::new(
            "multi_region_doc_missing",
            key.clone(),
            "service must carry a multi-region.md companion for disposition rationale",
        ));
        return;
    }

    for section in ["disposition_statement", "rationale"] {
        if !required_section(row, section) {
            findings.insert(Finding::new(
                "multi_region_doc_missing_required_section",
                format!("{key}:{section}"),
                format!("multi-region.md missing required section `{section}`"),
            ));
        }
    }

    if manifest_disposition == Some("active_passive")
        && !required_section(row, "rpo_rto_numbers_if_active_passive")
    {
        findings.insert(Finding::new(
            "active_passive_missing_rpo_rto",
            key.clone(),
            "active_passive disposition must document concrete RPO/RTO numbers",
        ));
    }

    let doc_disposition = string_field(row, "doc_disposition").filter(|v| !v.is_empty());
    if let (Some(manifest), Some(doc)) = (manifest_disposition, doc_disposition)
        && manifest != doc
    {
        findings.insert(Finding::new(
            "disposition_doc_mismatch",
            key.clone(),
            "manifest and multi-region.md dispositions disagree",
        ));
    }

    let deployment_shape =
        string_field(row, "deployment_shape_disposition").filter(|v| !v.is_empty());
    if let (Some(manifest), Some(shape)) = (manifest_disposition, deployment_shape)
        && manifest != shape
    {
        findings.insert(Finding::new(
            "deployment_shape_mismatch",
            key,
            "declared disposition disagrees with producer-supplied deployment shape evidence",
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
            "manifest_missing_multi_region_disposition",
            "<multi-region-disposition-corpus>",
            "producer emitted no service manifest rows for multi-region disposition evaluation",
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
