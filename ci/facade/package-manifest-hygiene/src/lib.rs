//! # cloud-ci-manifest-hygiene (§2.5#7 — MIG-PREREQ floor gate S2)
//!
//! Enforces the per-crate Cargo.toml hygiene every first-party `oya-*` crate must follow:
//! `version`/`rust-version` inherit the workspace, `publish = false`, a `license` is declared,
//! `[lints]` inherits the workspace, and — when a `[lib]` table is present — `doctest = false`.
//!
//! ## Pure flag→Finding policy (CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN Principle 1, shape c)
//! No pure crate owned the full §2.5#7 field-set, so this is a NET-NEW predicate — but the I/O
//! stays in the producer: `oya-cloud-ci-accounting-registry-app` parses each manifest into a row
//! of booleans, and this gate is a pure policy over those flags (zero file access). That keeps
//! the gate deterministic + trivially unit-testable; the producer's parser is the only thing
//! touching the filesystem.
//!
//! ## Contract
//! Input: `{"rows":[{"crate_name": "...", "has_version_workspace": bool, "has_publish_false":
//! bool, "has_license": bool, "has_rust_version_workspace": bool, "has_lints_workspace": bool,
//! "has_lib": bool, "has_lib_doctest_false": bool}]}`. `evaluate_keyed` emits one
//! `Finding{code,key}` per missing field (`key` = crate name); `evaluate` is the bare-code
//! projection. `baseline-block-on-new` freezes today's keys so only NEW debt blocks.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching the buck2 target + the firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-manifest-hygiene";

/// The blocking violation codes (stable slugs), one per §2.5#7 field.
pub const VIOLATION_CODES: [&str; 6] = [
    "manifest_missing_version_workspace",
    "manifest_missing_rust_version_workspace",
    "manifest_missing_publish_false",
    "manifest_missing_license",
    "manifest_missing_lints_workspace",
    "manifest_missing_lib_doctest_false",
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

fn malformed_input(key: &str) -> Finding {
    Finding::new("manifest_missing_version_workspace", key)
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

fn bool_flag(row: &Value, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}

fn flag(row: &Value, key: &str) -> bool {
    bool_flag(row, key).unwrap_or(false)
}

/// Pure evaluator: one `Finding` per missing §2.5#7 field per crate row.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(rows_value) = input.get("rows") else {
        findings.insert(malformed_input("<missing-rows>"));
        return findings;
    };
    let Some(rows) = rows_value.as_array() else {
        findings.insert(malformed_input("<non-array-rows>"));
        return findings;
    };
    if rows.is_empty() {
        findings.insert(malformed_input("<empty-rows>"));
        return findings;
    }
    for (index, row) in rows.iter().enumerate() {
        if !row.is_object() {
            findings.insert(malformed_input(&format!("<malformed-row-{index}>")));
            continue;
        }
        let Some(name) = row.get("crate_name").and_then(Value::as_str) else {
            findings.insert(malformed_input(&format!("<malformed-row-{index}>")));
            continue;
        };
        if !flag(row, "has_version_workspace") {
            findings.insert(Finding::new("manifest_missing_version_workspace", name));
        }
        if !flag(row, "has_rust_version_workspace") {
            findings.insert(Finding::new(
                "manifest_missing_rust_version_workspace",
                name,
            ));
        }
        if !flag(row, "has_publish_false") {
            findings.insert(Finding::new("manifest_missing_publish_false", name));
        }
        if !flag(row, "has_license") {
            findings.insert(Finding::new("manifest_missing_license", name));
        }
        if !flag(row, "has_lints_workspace") {
            findings.insert(Finding::new("manifest_missing_lints_workspace", name));
        }
        // doctest=false is only required when the crate declares a [lib] table.
        // `has_lib` is the shape discriminator; if it is absent or not a bool,
        // fail closed rather than silently treating the crate as bin-only.
        let Some(has_lib) = bool_flag(row, "has_lib") else {
            findings.insert(malformed_input(&format!("<malformed-row-{index}.has_lib>")));
            continue;
        };
        if has_lib && !flag(row, "has_lib_doctest_false") {
            findings.insert(Finding::new("manifest_missing_lib_doctest_false", name));
        }
    }
    findings
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input).into_iter().map(|f| f.code).collect();
    Report::from_codes(codes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn clean(name: &str, has_lib: bool) -> Value {
        json!({
            "crate_name": name,
            "has_version_workspace": true,
            "has_rust_version_workspace": true,
            "has_publish_false": true,
            "has_license": true,
            "has_lints_workspace": true,
            "has_lib": has_lib,
            "has_lib_doctest_false": has_lib,
        })
    }

    #[test]
    fn fully_hygienic_crate_is_green() {
        let input = json!({ "rows": [clean("oya-foo-domain", true), clean("oya-bar-app", false)] });
        assert_eq!(
            evaluate(&input).verdict,
            Verdict::Green,
            "{:?}",
            evaluate(&input).violations
        );
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn each_missing_field_fires_its_code() {
        let input = json!({ "rows": [{
            "crate_name": "oya-bad-domain",
            "has_version_workspace": false,
            "has_rust_version_workspace": false,
            "has_publish_false": false,
            "has_license": false,
            "has_lints_workspace": false,
            "has_lib": true,
            "has_lib_doctest_false": false,
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&input).into_iter().map(|f| f.code).collect();
        for code in VIOLATION_CODES {
            assert!(codes.contains(code), "expected {code} in {codes:?}");
        }
        assert!(
            evaluate_keyed(&input)
                .iter()
                .all(|f| f.key == "oya-bad-domain")
        );
    }

    #[test]
    fn doctest_not_required_without_lib() {
        // No [lib] table → manifest_missing_lib_doctest_false must NOT fire even if the flag is false.
        let input = json!({ "rows": [{
            "crate_name": "oya-binonly-app",
            "has_version_workspace": true,
            "has_rust_version_workspace": true,
            "has_publish_false": true,
            "has_license": true,
            "has_lints_workspace": true,
            "has_lib": false,
            "has_lib_doctest_false": false,
        }]});
        assert!(
            evaluate_keyed(&input).is_empty(),
            "bin-only crate must be green: {:?}",
            evaluate_keyed(&input)
        );
    }

    #[test]
    fn missing_lib_doctest_fires_when_lib_present() {
        let mut row = clean("oya-lib-domain", true);
        row["has_lib_doctest_false"] = json!(false);
        let input = json!({ "rows": [row] });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings.iter().next().unwrap().code,
            "manifest_missing_lib_doctest_false"
        );
    }

    #[test]
    fn missing_has_lib_discriminator_fails_closed() {
        let mut row = clean("oya-missing-has-lib-domain", false);
        row.as_object_mut().unwrap().remove("has_lib");
        let input = json!({ "rows": [row] });
        assert_eq!(
            evaluate_keyed(&input),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<malformed-row-0.has_lib>"
            )])
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn non_bool_has_lib_discriminator_fails_closed() {
        let mut row = clean("oya-non-bool-has-lib-domain", false);
        row["has_lib"] = json!("false");
        let input = json!({ "rows": [row] });
        assert_eq!(
            evaluate_keyed(&input),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<malformed-row-0.has_lib>"
            )])
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let mut row = clean("oya-x-domain", true);
        row["has_license"] = json!(false);
        let input = json!({ "rows": [row] });
        let projected: BTreeSet<String> =
            evaluate_keyed(&input).into_iter().map(|f| f.code).collect();
        assert_eq!(evaluate(&input).violations, projected);
    }

    #[test]
    fn empty_corpus_fails_closed() {
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Red);
    }

    #[test]
    fn missing_rows_fails_closed() {
        assert_eq!(
            evaluate_keyed(&json!({})),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<missing-rows>"
            )])
        );
    }

    #[test]
    fn non_array_rows_fails_closed() {
        assert_eq!(
            evaluate_keyed(&json!({ "rows": {} })),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<non-array-rows>"
            )])
        );
    }

    #[test]
    fn empty_rows_fails_closed() {
        assert_eq!(
            evaluate_keyed(&json!({ "rows": [] })),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<empty-rows>"
            )])
        );
    }

    #[test]
    fn malformed_row_fails_closed() {
        assert_eq!(
            evaluate_keyed(&json!({ "rows": [null] })),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<malformed-row-0>"
            )])
        );
    }

    #[test]
    fn missing_string_crate_name_fails_closed() {
        assert_eq!(
            evaluate_keyed(&json!({ "rows": [{ "crate_name": 7 }] })),
            BTreeSet::from([Finding::new(
                "manifest_missing_version_workspace",
                "<malformed-row-0>"
            )])
        );
    }
}
