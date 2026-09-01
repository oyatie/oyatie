//! Decision-scoped expansion budget shared across otherwise independent walks.

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTupleStore, ResolvedRebacSnapshot,
};

use crate::bounds::Budget;
use crate::error::ExpansionError;
use crate::expander::ResolvedExpansion;
use crate::namespace::ValidatedNamespace;
use crate::walk::Walk;

/// A bounded group of checks over one expander snapshot.
///
/// Candidate and tuple charges accumulate across the group. Every check still
/// creates a fresh walk, so traversal path, depth, and negation state cannot
/// leak between candidate roots.
pub struct ExpansionSession<'store, S: RebacTupleStore> {
    evaluator: ResolvedExpansion<'store, S>,
    budget: Budget,
}

impl<'store, S: RebacTupleStore> ExpansionSession<'store, S> {
    /// Start one decision-wide group at a previously resolved store snapshot.
    #[must_use]
    pub fn new(
        store: &'store S,
        namespace: &'store ValidatedNamespace,
        snapshot: ResolvedRebacSnapshot,
        bounds: crate::ExpansionBounds,
    ) -> Self {
        let evaluator = ResolvedExpansion::new(store, namespace, snapshot, bounds);
        Self {
            budget: Budget::new(evaluator.bounds()),
            evaluator,
        }
    }

    #[must_use]
    pub fn resolved_snapshot(&self) -> &ResolvedRebacSnapshot {
        self.evaluator.resolved_snapshot()
    }

    /// Check one candidate against the session's shared budget.
    ///
    /// # Errors
    /// Refuses when the total session bounds or any individual walk invariant
    /// cannot be satisfied.
    pub fn check(
        &mut self,
        subject: &RebacSubjectRef,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        self.budget.charge_candidate()?;
        let mut walk = Walk::new(subject, self.evaluator.bounds().max_depth, &mut self.budget);
        self.evaluator.resolve(&mut walk, relation, object)
    }
}
