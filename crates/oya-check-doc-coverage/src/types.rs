//! Shared types for the doc-coverage check.

use serde::Serialize;

/// A single missing-artifact or stale-artifact violation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ViolationKind {
    /// Required canonical artifact missing for a registered µservice (ADR-0063 §1).
    MissingCanonicalArtifact,
    /// Required pack overlay artifact missing for a (pack × µservice) pair (ADR-0063 §2).
    MissingPackOverlay,
    /// Required milestone artifact missing (ADR-0063 §3).
    MissingMilestoneArtifact,
    /// Required PRD / Phase-Spec / Impl-Plan section missing or empty (ADR-0063 §4).
    MissingSection,
    /// Doc file references a µservice that is not in workspace metadata (orphan; ADR-0063 §7).
    OrphanDoc,
    /// MASTERPLAN §2.1 catalog names a µservice that has no workspace metadata entry
    /// AND no Phase-Spec referencing it (registry/masterplan reconciliation gap).
    UnreconciledPlanned,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub violations: Vec<Violation>,
}

impl Report {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, v: Violation) {
        self.violations.push(v);
    }

    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn render_markdown(&self) -> String {
        if self.violations.is_empty() {
            return "# Doc-coverage report\n\nAll checks passed.\n".to_string();
        }
        let mut out = String::from("# Doc-coverage report\n\n");
        out.push_str(&format!("**{}** violation(s):\n\n", self.violations.len()));
        for v in &self.violations {
            out.push_str(&format!(
                "- {:?}: `{}` — {}\n",
                v.kind, v.path, v.description
            ));
        }
        out
    }
}
