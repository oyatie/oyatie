//! ADR-0531 `oya-bot-autofix` skeleton.
//!
//! This crate is intentionally narrow: it can render a dry-run preview for
//! described `oya-ci-gate-contract` remediation data, and its policy surface is
//! PROPOSE-only. It does not contain merge or gate-bypass capabilities.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Component, Path};

use oya_ci_gate_contract::{Edit, NewFile, Remediation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    DryRun,
    ProposeReviewablePullRequest,
    MergePullRequest,
    BypassGates,
}

impl Action {
    fn denial_reason(self) -> Option<&'static str> {
        match self {
            Self::DryRun | Self::ProposeReviewablePullRequest => None,
            Self::MergePullRequest => {
                Some("oya-bot-autofix is PROPOSE-only and cannot merge pull requests")
            }
            Self::BypassGates => {
                Some("oya-bot-autofix cannot bypass oya-ci-required or any adopter gate fan-in")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotPolicy;

impl BotPolicy {
    pub fn propose_only() -> Self {
        Self
    }

    pub fn authorize(&self, action: Action) -> Result<(), AutofixError> {
        let _ = self;
        if let Some(reason) = action.denial_reason() {
            Err(AutofixError::ForbiddenAction { action, reason })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    DryRun,
    ProposeReviewablePullRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyEnvelope {
    pub mode: DeliveryMode,
    pub writes_performed: bool,
    pub can_merge: bool,
    pub can_bypass_gates: bool,
}

impl SafetyEnvelope {
    pub fn dry_run() -> Self {
        Self {
            mode: DeliveryMode::DryRun,
            writes_performed: false,
            can_merge: false,
            can_bypass_gates: false,
        }
    }

    pub fn propose_reviewable_pull_request() -> Self {
        Self {
            mode: DeliveryMode::ProposeReviewablePullRequest,
            writes_performed: true,
            can_merge: false,
            can_bypass_gates: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DryRunInput<'a> {
    pub remediation: &'a Remediation,
    pub original_text: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub diff: String,
    pub safety: SafetyEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutofixError {
    ForbiddenAction {
        action: Action,
        reason: &'static str,
    },
    NoRemediation,
    ByteRangeOutOfBounds {
        path: String,
        start: usize,
        end: usize,
        len: usize,
    },
    ByteRangeNotUtf8Boundary {
        path: String,
        start: usize,
        end: usize,
    },
    InvalidPath {
        path: String,
        reason: &'static str,
    },
}

impl fmt::Display for AutofixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenAction { reason, .. } => write!(f, "{reason}"),
            Self::NoRemediation => write!(f, "no remediation was available to render"),
            Self::ByteRangeOutOfBounds {
                path,
                start,
                end,
                len,
            } => write!(
                f,
                "remediation byte range {start}..{end} is outside {path} length {len}"
            ),
            Self::ByteRangeNotUtf8Boundary { path, start, end } => write!(
                f,
                "remediation byte range {start}..{end} does not align to UTF-8 boundaries in {path}"
            ),
            Self::InvalidPath { path, reason } => {
                write!(f, "remediation path {path:?} is not reviewable: {reason}")
            }
        }
    }
}

impl Error for AutofixError {}

pub fn render_dry_run(input: DryRunInput<'_>) -> Result<DryRunReport, AutofixError> {
    BotPolicy::propose_only().authorize(Action::DryRun)?;

    let diff = match input.remediation {
        Remediation::AutoFix(edit) => render_edit_diff(edit, input.original_text)?,
        Remediation::AutoGenerate(new_file) => render_new_file_diff(new_file)?,
        Remediation::None => return Err(AutofixError::NoRemediation),
    };

    Ok(DryRunReport {
        diff,
        safety: SafetyEnvelope::dry_run(),
    })
}

fn render_edit_diff(edit: &Edit, original_text: &str) -> Result<String, AutofixError> {
    validate_reviewable_repo_path(&edit.path)?;

    let len = original_text.len();
    let start = edit.byte_range.start;
    let end = edit.byte_range.end;
    if start > end || end > len {
        return Err(AutofixError::ByteRangeOutOfBounds {
            path: edit.path.clone(),
            start,
            end,
            len,
        });
    }
    if !original_text.is_char_boundary(start) || !original_text.is_char_boundary(end) {
        return Err(AutofixError::ByteRangeNotUtf8Boundary {
            path: edit.path.clone(),
            start,
            end,
        });
    }

    let mut updated = String::with_capacity(original_text.len() + edit.replacement.len());
    updated.push_str(&original_text[..start]);
    updated.push_str(&edit.replacement);
    updated.push_str(&original_text[end..]);

    Ok(render_scoped_unified_diff(
        &edit.path,
        original_text,
        &updated,
    ))
}

fn render_new_file_diff(new_file: &NewFile) -> Result<String, AutofixError> {
    validate_reviewable_repo_path(&new_file.path)?;

    let mut diff = String::new();
    diff.push_str(&format!(
        "diff --git a/{path} b/{path}\n",
        path = new_file.path
    ));
    // `git apply` only treats a `--- /dev/null` header as file-creation when a
    // `new file mode` line marks the section as such; without it, a patch
    // that "looks right" is still rejected (verified with `git apply
    // --check`, see the `dry_run_new_file_diff_is_git_apply_clean_*` tests,
    // which pipe the rendered diff through a real `git apply --check`).
    diff.push_str("new file mode 100644\n");
    let lines = split_lines_preserving_endings(&new_file.body);
    if !lines.is_empty() {
        diff.push_str("--- /dev/null\n");
        diff.push_str(&format!("+++ b/{path}\n", path = new_file.path));
        diff.push_str(&format!(
            "@@ -0,0 +{},{} @@\n",
            hunk_start(0, lines.len()),
            lines.len()
        ));
        for line in lines {
            push_diff_line(&mut diff, '+', line);
        }
    }
    // An empty new file has no lines to hunk over, so `---`/`+++`/`@@` are
    // omitted entirely — matching what `git diff` itself renders for a
    // newly added empty file.
    Ok(diff)
}

fn render_scoped_unified_diff(path: &str, original: &str, updated: &str) -> String {
    const CONTEXT_LINES: usize = 3;

    let original_lines = split_lines_preserving_endings(original);
    let updated_lines = split_lines_preserving_endings(updated);
    let prefix_len = common_prefix_len(&original_lines, &updated_lines);
    let suffix_len = common_suffix_len(&original_lines, &updated_lines, prefix_len);
    let original_changed_end = original_lines.len() - suffix_len;
    let updated_changed_end = updated_lines.len() - suffix_len;
    let context_start = prefix_len.saturating_sub(CONTEXT_LINES);
    let suffix_context_len = CONTEXT_LINES.min(suffix_len);
    let original_context_end =
        (original_changed_end + suffix_context_len).min(original_lines.len());
    let updated_context_end = (updated_changed_end + suffix_context_len).min(updated_lines.len());
    let original_count = original_context_end - context_start;
    let updated_count = updated_context_end - context_start;

    let mut diff = String::new();
    diff.push_str(&format!("diff --git a/{path} b/{path}\n"));
    diff.push_str(&format!("--- a/{path}\n"));
    diff.push_str(&format!("+++ b/{path}\n"));
    diff.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        hunk_start(context_start, original_count),
        original_count,
        hunk_start(context_start, updated_count),
        updated_count
    ));
    for line in &original_lines[context_start..prefix_len] {
        push_diff_line(&mut diff, ' ', line);
    }
    for line in &original_lines[prefix_len..original_changed_end] {
        push_diff_line(&mut diff, '-', line);
    }
    for line in &updated_lines[prefix_len..updated_changed_end] {
        push_diff_line(&mut diff, '+', line);
    }
    for line in &original_lines[original_changed_end..original_context_end] {
        push_diff_line(&mut diff, ' ', line);
    }
    diff
}

fn split_lines_preserving_endings(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

fn common_prefix_len<'a>(left: &[&'a str], right: &[&'a str]) -> usize {
    left.iter()
        .zip(right.iter())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_suffix_len<'a>(left: &[&'a str], right: &[&'a str], prefix_len: usize) -> usize {
    left[prefix_len..]
        .iter()
        .rev()
        .zip(right[prefix_len..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn hunk_start(zero_based_start: usize, count: usize) -> usize {
    if count == 0 {
        zero_based_start
    } else {
        zero_based_start + 1
    }
}

fn push_diff_line(diff: &mut String, marker: char, line: &str) {
    diff.push(marker);
    diff.push_str(line);
    if !line.ends_with('\n') {
        diff.push('\n');
        diff.push_str("\\ No newline at end of file\n");
    }
}

fn validate_reviewable_repo_path(path: &str) -> Result<(), AutofixError> {
    if path.is_empty() {
        return Err(AutofixError::InvalidPath {
            path: path.to_owned(),
            reason: "path is empty",
        });
    }
    if path.chars().any(|character| {
        matches!(
            character,
            '\n' | '\r' | '\0'
                // Trojan-Source bidi overrides/embeddings/isolates (CVE-2021-42574 class):
                // these can reorder how the rendered diff *displays* without changing its
                // bytes, so a "reviewable" diff must reject them outright.
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
                // directional marks
                | '\u{200E}' | '\u{200F}'
                // zero-width characters + BOM
                | '\u{200B}'..='\u{200D}'
                | '\u{FEFF}'
        )
    }) {
        return Err(AutofixError::InvalidPath {
            path: path.to_owned(),
            reason: "path contains a control character",
        });
    }

    let mut has_normal_component = false;
    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir => {
                return Err(AutofixError::InvalidPath {
                    path: path.to_owned(),
                    reason: "path must be normalized without '.' components",
                });
            }
            Component::ParentDir => {
                return Err(AutofixError::InvalidPath {
                    path: path.to_owned(),
                    reason: "path must stay within the repository",
                });
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AutofixError::InvalidPath {
                    path: path.to_owned(),
                    reason: "path must be repository-relative",
                });
            }
        }
    }

    if !has_normal_component {
        return Err(AutofixError::InvalidPath {
            path: path.to_owned(),
            reason: "path does not name a file",
        });
    }

    Ok(())
}
