//! Governance check for documentation substance floors.
//!
//! The lane scans Markdown documents with YAML frontmatter `doc_class` and
//! enforces the line floors declared by `docs/standards/documentation-rigor.md`
//! §2 "Doc-class rigor matrix" (the task calls this the documentation-rigor
//! §1.2 line-floor source). Documents without frontmatter `doc_class` are not
//! in scope for this crate.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "documentation-rigor-1.2-line-floor";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every doc artifact meets the line floor declared by its doc_class metadata.";

/// Machine-readable pass/fail state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// Canonical documentation-rigor floor row.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct DocClassFloor {
    pub canonical_class: &'static str,
    pub min_lines: usize,
}

/// Specific substance-bar violation kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstanceViolationKind {
    BelowLineFloor,
    UnknownDocClassFloor,
}

/// One doc-class frontmatter finding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubstanceViolation {
    pub path: PathBuf,
    pub line: usize,
    pub kind: SubstanceViolationKind,
    pub doc_class: String,
    pub canonical_class: Option<String>,
    pub observed_lines: usize,
    pub required_lines: Option<usize>,
    pub message: String,
    pub suggested_fix: String,
}

/// A passing doc-class observation retained for JSON evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubstanceObservation {
    pub path: PathBuf,
    pub line: usize,
    pub doc_class: String,
    pub canonical_class: String,
    pub observed_lines: usize,
    pub required_lines: usize,
}

/// Machine-readable outcome from the enforcement entrypoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GovernanceCheckOutcome {
    pub rule_id: String,
    pub enforced_rule: String,
    pub repo_root: PathBuf,
    pub status: EnforcementStatus,
    pub scanned_markdown_files: usize,
    pub docs_with_doc_class: usize,
    pub observations: Vec<SubstanceObservation>,
    pub violations: Vec<SubstanceViolation>,
}

impl GovernanceCheckOutcome {
    pub fn is_success(&self) -> bool {
        self.status == EnforcementStatus::Passed
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrontmatterDocClass {
    value: String,
    line: usize,
}

/// Enforces the documentation-rigor substance bar for docs with frontmatter doc_class.
pub fn enforce_substance_bar(repo_root: impl AsRef<Path>) -> Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let markdown_files = discover_markdown_files(&repo_root)?;
    let mut observations = Vec::new();
    let mut violations = Vec::new();

    for file in &markdown_files {
        let content = fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        inspect_document(
            &repo_root,
            file,
            &content,
            &mut observations,
            &mut violations,
        );
    }

    let status = if violations.is_empty() {
        EnforcementStatus::Passed
    } else {
        EnforcementStatus::Failed
    };
    let docs_with_doc_class = observations.len() + violations.len();

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID.to_string(),
        enforced_rule: ENFORCED_RULE.to_string(),
        repo_root,
        status,
        scanned_markdown_files: markdown_files.len(),
        docs_with_doc_class,
        observations,
        violations,
    })
}

/// Formats a compact text report for CLI output.
pub fn format_text_report(outcome: &GovernanceCheckOutcome) -> String {
    let mut report = String::new();
    report.push_str(&format!(
        "{}: {:?} ({} markdown files, {} doc_class docs, {} violations)\n",
        outcome.rule_id,
        outcome.status,
        outcome.scanned_markdown_files,
        outcome.docs_with_doc_class,
        outcome.violation_count()
    ));

    if outcome.violations.is_empty() {
        report.push_str("OK: every doc_class document meets its line floor.\n");
        return report;
    }

    for violation in &outcome.violations {
        let required = violation
            .required_lines
            .map(|lines| lines.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        report.push_str(&format!(
            "{}:{}: {:?}: doc_class={} observed_lines={} required_lines={}\n  fix: {}\n",
            violation.path.display(),
            violation.line,
            violation.kind,
            violation.doc_class,
            violation.observed_lines,
            required,
            violation.suggested_fix
        ));
    }

    report
}

/// Returns the canonical line floors from documentation-rigor.
pub fn documentation_rigor_floors() -> BTreeMap<&'static str, DocClassFloor> {
    let rows = [
        ("adr", "ADR (decision)", 1500),
        ("decision", "ADR (decision)", 1500),
        ("adrdecision", "ADR (decision)", 1500),
        ("amendmentadr", "Amendment ADR", 1000),
        ("prd", "PRD (product requirements)", 1500),
        ("productrequirements", "PRD (product requirements)", 1500),
        ("spec", "Spec (machine-readable, JSON Schema)", 600),
        (
            "machinereadablespec",
            "Spec (machine-readable, JSON Schema)",
            600,
        ),
        ("jsonschema", "Spec (machine-readable, JSON Schema)", 600),
        ("runbook", "Runbook", 250),
        ("standard", "Standard", 250),
        ("onboarding", "Onboarding", 1000),
        ("userstories", "User stories", 2000),
        ("architecture", "Architecture deep-dive / walkthrough", 1500),
        (
            "architecturedeepdive",
            "Architecture deep-dive / walkthrough",
            1500,
        ),
        ("walkthrough", "Architecture deep-dive / walkthrough", 1500),
        ("migrationplaybook", "Migration playbook", 500),
    ];

    rows.into_iter()
        .map(|(key, canonical_class, min_lines)| {
            (
                key,
                DocClassFloor {
                    canonical_class,
                    min_lines,
                },
            )
        })
        .collect()
}

fn discover_markdown_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for base in ["docs", "microservices", "packs", "specs", "crates"] {
        let root = repo_root.join(base);
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry.with_context(|| format!("failed to walk {}", root.display()))?;
            if entry.file_type().is_file() && has_markdown_extension(entry.path()) {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    files.sort();
    Ok(files)
}

fn inspect_document(
    repo_root: &Path,
    file: &Path,
    content: &str,
    observations: &mut Vec<SubstanceObservation>,
    violations: &mut Vec<SubstanceViolation>,
) {
    let Some(frontmatter) = parse_frontmatter_doc_class(content) else {
        return;
    };

    let relative = relative_path(repo_root, file);
    let observed_lines = count_lines(content);
    let normalized_class = normalize_doc_class(&frontmatter.value);
    let floors = documentation_rigor_floors();

    match floors.get(normalized_class.as_str()) {
        Some(floor) if observed_lines >= floor.min_lines => {
            observations.push(SubstanceObservation {
                path: relative,
                line: frontmatter.line,
                doc_class: frontmatter.value,
                canonical_class: floor.canonical_class.to_string(),
                observed_lines,
                required_lines: floor.min_lines,
            });
        }
        Some(floor) => {
            violations.push(SubstanceViolation {
                path: relative,
                line: frontmatter.line,
                kind: SubstanceViolationKind::BelowLineFloor,
                doc_class: frontmatter.value,
                canonical_class: Some(floor.canonical_class.to_string()),
                observed_lines,
                required_lines: Some(floor.min_lines),
                message: "document is below the documentation-rigor line floor".to_string(),
                suggested_fix: format!(
                    "Expand this {} document to at least {} lines with the required sections and density signals from documentation-rigor §2.",
                    floor.canonical_class, floor.min_lines
                ),
            });
        }
        None => {
            violations.push(SubstanceViolation {
                path: relative,
                line: frontmatter.line,
                kind: SubstanceViolationKind::UnknownDocClassFloor,
                doc_class: frontmatter.value,
                canonical_class: None,
                observed_lines,
                required_lines: None,
                message: "doc_class does not map to a documentation-rigor line floor".to_string(),
                suggested_fix:
                    "Use a documented doc_class from documentation-rigor §2 or add the class to the rigor matrix before enforcing it.".to_string(),
            });
        }
    }
}

fn parse_frontmatter_doc_class(content: &str) -> Option<FrontmatterDocClass> {
    let mut lines = content.lines().enumerate();
    let (_, first) = lines.next()?;
    if first.trim() != "---" {
        return None;
    }

    for (index, line) in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            return None;
        }
        if let Some(value) = parse_doc_class_line(trimmed) {
            return Some(FrontmatterDocClass {
                value,
                line: index + 1,
            });
        }
    }

    None
}

fn parse_doc_class_line(line: &str) -> Option<String> {
    let (key, value) = line.split_once(':')?;
    if key.trim() != "doc_class" {
        return None;
    }

    let cleaned = value.trim().trim_matches('"').trim_matches('\'').trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn normalize_doc_class(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

fn has_markdown_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_machine_readable_spec_alias() {
        assert_eq!(
            normalize_doc_class("Machine-Readable-Spec"),
            "machinereadablespec"
        );
    }

    #[test]
    fn parses_frontmatter_doc_class_line_number() {
        let doc = "---\ntitle: Example\ndoc_class: Standard\n---\n# Body\n";
        let parsed = parse_frontmatter_doc_class(doc).expect("doc_class exists");
        assert_eq!(parsed.value, "Standard");
        assert_eq!(parsed.line, 3);
    }

    #[test]
    fn floor_map_contains_standard_floor() {
        let floors = documentation_rigor_floors();
        assert_eq!(floors["standard"].min_lines, 250);
    }
}
