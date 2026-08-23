//! Contract-harness conformance + lifecycle-ledger behavior for the tenant
//! lifecycle control plane.
//!
//! The in-memory store here is a TEST FIXTURE for the `TenantLifecycleStore`
//! port (the production adapter is the G03 persistence lane's job); the
//! provider under test is the real usecase crate. The G001 conformance
//! harness runs in full — per the FD-001 contract lock, every resource
//! provider proves the uniform contract before it ships.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use shared_platform_contracts_kernel::tenancy::{
    IsolationPosture, Tenant, TenantLifecycleOperation, TenantLifecycleState,
};
use shared_resource_provider_contract_kernel::conformance::{
    ConformanceFixture, check_async_delete_operation, check_create_idempotency,
    check_idempotent_put, check_read_after_write, check_stable_pagination, run_all_checks,
};
use shared_resource_provider_contract_kernel::{
    IdempotencyKey, OperationResult, ProviderError, ResourceName, ResourceProvider,
};
use tenancy_tenant_lifecycle_kernel::{
    AppliedWriteRecord, OperationRecord, StoreError, TenantLifecycleStore,
};
use tenancy_tenant_lifecycle_usecase::{TENANT_COLLECTION, TenantLifecycleProvider};

/// In-memory `TenantLifecycleStore`: ordered maps mirror the owned data
/// shape (point get/put + ordered range scan).
#[derive(Debug, Default)]
struct MemoryStore {
    tenants: BTreeMap<String, Tenant>,
    applied: BTreeMap<String, AppliedWriteRecord>,
    operations: BTreeMap<String, OperationRecord>,
    operation_seq: u64,
}

impl TenantLifecycleStore for MemoryStore {
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
            Ok(self
                .tenants
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .filter(|(key, _)| start_at.is_none_or(|start| key.as_str() >= start))
                .take(limit as usize)
                .map(|(key, tenant)| (key.clone(), tenant.clone()))
                .collect())
        })
    }

    fn get_applied<'a>(
        &'a self,
        _tenant_id: &'a str,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppliedWriteRecord>, StoreError>> + Send + 'a>>
    {
        Box::pin(async move { Ok(self.applied.get(key).cloned()) })
    }

    fn put_applied<'a>(
        &'a mut self,
        _tenant_id: &'a str,
        key: &'a str,
        record: &'a AppliedWriteRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.applied.insert(key.to_owned(), record.clone());
            Ok(())
        })
    }

    fn get_operation<'a>(
        &'a self,
        _tenant_id: &'a str,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<OperationRecord>, StoreError>> + Send + 'a>>
    {
        Box::pin(async move { Ok(self.operations.get(operation_name).cloned()) })
    }

    fn put_operation<'a>(
        &'a mut self,
        _tenant_id: &'a str,
        operation_name: &'a str,
        record: &'a OperationRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.operations
                .insert(operation_name.to_owned(), record.clone());
            Ok(())
        })
    }

    fn next_operation_seq<'a>(
        &'a mut self,
        _tenant_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            self.operation_seq += 1;
            Ok(self.operation_seq)
        })
    }
}

struct TenantFixture;

impl ConformanceFixture for TenantFixture {
    type Provider = TenantLifecycleProvider<MemoryStore>;

    fn fresh_provider(&self) -> Self::Provider {
        TenantLifecycleProvider::new(MemoryStore::default())
    }

    fn collection(&self) -> &str {
        TENANT_COLLECTION
    }

    fn resource_payload(&self, ordinal: u32) -> Tenant {
        Tenant {
            tenant_id: format!("res-{ordinal:04}"),
            display_name: format!("Tenant {ordinal}"),
            state: TenantLifecycleState::initial(),
            isolation_posture: IsolationPosture::Pooled,
            cell_id: "cell-001".to_owned(),
            residency_zone: None,
        }
    }

    fn resource_orn(&self, name: &ResourceName) -> String {
        format!("orn:oya:tenancy:{}:{}", name.resource_id(), name)
    }

    fn tenant_account_project(&self) -> &str {
        "tenant/res-0001"
    }

    fn region_cell(&self) -> &str {
        "control-plane/default"
    }

    fn principal(&self) -> &str {
        "tenant-lifecycle-provider"
    }
}

#[tokio::test]
async fn lifecycle_provider_passes_idempotent_put() {
    check_idempotent_put(&TenantFixture).await.unwrap();
}

#[tokio::test]
async fn lifecycle_provider_passes_create_idempotency() {
    check_create_idempotency(&TenantFixture).await.unwrap();
}

#[tokio::test]
async fn lifecycle_provider_passes_read_after_write() {
    check_read_after_write(&TenantFixture).await.unwrap();
}

#[tokio::test]
async fn lifecycle_provider_passes_stable_pagination() {
    check_stable_pagination(&TenantFixture).await.unwrap();
}

#[tokio::test]
async fn lifecycle_provider_passes_async_delete_operation() {
    check_async_delete_operation(&TenantFixture).await.unwrap();
}

#[tokio::test]
async fn lifecycle_provider_passes_the_full_contract() {
    let violations = run_all_checks(&TenantFixture).await;
    assert!(violations.is_empty(), "{violations:#?}");
}

// ---------------------------------------------------------------------------
// Lifecycle-ledger behavior beyond the generic harness.
// ---------------------------------------------------------------------------

async fn provider_with_tenant(name: &ResourceName) -> TenantLifecycleProvider<MemoryStore> {
    let mut provider = TenantFixture.fresh_provider();
    provider
        .create(
            name,
            TenantFixture.resource_payload(1),
            &TenantFixture.idempotency_key(1).unwrap(),
        )
        .await
        .unwrap();
    provider
}

async fn drive_to_done(
    provider: &mut TenantLifecycleProvider<MemoryStore>,
    name: &ResourceName,
    operation: TenantLifecycleOperation,
    key_ordinal: u32,
) -> shared_resource_provider_contract_kernel::Operation {
    let key = TenantFixture.idempotency_key(key_ordinal).unwrap();
    let mut op = provider
        .apply_lifecycle(name, operation, &key)
        .await
        .unwrap();
    assert!(!op.done, "lifecycle operations start pending");
    while !op.done {
        op = provider.poll_operation(&op.name.clone()).await.unwrap();
        op.validate().unwrap();
    }
    op
}

#[tokio::test]
async fn lifecycle_happy_path_walks_the_contract_state_machine() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;

    let op = drive_to_done(&mut provider, &name, TenantLifecycleOperation::Activate, 10).await;
    assert!(matches!(op.result, Some(OperationResult::Response(_))));
    assert_eq!(
        provider.get(&name).await.unwrap().state,
        TenantLifecycleState::Active
    );

    drive_to_done(&mut provider, &name, TenantLifecycleOperation::Suspend, 11).await;
    assert_eq!(
        provider.get(&name).await.unwrap().state,
        TenantLifecycleState::Suspended
    );

    drive_to_done(&mut provider, &name, TenantLifecycleOperation::Resume, 12).await;
    assert_eq!(
        provider.get(&name).await.unwrap().state,
        TenantLifecycleState::Active
    );

    let op = drive_to_done(&mut provider, &name, TenantLifecycleOperation::Retire, 13).await;
    assert!(matches!(op.result, Some(OperationResult::Response(_))));
    assert!(matches!(
        provider.get(&name).await,
        Err(ProviderError::NotFound { .. })
    ));
}

#[tokio::test]
async fn invalid_transition_completes_as_failed_precondition_and_changes_nothing() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;

    // Suspend from Provisioning is not a legal contract transition.
    let op = drive_to_done(&mut provider, &name, TenantLifecycleOperation::Suspend, 20).await;
    match &op.result {
        Some(OperationResult::Error(error)) => {
            assert_eq!(error.code, "failed_precondition", "{error:?}");
        }
        other => panic!("expected a failed operation, got {other:?}"),
    }
    assert_eq!(
        provider.get(&name).await.unwrap().state,
        TenantLifecycleState::initial(),
        "a failed transition must not move state"
    );
}

#[tokio::test]
async fn terminal_failed_operations_are_immutable_and_replayable() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;
    let key = TenantFixture.idempotency_key(21).unwrap();

    let pending = provider
        .apply_lifecycle(&name, TenantLifecycleOperation::Suspend, &key)
        .await
        .unwrap();
    let failed = provider.poll_operation(&pending.name).await.unwrap();
    assert!(failed.done);

    // Replay under the same key returns the SAME terminal operation.
    let replay = provider
        .apply_lifecycle(&name, TenantLifecycleOperation::Suspend, &key)
        .await
        .unwrap();
    assert_eq!(replay, failed);

    // Re-polling a terminal entry never rewrites it — even after the tenant
    // becomes legally suspendable.
    drive_to_done(&mut provider, &name, TenantLifecycleOperation::Activate, 22).await;
    let repoll = provider.poll_operation(&pending.name).await.unwrap();
    assert_eq!(repoll, failed, "terminal operations are immutable");
}

#[tokio::test]
async fn idempotency_key_reuse_across_different_lifecycle_params_is_rejected() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;
    let key = TenantFixture.idempotency_key(23).unwrap();

    provider
        .apply_lifecycle(&name, TenantLifecycleOperation::Activate, &key)
        .await
        .unwrap();
    assert!(matches!(
        provider
            .apply_lifecycle(&name, TenantLifecycleOperation::Suspend, &key)
            .await,
        Err(ProviderError::IdempotencyKeyReuse { .. })
    ));
}

#[tokio::test]
async fn lifecycle_on_unknown_tenant_is_not_found() {
    let mut provider = TenantFixture.fresh_provider();
    let name = TenantFixture.resource_name(7).unwrap();
    assert!(matches!(
        provider
            .apply_lifecycle(
                &name,
                TenantLifecycleOperation::Activate,
                &TenantFixture.idempotency_key(30).unwrap(),
            )
            .await,
        Err(ProviderError::NotFound { .. })
    ));
}

#[tokio::test]
async fn create_rejects_non_initial_state_and_invalid_tenants() {
    let mut provider = TenantFixture.fresh_provider();
    let name = TenantFixture.resource_name(1).unwrap();

    let active = Tenant {
        state: TenantLifecycleState::Active,
        ..TenantFixture.resource_payload(1)
    };
    assert!(matches!(
        provider
            .create(&name, active, &TenantFixture.idempotency_key(40).unwrap())
            .await,
        Err(ProviderError::InvalidArgument { .. })
    ));

    let malformed = Tenant {
        tenant_id: "Not A Slug".to_owned(),
        ..TenantFixture.resource_payload(1)
    };
    assert!(matches!(
        provider
            .create(
                &name,
                malformed,
                &TenantFixture.idempotency_key(41).unwrap()
            )
            .await,
        Err(ProviderError::InvalidArgument { .. })
    ));
}

#[tokio::test]
async fn retired_tombstones_refuse_put_and_create() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;
    drive_to_done(&mut provider, &name, TenantLifecycleOperation::Retire, 60).await;
    assert!(matches!(
        provider.get(&name).await,
        Err(ProviderError::NotFound { .. })
    ));

    assert!(matches!(
        provider
            .put(
                &name,
                TenantFixture.resource_payload(1),
                &TenantFixture.idempotency_key(61).unwrap()
            )
            .await,
        Err(ProviderError::FailedPrecondition { .. })
    ));
    assert!(matches!(
        provider
            .create(
                &name,
                TenantFixture.resource_payload(1),
                &TenantFixture.idempotency_key(62).unwrap()
            )
            .await,
        Err(ProviderError::AlreadyExists { .. })
    ));
}

#[tokio::test]
async fn put_may_never_change_lifecycle_state() {
    let name = TenantFixture.resource_name(1).unwrap();
    let mut provider = provider_with_tenant(&name).await;

    let sneaky = Tenant {
        state: TenantLifecycleState::Active,
        ..TenantFixture.resource_payload(1)
    };
    assert!(matches!(
        provider
            .put(&name, sneaky, &TenantFixture.idempotency_key(50).unwrap())
            .await,
        Err(ProviderError::FailedPrecondition { .. })
    ));
    assert_eq!(
        provider.get(&name).await.unwrap().state,
        TenantLifecycleState::initial()
    );
}
