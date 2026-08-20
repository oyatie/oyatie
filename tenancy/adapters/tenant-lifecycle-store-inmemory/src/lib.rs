//! In-memory [`TenantLifecycleStore`] adapter.
//!
//! A faithful, ordered-keyed implementation of the kernel storage port over
//! plain `BTreeMap`s: point get/put/remove, an ascending-key range scan, the
//! idempotency dedup table, and the monotonic operation-ledger ordinal. This
//! is the valid ports/adapters realization of the OWNED oya-data store shape
//! for single-node bring-up and acceptance tests; the G03 persistent
//! (sqlx/Postgres) adapter plugs in behind the SAME port with no usecase
//! change.
//!
//! Layering (ADR-0131): adapters depend path-inward on the kernel port and the
//! locked G001 contracts only. No business logic lives here — the lifecycle
//! decision algorithm stays in the usecase + contract FSM.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;
use std::collections::BTreeMap;

use oya_shared_platform_contracts_kernel::tenancy::Tenant;
use tenancy_tenant_lifecycle_kernel::{
    AppliedWriteRecord, OperationRecord, StoreError, TenantLifecycleStore,
};

/// In-memory tenant lifecycle store: ordered keyed records plus the
/// idempotency and operation-ledger side tables.
///
/// Not `Clone`: the lifecycle usecase owns exactly one store and mutates it
/// behind a single composition-root lock. Cloning would fork the ledger and
/// break idempotency, so it is deliberately omitted.
#[derive(Debug, Default)]
pub struct InMemoryTenantLifecycleStore {
    /// `tenants/<id>` -> tenant aggregate (tombstones retained as Retired).
    tenants: BTreeMap<String, Tenant>,
    /// Idempotency dedup table, keyed PER-TENANT by `(tenant_id, key)` — the
    /// SAME multi-tenant shape as the durable backend's
    /// `PRIMARY KEY (tenant_id, idempotency_key)`. A key reused under two
    /// different tenants addresses two INDEPENDENT records; one tenant can never
    /// read another tenant's applied record under a shared key.
    applied: BTreeMap<(String, String), AppliedWriteRecord>,
    /// AIP-151 operation ledger, keyed PER-TENANT by `(tenant_id,
    /// operation_name)` — matching the durable backend's
    /// `PRIMARY KEY (tenant_id, operation_name)`.
    operations: BTreeMap<(String, String), OperationRecord>,
    /// PER-TENANT monotonic operation ordinal, keyed by `tenant_id` — matching
    /// the durable backend's per-tenant `max(operation_seq) + 1` derivation
    /// (`NEXT_SEQ_SQL ... WHERE tenant_id = $1`). A global counter would diverge
    /// from the durable value (tenant B's first seq would not be 1), so the
    /// ordinal is namespaced per tenant for backend parity.
    operation_seq: BTreeMap<String, u64>,
}

impl InMemoryTenantLifecycleStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored tenant records, INCLUDING retired tombstones. For
    /// observability and tests; not part of the storage port.
    #[must_use]
    pub fn stored_tenant_count(&self) -> usize {
        self.tenants.len()
    }
}

impl TenantLifecycleStore for InMemoryTenantLifecycleStore {
    fn get_tenant<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, StoreError>> + Send + 'a>> {
        Box::pin(async move { Ok(self.tenants.get(name).cloned()) })
    }

    fn put_tenant<'a>(
        &'a mut self,
        name: &'a str,
        tenant: &'a Tenant,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.tenants.insert(name.to_owned(), tenant.clone());
            Ok(())
        })
    }

    fn remove_tenant<'a>(
        &'a mut self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.tenants.remove(name);
            Ok(())
        })
    }

    fn scan_tenants<'a>(
        &'a self,
        prefix: &'a str,
        start_at: Option<&'a str>,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(String, Tenant)>, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            // Ascending-key walk from the inclusive lower bound (the larger of
            // `prefix` and `start_at`), stopping at the first key that escapes
            // the prefix or once `limit` rows are gathered. BTreeMap ranges
            // already yield a stable total order over keys (AIP-158).
            let lower = match start_at {
                Some(start) if start >= prefix => start,
                _ => prefix,
            };
            let mut out = Vec::new();
            if limit == 0 {
                return Ok(out);
            }
            for (key, tenant) in self.tenants.range(lower.to_owned()..) {
                if !key.starts_with(prefix) {
                    break;
                }
                out.push((key.clone(), tenant.clone()));
                if out.len() as u32 == limit {
                    break;
                }
            }
            Ok(out)
        })
    }

    fn get_applied<'a>(
        &'a self,
        tenant_id: &'a str,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppliedWriteRecord>, StoreError>> + Send + 'a>>
    {
        // The idempotency dedup table is keyed PER-TENANT by `(tenant_id, key)`,
        // matching the durable backend's `PRIMARY KEY (tenant_id,
        // idempotency_key)`. Keying globally by `key` alone would let tenant B
        // read tenant A's applied record under a shared key — a cross-tenant
        // leak and a divergence from the durable shape.
        Box::pin(async move {
            Ok(self
                .applied
                .get(&(tenant_id.to_owned(), key.to_owned()))
                .cloned())
        })
    }

    fn put_applied<'a>(
        &'a mut self,
        tenant_id: &'a str,
        key: &'a str,
        record: &'a AppliedWriteRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.applied
                .insert((tenant_id.to_owned(), key.to_owned()), record.clone());
            Ok(())
        })
    }

    fn get_operation<'a>(
        &'a self,
        tenant_id: &'a str,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<OperationRecord>, StoreError>> + Send + 'a>>
    {
        Box::pin(async move {
            Ok(self
                .operations
                .get(&(tenant_id.to_owned(), operation_name.to_owned()))
                .cloned())
        })
    }

    fn put_operation<'a>(
        &'a mut self,
        tenant_id: &'a str,
        operation_name: &'a str,
        record: &'a OperationRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.operations.insert(
                (tenant_id.to_owned(), operation_name.to_owned()),
                record.clone(),
            );
            Ok(())
        })
    }

    fn next_operation_seq<'a>(
        &'a mut self,
        tenant_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, StoreError>> + Send + 'a>> {
        // PER-TENANT monotonic ordinal, matching the durable backend's
        // per-tenant `max(operation_seq) + 1` (NEXT_SEQ_SQL is scoped
        // `WHERE tenant_id = $1`). Each tenant's first minted seq is 1, so the
        // value parity holds across backends — a global counter would make
        // tenant B start above 1 and diverge.
        Box::pin(async move {
            let seq = self.operation_seq.entry(tenant_id.to_owned()).or_insert(0);
            *seq = seq.saturating_add(1);
            Ok(*seq)
        })
    }
}

#[cfg(test)]
mod tests {
    use oya_shared_platform_contracts_kernel::tenancy::{
        IsolationPosture, TenantLifecycleOperation, TenantLifecycleState,
    };
    use oya_shared_resource_provider_contract_kernel::{
        CancellationMetadata, CompensationMetadata, Operation, OperationLedgerEntry,
        OperationPhase, OperationState, RetryPolicy,
    };

    use super::*;

    fn tenant(id: &str) -> Tenant {
        Tenant {
            tenant_id: id.to_owned(),
            display_name: format!("Tenant {id}"),
            state: TenantLifecycleState::initial(),
            isolation_posture: IsolationPosture::Pooled,
            cell_id: "cell-001".to_owned(),
            residency_zone: None,
        }
    }

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

    #[tokio::test]
    async fn put_then_get_round_trips() {
        let mut store = InMemoryTenantLifecycleStore::new();
        store
            .put_tenant("tenants/acme", &tenant("acme"))
            .await
            .unwrap();
        assert_eq!(
            store.get_tenant("tenants/acme").await.unwrap(),
            Some(tenant("acme"))
        );
        assert_eq!(store.get_tenant("tenants/ghost").await.unwrap(), None);
    }

    #[tokio::test]
    async fn scan_is_ordered_prefix_bounded_and_paged() {
        let mut store = InMemoryTenantLifecycleStore::new();
        // Insert out of order; also a key OUTSIDE the prefix to prove bounding.
        for id in ["c", "a", "b"] {
            store
                .put_tenant(&format!("tenants/{id}"), &tenant(id))
                .await
                .unwrap();
        }
        store
            .put_tenant("operations/lifecycle-1", &tenant("noise"))
            .await
            .unwrap();

        let page = store.scan_tenants("tenants/", None, 2).await.unwrap();
        let keys: Vec<_> = page.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec!["tenants/a".to_owned(), "tenants/b".to_owned()]);

        // Resume strictly after the last returned key.
        let next = store
            .scan_tenants("tenants/", Some("tenants/b\0"), 10)
            .await
            .unwrap();
        let keys: Vec<_> = next.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec!["tenants/c".to_owned()]);
    }

    #[tokio::test]
    async fn operation_seq_is_strictly_monotonic() {
        let mut store = InMemoryTenantLifecycleStore::new();
        let first = store.next_operation_seq("acme").await.unwrap();
        let second = store.next_operation_seq("acme").await.unwrap();
        assert_eq!(first, 1);
        assert_eq!(second, 2);
    }

    #[tokio::test]
    async fn operation_seq_is_per_tenant_for_durable_parity() {
        // The ordinal is namespaced per tenant, matching the durable backend's
        // per-tenant `max(operation_seq) + 1`: tenant beta's first seq is 1,
        // independent of how many acme has minted (value parity, no global
        // counter leak across tenants).
        let mut store = InMemoryTenantLifecycleStore::new();
        assert_eq!(store.next_operation_seq("acme").await.unwrap(), 1);
        assert_eq!(store.next_operation_seq("acme").await.unwrap(), 2);
        assert_eq!(
            store.next_operation_seq("beta").await.unwrap(),
            1,
            "tenant beta's first ordinal must be 1, not continue acme's counter"
        );
        assert_eq!(store.next_operation_seq("acme").await.unwrap(), 3);
        assert_eq!(store.next_operation_seq("beta").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn applied_and_operation_tables_round_trip() {
        let mut store = InMemoryTenantLifecycleStore::new();
        let record = AppliedWriteRecord::Create {
            name: "tenants/acme".to_owned(),
            tenant: tenant("acme"),
        };
        store.put_applied("acme", "key-1", &record).await.unwrap();
        assert_eq!(
            store.get_applied("acme", "key-1").await.unwrap(),
            Some(record)
        );

        let op = OperationRecord {
            operation: Operation::pending(
                "operations/acme-lifecycle-000001",
                operation_ledger_entry("acme-lifecycle-000001"),
            )
            .unwrap(),
            kind: TenantLifecycleOperation::Activate,
            target: "tenants/acme".to_owned(),
        };
        store
            .put_operation("acme", "operations/lifecycle-000001", &op)
            .await
            .unwrap();
        assert_eq!(
            store
                .get_operation("acme", "operations/lifecycle-000001")
                .await
                .unwrap(),
            Some(op)
        );
    }

    #[tokio::test]
    async fn applied_dedup_namespace_is_per_tenant() {
        // The idempotency dedup table is keyed PER-TENANT by `(tenant_id, key)`,
        // matching the durable backend's `PRIMARY KEY (tenant_id,
        // idempotency_key)`. The SAME key under two tenant scopes addresses two
        // INDEPENDENT records — tenant beta must NEVER read tenant acme's record
        // under a shared key (no cross-tenant leak).
        let mut store = InMemoryTenantLifecycleStore::new();
        let acme_record = AppliedWriteRecord::Create {
            name: "tenants/acme".to_owned(),
            tenant: tenant("acme"),
        };
        let beta_record = AppliedWriteRecord::Create {
            name: "tenants/beta".to_owned(),
            tenant: tenant("beta"),
        };
        store
            .put_applied("acme", "shared-key", &acme_record)
            .await
            .unwrap();

        // A different tenant scope does NOT see acme's record under the same key.
        assert_eq!(
            store.get_applied("beta", "shared-key").await.unwrap(),
            None,
            "tenant beta must not read tenant acme's applied record"
        );

        // Each tenant keeps its own independent record under the shared key.
        store
            .put_applied("beta", "shared-key", &beta_record)
            .await
            .unwrap();
        assert_eq!(
            store.get_applied("acme", "shared-key").await.unwrap(),
            Some(acme_record)
        );
        assert_eq!(
            store.get_applied("beta", "shared-key").await.unwrap(),
            Some(beta_record)
        );
    }

    #[tokio::test]
    async fn operation_ledger_is_per_tenant() {
        // The operation ledger is likewise keyed `(tenant_id, operation_name)`:
        // the same operation_name under two tenants is two independent entries.
        let mut store = InMemoryTenantLifecycleStore::new();
        let op_acme = OperationRecord {
            operation: Operation::pending(
                "operations/acme-lifecycle-000001",
                operation_ledger_entry("acme-lifecycle-000001"),
            )
            .unwrap(),
            kind: TenantLifecycleOperation::Activate,
            target: "tenants/acme".to_owned(),
        };
        store
            .put_operation("acme", "operations/shared", &op_acme)
            .await
            .unwrap();
        assert_eq!(
            store
                .get_operation("beta", "operations/shared")
                .await
                .unwrap(),
            None,
            "tenant beta must not read tenant acme's operation record"
        );
        assert_eq!(
            store
                .get_operation("acme", "operations/shared")
                .await
                .unwrap(),
            Some(op_acme)
        );
    }
}
