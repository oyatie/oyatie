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
//! always `member_path`; the firewall freezes today's accepted debt by key and blocks only
//! new keys for `member_test_code_without_rust_test_target`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_ci_gate_contract::{
    Finding as ContractFinding, Gate as ContractGate, GateCode, GateManifest, NewFile, Remediation,
    RemediationTier,
};
use serde_json::Value;

/// The gate id, matching the buck2 target + firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-target-parity";

pub const MEMBER_MISSING_BUCK_CODE: &str = "member_missing_buck";
pub const MEMBER_TEST_CODE_WITHOUT_RUST_TEST_TARGET_CODE: &str =
    "member_test_code_without_rust_test_target";

/// The blocking violation codes for ADR-0540's target-parity contract.
pub const VIOLATION_CODES: [&str; 2] = [
    MEMBER_MISSING_BUCK_CODE,
    MEMBER_TEST_CODE_WITHOUT_RUST_TEST_TARGET_CODE,
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

#[derive(Debug, Clone)]
pub struct TargetParityGate {
    manifest: GateManifest,
}

impl TargetParityGate {
    pub fn new() -> Result<Self, oya_ci_gate_contract::ContractError> {
        Ok(Self {
            manifest: GateManifest::new(
                GATE_ID,
                vec![
                    GateCode::new(MEMBER_MISSING_BUCK_CODE, RemediationTier::AutoGenerate),
                    GateCode::new(
                        MEMBER_TEST_CODE_WITHOUT_RUST_TEST_TARGET_CODE,
                        RemediationTier::Block {
                            rationale: "adding or changing a rust_test target requires humans to review the build graph target name, deps, and test coverage boundary".to_owned(),
                        },
                    ),
                ],
            )?,
        })
    }
}

impl ContractGate for TargetParityGate {
    fn manifest(&self) -> &GateManifest {
        &self.manifest
    }

    fn evaluate_keyed(&self, face: &Value) -> BTreeSet<ContractFinding> {
        evaluate_keyed(face)
            .into_iter()
            .map(|finding| ContractFinding::new(finding.code, finding.key))
            .collect()
    }

    fn remediate(&self, finding: &ContractFinding, face: &Value) -> Remediation {
        remediate_code(&finding.code, &finding.key, face)
    }
}

fn bool_field(row: &Value, key: &str) -> bool {
    row.get(key).and_then(Value::as_bool).unwrap_or(false)
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
}

/// Pure ADR-0528 remediation sibling for one ADR-0540 code.
///
/// The response is a described new-file proposal only. It never writes, and it returns
/// [`Remediation::None`] once the same face reports the member has a BUCK file.
pub fn remediate(finding: &Finding, face: &Value) -> Remediation {
    remediate_code(&finding.code, &finding.key, face)
}

fn remediate_code(code: &str, member_path: &str, face: &Value) -> Remediation {
    if code != MEMBER_MISSING_BUCK_CODE || !face_reports_missing_buck(member_path, face) {
        return Remediation::None;
    }

    Remediation::AutoGenerate(NewFile::new(
        format!("{member_path}/BUCK"),
        buck_library_file_body(member_path),
    ))
}

fn face_reports_missing_buck(member_path: &str, face: &Value) -> bool {
    face.get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| {
            row.get("member_path").and_then(Value::as_str) == Some(member_path)
                && !bool_field(row, "has_buck")
        })
}

fn buck_library_file_body(member_path: &str) -> String {
    let target_name = member_path
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(member_path);
    let crate_name = target_name.replace('-', "_");

    format!(
        "rust_library(\n    name = \"{target_name}\",\n    srcs = glob([\"src/**/*.rs\", \"migrations/**/*.sql\", \"**/*.cedar\", \"**/*.sql\", \"**/*.json\", \"**/*.toml\", \"**/*.yaml\", \"**/*.yml\", \"**/*.proto\", \"**/*.graphql\", \"**/*.html\", \"**/*.css\", \"**/*.txt\"]),\n    crate = \"{crate_name}\",\n    crate_root = \"src/lib.rs\",\n    visibility = [\"PUBLIC\"],\n)\n"
    )
}

/// Pure evaluator for producer-emitted target-parity rows.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for row in &rows {
        let Some(member_path) = row.get("member_path").and_then(Value::as_str) else {
            continue;
        };
        if !bool_field(row, "has_buck") {
            findings.insert(Finding::new(MEMBER_MISSING_BUCK_CODE, member_path));
        }
        if bool_field(row, "has_test_code") && !bool_field(row, "has_rust_test_target") {
            findings.insert(Finding::new(
                MEMBER_TEST_CODE_WITHOUT_RUST_TEST_TARGET_CODE,
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
    use oya_ci_gate_contract::{Gate as ContractGate, Remediation, RemediationTier};
    use serde_json::json;

    #[test]
    fn green_rows_cover_tested_and_untested_members() {
        let input = json!({
            "rows": [
                {
                    "member_path": "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app",
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
    fn gate_manifest_declares_missing_buck_as_auto_generate() {
        let gate = TargetParityGate::new().expect("target parity manifest is valid");
        let manifest = ContractGate::manifest(&gate);

        assert_eq!(manifest.gate_id, GATE_ID);
        let missing_buck = manifest
            .codes
            .iter()
            .find(|code| code.code == "member_missing_buck")
            .expect("manifest declares member_missing_buck");
        assert_eq!(missing_buck.remediation_tier, RemediationTier::AutoGenerate);
    }

    #[test]
    fn missing_buck_remediate_auto_generates_buck_file_and_is_idempotent_after_fix() {
        let finding = Finding::new("member_missing_buck", "libs/oya-new-domain");
        let broken = json!({
            "rows": [{
                "member_path": "libs/oya-new-domain",
                "has_buck": false,
                "has_rust_test_target": false,
                "has_test_code": false
            }]
        });
        let fixed = json!({
            "rows": [{
                "member_path": "libs/oya-new-domain",
                "has_buck": true,
                "has_rust_test_target": false,
                "has_test_code": false
            }]
        });

        let remediation = remediate(&finding, &broken);
        let Remediation::AutoGenerate(new_file) = remediation else {
            panic!("expected AutoGenerate remediation for missing BUCK");
        };
        assert_eq!(new_file.path, "libs/oya-new-domain/BUCK");
        assert!(new_file.body.contains("rust_library("));
        assert!(new_file.body.contains("crate = \"oya_new_domain\""));

        assert_eq!(remediate(&finding, &fixed), Remediation::None);
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
    fn empty_corpus_is_green() {
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Green);
    }
}
