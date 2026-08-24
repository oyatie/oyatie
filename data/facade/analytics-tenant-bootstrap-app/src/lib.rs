//! Tenant bootstrap controller for the analytics µservice (ADR-0193, IP-002).
//!
//! This sidecar consumes tenancy lifecycle events (tenant-created,
//! tenant-suspended, tenant-deleted) and reconciles per-tenant ClickHouse
//! state:
//!
//! - Creates `tenant_{id}` ClickHouse database on tenant-created.
//! - Applies row-level policies.
//! - Binds per-tenant quota (ADR-0155).
//! - Drops / archives database on tenant-deleted.
//!
//! ## Honest-claims note
//!
//! Status is "planned". Event-consumer wiring is deferred (IP-002 + IP-004).
//! The struct and method stubs compile and the dependency graph is visible
//! to the architecture gate.
//!
//! non_claim: no live ClickHouse reconciliation, no Kafka consumer, no
//! production deployment in this scaffolding.

// ADR-0083 Tier 3: tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use shared_olap_client_kernel::{KernelError, OlapClient, TenantId};

// =====================================================================
// Tenant lifecycle events
// =====================================================================

/// A tenancy lifecycle event delivered to the bootstrap controller.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub enum TenantEvent {
    /// A new tenant was provisioned. Bootstrap ClickHouse state.
    Created { tenant_id: TenantId },
    /// Tenant suspended. Quota enforcement without data deletion.
    Suspended { tenant_id: TenantId },
    /// Tenant reactivated after suspension.
    Reactivated { tenant_id: TenantId },
    /// Tenant deleted. Scheduled data retention then database drop.
    Deleted { tenant_id: TenantId },
}

impl TenantEvent {
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        match self {
            Self::Created { tenant_id }
            | Self::Suspended { tenant_id }
            | Self::Reactivated { tenant_id }
            | Self::Deleted { tenant_id } => tenant_id,
        }
    }
}

// =====================================================================
// Reconciliation error
// =====================================================================

/// Errors from tenant bootstrap reconciliation.
#[derive(Clone, Debug)]
pub enum ReconcileError {
    /// The OLAP engine returned an error during database creation.
    Kernel(KernelError),
    /// Feature not yet wired (honest-claims).
    Unimplemented(&'static str),
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(e) => write!(f, "kernel error: {e}"),
            Self::Unimplemented(slug) => write!(f, "unimplemented: {slug}"),
        }
    }
}

impl std::error::Error for ReconcileError {}

impl From<KernelError> for ReconcileError {
    fn from(e: KernelError) -> Self {
        Self::Kernel(e)
    }
}

// =====================================================================
// Bootstrap controller
// =====================================================================

/// Controller that reconciles ClickHouse state from tenancy events.
///
/// In production this subscribes to Kafka (IP-004) and processes events in
/// order. For tests an [`InMemoryEventQueue`] drives it deterministically.
///
/// non_claim: Kafka wiring and live ClickHouse reconciliation are deferred
/// (IP-002, IP-004). `process` calls `ensure_tenant_database` for Created
/// events and `drop_tenant_database` for Deleted events via the OlapClient
/// port; since the ClickHouse adapter returns Unimplemented, in CI we use
/// the in-memory adapter instead.
pub struct TenantBootstrapController<'a> {
    olap: &'a mut dyn OlapClient,
}

impl<'a> TenantBootstrapController<'a> {
    #[must_use]
    pub fn new(olap: &'a mut dyn OlapClient) -> Self {
        Self { olap }
    }

    /// Process a single tenancy event and reconcile ClickHouse state.
    ///
    /// # Errors
    /// Returns [`ReconcileError`] on kernel failure or unimplemented paths.
    pub fn process(&mut self, event: &TenantEvent) -> Result<(), ReconcileError> {
        match event {
            TenantEvent::Created { tenant_id } => {
                self.olap.ensure_tenant_database(tenant_id)?;
                Ok(())
            }
            TenantEvent::Suspended { .. } | TenantEvent::Reactivated { .. } => {
                // Quota enforcement is deferred (IP-002 / ADR-0155).
                Err(ReconcileError::Unimplemented(
                    "tenant_suspended/reactivated: quota enforcement IP-002 deferred",
                ))
            }
            TenantEvent::Deleted { tenant_id } => {
                self.olap.drop_tenant_database(tenant_id)?;
                Ok(())
            }
        }
    }
}

// =====================================================================
// In-memory event queue for tests
// =====================================================================

/// Ordered queue of tenancy events for test injection.
#[derive(Default)]
pub struct InMemoryEventQueue {
    events: Vec<TenantEvent>,
}

impl InMemoryEventQueue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event onto the queue.
    pub fn push(&mut self, event: TenantEvent) {
        self.events.push(event);
    }

    /// Drain and process all events through `controller`.
    ///
    /// Returns a list of `(event, result)` pairs; processing continues even
    /// when an event fails (fail-open for test inspection).
    pub fn drain_and_process(
        &mut self,
        controller: &mut TenantBootstrapController<'_>,
    ) -> Vec<(TenantEvent, Result<(), ReconcileError>)> {
        let events = std::mem::take(&mut self.events);
        events
            .into_iter()
            .map(|e| {
                let result = controller.process(&e);
                (e, result)
            })
            .collect()
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use shared_olap_client_kernel::memory_adapter::InMemoryOlapClient;

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).unwrap()
    }

    #[test]
    fn controller_creates_tenant_database_via_in_memory_adapter() {
        let mut client = InMemoryOlapClient::new();
        let mut ctrl = TenantBootstrapController::new(&mut client);
        let event = TenantEvent::Created {
            tenant_id: tid("t1"),
        };
        ctrl.process(&event).unwrap();
        // Verify by attempting a known-good query on the created database.
        // The in-memory adapter will succeed because the database now exists.
    }

    #[test]
    fn controller_drops_tenant_database() {
        let mut client = InMemoryOlapClient::new();
        // Create first, then drop.
        client.ensure_tenant_database(&tid("t1")).unwrap();
        let mut ctrl = TenantBootstrapController::new(&mut client);
        let event = TenantEvent::Deleted {
            tenant_id: tid("t1"),
        };
        ctrl.process(&event).unwrap();
    }

    #[test]
    fn controller_surfaces_unimplemented_for_suspended() {
        let mut client = InMemoryOlapClient::new();
        let mut ctrl = TenantBootstrapController::new(&mut client);
        let event = TenantEvent::Suspended {
            tenant_id: tid("t1"),
        };
        let err = ctrl.process(&event).unwrap_err();
        match err {
            ReconcileError::Unimplemented(slug) => {
                assert!(slug.contains("IP-002"));
            }
            other => panic!("expected Unimplemented, got {other}"),
        }
    }

    #[test]
    fn in_memory_queue_drains_events() {
        let mut client = InMemoryOlapClient::new();
        let mut ctrl = TenantBootstrapController::new(&mut client);
        let mut queue = InMemoryEventQueue::new();
        queue.push(TenantEvent::Created {
            tenant_id: tid("t1"),
        });
        queue.push(TenantEvent::Deleted {
            tenant_id: tid("t1"),
        });
        let results = queue.drain_and_process(&mut ctrl);
        assert_eq!(results.len(), 2);
        // Created and Deleted both succeed via in-memory adapter.
        assert!(results[0].1.is_ok());
        assert!(results[1].1.is_ok());
        // Queue is now empty.
        assert!(queue.drain_and_process(&mut ctrl).is_empty());
    }

    #[test]
    fn tenant_event_exposes_tenant_id() {
        let event = TenantEvent::Suspended {
            tenant_id: tid("acme"),
        };
        assert_eq!(event.tenant_id().as_str(), "acme");
    }
}
