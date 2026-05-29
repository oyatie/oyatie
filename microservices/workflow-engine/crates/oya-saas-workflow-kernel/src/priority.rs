//! Workflow run priority — scheduling tier for the preview engine.
//!
//! [`WorkflowPriority`] is a four-level enum that the SaaS workflow engine
//! uses to order competing run-starts in the preview scheduling lane.
//! Higher-priority runs are admitted before lower-priority ones when the
//! ledger is under concurrency pressure.
//!
//! Kept in a sub-module so it can be evolved independently without touching
//! the top-level identity/contract surface of the kernel.

/// Scheduling priority for a workflow run in the preview engine.
///
/// Variants are ordered lowest-to-highest so that `Ord` comparisons work
/// intuitively: `Low < Normal < High < Critical`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorkflowPriority {
    /// Background / batch work — scheduled when capacity is available.
    Low,
    /// Standard interactive run — default for all tenant-initiated runs.
    #[default]
    Normal,
    /// Elevated run — tenant-flagged SLA-sensitive workflow.
    High,
    /// Incident-class run — bypasses normal queue ordering per ADR-0023.
    Critical,
}

impl WorkflowPriority {
    /// Returns the canonical string label used in REST API payloads and audit
    /// events.  Labels are lowercase to match the kebab-case envelope used by
    /// the rest of the SaaS preview tier.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Parse a priority from a label produced by [`WorkflowPriority::label`].
    /// Returns `None` for any unrecognised string so callers can surface a
    /// typed validation error.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }

    /// Returns `true` when this priority level requires immediate queue
    /// admission, bypassing normal FIFO ordering.
    pub fn is_expedited(self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_ordering_is_low_to_critical() {
        assert!(WorkflowPriority::Low < WorkflowPriority::Normal);
        assert!(WorkflowPriority::Normal < WorkflowPriority::High);
        assert!(WorkflowPriority::High < WorkflowPriority::Critical);
    }

    #[test]
    fn label_round_trips_for_all_variants() {
        for p in [
            WorkflowPriority::Low,
            WorkflowPriority::Normal,
            WorkflowPriority::High,
            WorkflowPriority::Critical,
        ] {
            let label = p.label();
            let parsed = WorkflowPriority::from_label(label).expect("label must round-trip");
            assert_eq!(parsed, p, "round-trip failed for {:?}", p);
        }
    }

    #[test]
    fn from_label_rejects_unknown_strings() {
        assert!(WorkflowPriority::from_label("urgent").is_none());
        assert!(WorkflowPriority::from_label("").is_none());
        assert!(WorkflowPriority::from_label("NORMAL").is_none());
    }

    #[test]
    fn is_expedited_only_for_high_and_critical() {
        assert!(!WorkflowPriority::Low.is_expedited());
        assert!(!WorkflowPriority::Normal.is_expedited());
        assert!(WorkflowPriority::High.is_expedited());
        assert!(WorkflowPriority::Critical.is_expedited());
    }

    #[test]
    fn default_priority_is_normal() {
        assert_eq!(WorkflowPriority::default(), WorkflowPriority::Normal);
    }
}
