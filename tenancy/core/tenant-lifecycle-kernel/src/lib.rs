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

use core::future::Future;
use core::pin::Pin;
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
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(String, Tenant)>, StoreError>> + Send + 'a>>;

    // S-B: thread tenant_id (RLS scope) through the idempotency + operation
    // ledger methods below so the durable sqlx adapter can key per-tenant. The
    // call sites in the usecase lack tenant context today (the applied/operation
    // tables are global), so S-A keeps them opaque-keyed; threading the scope is
    // a separate, orthogonal change deferred to S-B with the real adapter.

    /// Point read of the idempotency dedup record for `key`.
    fn get_applied<'a>(
        &'a self,
        key: &'a str, // S-B: thread tenant_id for RLS
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppliedWriteRecord>, StoreError>> + Send + 'a>>;

    /// Record what `key` was first applied to.
    fn put_applied<'a>(
        &'a mut self,
        key: &'a str, // S-B: thread tenant_id for RLS
        record: &'a AppliedWriteRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Point read of the ledger entry for `operation_name`.
    fn get_operation<'a>(
        &'a self,
        operation_name: &'a str, // S-B: thread tenant_id for RLS
    ) -> Pin<Box<dyn Future<Output = Result<Option<OperationRecord>, StoreError>> + Send + 'a>>;

    /// Write a ledger entry. Callers MUST never overwrite a terminal entry
    /// (the usecase layer enforces immutability before calling this).
    fn put_operation<'a>(
        &'a mut self,
        operation_name: &'a str, // S-B: thread tenant_id for RLS
        record: &'a OperationRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    /// Next monotonic ledger ordinal, used to mint unique operation names.
    fn next_operation_seq<'a>(
        &'a mut self, // S-B: thread tenant_id for RLS
    ) -> Pin<Box<dyn Future<Output = Result<u64, StoreError>> + Send + 'a>>;
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
