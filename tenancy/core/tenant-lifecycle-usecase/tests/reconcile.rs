//! Reconcile-pass behavior: convergence, restart-replay, metadata drift,
//! unreachable specs, and store-failure injection.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_shared_platform_contracts_kernel::tenancy::{
    IsolationPosture, Tenant, TenantLifecycleState,
};
use oya_shared_resource_provider_contract_kernel::{
    ProviderError, ResourceName, ResourceProvider,
};
use tenancy_tenant_lifecycle_domain::DesiredTenantState;
use tenancy_tenant_lifecycle_kernel::{
    AppliedWriteRecord, OperationRecord, StoreError, TenantLifecycleStore,
};
use tenancy_tenant_lifecycle_usecase::TenantLifecycleProvider;
use tenancy_tenant_lifecycle_usecase::reconcile::{
    ReconcileContext, ReconcileOutcome, TenantSpec,
};

// The same in-memory port fixture as tests/conformance.rs (duplicated by
// design: integration-test binaries are independent compilation units).
#[derive(Debug, Default)]
struct MemoryStore {
    tenants: BTreeMap<String, Tenant>,
    applied: BTreeMap<String, AppliedWriteRecord>,
    operations: BTreeMap<String, OperationRecord>,
    operation_seq: u64,
    /// When set, the next put_tenant fails with Unavailable (then clears).
    fail_next_put: bool,
}

impl TenantLifecycleStore for MemoryStore {
    fn get_tenant(&self, name: &str) -> Result<Option<Tenant>, StoreError> {
        Ok(self.tenants.get(name).cloned())
    }

    fn put_tenant(&mut self, name: &str, tenant: &Tenant) -> Result<(), StoreError> {
        if self.fail_next_put {
            self.fail_next_put = false;
            return Err(StoreError::Unavailable {
                detail: "injected put failure".to_owned(),
            });
        }
        self.tenants.insert(name.to_owned(), tenant.clone());
        Ok(())
    }

    fn remove_tenant(&mut self, name: &str) -> Result<(), StoreError> {
        self.tenants.remove(name);
        Ok(())
    }

    fn scan_tenants(
        &self,
        prefix: &str,
        start_at: Option<&str>,
        limit: u32,
    ) -> Result<Vec<(String, Tenant)>, StoreError> {
        Ok(self
            .tenants
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .filter(|(key, _)| start_at.is_none_or(|start| key.as_str() >= start))
            .take(limit as usize)
            .map(|(key, tenant)| (key.clone(), tenant.clone()))
            .collect())
    }

    fn get_applied(&self, key: &str) -> Result<Option<AppliedWriteRecord>, StoreError> {
        Ok(self.applied.get(key).cloned())
    }

    fn put_applied(&mut self, key: &str, record: &AppliedWriteRecord) -> Result<(), StoreError> {
        self.applied.insert(key.to_owned(), record.clone());
        Ok(())
    }

    fn get_operation(&self, operation_name: &str) -> Result<Option<OperationRecord>, StoreError> {
        Ok(self.operations.get(operation_name).cloned())
    }

    fn put_operation(
        &mut self,
        operation_name: &str,
        record: &OperationRecord,
    ) -> Result<(), StoreError> {
        self.operations
            .insert(operation_name.to_owned(), record.clone());
        Ok(())
    }

    fn next_operation_seq(&mut self) -> Result<u64, StoreError> {
        self.operation_seq += 1;
        Ok(self.operation_seq)
    }
}

fn name() -> ResourceName {
    ResourceName::new("tenants", "acme").unwrap()
}

fn spec(desired: DesiredTenantState) -> TenantSpec {
    TenantSpec {
        display_name: "Acme Corp".to_owned(),
        isolation_posture: IsolationPosture::Pooled,
        cell_id: "cell-001".to_owned(),
        residency_zone: Some("kr-seoul".to_owned()),
        desired,
    }
}

const CTX: ReconcileContext<'_> = ReconcileContext {
    cr_uid: "8f1d2c3a-uid",
    generation: 1,
};

/// Drive reconcile passes until a non-Progressing outcome, bounding passes.
fn reconcile_until_settled(
    provider: &mut TenantLifecycleProvider<MemoryStore>,
    spec: &TenantSpec,
    ctx: ReconcileContext<'_>,
) -> (ReconcileOutcome, u32) {
    let mut passes = 0;
    loop {
        passes += 1;
        assert!(passes <= 8, "reconcile did not settle within 8 passes");
        match provider.reconcile(&name(), spec, ctx).unwrap() {
            ReconcileOutcome::Progressing { .. } => {}
            settled => return (settled, passes),
        }
    }
}

#[test]
fn converges_from_nothing_to_active() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    let (outcome, passes) =
        reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Active), CTX);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Active)
        }
    );
    assert_eq!(passes, 3, "create, activate, observe-converged");
    assert_eq!(provider.get(&name()).unwrap().state, TenantLifecycleState::Active);
}

#[test]
fn converges_from_nothing_to_suspended_via_activate() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    let (outcome, _) =
        reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Suspended), CTX);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Suspended)
        }
    );
}

#[test]
fn converges_to_retired_as_a_visible_tombstone() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Active), CTX);

    let (outcome, _) =
        reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Retired), CTX);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Retired)
        },
        "the reconciler sees the tombstone"
    );
    // ...but the public read surface does not.
    assert!(matches!(
        provider.get(&name()),
        Err(ProviderError::NotFound { .. })
    ));

    // Re-reconciling the retired spec stays converged forever.
    let again = provider
        .reconcile(&name(), &spec(DesiredTenantState::Retired), CTX)
        .unwrap();
    assert_eq!(
        again,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Retired)
        }
    );

    // A never-existed tenant with a retired spec converges as absent.
    let mut fresh = TenantLifecycleProvider::new(MemoryStore::default());
    let absent = fresh
        .reconcile(&name(), &spec(DesiredTenantState::Retired), CTX)
        .unwrap();
    assert_eq!(absent, ReconcileOutcome::Converged { observed: None });
}

#[test]
fn restart_replay_does_not_duplicate_work() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    let target = spec(DesiredTenantState::Active);
    let (_, first_passes) = reconcile_until_settled(&mut provider, &target, CTX);
    assert_eq!(first_passes, 3);

    // A controller restart re-runs the same generation from scratch: every
    // step key rederives identically, so replays are no-ops and the state
    // settles immediately without re-walking the FSM.
    let (outcome, replay_passes) = reconcile_until_settled(&mut provider, &target, CTX);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Active)
        }
    );
    assert_eq!(replay_passes, 1, "converged state needs exactly one pass");
}

#[test]
fn metadata_drift_is_reconciled_through_idempotent_put() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Active), CTX);

    let renamed = TenantSpec {
        display_name: "Acme Corporation".to_owned(),
        ..spec(DesiredTenantState::Active)
    };
    let bumped = ReconcileContext {
        cr_uid: CTX.cr_uid,
        generation: 2,
    };
    let (outcome, _) = reconcile_until_settled(&mut provider, &renamed, bumped);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Active)
        }
    );
    assert_eq!(provider.get(&name()).unwrap().display_name, "Acme Corporation");
    assert_eq!(
        provider.get(&name()).unwrap().state,
        TenantLifecycleState::Active,
        "metadata reconciliation must never move lifecycle state"
    );
}

#[test]
fn retired_id_is_never_reused_end_to_end() {
    let mut provider = TenantLifecycleProvider::new(MemoryStore::default());
    reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Active), CTX);
    reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Retired), CTX);

    // A spec asking the retired tenant to be Active again is terminally
    // Blocked — the reconciler never re-creates over a tombstone.
    let bumped = ReconcileContext {
        cr_uid: CTX.cr_uid,
        generation: 2,
    };
    let blocked = provider
        .reconcile(&name(), &spec(DesiredTenantState::Active), bumped)
        .unwrap();
    assert!(
        matches!(blocked, ReconcileOutcome::Blocked { ref reason } if reason.contains("unreachable")),
        "{blocked:?}"
    );

    // Direct API attempts are equally fail-closed.
    let key = tenancy_tenant_lifecycle_domain::derive_step_key("other", 9, "create").unwrap();
    assert!(matches!(
        provider.create(
            &name(),
            Tenant {
                tenant_id: "acme".to_owned(),
                display_name: "Acme Reborn".to_owned(),
                state: TenantLifecycleState::initial(),
                isolation_posture: IsolationPosture::Pooled,
                cell_id: "cell-001".to_owned(),
                residency_zone: None,
            },
            &key,
        ),
        Err(ProviderError::AlreadyExists { .. })
    ));
}

#[test]
fn injected_store_failure_surfaces_and_retry_recovers() {
    let mut store = MemoryStore::default();
    store.fail_next_put = true;
    let mut provider = TenantLifecycleProvider::new(store);

    // First pass: create hits the injected put failure and surfaces it
    // (fail closed — no partial state, no swallowed error).
    let error = provider
        .reconcile(&name(), &spec(DesiredTenantState::Active), CTX)
        .unwrap_err();
    assert!(matches!(error, ProviderError::Internal { .. }), "{error}");
    assert!(matches!(
        provider.get(&name()),
        Err(ProviderError::NotFound { .. })
    ));

    // Retry (next reconcile pass) recovers and converges normally.
    let (outcome, _) =
        reconcile_until_settled(&mut provider, &spec(DesiredTenantState::Active), CTX);
    assert_eq!(
        outcome,
        ReconcileOutcome::Converged {
            observed: Some(TenantLifecycleState::Active)
        }
    );
}
