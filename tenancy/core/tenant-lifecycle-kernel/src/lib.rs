//! Tenant lifecycle kernel — the storage port and persisted record shapes
//! for the tenant lifecycle control plane.
//!
//! Grounded in the locked G001 contracts: the tenant aggregate, lifecycle
//! state machine, and isolation posture come from
//! `shared-platform-contracts-kernel`; the resource/operation/idempotency
//! shapes come from `shared-resource-provider-contract-kernel`. This
//! crate never re-invents either — it only defines what the lifecycle
//! control plane persists and the port it persists through.
//!
//! The port models the OWNED destination store (data: ordered keyed
//! records with point get/put and range scans — the multi-Raft
//! leader-per-range KV shape). Transient adapters (sqlx/Postgres from the
//! G03 lane, in-memory test fixtures) absorb all impedance behind it; the
//! trait would not change at W5 cutover.
//!
//! Per ADR-0105 the kernel layer is pure types and ports: zero I/O, zero
//! business logic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;
use std::fmt;

use serde::{Deserialize, Serialize};
use shared_platform_contracts_kernel::tenancy::{Tenant, TenantLifecycleOperation};
use shared_resource_provider_contract_kernel::Operation;

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

/// Async ordered tenant-scan result returned by [`TenantLifecycleStore`].
pub type TenantScanFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<(String, Tenant)>, StoreError>> + Send + 'a>>;

/// The lifecycle control plane's storage port: ordered keyed records with
/// point get/put/remove and an ordered range scan (the owned data
/// shape). Async (the durable backend performs real I/O) but IO-free at this
/// layer; adapters own transport. Async is modelled with a return-position
/// boxed future — `core::future::Future` + `core::pin::Pin` + `Box::pin`, no
/// `async-trait` / `futures` dep — so the kernel stays dependency-free
/// (kernel-purity gate, ADR-0547; ADR-0376 rejects async-trait for ports).
pub trait TenantLifecycleStore {
    /// Point read of the tenant stored under `name` (`tenants/<id>`).
    fn get_tenant<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, StoreError>> + Send + 'a>>;

    /// Point write of `tenant` under `name`.
    fn put_tenant<'a>(
        &'a mut self,
        name: &'a str,
        tenant: &'a Tenant,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Remove the tenant under `name` (no-op when absent).
    fn remove_tenant<'a>(
        &'a mut self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Ordered scan of tenant records whose key starts with `prefix`,
    /// beginning at `start_at` (inclusive) when given, yielding at most
    /// `limit` entries in ascending key order. The order MUST be a stable
    /// total order over keys (AIP-158 pagination is built on it).
    fn scan_tenants<'a>(
        &'a self,
        prefix: &'a str,
        start_at: Option<&'a str>,
        limit: u32,
    ) -> TenantScanFuture<'a>;

    // S-B: tenant_id (RLS scope) is threaded through the idempotency + operation
    // ledger methods below so the durable sqlx adapter can set the per-transaction
    // tenant GUC (`oyatie.tenant_id`) before any tenant-scoped statement. The
    // applied/operation tables are tenant-scoped (RLS RESTRICTIVE per tenant_id);
    // the in-memory adapter ignores the scope (its maps are process-local) but
    // accepts it so both adapters share one port.

    /// Point read of the idempotency dedup record for `key`, within `tenant_id`.
    fn get_applied<'a>(
        &'a self,
        tenant_id: &'a str,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppliedWriteRecord>, StoreError>> + Send + 'a>>;

    /// Record what `key` was first applied to, within `tenant_id`.
    fn put_applied<'a>(
        &'a mut self,
        tenant_id: &'a str,
        key: &'a str,
        record: &'a AppliedWriteRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Point read of the ledger entry for `operation_name`, within `tenant_id`.
    fn get_operation<'a>(
        &'a self,
        tenant_id: &'a str,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<OperationRecord>, StoreError>> + Send + 'a>>;

    /// Write a ledger entry within `tenant_id`. Callers MUST never overwrite a
    /// terminal entry (the usecase layer enforces immutability before calling
    /// this).
    fn put_operation<'a>(
        &'a mut self,
        tenant_id: &'a str,
        operation_name: &'a str,
        record: &'a OperationRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Next monotonic ledger ordinal for `tenant_id`, used to mint unique
    /// operation names. The ordinal need only be unique within the tenant.
    fn next_operation_seq<'a>(
        &'a mut self,
        tenant_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, StoreError>> + Send + 'a>>;
}

#[cfg(test)]
mod tests {
    use shared_platform_contracts_kernel::tenancy::{IsolationPosture, TenantLifecycleState};
    use shared_resource_provider_contract_kernel::{
        CancellationMetadata, CompensationMetadata, OperationLedgerEntry, OperationPhase,
        OperationState, RetryPolicy,
    };

    use super::*;
    fn operation_ledger_entry(operation_id: &str) -> OperationLedgerEntry {
        OperationLedgerEntry {
            operation_id: operation_id.to_owned(),
            idempotency_key: "00000000-0000-4000-8000-000000000001".to_owned(),
            request_hash: format!("fixture-hash:{operation_id}"),
            resource_orn: "orn:oya:tenancy:acme:tenants/acme".to_owned(),
            desired_generation: 1,
            observed_generation: 0,
            state: OperationState::Accepted,
            phase: OperationPhase::OperationLedger,
            tenant_account_project: "tenant/acme".to_owned(),
            region_cell: "control-plane/default".to_owned(),
            principal: "principal:test".to_owned(),
            audit_chain_id: format!("audit-chain/{operation_id}"),
            retry_policy: RetryPolicy {
                backoff: "bounded-exponential-jitter".to_owned(),
                max_attempts: 3,
                retry_classification: "transient".to_owned(),
            },
            cancellation: CancellationMetadata {
                cancel_safe: true,
                audit_required: true,
            },
            compensation: CompensationMetadata {
                required: false,
                strategy: "none".to_owned(),
            },
            transition_sequence: 1,
        }
    }

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
            operation: Operation::pending(
                "operations/acme-lifecycle-000001",
                operation_ledger_entry("acme-lifecycle-000001"),
            )
            .unwrap(),
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
