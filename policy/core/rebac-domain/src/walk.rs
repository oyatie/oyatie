//! State carried through one expansion.
//!
//! Depth, the cycle-detection path and the tuple budget all belong to a single
//! walk rather than to the expander, which is shared and immutable. Bundling
//! them keeps the recursive signatures readable and makes it impossible to
//! recurse while forgetting to charge the budget or record the path.

use std::collections::BTreeSet;

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, SnapshotToken,
};

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
    /// The same pairs in visit order, so a re-entry can be placed relative to
    /// where each enclosing subtraction began.
    order: Vec<(String, String)>,
    pub(crate) budget: Budget,
    /// The snapshot every read after the first is served at.
    ///
    /// A decision must see one state. Passing the caller's `Latest` to each
    /// read re-resolves it to whatever head is at that moment, so a write
    /// landing mid-walk becomes visible to later reads but not earlier ones —
    /// the new-enemy problem, and an answer about a tree that never existed.
    /// The first read reports the snapshot it was served at; every subsequent
    /// read pins to it. If the store later cannot serve that pin it returns
    /// `StaleSnapshot`, which is an `ExpansionError`, so the walk fails closed
    /// rather than silently drifting.
    pinned: Option<SnapshotToken>,
    /// Where each enclosing subtraction began.
    ///
    /// Re-entry returns "not a member", which is sound while every enclosing
    /// operator is monotone. Under a subtraction it is not: the re-entry reads
    /// as "not excluded" and grants. The model-time stratifier catches cycles
    /// the MODEL declares; a tuple whose subject is a userset can close the
    /// same cycle in data, where no static check can see it.
    /// `order.len()` when each enclosing subtraction was entered.
    ///
    /// Re-entry is only unsound when the cycle CROSSES the subtraction. A
    /// monotone cycle sitting wholly inside the subtracted set - two groups
    /// that contain each other, named by a blocklist - is a legitimate shape,
    /// and refusing it turns two tenant-writable tuples into a permanently
    /// failing check.
    marks: Vec<usize>,
}

impl<'a> Walk<'a> {
    pub(crate) fn new(subject: &'a RebacSubjectRef, max_depth: u32, tuple_budget: usize) -> Self {
        Self {
            subject,
            depth: 0,
            max_depth,
            path: BTreeSet::new(),
            order: Vec::new(),
            budget: Budget::new(tuple_budget),
            pinned: None,
            marks: Vec::new(),
        }
    }

    /// The snapshot to read at: the pin once taken, else the caller's request.
    pub(crate) fn read_at(&self, requested: &RebacReadSnapshot) -> RebacReadSnapshot {
        match &self.pinned {
            Some(token) => RebacReadSnapshot::at(token.clone()),
            None => requested.clone(),
        }
    }

    /// Record the snapshot the first read was served at. Later calls are
    /// ignored: the pin is taken once and holds for the whole decision.
    pub(crate) fn pin(&mut self, served: SnapshotToken) {
        if self.pinned.is_none() {
            self.pinned = Some(served);
        }
    }

    /// Enter the subtracted side of a `Difference`.
    pub(crate) fn enter_negation(&mut self) {
        self.marks.push(self.order.len());
    }

    pub(crate) fn leave_negation(&mut self) {
        self.marks.pop();
    }

    /// Does re-entering `object#relation` close a cycle that CROSSES the
    /// innermost enclosing subtraction?
    ///
    /// True only when the node was already on the path before that
    /// subtraction was entered: the cycle then runs through the negated edge,
    /// where "not a member" inverts to "not excluded" and grants. A node first
    /// visited inside the subtracted set is an ordinary monotone cycle and
    /// contributes nothing, as it would anywhere else.
    pub(crate) fn crosses_negation(
        &self,
        object: &RebacObjectRef,
        relation: &RebacRelation,
    ) -> bool {
        let Some(&mark) = self.marks.last() else {
            return false;
        };
        let key = (object.to_canonical_string(), relation.as_str().to_owned());
        self.order
            .iter()
            .position(|entry| *entry == key)
            .is_some_and(|position| position < mark)
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
        let key = (object.to_canonical_string(), relation.as_str().to_owned());
        if self.path.insert(key.clone()) {
            self.order.push(key);
            return true;
        }
        false
    }

    pub(crate) fn leave(&mut self, object: &RebacObjectRef, relation: &RebacRelation) {
        let key = (object.to_canonical_string(), relation.as_str().to_owned());
        self.path.remove(&key);
        // `resolve` calls `leave` only on the frame whose `enter` returned
        // true, and takes no `?` between the two, so this frame is the last
        // one pushed. Popping unconditionally keeps `order` and `path` the
        // same set; the assertion states the invariant that makes reading
        // `order` by position meaningful.
        debug_assert_eq!(
            self.order.last(),
            Some(&key),
            "leave must unwind the frame enter pushed"
        );
        self.order.pop();
    }
}
