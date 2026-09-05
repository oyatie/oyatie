//! A tuple page cannot substitute the tenant or token of the resolved snapshot.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTenantScope, RebacTuple, RebacTuplePage, RebacTupleQuery,
    RebacTupleStore, RebacTupleStoreError, ResolvedRebacSnapshot, SnapshotToken, Zookie,
};
use policy_decision_domain::{DecisionError, decide};
use policy_rebac_domain::ExpansionError;

use crate::decision_scope_support::MustNotDecide;
use crate::join_fixtures::{context_entities, eng_candidate, graph, model, request};

#[derive(Clone, Copy)]
enum PageSnapshotSubstitution {
    Tenant,
    Token,
}

struct PageSnapshotSubstitutionStore {
    substitution: PageSnapshotSubstitution,
    reads: AtomicUsize,
}

impl PageSnapshotSubstitutionStore {
    fn new(substitution: PageSnapshotSubstitution) -> Self {
        Self {
            substitution,
            reads: AtomicUsize::new(0),
        }
    }
}

impl RebacTupleStore for PageSnapshotSubstitutionStore {
    fn write_tuple(&mut self, _tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        Err(RebacTupleStoreError::Backend(
            "read-only adversarial store".to_owned(),
        ))
    }

    fn resolve_snapshot(
        &self,
        tenant: &RebacTenantScope,
        _requested: RebacReadSnapshot,
    ) -> Result<ResolvedRebacSnapshot, RebacTupleStoreError> {
        Ok(ResolvedRebacSnapshot::new(
            tenant.clone(),
            SnapshotToken::new("coherent-head").expect("token is valid"),
        ))
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: &ResolvedRebacSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let page_snapshot = match self.substitution {
            PageSnapshotSubstitution::Tenant => ResolvedRebacSnapshot::new(
                RebacTenantScope::new("ten_other").expect("other tenant is valid"),
                snapshot.token().clone(),
            ),
            PageSnapshotSubstitution::Token => ResolvedRebacSnapshot::new(
                snapshot.tenant().clone(),
                SnapshotToken::new("substituted-page").expect("token is valid"),
            ),
        };
        Ok(RebacTuplePage {
            tuples: vec![
                RebacTuple::parse(query.tenant.clone(), "group:eng#member@user:alice")
                    .expect("grant tuple is valid"),
            ],
            snapshot: page_snapshot,
            next_page_token: None,
        })
    }
}

#[test]
fn tuple_page_cannot_substitute_the_resolved_snapshot_tenant() {
    let store = PageSnapshotSubstitutionStore::new(PageSnapshotSubstitution::Tenant);
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    let error = match result {
        Err(DecisionError::Expansion(ExpansionError::Store(error))) => error,
        _ => panic!("page snapshot substitution must be refused"),
    };
    assert!(matches!(
        &error,
        RebacTupleStoreError::InconsistentSnapshot { requested, served }
            if requested.tenant().as_str() == "ten_join"
                && requested.as_str() == "coherent-head"
                && served.tenant().as_str() == "ten_other"
                && served.as_str() == "coherent-head"
    ));
    assert_eq!(
        error.to_string(),
        "ReBAC tuple store served tenant ten_other snapshot coherent-head for requested tenant ten_join snapshot coherent-head"
    );
    assert_eq!(store.reads.load(Ordering::SeqCst), 1);
    assert!(!engine.was_consulted());
}

#[test]
fn tuple_page_cannot_substitute_the_resolved_snapshot_token() {
    let store = PageSnapshotSubstitutionStore::new(PageSnapshotSubstitution::Token);
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let engine = MustNotDecide::new();

    let result = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );

    assert!(matches!(
        result,
        Err(DecisionError::Expansion(ExpansionError::Store(
            RebacTupleStoreError::InconsistentSnapshot { requested, served }
        ))) if requested.tenant().as_str() == "ten_join"
            && requested.as_str() == "coherent-head"
            && served.tenant().as_str() == "ten_join"
            && served.as_str() == "substituted-page"
    ));
    assert_eq!(store.reads.load(Ordering::SeqCst), 1);
    assert!(!engine.was_consulted());
}
