//! Tenant lifecycle kernel — the storage port and persisted record shapes
//! for the tenant lifecycle control plane.
//!
//! Grounded in the locked G001 contracts: the tenant aggregate, lifecycle
//! state machine, and isolation posture come from
//! `oya-shared-platform-contracts-kernel`; the resource/operation/idempotency
//! shapes come from `oya-shared-resource-provider-contract-kernel`. This
//! crate never re-invents either — it only defines what the lifecycle
//! control plane persists and the port it persists through.
//!
//! The port models the OWNED destination store (oya-data: ordered keyed
//! records with point get/put and range scans — the multi-Raft
//! leader-per-range KV shape). Transient adapters (sqlx/Postgres from the
//! G03 lane, in-memory test fixtures) absorb all impedance behind it; the
//! trait would not change at W5 cutover.
//!
//! Per ADR-0105 the kernel layer is pure types and ports: zero I/O, zero
//! business logic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use oya_shared_platform_contracts_kernel::tenancy::{Tenant, TenantLifecycleOperation};
use oya_shared_resource_provider_contract_kernel::Operation;
use serde::{Deserialize, Serialize};

/// What a client-UUID idempotency key was first applied to: the dedup record
/// consulted on every replay (AIP-155 request ids / AWS client tokens).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedWriteRecord {
    /// A synchronous create of `name` with exactly this payload.
    Create { name: String, tenant: Tenant },
    /// A synchronous full-replace put of `name` with exactly this payload.
    Put { name: String, tenant: Tenant },
    /// An async lifecycle mutation of `name`; replays return the SAME
    /// operation resource from the ledger.
    Lifecycle {
        name: String,
        operation: TenantLifecycleOperation,
        operation_name: String,
    },
}

/// One entry in the AIP-151 operation ledger: the operation resource plus
/// the lifecycle mutation it tracks. Terminal entries are immutable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRecord {
    /// The AIP-151 operation resource (`operations/...`, done, result).
    pub operation: Operation, // data_class: INTERNAL_ONLY
    /// The lifecycle transition this operation applies.
    pub kind: TenantLifecycleOperation, // data_class: INTERNAL_ONLY
    /// Resource name (`tenants/<id>`) the transition targets.
    pub target: String, // data_class: TENANT_SCOPED
}

/// Storage-port failures. The port is infallible on semantics (absence is
/// `Ok(None)`); errors are availability/integrity only, so the usecase layer
/// can map them to `internal` without guessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The backing store cannot serve the request right now.
    Unavailable { detail: String },
    /// A persisted record failed to decode against the locked contracts.
    Corrupt { detail: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { detail } => write!(f, "store unavailable: {detail}"),
            Self::Corrupt { detail } => write!(f, "store record corrupt: {detail}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// The lifecycle control plane's storage port: ordered keyed records with
/// point get/put/remove and an ordered range scan (the owned oya-data
/// shape). Synchronous and IO-free at this layer; adapters own transport.
pub trait TenantLifecycleStore {
    /// Point read of the tenant stored under `name` (`tenants/<id>`).
    fn get_tenant(&self, name: &str) -> Result<Option<Tenant>, StoreError>;

    /// Point write of `tenant` under `name`.
    fn put_tenant(&mut self, name: &str, tenant: &Tenant) -> Result<(), StoreError>;

    /// Remove the tenant under `name` (no-op when absent).
    fn remove_tenant(&mut self, name: &str) -> Result<(), StoreError>;

    /// Ordered scan of tenant records whose key starts with `prefix`,
    /// beginning at `start_at` (inclusive) when given, yielding at most
    /// `limit` entries in ascending key order. The order MUST be a stable
    /// total order over keys (AIP-158 pagination is built on it).
    fn scan_tenants(
        &self,
        prefix: &str,
        start_at: Option<&str>,
        limit: u32,
    ) -> Result<Vec<(String, Tenant)>, StoreError>;

    /// Point read of the idempotency dedup record for `key`.
    fn get_applied(&self, key: &str) -> Result<Option<AppliedWriteRecord>, StoreError>;

    /// Record what `key` was first applied to.
    fn put_applied(&mut self, key: &str, record: &AppliedWriteRecord) -> Result<(), StoreError>;

    /// Point read of the ledger entry for `operation_name`.
    fn get_operation(&self, operation_name: &str) -> Result<Option<OperationRecord>, StoreError>;

    /// Write a ledger entry. Callers MUST never overwrite a terminal entry
    /// (the usecase layer enforces immutability before calling this).
    fn put_operation(
        &mut self,
        operation_name: &str,
        record: &OperationRecord,
    ) -> Result<(), StoreError>;

    /// Next monotonic ledger ordinal, used to mint unique operation names.
    fn next_operation_seq(&mut self) -> Result<u64, StoreError>;
}

#[cfg(test)]
mod tests {
    use oya_shared_platform_contracts_kernel::tenancy::{
        IsolationPosture, TenantLifecycleState,
    };

    use super::*;

    #[test]
    fn applied_write_record_round_trips() {
        let record = AppliedWriteRecord::Create {
            name: "tenants/acme".to_owned(),
            tenant: Tenant {
                tenant_id: "acme".to_owned(),
                display_name: "Acme Corp".to_owned(),
                state: TenantLifecycleState::initial(),
                isolation_posture: IsolationPosture::Pooled,
                cell_id: "cell-001".to_owned(),
                residency_zone: None,
            },
        };
        let json = serde_json::to_string(&record).unwrap();
        assert_eq!(
            serde_json::from_str::<AppliedWriteRecord>(&json).unwrap(),
            record
        );
    }

    #[test]
    fn operation_record_round_trips_and_rejects_unknown_fields() {
        let record = OperationRecord {
            operation: Operation::pending("operations/lifecycle-000001").unwrap(),
            kind: TenantLifecycleOperation::Activate,
            target: "tenants/acme".to_owned(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert_eq!(
            serde_json::from_str::<OperationRecord>(&json).unwrap(),
            record
        );
        let mut value = serde_json::to_value(&record).unwrap();
        value["surprise"] = serde_json::json!(true);
        assert!(serde_json::from_value::<OperationRecord>(value).is_err());
    }
}
