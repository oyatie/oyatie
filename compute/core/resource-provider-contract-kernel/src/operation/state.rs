use serde::{Deserialize, Serialize};

/// Structured terminal error of an operation (AIP-193-shaped code+message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationError {
    pub code: String,    // data_class: INTERNAL_ONLY
    pub message: String, // data_class: INTERNAL_ONLY
}

/// Terminal outcome of an operation: response XOR error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationResult {
    Response(serde_json::Value),
    Error(OperationError),
}

/// Durable control-plane state for an AIP-151 long-running operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Accepted,
    Validating,
    Queued,
    Running,
    WaitingForReconciler,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
    Compensating,
    RolledBack,
}

impl OperationState {
    /// Whether this state is terminal per the control-plane operation contract.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::RolledBack
        )
    }

    /// Whether the control-plane operation state machine allows this state to
    /// transition to `next`.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Accepted, Self::Validating)
                | (Self::Accepted, Self::CancelRequested)
                | (Self::Validating, Self::Queued)
                | (Self::Validating, Self::Failed)
                | (Self::Validating, Self::CancelRequested)
                | (Self::Queued, Self::Running)
                | (Self::Queued, Self::CancelRequested)
                | (Self::Running, Self::WaitingForReconciler)
                | (Self::Running, Self::Succeeded)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::CancelRequested)
                | (Self::Running, Self::Compensating)
                | (Self::WaitingForReconciler, Self::Running)
                | (Self::WaitingForReconciler, Self::CancelRequested)
                | (Self::CancelRequested, Self::Cancelled)
                | (Self::CancelRequested, Self::Failed)
                | (Self::Compensating, Self::RolledBack)
                | (Self::Compensating, Self::Failed)
        )
    }
}

/// The control-plane pipeline phase owning the current operation ledger row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationPhase {
    ApiGateway,
    ResourceRegistry,
    OperationLedger,
    WorkflowReconciler,
    BackendActuationBoundary,
}
