//! Purpose-discipline fitness kernel (M-CC-P03-IP-001).
//!
//! Validates that every artifact (Markdown or JSON) has a declared purpose.
//! Markdown files must have `purpose:` in YAML frontmatter.
//! JSON files must have `_meta.purpose` or a top-level `purpose` field.

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeNode {
    pub path: String,            // data_class: INTERNAL_ONLY
    pub purpose: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeViolation {
    pub path: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeReport {
    pub nodes_checked: usize,
    pub violations: Vec<PurposeViolation>,
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

        if node.purpose.as_deref().is_none_or(str::is_empty) {
            violations.push(PurposeViolation {
                path: node.path.clone(),
            });
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
    }
}
