//! `oya gate validate adr-planning-completeness` (ADR-0364 D2).
//!
//! Scans `planning_impact: true` ADRs and enforces the generative-template
//! contract (ADR-0364 §2):
//!
//!  - For ADRs that HAVE a `deliverables` field: FAIL if any deliverable lacks
//!    `id` / `description` / `exit_criteria` / `verified_by`, or if `milestone`
//!    is absent.
//!  - For planning_impact ADRs WITHOUT a `deliverables` field: ADVISORY count
//!    only (NOT a failure). Backfilling the legacy ADRs into the generative
//!    template is deferred to ADR-0364 D7 (re-foundation).
//!
//! Prints `validation passed: N complete, M advisory-missing-deliverables`.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::adr_planning_frontmatter::{PlanningAdr, read_planning_impact_adrs};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdrPlanningCompletenessArgs {
    pub(crate) decisions_dir: PathBuf,
}

pub(crate) fn parse_adr_planning_completeness_args(
    args: Vec<String>,
) -> Result<AdrPlanningCompletenessArgs, String> {
    let mut parsed = AdrPlanningCompletenessArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--decisions-dir requires a value".to_string())?,
                );
            }
            other => {
                return Err(format!(
                    "adr-planning-completeness: unknown flag {other:?}; allowed: --decisions-dir"
                ));
            }
        }
    }
    Ok(parsed)
}

#[derive(Default, Debug, Eq, PartialEq)]
pub(crate) struct AdrPlanningCompletenessReport {
    pub(crate) complete: usize,
    pub(crate) advisory_missing_deliverables: usize,
    pub(crate) failures: Vec<String>,
}

/// Validate one ADR; push any failures (only for ADRs that DECLARE a
/// deliverables field). Returns the classification for counting.
fn classify(adr: &PlanningAdr, failures: &mut Vec<String>) -> Classification {
    if !adr.has_deliverables_field {
        // Advisory: backfill deferred to ADR-0364 D7.
        return Classification::AdvisoryMissingDeliverables;
    }
    let mut adr_failures = Vec::new();
    if adr.milestone.trim().is_empty() {
        adr_failures.push(format!(
            "[MILESTONE_MISSING] {} declares deliverables but has no `milestone`",
            adr.id
        ));
    }
    if adr.deliverables.is_empty() {
        adr_failures.push(format!(
            "[DELIVERABLES_EMPTY] {} has a `deliverables:` field with no entries",
            adr.id
        ));
    }
    for (index, deliverable) in adr.deliverables.iter().enumerate() {
        let label = if deliverable.id.trim().is_empty() {
            format!("#{}", index + 1)
        } else {
            deliverable.id.clone()
        };
        let mut missing = Vec::new();
        if deliverable.id.trim().is_empty() {
            missing.push("id");
        }
        if deliverable.description.trim().is_empty() {
            missing.push("description");
        }
        if deliverable.exit_criteria.trim().is_empty() {
            missing.push("exit_criteria");
        }
        if deliverable.verified_by.trim().is_empty() {
            missing.push("verified_by");
        }
        if !missing.is_empty() {
            adr_failures.push(format!(
                "[DELIVERABLE_INCOMPLETE] {} deliverable {label} missing: {}",
                adr.id,
                missing.join(", ")
            ));
        }
    }
    if adr_failures.is_empty() {
        Classification::Complete
    } else {
        failures.extend(adr_failures);
        Classification::Failed
    }
}

enum Classification {
    Complete,
    AdvisoryMissingDeliverables,
    Failed,
}

pub(crate) fn validate_adr_planning_completeness_gate(
    args: AdrPlanningCompletenessArgs,
) -> Result<AdrPlanningCompletenessReport, String> {
    let adrs = read_planning_impact_adrs(&args.decisions_dir)?;
    let mut report = AdrPlanningCompletenessReport::default();
    for adr in &adrs {
        match classify(adr, &mut report.failures) {
            Classification::Complete => report.complete += 1,
            Classification::AdvisoryMissingDeliverables => {
                report.advisory_missing_deliverables += 1
            }
            Classification::Failed => {}
        }
    }
    Ok(report)
}

pub(crate) fn run_adr_planning_completeness(args: Vec<String>) -> ExitCode {
    let parsed = match parse_adr_planning_completeness_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match validate_adr_planning_completeness_gate(parsed) {
        Ok(report) => {
            if report.failures.is_empty() {
                println!(
                    "adr-planning-completeness validation passed: {} complete, {} advisory-missing-deliverables",
                    report.complete, report.advisory_missing_deliverables
                );
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "adr-planning-completeness validation failed: {} failing deliverable(s) [{} complete, {} advisory-missing-deliverables]",
                    report.failures.len(),
                    report.complete,
                    report.advisory_missing_deliverables
                );
                for failure in &report.failures {
                    eprintln!("  - {failure}");
                }
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("adr-planning-completeness validation error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr_planning_frontmatter::{PlanningAdr, PlanningDeliverable};

    fn complete_deliverable(id: &str) -> PlanningDeliverable {
        PlanningDeliverable {
            id: id.into(),
            description: "d".into(),
            exit_criteria: "e".into(),
            verified_by: "v".into(),
        }
    }

    fn base(id: &str) -> PlanningAdr {
        PlanningAdr {
            id: id.into(),
            status: "Accepted".into(),
            milestone: "M1".into(),
            depends_on: vec![],
            has_deliverables_field: true,
            deliverables: vec![complete_deliverable(&format!("{id}-D1"))],
            path: format!("docs/decisions/{id}-x.md"),
        }
    }

    #[test]
    fn complete_adr_passes() {
        let mut failures = Vec::new();
        assert!(matches!(
            classify(&base("ADR-0364"), &mut failures),
            Classification::Complete
        ));
        assert!(failures.is_empty());
    }

    #[test]
    fn missing_milestone_fails() {
        let mut adr = base("ADR-0001");
        adr.milestone = String::new();
        let mut failures = Vec::new();
        assert!(matches!(
            classify(&adr, &mut failures),
            Classification::Failed
        ));
        assert!(failures.iter().any(|f| f.contains("MILESTONE_MISSING")));
    }

    #[test]
    fn incomplete_deliverable_fails() {
        let mut adr = base("ADR-0002");
        adr.deliverables[0].verified_by = String::new();
        let mut failures = Vec::new();
        assert!(matches!(
            classify(&adr, &mut failures),
            Classification::Failed
        ));
        assert!(failures.iter().any(|f| f.contains("verified_by")));
    }

    #[test]
    fn no_deliverables_field_is_advisory_not_failure() {
        let mut adr = base("ADR-0003");
        adr.has_deliverables_field = false;
        adr.deliverables.clear();
        adr.milestone = String::new(); // advisory ADRs are not checked for milestone
        let mut failures = Vec::new();
        assert!(matches!(
            classify(&adr, &mut failures),
            Classification::AdvisoryMissingDeliverables
        ));
        assert!(failures.is_empty());
    }
}
