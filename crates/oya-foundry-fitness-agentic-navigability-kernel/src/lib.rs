//! Agentic-navigability fitness kernel — blocks plan/spec trees with
//! missing INDEX.md, missing `parent:` pointer, undeclared `symbols-touched`,
//! or undeclared `purpose`. Per M01-P11 directive: every artifact agents
//! need to read must be reachable from the root hub in two hops.
//!
//! I/O-free. Runners walk the tree and feed typed [`PlanNode`] records
//! into [`check`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// A single planning artifact (Milestone INDEX, Phase INDEX, IP file, …).
/// Path is repo-relative for stable error messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanNode {
    pub path: String,                   // data_class: INTERNAL_ONLY
    pub kind: PlanNodeKind,             // data_class: INTERNAL_ONLY
    pub parent_pointer: Option<String>, // data_class: INTERNAL_ONLY
    pub purpose: Option<String>,        // data_class: INTERNAL_ONLY
    pub symbols_touched: Vec<String>,   // data_class: INTERNAL_ONLY
    pub has_index_md_sibling: bool,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlanNodeKind {
    MilestoneIndex,
    PhaseIndex,
    ImplementationPlan,
}

impl PlanNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MilestoneIndex => "MilestoneIndex",
            Self::PhaseIndex => "PhaseIndex",
            Self::ImplementationPlan => "ImplementationPlan",
        }
    }

    pub fn requires_parent_pointer(self) -> bool {
        matches!(self, Self::PhaseIndex | Self::ImplementationPlan)
    }

    pub fn requires_symbols_touched(self) -> bool {
        matches!(self, Self::ImplementationPlan)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NavigabilityViolationKind {
    MissingParentPointer,
    MissingPurpose,
    MissingSymbolsTouched,
    PhaseWithoutSiblingIndex,
}

impl NavigabilityViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingParentPointer => "missing parent pointer",
            Self::MissingPurpose => "missing purpose",
            Self::MissingSymbolsTouched => "missing symbols-touched list",
            Self::PhaseWithoutSiblingIndex => "phase tree missing INDEX.md sibling",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigabilityViolation {
    pub path: String,                    // data_class: INTERNAL_ONLY
    pub kind: NavigabilityViolationKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigabilityReport {
    pub nodes_checked: usize,                   // data_class: INTERNAL_ONLY
    pub violations: Vec<NavigabilityViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NavigabilityError {
    EmptyPath,
    DuplicatePath { path: String },
}

impl NavigabilityError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPath => "empty path in plan node".to_owned(),
            Self::DuplicatePath { path } => format!("duplicate plan node path: {path}"),
        }
    }
}

/// Check the supplied plan tree for navigability violations.
pub fn check(nodes: &[PlanNode]) -> Result<NavigabilityReport, NavigabilityError> {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut violations = Vec::new();

    for node in nodes {
        if node.path.is_empty() {
            return Err(NavigabilityError::EmptyPath);
        }
        if !seen.insert(node.path.as_str()) {
            return Err(NavigabilityError::DuplicatePath {
                path: node.path.clone(),
            });
        }

        if node.kind.requires_parent_pointer()
            && node.parent_pointer.as_deref().is_none_or(str::is_empty)
        {
            violations.push(NavigabilityViolation {
                path: node.path.clone(),
                kind: NavigabilityViolationKind::MissingParentPointer,
            });
        }

        if node.purpose.as_deref().is_none_or(str::is_empty) {
            violations.push(NavigabilityViolation {
                path: node.path.clone(),
                kind: NavigabilityViolationKind::MissingPurpose,
            });
        }

        if node.kind.requires_symbols_touched() && node.symbols_touched.is_empty() {
            violations.push(NavigabilityViolation {
                path: node.path.clone(),
                kind: NavigabilityViolationKind::MissingSymbolsTouched,
            });
        }

        if matches!(node.kind, PlanNodeKind::PhaseIndex) && !node.has_index_md_sibling {
            violations.push(NavigabilityViolation {
                path: node.path.clone(),
                kind: NavigabilityViolationKind::PhaseWithoutSiblingIndex,
            });
        }
    }

    Ok(NavigabilityReport {
        nodes_checked: nodes.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_formed_ip() -> PlanNode {
        PlanNode {
            path: "milestones/M02/phases/P00/IP-001.md".into(),
            kind: PlanNodeKind::ImplementationPlan,
            parent_pointer: Some("./INDEX.md".into()),
            purpose: Some("Ship X".into()),
            symbols_touched: vec!["crates/a/src/lib.rs::Foo".into()],
            has_index_md_sibling: true,
        }
    }

    fn well_formed_phase() -> PlanNode {
        PlanNode {
            path: "milestones/M02/phases/P00/INDEX.md".into(),
            kind: PlanNodeKind::PhaseIndex,
            parent_pointer: Some("../../INDEX.md".into()),
            purpose: Some("Phase purpose".into()),
            symbols_touched: vec![],
            has_index_md_sibling: true,
        }
    }

    fn well_formed_milestone() -> PlanNode {
        PlanNode {
            path: "milestones/M02/INDEX.md".into(),
            kind: PlanNodeKind::MilestoneIndex,
            parent_pointer: None,
            purpose: Some("Milestone purpose".into()),
            symbols_touched: vec![],
            has_index_md_sibling: true,
        }
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = check(&[]).unwrap();
        assert_eq!(r.nodes_checked, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn well_formed_tree_passes() {
        let r = check(&[
            well_formed_milestone(),
            well_formed_phase(),
            well_formed_ip(),
        ])
        .unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert_eq!(r.nodes_checked, 3);
    }

    #[test]
    fn ip_missing_parent_pointer_flagged() {
        let mut n = well_formed_ip();
        n.parent_pointer = None;
        let r = check(&[n]).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(
            r.violations[0].kind,
            NavigabilityViolationKind::MissingParentPointer
        );
    }

    #[test]
    fn ip_missing_purpose_flagged() {
        let mut n = well_formed_ip();
        n.purpose = None;
        let r = check(&[n]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NavigabilityViolationKind::MissingPurpose)
        );
    }

    #[test]
    fn ip_missing_symbols_flagged() {
        let mut n = well_formed_ip();
        n.symbols_touched.clear();
        let r = check(&[n]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NavigabilityViolationKind::MissingSymbolsTouched)
        );
    }

    #[test]
    fn phase_without_sibling_index_flagged() {
        let mut n = well_formed_phase();
        n.has_index_md_sibling = false;
        let r = check(&[n]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NavigabilityViolationKind::PhaseWithoutSiblingIndex)
        );
    }

    #[test]
    fn milestone_does_not_require_parent_pointer() {
        let mut n = well_formed_milestone();
        n.parent_pointer = None;
        let r = check(&[n]).unwrap();
        // Only MissingPurpose-class violations would fire; this milestone has a purpose.
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn milestone_missing_purpose_flagged() {
        let mut n = well_formed_milestone();
        n.purpose = None;
        let r = check(&[n]).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(
            r.violations[0].kind,
            NavigabilityViolationKind::MissingPurpose
        );
    }

    #[test]
    fn empty_purpose_treated_as_missing() {
        let mut n = well_formed_ip();
        n.purpose = Some(String::new());
        let r = check(&[n]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NavigabilityViolationKind::MissingPurpose)
        );
    }

    #[test]
    fn empty_parent_pointer_treated_as_missing() {
        let mut n = well_formed_ip();
        n.parent_pointer = Some(String::new());
        let r = check(&[n]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NavigabilityViolationKind::MissingParentPointer)
        );
    }

    #[test]
    fn empty_path_errors() {
        let mut n = well_formed_ip();
        n.path = String::new();
        let err = check(&[n]).unwrap_err();
        assert!(matches!(err, NavigabilityError::EmptyPath));
    }

    #[test]
    fn duplicate_path_errors() {
        let n = well_formed_ip();
        let err = check(&[n.clone(), n]).unwrap_err();
        assert!(matches!(err, NavigabilityError::DuplicatePath { .. }));
    }

    #[test]
    fn plan_node_kind_as_str_distinct() {
        let kinds = [
            PlanNodeKind::MilestoneIndex,
            PlanNodeKind::PhaseIndex,
            PlanNodeKind::ImplementationPlan,
        ];
        let names: std::collections::HashSet<_> = kinds.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), kinds.len());
    }

    #[test]
    fn violation_kind_as_str_distinct() {
        let kinds = [
            NavigabilityViolationKind::MissingParentPointer,
            NavigabilityViolationKind::MissingPurpose,
            NavigabilityViolationKind::MissingSymbolsTouched,
            NavigabilityViolationKind::PhaseWithoutSiblingIndex,
        ];
        let names: std::collections::HashSet<_> = kinds.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), kinds.len());
    }
}
