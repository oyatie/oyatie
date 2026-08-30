//! Authoring-time lint over a policy version: conflicts, duplicates, and
//! shadowing, classified by blocking severity.

use serde::{Deserialize, Serialize};

use crate::policy::PolicyVersion;

// ── Cedar policy authoring-time lint ─────────────────────────────────────────

/// Severity of a lint finding.
///
/// `Error` findings block publish; `Warning` findings are advisory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
}

/// A single finding produced by the authoring-time lint pass.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyLintFinding {
    pub severity: LintSeverity,
    /// Indices into `PolicyVersion::rules` of the rules involved in this finding.
    pub rule_indices: Vec<usize>,
    pub reason: String,
}

/// The aggregated result of linting a candidate `PolicyVersion`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyLintReport {
    pub findings: Vec<PolicyLintFinding>,
}

impl PolicyLintReport {
    /// Returns `true` if any finding has `LintSeverity::Error`.
    pub fn has_blocking(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == LintSeverity::Error)
    }

    /// Returns `true` if there are no findings at all.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Lint a candidate `PolicyVersion` without publishing it.
///
/// Detects:
/// - Intra-policy conflicts: an Allow and a Deny rule on identical
///   `(principal_role, action, resource_prefix, required_attribute)` → `Error`.
/// - Duplicate rules: two rules with identical
///   `(effect, principal_role, action, resource_prefix, required_attribute)` → `Error`.
/// - Shadowed/unreachable rules: a later same-effect rule whose
///   `resource_prefix` is subsumed by an earlier rule's prefix and whose
///   `required_attribute` is equal-or-weaker (the earlier rule's attribute
///   guard subsumes the later one) → `Warning`.
///
/// Pure, deterministic, no network or storage access.
pub fn lint_policy_version(version: &PolicyVersion) -> PolicyLintReport {
    let rules = &version.rules;
    let mut findings: Vec<PolicyLintFinding> = Vec::new();

    for i in 0..rules.len() {
        for j in (i + 1)..rules.len() {
            let a = &rules[i];
            let b = &rules[j];

            let same_tuple = a.principal_role == b.principal_role
                && a.action == b.action
                && a.resource_prefix == b.resource_prefix
                && a.required_attribute == b.required_attribute;

            if same_tuple {
                if a.effect == b.effect {
                    // Duplicate rule (same effect + identical tuple).
                    findings.push(PolicyLintFinding {
                        severity: LintSeverity::Error,
                        rule_indices: vec![i, j],
                        reason: format!(
                            "rules {i} and {j} are duplicates: identical (effect, principal_role, \
                             action, resource_prefix, required_attribute)"
                        ),
                    });
                } else {
                    // Conflicting Allow/Deny pair on identical tuple.
                    findings.push(PolicyLintFinding {
                        severity: LintSeverity::Error,
                        rule_indices: vec![i, j],
                        reason: format!(
                            "rules {i} and {j} conflict: Allow and Deny on identical \
                             (principal_role, action, resource_prefix, required_attribute)"
                        ),
                    });
                }
            } else if a.effect == b.effect
                && a.principal_role == b.principal_role
                && a.action == b.action
                && b.resource_prefix.starts_with(&a.resource_prefix)
                && attr_subsumed_by(&b.required_attribute, &a.required_attribute)
            {
                // Later rule j is shadowed/unreachable under earlier rule i.
                findings.push(PolicyLintFinding {
                    severity: LintSeverity::Warning,
                    rule_indices: vec![i, j],
                    reason: format!(
                        "rule {j} is unreachable: its resource_prefix {:?} is subsumed by rule \
                         {i}'s prefix {:?} with an equal-or-weaker attribute guard",
                        b.resource_prefix, a.resource_prefix
                    ),
                });
            }
        }
    }

    PolicyLintReport { findings }
}

/// Returns `true` if `candidate`'s attribute guard is subsumed by (i.e., at
/// least as restrictive as) `dominator`'s attribute guard.
///
/// - `dominator = None` matches everything → always subsumes.
/// - `dominator = Some(x)` and `candidate = Some(x)` → equal → subsumes.
/// - `dominator = Some(x)` and `candidate = None` → candidate is broader → does NOT subsume.
fn attr_subsumed_by(
    candidate: &Option<(String, String)>,
    dominator: &Option<(String, String)>,
) -> bool {
    match (candidate, dominator) {
        (_, None) => true,
        (Some(c), Some(d)) => c == d,
        (None, Some(_)) => false,
    }
}
