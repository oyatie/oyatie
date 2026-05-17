//! Purpose-discipline fitness kernel (M01-P10-IP-001).
//!
//! Validates that every artifact (Markdown or JSON) has a declared purpose.
//! Markdown files must have `purpose:` in YAML frontmatter.
//! JSON files must have `_meta.purpose` or a top-level `purpose` field.
//!
//! Hardened 2026-05-15: placeholder strings (`Auto-backfilled`, `TODO`,
//! `PLACEHOLDER`, `FIXME`, `XXX`, `tbd`, …) are treated as MISSING, not
//! present. The 90cc8af commit had stamped 745 files with
//! `purpose: Auto-backfilled purpose for <basename>` to satisfy a prior
//! `check()` that accepted any non-empty string — this kernel now refuses
//! that pattern at the source so the loop cannot recur.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeNode {
    pub path: String,            // data_class: INTERNAL_ONLY
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeViolation {
    pub path: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: PurposeViolationKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PurposeViolationKind {
    Missing,
    Placeholder,
}

impl fmt::Display for PurposeViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "missing purpose"),
            Self::Placeholder => write!(f, "placeholder purpose"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeReport {
    // data_class: INTERNAL_ONLY
    pub nodes_checked: usize, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub violations: Vec<PurposeViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PurposeError {
    EmptyPath,
    DuplicatePath { path: String },
}

impl fmt::Display for PurposeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "empty path in purpose node"),
            Self::DuplicatePath { path } => write!(f, "duplicate purpose node path: {}", path),
        }
    }
}

impl std::error::Error for PurposeError {}

/// Strings that, when found in a `purpose:` value, mark it as a
/// placeholder rather than real content. Case-insensitive contains-match.
///
/// The list is closed and hardcoded — extending it is a 1-ADR action,
/// per `decision-principles.json` DP-06 (Bounded scope per doc).
pub const PLACEHOLDER_NEEDLES: &[&str] = &[
    "auto-backfilled",
    "todo",
    "placeholder",
    "fixme",
    "xxx",
    "tbd",
    "to be determined",
    "to do",
];

pub fn is_placeholder(purpose: &str) -> bool {
    let trimmed = purpose.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    PLACEHOLDER_NEEDLES
        .iter()
        .any(|needle| lower.contains(needle))
}

pub fn check(nodes: &[PurposeNode]) -> Result<PurposeReport, PurposeError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut violations = Vec::new();

    for node in nodes {
        if node.path.is_empty() {
            return Err(PurposeError::EmptyPath);
        }
        if !seen.insert(node.path.clone()) {
            return Err(PurposeError::DuplicatePath {
                path: node.path.clone(),
            });
        }

        match node.purpose.as_deref() {
            None => violations.push(PurposeViolation {
                path: node.path.clone(),
                kind: PurposeViolationKind::Missing,
            }),
            Some(p) if p.trim().is_empty() => violations.push(PurposeViolation {
                path: node.path.clone(),
                kind: PurposeViolationKind::Missing,
            }),
            Some(p) if is_placeholder(p) => violations.push(PurposeViolation {
                path: node.path.clone(),
                kind: PurposeViolationKind::Placeholder,
            }),
            Some(_) => {}
        }
    }

    Ok(PurposeReport {
        nodes_checked: nodes.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_formed_passes() {
        let r = check(&[PurposeNode {
            path: "docs/PRD.md".into(),
            purpose: Some("Define product".into()),
        }])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_purpose_flagged() {
        let r = check(&[PurposeNode {
            path: "docs/PRD.md".into(),
            purpose: None,
        }])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Missing);
    }

    #[test]
    fn empty_string_purpose_flagged_as_missing() {
        let r = check(&[PurposeNode {
            path: "x.md".into(),
            purpose: Some("   ".into()),
        }])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Missing);
    }

    #[test]
    fn auto_backfilled_purpose_flagged_as_placeholder() {
        // Regression for the 90cc8af stub-stamping pattern.
        let r = check(&[PurposeNode {
            path: "docs/anything.md".into(),
            purpose: Some("Auto-backfilled purpose for anything.md".into()),
        }])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Placeholder);
    }

    #[test]
    fn todo_purpose_flagged_as_placeholder() {
        let r = check(&[PurposeNode {
            path: "x.md".into(),
            purpose: Some("TODO: write me later".into()),
        }])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Placeholder);
    }

    #[test]
    fn placeholder_case_insensitive() {
        let r = check(&[PurposeNode {
            path: "x.md".into(),
            purpose: Some("PLACEHOLDER — author this".into()),
        }])
        .unwrap();
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Placeholder);
    }

    #[test]
    fn tbd_flagged() {
        let r = check(&[PurposeNode {
            path: "x.md".into(),
            purpose: Some("tbd, pending decision".into()),
        }])
        .unwrap();
        assert_eq!(r.violations[0].kind, PurposeViolationKind::Placeholder);
    }

    #[test]
    fn substantive_purpose_passes() {
        let r = check(&[PurposeNode {
            path: "docs/PRD.md".into(),
            purpose: Some("Define the unified B2B shell scope per ADR-0061".into()),
        }])
        .unwrap();
        assert!(r.violations.is_empty());
    }
}
