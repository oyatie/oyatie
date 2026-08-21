//! # cloud-ci-license-policy
//!
//! Portable conformance gate for workspace package license declarations. The producer owns all
//! repository I/O: it resolves workspace members, reads each member `Cargo.toml`, and emits rows
//! shaped as `{package_name, manifest_path, license}`. This crate stays pure and reuses
//! `check_license_policy::LicensePolicy` so the legacy dev-cli predicate and the cloud-ci gate
//! cannot drift.
//!
//! `evaluate_keyed` returns one `Finding{code,key}` per invalid package row. Current accepted debt
//! is frozen by the firewall baseline with `baseline-block-on-new`, so license-policy is a
//! shrink-only gate: existing findings may shrink away, but new keys make `oya-ci-required` red.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use check_license_policy::{LicensePolicy, LicensePolicyError};
use serde_json::Value;

/// The gate id, matching oya-ci config and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-license-policy";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 5] = [
    "license_policy_missing_license",
    "license_policy_unknown_license",
    "license_policy_forbidden_license",
    "license_policy_review_required",
    "license_policy_no_workspace_members",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the stable `code` plus the offending package key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    fn new(code: &str, key: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_codes(violations: BTreeSet<String>) -> Self {
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

fn row_key(row: &Value) -> String {
    row.get("package_name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .or_else(|| row.get("manifest_path").and_then(Value::as_str))
        .unwrap_or("<unknown-package>")
        .to_owned()
}

fn finding_for(error: LicensePolicyError, key: &str) -> Finding {
    let code = match error {
        LicensePolicyError::MissingLicense => "license_policy_missing_license",
        LicensePolicyError::UnknownLicense => "license_policy_unknown_license",
        LicensePolicyError::ForbiddenLicense => "license_policy_forbidden_license",
        LicensePolicyError::ReviewRequired => "license_policy_review_required",
    };
    Finding::new(code, key)
}

/// Pure evaluator: takes `{"rows":[{"package_name":"...","license":"..."}, ...]}` and
/// emits one finding per invalid workspace package license. Running the legacy validator per row
/// converts its fail-fast whole-workspace contract into surface-all cloud-ci findings.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = match input.get("rows").and_then(Value::as_array) {
        Some(rows) if !rows.is_empty() => rows,
        _ => {
            findings.insert(Finding::new(
                "license_policy_no_workspace_members",
                "<empty-license-policy-corpus>",
            ));
            return findings;
        }
    };

    let policy = LicensePolicy::adr_0013_product_policy();
    for row in rows {
        let key = row_key(row);
        let license = row.get("license").and_then(Value::as_str).unwrap_or("");
        if let Err(error) = policy.validate_product_license(license) {
            findings.insert(finding_for(error, &key));
        }
    }
    findings
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input).into_iter().map(|f| f.code).collect();
    Report::from_codes(codes)
}
