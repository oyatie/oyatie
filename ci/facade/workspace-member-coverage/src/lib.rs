//! # cloud-ci-workspace-glob-coverage (ADR-0538)
//!
//! Enforces the globbed root workspace contract:
//! - every root `[workspace].members` entry must be a glob;
//! - every unexcluded member-glob match must contain `Cargo.toml`;
//! - every tracked first-party crate manifest dir must be covered by the resolved member set
//!   unless it is explicitly excluded.
//!
//! The accounting-registry producer emits the input rows after reading the root manifest and
//! resolving concrete members through `oya-workspace-members-kernel`. This gate is pure policy:
//! no filesystem access, no Cargo invocation, no duplicated glob expansion.
//!
//! ## Contract
//! Input: `{"rows":[{"member_entry": "...", "is_glob": bool},
//! {"member_match": "...", "has_manifest": bool},
//! {"crate_dir": "...", "covered": bool, "excluded": bool}]}`.
//! Missing or empty `rows` is red: the producer must prove it enumerated the workspace.
//!
//! `evaluate_keyed` emits one `Finding{code,key}` per offending member entry or crate dir;
//! `evaluate` is the bare-code projection. ADR-0083 Tier-3: production code carries no
//! unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching the buck2 target + firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-workspace-glob-coverage";

/// The blocking violation codes for ADR-0538's workspace glob contract.
pub const VIOLATION_CODES: [&str; 4] = [
    "workspace_member_explicit_path",
    "workspace_member_missing_manifest",
    "crate_dir_not_covered",
    "workspace_glob_row_malformed",
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

/// Pure evaluator for workspace member-entry and crate-dir coverage rows.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(rows) = input.get("rows").and_then(Value::as_array) else {
        findings.insert(Finding::new("crate_dir_not_covered", "<missing-rows>"));
        return findings;
    };
    if rows.is_empty() {
        findings.insert(Finding::new("crate_dir_not_covered", "<empty-rows>"));
        return findings;
    }
    for (index, row) in rows.iter().enumerate() {
        let malformed_key = format!("<row-{index}>");
        match (
            row.get("member_entry"),
            row.get("member_match"),
            row.get("crate_dir"),
        ) {
            (Some(member_entry), None, None) => {
                let Some(member_entry) = member_entry.as_str().filter(|value| !value.is_empty())
                else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                let Some(is_glob) = row.get("is_glob").and_then(Value::as_bool) else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                if !is_glob {
                    findings.insert(Finding::new("workspace_member_explicit_path", member_entry));
                }
            }
            (None, Some(member_match), None) => {
                let Some(member_match) = member_match.as_str().filter(|value| !value.is_empty())
                else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                let Some(has_manifest) = row.get("has_manifest").and_then(Value::as_bool) else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                if !has_manifest {
                    findings.insert(Finding::new(
                        "workspace_member_missing_manifest",
                        member_match,
                    ));
                }
            }
            (None, None, Some(crate_dir)) => {
                let Some(crate_dir) = crate_dir.as_str().filter(|value| !value.is_empty()) else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                let Some(covered) = row.get("covered").and_then(Value::as_bool) else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                let Some(excluded) = row.get("excluded").and_then(Value::as_bool) else {
                    findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
                    continue;
                };
                if !covered && !excluded {
                    findings.insert(Finding::new("crate_dir_not_covered", crate_dir));
                }
            }
            _ => {
                findings.insert(Finding::new("workspace_glob_row_malformed", &malformed_key));
            }
        }
    }
    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input).into_iter().map(|f| f.code).collect();
    Report::from_codes(codes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn glob_entries_and_covered_crates_are_green() {
        let input = json!({
            "rows": [
                {"member_entry": "libs/oya-*", "is_glob": true},
                {"crate_dir": "libs/oya-foo-kernel", "covered": true, "excluded": false}
            ]
        });
        assert_eq!(
            evaluate(&input).verdict,
            Verdict::Green,
            "{:?}",
            evaluate(&input).violations
        );
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn explicit_member_path_is_red() {
        let input = json!({
            "rows": [
                {"member_entry": "libs/oya-foo-kernel", "is_glob": false}
            ]
        });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "workspace_member_explicit_path");
        assert_eq!(finding.key, "libs/oya-foo-kernel");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn uncovered_unexcluded_crate_is_red() {
        let input = json!({
            "rows": [
                {"crate_dir": "tools/oya-orphan-app", "covered": false, "excluded": false}
            ]
        });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "crate_dir_not_covered");
        assert_eq!(finding.key, "tools/oya-orphan-app");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn every_member_glob_match_without_manifest_is_red() {
        let input = json!({
            "rows": [
                {"member_match": "comms/messenger/chaos", "has_manifest": false},
                {"member_match": "comms/messenger/resilience", "has_manifest": false}
            ]
        });
        let findings = evaluate_keyed(&input);
        assert_eq!(
            findings,
            BTreeSet::from([
                Finding::new("workspace_member_missing_manifest", "comms/messenger/chaos"),
                Finding::new(
                    "workspace_member_missing_manifest",
                    "comms/messenger/resilience",
                ),
            ])
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn uncovered_excluded_crate_is_green() {
        let input = json!({
            "rows": [
                {"crate_dir": "cloud/cloud-kernel/crates/oya-kernel", "covered": false, "excluded": true}
            ]
        });
        assert!(evaluate_keyed(&input).is_empty());
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = json!({
            "rows": [
                {"member_entry": "libs/oya-foo-kernel", "is_glob": false},
                {"member_match": "comms/messenger/chaos", "has_manifest": false},
                {"crate_dir": "tools/oya-orphan-app", "covered": false, "excluded": false},
                {"unexpected": "producer-shape-regression"}
            ]
        });
        let projected: BTreeSet<String> =
            evaluate_keyed(&input).into_iter().map(|f| f.code).collect();
        assert_eq!(evaluate(&input).violations, projected);
        for code in VIOLATION_CODES {
            assert!(projected.contains(code), "expected {code} in {projected:?}");
        }
    }

    #[test]
    fn empty_corpus_is_red() {
        let findings = evaluate_keyed(&json!({ "rows": [] }));
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "crate_dir_not_covered");
        assert_eq!(finding.key, "<empty-rows>");
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Red);
    }

    #[test]
    fn missing_rows_is_red() {
        let findings = evaluate_keyed(&json!({}));
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "crate_dir_not_covered");
        assert_eq!(finding.key, "<missing-rows>");
        assert_eq!(evaluate(&json!({})).verdict, Verdict::Red);
    }

    #[test]
    fn malformed_row_without_known_shape_is_red() {
        let input = json!({
            "rows": [
                {"unexpected": "producer-shape-regression"}
            ]
        });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "workspace_glob_row_malformed");
        assert_eq!(finding.key, "<row-0>");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn malformed_member_row_missing_is_glob_is_red() {
        let input = json!({
            "rows": [
                {"member_entry": "libs/oya-foo-kernel"}
            ]
        });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "workspace_glob_row_malformed");
        assert_eq!(finding.key, "<row-0>");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn malformed_crate_row_missing_coverage_booleans_is_red() {
        for input in [
            json!({"rows": [{"crate_dir": "tools/oya-orphan-app", "excluded": false}]}),
            json!({"rows": [{"crate_dir": "tools/oya-orphan-app", "covered": false}]}),
        ] {
            let findings = evaluate_keyed(&input);
            assert_eq!(findings.len(), 1);
            let finding = findings.iter().next().unwrap();
            assert_eq!(finding.code, "workspace_glob_row_malformed");
            assert_eq!(finding.key, "<row-0>");
            assert_eq!(evaluate(&input).verdict, Verdict::Red);
        }
    }
}
