//! Governance check for template-stamping detection.
//!
//! The synthesis audit P0 threshold rejects runs of three or more adjacent
//! Markdown documents in the same directory when every adjacent pair has line
//! shape Jaccard similarity above 0.70. This crate implements that check for
//! `docs/` and `microservices/` without attempting to judge prose quality; it
//! only flags repeated structural shape.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

/// Stable identifier for the governance rule enforced by this crate.
pub const RULE_ID: &str = "synthesis-audit-P0-template-stamping";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Three or more adjacent docs must not share more than 70 percent identical line shape.";

/// P0 threshold from the synthesis audit.
pub const SIMILARITY_THRESHOLD: f64 = 0.70;

const MIN_RUN_LENGTH: usize = 3;
const MIN_LINE_SHAPES: usize = 4;

/// Machine-readable pass/fail state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementStatus {
    Passed,
    Failed,
}

/// A detected template-stamped adjacent run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TemplateStampingViolation {
    pub directory: PathBuf,
    pub files: Vec<PathBuf>,
    pub pair_similarities: Vec<f64>,
    pub threshold: f64,
    pub message: String,
    pub suggested_fix: String,
}

/// Machine-readable outcome from the enforcement entrypoint.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GovernanceCheckOutcome {
    pub rule_id: String,
    pub enforced_rule: String,
    pub repo_root: PathBuf,
    pub status: EnforcementStatus,
    pub scanned_markdown_files: usize,
    pub scanned_directories: usize,
    pub violations: Vec<TemplateStampingViolation>,
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
struct DocumentShape {
    path: PathBuf,
    directory: PathBuf,
    line_shapes: BTreeSet<String>,
}

/// Enforces the no-template-stamping rule for adjacent governed documents.
pub fn enforce_no_template_stamping(repo_root: impl AsRef<Path>) -> Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let documents = discover_document_shapes(&repo_root)?;
    let grouped = group_by_directory(documents);
    let mut violations = Vec::new();

    for (directory, docs) in &grouped {
        inspect_directory_run(&repo_root, directory, docs, &mut violations);
    }

    let status = if violations.is_empty() {
        EnforcementStatus::Passed
    } else {
        EnforcementStatus::Failed
    };

    let scanned_markdown_files = grouped.values().map(Vec::len).sum();
    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID.to_string(),
        enforced_rule: ENFORCED_RULE.to_string(),
        repo_root,
        status,
        scanned_markdown_files,
        scanned_directories: grouped.len(),
        violations,
    })
}

/// Formats a compact text report for CLI output.
pub fn format_text_report(outcome: &GovernanceCheckOutcome) -> String {
    let mut report = String::new();
    report.push_str(&format!(
        "{}: {:?} ({} markdown files, {} directories, {} violations)\n",
        outcome.rule_id,
        outcome.status,
        outcome.scanned_markdown_files,
        outcome.scanned_directories,
        outcome.violation_count()
    ));

    if outcome.violations.is_empty() {
        report.push_str("OK: no adjacent template-stamped doc runs detected.\n");
        return report;
    }

    for violation in &outcome.violations {
        report.push_str(&format!(
            "{}: {} files above {:.2} line-shape Jaccard threshold\n",
            violation.directory.display(),
            violation.files.len(),
            violation.threshold
        ));
        report.push_str(&format!("  files: {}\n", display_paths(&violation.files)));
        report.push_str(&format!(
            "  pair_similarities: {}\n",
            display_similarities(&violation.pair_similarities)
        ));
        report.push_str(&format!("  fix: {}\n", violation.suggested_fix));
    }

    report
}

fn discover_document_shapes(repo_root: &Path) -> Result<Vec<DocumentShape>> {
    let mut documents = Vec::new();
    // Walks the WHOLE repository, not a hardcoded base list.
    //
    // The previous scope was `["docs", "microservices"]`. `microservices/` holds ZERO tracked
    // files -- the tier was drained -- so half the declared scope was a dead root, the exact
    // defect ci/facade/scan-root-liveness exists to catch and does not cover this crate. The
    // surviving half made the check docs-only, and template stamping is not a docs-only
    // phenomenon: 78 identical-length `hot-split.md`, 78 `cold-merge.md` and 78
    // `auto-rebalance.md` sit under `<capability>/runbooks/`, and NOT ONE of them is under
    // `docs/`. The detector was blind to every one.
    for entry in WalkDir::new(repo_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            // Prune build output and VCS internals rather than reading them: they are not authored
            // documents, and target/ alone is large enough to dominate the walk.
            !matches!(
                entry.file_name().to_str(),
                Some(".git" | "target" | "node_modules" | "buck-out" | ".jj")
            )
        })
    {
        let entry = entry.with_context(|| format!("failed to walk {}", repo_root.display()))?;
        if !entry.file_type().is_file() || !has_markdown_extension(entry.path()) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let line_shapes = compute_line_shapes(&content);
        if line_shapes.len() >= MIN_LINE_SHAPES {
            documents.push(DocumentShape {
                directory: path.parent().unwrap_or(repo_root).to_path_buf(),
                path,
                line_shapes,
            });
        }
    }

    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn group_by_directory(documents: Vec<DocumentShape>) -> BTreeMap<PathBuf, Vec<DocumentShape>> {
    let mut grouped: BTreeMap<PathBuf, Vec<DocumentShape>> = BTreeMap::new();
    for document in documents {
        grouped
            .entry(document.directory.clone())
            .or_default()
            .push(document);
    }
    for docs in grouped.values_mut() {
        docs.sort_by(|left, right| left.path.cmp(&right.path));
    }
    grouped
}

fn inspect_directory_run(
    repo_root: &Path,
    directory: &Path,
    docs: &[DocumentShape],
    violations: &mut Vec<TemplateStampingViolation>,
) {
    if docs.len() < MIN_RUN_LENGTH {
        return;
    }

    let mut index = 0;
    while index + 1 < docs.len() {
        let mut run_files = vec![docs[index].path.clone()];
        let mut pair_similarities = Vec::new();
        let mut cursor = index;

        while cursor + 1 < docs.len() {
            let similarity = jaccard(&docs[cursor].line_shapes, &docs[cursor + 1].line_shapes);
            if similarity > SIMILARITY_THRESHOLD {
                pair_similarities.push(round_similarity(similarity));
                run_files.push(docs[cursor + 1].path.clone());
                cursor += 1;
            } else {
                break;
            }
        }

        if run_files.len() >= MIN_RUN_LENGTH {
            let relative_files = run_files
                .iter()
                .map(|path| relative_path(repo_root, path))
                .collect::<Vec<_>>();
            violations.push(TemplateStampingViolation {
                directory: relative_path(repo_root, directory),
                files: relative_files,
                pair_similarities,
                threshold: SIMILARITY_THRESHOLD,
                message: "three or more adjacent docs exceed the synthesis-audit P0 line-shape similarity threshold".to_string(),
                suggested_fix:
                    "Collapse duplicated template prose into a shared standard or rewrite each doc with artifact-specific structure, evidence, and sections.".to_string(),
            });
            index = cursor + 1;
        } else {
            index += 1;
        }
    }
}

fn compute_line_shapes(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .filter_map(line_shape)
        .collect::<BTreeSet<_>>()
}

fn line_shape(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut shape = String::new();
    let mut last_was_space = false;
    let mut last_shape_char = '\0';

    for ch in trimmed.chars() {
        let mapped = match ch {
            'a'..='z' | 'A'..='Z' => 'a',
            '0'..='9' => '0',
            ch if ch.is_whitespace() => ' ',
            _ => ch,
        };

        if mapped == ' ' {
            if !last_was_space {
                shape.push(' ');
            }
            last_was_space = true;
            last_shape_char = mapped;
            continue;
        }

        last_was_space = false;
        if mapped == 'a' && last_shape_char == 'a' {
            continue;
        }
        if mapped == '0' && last_shape_char == '0' {
            continue;
        }
        shape.push(mapped);
        last_shape_char = mapped;
    }

    let compact = shape.trim().to_string();
    if compact.is_empty() {
        None
    } else {
        Some(compact)
    }
}

fn jaccard(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }

    let intersection = left.intersection(right).count() as f64;
    let union = left.union(right).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn round_similarity(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn has_markdown_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_similarities(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_shape_removes_words_but_keeps_structure() {
        assert_eq!(
            line_shape("### D-10. Provider BYOK contract").as_deref(),
            Some("### a-0. a a a")
        );
    }

    #[test]
    fn identical_shape_sets_have_full_similarity() {
        let left = compute_line_shapes("# A\n- first 123\n- second 456\n");
        let right = compute_line_shapes("# B\n- alpha 999\n- beta 111\n");
        assert_eq!(jaccard(&left, &right), 1.0);
    }

    #[test]
    fn different_shape_sets_have_low_similarity() {
        let left = compute_line_shapes("# A\n- one\n- two\n");
        let right = compute_line_shapes("paragraph\n```rust\nfn main() {}\n```\n");
        assert!(jaccard(&left, &right) < 0.7);
    }
}
