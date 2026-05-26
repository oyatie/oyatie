//! Aspirational enforcement claim validator.
//!
//! The kernel is intentionally I/O-free. Runners provide corpus documents and
//! the known enforcement surfaces from crates, workflows, quality lanes, and branch
//! protection. The validator fails only explicit binding claims; advisory and
//! proposed lane mentions remain allowed.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KnownEnforcementSurfaces {
    pub crate_names: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub workflow_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub quality_lane_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub branch_required_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// All lane ids DECLARED in the quality-lane registry (any status). A
    /// binding enforcement claim that references a governance lane NOT declared
    /// here is treated as advisory/planned (a future lane), not a violation —
    /// only declared-but-unresolved lanes fail (ADR-0362 (a): planned refs are
    /// advisory regardless of prefix). data_class: INTERNAL_ONLY
    pub declared_lane_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
    pub lines_checked: usize,     // data_class: INTERNAL_ONLY
    pub binding_mentions: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalViolation {
    pub path: String,                // data_class: INTERNAL_ONLY
    pub line: usize,                 // data_class: INTERNAL_ONLY
    pub token: String,               // data_class: INTERNAL_ONLY
    pub kind: AspirationalIssueKind, // data_class: INTERNAL_ONLY
    pub summary: String,             // data_class: INTERNAL_ONLY
    pub fix: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AspirationalIssueKind {
    MissingCrate,
    MissingWorkflow,
    MissingQualityLane,
    MissingRequiredContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingBindingContext {
    indent: usize,
    requires_branch_context: bool,
}

impl fmt::Display for AspirationalViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {} ({}) fix: {}",
            self.path, self.line, self.kind, self.summary, self.token, self.fix
        )
    }
}

pub fn validate_aspirational_enforcement<D>(
    documents: D,
    known: &KnownEnforcementSurfaces,
) -> Result<AspirationalReport, Vec<AspirationalViolation>>
where
    D: IntoIterator<Item = AspirationalDocument>,
{
    let mut documents_checked = 0usize;
    let mut lines_checked = 0usize;
    let mut binding_mentions = 0usize;
    let mut violations = Vec::new();

    for document in documents {
        documents_checked += 1;
        let mut pending_binding_context = None::<PendingBindingContext>;
        for (index, raw_line) in document.contents.lines().enumerate() {
            lines_checked += 1;
            let line_number = index + 1;
            let lower = raw_line.to_ascii_lowercase();
            let indent = raw_line.len() - raw_line.trim_start().len();
            let stripped = raw_line.trim_start();
            if let Some(pending) = pending_binding_context
                && !stripped.is_empty()
                && indent <= pending.indent
                && !stripped.starts_with("- ")
                && !starts_multiline_binding_header(&lower)
            {
                pending_binding_context = None;
            }
            let tokens = enforcement_tokens(raw_line);
            let advisory = is_advisory_context(&lower);
            let current_line_is_binding = is_binding_context(&lower) && !advisory;
            let pending_context = pending_binding_context.filter(|_| !advisory);
            if starts_multiline_binding_header(&lower) && !advisory {
                pending_binding_context = Some(PendingBindingContext {
                    indent,
                    requires_branch_context: requires_branch_context(&lower),
                });
            }
            if tokens.is_empty() || (!current_line_is_binding && pending_context.is_none()) {
                continue;
            }
            let branch_context_required = requires_branch_context(&lower)
                || pending_context
                    .map(|pending| pending.requires_branch_context)
                    .unwrap_or(false);
            for token in tokens {
                binding_mentions += 1;
                if token.starts_with("oya-check-") {
                    if !known.crate_names.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingCrate,
                            summary: "binding claim references a missing check crate".to_string(),
                            fix: "add the crate or mark the claim advisory/proposed".to_string(),
                        });
                    }
                } else if token.starts_with("oya-governance-")
                    && known.declared_lane_ids.contains(&token)
                {
                    // Only DECLARED governance lanes are validated. A binding
                    // claim referencing an undeclared oya-governance-* lane is a
                    // planned/future lane => advisory (falls through, no
                    // violation), per ADR-0362 (a).
                    if !known.workflow_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingWorkflow,
                            summary: "binding claim references a missing workflow/job context"
                                .to_string(),
                            fix: "add the workflow/job or mark the claim advisory/proposed"
                                .to_string(),
                        });
                    }
                    if !known.quality_lane_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingQualityLane,
                            summary:
                                "binding claim references a missing active quality-lane registry row"
                                    .to_string(),
                            fix: "add the active quality-lane registry row or mark the claim advisory/proposed"
                                .to_string(),
                        });
                    }
                    if branch_context_required && !known.branch_required_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token,
                            kind: AspirationalIssueKind::MissingRequiredContext,
                            summary: "required-check claim is absent from branch protection"
                                .to_string(),
                            fix: "add the exact context to branch protection or remove the required-check claim"
                                .to_string(),
                        });
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(AspirationalReport {
            documents_checked,
            lines_checked,
            binding_mentions,
        })
    } else {
        Err(violations)
    }
}

fn enforcement_tokens(line: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for prefix in ["oya-governance-", "oya-check-"] {
        let mut start = 0usize;
        while let Some(relative_index) = line[start..].find(prefix) {
            let token_start = start + relative_index;
            let token_end = line[token_start..]
                .find(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '-' || character == '_')
                })
                .map_or(line.len(), |end| token_start + end);
            let token = &line[token_start..token_end];
            if !token.ends_with('-') && !token.ends_with('_') {
                tokens.insert(token.to_string());
            }
            start = token_end;
        }
    }
    tokens
}

fn is_binding_context(lower: &str) -> bool {
    [
        "blocks merge",
        "blocking",
        "branch protection",
        "enforced by",
        "enforced_by",
        "required check",
        "required status",
        "shall",
        "status: active",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_advisory_context(lower: &str) -> bool {
    if [
        "not advisory",
        "not merely advisory",
        "not just advisory",
        "no longer advisory",
        "not planned",
        "not proposed",
        "not candidate",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return false;
    }
    [
        "advisory",
        "backlog",
        "candidate",
        "not active",
        "not enforced",
        "not required",
        "not yet",
        "planned",
        "proposed",
        "retired",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn requires_branch_context(lower: &str) -> bool {
    lower.contains("branch protection")
        || lower.contains("blocks merge")
        || lower.contains("blocking")
        || lower.contains("required check")
        || lower.contains("required status")
}

fn starts_multiline_binding_header(lower: &str) -> bool {
    let stripped = lower.trim_start();
    stripped.starts_with("enforced_by:")
        || stripped.starts_with("\"enforced_by\":")
        || stripped.starts_with("'enforced_by':")
        || stripped.starts_with("required check:")
        || stripped.starts_with("required status:")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known() -> KnownEnforcementSurfaces {
        KnownEnforcementSurfaces {
            crate_names: BTreeSet::from(["oya-check-real".to_string()]),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            // Both declared lanes; "missing" is declared-but-unresolved so the
            // missing-* violation tests still fire, while undeclared tokens are
            // advisory (ADR-0362 (a)).
            declared_lane_ids: BTreeSet::from([
                "oya-governance-real".to_string(),
                "oya-governance-missing".to_string(),
            ]),
        }
    }

    fn doc(contents: &str) -> AspirationalDocument {
        AspirationalDocument {
            path: "docs/decisions/ADR-9999.md".to_string(),
            contents: contents.to_string(),
        }
    }

    #[test]
    fn accepts_binding_claims_with_real_surfaces() {
        let report = validate_aspirational_enforcement(
            [doc(
                "enforced_by: oya-check-real\nrequired check: oya-governance-real\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 2);
    }

    #[test]
    fn accepts_planned_missing_lane_mentions() {
        let report = validate_aspirational_enforcement(
            [doc(
                "candidate validator oya-governance-missing remains planned and advisory\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn treats_undeclared_governance_enforced_by_as_advisory() {
        // ADR-0362 (a): a binding `enforced_by:` claim referencing an
        // oya-governance-* lane that is NOT declared in the registry is a
        // planned/future lane => advisory, not a violation.
        let report = validate_aspirational_enforcement(
            [doc("enforced_by: oya-governance-doc-rigor\n")],
            &known(),
        )
        .expect("undeclared governance lane ref is advisory, not a violation");
        // The token is seen on a binding line (counted) but not flagged, because
        // the lane is undeclared => treated as planned/advisory.
        assert_eq!(report.binding_mentions, 1);
    }

    #[test]
    fn accepts_non_binding_workflow_mentions() {
        let report = validate_aspirational_enforcement(
            [doc(
                "workflow catalog documents oya-governance-missing as future context\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn ignores_wildcard_prefix_mentions() {
        let report = validate_aspirational_enforcement(
            [doc("required check family oya-check-*\n")],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn rejects_missing_check_crates() {
        let violations =
            validate_aspirational_enforcement([doc("enforced_by: oya-check-missing\n")], &known())
                .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_missing_workflow_contexts() {
        let violations = validate_aspirational_enforcement(
            [doc("required check: oya-governance-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_missing_quality_lane_contexts() {
        let known = KnownEnforcementSurfaces {
            crate_names: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::new(),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations =
            validate_aspirational_enforcement([doc("enforced_by: oya-governance-real\n")], &known)
                .unwrap_err();
        assert_eq!(
            violations[0].kind,
            AspirationalIssueKind::MissingQualityLane
        );
    }

    #[test]
    fn rejects_missing_branch_required_contexts() {
        let known = KnownEnforcementSurfaces {
            crate_names: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations = validate_aspirational_enforcement(
            [doc(
                "branch protection required check: oya-governance-real\n",
            )],
            &known,
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingRequiredContext)
        );
    }

    #[test]
    fn rejects_negated_advisory_binding_claims() {
        let violations = validate_aspirational_enforcement(
            [doc(
                "required check: oya-governance-missing is active, not advisory\n",
            )],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_multiline_enforced_by_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("enforced_by:\n  - oya-check-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_same_indent_yaml_enforced_by_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("enforced_by:\n- oya-check-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_same_indent_yaml_required_check_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("required check:\n- oya-governance-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_same_indent_yaml_required_status_claims_without_branch_context() {
        let known = KnownEnforcementSurfaces {
            crate_names: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations = validate_aspirational_enforcement(
            [doc("required status:\n- oya-governance-real\n")],
            &known,
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingRequiredContext)
        );
    }
}
