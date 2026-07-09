//! # cloud-ci-target-parity (ADR-0540)
//!
//! Enforces the cargo/Buck target-parity invariant for Rust workspace members: every member
//! must carry a tracked BUCK file, and every member with Rust test code must declare a
//! `rust_test` target so CI compiles that test code under Buck.
//!
//! ## Contract
//! Input: `{"rows":[{"member_path": "...", "has_buck": bool, "has_rust_test_target": bool,
//! "has_test_code": bool}]}`.
//!
//! `evaluate_keyed` emits one `Finding{code,key,remediation}` per offending member. `key` is
//! normally `member_path`; producer-contract defects use stable synthetic keys so malformed
//! evidence cannot pass quietly. The firewall freezes today's accepted debt by key and blocks
//! only new keys for `member_test_code_without_rust_test_target`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching the buck2 target + firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-target-parity";

/// The blocking violation codes for ADR-0540's target-parity contract.
pub const VIOLATION_CODES: [&str; 2] = [
    "member_missing_buck",
    "member_test_code_without_rust_test_target",
];

const TARGET_PARITY_ROWS_KEY: &str = "<cloud-ci-target-parity#rows>";
const TARGET_PARITY_ROW_KEY: &str = "<cloud-ci-target-parity#row>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub remediation: String,
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

fn bool_field(row: &Value, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}

fn row_has_required_bool_fields(row: &Value) -> bool {
    ["has_buck", "has_rust_test_target", "has_test_code"]
        .into_iter()
        .all(|field| row.get(field).and_then(Value::as_bool).is_some())
}

fn remediation(member_path: &str) -> String {
    format!(
        "declare a rust_test target in {member_path}/BUCK (see any gates/* BUCK for the stanza shape) and ensure `buck2 test <target>` passes; see ADR-0540"
    )
}

impl Finding {
    fn new(code: &str, member_path: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: member_path.to_owned(),
            remediation: remediation(member_path),
        }
    }

    /// Reuse the born-blocking `member_missing_buck` code for an absent corpus so the firewall
    /// treats "no target-parity evidence" as immediately red without widening the public code set.
    fn missing_target_parity_rows() -> Self {
        Self {
            code: "member_missing_buck".to_owned(),
            key: TARGET_PARITY_ROWS_KEY.to_owned(),
            remediation: "producer must emit a non-empty target-parity rows array so BUCK/workspace parity cannot pass on an absent corpus; see ADR-0540".to_owned(),
        }
    }

    /// Reuse the born-blocking `member_missing_buck` code for malformed row entries so a
    /// partially valid corpus cannot hide producer defects behind skipped members.
    fn malformed_target_parity_row() -> Self {
        Self {
            code: "member_missing_buck".to_owned(),
            key: TARGET_PARITY_ROW_KEY.to_owned(),
            remediation: "producer must emit a non-empty string member_path for every target-parity row so BUCK/workspace parity cannot silently skip malformed members; see ADR-0540".to_owned(),
        }
    }
}

/// Pure evaluator for producer-emitted target-parity rows.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = match input.get("rows").and_then(Value::as_array) {
        Some(rows) if !rows.is_empty() => rows,
        _ => {
            findings.insert(Finding::missing_target_parity_rows());
            return findings;
        }
    };
    for row in rows {
        let Some(member_path) = row.get("member_path").and_then(Value::as_str) else {
            findings.insert(Finding::malformed_target_parity_row());
            continue;
        };
        if member_path.trim().is_empty()
            || member_path.trim().len() != member_path.len()
            || !row_has_required_bool_fields(row)
        {
            findings.insert(Finding::malformed_target_parity_row());
            continue;
        }
        if !bool_field(row, "has_buck").unwrap_or(false) {
            findings.insert(Finding::new("member_missing_buck", member_path));
        }
        if bool_field(row, "has_test_code").unwrap_or(false)
            && !bool_field(row, "has_rust_test_target").unwrap_or(false)
        {
            findings.insert(Finding::new(
                "member_test_code_without_rust_test_target",
                member_path,
            ));
        }
    }
    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_codes(codes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn malformed_member_path_rows_fail_closed_instead_of_skipping() {
        for row in [
            json!({
                "has_buck": true,
                "has_rust_test_target": true,
                "has_test_code": false
            }),
            json!({
                "member_path": 42,
                "has_buck": true,
                "has_rust_test_target": true,
                "has_test_code": false
            }),
            json!({
                "member_path": "   ",
                "has_buck": true,
                "has_rust_test_target": true,
                "has_test_code": false
            }),
            json!({
                "member_path": " libs/oya-ci-config ",
                "has_buck": true,
                "has_rust_test_target": true,
                "has_test_code": false
            }),
            json!({
                "member_path": "libs/oya-ci-config",
                "has_buck": "true",
                "has_rust_test_target": true,
                "has_test_code": false
            }),
            json!({
                "member_path": "libs/oya-ci-config",
                "has_buck": true,
                "has_rust_test_target": true
            }),
        ] {
            let input = json!({"rows": [row]});
            let findings = evaluate_keyed(&input);
            assert_eq!(findings.len(), 1);
            let finding = findings.iter().next().unwrap();
            assert_eq!(finding.code, "member_missing_buck");
            assert_eq!(finding.key, TARGET_PARITY_ROW_KEY);
            assert!(
                finding.remediation.contains("member_path")
                    && finding.remediation.contains("ADR-0540"),
                "malformed row remediation should name the broken face contract: {finding:?}"
            );
            assert_eq!(evaluate(&input).verdict, Verdict::Red);
        }
    }

    #[test]
    fn green_rows_cover_tested_and_untested_members() {
        let input = json!({
            "rows": [
                {
                    "member_path": "ci/facade/baseline-ratchet",
                    "has_buck": true,
                    "has_rust_test_target": true,
                    "has_test_code": true
                },
                {
                    "member_path": "libs/oya-ci-config",
                    "has_buck": true,
                    "has_rust_test_target": false,
                    "has_test_code": false
                }
            ]
        });

        assert_eq!(evaluate(&input).verdict, Verdict::Green);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn missing_buck_is_red() {
        let input = json!({
            "rows": [{
                "member_path": "libs/oya-new-domain",
                "has_buck": false,
                "has_rust_test_target": false,
                "has_test_code": false
            }]
        });

        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "member_missing_buck");
        assert_eq!(finding.key, "libs/oya-new-domain");
        assert!(
            finding.remediation.contains("declare a rust_test target")
                && finding.remediation.contains("ADR-0540"),
            "remediation should point contributors at the fix and ADR-0540: {finding:?}"
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn member_with_test_code_requires_rust_test_target() {
        let input = json!({
            "rows": [{
                "member_path": "oya/example/crates/oya-example-domain",
                "has_buck": true,
                "has_rust_test_target": false,
                "has_test_code": true
            }]
        });

        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "member_test_code_without_rust_test_target");
        assert_eq!(finding.key, "oya/example/crates/oya-example-domain");
        assert!(
            finding.remediation.contains(
                "declare a rust_test target in oya/example/crates/oya-example-domain/BUCK"
            ),
            "remediation should name the member BUCK path: {finding:?}"
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = json!({
            "rows": [
                {
                    "member_path": "libs/oya-missing-buck-domain",
                    "has_buck": false,
                    "has_rust_test_target": false,
                    "has_test_code": false
                },
                {
                    "member_path": "libs/oya-unwired-tests-domain",
                    "has_buck": true,
                    "has_rust_test_target": false,
                    "has_test_code": true
                }
            ]
        });

        let projected: BTreeSet<String> = evaluate_keyed(&input)
            .into_iter()
            .map(|finding| finding.code)
            .collect();
        assert_eq!(evaluate(&input).violations, projected);
        for code in VIOLATION_CODES {
            assert!(projected.contains(code), "expected {code} in {projected:?}");
        }
    }

    #[test]
    fn missing_rows_input_fails_closed() {
        let input = json!({});

        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "member_missing_buck");
        assert_eq!(finding.key, TARGET_PARITY_ROWS_KEY);
        assert!(
            finding.remediation.contains("target-parity rows")
                && finding.remediation.contains("ADR-0540"),
            "missing producer rows remediation should name the broken face contract: {finding:?}"
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn malformed_or_empty_rows_input_fails_closed() {
        for input in [json!({ "rows": "not-an-array" }), json!({ "rows": [] })] {
            let findings = evaluate_keyed(&input);
            assert_eq!(findings.len(), 1);
            let finding = findings.iter().next().unwrap();
            assert_eq!(finding.code, "member_missing_buck");
            assert_eq!(finding.key, TARGET_PARITY_ROWS_KEY);
            assert_eq!(evaluate(&input).verdict, Verdict::Red);
        }
    }
}
