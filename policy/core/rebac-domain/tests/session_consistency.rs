//! Decision-scoped expansion properties owned by the ReBAC domain.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::cell::{Cell, RefCell};

use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTenantScope,
    RebacTuple, RebacTuplePage, RebacTupleQuery, RebacTupleStore, RebacTupleStoreError,
    ResolvedRebacSnapshot, SnapshotToken, UsersetRewrite, Zookie,
};
use policy_rebac_domain::{
    ExpansionBounds, ExpansionError, ExpansionSession, NamespaceConfig, ValidatedNamespace,
};

struct RecordingStore {
    tuples: Vec<RebacTuple>,
    resolutions: Cell<usize>,
    reads: RefCell<Vec<ResolvedRebacSnapshot>>,
}

impl RecordingStore {
    fn with_tuple(tuple: RebacTuple) -> Self {
        Self {
            tuples: vec![tuple],
            resolutions: Cell::new(0),
            reads: RefCell::new(Vec::new()),
        }
    }
}

impl RebacTupleStore for RecordingStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "session test store is read-only".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        self.resolutions.set(self.resolutions.get() + 1);
        Ok(resolved_snapshot(tenant, "unexpected-resolution"))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        self.reads.borrow_mut().push(snapshot.clone());
        Ok(RebacTuplePage {
            tuples: self
                .tuples
                .iter()
                .filter(|tuple| query.matches(tuple))
                .cloned()
                .collect(),
            snapshot: snapshot.clone(),
            next_page_token: None,
        })
    }
}

fn tenant() -> RebacTenantScope {
    RebacTenantScope::new("tenant-a").expect("tenant is valid")
}

fn relation() -> RebacRelation {
    RebacRelation::new("viewer").expect("relation is valid")
}

fn document() -> RebacObjectRef {
    RebacObjectRef::new("document", "report").expect("document is valid")
}

fn subject() -> RebacSubjectRef {
    RebacSubjectRef::object(RebacObjectRef::new("user", "alice").expect("subject is valid"))
}

fn namespace(relation: &RebacRelation) -> ValidatedNamespace {
    NamespaceConfig::new()
        .define("document", relation, UsersetRewrite::This)
        .validated()
        .expect("direct relation is stratified")
}

fn resolved_snapshot(tenant: &RebacTenantScope, token: &str) -> ResolvedRebacSnapshot {
    ResolvedRebacSnapshot::new(
        tenant.clone(),
        SnapshotToken::new(token).expect("snapshot token is valid"),
    )
}

fn store() -> RecordingStore {
    RecordingStore::with_tuple(RebacTuple::new(tenant(), document(), relation(), subject()))
}

#[test]
fn session_reuses_one_snapshot_and_one_candidate_budget() {
    let tenant = tenant();
    let snapshot = resolved_snapshot(&tenant, "snapshot-7");
    let store = store();
    let relation = relation();
    let namespace = namespace(&relation);
    let bounds = ExpansionBounds {
        max_candidates: 2,
        max_tuples_read: 2,
        ..ExpansionBounds::DEFAULT
    };
    let mut session = ExpansionSession::new(&store, &namespace, snapshot.clone(), bounds);

    assert!(session.check(&subject(), &relation, &document()).unwrap());
    assert!(session.check(&subject(), &relation, &document()).unwrap());
    assert_eq!(
        session.check(&subject(), &relation, &document()),
        Err(ExpansionError::CandidateBudgetExceeded { limit: 2 })
    );
    assert_eq!(
        store.resolutions.get(),
        0,
        "a session must not resolve again"
    );
    assert_eq!(
        store.reads.borrow().as_slice(),
        &[snapshot.clone(), snapshot]
    );
}

#[test]
fn tuple_budget_accumulates_across_session_checks() {
    let tenant = tenant();
    let store = store();
    let relation = relation();
    let namespace = namespace(&relation);
    let bounds = ExpansionBounds {
        max_candidates: 2,
        max_tuples_read: 1,
        ..ExpansionBounds::DEFAULT
    };
    let mut session = ExpansionSession::new(
        &store,
        &namespace,
        resolved_snapshot(&tenant, "snapshot-11"),
        bounds,
    );

    assert!(session.check(&subject(), &relation, &document()).unwrap());
    assert_eq!(
        session.check(&subject(), &relation, &document()),
        Err(ExpansionError::TupleBudgetExceeded { limit: 1 })
    );
}
