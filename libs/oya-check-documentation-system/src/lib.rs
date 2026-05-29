//! Documentation-system fitness kernel.
//!
//! `docs/DOCUMENTATION.md` names a productized documentation pipeline. This
//! kernel prevents that contract from drifting into prose-only intent: every
//! documented generator is represented in `registry/docs/pipeline.tsv`, and the
//! repo must either wire an active/adoption guard or carry a registry rationale
//! for a deferred generator.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

const REQUIRED_PIPELINE_STEPS: [&str; 6] = [
    "rustdoc",
    "openapi",
    "mdbook",
    "adr-index",
    "catalog",
    "lint",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentationPipelineState {
    Active,
    AdoptionGuard,
    TrackedDeferred,
}

impl DocumentationPipelineState {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "adoption-guard" => Some(Self::AdoptionGuard),
            "tracked-deferred" => Some(Self::TrackedDeferred),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::AdoptionGuard => "adoption-guard",
            Self::TrackedDeferred => "tracked-deferred",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationPipelineRecord {
    pub step_id: String,                   // data_class: INTERNAL_ONLY
    pub documented_command: String,        // data_class: INTERNAL_ONLY
    pub state: DocumentationPipelineState, // data_class: INTERNAL_ONLY
    pub check_command: Option<String>,     // data_class: INTERNAL_ONLY
    pub check_command_wired: bool,         // data_class: INTERNAL_ONLY
    pub scope_path: String,                // data_class: INTERNAL_ONLY
    pub scope_present: bool,               // data_class: INTERNAL_ONLY
    pub rationale: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationSystemEvidence {
    pub documentation_lane_declared: bool, // data_class: INTERNAL_ONLY
    pub wiki_quickref_referenced: bool,    // data_class: INTERNAL_ONLY
    pub wiki_quickref_present: bool,       // data_class: INTERNAL_ONLY
    pub records: Vec<DocumentationPipelineRecord>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentationSystemReport {
    pub pipeline_records_checked: usize, // data_class: INTERNAL_ONLY
    pub active_records: usize,           // data_class: INTERNAL_ONLY
    pub adoption_guard_records: usize,   // data_class: INTERNAL_ONLY
    pub tracked_deferred_records: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentationSystemError {
    DocumentationLaneNotDeclared,
    WikiQuickrefMissing,
    MissingPipelineRecord { step_id: String },
    ExtraPipelineRecord { step_id: String },
    DuplicatePipelineRecord { step_id: String },
    InvalidPipelineRecord { step_id: String, reason: String },
    UnwiredPipelineCommand { step_id: String, command: String },
}

pub fn validate_documentation_system(
    evidence: DocumentationSystemEvidence,
) -> Result<DocumentationSystemReport, DocumentationSystemError> {
    if !evidence.documentation_lane_declared {
        return Err(DocumentationSystemError::DocumentationLaneNotDeclared);
    }
    if evidence.wiki_quickref_referenced && !evidence.wiki_quickref_present {
        return Err(DocumentationSystemError::WikiQuickrefMissing);
    }

    let records = record_map(evidence.records)?;
    let required = REQUIRED_PIPELINE_STEPS.into_iter().collect::<BTreeSet<_>>();
    for step_id in &required {
        if !records.contains_key(*step_id) {
            return Err(DocumentationSystemError::MissingPipelineRecord {
                step_id: (*step_id).into(),
            });
        }
    }
    if let Some(step_id) = records
        .keys()
        .find(|step_id| !required.contains(step_id.as_str()))
    {
        return Err(DocumentationSystemError::ExtraPipelineRecord {
            step_id: step_id.clone(),
        });
    }

    let mut active_records = 0usize;
    let mut adoption_guard_records = 0usize;
    let mut tracked_deferred_records = 0usize;
    for record in records.values() {
        validate_record(record)?;
        match record.state {
            DocumentationPipelineState::Active => active_records += 1,
            DocumentationPipelineState::AdoptionGuard => adoption_guard_records += 1,
            DocumentationPipelineState::TrackedDeferred => tracked_deferred_records += 1,
        }
    }

    Ok(DocumentationSystemReport {
        pipeline_records_checked: records.len(),
        active_records,
        adoption_guard_records,
        tracked_deferred_records,
    })
}

fn record_map(
    records: Vec<DocumentationPipelineRecord>,
) -> Result<BTreeMap<String, DocumentationPipelineRecord>, DocumentationSystemError> {
    let mut map = BTreeMap::new();
    for record in records {
        if map.insert(record.step_id.clone(), record.clone()).is_some() {
            return Err(DocumentationSystemError::DuplicatePipelineRecord {
                step_id: record.step_id,
            });
        }
    }
    Ok(map)
}

fn validate_record(record: &DocumentationPipelineRecord) -> Result<(), DocumentationSystemError> {
    if !valid_step_id(&record.step_id) {
        return Err(DocumentationSystemError::InvalidPipelineRecord {
            step_id: record.step_id.clone(),
            reason: "step_id must be lowercase alphanumeric plus hyphen".into(),
        });
    }
    if !record.documented_command.starts_with("oya doc ") {
        return Err(DocumentationSystemError::InvalidPipelineRecord {
            step_id: record.step_id.clone(),
            reason: "documented_command must name an oya doc subcommand".into(),
        });
    }
    if record.scope_path.trim().is_empty() || record.scope_path.contains('\t') {
        return Err(DocumentationSystemError::InvalidPipelineRecord {
            step_id: record.step_id.clone(),
            reason: "scope_path must be non-empty and tab-free".into(),
        });
    }
    if record.state != DocumentationPipelineState::Active && record.rationale.trim().is_empty() {
        return Err(DocumentationSystemError::InvalidPipelineRecord {
            step_id: record.step_id.clone(),
            reason: "non-active documentation generators require a rationale".into(),
        });
    }
    if record.state == DocumentationPipelineState::TrackedDeferred
        && !record.rationale.contains("blocked:")
    {
        return Err(DocumentationSystemError::InvalidPipelineRecord {
            step_id: record.step_id.clone(),
            reason: "tracked-deferred generators require a blocked: rationale".into(),
        });
    }
    if matches!(
        record.state,
        DocumentationPipelineState::Active | DocumentationPipelineState::AdoptionGuard
    ) {
        let Some(command) = record.check_command.as_ref().map(|command| command.trim()) else {
            return Err(DocumentationSystemError::InvalidPipelineRecord {
                step_id: record.step_id.clone(),
                reason: "active/adoption-guard generators require check_command".into(),
            });
        };
        if command.is_empty() {
            return Err(DocumentationSystemError::InvalidPipelineRecord {
                step_id: record.step_id.clone(),
                reason: "check_command must be non-empty".into(),
            });
        }
        if !record.check_command_wired {
            return Err(DocumentationSystemError::UnwiredPipelineCommand {
                step_id: record.step_id.clone(),
                command: command.into(),
            });
        }
    }
    Ok(())
}

fn valid_step_id(step_id: &str) -> bool {
    !step_id.is_empty()
        && step_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_documented_pipeline_with_active_guards_and_deferred_rationale() {
        let report = validate_documentation_system(evidence(records()))
            .expect("documentation system validates");

        assert_eq!(report.pipeline_records_checked, 6);
        assert_eq!(report.active_records, 2);
        assert_eq!(report.adoption_guard_records, 3);
        assert_eq!(report.tracked_deferred_records, 1);
    }

    #[test]
    fn rejects_missing_phantom_docs_lane_declaration() {
        assert_eq!(
            validate_documentation_system(DocumentationSystemEvidence {
                documentation_lane_declared: false,
                ..evidence(records())
            }),
            Err(DocumentationSystemError::DocumentationLaneNotDeclared)
        );
    }

    #[test]
    fn rejects_missing_wiki_quickref_when_documentation_references_it() {
        assert_eq!(
            validate_documentation_system(DocumentationSystemEvidence {
                wiki_quickref_present: false,
                ..evidence(records())
            }),
            Err(DocumentationSystemError::WikiQuickrefMissing)
        );
    }

    #[test]
    fn rejects_missing_required_pipeline_step() {
        let mut records = records();
        records.retain(|record| record.step_id != "mdbook");

        assert_eq!(
            validate_documentation_system(evidence(records)),
            Err(DocumentationSystemError::MissingPipelineRecord {
                step_id: "mdbook".into(),
            })
        );
    }

    #[test]
    fn rejects_unwired_active_or_adoption_guard_command() {
        let mut records = records();
        records[1].check_command_wired = false;

        assert_eq!(
            validate_documentation_system(evidence(records)),
            Err(DocumentationSystemError::UnwiredPipelineCommand {
                step_id: "openapi".into(),
                command: "cargo run -p oya-dev-cli -- gate validate api-semver".into(),
            })
        );
    }

    #[test]
    fn rejects_deferred_generator_without_blocked_rationale() {
        let mut records = records();
        records[0].rationale = "not yet".into();

        assert_eq!(
            validate_documentation_system(evidence(records)),
            Err(DocumentationSystemError::InvalidPipelineRecord {
                step_id: "rustdoc".into(),
                reason: "tracked-deferred generators require a blocked: rationale".into(),
            })
        );
    }

    fn evidence(records: Vec<DocumentationPipelineRecord>) -> DocumentationSystemEvidence {
        DocumentationSystemEvidence {
            documentation_lane_declared: true,
            wiki_quickref_referenced: true,
            wiki_quickref_present: true,
            records,
        }
    }

    fn records() -> Vec<DocumentationPipelineRecord> {
        vec![
            record(
                "rustdoc",
                DocumentationPipelineState::TrackedDeferred,
                None,
                false,
                "crates",
                true,
                "blocked: full rustdoc artifact publication is not part of the bootstrap lane",
            ),
            record(
                "openapi",
                DocumentationPipelineState::AdoptionGuard,
                Some("cargo run -p oya-dev-cli -- gate validate api-semver"),
                true,
                "contracts",
                false,
                "contracts are absent; api-semver guards first contract adoption",
            ),
            record(
                "mdbook",
                DocumentationPipelineState::AdoptionGuard,
                Some("cargo run -p oya-dev-cli -- gate validate documentation-system"),
                true,
                "docs/site",
                false,
                "public mdbook source is absent; documentation-system guards the pipeline registry",
            ),
            record(
                "adr-index",
                DocumentationPipelineState::AdoptionGuard,
                Some("cargo run -p oya-dev-cli -- gate validate adr-citation"),
                true,
                "docs/decisions",
                true,
                "adr-citation prevents stale ADR references until generator publication ships",
            ),
            record(
                "catalog",
                DocumentationPipelineState::Active,
                Some("cargo run -p oya-dev-cli -- catalog validate"),
                true,
                "registry/catalog",
                true,
                "",
            ),
            record(
                "lint",
                DocumentationPipelineState::Active,
                Some("cargo run -p oya-dev-cli -- gate validate doc-catalog"),
                true,
                "docs",
                true,
                "",
            ),
        ]
    }

    fn record(
        step_id: &str,
        state: DocumentationPipelineState,
        check_command: Option<&str>,
        check_command_wired: bool,
        scope_path: &str,
        scope_present: bool,
        rationale: &str,
    ) -> DocumentationPipelineRecord {
        DocumentationPipelineRecord {
            step_id: step_id.into(),
            documented_command: format!("oya doc {step_id}"),
            state,
            check_command: check_command.map(str::to_string),
            check_command_wired,
            scope_path: scope_path.into(),
            scope_present,
            rationale: rationale.into(),
        }
    }
}
