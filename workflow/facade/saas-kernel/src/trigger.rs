//! Workflow trigger and step-status primitives for the SaaS workflow kernel.
//!
//! `TriggerKind` classifies how a workflow run is initiated; `StepStatus`
//! carries the per-step execution outcome visible to the domain layer.
//! Both enums are closed-set kernel contracts per ADR-0023 (plugin sandbox):
//! new variants require a version bump and ADR amendment.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// How a workflow run was initiated.
///
/// Covers the six canonical trigger surfaces declared in the workflow schema
/// (`workflow.runs.triggered_by_kind` CHECK constraint and
/// `workflow.triggers.trigger_type` CHECK constraint), per the P12 DDL and
/// Bominal ADR-0148 translated to oyatie BNF v4.1:
///
/// * `Manual`    – human actor pressed "Run" in Workflow Studio
/// * `Cron`      – cron/one-shot timer fired via the triggers BC
/// * `Webhook`   – inbound HTTP POST to the webhook receiver kernel
/// * `Event`     – internal domain event delivered via the eventing BC
/// * `Ontology`  – ontology object mutation triggered a workflow run (FR-10)
/// * `Api`       – direct API invocation (e.g. REST trigger endpoint)
///
/// Schema contract: `as_str()` values must satisfy the DB CHECK constraint
/// `triggered_by_kind IN ('cron','webhook','event','ontology','manual','api')`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TriggerKind {
    Manual,
    Cron,
    Webhook,
    Event,
    Ontology,
    Api,
}

impl TriggerKind {
    /// Returns the canonical lowercase label used in audit events, OpenAPI
    /// discriminators, and the `workflow.runs.triggered_by_kind` DB column.
    ///
    /// Values satisfy: `CHECK (triggered_by_kind IN
    /// ('cron','webhook','event','ontology','manual','api'))`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Cron => "cron",
            Self::Webhook => "webhook",
            Self::Event => "event",
            Self::Ontology => "ontology",
            Self::Api => "api",
        }
    }
}

impl std::fmt::Display for TriggerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Execution outcome of a single workflow step.
///
/// Distinct from [`crate::WorkflowRunState`] (which tracks the run lifecycle)
/// — `StepStatus` is the per-step result written into the `workflow.step_runs`
/// table and surfaced in `WorkflowEventKind::StepCompleted` / `StepFailed`
/// events.
///
/// Schema contract: `as_str()` values must satisfy the DB CHECK constraint
/// `status IN ('pending','running','completed','failed','skipped')`.
///
/// Variant semantics:
///
/// * `Pending`   – step is queued but not yet started
/// * `Running`   – step worker has claimed the step
/// * `Completed` – step finished with a usable output
/// * `Failed`    – step finished with a non-retryable error
/// * `Skipped`   – branch logic determined the step should not execute
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl StepStatus {
    /// `true` when no further transitions are possible.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Skipped)
    }

    /// Returns the canonical lowercase label used in the `workflow.step_runs`
    /// status column and audit events.
    ///
    /// Values satisfy: `CHECK (status IN
    /// ('pending','running','completed','failed','skipped'))`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_kind_labels_are_unique_and_non_empty() {
        let kinds = [
            TriggerKind::Manual,
            TriggerKind::Cron,
            TriggerKind::Webhook,
            TriggerKind::Event,
            TriggerKind::Ontology,
            TriggerKind::Api,
        ];
        let mut labels: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), kinds.len(), "all labels must be unique");
        for k in &kinds {
            assert!(!k.as_str().is_empty());
            assert_eq!(k.to_string(), k.as_str());
        }
    }

    /// Synthetic violation: verify that invalid schema labels are NOT emitted.
    /// The DB CHECK constraint rejects these; this test documents the contract.
    #[test]
    fn trigger_kind_labels_match_db_check_constraint() {
        let allowed = ["cron", "webhook", "event", "ontology", "manual", "api"];
        let kinds = [
            TriggerKind::Manual,
            TriggerKind::Cron,
            TriggerKind::Webhook,
            TriggerKind::Event,
            TriggerKind::Ontology,
            TriggerKind::Api,
        ];
        for k in &kinds {
            assert!(
                allowed.contains(&k.as_str()),
                "TriggerKind::{:?} emits '{}' which is not in the DB CHECK constraint",
                k,
                k.as_str()
            );
        }
        // Confirm the old invalid labels are gone
        let emitted: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
        assert!(
            !emitted.contains(&"schedule"),
            "old 'schedule' label must not be emitted"
        );
        assert!(
            !emitted.contains(&"event-driven"),
            "old 'event-driven' label must not be emitted"
        );
        assert!(
            !emitted.contains(&"sub-workflow"),
            "old 'sub-workflow' label must not be emitted"
        );
    }

    #[test]
    fn step_status_terminal_predicate() {
        assert!(!StepStatus::Pending.is_terminal());
        assert!(!StepStatus::Running.is_terminal());
        assert!(StepStatus::Completed.is_terminal());
        assert!(StepStatus::Failed.is_terminal());
        assert!(StepStatus::Skipped.is_terminal());
    }

    #[test]
    fn step_status_labels_are_unique_and_non_empty() {
        let statuses = [
            StepStatus::Pending,
            StepStatus::Running,
            StepStatus::Completed,
            StepStatus::Failed,
            StepStatus::Skipped,
        ];
        let mut labels: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), statuses.len(), "all labels must be unique");
        for s in &statuses {
            assert!(!s.as_str().is_empty());
            assert_eq!(s.to_string(), s.as_str());
        }
    }

    /// Synthetic violation: verify that invalid schema labels are NOT emitted.
    /// The DB CHECK constraint for step_runs.status rejects these.
    #[test]
    fn step_status_labels_match_db_check_constraint() {
        let allowed = ["pending", "running", "completed", "failed", "skipped"];
        let statuses = [
            StepStatus::Pending,
            StepStatus::Running,
            StepStatus::Completed,
            StepStatus::Failed,
            StepStatus::Skipped,
        ];
        for s in &statuses {
            assert!(
                allowed.contains(&s.as_str()),
                "StepStatus::{:?} emits '{}' which is not in the DB CHECK constraint",
                s,
                s.as_str()
            );
        }
        // Confirm the old invalid labels are gone
        let emitted: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
        assert!(
            !emitted.contains(&"succeeded"),
            "old 'succeeded' label must not be emitted"
        );
        assert!(
            !emitted.contains(&"timed-out"),
            "old 'timed-out' label must not be emitted"
        );
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(TriggerKind::Webhook.to_string(), "webhook");
        assert_eq!(TriggerKind::Event.to_string(), "event");
        assert_eq!(TriggerKind::Ontology.to_string(), "ontology");
        assert_eq!(StepStatus::Completed.to_string(), "completed");
        assert_eq!(StepStatus::Skipped.to_string(), "skipped");
    }
}
