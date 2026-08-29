//! State carried through one expansion.
//!
//! Depth, the cycle-detection path and the tuple budget all belong to a single
//! walk rather than to the expander, which is shared and immutable. Bundling
//! them keeps the recursive signatures readable and makes it impossible to
//! recurse while forgetting to charge the budget or record the path.

use std::collections::BTreeSet;

use policy_cedar_domain::rebac::{RebacObjectRef, RebacRelation, RebacSubjectRef};

use crate::bounds::Budget;
use crate::error::ExpansionError;

/// One in-progress expansion, for one subject.
pub(crate) struct Walk<'a> {
    /// The subject the whole walk is asking about; never changes.
    pub(crate) subject: &'a RebacSubjectRef,
    depth: u32,
    max_depth: u32,
    /// `(object, relation)` pairs on the current path, for cycle detection.
    path: BTreeSet<(String, String)>,
    pub(crate) budget: Budget,
}

impl<'a> Walk<'a> {
    pub(crate) fn new(subject: &'a RebacSubjectRef, max_depth: u32, tuple_budget: usize) -> Self {
        Self {
            subject,
            depth: 0,
            max_depth,
            path: BTreeSet::new(),
            budget: Budget::new(tuple_budget),
        }
    }

    /// Descend one level, refusing past the depth bound.
    pub(crate) fn descend(&mut self) -> Result<(), ExpansionError> {
        self.depth = self.depth.saturating_add(1);
        if self.depth > self.max_depth {
            return Err(ExpansionError::DepthExceeded {
                limit: self.max_depth,
            });
        }
        Ok(())
    }

    pub(crate) fn ascend(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Record `object#relation` on the current path. `false` means it is
    /// already there — a cycle, which contributes no new grant.
    pub(crate) fn enter(&mut self, object: &RebacObjectRef, relation: &RebacRelation) -> bool {
        self.path
            .insert((object.to_canonical_string(), relation.as_str().to_owned()))
    }

    pub(crate) fn leave(&mut self, object: &RebacObjectRef, relation: &RebacRelation) {
        self.path
            .remove(&(object.to_canonical_string(), relation.as_str().to_owned()));
    }
}
