//! Policy engine for the ADR-0632 product-protocol contract.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-product-protocol-policy";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub path: String,
    pub detail: String,
}

impl Finding {
    fn new(code: impl Into<String>, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            detail: detail.into(),
        }
    }
}

fn string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn unordered_string_array(value: &Value) -> Option<(usize, BTreeSet<&str>)> {
    let values = value.as_array()?;
    let strings = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<BTreeSet<_>>>()?;
    Some((values.len(), strings))
}

fn rule_matches(mode: &str, observed: &Value, expected: &Value) -> Result<bool, &'static str> {
    match mode {
        "equals" => Ok(observed == expected),
        "unordered_equals" => {
            let Some((observed_len, observed)) = unordered_string_array(observed) else {
                return Err("unordered_equals observed value must be an array of unique strings");
            };
            let Some((expected_len, expected)) = unordered_string_array(expected) else {
                return Err("unordered_equals expected value must be an array of unique strings");
            };
            Ok(observed_len == observed.len()
                && expected_len == expected.len()
                && observed == expected)
        }
        _ => Err("rule mode must be equals or unordered_equals"),
    }
}

pub fn evaluate_keyed(
    policy: &Value,
    artifacts: &BTreeMap<String, Value>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    if string(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "PP-POLICY-GATE-ID",
            "gate_id",
            format!("policy gate_id must equal {GATE_ID}"),
        ));
    }

    let Some(declared_artifacts) = policy.get("artifacts").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "PP-POLICY-ARTIFACTS",
            "artifacts",
            "policy artifacts must be a non-empty object",
        ));
        return findings;
    };
    if declared_artifacts.is_empty() {
        findings.insert(Finding::new(
            "PP-POLICY-ARTIFACTS",
            "artifacts",
            "policy artifacts must be a non-empty object",
        ));
    }

    let Some(rules) = policy.get("rules").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "PP-POLICY-RULES",
            "rules",
            "policy rules must be a non-empty array",
        ));
        return findings;
    };
    if rules.is_empty() {
        findings.insert(Finding::new(
            "PP-POLICY-RULES",
            "rules",
            "policy rules must be a non-empty array",
        ));
        return findings;
    }

    let mut codes = BTreeSet::new();
    for (index, rule) in rules.iter().enumerate() {
        let rule_path = format!("rules[{index}]");
        let Some(code) = string(rule, "code").filter(|value| !value.trim().is_empty()) else {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED-RULE",
                &rule_path,
                "rule code must be a non-empty string",
            ));
            continue;
        };
        if !codes.insert(code) {
            findings.insert(Finding::new(
                "PP-POLICY-DUPLICATE-CODE",
                &rule_path,
                format!("duplicate rule code {code}"),
            ));
            continue;
        }
        let Some(artifact_name) = string(rule, "artifact") else {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED-RULE",
                &rule_path,
                "rule artifact must be a string",
            ));
            continue;
        };
        if !declared_artifacts.contains_key(artifact_name) {
            findings.insert(Finding::new(
                "PP-POLICY-UNKNOWN-ARTIFACT",
                &rule_path,
                format!("rule references undeclared artifact {artifact_name}"),
            ));
            continue;
        }
        let Some(pointer) = string(rule, "pointer").filter(|value| value.starts_with('/')) else {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED-RULE",
                &rule_path,
                "rule pointer must be a JSON Pointer beginning with /",
            ));
            continue;
        };
        let Some(mode) = string(rule, "mode") else {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED-RULE",
                &rule_path,
                "rule mode must be a string",
            ));
            continue;
        };
        let Some(expected) = rule.get("expected") else {
            findings.insert(Finding::new(
                "PP-POLICY-MALFORMED-RULE",
                &rule_path,
                "rule expected value is required",
            ));
            continue;
        };
        let Some(artifact) = artifacts.get(artifact_name) else {
            findings.insert(Finding::new(
                code,
                format!("{artifact_name}:{pointer}"),
                "declared artifact was not collected",
            ));
            continue;
        };
        let Some(observed) = artifact.pointer(pointer) else {
            findings.insert(Finding::new(
                code,
                format!("{artifact_name}:{pointer}"),
                "required contract field is missing",
            ));
            continue;
        };
        match rule_matches(mode, observed, expected) {
            Ok(true) => {}
            Ok(false) => {
                findings.insert(Finding::new(
                    code,
                    format!("{artifact_name}:{pointer}"),
                    format!("expected {expected}, observed {observed}"),
                ));
            }
            Err(detail) => {
                findings.insert(Finding::new(
                    "PP-POLICY-MALFORMED-RULE",
                    &rule_path,
                    detail,
                ));
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "artifacts": {"contract": "contract.json"},
            "rules": [
                {"code": "PP-ONE", "artifact": "contract", "pointer": "/one", "mode": "equals", "expected": true},
                {"code": "PP-SET", "artifact": "contract", "pointer": "/set", "mode": "unordered_equals", "expected": ["a", "b"]}
            ]
        })
    }

    #[test]
    fn matching_contract_is_green() {
        let artifacts = BTreeMap::from([(
            "contract".to_owned(),
            json!({"one": true, "set": ["b", "a"]}),
        )]);
        assert!(evaluate_keyed(&policy(), &artifacts).is_empty());
    }

    #[test]
    fn mismatched_and_missing_values_are_keyed_red() {
        let artifacts = BTreeMap::from([("contract".to_owned(), json!({"one": false}))]);
        let findings = evaluate_keyed(&policy(), &artifacts);
        assert!(findings.iter().any(|finding| finding.code == "PP-ONE"));
        assert!(findings.iter().any(|finding| finding.code == "PP-SET"));
    }

    #[test]
    fn duplicate_set_members_do_not_launder_a_rule() {
        let artifacts = BTreeMap::from([(
            "contract".to_owned(),
            json!({"one": true, "set": ["a", "a", "b"]}),
        )]);
        assert!(
            evaluate_keyed(&policy(), &artifacts)
                .iter()
                .any(|finding| finding.code == "PP-SET")
        );
    }

    #[test]
    fn empty_policy_fails_closed() {
        assert!(!evaluate_keyed(&json!({}), &BTreeMap::new()).is_empty());
    }
}
