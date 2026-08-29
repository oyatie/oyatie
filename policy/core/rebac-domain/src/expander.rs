//! The bounded walk over a userset rewrite.
//!
//! This is what makes a `UsersetRewrite` mean something. `check` answers
//! whether a subject holds a relation on an object by evaluating that
//! object type's rewrite against tuples read at one pinned snapshot, so every
//! read in a single decision sees the same state.

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTuple, RebacTupleQuery, RebacTupleStore, UsersetRewrite,
};

use crate::bounds::{Budget, ExpansionBounds};
use crate::error::ExpansionError;
use crate::namespace::NamespaceConfig;
use crate::walk::Walk;

/// Evaluates relationship questions for one tenant at one snapshot.
pub struct Expander<'a, S: RebacTupleStore> {
    store: &'a S,
    namespace: &'a NamespaceConfig,
    tenant: RebacTenantScope,
    snapshot: RebacReadSnapshot,
    bounds: ExpansionBounds,
}

impl<'a, S: RebacTupleStore> Expander<'a, S> {
    #[must_use]
    pub fn new(
        store: &'a S,
        namespace: &'a NamespaceConfig,
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

    /// Does `subject` hold `relation` on `object` at this snapshot?
    ///
    /// # Errors
    /// Every [`ExpansionError`] is a denial. A caller must not read `Ok(false)`
    /// and an error as the same outcome: the first says the graph was walked
    /// and no grant exists, the second says the graph was not fully walked.
    pub fn check(
        &self,
        subject: &RebacSubjectRef,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        let mut walk = Walk::new(subject, self.bounds.max_depth, self.bounds.max_tuples_read);
        self.resolve(&mut walk, relation, object)
    }

    /// Resolve `object#relation` for the walk's subject.
    fn resolve(
        &self,
        walk: &mut Walk<'_>,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        // A relation already on this path contributes no new grant. Answering
        // false rather than refusing keeps a legitimately cyclic graph (groups
        // that contain each other) usable instead of unanswerable.
        if !walk.enter(object, relation) {
            return Ok(false);
        }
        let rewrite = self.namespace.rewrite(object.object_type(), relation)?;
        let held = self.eval(walk, rewrite, relation, object);
        walk.leave(object, relation);
        held
    }

    /// Descend into `object#relation`, honouring the depth bound.
    fn descend_into(
        &self,
        walk: &mut Walk<'_>,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        walk.descend()?;
        let held = self.resolve(walk, relation, object);
        walk.ascend();
        held
    }

    fn eval(
        &self,
        walk: &mut Walk<'_>,
        rewrite: &UsersetRewrite,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        match rewrite {
            UsersetRewrite::This => self.direct(walk, relation, object),
            UsersetRewrite::ComputedUserset { relation: computed } => {
                self.descend_into(walk, computed, object)
            }
            UsersetRewrite::TupleToUserset {
                tupleset_relation,
                computed_userset_relation,
            } => self.tuple_to_userset(walk, tupleset_relation, computed_userset_relation, object),
            UsersetRewrite::Union { children } => {
                for child in children {
                    if self.eval(walk, child, relation, object)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            UsersetRewrite::Intersection { children } => {
                for child in children {
                    if !self.eval(walk, child, relation, object)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            UsersetRewrite::Difference { base, subtract } => {
                if !self.eval(walk, base, relation, object)? {
                    return Ok(false);
                }
                let excluded = self.eval(walk, subtract, relation, object)?;
                Ok(!excluded)
            }
        }
    }

    /// `This`: tuples written directly against `object#relation`. A tuple
    /// whose subject is a userset expands in turn.
    fn direct(
        &self,
        walk: &mut Walk<'_>,
        relation: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        for tuple in self.read_tupleset(object, relation, &mut walk.budget)? {
            match &tuple.subject {
                candidate if candidate == walk.subject => return Ok(true),
                RebacSubjectRef::Userset {
                    object: via,
                    relation: via_relation,
                } => {
                    if self.descend_into(walk, via_relation, via)? {
                        return Ok(true);
                    }
                }
                RebacSubjectRef::Object { .. } => {}
            }
        }
        Ok(false)
    }

    /// `TupleToUserset`: walk `object#tupleset` to its objects, then ask each
    /// for `computed`. This is what makes `document#viewer` follow
    /// `document#parent` into `folder#viewer`.
    fn tuple_to_userset(
        &self,
        walk: &mut Walk<'_>,
        tupleset_relation: &RebacRelation,
        computed: &RebacRelation,
        object: &RebacObjectRef,
    ) -> Result<bool, ExpansionError> {
        for tuple in self.read_tupleset(object, tupleset_relation, &mut walk.budget)? {
            let via = match &tuple.subject {
                RebacSubjectRef::Object { object } => object,
                RebacSubjectRef::Userset { object, .. } => object,
            };
            if self.descend_into(walk, computed, via)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Reads every page of `object#relation`. A reader that stopped at the
    /// first page would return a wrong answer, not a partial one.
    fn read_tupleset(
        &self,
        object: &RebacObjectRef,
        relation: &RebacRelation,
        budget: &mut Budget,
    ) -> Result<Vec<RebacTuple>, ExpansionError> {
        let mut collected = Vec::new();
        let mut page_token = None;
        for _ in 0..self.bounds.max_pages_per_tupleset {
            let query = RebacTupleQuery::object_relation(
                self.tenant.clone(),
                object.clone(),
                relation.clone(),
            )
            .at_page(page_token);
            let page = self.store.read_tuples(&query, self.snapshot.clone())?;
            budget.charge(page.tuples.len())?;
            collected.extend(page.tuples);
            match page.next_page_token {
                Some(token) => page_token = Some(token),
                None => return Ok(collected),
            }
        }
        Err(ExpansionError::PageBudgetExceeded {
            limit: self.bounds.max_pages_per_tupleset,
        })
    }
}
