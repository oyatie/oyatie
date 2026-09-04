use serde::{Deserialize, Serialize};

use super::{OperationPhase, OperationState};
use crate::error::ContractShapeError;
use crate::identity::{IdempotencyKey, is_slug};

/// Retry metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    pub backoff: String,              // data_class: INTERNAL_ONLY
    pub max_attempts: u32,            // data_class: INTERNAL_ONLY
    pub retry_classification: String, // data_class: INTERNAL_ONLY
}

/// Retry classifications allowed by
/// `specs/cloud-control-plane-operation-contract.json#idempotency_retry_cancel_contract`.
pub const ALLOWED_RETRY_CLASSIFICATIONS: &[&str] = &[
    "transient",
    "quota",
    "policy",
    "dependency",
    "operator_required",
];

/// Cancellation metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancellationMetadata {
    pub cancel_safe: bool,    // data_class: INTERNAL_ONLY
    pub audit_required: bool, // data_class: INTERNAL_ONLY
}

/// Compensation metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompensationMetadata {
    pub required: bool,   // data_class: INTERNAL_ONLY
    pub strategy: String, // data_class: INTERNAL_ONLY
}

/// Durable operation-ledger row required before acknowledging a mutating
/// resource-provider request. This mirrors
/// `specs/cloud-control-plane-operation-contract.json#operation_ledger_entry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationLedgerEntry {
    pub operation_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub request_hash: String,               // data_class: INTERNAL_ONLY
    pub resource_orn: String,               // data_class: TENANT_SCOPED
    pub desired_generation: u64,            // data_class: INTERNAL_ONLY
    pub observed_generation: u64,           // data_class: INTERNAL_ONLY
    pub state: OperationState,              // data_class: INTERNAL_ONLY
    pub phase: OperationPhase,              // data_class: INTERNAL_ONLY
    pub tenant_account_project: String,     // data_class: TENANT_SCOPED
    pub region_cell: String,                // data_class: TENANT_SCOPED
    pub principal: String,                  // data_class: INTERNAL_ONLY
    pub audit_chain_id: String,             // data_class: INTERNAL_ONLY
    pub retry_policy: RetryPolicy,          // data_class: INTERNAL_ONLY
    pub cancellation: CancellationMetadata, // data_class: INTERNAL_ONLY
    pub compensation: CompensationMetadata, // data_class: INTERNAL_ONLY
    pub transition_sequence: u64,           // data_class: INTERNAL_ONLY
}

impl OperationLedgerEntry {
    /// Validate the metadata-only operation-ledger contract: write-before-ack
    /// idempotency key, request hash, audit-chain linkage, generation bounds,
    /// retry/cancel/compensation metadata, and monotonic sequence presence.
    pub fn validate(&self) -> Result<(), ContractShapeError> {
        if !is_slug(&self.operation_id) {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!("operation_id {:?} is not slug-shaped", self.operation_id),
            });
        }
        IdempotencyKey::new(self.idempotency_key.clone()).map_err(|error| {
            ContractShapeError::MalformedOperationLedger {
                message: error.to_string(),
            }
        })?;
        if self.request_hash.is_empty() {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "request_hash must be non-empty".to_owned(),
            });
        }
        if !self.resource_orn.starts_with("orn:") || !self.resource_orn.contains('/') {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!("resource_orn {:?} is not ORN-shaped", self.resource_orn),
            });
        }
        if self.desired_generation == 0 || self.observed_generation > self.desired_generation {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!(
                    "generation bounds invalid: desired={}, observed={}",
                    self.desired_generation, self.observed_generation
                ),
            });
        }
        for (field, value) in [
            (
                "tenant_account_project",
                self.tenant_account_project.as_str(),
            ),
            ("region_cell", self.region_cell.as_str()),
            ("principal", self.principal.as_str()),
            ("audit_chain_id", self.audit_chain_id.as_str()),
            ("retry_policy.backoff", self.retry_policy.backoff.as_str()),
            (
                "retry_policy.retry_classification",
                self.retry_policy.retry_classification.as_str(),
            ),
            ("compensation.strategy", self.compensation.strategy.as_str()),
        ] {
            if value.is_empty() {
                return Err(ContractShapeError::MalformedOperationLedger {
                    message: format!("{field} must be non-empty"),
                });
            }
        }
        if self.retry_policy.max_attempts == 0 {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "retry_policy.max_attempts must be non-zero".to_owned(),
            });
        }
        if !ALLOWED_RETRY_CLASSIFICATIONS.contains(&self.retry_policy.retry_classification.as_str())
        {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!(
                    "retry_policy.retry_classification {:?} is not one of {:?}",
                    self.retry_policy.retry_classification, ALLOWED_RETRY_CLASSIFICATIONS
                ),
            });
        }
        if !self.cancellation.audit_required {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "cancellation.audit_required must be true".to_owned(),
            });
        }
        if self.transition_sequence == 0 {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "transition_sequence must be non-zero".to_owned(),
            });
        }
        Ok(())
    }
}
