//! Reusable standalone relationship checks.

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTupleStore, RebacTupleStoreError,
};

use crate::bounds::ExpansionBounds;
use crate::error::ExpansionError;
use crate::namespace::ValidatedNamespace;
use crate::session::ExpansionSession;

/// Evaluates independent relationship questions for one tenant.
///
/// Every standalone check resolves its requested snapshot and owns a fresh
/// budget. Callers combining multiple checks into one authorization decision
/// use [`ExpansionSession`] instead.
pub struct Expander<'a, S: RebacTupleStore> {
    store: &'a S,
    namespace: &'a ValidatedNamespace,
    tenant: RebacTenantScope,
    snapshot: RebacReadSnapshot,
    bounds: ExpansionBounds,
}

impl<'a, S: RebacTupleStore> Expander<'a, S> {
    #[must_use]
    pub fn new(
        store: &'a S,
        namespace: &'a ValidatedNamespace,
        tenant: RebacTenantScope,
        snapshot: RebacReadSnapshot,
    ) -> Self {
        Self {
            store,
            namespace,
            tenant,
            snapshot,
            bounds: ExpansionBounds::DEFAULT,
        }
    }

    #[must_use]
    pub fn with_bounds(mut self, bounds: ExpansionBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Does `subject` hold `relation` on `object` at a freshly resolved
    /// instance of the configured snapshot request?
    ///
    /// # Errors
    /// Every [`ExpansionError`] is a refusal. `Ok(false)` alone means the
    /// complete graph contains no grant.
    pub fn check(
        &self,
        subject: &RebacSubjectRef,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        let snapshot = self
            .store
            .resolve_snapshot(&self.tenant, self.snapshot.clone())?;
        if snapshot.tenant() != &self.tenant {
            return Err(RebacTupleStoreError::SnapshotScopeMismatch {
                query_tenant: self.tenant.clone(),
                snapshot_tenant: snapshot.tenant().clone(),
            }
            .into());
        }
        ExpansionSession::new(self.store, self.namespace, snapshot, self.bounds)
            .check(subject, relation, object)
    }
}
