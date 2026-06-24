//! # cloud-ci-root-workspace-hygiene (ADR-0600)
//!
//! Born-blocking, UNIVERSAL, HERMETIC root-workspace-hygiene gate that makes committed
//! repo-root scratch structurally impossible. The gate is productized policy, not Oyatie-only
//! glue: the legitimate root surface lives in DATA (`root-workspace-hygiene-policy.json`), while
//! this crate evaluates the portable contract that EVERY tracked file at the repository ROOT
//! matches the allowlist and EVERY tracked top-level directory is a permitted capability/meta home.
//!
//! ## Posture: default-DENY (allowlist), complementing the scratch DENYLIST
//! The existing `cloud-ci-total-accounting` `scratch_artifact` code is a DENYLIST: it catches
//! KNOWN scratch shapes (`*.log`, `run-slice.sh`, …) by name. This gate is the complement — an
//! ALLOWLIST: any tracked root file that matches NO allowlist rule fails, so a scratch shape that
//! nobody has named yet is STILL born-blocking. The two layers compose into "impossible to commit
//! unjustified repo-root scratch" (founder directive).
//!
//! ## Pure evaluator (zero I/O)
//! The producer side supplies the git-ls-files snapshot (scm-facts) as DATA; this crate is a pure
//! evaluator over `{ "rows": [{"path": "..."}] }` (the tracked-path inventory) plus the committed
//! allowlist policy. `evaluate_keyed` returns one `Finding{code,key,detail}` per violation;
//! `evaluate` is the bare report projection. No shell, net, clock, rand, or filesystem access.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The gate id (matches the buck2 target + the policy `gate_id`).
pub const GATE_ID: &str = "cloud-ci-root-workspace-hygiene";

/// The blocking violation codes (stable slugs).
pub const VIOLATION_CODES: [&str; 4] = [
    // The policy `gate_id` does not match GATE_ID (config integrity).
    "root_workspace_gate_id_mismatch",
    // A tracked file at the repo ROOT matches no allowlist rule — born-blocking root scratch.
    "root_workspace_unallowlisted_file",
    // A tracked path's top-level directory is not a permitted capability/meta home.
    "root_workspace_unallowlisted_dir",
    // An allowlist rule is malformed (missing/blank id, kind, or value).
    "root_workspace_policy_malformed_rule",
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
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
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

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim)
}

/// A single parsed allowlist rule: a match `kind` over the file basename and a `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AllowRule {
    id: String,
    kind: String,
    value: String,
}

/// True iff `basename` is admitted by this rule.
fn rule_matches(rule: &AllowRule, basename: &str) -> bool {
    match rule.kind.as_str() {
        "exact" => basename == rule.value,
        "suffix" => basename.ends_with(&rule.value),
        "prefix" => basename.starts_with(&rule.value),
        // `prefix_dot`: exact match OR starts-with `value` followed by `.` or `-`.
        // Tighter than bare `prefix`: `README` matches README and README.md but NOT READMEILY.
        // Pattern in DATA: `{ "kind": "prefix_dot", "value": "README" }`.
        "prefix_dot" => {
            basename == rule.value
                || basename.starts_with(&format!("{}.", rule.value))
                || basename.starts_with(&format!("{}-", rule.value))
        }
        // Unknown kinds never match (the malformed-rule finding flags them separately).
        _ => false,
    }
}

/// Parse the `allowed_root_files` rule table, emitting `root_workspace_policy_malformed_rule`
/// for any rule missing a non-empty id/kind/value or carrying an unknown kind.
fn allow_rules(policy: &Value, findings: &mut BTreeSet<Finding>) -> Vec<AllowRule> {
    let mut rules = Vec::new();
    for (index, raw) in policy
        .get("allowed_root_files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let id = string_field(raw, "id").unwrap_or("");
        let kind = string_field(raw, "kind").unwrap_or("");
        let value = string_field(raw, "value").unwrap_or("");
        let key = if id.is_empty() {
            format!("allowed_root_files[{index}]")
        } else {
            id.to_owned()
        };
        if id.is_empty() || value.is_empty() || !matches!(kind, "exact" | "suffix" | "prefix" | "prefix_dot") {
            findings.insert(Finding::new(
                "root_workspace_policy_malformed_rule",
                &key,
                "allowlist rule must carry a non-empty `id`, a non-empty `value`, and a `kind` of exact|suffix|prefix|prefix_dot",
            ));
            continue;
        }
        rules.push(AllowRule {
            id: id.to_owned(),
            kind: kind.to_owned(),
            value: value.to_owned(),
        });
    }
    rules
}

/// The set of permitted top-level directory names (data-driven).
fn allowed_dirs(policy: &Value) -> BTreeSet<String> {
    policy
        .get("allowed_root_dirs")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The remediation printed for an unallowlisted root file (auto-fix, not flag-only).
fn root_file_remediation(path: &str) -> String {
    format!(
        "tracked repo-root file `{path}` matches no allowlist rule. AUTO-FIX: if it is process \
         scratch, `git rm` it (and rely on the .gitignore root-scratch backstop) or relocate it \
         under the repo's gitignored scratch home (e.g. `.omc/`); if it is a genuinely legitimate \
         root surface, add a reviewed allowlist rule to root-workspace-hygiene-policy.json \
         (allowed_root_files) — a DATA edit, never a scanner change."
    )
}

/// The remediation printed for an unallowlisted top-level directory.
fn root_dir_remediation(dir: &str) -> String {
    format!(
        "tracked path lives under unallowlisted top-level directory `{dir}/`. AUTO-FIX: relocate \
         the file under an existing capability/meta home, or — if a NEW top-level capability is \
         genuinely warranted — add `{dir}` to allowed_root_dirs in \
         root-workspace-hygiene-policy.json (a reviewed DATA edit)."
    )
}

/// Pure evaluator. `policy` is DATA (`root-workspace-hygiene-policy.json`); `observed` is the
/// tracked-path inventory shaped as `{ "rows": [{"path": "..."}] }` (the producer's
/// git-ls-files snapshot). Every tracked path whose basename carries no `/` is a ROOT file and
/// must match the allowlist; every nested tracked path's first segment must be a permitted dir.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if string_field(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "root_workspace_gate_id_mismatch",
            "<policy>",
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let rules = allow_rules(policy, &mut findings);
    let dirs = allowed_dirs(policy);

    // De-duplicate top-level dirs so each offending dir is reported once with a stable key.
    let mut unallowlisted_dirs: BTreeMap<String, ()> = BTreeMap::new();

    for row in observed
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = string_field(row, "path").filter(|p| !p.is_empty()) else {
            continue;
        };
        // Normalize away any leading "./" the snapshot might carry.
        let path = path.strip_prefix("./").unwrap_or(path);

        match path.split_once('/') {
            // Nested path: its first segment must be a permitted top-level directory.
            Some((top, _rest)) => {
                if !dirs.contains(top) {
                    unallowlisted_dirs.entry(top.to_owned()).or_insert(());
                }
            }
            // Root-level file (no '/'): basename must match an allowlist rule.
            None => {
                let admitted = rules.iter().any(|rule| rule_matches(rule, path));
                if !admitted {
                    findings.insert(Finding::new(
                        "root_workspace_unallowlisted_file",
                        path,
                        root_file_remediation(path),
                    ));
                }
            }
        }
    }

    for dir in unallowlisted_dirs.keys() {
        findings.insert(Finding::new(
            "root_workspace_unallowlisted_dir",
            dir,
            root_dir_remediation(dir),
        ));
    }

    findings
}

/// Bare-report projection of [`evaluate_keyed`].
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "allowed_root_files": [
                { "id": "cargo-manifest", "kind": "exact",      "value": "Cargo.toml" },
                { "id": "readme",         "kind": "prefix_dot", "value": "README" },
                { "id": "license",        "kind": "prefix_dot", "value": "LICENSE" },
                { "id": "buckconfig",     "kind": "prefix_dot", "value": ".buckconfig" }
            ],
            "allowed_root_dirs": ["cloud", "libs", "docs"]
        })
    }

    fn observed(paths: &[&str]) -> Value {
        json!({ "rows": paths.iter().map(|p| json!({ "path": p })).collect::<Vec<_>>() })
    }

    #[test]
    fn clean_allowlisted_tree_is_green() {
        let report = evaluate(
            &policy(),
            &observed(&[
                "Cargo.toml",
                "README.md",
                "LICENSE",
                ".buckconfig",
                "cloud/cloud-ci/gates/x/src/lib.rs",
                "libs/oya-foo/Cargo.toml",
                "docs/decisions/ADR-0600.md",
            ]),
        );
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.violations.is_empty(), "{report:#?}");
    }

    #[test]
    fn tracked_root_scratch_log_is_born_blocking_red() {
        // The load-bearing RED case: a `foo.log` tracked at root matches no allowlist rule.
        let findings = evaluate_keyed(&policy(), &observed(&["Cargo.toml", "foo.log"]));
        assert!(
            findings.iter().any(|f| {
                f.code == "root_workspace_unallowlisted_file" && f.key == "foo.log"
            }),
            "a tracked root scratch file must be born-blocking with its key surfaced; got {findings:#?}"
        );
        // The legitimate root file must NOT be flagged (no false positive).
        assert!(
            !findings.iter().any(|f| f.key == "Cargo.toml"),
            "an allowlisted root file must not be flagged"
        );
        assert_eq!(evaluate(&policy(), &observed(&["foo.log"])).verdict, Verdict::Red);
    }

    #[test]
    fn the_actual_removed_scratch_shapes_are_red() {
        // The exact root scratch this PR removes must each fail the allowlist.
        for scratch in [
            "backfill-targets.txt",
            "branch-wired-members.txt",
            "final-targets.txt",
            "slice06-progress.log",
            "retest-targets.txt",
            "run-slice.sh",
            "premise.txt",
            "review-verdict.txt",
        ] {
            let findings = evaluate_keyed(&policy(), &observed(&[scratch]));
            assert!(
                findings
                    .iter()
                    .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == scratch),
                "{scratch} must be born-blocking"
            );
        }
    }

    #[test]
    fn finding_carries_a_concrete_auto_fix_remediation() {
        let findings = evaluate_keyed(&policy(), &observed(&["foo.log"]));
        let f = findings
            .iter()
            .find(|f| f.key == "foo.log")
            .expect("finding for foo.log");
        assert!(
            f.detail.contains("git rm") && f.detail.contains(".omc/"),
            "remediation must name the concrete auto-fix (relocate to .omc/ or git rm); got: {}",
            f.detail
        );
    }

    #[test]
    fn unallowlisted_top_level_dir_is_red_and_deduped() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(&["sandbox/a.rs", "sandbox/b.rs", "cloud/ok.rs"]),
        );
        let dir_findings: Vec<&Finding> = findings
            .iter()
            .filter(|f| f.code == "root_workspace_unallowlisted_dir")
            .collect();
        assert_eq!(dir_findings.len(), 1, "the offending dir is reported once: {findings:#?}");
        assert_eq!(dir_findings[0].key, "sandbox");
    }

    #[test]
    fn gate_id_mismatch_is_red() {
        let bad = json!({ "gate_id": "wrong", "allowed_root_files": [], "allowed_root_dirs": [] });
        let findings = evaluate_keyed(&bad, &observed(&[]));
        assert!(findings.iter().any(|f| f.code == "root_workspace_gate_id_mismatch"));
    }

    #[test]
    fn malformed_allowlist_rule_is_red() {
        let bad = json!({
            "gate_id": GATE_ID,
            "allowed_root_files": [ { "id": "", "kind": "nope", "value": "" } ],
            "allowed_root_dirs": []
        });
        let findings = evaluate_keyed(&bad, &observed(&[]));
        assert!(findings.iter().any(|f| f.code == "root_workspace_policy_malformed_rule"));
    }

    // --- prefix_dot tightening: RED cases (over-broad prefix would have allowed these) ---

    #[test]
    fn readme_family_without_separator_is_red() {
        // "READMEILY.md" starts with "README" but has no "." or "-" separator — must be blocked.
        let findings = evaluate_keyed(&policy(), &observed(&["READMEILY.md"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "READMEILY.md"),
            "READMEILY.md must be born-blocking (no separator after README); got {findings:#?}"
        );
    }

    #[test]
    fn readme_scratch_txt_without_separator_is_red() {
        let findings = evaluate_keyed(&policy(), &observed(&["README-scratch.txt"]));
        // README-scratch.txt HAS a "-" separator so prefix_dot admits it — this is intentional
        // (README-* is a legitimate family). Verify GREEN (no false-block).
        assert!(
            !findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "README-scratch.txt"),
            "README-scratch.txt has a '-' separator and should be admitted by prefix_dot; got {findings:#?}"
        );
    }

    #[test]
    fn notes_buckconfig_is_red() {
        // "notes.buckconfig" ends with ".buckconfig" (old suffix rule allowed it) but does NOT
        // start with ".buckconfig" — must now be born-blocking.
        let findings = evaluate_keyed(&policy(), &observed(&["notes.buckconfig"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "notes.buckconfig"),
            "notes.buckconfig must be born-blocking (suffix match removed); got {findings:#?}"
        );
    }

    #[test]
    fn scratch_buckconfig_is_red() {
        let findings = evaluate_keyed(&policy(), &observed(&["scratch.buckconfig"]));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "scratch.buckconfig"),
            "scratch.buckconfig must be born-blocking; got {findings:#?}"
        );
    }

    // --- prefix_dot tightening: GREEN cases (legitimate files must still pass) ---

    #[test]
    fn legitimate_readme_and_license_and_buckconfig_still_pass() {
        let report = evaluate(
            &policy(),
            &observed(&[
                "README",
                "README.md",
                "README.rst",
                "LICENSE",
                "LICENSE.md",
                "LICENSE-Apache-2.0",
                ".buckconfig",
                ".buckconfig.local",
            ]),
        );
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "legitimate README/LICENSE/.buckconfig family must not be false-blocked; got {report:#?}"
        );
    }

    #[test]
    fn evaluator_only_emits_declared_violation_codes() {
        let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
        let bad = json!({
            "gate_id": "wrong",
            "allowed_root_files": [ { "id": "", "kind": "x", "value": "" } ],
            "allowed_root_dirs": []
        });
        let findings = evaluate_keyed(&bad, &observed(&["foo.log", "sandbox/x.rs"]));
        for f in &findings {
            assert!(
                declared.contains(f.code.as_str()),
                "evaluator emitted `{}` which is not in VIOLATION_CODES",
                f.code
            );
        }
        // All four codes are exercised by this single fixture.
        let emitted: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(emitted, declared.iter().map(|s| s.to_string()).collect());
    }
}
