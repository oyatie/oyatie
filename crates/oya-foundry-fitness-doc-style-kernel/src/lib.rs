//! Doc-style fitness kernel — enforces `docs/standards/doc-style.md`.
//!
//! I/O-free. Runners read each Markdown file line-by-line and feed
//! typed [`DocLine`] records into [`check_style`]. The kernel returns
//! violations classified by rule.
//!
//! Rules enforced today:
//! - No trailing whitespace.
//! - No tabs (use spaces).
//! - One H1 per document.
//! - H1 must be the first non-empty content line.
//! - Lines do not exceed `MAX_LINE_WIDTH` columns (excluding fenced code).

pub const MAX_LINE_WIDTH: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocLine {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: u32,       // data_class: INTERNAL_ONLY (1-based)
    pub content: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocStyleViolationKind {
    TrailingWhitespace,
    TabIndentation,
    MissingH1,
    MultipleH1,
    H1NotFirstContentLine,
    LineTooLong { actual: usize, max: usize },
}

impl DocStyleViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TrailingWhitespace => "trailing whitespace",
            Self::TabIndentation => "tab indentation",
            Self::MissingH1 => "missing H1 heading",
            Self::MultipleH1 => "more than one H1 heading",
            Self::H1NotFirstContentLine => "H1 is not the first non-empty content line",
            Self::LineTooLong { .. } => "line exceeds max width",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocStyleViolation {
    pub path: String,                // data_class: INTERNAL_ONLY
    pub line: u32,                   // data_class: INTERNAL_ONLY
    pub kind: DocStyleViolationKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocStyleReport {
    pub files_checked: usize,               // data_class: INTERNAL_ONLY
    pub violations: Vec<DocStyleViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocStyleError {
    EmptyPath,
}

impl DocStyleError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPath => "empty path in doc line".to_owned(),
        }
    }
}

/// Check each document's lines against the doc-style rules.
///
/// `lines` may interleave multiple files; we group by `path` and run
/// per-document checks (H1 count, H1 position) on each group.
pub fn check_style(lines: &[DocLine]) -> Result<DocStyleReport, DocStyleError> {
    use std::collections::BTreeMap;

    if lines.iter().any(|l| l.path.is_empty()) {
        return Err(DocStyleError::EmptyPath);
    }

    let mut by_path: BTreeMap<&str, Vec<&DocLine>> = BTreeMap::new();
    for l in lines {
        by_path.entry(l.path.as_str()).or_default().push(l);
    }

    let mut violations = Vec::new();

    for (path, doc_lines) in &by_path {
        // Per-line checks.
        let mut in_fence = false;
        let mut h1_count = 0usize;
        let mut first_h1_line: Option<u32> = None;
        let mut first_content_line: Option<u32> = None;

        for l in doc_lines {
            let trimmed = l.content.trim_end_matches('\n');

            // Fence toggling first; never flag width inside fences.
            if trimmed.trim_start().starts_with("```") {
                in_fence = !in_fence;
            }

            if trimmed.ends_with(' ') || trimmed.ends_with('\t') {
                violations.push(DocStyleViolation {
                    path: (*path).to_owned(),
                    line: l.line,
                    kind: DocStyleViolationKind::TrailingWhitespace,
                });
            }
            if trimmed.starts_with('\t') {
                violations.push(DocStyleViolation {
                    path: (*path).to_owned(),
                    line: l.line,
                    kind: DocStyleViolationKind::TabIndentation,
                });
            }
            if !in_fence && trimmed.chars().count() > MAX_LINE_WIDTH {
                violations.push(DocStyleViolation {
                    path: (*path).to_owned(),
                    line: l.line,
                    kind: DocStyleViolationKind::LineTooLong {
                        actual: trimmed.chars().count(),
                        max: MAX_LINE_WIDTH,
                    },
                });
            }

            if !trimmed.is_empty() && first_content_line.is_none() {
                first_content_line = Some(l.line);
            }
            // H1 detection (ATX style, single `#` followed by space).
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                h1_count += 1;
                if first_h1_line.is_none() {
                    first_h1_line = Some(l.line);
                }
            }
        }

        // Per-document checks.
        match (h1_count, first_h1_line, first_content_line) {
            (0, _, _) => violations.push(DocStyleViolation {
                path: (*path).to_owned(),
                line: 0,
                kind: DocStyleViolationKind::MissingH1,
            }),
            (n, Some(h1), Some(first)) if n > 1 || h1 != first => {
                if n > 1 {
                    violations.push(DocStyleViolation {
                        path: (*path).to_owned(),
                        line: h1,
                        kind: DocStyleViolationKind::MultipleH1,
                    });
                }
                if h1 != first {
                    violations.push(DocStyleViolation {
                        path: (*path).to_owned(),
                        line: h1,
                        kind: DocStyleViolationKind::H1NotFirstContentLine,
                    });
                }
            }
            _ => {}
        }
    }

    violations.sort_by(|a, b| {
        (a.path.as_str(), a.line, a.kind.as_str()).cmp(&(b.path.as_str(), b.line, b.kind.as_str()))
    });

    Ok(DocStyleReport {
        files_checked: by_path.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(path: &str, n: u32, c: &str) -> DocLine {
        DocLine {
            path: path.into(),
            line: n,
            content: c.into(),
        }
    }

    fn doc(path: &str, lines: &[&str]) -> Vec<DocLine> {
        lines
            .iter()
            .enumerate()
            .map(|(i, c)| line(path, (i + 1) as u32, c))
            .collect()
    }

    #[test]
    fn well_formed_doc_passes() {
        let r = check_style(&doc("a.md", &["# Title", "", "Body."])).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn missing_h1_flagged() {
        let r = check_style(&doc("a.md", &["Body only."])).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == DocStyleViolationKind::MissingH1)
        );
    }

    #[test]
    fn multiple_h1_flagged() {
        let r = check_style(&doc("a.md", &["# One", "# Two"])).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == DocStyleViolationKind::MultipleH1)
        );
    }

    #[test]
    fn h1_not_first_content_line_flagged() {
        let r = check_style(&doc("a.md", &["Body first", "# Title"])).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == DocStyleViolationKind::H1NotFirstContentLine)
        );
    }

    #[test]
    fn trailing_whitespace_flagged() {
        let r = check_style(&doc("a.md", &["# Title", "trailing   "])).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == DocStyleViolationKind::TrailingWhitespace)
        );
    }

    #[test]
    fn tab_indentation_flagged() {
        let r = check_style(&doc("a.md", &["# Title", "\ttab"])).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == DocStyleViolationKind::TabIndentation)
        );
    }

    #[test]
    fn long_line_flagged() {
        let long = "x".repeat(MAX_LINE_WIDTH + 1);
        let mut lines = doc("a.md", &["# Title"]);
        lines.push(line("a.md", 2, &long));
        let r = check_style(&lines).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| matches!(v.kind, DocStyleViolationKind::LineTooLong { .. }))
        );
    }

    #[test]
    fn long_line_inside_fenced_code_block_not_flagged() {
        let long = "x".repeat(MAX_LINE_WIDTH + 50);
        let lines = vec![
            line("a.md", 1, "# Title"),
            line("a.md", 2, "```"),
            line("a.md", 3, &long),
            line("a.md", 4, "```"),
        ];
        let r = check_style(&lines).unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| matches!(v.kind, DocStyleViolationKind::LineTooLong { .. }))
        );
    }

    #[test]
    fn h2_not_counted_as_h1() {
        let r = check_style(&doc("a.md", &["# Title", "## Sub"])).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn empty_path_errors() {
        let err = check_style(&[line("", 1, "# Title")]).unwrap_err();
        assert!(matches!(err, DocStyleError::EmptyPath));
    }

    #[test]
    fn multiple_docs_aggregated() {
        let mut all = doc("a.md", &["# A", "ok"]);
        all.extend(doc("b.md", &["body without h1"]));
        let r = check_style(&all).unwrap();
        assert_eq!(r.files_checked, 2);
        assert!(r.violations.iter().any(|v| v.path == "b.md"));
    }

    #[test]
    fn violations_sorted_by_path_then_line() {
        let r = check_style(&doc("a.md", &["# One", "# Two"])).unwrap();
        // Both violations on line 2 (the duplicate H1) — they should be
        // sorted deterministically by `(path, line, kind str)`.
        assert_eq!(r.violations[0].path, "a.md");
    }
}
