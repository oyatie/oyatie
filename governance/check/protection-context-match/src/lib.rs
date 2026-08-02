//! Foundry branch-protection / workflow job-name match kernel.
//!
//! Asserts the invariant that every `required_status_checks` context
//! in `.github/branch-protection.yaml` is the `name:` field of some
//! job in some `.github/workflows/*.yml`. When the names diverge,
//! GitHub silently waits forever for non-existent check_runs and the
//! protected branch becomes silently un-merge-able — the failure
//! class that the 2026-05-15 PR #3 merge-block surfaced.
//!
//! Lane id: `oya-governance-protection-context-match`. The
//! lane is the machine-checkable encoding of the
//! [[feedback_no_silent_regression]] directive applied to the
//! protection/workflow seam.
//!
//! Naming justification: crate name follows the `check-<lane>`
//! family. Kernel sits on the `domain` layer (port-in-kernel,
//! ADR-0056); pure I/O-free static matching of strings handed in by
//! the runner. Types use the noun form so callers can grep one
//! prefix.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// Per-workflow record: the workflow's file path plus every job
/// `name:` field it declares. Used by the kernel to look up whether
/// a given protection context is posted by some workflow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowJobNames {
    pub workflow_path: String,  // data_class: INTERNAL_ONLY
    pub job_names: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Per-run report (positive shape — fields recorded for evidence).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectionContextMatchReport {
    pub contexts_checked: usize,       // data_class: INTERNAL_ONLY
    pub workflow_jobs_indexed: usize,  // data_class: INTERNAL_ONLY
    pub workflows_indexed: usize,      // data_class: INTERNAL_ONLY
    pub matched_contexts: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Validation error variants.
///
/// Naming justification: variants describe the missing positive scope
/// in canonical terms (no "exception" / "exempt" phrasing per
/// [[feedback_no_exceptions_canonical]]). Each variant carries the
/// data the runner needs to render an exact remediation hint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectionContextMatchError {
    EmptyRequiredContexts,
    EmptyWorkflowIndex,
    ContextMissingFromWorkflows {
        context: String,
        all_job_names: Vec<String>,
    },
    DuplicateRequiredContext(String),
}

impl fmt::Display for ProtectionContextMatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequiredContexts => write!(
                formatter,
                "branch-protection has zero required contexts — the gate \
                 is silently empty"
            ),
            Self::EmptyWorkflowIndex => write!(
                formatter,
                "workflow index has zero jobs — no workflow `name:` fields \
                 to match against"
            ),
            Self::ContextMissingFromWorkflows {
                context,
                all_job_names,
            } => {
                writeln!(
                    formatter,
                    "required context `{context}` is not posted by any workflow job. \
                     GitHub will wait forever for a check_run with this name."
                )?;
                writeln!(
                    formatter,
                    "  Indexed workflow `name:` fields ({} total):",
                    all_job_names.len()
                )?;
                for name in all_job_names {
                    writeln!(formatter, "    - {name}")?;
                }
                writeln!(
                    formatter,
                    "  Fix: rename a workflow job `name:` field to `{context}` \
                     OR remove `{context}` from `required_status_checks`."
                )
            }
            Self::DuplicateRequiredContext(context) => write!(
                formatter,
                "duplicate required context `{context}` — every entry must be unique"
            ),
        }
    }
}

impl std::error::Error for ProtectionContextMatchError {}

/// Validate the protection contexts against the workflow index.
///
/// Returns `Ok(report)` when every required context is the `name:` of
/// some job in some workflow; otherwise returns the FIRST missing
/// context (callers can re-run after fixing one to surface the next).
///
/// The kernel performs no I/O. The runner is responsible for parsing
/// `.github/branch-protection.yaml` (required_status_checks contexts)
/// and walking `.github/workflows/*.yml` (job `name:` fields).
pub fn validate_protection_context_match(
    required_contexts: &[String],
    workflows: &[WorkflowJobNames],
) -> Result<ProtectionContextMatchReport, ProtectionContextMatchError> {
    if required_contexts.is_empty() {
        return Err(ProtectionContextMatchError::EmptyRequiredContexts);
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for context in required_contexts {
        if !seen.insert(context.as_str()) {
            return Err(ProtectionContextMatchError::DuplicateRequiredContext(
                context.clone(),
            ));
        }
    }

    if workflows.is_empty() {
        return Err(ProtectionContextMatchError::EmptyWorkflowIndex);
    }
    let all_job_names: Vec<String> = workflows
        .iter()
        .flat_map(|workflow| workflow.job_names.iter().cloned())
        .collect();
    if all_job_names.is_empty() {
        return Err(ProtectionContextMatchError::EmptyWorkflowIndex);
    }
    let job_name_set: BTreeSet<&str> = all_job_names.iter().map(String::as_str).collect();

    let mut matched: Vec<String> = Vec::new();
    for context in required_contexts {
        if !job_name_set.contains(context.as_str()) {
            return Err(ProtectionContextMatchError::ContextMissingFromWorkflows {
                context: context.clone(),
                all_job_names: all_job_names.clone(),
            });
        }
        matched.push(context.clone());
    }

    Ok(ProtectionContextMatchReport {
        contexts_checked: required_contexts.len(),
        workflow_jobs_indexed: all_job_names.len(),
        workflows_indexed: workflows.len(),
        matched_contexts: matched,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workflow(path: &str, names: &[&str]) -> WorkflowJobNames {
        WorkflowJobNames {
            workflow_path: path.to_string(),
            job_names: names.iter().map(|n| (*n).to_string()).collect(),
        }
    }

    fn s(value: &str) -> String {
        value.to_string()
    }

    #[test]
    fn accepts_canonical_aligned_inputs() {
        let contexts = vec![s("cargo-fmt"), s("cargo-clippy")];
        let workflows = vec![workflow("pr-tests.yml", &["cargo-fmt", "cargo-clippy"])];
        let report = validate_protection_context_match(&contexts, &workflows)
            .expect("aligned inputs are accepted");
        assert_eq!(report.contexts_checked, 2);
        assert_eq!(report.workflow_jobs_indexed, 2);
        assert_eq!(report.workflows_indexed, 1);
        assert_eq!(report.matched_contexts, contexts);
    }

    #[test]
    fn accepts_contexts_matched_across_multiple_workflows() {
        let contexts = vec![s("cargo-fmt"), s("oya-governance-supply-chain")];
        let workflows = vec![
            workflow(
                "pr-tests.yml",
                &["cargo-fmt", "cargo-clippy", "cargo-check"],
            ),
            workflow(
                "oya-governance-supply-chain.yml",
                &["oya-governance-supply-chain"],
            ),
        ];
        let report = validate_protection_context_match(&contexts, &workflows)
            .expect("multi-workflow match accepted");
        assert_eq!(report.workflows_indexed, 2);
        assert_eq!(report.workflow_jobs_indexed, 4);
    }

    #[test]
    fn flags_missing_context_with_remediation_hint() {
        let contexts = vec![s("cargo-fmt"), s("missing-gate")];
        let workflows = vec![workflow("pr-tests.yml", &["cargo-fmt", "cargo-clippy"])];
        let error = validate_protection_context_match(&contexts, &workflows).unwrap_err();
        match error {
            ProtectionContextMatchError::ContextMissingFromWorkflows {
                context,
                all_job_names,
            } => {
                assert_eq!(context, "missing-gate");
                assert_eq!(all_job_names, vec![s("cargo-fmt"), s("cargo-clippy")]);
            }
            other => panic!("expected ContextMissingFromWorkflows, got {other:?}"),
        }
    }

    #[test]
    fn flags_display_name_vs_job_key_mismatch() {
        // The 2026-05-15 PR #3 silent-bypass: branch-protection lists
        // `cargo-fmt` (the job-key), but the workflow posts
        // `cargo fmt --check` (the `name:` field). The kernel must
        // catch this exact failure class.
        let contexts = vec![s("cargo-fmt")];
        let workflows = vec![workflow("pr-tests.yml", &["cargo fmt --check"])];
        let error = validate_protection_context_match(&contexts, &workflows).unwrap_err();
        assert!(matches!(
            error,
            ProtectionContextMatchError::ContextMissingFromWorkflows { .. }
        ));
    }

    #[test]
    fn rejects_empty_required_contexts() {
        let contexts: Vec<String> = Vec::new();
        let workflows = vec![workflow("pr-tests.yml", &["cargo-fmt"])];
        assert_eq!(
            validate_protection_context_match(&contexts, &workflows).unwrap_err(),
            ProtectionContextMatchError::EmptyRequiredContexts
        );
    }

    #[test]
    fn rejects_empty_workflow_index() {
        let contexts = vec![s("cargo-fmt")];
        let workflows: Vec<WorkflowJobNames> = Vec::new();
        assert_eq!(
            validate_protection_context_match(&contexts, &workflows).unwrap_err(),
            ProtectionContextMatchError::EmptyWorkflowIndex
        );
    }

    #[test]
    fn rejects_duplicate_required_context() {
        let contexts = vec![s("cargo-fmt"), s("cargo-fmt")];
        let workflows = vec![workflow("pr-tests.yml", &["cargo-fmt"])];
        let error = validate_protection_context_match(&contexts, &workflows).unwrap_err();
        assert!(matches!(
            error,
            ProtectionContextMatchError::DuplicateRequiredContext(_)
        ));
    }

    #[test]
    fn workflow_with_zero_named_jobs_treated_as_empty_index() {
        let contexts = vec![s("cargo-fmt")];
        let workflows = vec![WorkflowJobNames {
            workflow_path: "empty.yml".to_string(),
            job_names: Vec::new(),
        }];
        assert_eq!(
            validate_protection_context_match(&contexts, &workflows).unwrap_err(),
            ProtectionContextMatchError::EmptyWorkflowIndex
        );
    }
}
