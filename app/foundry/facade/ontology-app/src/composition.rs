use crate::authz::PolicyEnforcementPoint;
use foundry_caller_draft::CallerVerifier;
use std::collections::BTreeMap;
use tokio::sync::Mutex;

mod build;
mod injection;
mod stores;
mod tenant;

pub use crate::boot::BootError;
pub use build::{compose, compose_with_models};
pub use tenant::{SyncError, TenantState, WriteHandles, WriteHandlesRefused};

/// Everything the surface serves from. One mutex per tenant is honest: a
/// submission needs the log and the projection together, and SQLite is a
/// single-writer store.
pub struct AppState {
    pub tenants: BTreeMap<String, Mutex<TenantState>>,
    pub pep: PolicyEnforcementPoint,
    pub metrics: crate::metrics::Metrics,
    /// Held behind the PORT so a test can install a verifier of its own.
    pub verifier: Box<dyn CallerVerifier>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("tenants", &self.tenants)
            .field("pep", &self.pep)
            .field("metrics", &self.metrics)
            .finish_non_exhaustive()
    }
}

impl AppState {
    pub fn tenant_count(&self) -> usize {
        self.tenants.len()
    }
}
